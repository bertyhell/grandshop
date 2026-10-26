//! High quality SIMD resizing shared by the proxy and the modifications.

use fast_image_resize::{images::Image, FilterType, PixelType, ResizeAlg, ResizeOptions, Resizer};
use image::RgbaImage;

/// Resizes to exactly `width` × `height` with Lanczos3, handling alpha correctly.
pub fn resize_exact(image: &RgbaImage, width: u32, height: u32) -> RgbaImage {
    let width = width.max(1);
    let height = height.max(1);
    if image.dimensions() == (width, height) {
        return image.clone();
    }
    let mut dst = Image::new(width, height, PixelType::U8x4);
    let options = ResizeOptions::new().resize_alg(ResizeAlg::Convolution(FilterType::Lanczos3));
    Resizer::new()
        .resize(image, &mut dst, &options)
        .expect("source and destination are both RGBA8");
    RgbaImage::from_raw(width, height, dst.into_vec()).expect("buffer matches dimensions")
}

/// Largest size with the same aspect ratio that fits inside `max` × `max`.
pub fn fit_inside(width: u32, height: u32, max: u32) -> (u32, u32) {
    if width >= height {
        let h = (height as f64 * max as f64 / width as f64).round() as u32;
        (max, h.max(1))
    } else {
        let w = (width as f64 * max as f64 / height as f64).round() as u32;
        (w.max(1), max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_inside_keeps_aspect_ratio() {
        assert_eq!(fit_inside(4000, 3000, 1000), (1000, 750));
        assert_eq!(fit_inside(3000, 4000, 1000), (750, 1000));
        assert_eq!(fit_inside(100, 100, 50), (50, 50));
    }

    #[test]
    fn resize_exact_produces_requested_size() {
        let image = RgbaImage::from_pixel(100, 60, image::Rgba([200, 100, 50, 255]));
        let resized = resize_exact(&image, 33, 17);
        assert_eq!(resized.dimensions(), (33, 17));
        assert_eq!(resized.get_pixel(16, 8), &image::Rgba([200, 100, 50, 255]));
    }
}
