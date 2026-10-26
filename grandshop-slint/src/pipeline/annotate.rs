use std::sync::OnceLock;

use ab_glyph::{Font, FontRef, PxScale};
use image::{Rgba, RgbaImage};
use imageproc::drawing::{draw_text_mut, text_size};

fn font() -> &'static FontRef<'static> {
    static FONT: OnceLock<FontRef<'static>> = OnceLock::new();
    FONT.get_or_init(|| {
        FontRef::try_from_slice(include_bytes!("../../assets/fonts/NotoSans-Regular.ttf"))
            .expect("bundled font is valid")
    })
}

/// Draws black text, one line per `\n`. Each line is centred horizontally on `x_percent` of
/// the width, and the block starts with its first line centred on `y_percent` of the height.
/// `font_size` is the em size in pixels, like CSS `font-size`.
pub fn annotate(image: &RgbaImage, text: &str, font_size: f32, x_percent: f32, y_percent: f32) -> RgbaImage {
    let mut out = image.clone();
    if text.trim().is_empty() || font_size <= 0.0 {
        return out;
    }

    let font = font();
    let units_per_em = font.units_per_em().unwrap_or(1000.0);
    // ab_glyph scales so that ascent - descent equals the PxScale, so convert from em size
    let line_height = font_size * font.height_unscaled() / units_per_em;
    let scale = PxScale::from(line_height);
    let (width, height) = (image.width() as f32, image.height() as f32);

    for (index, line) in text.split('\n').enumerate() {
        let (line_width, _) = text_size(scale, font, line);
        let left = (width * x_percent / 100.0 - line_width as f32 / 2.0).round().clamp(0.0, width);
        let top = (height * y_percent / 100.0 - line_height / 2.0 + index as f32 * line_height)
            .round()
            .clamp(0.0, height);
        draw_text_mut(&mut out, Rgba([0, 0, 0, 255]), left as i32, top as i32, scale, font, line);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dark_pixel_bounds(image: &RgbaImage) -> Option<(u32, u32, u32, u32)> {
        let mut bounds: Option<(u32, u32, u32, u32)> = None;
        for (x, y, p) in image.enumerate_pixels() {
            if p[0] < 128 {
                let b = bounds.get_or_insert((x, y, x, y));
                *b = (b.0.min(x), b.1.min(y), b.2.max(x), b.3.max(y));
            }
        }
        bounds
    }

    #[test]
    fn empty_text_draws_nothing() {
        let image = RgbaImage::from_pixel(100, 100, Rgba([255, 255, 255, 255]));
        assert_eq!(annotate(&image, "  ", 30.0, 50.0, 50.0), image);
    }

    #[test]
    fn text_is_centred_on_the_position() {
        let image = RgbaImage::from_pixel(400, 200, Rgba([255, 255, 255, 255]));
        let (x0, y0, x1, y1) = dark_pixel_bounds(&annotate(&image, "Hello", 40.0, 50.0, 50.0)).unwrap();
        let (cx, cy) = ((x0 + x1) / 2, (y0 + y1) / 2);
        assert!(cx.abs_diff(200) <= 4, "horizontal centre {cx}");
        assert!(cy.abs_diff(100) <= 12, "vertical centre {cy}");
    }

    #[test]
    fn each_line_goes_below_the_previous() {
        let image = RgbaImage::from_pixel(400, 300, Rgba([255, 255, 255, 255]));
        let one = dark_pixel_bounds(&annotate(&image, "Hello", 40.0, 50.0, 30.0)).unwrap();
        let two = dark_pixel_bounds(&annotate(&image, "Hello\nWorld", 40.0, 50.0, 30.0)).unwrap();
        assert_eq!(one.1, two.1);
        assert!(two.3 > one.3 + 30, "second line should extend the block downwards");
    }
}
