//! HSV colour maths for the colour picker, and the picker's hue/saturation wheel.

use image::{Rgba, RgbaImage};

/// `hue` in degrees, `saturation` and `value` from 0 to 1.
pub fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> [u8; 3] {
    let hue = hue.rem_euclid(360.0) / 60.0;
    let (s, v) = (saturation.clamp(0.0, 1.0), value.clamp(0.0, 1.0));
    let chroma = v * s;
    let x = chroma * (1.0 - (hue % 2.0 - 1.0).abs());
    let (r, g, b) = match hue as u32 {
        0 => (chroma, x, 0.0),
        1 => (x, chroma, 0.0),
        2 => (0.0, chroma, x),
        3 => (0.0, x, chroma),
        4 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };
    let m = v - chroma;
    [r, g, b].map(|c| ((c + m) * 255.0).round() as u8)
}

/// Returns `(hue in degrees, saturation, value)`.
pub fn rgb_to_hsv([r, g, b]: [u8; 3]) -> (f32, f32, f32) {
    let [r, g, b] = [r, g, b].map(|c| c as f32 / 255.0);
    let max = r.max(g).max(b);
    let delta = max - r.min(g).min(b);
    let hue = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * ((g - b) / delta).rem_euclid(6.0)
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };
    let saturation = if max == 0.0 { 0.0 } else { delta / max };
    (hue, saturation, max)
}

/// Colour wheel at full value: hue goes counter-clockwise from red on the right, and
/// saturation grows from the centre outwards. Outside the circle is transparent.
pub fn color_wheel(size: u32) -> RgbaImage {
    let radius = size as f32 / 2.0;
    RgbaImage::from_fn(size, size, |x, y| {
        let dx = x as f32 + 0.5 - radius;
        let dy = radius - (y as f32 + 0.5);
        let distance = (dx * dx + dy * dy).sqrt();
        // One pixel of anti-aliasing on the edge
        let alpha = (radius - distance + 0.5).clamp(0.0, 1.0);
        let hue = dy.atan2(dx).to_degrees();
        let [r, g, b] = hsv_to_rgb(hue, (distance / radius).min(1.0), 1.0);
        Rgba([r, g, b, (alpha * 255.0).round() as u8])
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_primary_colours() {
        assert_eq!(hsv_to_rgb(0.0, 1.0, 1.0), [255, 0, 0]);
        assert_eq!(hsv_to_rgb(120.0, 1.0, 1.0), [0, 255, 0]);
        assert_eq!(hsv_to_rgb(240.0, 1.0, 1.0), [0, 0, 255]);
        assert_eq!(hsv_to_rgb(-120.0, 1.0, 1.0), [0, 0, 255]);
        assert_eq!(hsv_to_rgb(30.0, 0.0, 0.5), [128, 128, 128]);
    }

    #[test]
    fn round_trips_through_hsv() {
        for rgb in [[0x22, 0x5e, 0xff], [255, 128, 0], [10, 200, 30], [0, 0, 0], [255, 255, 255]] {
            let (h, s, v) = rgb_to_hsv(rgb);
            assert_eq!(hsv_to_rgb(h, s, v), rgb);
        }
    }

    #[test]
    fn wheel_has_red_on_the_right_and_white_in_the_centre() {
        let wheel = color_wheel(101);
        assert_eq!(wheel.get_pixel(50, 50), &Rgba([255, 255, 255, 255]));
        let right = wheel.get_pixel(98, 50);
        assert!(right[0] > 240 && right[1] < 20 && right[2] < 20, "{right:?}");
        let top = wheel.get_pixel(50, 2); // 90°, green-yellow side
        assert!(top[1] > 240, "{top:?}");
        assert_eq!(wheel.get_pixel(0, 0)[3], 0, "outside the circle");
    }
}
