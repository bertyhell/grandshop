use image::{Rgba, RgbaImage};

/// Removes pixels from each side. Clamps so at least one pixel remains.
pub fn crop(image: &RgbaImage, top: u32, right: u32, bottom: u32, left: u32) -> RgbaImage {
    let (width, height) = image.dimensions();
    let left = left.min(width - 1);
    let top = top.min(height - 1);
    let new_width = width.saturating_sub(left + right).max(1);
    let new_height = height.saturating_sub(top + bottom).max(1);
    image::imageops::crop_imm(image, left, top, new_width, new_height).to_image()
}

/// Adds `background` pixels to each side.
pub fn extend(image: &RgbaImage, top: u32, right: u32, bottom: u32, left: u32, background: Rgba<u8>) -> RgbaImage {
    let (width, height) = image.dimensions();
    let mut out = RgbaImage::from_pixel(width + left + right, height + top + bottom, background);
    image::imageops::replace(&mut out, image, left as i64, top as i64);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_the_requested_edges() {
        let image = RgbaImage::from_fn(10, 8, |x, y| Rgba([x as u8, y as u8, 0, 255]));
        let out = crop(&image, 1, 2, 3, 4);
        assert_eq!(out.dimensions(), (4, 4));
        assert_eq!(out.get_pixel(0, 0), &Rgba([4, 1, 0, 255]));
        assert_eq!(out.get_pixel(3, 3), &Rgba([7, 4, 0, 255]));
    }

    #[test]
    fn never_produces_an_empty_image() {
        let image = RgbaImage::new(10, 10);
        assert_eq!(crop(&image, 5, 5, 5, 5).dimensions(), (1, 1));
    }

    #[test]
    fn extend_fills_the_new_edges() {
        let image = RgbaImage::from_pixel(4, 3, Rgba([1, 2, 3, 255]));
        let white = Rgba([255, 255, 255, 255]);
        let out = extend(&image, 1, 2, 3, 4, white);
        assert_eq!(out.dimensions(), (10, 7));
        assert_eq!(out.get_pixel(0, 0), &white);
        assert_eq!(out.get_pixel(4, 1), &Rgba([1, 2, 3, 255]));
        assert_eq!(out.get_pixel(7, 3), &Rgba([1, 2, 3, 255]));
        assert_eq!(out.get_pixel(8, 3), &white);
    }
}
