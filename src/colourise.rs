//! "Colourise": a network guesses the colours of a black-and-white photo at a small size; they
//! are enlarged and combined with the photo's own lightness, so the detail stays sharp at any
//! size.
//!
//! The model is downloaded the first time the operation is used (see `ai::models`).

use std::path::Path;
use std::sync::{Arc, Mutex};

use anyhow::{ensure, Result};
use image::{Rgba, RgbaImage};
use ort::value::Tensor;

use crate::ai::models::{self, ModelFile};
use crate::ai::runtime::{self, SharedSession};
use crate::ai::Feedback;
use crate::resample::resize_exact;

/// The ONNX exports of facefusion's model collection, at a fixed revision
macro_rules! facefusion {
    ($file:literal) => {
        concat!(
            "https://huggingface.co/facefusion/models-3.0.0/resolve/728b9659bd9691bf32cbf7f61af478d94b7ba81e/",
            $file
        )
    };
}

const DDCOLOR: ModelFile = ModelFile {
    file: "ddcolor.onnx",
    url: facefusion!("ddcolor.onnx"),
    sha256: "dd6156f056d0690891ef35f9ee87176d4d976ab19bbd36e57d18e841e064d797",
    bytes: 980_103_562,
    licence: "DDColor by Xiaoyang Kang et al. (Alibaba DAMO): Apache-2.0",
};

/// Everything the operation downloads
pub const MODELS: &[&ModelFile] = &[&DDCOLOR];

pub fn is_installed() -> bool {
    models::is_installed(MODELS)
}

/// Downloads the model, reporting the fraction done. Blocks for as long as it takes.
pub fn install(progress: &dyn Fn(f32)) -> Result<()> {
    models::install(MODELS, progress)
}

/// How a colourising network wants its input and gives its output
#[derive(Debug, Clone, Copy)]
pub enum Colourizer {
    /// Grey RGB in 0..1, the a and b channels of Lab out
    DdColor { size: u32 },
    /// Grey RGB in 0..255, coloured RGB in 0..255 out
    DeOldify { size: u32 },
}

pub const DDCOLOR_512: Colourizer = Colourizer::DdColor { size: 512 };

/// The last few results: the preview and the full resolution render often colourise the same
/// image, and each run takes a while.
type CachedColours = (Arc<RgbaImage>, Arc<ColourPlanes>);

static COLOURS: Mutex<Vec<CachedColours>> = Mutex::new(Vec::new());

/// Colours the photo. `strength` (0 to 1) goes from grey to the full guessed colours.
pub fn colourise(image: &RgbaImage, strength: f32, feedback: &Feedback) -> Result<RgbaImage> {
    let cached = COLOURS.lock().unwrap().iter().find(|(input, _)| **input == *image).map(|(_, c)| c.clone());
    let colours = match cached {
        Some(colours) => colours,
        None => {
            ensure!(is_installed(), "the colouring model is not downloaded yet");
            let session = runtime::session(&models::path(&DDCOLOR)?)?;
            let colours = Arc::new(guess_colours(image, &session, DDCOLOR_512, feedback)?);
            let mut cache = COLOURS.lock().unwrap();
            cache.insert(0, (Arc::new(image.clone()), colours.clone()));
            cache.truncate(2);
            colours
        }
    };
    Ok(apply_colours(image, &colours, strength))
}

/// The a and b channels of Lab at the network's size
pub struct ColourPlanes {
    size: usize,
    a: Vec<f32>,
    b: Vec<f32>,
}

