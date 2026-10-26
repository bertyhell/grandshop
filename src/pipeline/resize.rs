use image::RgbaImage;

use crate::resample::{fit_inside, resize_exact};

/// Resizes so the image fits inside `size` × `size`, keeping the aspect ratio.
pub fn resize(image: &RgbaImage, size: u32) -> RgbaImage {
    let (width, height) = fit_inside(image.width(), image.height(), size);
    resize_exact(image, width, height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fits_the_longest_edge() {
        assert_eq!(resize(&RgbaImage::new(1920, 1080), 960).dimensions(), (960, 540));
        assert_eq!(resize(&RgbaImage::new(1080, 1920), 960).dimensions(), (540, 960));
    }
}
