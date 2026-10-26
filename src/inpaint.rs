//! "Remove objects": the painted brush strokes mark what should go, and LaMa fills those parts
//! in from their surroundings. Each group of strokes is cut out with plenty of context around
//! it, shrunk to the network's 512×512, filled in and pasted back with soft edges.
//!
//! The model is downloaded the first time the operation is used (see `ai::models`).

use std::path::Path;
use std::sync::{Arc, Mutex};

use anyhow::{ensure, Result};
use image::{GrayImage, Luma, Rgba, RgbaImage};
use ort::value::Tensor;

use crate::ai::models::{self, ModelFile};
use crate::ai::runtime::{self, SharedSession};
use crate::ai::Feedback;
use crate::resample::resize_exact;

const LAMA: ModelFile = ModelFile {
    file: "lama_fp32.onnx",
    url: "https://huggingface.co/Carve/LaMa-ONNX/resolve/c3c0c9e468934d62e79c329e35d82dd09ff8c444/lama_fp32.onnx",
    sha256: "1faef5301d78db7dda502fe59966957ec4b79dd64e16f03ed96913c7a4eb68d6",
    bytes: 208_044_816,
    licence: "LaMa (big-lama) by Roman Suvorov et al., ONNX port by Carve: Apache-2.0",
};

/// Everything the operation downloads
pub const MODELS: &[&ModelFile] = &[&LAMA];

pub fn is_installed() -> bool {
    models::is_installed(MODELS)
}

/// Downloads the model, reporting the fraction done. Blocks for as long as it takes.
pub fn install(progress: &dyn Fn(f32)) -> Result<()> {
    models::install(MODELS, progress)
}

/// The network's fixed input size
const SIZE: u32 = 512;

/// One round brush mark: centre and radius, in image pixels
pub type Dab = [f32; 3];

type CachedResult = (Arc<RgbaImage>, Vec<Dab>, Arc<RgbaImage>);

/// The last few results: the preview and the full resolution render often fill in the same
/// image, and each run takes a while.
static RESULTS: Mutex<Vec<CachedResult>> = Mutex::new(Vec::new());

/// Fills in everything under `dabs`.
pub fn remove_objects(image: &RgbaImage, dabs: &[Dab], feedback: &Feedback) -> Result<RgbaImage> {
    if dabs.is_empty() {
        return Ok(image.clone());
    }
    {
        let results = RESULTS.lock().unwrap();
        if let Some((_, _, result)) = results.iter().find(|(input, d, _)| d == dabs && **input == *image) {
            return Ok((**result).clone());
        }
    }
    ensure!(is_installed(), "the object removal model is not downloaded yet");
    let session = runtime::session(&models::path(&LAMA)?)?;
    let result = remove_with(image, dabs, &session, feedback)?;
    let mut results = RESULTS.lock().unwrap();
    results.insert(0, (Arc::new(image.clone()), dabs.to_vec(), Arc::new(result.clone())));
    results.truncate(2);
    Ok(result)
}

/// `remove_objects` with the given model.
pub fn remove_with(image: &RgbaImage, dabs: &[Dab], session: &SharedSession, feedback: &Feedback) -> Result<RgbaImage> {
    let (width, height) = image.dimensions();
    let mask = paint_mask(width, height, dabs);
    let areas = areas(width, height, dabs);
    let mut out = image.clone();
    for (index, area) in areas.iter().enumerate() {
        feedback.check()?;
        fill_area(&mut out, &mask, *area, session)?;
        (feedback.progress)((index + 1) as f32 / areas.len() as f32);
    }
    Ok(out)
}

/// Extra pixels around every dab in the mask, so the edge of an object goes too
const GROW: f32 = 3.0;

