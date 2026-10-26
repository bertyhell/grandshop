//! Fast conversions between sRGB bytes and linear light, through lookup tables.

use std::sync::OnceLock;

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

/// sRGB byte → linear light, 0 to 1
pub fn decode(value: u8) -> f32 {
    decode_lut()[value as usize]
}

/// Linear light → sRGB byte, clamped
pub fn encode(linear: f32) -> u8 {
    encode_lut()[(linear.clamp(0.0, 1.0) * (ENCODE_STEPS - 1) as f32).round() as usize]
}
