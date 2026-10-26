//! Brightness in CIE L*a*b*, matching sharp's `modulate({ brightness })`, which multiplies
//! the lightness of LCh. Chroma and hue (so a* and b*) stay the same.

use std::sync::OnceLock;

use image::RgbaImage;
use rayon::prelude::*;

const WHITE: [f32; 3] = [0.950_47, 1.0, 1.088_83];
const DELTA: f32 = 6.0 / 29.0;

/// sRGB byte → linear light
fn decode_lut() -> &'static [f32; 256] {
    static LUT: OnceLock<[f32; 256]> = OnceLock::new();
    LUT.get_or_init(|| {
        std::array::from_fn(|i| {
            let c = i as f32 / 255.0;
            if c <= 0.040_45 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
        })
    })
}

const ENCODE_STEPS: usize = 1 << 16;

/// Linear light (quantised to 16 bits) → sRGB byte
fn encode_lut() -> &'static [u8] {
    static LUT: OnceLock<Vec<u8>> = OnceLock::new();
    LUT.get_or_init(|| {
        (0..ENCODE_STEPS)
            .map(|i| {
                let l = i as f32 / (ENCODE_STEPS - 1) as f32;
                let c = if l <= 0.003_130_8 { l * 12.92 } else { 1.055 * l.powf(1.0 / 2.4) - 0.055 };
                (c * 255.0).round().clamp(0.0, 255.0) as u8
            })
            .collect()
    })
}

fn f(t: f32) -> f32 {
    if t > DELTA * DELTA * DELTA { t.cbrt() } else { t / (3.0 * DELTA * DELTA) + 4.0 / 29.0 }
}

fn f_inv(t: f32) -> f32 {
    if t > DELTA { t * t * t } else { 3.0 * DELTA * DELTA * (t - 4.0 / 29.0) }
}

pub fn brightness(image: &RgbaImage, factor: f32) -> RgbaImage {
    let decode = decode_lut();
    let encode = encode_lut();
    let encode_channel = |linear: f32| {
        encode[(linear.clamp(0.0, 1.0) * (ENCODE_STEPS - 1) as f32).round() as usize]
    };

    let mut out = image.clone();
    out.par_chunks_exact_mut(4).for_each(|pixel| {
        let [r, g, b] = [pixel[0], pixel[1], pixel[2]].map(|c| decode[c as usize]);

        // linear sRGB → XYZ (D65) → Lab
        let x = (0.412_456_4 * r + 0.357_576_1 * g + 0.180_437_5 * b) / WHITE[0];
        let y = (0.212_672_9 * r + 0.715_152_2 * g + 0.072_175 * b) / WHITE[1];
        let z = (0.019_333_9 * r + 0.119_192 * g + 0.950_304_1 * b) / WHITE[2];
        let (fx, fy, fz) = (f(x), f(y), f(z));
        let lightness = (116.0 * fy - 16.0) * factor;
        let a = 500.0 * (fx - fy);
        let b_star = 200.0 * (fy - fz);

        // Lab → XYZ → linear sRGB
        let fy = (lightness + 16.0) / 116.0;
        let x = WHITE[0] * f_inv(fy + a / 500.0);
        let y = WHITE[1] * f_inv(fy);
        let z = WHITE[2] * f_inv(fy - b_star / 200.0);
        pixel[0] = encode_channel(3.240_454_2 * x - 1.537_138_5 * y - 0.498_531_4 * z);
        pixel[1] = encode_channel(-0.969_266 * x + 1.876_010_8 * y + 0.041_556 * z);
        pixel[2] = encode_channel(0.055_643_4 * x - 0.204_025_9 * y + 1.057_225_2 * z);
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
        let out = brightness(&image, 1.0);
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
        let out = brightness(&image, 0.5);
        for (before, after) in image.pixels().zip(out.pixels()) {
            assert!(luma(after) <= luma(before) + 0.5, "{before:?} → {after:?}");
        }
    }

    #[test]
    fn factor_zero_removes_lightness_of_greys() {
        // Saturated colours keep their chroma, like sharp, so only greys end up black
        let image = RgbaImage::from_fn(16, 1, |x, _| image::Rgb([(x * 16) as u8; 3]).to_rgba());
        assert!(brightness(&image, 0.0).pixels().all(|p| p[0] <= 1 && p[1] <= 1 && p[2] <= 1));
    }

    #[test]
    fn grey_gets_lighter_and_stays_grey() {
        let image = RgbaImage::from_pixel(1, 1, Rgba([100, 100, 100, 255]));
        let p = *brightness(&image, 1.5).get_pixel(0, 0);
        assert!(p[0] > 140, "{p:?}");
        assert!(p[0].abs_diff(p[1]) <= 1 && p[1].abs_diff(p[2]) <= 1, "{p:?}");
    }
}
