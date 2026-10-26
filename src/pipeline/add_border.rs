use image::{Rgba, RgbaImage};

/// Extends the canvas by `thickness` on every side, filled with `color`.
pub fn add_border(image: &RgbaImage, thickness: u32, color: Rgba<u8>) -> RgbaImage {
    let (width, height) = image.dimensions();
    let mut out = RgbaImage::from_pixel(width + 2 * thickness, height + 2 * thickness, color);
    image::imageops::replace(&mut out, image, thickness as i64, thickness as i64);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extends_every_side_with_the_colour() {
        let blue = Rgba([0x22, 0x5e, 0xff, 255]);
        let image = RgbaImage::from_pixel(10, 5, Rgba([1, 1, 1, 255]));
        let out = add_border(&image, 3, blue);
        assert_eq!(out.dimensions(), (16, 11));
        assert_eq!(out.get_pixel(0, 0), &blue);
        assert_eq!(out.get_pixel(15, 10), &blue);
        assert_eq!(out.get_pixel(2, 5), &blue);
        assert_eq!(out.get_pixel(3, 3), &Rgba([1, 1, 1, 255]));
        assert_eq!(out.get_pixel(12, 7), &Rgba([1, 1, 1, 255]));
    }

    #[test]
    fn zero_thickness_is_identity() {
        let image = RgbaImage::from_pixel(4, 4, Rgba([9, 8, 7, 255]));
        assert_eq!(add_border(&image, 0, Rgba([0, 0, 0, 255])), image);
    }
}