/// Runs the network on a grey version of `image`.
pub fn guess_colours(
    image: &RgbaImage,
    session: &SharedSession,
    colourizer: Colourizer,
    feedback: &Feedback,
) -> Result<ColourPlanes> {
    feedback.check()?;
    let (size, scale) = match colourizer {
        Colourizer::DdColor { size } => (size, 1.0 / 255.0),
        Colourizer::DeOldify { size } => (size, 1.0),
    };
    let small = resize_exact(image, size, size);
    let n = size as usize;
    let mut input = vec![0.0; 3 * n * n];
    for (x, y, pixel) in small.enumerate_pixels() {
        let [l, _, _] = srgb_to_lab([pixel[0], pixel[1], pixel[2]]);
        let grey = lab_to_srgb([l, 0.0, 0.0]);
        for c in 0..3 {
            input[c * n * n + y as usize * n + x as usize] = grey[c] as f32 * scale;
        }
    }
    let tensor = Tensor::from_array(([1usize, 3, n, n], input))?.into_dyn();
    let outputs = runtime::run(session, tensor, Vec::new())?;
    let (shape, values) = outputs.into_iter().next().unwrap_or_default();
    (feedback.progress)(0.9);
    let plane = n * n;
    match colourizer {
        Colourizer::DdColor { .. } => {
            ensure!(values.len() == 2 * plane, "colouring gave shape {shape:?}");
            Ok(ColourPlanes { size: n, a: values[..plane].to_vec(), b: values[plane..].to_vec() })
        }
        Colourizer::DeOldify { .. } => {
            ensure!(values.len() == 3 * plane, "colouring gave shape {shape:?}");
            let (mut a, mut b) = (Vec::with_capacity(plane), Vec::with_capacity(plane));
            for i in 0..plane {
                let rgb = [0, 1, 2].map(|c| values[c * plane + i].round().clamp(0.0, 255.0) as u8);
                let [_, la, lb] = srgb_to_lab(rgb);
                a.push(la);
                b.push(lb);
            }
            Ok(ColourPlanes { size: n, a, b })
        }
    }
}

/// The lightness of `image` with the guessed colours, enlarged bilinearly to its size.
pub fn apply_colours(image: &RgbaImage, colours: &ColourPlanes, strength: f32) -> RgbaImage {
    let (width, height) = image.dimensions();
    let n = colours.size;
    let sample = |plane: &[f32], fx: f32, fy: f32| {
        let x = fx.clamp(0.0, (n - 1) as f32);
        let y = fy.clamp(0.0, (n - 1) as f32);
        let (x0, y0) = (x.floor() as usize, y.floor() as usize);
        let (x1, y1) = ((x0 + 1).min(n - 1), (y0 + 1).min(n - 1));
        let (tx, ty) = (x - x0 as f32, y - y0 as f32);
        let top = plane[y0 * n + x0] + (plane[y0 * n + x1] - plane[y0 * n + x0]) * tx;
        let bottom = plane[y1 * n + x0] + (plane[y1 * n + x1] - plane[y1 * n + x0]) * tx;
        top + (bottom - top) * ty
    };
    RgbaImage::from_fn(width, height, |x, y| {
        let pixel = image.get_pixel(x, y);
        let [l, _, _] = srgb_to_lab([pixel[0], pixel[1], pixel[2]]);
        let fx = (x as f32 + 0.5) * n as f32 / width as f32 - 0.5;
        let fy = (y as f32 + 0.5) * n as f32 / height as f32 - 0.5;
        let a = sample(&colours.a, fx, fy) * strength;
        let b = sample(&colours.b, fx, fy) * strength;
        let [r, g, bl] = lab_to_srgb([l, a, b]);
        Rgba([r, g, bl, pixel[3]])
    })
}

// sRGB ↔ CIE Lab (D65), with L in 0..100

const WHITE: [f32; 3] = [0.950_47, 1.0, 1.088_83];

