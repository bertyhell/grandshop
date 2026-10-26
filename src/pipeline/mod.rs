//! The modification stack: an ordered list of parameterised operations that is always
//! re-applied to the original (or its proxy).
//!
//! Pixel parameters are expressed in original-image pixels, which is what the UI shows.
//! When running on the proxy, `scale` converts them to proxy pixels. Alongside the real
//! image, the pipeline tracks the *logical* size: the size the result would have at full
//! resolution. The UI uses it for slider ranges.

mod add_border;
pub mod annotate;
mod brightness;
mod crop;
mod greyscale;
pub mod image_border;
mod mirror;
mod resize;
mod rotate;
mod sepia;
mod srgb;

use anyhow::Result;
use image::{Rgba, RgbaImage};

pub use annotate::TextEffect;
pub use image_border::available_borders;

#[derive(Debug, Clone, PartialEq)]
pub enum Modification {
    /// Solid border around the image on every side, `percent` of its shorter side thick.
    AddBorder { percent: f32, color: Rgba<u8> },
    /// Decorative border drawn over the edges of the image, its corners `percent` of the
    /// shorter side wide. `None` until a style is picked.
    AddImageBorder { border: Option<String>, percent: f32 },
    /// Multiplies the lightness and the colour intensity, then stretches the contrast; 1.0
    /// leaves the image unchanged.
    Brightness { factor: f32, contrast: f32, saturation: f32 },
    /// Strength 0 to 1; 1 is the full effect.
    Sepia { strength: f32 },
    Greyscale { strength: f32 },
    /// Turns the image clockwise by any angle; the canvas grows and the corners get `background`.
    /// With `fill`, the corners are cut off instead, keeping the aspect ratio.
    Rotate { angle: f32, background: Rgba<u8>, fill: bool },
    Mirror { horizontal: bool, vertical: bool },
    /// Shrinks the image to fit inside `size` × `size`.
    Resize { size: u32 },
    /// Enlarges `factor` times with the AI model; 1 leaves the image as is (model not
    /// downloaded yet).
    Upscale { factor: u32, quality: crate::upscale::Quality },
    /// Text, one line per `\n`, centred on a point given in percent of the image size. The font
    /// size is a percentage of the shorter side.
    Annotate { text: String, font_percent: f32, x_percent: f32, y_percent: f32, color: Rgba<u8>, effect: TextEffect },
    /// Removes the given number of pixels from each side.
    Crop { top: u32, right: u32, bottom: u32, left: u32 },
    /// Adds the given number of pixels of `background` to each side.
    Extend { top: u32, right: u32, bottom: u32, left: u32, background: Rgba<u8> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

impl Dimensions {
    pub fn of(image: &RgbaImage) -> Self {
        let (width, height) = image.dimensions();
        Self { width, height }
    }
}

/// Length in original-image pixels of `percent` of the shorter side.
fn percent_of_shorter_side(percent: f32, logical: Dimensions) -> f32 {
    logical.width.min(logical.height) as f32 * percent / 100.0
}

/// Converts a length in original-image pixels to working-image pixels.
fn scale_px(value: u32, scale: f32) -> u32 {
    (value as f32 * scale).round() as u32
}

/// Applies one modification. Returns the new image and its logical size.
pub fn apply_step(
    image: &RgbaImage,
    logical: Dimensions,
    modification: &Modification,
    scale: f32,
) -> Result<(RgbaImage, Dimensions)> {
    match modification {
        Modification::AddBorder { percent, color } => {
            let thickness = &(percent_of_shorter_side(*percent, logical).round().max(0.0) as u32);
            let logical = Dimensions {
                width: logical.width + 2 * thickness,
                height: logical.height + 2 * thickness,
            };
            Ok((add_border::add_border(image, scale_px(*thickness, scale), *color), logical))
        }
        Modification::AddImageBorder { border: None, .. } => Ok((image.clone(), logical)),
        Modification::AddImageBorder { border: Some(name), percent } => {
            let border_width = (percent_of_shorter_side(*percent, logical).round() * scale).max(1.0);
            Ok((image_border::add_image_border(image, name, border_width)?, logical))
        }
        Modification::Brightness { factor, contrast, saturation } => {
            Ok((brightness::brightness(image, *factor, *contrast, *saturation), logical))
        }
        Modification::Sepia { strength } => Ok((sepia::sepia(image, *strength), logical)),
        Modification::Greyscale { strength } => Ok((greyscale::greyscale(image, *strength), logical)),
        Modification::Rotate { angle, fill: true, .. } => {
            let (width, height) = rotate::filled_size(logical.width, logical.height, *angle);
            Ok((rotate::rotate_filled(image, *angle), Dimensions { width, height }))
        }
        Modification::Rotate { angle, background, fill: false } => {
            let (width, height) = rotate::rotated_size(logical.width, logical.height, *angle);
            Ok((rotate::rotate(image, *angle, *background), Dimensions { width, height }))
        }
        Modification::Mirror { horizontal, vertical } => {
            Ok((mirror::mirror(image, *horizontal, *vertical), logical))
        }
        Modification::Resize { size } => {
            let (width, height) =
                crate::resample::fit_inside(logical.width, logical.height, (*size).max(1));
            let target = scale_px(*size, scale).max(1);
            Ok((resize::resize(image, target), Dimensions { width, height }))
        }
        Modification::Upscale { factor: 0 | 1, .. } => Ok((image.clone(), logical)),
        Modification::Upscale { factor, quality } => {
            let logical = Dimensions { width: logical.width * factor, height: logical.height * factor };
            if logical.width.max(logical.height) > crate::upscale::MAX_RESULT_EDGE {
                anyhow::bail!(
                    "enlarging to {}×{} goes over the {} pixel limit",
                    logical.width,
                    logical.height,
                    crate::upscale::MAX_RESULT_EDGE
                );
            }
            Ok((crate::upscale::upscale(image, *factor, *quality)?, logical))
        }
        Modification::Annotate { text, font_percent, x_percent, y_percent, color, effect } => Ok((
            annotate::annotate(
                image,
                text,
                percent_of_shorter_side(*font_percent, logical) * scale,
                *x_percent,
                *y_percent,
                *color,
                *effect,
            ),
            logical,
        )),
        Modification::Crop { top, right, bottom, left } => {
            if left + right >= logical.width || top + bottom >= logical.height {
                anyhow::bail!(
                    "crop of {top}/{right}/{bottom}/{left} leaves nothing of a {}×{} image",
                    logical.width,
                    logical.height
                );
            }
            let logical = Dimensions {
                width: logical.width - left - right,
                height: logical.height - top - bottom,
            };
            let edges = [top, right, bottom, left].map(|v| scale_px(*v, scale));
            Ok((crop::crop(image, edges[0], edges[1], edges[2], edges[3]), logical))
        }
        Modification::Extend { top, right, bottom, left, background } => {
            let logical = Dimensions {
                width: logical.width + left + right,
                height: logical.height + top + bottom,
            };
            let edges = [top, right, bottom, left].map(|v| scale_px(*v, scale));
            Ok((crop::extend(image, edges[0], edges[1], edges[2], edges[3], *background), logical))
        }
    }
}

pub struct PipelineOutput {
    pub image: RgbaImage,
    pub logical: Dimensions,
    /// Modifications that failed, by index. They are skipped and the rest still applies.
    pub errors: Vec<(usize, anyhow::Error)>,
}

/// Applies the whole stack to `source`, whose original (full resolution) size is `original`.
pub fn apply(
    source: &RgbaImage,
    original: Dimensions,
    modifications: &[Modification],
    scale: f32,
) -> PipelineOutput {
    let mut image = source.clone();
    let mut logical = original;
    let mut errors = Vec::new();
    for (index, modification) in modifications.iter().enumerate() {
        match apply_step(&image, logical, modification, scale) {
            Ok((next, next_logical)) => {
                image = next;
                logical = next_logical;
            }
            Err(err) => errors.push((index, err)),
        }
    }
    PipelineOutput { image, logical, errors }
}

/// Parses `#rgb`, `#rrggbb` or `#rrggbbaa`.
pub fn parse_hex_color(hex: &str) -> Option<Rgba<u8>> {
    let hex = hex.strip_prefix('#')?;
    let channel = |i: usize| u8::from_str_radix(hex.get(i..i + 2)?, 16).ok();
    match hex.len() {
        3 => {
            let short = |i: usize| u8::from_str_radix(hex.get(i..i + 1)?, 16).ok().map(|v| v * 17);
            Some(Rgba([short(0)?, short(1)?, short(2)?, 255]))
        }
        6 => Some(Rgba([channel(0)?, channel(2)?, channel(4)?, 255])),
        8 => Some(Rgba([channel(0)?, channel(2)?, channel(4)?, channel(6)?])),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{io::load_image, proxy::make_proxy, resample::resize_exact};

    fn fixture() -> RgbaImage {
        load_image(std::path::Path::new("tests/fixtures/paint.jpg")).unwrap()
    }

    #[test]
    fn parses_hex_colors() {
        assert_eq!(parse_hex_color("#225eff"), Some(Rgba([0x22, 0x5e, 0xff, 255])));
        assert_eq!(parse_hex_color("#fff"), Some(Rgba([255, 255, 255, 255])));
        assert_eq!(parse_hex_color("#00000080"), Some(Rgba([0, 0, 0, 128])));
        assert_eq!(parse_hex_color("225eff"), None);
        assert_eq!(parse_hex_color("#22zz00"), None);
    }

    #[test]
    fn empty_stack_returns_source() {
        let image = RgbaImage::from_pixel(4, 4, Rgba([1, 2, 3, 255]));
        let output = apply(&image, Dimensions::of(&image), &[], 1.0);
        assert_eq!(output.image, image);
        assert!(output.errors.is_empty());
    }

    #[test]
    fn tracks_logical_dimensions_through_the_stack() {
        let image = RgbaImage::new(1000, 500);
        let output = apply(
            &image,
            Dimensions::of(&image),
            &[
                Modification::AddBorder { percent: 10.0, color: Rgba([0, 0, 0, 255]) },
                Modification::Crop { top: 10, right: 20, bottom: 30, left: 40 },
                Modification::Resize { size: 520 },
                Modification::Rotate { angle: 90.0, background: Rgba([0, 0, 0, 255]), fill: false },
            ],
            1.0,
        );
        // 1000×500 → border 1100×600 → crop 1040×560 → fit in 520 → quarter turn
        assert_eq!(output.logical, Dimensions { width: 280, height: 520 });
        assert_eq!(Dimensions::of(&output.image), output.logical);
    }

    #[test]
    fn failing_modification_is_skipped_and_reported() {
        let image = RgbaImage::new(100, 100);
        let output = apply(
            &image,
            Dimensions::of(&image),
            &[
                Modification::Crop { top: 60, right: 0, bottom: 60, left: 0 },
                Modification::AddBorder { percent: 5.0, color: Rgba([0, 0, 0, 255]) },
            ],
            1.0,
        );
        assert_eq!(output.errors.len(), 1);
        assert_eq!(output.errors[0].0, 0);
        assert_eq!(Dimensions::of(&output.image), Dimensions { width: 110, height: 110 });
    }

    #[test]
    fn upscale_without_model_keeps_the_image_and_too_large_fails() {
        use crate::upscale::Quality;
        let image = RgbaImage::new(100, 50);
        let output = apply(&image, Dimensions::of(&image), &[Modification::Upscale { factor: 1, quality: Quality::Fast }], 1.0);
        assert_eq!(output.image, image);
        assert!(output.errors.is_empty());

        let huge = Dimensions { width: crate::upscale::MAX_RESULT_EDGE, height: 10 };
        let output = apply(&image, huge, &[Modification::Upscale { factor: 2, quality: Quality::Fast }], 1.0);
        assert_eq!(output.errors.len(), 1);
        assert_eq!(output.logical, huge);
    }

    #[test]
    fn proxy_and_logical_sizes_agree() {
        let original = RgbaImage::from_pixel(6000, 4000, Rgba([90, 120, 150, 255]));
        let proxy = make_proxy(&original);
        let output = apply(
            &proxy.image,
            Dimensions::of(&original),
            &[
                Modification::AddBorder { percent: 7.5, color: Rgba([255, 0, 0, 255]) },
                Modification::Crop { top: 100, right: 200, bottom: 300, left: 400 },
            ],
            proxy.scale,
        );
        assert_eq!(output.logical, Dimensions { width: 6000, height: 4200 });
        let expected = (6000.0 * proxy.scale, 4200.0 * proxy.scale);
        let actual = output.image.dimensions();
        assert!((actual.0 as f32 - expected.0).abs() <= 2.0, "{actual:?} vs {expected:?}");
        assert!((actual.1 as f32 - expected.1).abs() <= 2.0, "{actual:?} vs {expected:?}");
    }

    /// Running the stack on the proxy must look like running it at full resolution and
    /// downscaling the result.
    #[test]
    fn proxy_preview_matches_downscaled_full_render() {
        let original = resize_exact(&fixture(), 3840, 2160);
        let proxy = make_proxy(&original);
        let stack = [
            Modification::Brightness { factor: 1.3, contrast: 1.2, saturation: 1.1 },
            Modification::AddBorder { percent: 120.0 / 21.6, color: Rgba([34, 94, 255, 255]) },
            Modification::Crop { top: 40, right: 80, bottom: 120, left: 160 },
            Modification::Sepia { strength: 1.0 },
            Modification::AddImageBorder { border: Some("floral_curly".into()), percent: 200.0 / 21.6 },
            Modification::Rotate { angle: 12.5, background: Rgba([255, 255, 255, 255]), fill: false },
            Modification::Mirror { horizontal: true, vertical: false },
            Modification::Greyscale { strength: 1.0 },
        ];
        let full = apply(&original, Dimensions::of(&original), &stack, 1.0);
        let preview = apply(&proxy.image, Dimensions::of(&original), &stack, proxy.scale);
        assert!(full.errors.is_empty() && preview.errors.is_empty());

        let (w, h) = preview.image.dimensions();
        let downscaled = resize_exact(&full.image, w, h);
        let total: u64 = downscaled
            .as_raw()
            .iter()
            .zip(preview.image.as_raw())
            .map(|(a, b)| a.abs_diff(*b) as u64)
            .sum();
        let mean = total as f64 / downscaled.as_raw().len() as f64;
        assert!(mean < 4.0, "mean absolute difference {mean}");
    }
}
