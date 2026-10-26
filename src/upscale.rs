//! AI enlarging, run with ONNX Runtime. Two qualities:
//!
//! - Fast: 4x-ClearRealityV1, a small SPAN network that also cleans up JPEG damage. Well under
//!   a second for a typical photo, even on the CPU.
//! - Best: Real-ESRGAN x4plus, and x2plus for doubling: large RRDB networks with more natural
//!   detail, many times slower.
//!
//! The models are downloaded the first time a quality is asked for (see `ai::models`).

use std::sync::{Arc, Mutex};

use anyhow::{ensure, Result};
use image::RgbaImage;

use crate::ai::models::{self, ModelFile};
use crate::ai::runtime;
use crate::ai::tiling::{run_tiled, Tiling};
use crate::ai::Feedback;
use crate::resample::resize_exact;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Quality {
    Fast,
    Best,
}

/// The ONNX exports of facefusion's model collection, at a fixed revision
macro_rules! facefusion {
    ($file:literal) => {
        concat!(
            "https://huggingface.co/facefusion/models-3.0.0/resolve/728b9659bd9691bf32cbf7f61af478d94b7ba81e/",
            $file
        )
    };
}

const CLEAR_REALITY_X4: ModelFile = ModelFile {
    file: "clear_reality_x4.onnx",
    url: facefusion!("clear_reality_x4.onnx"),
    sha256: "bbce9d5a653281cfae07788d620bd4ec45712709bdf6349a06a893159efd97ce",
    bytes: 1_718_947,
    licence: "4x-ClearRealityV1 by Kim2091: CC BY-NC-SA 4.0",
};
const REAL_ESRGAN_X4: ModelFile = ModelFile {
    file: "real_esrgan_x4.onnx",
    url: facefusion!("real_esrgan_x4.onnx"),
    sha256: "afa7fa42eed0315a3d7b2a54e79e4e44a672d3c0889c2828c4f84c9ab91edb0b",
    bytes: 69_539_503,
    licence: "Real-ESRGAN x4plus by Xintao Wang: BSD-3-Clause",
};
const REAL_ESRGAN_X2: ModelFile = ModelFile {
    file: "real_esrgan_x2.onnx",
    url: facefusion!("real_esrgan_x2.onnx"),
    sha256: "5be2d62ab3b0357986efb20a19638334c0e7a2a055ccbc52c77063da0482b3bb",
    bytes: 69_552_244,
    licence: "Real-ESRGAN x2plus by Xintao Wang: BSD-3-Clause",
};

/// Weights of the earlier candle version, no longer used
const OBSOLETE_FILES: [&str; 3] = ["realesr-general-x4v3.pth", "realesr-general-wdn-x4v3.pth", "RealESRGAN_x4plus.pth"];

/// Longest side of an enlarged image, in original-image pixels
pub const MAX_RESULT_EDGE: u32 = 8000;

impl Quality {
    /// Everything this quality downloads
    pub fn models(self) -> &'static [&'static ModelFile] {
        match self {
            Quality::Fast => &[&CLEAR_REALITY_X4],
            Quality::Best => &[&REAL_ESRGAN_X4, &REAL_ESRGAN_X2],
        }
    }

    /// Download size in bytes
    pub fn download_bytes(self) -> u64 {
        self.models().iter().map(|model| model.bytes).sum()
    }

    /// The network for enlarging `factor` times, and how it is cut into tiles. Larger factors
    /// than the network's are not asked for; smaller ones shrink its result.
    fn network(self, factor: u32) -> (&'static ModelFile, Tiling) {
        match (self, factor) {
            (Quality::Fast, _) => (&CLEAR_REALITY_X4, Tiling { tile: 256, pad: 16, scale: 4 }),
            (Quality::Best, 2) => (&REAL_ESRGAN_X2, Tiling { tile: 128, pad: 16, scale: 2 }),
            (Quality::Best, _) => (&REAL_ESRGAN_X4, Tiling { tile: 128, pad: 16, scale: 4 }),
        }
    }
}

pub fn is_installed(quality: Quality) -> bool {
    models::is_installed(quality.models())
}

