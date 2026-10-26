use image::RgbaImage;

pub fn mirror(image: &RgbaImage, horizontal: bool, vertical: bool) -> RgbaImage {
    let mut out = if horizontal { image::imageops::flip_horizontal(image) } else { image.clone() };
    if vertical {
        image::imageops::flip_vertical_in_place(&mut out);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn flips_the_chosen_axes() {
        let image = RgbaImage::from_fn(3, 2, |x, y| Rgba([x as u8, y as u8, 0, 255]));
        assert_eq!(mirror(&image, false, false), image);
        assert_eq!(mirror(&image, true, false).get_pixel(0, 0), &Rgba([2, 0, 0, 255]));
        assert_eq!(mirror(&image, false, true).get_pixel(0, 0), &Rgba([0, 1, 0, 255]));
        assert_eq!(mirror(&image, true, true).get_pixel(0, 0), &Rgba([2, 1, 0, 255]));
    }
}
