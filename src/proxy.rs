//! Downscaled working copy used for fast previews while editing.

use image::RgbaImage;

use crate::resample::{fit_inside, resize_exact};

/// Longest edge of the preview proxy in pixels.
pub const PROXY_MAX_EDGE: u32 = 2560;

pub struct Proxy {
    pub image: RgbaImage,
    /// Proxy pixels per original pixel (1.0 when the original is small enough).
    pub scale: f32,
}

pub fn make_proxy(original: &RgbaImage) -> Proxy {
    let (width, height) = original.dimensions();
    if width.max(height) <= PROXY_MAX_EDGE {
        return Proxy { image: original.clone(), scale: 1.0 };
    }
    let (proxy_width, proxy_height) = fit_inside(width, height, PROXY_MAX_EDGE);
    Proxy {
        image: resize_exact(original, proxy_width, proxy_height),
        scale: proxy_width.max(proxy_height) as f32 / width.max(height) as f32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_images_are_used_as_is() {
        let proxy = make_proxy(&RgbaImage::new(800, 600));
        assert_eq!(proxy.image.dimensions(), (800, 600));
        assert_eq!(proxy.scale, 1.0);
    }

    #[test]
    fn large_images_are_downscaled_to_the_max_edge() {
        let proxy = make_proxy(&RgbaImage::new(6000, 4000));
        assert_eq!(proxy.image.dimensions(), (2560, 1707));
        assert!((proxy.scale - 2560.0 / 6000.0).abs() < 1e-6);
    }
}
