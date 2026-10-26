use std::sync::OnceLock;

use ab_glyph::{Font, FontRef, PxScale};
use image::{GrayImage, Luma, Rgba, RgbaImage};
use imageproc::distance_transform::euclidean_squared_distance_transform;
use imageproc::drawing::{draw_text_mut, text_size};
use imageproc::filter::gaussian_blur_f32;

fn font() -> &'static FontRef<'static> {
    static FONT: OnceLock<FontRef<'static>> = OnceLock::new();
    FONT.get_or_init(|| {
        FontRef::try_from_slice(include_bytes!("../../assets/fonts/NotoSans-Regular.ttf"))
            .expect("bundled font is valid")
    })
}

/// Makes text readable on busy photos. The colour is black or white, whichever contrasts
/// with the text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextEffect {
    #[default]
    None,
    Outline,
    Shadow,
}

/// Outline thickness, shadow offset and shadow blur as a share of the font size
const OUTLINE: f32 = 0.06;
const SHADOW_OFFSET: f32 = 0.06;
const SHADOW_BLUR: f32 = 0.05;
const SHADOW_OPACITY: f32 = 0.75;

/// Black for light text, white for dark text
fn effect_color(text: Rgba<u8>) -> Rgba<u8> {
    let luma = 0.2126 * text[0] as f32 + 0.7152 * text[1] as f32 + 0.0722 * text[2] as f32;
    if luma > 128.0 { Rgba([0, 0, 0, 255]) } else { Rgba([255, 255, 255, 255]) }
}

/// Draws text in `color`, one line per `\n`. Each line is centred horizontally on `x_percent` of
/// the width, and the block starts with its first line centred on `y_percent` of the height.
/// `font_size` is the em size in pixels, like CSS `font-size`.
pub fn annotate(
    image: &RgbaImage,
    text: &str,
    font_size: f32,
    x_percent: f32,
    y_percent: f32,
    color: Rgba<u8>,
    effect: TextEffect,
) -> RgbaImage {
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

    // Top left corner and width of each line
    let lines: Vec<(i32, i32, u32, &str)> = text
        .split('\n')
        .enumerate()
        .map(|(index, line)| {
            let (line_width, _) = text_size(scale, font, line);
            let left = (width * x_percent / 100.0 - line_width as f32 / 2.0).round().clamp(0.0, width);
            let top = (height * y_percent / 100.0 - line_height / 2.0 + index as f32 * line_height)
                .round()
                .clamp(0.0, height);
            (left as i32, top as i32, line_width, line)
        })
        .collect();

    if effect != TextEffect::None {
        draw_effect(&mut out, &lines, line_height, scale, font_size, color, effect);
    }
    for &(left, top, _, line) in &lines {
        draw_text_mut(&mut out, color, left, top, scale, font, line);
    }
    out
}

