//! Runs an image-to-image network over a large image in tiles. Tiles bound the memory, give
//! progress to report and points to stop at. Every tile has the same size, padded with the
//! nearest edge pixels where it sticks out of the image: graphics card back ends run fixed
//! sizes best, and some exported models only take one size.

use anyhow::{ensure, Result};
use image::RgbaImage;

use super::Feedback;
use crate::resample::resize_exact;

#[derive(Debug, Clone, Copy)]
pub struct Tiling {
    /// Image pixels each tile contributes to the result
    pub tile: u32,
    /// Extra pixels on every side of a tile, so tile edges get the same context as the inside
    pub pad: u32,
    /// How many times the network enlarges
    pub scale: u32,
}

impl Tiling {
    /// Side of the square the network gets
    pub fn input_size(self) -> u32 {
        self.tile + 2 * self.pad
    }
}

/// Runs `network` on every tile. It gets RGB planes in 0..1 of `input_size`² pixels and returns
/// `scale` times larger planes. Alpha is enlarged with Lanczos.
pub fn run_tiled(
    image: &RgbaImage,
    tiling: Tiling,
    feedback: &Feedback,
    mut network: impl FnMut(Vec<f32>, u32) -> Result<Vec<f32>>,
) -> Result<RgbaImage> {
    let (width, height) = image.dimensions();
    let Tiling { tile, pad, scale } = tiling;
    let size = tiling.input_size();
    let out_size = (size * scale) as usize;
    let mut out = RgbaImage::new(width * scale, height * scale);
    let columns = width.div_ceil(tile);
    let rows = height.div_ceil(tile);
    let mut done = 0;
    for tile_y in (0..height).step_by(tile as usize) {
        for tile_x in (0..width).step_by(tile as usize) {
            feedback.check()?;
            let mut rgb = Vec::with_capacity((3 * size * size) as usize);
            for channel in 0..3 {
                for y in 0..size {
                    let sy = (tile_y as i64 + y as i64 - pad as i64).clamp(0, height as i64 - 1) as u32;
                    rgb.extend((0..size).map(|x| {
                        let sx = (tile_x as i64 + x as i64 - pad as i64).clamp(0, width as i64 - 1) as u32;
                        image.get_pixel(sx, sy)[channel] as f32 / 255.0
                    }));
                }
            }
            let result = network(rgb, size)?;
            ensure!(result.len() == 3 * out_size * out_size, "network gave {} values for a {size}² tile", result.len());
            // Copy back only the tile itself, not the padding
            let keep_w = tile.min(width - tile_x) * scale;
            let keep_h = tile.min(height - tile_y) * scale;
            for y in 0..keep_h {
                let row = (pad * scale + y) as usize * out_size;
                for x in 0..keep_w {
                    let index = row + (pad * scale + x) as usize;
                    let pixel = out.get_pixel_mut(tile_x * scale + x, tile_y * scale + y);
                    for channel in 0..3 {
                        let value = result[channel * out_size * out_size + index];
                        pixel[channel] = (value.clamp(0.0, 1.0) * 255.0).round() as u8;
                    }
                }
            }
            done += 1;
            (feedback.progress)(done as f32 / (columns * rows) as f32);
        }
    }
    copy_alpha(image, &mut out);
    Ok(out)
}

/// Gives `out` the alpha of `image`, enlarged with Lanczos when it isn't all opaque.
fn copy_alpha(image: &RgbaImage, out: &mut RgbaImage) {
    if image.pixels().all(|p| p[3] == 255) {
        out.pixels_mut().for_each(|p| p[3] = 255);
        return;
    }
    let alpha = RgbaImage::from_fn(image.width(), image.height(), |x, y| {
        let a = image.get_pixel(x, y)[3];
        image::Rgba([a, a, a, 255])
    });
    let big = resize_exact(&alpha, out.width(), out.height());
    for (pixel, a) in out.pixels_mut().zip(big.pixels()) {
        pixel[3] = a[0];
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    use super::*;

    /// Nearest neighbour enlargement, standing in for a network
    fn nearest(scale: u32) -> impl FnMut(Vec<f32>, u32) -> Result<Vec<f32>> {
        move |input, size| {
            let (size, s) = (size as usize, scale as usize);
            let out = size * s;
            let mut result = vec![0.0; 3 * out * out];
            for c in 0..3 {
                for y in 0..out {
                    for x in 0..out {
                        result[c * out * out + y * out + x] = input[c * size * size + (y / s) * size + x / s];
                    }
                }
            }
            Ok(result)
        }
    }

    #[test]
    fn tiles_cover_the_image_without_seams() {
        let image = RgbaImage::from_fn(70, 45, |x, y| image::Rgba([(x * 3) as u8, (y * 5) as u8, 7, 255]));
        let tiling = Tiling { tile: 16, pad: 4, scale: 2 };
        let big = run_tiled(&image, tiling, &Feedback::NONE, nearest(2)).unwrap();
        assert_eq!(big.dimensions(), (140, 90));
        for (x, y, pixel) in big.enumerate_pixels() {
            assert_eq!(pixel, image.get_pixel(x / 2, y / 2), "at {x},{y}");
        }
    }

    #[test]
    fn images_smaller_than_a_tile_are_padded() {
        let image = RgbaImage::from_pixel(5, 3, image::Rgba([10, 20, 30, 128]));
        let tiling = Tiling { tile: 64, pad: 8, scale: 4 };
        let big = run_tiled(&image, tiling, &Feedback::NONE, nearest(4)).unwrap();
        assert_eq!(big.dimensions(), (20, 12));
        assert!(big.pixels().all(|p| p[0] == 10 && p[2] == 30 && p[3].abs_diff(128) <= 1));
    }

    #[test]
    fn reports_progress_and_stops_when_cancelled() {
        let image = RgbaImage::new(40, 40);
        let tiling = Tiling { tile: 10, pad: 2, scale: 1 };
        let reports = Mutex::new(Vec::new());
        let progress = |f: f32| reports.lock().unwrap().push(f);
        let feedback = Feedback { cancelled: &|| false, progress: &progress };
        run_tiled(&image, tiling, &feedback, nearest(1)).unwrap();
        let reports = reports.into_inner().unwrap();
        assert_eq!(reports.len(), 16);
        assert_eq!(reports.last(), Some(&1.0));

        let runs = AtomicUsize::new(0);
        let cancelled = || runs.load(Ordering::SeqCst) >= 3;
        let feedback = Feedback { cancelled: &cancelled, progress: &|_| {} };
        let mut network = nearest(1);
        let result = run_tiled(&image, tiling, &feedback, |input, size| {
            runs.fetch_add(1, Ordering::SeqCst);
            network(input, size)
        });
        assert!(result.unwrap_err().is::<crate::ai::Cancelled>());
        assert_eq!(runs.load(Ordering::SeqCst), 3);
    }
}
