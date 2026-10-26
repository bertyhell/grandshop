use image::RgbaImage;
use rayon::prelude::*;

use super::srgb::{decode, encode};

/// Black and white: the luminance of each pixel, computed in linear light like libvips.
/// `strength` 1 is fully grey, 0 leaves the image unchanged; in between blends in linear light.
pub fn greyscale(image: &RgbaImage, strength: f32) -> RgbaImage {
    let strength = strength.clamp(0.0, 1.0);
    let mut out = image.clone();
    out.par_chunks_exact_mut(4).for_each(|pixel| {
        let linear = [pixel[0], pixel[1], pixel[2]].map(decode);
        let grey = 0.212_672_9 * linear[0] + 0.715_152_2 * linear[1] + 0.072_175 * linear[2];
        if strength >= 1.0 {
            pixel[..3].fill(encode(grey));
        } else {
            for (channel, value) in pixel[..3].iter_mut().zip(linear) {
                *channel = encode(value + (grey - value) * strength);
            }
        }
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
        assert_eq!(greyscale(&image, 1.0), image);
    }

    #[test]
    fn colours_become_grey_by_luminance() {
        let image = RgbaImage::from_fn(3, 1, |x, _| match x {
            0 => Rgba([255, 0, 0, 255]),
            1 => Rgba([0, 255, 0, 255]),
            _ => Rgba([0, 0, 255, 255]),
        });
        let out = greyscale(&image, 1.0);
        let [red, green, blue] = [0, 1, 2].map(|x| out.get_pixel(x, 0)[0]);
        assert!(out.pixels().all(|p| p[0] == p[1] && p[1] == p[2]));
        assert!(green > red && red > blue, "{red} {green} {blue}");
    }

    #[test]
    fn strength_blends_with_the_original() {
        let image = RgbaImage::from_fn(3, 1, |x, _| Rgba([[200, 30, 90][x as usize], 120, 10, 255]));
        let half = greyscale(&image, 0.5);
        let full = greyscale(&image, 1.0);
        assert!(greyscale(&image, 0.0).pixels().zip(image.pixels()).all(|(a, b)| a[0].abs_diff(b[0]) <= 1));
        for ((before, mid), grey) in image.pixels().zip(half.pixels()).zip(full.pixels()) {
            for c in 0..3 {
                let (low, high) = (before[c].min(grey[c]), before[c].max(grey[c]));
                assert!((low..=high).contains(&mid[c]), "{before:?} {mid:?} {grey:?}");
            }
        }
    }
}