/// Draws the outline or shadow under the text, in a box around the text only.
fn draw_effect(
    out: &mut RgbaImage,
    lines: &[(i32, i32, u32, &str)],
    line_height: f32,
    scale: PxScale,
    font_size: f32,
    color: Rgba<u8>,
    effect: TextEffect,
) {
    let outline = (font_size * OUTLINE).max(1.0);
    let offset = (font_size * SHADOW_OFFSET).max(1.0).round() as i32;
    let blur = (font_size * SHADOW_BLUR).max(0.5);
    let margin = match effect {
        TextEffect::Outline => outline.ceil() as i32 + 1,
        _ => offset + (blur * 3.0).ceil() as i32,
    };

    let x0 = lines.iter().map(|l| l.0).min().unwrap_or(0) - margin;
    let y0 = lines.iter().map(|l| l.1).min().unwrap_or(0) - margin;
    let x1 = lines.iter().map(|l| l.0 + l.2 as i32).max().unwrap_or(0) + margin;
    let y1 = lines.iter().map(|l| l.1).max().unwrap_or(0) + line_height.ceil() as i32 + margin;
    let (box_width, box_height) = ((x1 - x0).max(1) as u32, (y1 - y0).max(1) as u32);

    // Coverage of the text, 0 to 255
    let mut mask = GrayImage::new(box_width, box_height);
    for &(left, top, _, line) in lines {
        draw_text_mut(&mut mask, Luma([255]), left - x0, top - y0, scale, font(), line);
    }

    let coverage: GrayImage = match effect {
        TextEffect::Outline => {
            // Anti-aliased band of `outline` pixels around the solid part of the glyphs
            let solid = GrayImage::from_fn(box_width, box_height, |x, y| Luma([if mask.get_pixel(x, y)[0] >= 128 { 255 } else { 0 }]));
            let distances = euclidean_squared_distance_transform(&solid);
            GrayImage::from_fn(box_width, box_height, |x, y| {
                let band = (outline + 0.5 - distances.get_pixel(x, y)[0].sqrt() as f32).clamp(0.0, 1.0);
                Luma([((band * 255.0).round() as u8).max(mask.get_pixel(x, y)[0])])
            })
        }
        _ => {
            let mut moved = GrayImage::new(box_width, box_height);
            image::imageops::replace(&mut moved, &mask, offset as i64, offset as i64);
            let blurred = gaussian_blur_f32(&moved, blur);
            GrayImage::from_fn(box_width, box_height, |x, y| {
                Luma([(blurred.get_pixel(x, y)[0] as f32 * SHADOW_OPACITY).round() as u8])
            })
        }
    };

    let shade = effect_color(color);
    for (x, y, Luma([alpha])) in coverage.enumerate_pixels() {
        let (px, py) = (x as i32 + x0, y as i32 + y0);
        if *alpha == 0 || px < 0 || py < 0 || px >= out.width() as i32 || py >= out.height() as i32 {
            continue;
        }
        let a = *alpha as f32 / 255.0;
        let pixel = out.get_pixel_mut(px as u32, py as u32);
        for channel in 0..4 {
            pixel[channel] = (pixel[channel] as f32 * (1.0 - a) + shade[channel] as f32 * a).round() as u8;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLACK: Rgba<u8> = Rgba([0, 0, 0, 255]);

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
        assert_eq!(annotate(&image, "  ", 30.0, 50.0, 50.0, BLACK, TextEffect::None), image);
    }

    #[test]
    fn text_is_centred_on_the_position() {
        let image = RgbaImage::from_pixel(400, 200, Rgba([255, 255, 255, 255]));
        let (x0, y0, x1, y1) = dark_pixel_bounds(&annotate(&image, "Hello", 40.0, 50.0, 50.0, BLACK, TextEffect::None)).unwrap();
        let (cx, cy) = ((x0 + x1) / 2, (y0 + y1) / 2);
        assert!(cx.abs_diff(200) <= 4, "horizontal centre {cx}");
        assert!(cy.abs_diff(100) <= 12, "vertical centre {cy}");
    }

    #[test]
    fn uses_the_text_colour() {
        let image = RgbaImage::from_pixel(200, 100, Rgba([255, 255, 255, 255]));
        let out = annotate(&image, "Hi", 60.0, 50.0, 50.0, Rgba([255, 0, 0, 255]), TextEffect::None);
        assert!(out.pixels().any(|p| *p == Rgba([255, 0, 0, 255])));
        assert!(out.pixels().all(|p| p[0] == 255), "only red and white");
    }

    #[test]
    fn each_line_goes_below_the_previous() {
        let image = RgbaImage::from_pixel(400, 300, Rgba([255, 255, 255, 255]));
        let one = dark_pixel_bounds(&annotate(&image, "Hello", 40.0, 50.0, 30.0, BLACK, TextEffect::None)).unwrap();
        let two = dark_pixel_bounds(&annotate(&image, "Hello\nWorld", 40.0, 50.0, 30.0, BLACK, TextEffect::None)).unwrap();
        assert_eq!(one.1, two.1);
        assert!(two.3 > one.3 + 30, "second line should extend the block downwards");
    }

    #[test]
    fn outline_surrounds_the_text_with_a_contrasting_colour() {
        let image = RgbaImage::from_pixel(400, 200, Rgba([128, 128, 128, 255]));
        let plain = annotate(&image, "Hello", 60.0, 50.0, 50.0, BLACK, TextEffect::None);
        let outlined = annotate(&image, "Hello", 60.0, 50.0, 50.0, BLACK, TextEffect::Outline);
        let white = |img: &RgbaImage| img.pixels().filter(|p| p[0] == 255).count();
        assert_eq!(white(&plain), 0);
        assert!(white(&outlined) > 500, "white outline around black text");
        let (outlined, plain) = (dark_pixel_bounds(&outlined).unwrap(), dark_pixel_bounds(&plain).unwrap());
        assert!(outlined.0.abs_diff(plain.0) <= 1 && outlined.2.abs_diff(plain.2) <= 1, "text itself is on top");
    }

    #[test]
    fn shadow_falls_below_and_right() {
        // Red is dark, so the shadow is white
        let image = RgbaImage::from_pixel(400, 200, Rgba([128, 128, 128, 255]));
        let red = Rgba([255, 0, 0, 255]);
        let plain = annotate(&image, "Hi", 80.0, 50.0, 50.0, red, TextEffect::None);
        let shadowed = annotate(&image, "Hi", 80.0, 50.0, 50.0, red, TextEffect::Shadow);
        let changed: Vec<(u32, u32)> = shadowed
            .enumerate_pixels()
            .filter(|(x, y, p)| *p != plain.get_pixel(*x, *y))
            .map(|(x, y, _)| (x, y))
            .collect();
        assert!(!changed.is_empty());
        let (sx, sy) = changed.iter().fold((0, 0), |(a, b), (x, y)| (a + x, b + y));
        let (cx, cy) = (sx / changed.len() as u32, sy / changed.len() as u32);
        assert!(cx >= 200 && cy >= 100, "shadow centre ({cx}, {cy})");
    }
}