fn to_linear(c: u8) -> f32 {
    let c = c as f32 / 255.0;
    if c <= 0.040_45 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

fn from_linear(c: f32) -> u8 {
    let c = if c <= 0.003_130_8 { c * 12.92 } else { 1.055 * c.max(0.0).powf(1.0 / 2.4) - 0.055 };
    (c * 255.0).round().clamp(0.0, 255.0) as u8
}

pub fn srgb_to_lab(rgb: [u8; 3]) -> [f32; 3] {
    let [r, g, b] = rgb.map(to_linear);
    let xyz = [
        0.412_456_4 * r + 0.357_576_1 * g + 0.180_437_5 * b,
        0.212_672_9 * r + 0.715_152_2 * g + 0.072_175 * b,
        0.019_333_9 * r + 0.119_192 * g + 0.950_304_1 * b,
    ];
    let f = |t: f32| if t > 0.008_856 { t.cbrt() } else { 7.787 * t + 16.0 / 116.0 };
    let [fx, fy, fz] = [0, 1, 2].map(|i| f(xyz[i] / WHITE[i]));
    [116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)]
}

pub fn lab_to_srgb([l, a, b]: [f32; 3]) -> [u8; 3] {
    let fy = (l + 16.0) / 116.0;
    let (fx, fz) = (fy + a / 500.0, fy - b / 200.0);
    let f_inv = |t: f32| if t > 0.206_893 { t * t * t } else { (t - 16.0 / 116.0) / 7.787 };
    let [x, y, z] = [f_inv(fx) * WHITE[0], f_inv(fy) * WHITE[1], f_inv(fz) * WHITE[2]];
    [
        3.240_454_2 * x - 1.537_138_5 * y - 0.498_531_4 * z,
        -0.969_266 * x + 1.876_010_8 * y + 0.041_556 * z,
        0.055_643_4 * x - 0.204_025_9 * y + 1.057_225_2 * z,
    ]
    .map(from_linear)
}

/// Opens a model for `guess_colours`.
pub fn load(path: &Path) -> Result<SharedSession> {
    runtime::session(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lab_round_trips() {
        for rgb in [[0, 0, 0], [255, 255, 255], [200, 30, 90], [12, 180, 240], [128, 128, 128]] {
            let back = lab_to_srgb(srgb_to_lab(rgb));
            for c in 0..3 {
                assert!(back[c].abs_diff(rgb[c]) <= 1, "{rgb:?} came back as {back:?}");
            }
        }
        let [l, a, b] = srgb_to_lab([255, 255, 255]);
        assert!((l - 100.0).abs() < 0.1 && a.abs() < 0.1 && b.abs() < 0.1);
    }

    #[test]
    fn colours_keep_the_lightness_and_zero_strength_stays_grey() {
        let image = RgbaImage::from_pixel(8, 6, Rgba([120, 120, 120, 200]));
        let colours = ColourPlanes { size: 2, a: vec![30.0; 4], b: vec![-20.0; 4] };
        let grey = apply_colours(&image, &colours, 0.0);
        assert!(grey.pixels().all(|p| p[0] == p[1] && p[1] == p[2] && p[0].abs_diff(120) <= 1 && p[3] == 200));
        let coloured = apply_colours(&image, &colours, 1.0);
        let p = coloured.get_pixel(3, 3);
        assert!(p[0] > p[1], "a > 0 is redder: {p:?}");
        let l = srgb_to_lab([p[0], p[1], p[2]])[0];
        assert!((l - srgb_to_lab([120, 120, 120])[0]).abs() < 1.0);
    }

    /// Downloads the model (1 GB) into the real data folder and colours the photo at
    /// `GRANDSHOP_COLOURISE_PHOTO`, writing `colourised.png` next to it:
    /// `cargo test --release colourise -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn colourises_a_photo() {
        install(&|_| {}).unwrap();
        let photo = std::path::PathBuf::from(std::env::var_os("GRANDSHOP_COLOURISE_PHOTO").unwrap());
        let image = crate::io::load_image(&photo).unwrap();
        let start = std::time::Instant::now();
        let coloured = colourise(&image, 1.0, &Feedback::NONE).unwrap();
        eprintln!("coloured in {:?}", start.elapsed());
        assert_eq!(coloured.dimensions(), image.dimensions());
        assert!(coloured.pixels().any(|p| p[0] != p[2]));
        coloured.save(photo.with_file_name("colourised.png")).unwrap();
    }
}
