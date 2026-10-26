//! Compares the Rust modifications with reference images rendered by the Electron app's
//! sharp implementation (see `tests/golden/params.json`). The input is the sharp-decoded
//! `original.png`, so JPEG decoder differences don't count.

use std::path::Path;

use grandshop::ai::Feedback;
use grandshop::io::load_image;
use grandshop::pipeline::{apply_step, parse_hex_color, Dimensions, Modification};
use image::RgbaImage;

const GOLDEN: &str = "tests/golden";

struct Diff {
    /// Mean absolute difference per colour channel, 0–255
    mean: f64,
    /// Share of pixels where some channel differs by more than 16
    outliers: f64,
}

fn diff(expected: &RgbaImage, actual: &RgbaImage) -> Diff {
    assert_eq!(expected.dimensions(), actual.dimensions(), "dimensions differ");
    let mut total = 0u64;
    let mut outliers = 0u64;
    for (e, a) in expected.pixels().zip(actual.pixels()) {
        let deltas = [0, 1, 2].map(|c| e[c].abs_diff(a[c]));
        total += deltas.iter().map(|&d| d as u64).sum::<u64>();
        if deltas.iter().any(|&d| d > 16) {
            outliers += 1;
        }
    }
    let pixels = expected.width() as u64 * expected.height() as u64;
    Diff { mean: total as f64 / (pixels * 3) as f64, outliers: outliers as f64 / pixels as f64 }
}

fn check(file: &str, modification: Modification, max_mean: f64, max_outliers: f64) {
    let original = load_image(&Path::new(GOLDEN).join("original.png")).unwrap();
    let expected = load_image(&Path::new(GOLDEN).join(file)).unwrap();
    let (actual, logical) = apply_step(&original, Dimensions::of(&original), &modification, 1.0, &Feedback::NONE).unwrap();
    assert_eq!(Dimensions::of(&actual), logical);
    let d = diff(&expected, &actual);
    println!("{file}: mean {:.3}, outliers {:.4}%", d.mean, d.outliers * 100.0);
    assert!(d.mean <= max_mean, "{file}: mean difference {:.3} > {max_mean}", d.mean);
    assert!(d.outliers <= max_outliers, "{file}: {:.4}% outlier pixels", d.outliers * 100.0);
}

#[test]
fn rust_jpeg_decoding_matches_sharp() {
    let ours = load_image(Path::new("tests/fixtures/paint.jpg")).unwrap();
    let sharp = load_image(&Path::new(GOLDEN).join("original.png")).unwrap();
    let d = diff(&sharp, &ours);
    println!("decode: mean {:.3}, outliers {:.4}%", d.mean, d.outliers * 100.0);
    assert!(d.mean < 1.0 && d.outliers < 0.001);
}

#[test]
fn add_border() {
    let color = parse_hex_color("#225eff").unwrap();
    check("addBorder-100-225eff.png", Modification::AddBorder { thickness: 100, color }, 0.0, 0.0);
    let black = parse_hex_color("#000000").unwrap();
    check("addBorder-0-000000.png", Modification::AddBorder { thickness: 0, color: black }, 0.0, 0.0);
}

#[test]
fn brightness() {
    for (file, factor) in [("brightness-0.5.png", 0.5), ("brightness-1.1.png", 1.1), ("brightness-2.0.png", 2.0)] {
        check(file, Modification::Brightness { factor, contrast: 1.0 }, 1.0, 0.001);
    }
}

#[test]
fn sepia() {
    check("sepia.png", Modification::Sepia, 0.5, 0.0);
}

#[test]
fn resize() {
    check("resize-960.png", Modification::Resize { size: 960 }, 1.0, 0.001);
}

#[test]
fn crop() {
    check("crop-10-20-30-40.png", Modification::Crop { top: 10, right: 20, bottom: 30, left: 40 }, 0.0, 0.0);
}

/// Different font (Noto Sans instead of text-to-svg's built-in IPA Gothic), so only the
/// placement can match: the text covers a tiny part of the image.
#[test]
fn annotate() {
    let modification = Modification::Annotate {
        text: "Hello\nWorld".into(),
        font_size: 30.0,
        x_percent: 50.0,
        y_percent: 95.0,
        color: image::Rgba([0, 0, 0, 255]),
    };
    check("annotate-hello-world-30-50-95.png", modification, 0.5, 0.002);
}

/// The golden was made with the older single-image border format (corner 188 px) and the
/// Rust version uses the split files (corner 187 px), so allow small geometric differences.
#[test]
fn add_image_border() {
    let modification = Modification::AddImageBorder { border: Some("floral_curly".into()), width: 133 };
    check("addImageBorder-floral_curly-133.png", modification, 6.0, 0.05);
}
