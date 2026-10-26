//! "Remove background": a segmentation network marks the subject, and everything else becomes
//! transparent or a chosen colour.
//!
//! The model is downloaded the first time the operation is used (see `ai::models`).

use std::path::Path;
use std::sync::{Arc, Mutex};

use anyhow::{ensure, Result};
use image::{GrayImage, Luma, Rgba, RgbaImage};
use ort::value::Tensor;

use crate::ai::models::{self, ModelFile};
use crate::ai::runtime::{self, SharedSession};
use crate::ai::Feedback;
use crate::resample::resize_exact;

const BIREFNET_LITE: ModelFile = ModelFile {
    file: "birefnet_general_lite.onnx",
    url: "https://github.com/danielgatis/rembg/releases/download/v0.0.0/BiRefNet-general-bb_swin_v1_tiny-epoch_232.onnx",
    sha256: "5600024376f572a557870a5eb0afb1e5961636bef4e1e22132025467d0f03333",
    bytes: 224_005_088,
    licence: "BiRefNet (general, lite) by Peng Zheng et al.: MIT",
};

/// Everything the operation downloads
pub const MODELS: &[&ModelFile] = &[&BIREFNET_LITE];

pub fn is_installed() -> bool {
    models::is_installed(MODELS)
}

/// Downloads the model, reporting the fraction done. Blocks for as long as it takes.
pub fn install(progress: &dyn Fn(f32)) -> Result<()> {
    models::install(MODELS, progress)
}

/// How a segmentation network wants its input and gives its output
#[derive(Debug, Clone, Copy)]
pub struct Segmenter {
    /// Side of the square input
    pub size: u32,
    pub mean: [f32; 3],
    pub std: [f32; 3],
    /// The output still needs a sigmoid to become 0..1
    pub logits: bool,
}

/// BiRefNet: ImageNet normalisation, logits out
pub const BIREFNET: Segmenter =
    Segmenter { size: 1024, mean: [0.485, 0.456, 0.406], std: [0.229, 0.224, 0.225], logits: true };
/// ISNet (DIS)
pub const ISNET: Segmenter = Segmenter { size: 1024, mean: [0.5; 3], std: [1.0; 3], logits: false };

/// The last few masks: the preview and the full resolution render often segment the same
/// image, and each run takes a while.
type CachedMask = (Arc<RgbaImage>, Arc<GrayImage>);

static MASKS: Mutex<Vec<CachedMask>> = Mutex::new(Vec::new());

/// Makes the background transparent, or `fill` when given.
pub fn remove_background(image: &RgbaImage, fill: Option<Rgba<u8>>, feedback: &Feedback) -> Result<RgbaImage> {
    let cached = MASKS.lock().unwrap().iter().find(|(input, _)| **input == *image).map(|(_, mask)| mask.clone());
    let mask = match cached {
        Some(mask) => mask,
        None => {
            ensure!(is_installed(), "the background model is not downloaded yet");
            let session = runtime::session(&models::path(&BIREFNET_LITE)?)?;
            let mask = Arc::new(subject_mask(image, &session, BIREFNET, feedback)?);
            let mut masks = MASKS.lock().unwrap();
            masks.insert(0, (Arc::new(image.clone()), mask.clone()));
            masks.truncate(2);
            mask
        }
    };
    Ok(apply_mask(image, &mask, fill))
}

