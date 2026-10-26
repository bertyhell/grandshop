//! Brightness in CIE L*a*b*, matching sharp's `modulate({ brightness })`, which multiplies
//! the lightness of LCh. Chroma and hue (so a* and b*) stay the same.
//! Saturation multiplies the chroma (a* and b*), so 0 gives greys of the same lightness.
//! Contrast then stretches the sRGB values away from (or towards) mid grey.

use image::RgbaImage;
use rayon::prelude::*;

use super::srgb::{decode, encode};

const WHITE: [f32; 3] = [0.950_47, 1.0, 1.088_83];
const DELTA: f32 = 6.0 / 29.0;

fn f(t: f32) -> f32 {
    if t > DELTA * DELTA * DELTA { t.cbrt() } else { t / (3.0 * DELTA * DELTA) + 4.0 / 29.0 }
}

fn f_inv(t: f32) -> f32 {
    if t > DELTA { t * t * t } else { 3.0 * DELTA * DELTA * (t - 4.0 / 29.0) }
}

/// `factor`, `contrast` and `saturation` 1 leave the image unchanged.
pub fn brightness(image: &RgbaImage, factor: f32, contrast: f32, saturation: f32) -> RgbaImage {
    let adjust_contrast = |value: u8| ((value as f32 - 127.5) * contrast + 127.5).round().clamp(0.0, 255.0) as u8;
    let contrast_changes = (contrast - 1.0).abs() > f32::EPSILON;

    let mut out = image.clone();
    out.par_chunks_exact_mut(4).for_each(|pixel| {
        let [r, g, b] = [pixel[0], pixel[1], pixel[2]].map(decode);

        // linear sRGB → XYZ (D65) → Lab
        let x = (0.412_456_4 * r + 0.357_576_1 * g + 0.180_437_5 * b) / WHITE[0];
        let y = (0.212_672_9 * r + 0.715_152_2 * g + 0.072_175 * b) / WHITE[1];
        let z = (0.019_333_9 * r + 0.119_192 * g + 0.950_304_1 * b) / WHITE[2];
        let (fx, fy, fz) = (f(x), f(y), f(z));
        let lightness = (116.0 * fy - 16.0) * factor;
        let a = 500.0 * (fx - fy) * saturation;
        let b_star = 200.0 * (fy - fz) * saturation;

        // Lab → XYZ → linear sRGB
        let fy = (lightness + 16.0) / 116.0;
        let x = WHITE[0] * f_inv(fy + a / 500.0);
        let y = WHITE[1] * f_inv(fy);
        let z = WHITE[2] * f_inv(fy - b_star / 200.0);
        pixel[0] = encode(3.240_454_2 * x - 1.537_138_5 * y - 0.498_531_4 * z);
        pixel[1] = encode(-0.969_266 * x + 1.876_010_8 * y + 0.041_556 * z);
        pixel[2] = encode(0.055_643_4 * x - 0.204_025_9 * y + 1.057_225_2 * z);
        if contrast_changes {
            for channel in &mut pixel[..3] {
                *channel = adjust_contrast(*channel);
            }
        }
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Pixel, Rgba};

    fn gradient() -> RgbaImage {
        RgbaImage::from_fn(16, 16, |x, y| Rgba([(x * 16) as u8, (y * 16) as u8, ((x + y) * 8) as u8, 77]))
    }

    #[test]
    fn factor_one_is_identity_within_rounding() {
        let image = gradient();
        let out = brightness(&image, 1.0, 1.0, 1.0);
        for (a, b) in image.pixels().zip(out.pixels()) {
            for c in 0..3 {
                assert!(a[c].abs_diff(b[c]) <= 1, "{a:?} → {b:?}");
            }
            assert_eq!(a[3], b[3], "alpha must be kept");
        }
    }

    fn luma(p: &Rgba<u8>) -> f32 {
        0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32
    }

    #[test]
    fn lower_factor_darkens_every_pixel() {
        let image = gradient();
        let out = brightness(&image, 0.5, 1.0, 1.0);
        for (before, after) in image.pixels().zip(out.pixels()) {
            assert!(luma(after) <= luma(before) + 0.5, "{before:?} → {after:?}");
        }
    }

    #[test]
    fn factor_zero_removes_lightness_of_greys() {
        // Saturated colours keep their chroma, like sharp, so only greys end up black
        let image = RgbaImage::from_fn(16, 1, |x, _| image::Rgb([(x * 16) as u8; 3]).to_rgba());
        assert!(brightness(&image, 0.0, 1.0, 1.0).pixels().all(|p| p[0] <= 1 && p[1] <= 1 && p[2] <= 1));
    }

    #[test]
    fn contrast_pushes_values_away_from_mid_grey() {
        let image = RgbaImage::from_fn(3, 1, |x, _| image::Rgb([[64u8, 128, 192][x as usize]; 3]).to_rgba());
        let out = brightness(&image, 1.0, 2.0, 1.0);
        assert!(out.get_pixel(0, 0)[0] < 10, "{:?}", out.get_pixel(0, 0));
        assert!(out.get_pixel(1, 0)[0].abs_diff(128) <= 2);
        assert!(out.get_pixel(2, 0)[0] > 245);

        let flat = brightness(&image, 1.0, 0.0, 1.0);
        assert!(flat.pixels().all(|p| p[0].abs_diff(128) <= 1), "contrast 0 is flat grey");
    }

    #[test]
    fn grey_gets_lighter_and_stays_grey() {
        let image = RgbaImage::from_pixel(1, 1, Rgba([100, 100, 100, 255]));
        let p = *brightness(&image, 1.5, 1.0, 1.0).get_pixel(0, 0);
        assert!(p[0] > 140, "{p:?}");
        assert!(p[0].abs_diff(p[1]) <= 1 && p[1].abs_diff(p[2]) <= 1, "{p:?}");
    }

    #[test]
    fn saturation_zero_gives_greys_and_more_saturates() {
        let image = gradient();
        assert!(brightness(&image, 1.0, 1.0, 0.0).pixels().all(|p| p[0].abs_diff(p[1]) <= 2 && p[1].abs_diff(p[2]) <= 2));

        let orange = RgbaImage::from_pixel(1, 1, Rgba([200, 120, 60, 255]));
        let spread = |p: &Rgba<u8>| p[0] as i32 - p[2] as i32;
        let more = *brightness(&orange, 1.0, 1.0, 1.5).get_pixel(0, 0);
        assert!(spread(&more) > spread(orange.get_pixel(0, 0)), "{more:?}");
    }
}
