use image::RgbaImage;
use rayon::prelude::*;

use super::srgb::{decode, encode};

/// Black and white: the luminance of each pixel, computed in linear light like libvips.
pub fn greyscale(image: &RgbaImage) -> RgbaImage {
    let mut out = image.clone();
    out.par_chunks_exact_mut(4).for_each(|pixel| {
        let [r, g, b] = [pixel[0], pixel[1], pixel[2]].map(decode);
        let grey = encode(0.212_672_9 * r + 0.715_152_2 * g + 0.072_175 * b);
        pixel[..3].fill(grey);
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn greys_stay_the_same_and_alpha_is_kept() {
        let image = RgbaImage::from_fn(16, 1, |x, _| Rgba([(x * 16) as u8, (x * 16) as u8, (x * 16) as u8, 99]));
        assert_eq!(greyscale(&image), image);
    }

    #[test]
    fn colours_become_grey_by_luminance() {
        let image = RgbaImage::from_fn(3, 1, |x, _| match x {
            0 => Rgba([255, 0, 0, 255]),
            1 => Rgba([0, 255, 0, 255]),
            _ => Rgba([0, 0, 255, 255]),
        });
        let out = greyscale(&image);
        let [red, green, blue] = [0, 1, 2].map(|x| out.get_pixel(x, 0)[0]);
        assert!(out.pixels().all(|p| p[0] == p[1] && p[1] == p[2]));
        assert!(green > red && red > blue, "{red} {green} {blue}");
    }
}
