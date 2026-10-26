//! "Repair old photo": SCUNet removes grain, noise and JPEG blocks, staying close to the photo
//! itself. A strength blends between the original and the repaired photo; the repaired photo
//! is kept, so changing the strength is quick.
//!
//! The model is downloaded the first time the operation is used (see `ai::models`).

use std::sync::{Arc, Mutex};

use anyhow::{ensure, Result};
use image::{Rgba, RgbaImage};

use crate::ai::models::{self, ModelFile};
use crate::ai::runtime::{self, SharedSession};
use crate::ai::tiling::{run_tiled, Tiling};
use crate::ai::Feedback;

/// The ONNX export of SCUNet, at a fixed revision
macro_rules! scunet {
    ($file:literal) => {
        concat!("https://huggingface.co/Heliosoph/scunet-onnx/resolve/6d11417ee2fbcc73783c502a238ac115097754fe/", $file)
    };
}

const SCUNET: ModelFile = ModelFile {
    file: "scunet_color_real_psnr.onnx",
    url: scunet!("scunet_color_real_psnr.onnx"),
    sha256: "231be201ab413dbc999d7951caa9844846b93a12a40a41e037d6b5888ed4e88c",
    bytes: 3_798_678,
    licence: "SCUNet (real, PSNR) by Kai Zhang et al.: Apache-2.0",
};
/// The weights, which the model file loads by this name from its own folder
const SCUNET_DATA: ModelFile = ModelFile {
    file: "scunet_color_real_psnr.onnx.data",
    url: scunet!("scunet_color_real_psnr.onnx.data"),
    sha256: "98825ea1210b641c71e5f052f582c70c49fd44b35387ebe2c034268c17df3feb",
    bytes: 73_138_176,
    licence: "SCUNet (real, PSNR) by Kai Zhang et al.: Apache-2.0",
};

/// Everything the operation downloads
pub const MODELS: &[&ModelFile] = &[&SCUNET, &SCUNET_DATA];

pub fn is_installed() -> bool {
    models::is_installed(MODELS)
}

/// Downloads the model, reporting the fraction done. Blocks for as long as it takes.
pub fn install(progress: &dyn Fn(f32)) -> Result<()> {
    models::install(MODELS, progress)
}

/// Sizes divisible by 8, as the network needs
const TILING: Tiling = Tiling { tile: 256, pad: 32, scale: 1 };

type CachedRepair = (Arc<RgbaImage>, Arc<RgbaImage>);

/// The last few repaired photos: the strength slider, the preview and the full resolution
/// render often need the same one, and each run takes a while.
static REPAIRED: Mutex<Vec<CachedRepair>> = Mutex::new(Vec::new());

/// Repairs the photo. `strength` (0 to 1) blends from the original to the repaired photo.
pub fn repair(image: &RgbaImage, strength: f32, feedback: &Feedback) -> Result<RgbaImage> {
    let cached = REPAIRED.lock().unwrap().iter().find(|(input, _)| **input == *image).map(|(_, r)| r.clone());
    let repaired = match cached {
        Some(repaired) => repaired,
        None => {
            ensure!(is_installed(), "the repair model is not downloaded yet");
            let session = runtime::session(&models::path(&SCUNET)?)?;
            let repaired = Arc::new(repair_with(image, &session, feedback)?);
            let mut cache = REPAIRED.lock().unwrap();
            cache.insert(0, (Arc::new(image.clone()), repaired.clone()));
            cache.truncate(2);
            repaired
        }
    };
    Ok(blend(image, &repaired, strength))
}

/// Runs the network over the whole photo.
pub fn repair_with(image: &RgbaImage, session: &SharedSession, feedback: &Feedback) -> Result<RgbaImage> {
    run_tiled(image, TILING, feedback, |rgb, size| Ok(runtime::run_image(session, rgb, size)?.1))
}

/// `original` with `amount` (0 to 1) of `repaired` over it; alpha from the original
pub fn blend(original: &RgbaImage, repaired: &RgbaImage, amount: f32) -> RgbaImage {
    RgbaImage::from_fn(original.width(), original.height(), |x, y| {
        let (a, b) = (original.get_pixel(x, y), repaired.get_pixel(x, y));
        let mix = |c: usize| (a[c] as f32 + (b[c] as f32 - a[c] as f32) * amount).round() as u8;
        Rgba([mix(0), mix(1), mix(2), a[3]])
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blend_goes_from_original_to_repaired() {
        let original = RgbaImage::from_pixel(2, 2, Rgba([0, 100, 200, 128]));
        let repaired = RgbaImage::from_pixel(2, 2, Rgba([100, 100, 100, 255]));
        assert_eq!(blend(&original, &repaired, 0.0), original);
        assert_eq!(blend(&original, &repaired, 0.5).get_pixel(0, 0), &Rgba([50, 100, 150, 128]));
        assert_eq!(blend(&original, &repaired, 1.0).get_pixel(1, 1), &Rgba([100, 100, 100, 128]));
    }

    /// Downloads the model into the real data folder and repairs the photo at
    /// `GRANDSHOP_REPAIR_PHOTO`, writing `repaired.png` next to it:
    /// `cargo test --release repair -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn repairs_a_photo() {
        install(&|_| {}).unwrap();
        let photo = std::path::PathBuf::from(std::env::var_os("GRANDSHOP_REPAIR_PHOTO").unwrap());
        let image = crate::io::load_image(&photo).unwrap();
        let start = std::time::Instant::now();
        let repaired = repair(&image, 1.0, &Feedback::NONE).unwrap();
        eprintln!("repaired in {:?}", start.elapsed());
        assert_eq!(repaired.dimensions(), image.dimensions());
        assert_ne!(repaired, image);
        // The second time comes from the cache
        let start = std::time::Instant::now();
        repair(&image, 0.5, &Feedback::NONE).unwrap();
        assert!(start.elapsed().as_secs_f32() < 1.0);
        repaired.save(photo.with_file_name("repaired.png")).unwrap();
    }
}