/// The marked pixels: 255 under the dabs, 0 elsewhere
pub fn paint_mask(width: u32, height: u32, dabs: &[Dab]) -> GrayImage {
    let mut mask = GrayImage::new(width, height);
    for &[cx, cy, r] in dabs {
        let r = r + GROW;
        let x0 = (cx - r).floor().max(0.0) as u32;
        let y0 = (cy - r).floor().max(0.0) as u32;
        let x1 = ((cx + r).ceil().max(0.0) as u32).min(width);
        let y1 = ((cy + r).ceil().max(0.0) as u32).min(height);
        for y in y0..y1 {
            for x in x0..x1 {
                let (dx, dy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
                if dx * dx + dy * dy <= r * r {
                    mask.put_pixel(x, y, Luma([255]));
                }
            }
        }
    }
    mask
}

/// `[left, top, right, bottom]` in pixels
type Area = [u32; 4];

/// Squares around groups of nearby dabs, with context around them for the network to copy
/// from. Each is filled in on its own.
fn areas(width: u32, height: u32, dabs: &[Dab]) -> Vec<Area> {
    // Boxes around the dabs, merged until none overlap
    let mut boxes: Vec<[f32; 4]> =
        dabs.iter().map(|&[x, y, r]| [x - r - GROW, y - r - GROW, x + r + GROW, y + r + GROW]).collect();
    loop {
        let mut merged = false;
        'outer: for i in 0..boxes.len() {
            for j in i + 1..boxes.len() {
                if overlap(&grow(boxes[i]), &grow(boxes[j])) {
                    let b = boxes.swap_remove(j);
                    let a = &mut boxes[i];
                    *a = [a[0].min(b[0]), a[1].min(b[1]), a[2].max(b[2]), a[3].max(b[3])];
                    merged = true;
                    break 'outer;
                }
            }
        }
        if !merged {
            break;
        }
    }
    boxes.into_iter().map(|b| context_square(width, height, b)).collect()
}

/// A box with the context it needs: as much again on every side
fn grow([l, t, r, b]: [f32; 4]) -> [f32; 4] {
    let margin = (r - l).max(b - t) / 2.0;
    [l - margin, t - margin, r + margin, b + margin]
}

fn overlap(a: &[f32; 4], b: &[f32; 4]) -> bool {
    a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3]
}

/// A square twice the size of the box, at least the network's size but at most the image's
/// shorter side, moved inside the image
fn context_square(width: u32, height: u32, [l, t, r, b]: [f32; 4]) -> Area {
    let side = ((r - l).max(b - t) * 2.0).max(SIZE as f32).min(width.min(height) as f32).round() as u32;
    let (cx, cy) = ((l + r) / 2.0, (t + b) / 2.0);
    let left = (cx - side as f32 / 2.0).round().clamp(0.0, (width - side) as f32) as u32;
    let top = (cy - side as f32 / 2.0).round().clamp(0.0, (height - side) as f32) as u32;
    [left, top, left + side, top + side]
}

