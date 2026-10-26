//! Loading and saving images.

use std::path::Path;

use anyhow::{Context, Result};
use chrono::NaiveDate;
use image::{ImageDecoder, ImageReader, RgbaImage};

/// Decodes any supported image file into RGBA8, applying the EXIF orientation.
pub fn load_image(path: &Path) -> Result<RgbaImage> {
    let is_svg = path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("svg"));
    if is_svg {
        let data = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
        return decode_svg(&data);
    }

    // Detect the format from the content, so files with a wrong extension still open
    let mut decoder = ImageReader::open(path)
        .with_context(|| format!("opening {}", path.display()))?
        .with_guessed_format()?
        .into_decoder()
        .with_context(|| format!("unsupported image format: {}", path.display()))?;
    let orientation = decoder.orientation()?;
    let mut image = image::DynamicImage::from_decoder(decoder)
        .with_context(|| format!("decoding {}", path.display()))?;
    image.apply_orientation(orientation);
    Ok(image.into_rgba8())
}

/// Rasterizes an SVG document at its intrinsic size.
pub fn decode_svg(data: &[u8]) -> Result<RgbaImage> {
    let tree = resvg::usvg::Tree::from_data(data, &resvg::usvg::Options::default())
        .context("parsing SVG")?;
    let size = tree.size().to_int_size();
    let mut pixmap = resvg::tiny_skia::Pixmap::new(size.width(), size.height())
        .context("SVG has an empty size")?;
    resvg::render(&tree, resvg::tiny_skia::Transform::default(), &mut pixmap.as_mut());

    // tiny-skia stores premultiplied alpha, image expects straight alpha
    let pixels = pixmap
        .pixels()
        .iter()
        .flat_map(|p| {
            let c = p.demultiply();
            [c.red(), c.green(), c.blue(), c.alpha()]
        })
        .collect();
    RgbaImage::from_raw(size.width(), size.height(), pixels).context("SVG pixel buffer size")
}

/// Encodes an image as PNG.
pub fn encode_png(image: &RgbaImage) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    image.write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png)?;
    Ok(bytes)
}

/// Saves an image as PNG, whatever extension the path has.
pub fn save_png(image: &RgbaImage, path: &Path) -> Result<()> {
    let bytes = encode_png(image)?;
    std::fs::write(path, bytes).with_context(|| format!("writing {}", path.display()))
}

/// Suggested file name for saving an edited image: `<original name>-<yyyy-mm-dd>.png`.
pub fn default_save_file_name(original: &Path, date: NaiveDate) -> String {
    let stem = original
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "image".to_owned());
    format!("{stem}-{}.png", date.format("%Y-%m-%d"))
}

/// Human readable size with three significant digits, like the `pretty-bytes` package:
/// `"512 B"`, `"1.23 MB"`.
pub fn format_file_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "kB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1000.0 && unit < UNITS.len() - 1 {
        value /= 1000.0;
        unit += 1;
    }
    let decimals = if unit == 0 || value >= 100.0 { 0 } else if value >= 10.0 { 1 } else { 2 };
    let text = format!("{value:.decimals$}");
    // Drop trailing zeros: 1.50 → 1.5, 2.00 → 2
    let text = if text.contains('.') { text.trim_end_matches('0').trim_end_matches('.') } else { &text };
    format!("{text} {}", UNITS[unit])
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn default_file_name_uses_stem_and_date() {
        let date = NaiveDate::from_ymd_opt(2021, 8, 9).unwrap();
        assert_eq!(
            default_save_file_name(Path::new("/photos/grandma.jpg"), date),
            "grandma-2021-08-09.png"
        );
    }

    #[test]
    fn formats_file_sizes() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(999), "999 B");
        assert_eq!(format_file_size(1500), "1.5 kB");
        assert_eq!(format_file_size(1_234_567), "1.23 MB");
        assert_eq!(format_file_size(45_600_000), "45.6 MB");
        assert_eq!(format_file_size(2_000_000_000), "2 GB");
    }

    #[test]
    fn png_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.png");
        let image = RgbaImage::from_pixel(3, 2, Rgba([10, 20, 30, 128]));
        save_png(&image, &path).unwrap();
        assert_eq!(load_image(&path).unwrap(), image);
    }

    #[test]
    fn loads_jpeg_fixture() {
        let image = load_image(Path::new("tests/fixtures/paint.jpg")).unwrap();
        assert_eq!(image.dimensions(), (1920, 1080));
    }

    #[test]
    fn loads_svg() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("red.svg");
        std::fs::write(
            &path,
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="10"><rect width="20" height="10" fill="#ff0000"/></svg>"##,
        )
        .unwrap();
        let image = load_image(&path).unwrap();
        assert_eq!(image.dimensions(), (20, 10));
        assert_eq!(image.get_pixel(5, 5), &Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn missing_file_is_an_error() {
        assert!(load_image(Path::new("tests/fixtures/does-not-exist.png")).is_err());
    }
}