/// How much each pixel belongs to the subject, 0 to 255, at the size of `image`.
pub fn subject_mask(
    image: &RgbaImage,
    session: &SharedSession,
    segmenter: Segmenter,
    feedback: &Feedback,
) -> Result<GrayImage> {
    feedback.check()?;
    let size = segmenter.size as usize;
    let small = resize_exact(image, segmenter.size, segmenter.size);
    let mut input = vec![0.0; 3 * size * size];
    for (x, y, pixel) in small.enumerate_pixels() {
        for c in 0..3 {
            let value = pixel[c] as f32 / 255.0;
            input[c * size * size + y as usize * size + x as usize] = (value - segmenter.mean[c]) / segmenter.std[c];
        }
    }
    let tensor = Tensor::from_array(([1usize, 3, size, size], input))?.into_dyn();
    let outputs = runtime::run(session, tensor, Vec::new())?;
    let mut mask = outputs.into_iter().next().map(|(_, values)| values).unwrap_or_default();
    ensure!(mask.len() == size * size, "segmentation gave {} values", mask.len());
    (feedback.progress)(0.9);
    if segmenter.logits {
        mask.iter_mut().for_each(|v| *v = 1.0 / (1.0 + (-*v).exp()));
    }
    // Stretched to the full range, as the networks rarely reach exactly 0 and 1
    let (low, high) = mask.iter().fold((f32::MAX, f32::MIN), |(lo, hi), &v| (lo.min(v), hi.max(v)));
    let range = (high - low).max(1e-6);
    let small_mask = RgbaImage::from_fn(segmenter.size, segmenter.size, |x, y| {
        let v = ((mask[y as usize * size + x as usize] - low) / range * 255.0).round() as u8;
        Rgba([v, v, v, 255])
    });
    let big = resize_exact(&small_mask, image.width(), image.height());
    Ok(GrayImage::from_fn(image.width(), image.height(), |x, y| Luma([big.get_pixel(x, y)[0]])))
}

/// Keeps the subject: the rest becomes transparent, or `fill`.
pub fn apply_mask(image: &RgbaImage, mask: &GrayImage, fill: Option<Rgba<u8>>) -> RgbaImage {
    RgbaImage::from_fn(image.width(), image.height(), |x, y| {
        let pixel = *image.get_pixel(x, y);
        let keep = mask.get_pixel(x, y)[0] as f32 / 255.0;
        match fill {
            None => Rgba([pixel[0], pixel[1], pixel[2], (pixel[3] as f32 * keep).round() as u8]),
            Some(colour) => {
                // The subject over the colour, the colour over whatever was transparent
                let a = pixel[3] as f32 / 255.0 * keep;
                let mix = |c: usize| (pixel[c] as f32 * a + colour[c] as f32 * (1.0 - a)).round() as u8;
                Rgba([mix(0), mix(1), mix(2), 255])
            }
        }
    })
}

/// Opens a model for `subject_mask`.
pub fn load(path: &Path) -> Result<SharedSession> {
    runtime::session(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_makes_the_background_transparent_or_filled() {
        let image = RgbaImage::from_pixel(2, 1, Rgba([10, 20, 30, 255]));
        let mask = GrayImage::from_raw(2, 1, vec![255, 0]).unwrap();
        let cut = apply_mask(&image, &mask, None);
        assert_eq!(cut.get_pixel(0, 0), &Rgba([10, 20, 30, 255]));
        assert_eq!(cut.get_pixel(1, 0)[3], 0);
        let filled = apply_mask(&image, &mask, Some(Rgba([255, 255, 255, 255])));
        assert_eq!(filled.get_pixel(0, 0), &Rgba([10, 20, 30, 255]));
        assert_eq!(filled.get_pixel(1, 0), &Rgba([255, 255, 255, 255]));
    }

    /// Downloads the model into the real data folder and cuts out the photo at
    /// `GRANDSHOP_BACKGROUND_PHOTO`, writing `cutout.png` next to it:
    /// `cargo test --release background -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn removes_the_background_of_a_photo() {
        install(&|_| {}).unwrap();
        let photo = std::path::PathBuf::from(std::env::var_os("GRANDSHOP_BACKGROUND_PHOTO").unwrap());
        let image = crate::io::load_image(&photo).unwrap();
        let start = std::time::Instant::now();
        let cut = remove_background(&image, None, &Feedback::NONE).unwrap();
        eprintln!("cut out in {:?}", start.elapsed());
        assert_eq!(cut.dimensions(), image.dimensions());
        assert!(cut.pixels().any(|p| p[3] == 0) && cut.pixels().any(|p| p[3] == 255));
        cut.save(photo.with_file_name("cutout.png")).unwrap();
    }
}