fn fill_area(out: &mut RgbaImage, mask: &GrayImage, area: Area, session: &SharedSession) -> Result<()> {
    let [left, top, right, bottom] = area;
    let (w, h) = (right - left, bottom - top);
    let crop = image::imageops::crop_imm(out, left, top, w, h).to_image();
    let crop_mask = image::imageops::crop_imm(mask, left, top, w, h).to_image();
    let small = resize_exact(&crop, SIZE, SIZE);
    let mask_rgba = RgbaImage::from_fn(w, h, |x, y| {
        let v = crop_mask.get_pixel(x, y)[0];
        Rgba([v, v, v, 255])
    });
    let small_mask = resize_exact(&mask_rgba, SIZE, SIZE);

    let n = SIZE as usize;
    let mut pixels = vec![0.0f32; 3 * n * n];
    let mut holes = vec![0.0f32; n * n];
    for (x, y, p) in small.enumerate_pixels() {
        let i = y as usize * n + x as usize;
        // Anything touched by the mask counts as a hole
        let hole = small_mask.get_pixel(x, y)[0] > 0;
        holes[i] = if hole { 1.0 } else { 0.0 };
        for c in 0..3 {
            pixels[c * n * n + i] = if hole { 0.0 } else { p[c] as f32 / 255.0 };
        }
    }
    let image = Tensor::from_array(([1usize, 3, n, n], pixels))?.into_dyn();
    let holes = Tensor::from_array(([1usize, 1, n, n], holes))?.into_dyn();
    let outputs = runtime::run(session, image, vec![("mask", holes)])?;
    let filled = &outputs[0].1;
    ensure!(filled.len() == 3 * n * n, "inpainting gave {} values", filled.len());
    let filled = RgbaImage::from_fn(SIZE, SIZE, |x, y| {
        let i = y as usize * n + x as usize;
        let c = |c: usize| filled[c * n * n + i].round().clamp(0.0, 255.0) as u8;
        Rgba([c(0), c(1), c(2), 255])
    });
    let filled = resize_exact(&filled, w, h);

    // Soft edge: the hard mask, widened by a blurred copy
    let soft = imageproc::filter::gaussian_blur_f32(&crop_mask, 2.0);
    for y in 0..h {
        for x in 0..w {
            let weight = crop_mask.get_pixel(x, y)[0].max(soft.get_pixel(x, y)[0]) as f32 / 255.0;
            if weight == 0.0 {
                continue;
            }
            let new = filled.get_pixel(x, y);
            let pixel = out.get_pixel_mut(left + x, top + y);
            for c in 0..3 {
                pixel[c] = (pixel[c] as f32 + (new[c] as f32 - pixel[c] as f32) * weight).round() as u8;
            }
        }
    }
    Ok(())
}

/// Opens a model for `remove_with`.
pub fn load(path: &Path) -> Result<SharedSession> {
    runtime::session(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_covers_the_dabs_and_a_little_more() {
        let mask = paint_mask(100, 100, &[[50.0, 50.0, 10.0]]);
        assert_eq!(mask.get_pixel(50, 50)[0], 255);
        assert_eq!(mask.get_pixel(50, 61)[0], 255);
        assert_eq!(mask.get_pixel(50, 70)[0], 0);
        assert_eq!(mask.get_pixel(0, 0)[0], 0);
    }

    #[test]
    fn nearby_dabs_share_an_area_and_far_ones_get_their_own() {
        let near = areas(4000, 3000, &[[1000.0, 1000.0, 20.0], [1060.0, 1000.0, 20.0]]);
        assert_eq!(near.len(), 1);
        let far = areas(4000, 3000, &[[500.0, 500.0, 20.0], [3500.0, 2500.0, 20.0]]);
        assert_eq!(far.len(), 2);
        for [l, t, r, b] in far {
            // At least the network's size, inside the image
            assert!(r - l >= SIZE && b - t >= SIZE && r <= 4000 && b <= 3000);
        }
    }

    #[test]
    fn areas_fit_in_small_images() {
        // As large as the shorter side allows, over the dab
        let [l, t, r, b] = areas(300, 200, &[[290.0, 10.0, 30.0]])[0];
        assert_eq!((l, t, r, b), (100, 0, 300, 200));
    }

    /// Downloads the model into the real data folder and fills in a disc in the middle of the
    /// photo at `GRANDSHOP_INPAINT_PHOTO`, writing `inpainted.png` next to it:
    /// `cargo test --release inpaint -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn fills_in_part_of_a_photo() {
        install(&|_| {}).unwrap();
        let photo = std::path::PathBuf::from(std::env::var_os("GRANDSHOP_INPAINT_PHOTO").unwrap());
        let image = crate::io::load_image(&photo).unwrap();
        let (w, h) = (image.width() as f32, image.height() as f32);
        let dabs: Vec<Dab> = (0..10).map(|i| [w * 0.4 + i as f32 * w * 0.02, h * 0.5, w.min(h) * 0.04]).collect();
        let start = std::time::Instant::now();
        let filled = remove_objects(&image, &dabs, &Feedback::NONE).unwrap();
        eprintln!("filled in {:?}", start.elapsed());
        assert_eq!(filled.dimensions(), image.dimensions());
        assert_ne!(filled, image);
        filled.save(photo.with_file_name("inpainted.png")).unwrap();
    }
}