/// Downloads the models for `quality`, reporting the fraction done. Blocks for as long as the
/// download takes.
pub fn install(quality: Quality, progress: &dyn Fn(f32)) -> Result<()> {
    if let Some(folder) = models::models_dir() {
        for file in OBSOLETE_FILES {
            let _ = std::fs::remove_file(folder.join(file));
        }
    }
    models::install(quality.models(), progress)
}

/// The last few results: the preview and the full resolution render often enlarge the same
/// image, and each run takes a while.
type CachedResult = (Arc<RgbaImage>, u32, Quality, Arc<RgbaImage>);

const CACHED_RESULTS: usize = 2;

static RESULTS: Mutex<Vec<CachedResult>> = Mutex::new(Vec::new());

/// Enlarges `factor` times (2 to 4) with the downloaded models for `quality`.
pub fn upscale(image: &RgbaImage, factor: u32, quality: Quality, feedback: &Feedback) -> Result<RgbaImage> {
    ensure!((2..=4).contains(&factor), "can only enlarge 2 to 4 times, not {factor}");
    {
        let results = RESULTS.lock().unwrap();
        let cached = results.iter().find(|(input, f, q, _)| *f == factor && *q == quality && **input == *image);
        if let Some((_, _, _, result)) = cached {
            return Ok((**result).clone());
        }
    }
    let (model, tiling) = quality.network(factor);
    ensure!(models::is_installed(&[model]), "the enlarging model is not downloaded yet");
    let session = runtime::session(&models::path(model)?)?;
    let enlarged = run_tiled(image, tiling, feedback, |rgb, size| Ok(runtime::run_image(&session, rgb, size)?.1))?;
    let (width, height) = image.dimensions();
    let result = if tiling.scale == factor {
        enlarged
    } else {
        resize_exact(&enlarged, width * factor, height * factor)
    };
    let mut results = RESULTS.lock().unwrap();
    results.insert(0, (Arc::new(image.clone()), factor, quality, Arc::new(result.clone())));
    results.truncate(CACHED_RESULTS);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Downloads into the real data folder: `cargo test --release upscale -- --ignored`
    #[test]
    #[ignore]
    fn downloads_and_enlarges_with_both_qualities() {
        for quality in [Quality::Fast, Quality::Best] {
            install(quality, &|_| {}).unwrap();
            assert!(is_installed(quality));
            for factor in [2, 3, 4] {
                let image = RgbaImage::from_pixel(20, 10, image::Rgba([10, 200, 30, 128]));
                let big = upscale(&image, factor, quality, &Feedback::NONE).unwrap();
                assert_eq!(big.dimensions(), (20 * factor, 10 * factor));
                assert!(big.pixels().all(|p| p[3].abs_diff(128) <= 2));
            }
        }
    }

    /// Needs the downloaded models: `cargo test --release upscale -- --ignored --nocapture`.
    /// Set `GRANDSHOP_UPSCALE_OUT` to a folder to look at the results.
    #[test]
    #[ignore]
    fn enlarges_the_fixture() {
        let photo = crate::io::load_image(std::path::Path::new("tests/fixtures/paint.jpg")).unwrap();
        let small = resize_exact(&photo, 300, 200);
        // Close to a plain enlargement overall, just sharper
        let plain = resize_exact(&small, 1200, 800);
        for (quality, name) in [(Quality::Fast, "fast"), (Quality::Best, "best")] {
            let start = std::time::Instant::now();
            let big = upscale(&small, 4, quality, &Feedback::NONE).unwrap();
            eprintln!("{name}: 300×200 → {:?} in {:?}", big.dimensions(), start.elapsed());
            assert_eq!(big.dimensions(), (1200, 800));
            let diff: u64 = big.as_raw().iter().zip(plain.as_raw()).map(|(a, b)| a.abs_diff(*b) as u64).sum();
            let mean = diff as f64 / big.as_raw().len() as f64;
            assert!(mean < 12.0, "{name}: mean difference {mean}");
            if let Ok(out) = std::env::var("GRANDSHOP_UPSCALE_OUT") {
                big.save(format!("{out}/{name}.png")).unwrap();
            }
        }
    }
}
