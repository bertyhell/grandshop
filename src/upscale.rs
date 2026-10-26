//! AI enlarging with Real-ESRGAN (`realesr-general-x4v3`), run on the CPU with candle.
//!
//! The network weights are not shipped with the app: they are downloaded the first time
//! the user adds an "Enlarge" card and kept in the app's data folder.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

use anyhow::{bail, ensure, Context, Result};
use candle_core::{DType, Device, Tensor};
use image::{GrayImage, RgbaImage};
use sha2::{Digest, Sha256};

use crate::resample::resize_exact;

const MODEL_URL: &str =
    "https://github.com/xinntao/Real-ESRGAN/releases/download/v0.2.5.0/realesr-general-x4v3.pth";
const MODEL_SHA256: &str = "8dc7edb9ac80ccdc30c3a5dca6616509367f05fbc184ad95b731f05bece96292";
const MODEL_FILE: &str = "realesr-general-x4v3.pth";
/// Shown to the user before downloading
pub const MODEL_SIZE_MB: u32 = 5;

/// The network always enlarges four times; smaller factors downscale its output
const NETWORK_SCALE: u32 = 4;
/// Feature maps are 64 floats per pixel, so the image goes through in tiles to bound memory
const TILE: u32 = 256;
/// Extra pixels around each tile, so tile edges get the same context as the inside
const TILE_PAD: u32 = 16;
/// Longest side of an enlarged image, in original-image pixels
pub const MAX_RESULT_EDGE: u32 = 8000;

/// Location of the downloaded weights, e.g. `~/.local/share/grandshop/models/` on Linux.
pub fn model_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("com", "grandshop", "grandshop")
        .map(|dirs| dirs.data_dir().join("models").join(MODEL_FILE))
}

pub fn is_installed() -> bool {
    model_path().is_some_and(|path| path.is_file())
}

/// Downloads and checks the weights. Blocks for as long as the download takes.
pub fn install() -> Result<()> {
    let path = model_path().context("no data folder for this user")?;
    if path.is_file() {
        return Ok(());
    }
    let folder = path.parent().expect("model path has a folder");
    std::fs::create_dir_all(folder).with_context(|| format!("creating {}", folder.display()))?;
    let partial = path.with_extension("part");
    let result = download(MODEL_URL, &partial).and_then(|()| {
        let data = std::fs::read(&partial)?;
        let digest: String = Sha256::digest(&data).iter().map(|b| format!("{b:02x}")).collect();
        ensure!(digest == MODEL_SHA256, "downloaded model has checksum {digest}, expected {MODEL_SHA256}");
        std::fs::rename(&partial, &path).with_context(|| format!("moving the model to {}", path.display()))
    });
    if result.is_err() {
        let _ = std::fs::remove_file(&partial);
    }
    result
}

/// Uses the system's `curl` (part of Windows 10 and later, macOS and most Linux systems),
/// which keeps an HTTPS client out of the app.
fn download(url: &str, target: &Path) -> Result<()> {
    let mut command = Command::new("curl");
    command.args(["--fail", "--silent", "--show-error", "--location", "--retry", "2", "--output"]);
    command.arg(target).arg(url);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // No console window flashing up from the windowed app
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let output = command.output().context("running curl")?;
    if !output.status.success() {
        bail!("downloading {url} failed: {}", String::from_utf8_lossy(&output.stderr).trim());
    }
    Ok(())
}

/// One 3×3 convolution, followed by a PReLU on every layer but the last.
struct Layer {
    weight: Tensor,
    bias: Tensor,
    prelu: Option<Tensor>,
}

/// SRVGGNetCompact: a plain stack of convolutions at the input resolution, a pixel shuffle
/// to four times the size, plus the input enlarged with nearest neighbour.
pub struct Model {
    layers: Vec<Layer>,
}

impl Model {
    pub fn load(path: &Path) -> Result<Self> {
        let tensors: std::collections::HashMap<String, Tensor> =
            candle_core::pickle::read_all_with_key(path, Some("params"))
                .with_context(|| format!("reading {}", path.display()))?
                .into_iter()
                .collect();
        let get = |name: String| {
            let tensor = tensors.get(&name).with_context(|| format!("model has no {name}"))?;
            Ok::<_, anyhow::Error>(tensor.to_dtype(DType::F32)?)
        };
        // body.0 conv, body.1 prelu, body.2 conv, ..., ending with a conv
        let mut layers = Vec::new();
        let mut index = 0;
        while tensors.contains_key(&format!("body.{index}.weight")) {
            let weight = get(format!("body.{index}.weight"))?;
            let bias = get(format!("body.{index}.bias"))?.reshape((1, (), 1, 1))?;
            let prelu_name = format!("body.{}.weight", index + 1);
            let is_prelu = tensors.get(&prelu_name).is_some_and(|t| t.rank() == 1);
            let prelu = if is_prelu { Some(get(prelu_name)?.reshape((1, (), 1, 1))?) } else { None };
            index += if is_prelu { 2 } else { 1 };
            layers.push(Layer { weight, bias, prelu });
        }
        ensure!(layers.len() > 1, "{} is not an SRVGGNetCompact model", path.display());
        Ok(Self { layers })
    }

    /// Enlarges an RGB tile four times. Input and output are `(1, 3, h, w)` in 0..1.
    fn forward(&self, input: &Tensor) -> Result<Tensor> {
        let mut x = input.clone();
        for layer in &self.layers {
            x = x.conv2d(&layer.weight, 1, 1, 1, 1)?.broadcast_add(&layer.bias)?;
            if let Some(slope) = &layer.prelu {
                x = (x.relu()? + x.minimum(0.0)?.broadcast_mul(slope)?)?;
            }
        }
        let (batch, channels, height, width) = x.dims4()?;
        let s = NETWORK_SCALE as usize;
        let out_channels = channels / (s * s);
        let shuffled = x
            .reshape((batch, out_channels, s, s, height, width))?
            .permute((0, 1, 4, 2, 5, 3))?
            .reshape((batch, out_channels, height * s, width * s))?;
        let base = input.upsample_nearest2d(height * s, width * s)?;
        Ok((shuffled + base)?.clamp(0.0, 1.0)?)
    }

    /// Enlarges four times, tile by tile. Alpha is enlarged with Lanczos.
    pub fn upscale_x4(&self, image: &RgbaImage) -> Result<RgbaImage> {
        let (width, height) = image.dimensions();
        let s = NETWORK_SCALE;
        let mut out = RgbaImage::new(width * s, height * s);
        for tile_y in (0..height).step_by(TILE as usize) {
            for tile_x in (0..width).step_by(TILE as usize) {
                let x0 = tile_x.saturating_sub(TILE_PAD);
                let y0 = tile_y.saturating_sub(TILE_PAD);
                let x1 = (tile_x + TILE + TILE_PAD).min(width);
                let y1 = (tile_y + TILE + TILE_PAD).min(height);
                let (w, h) = (x1 - x0, y1 - y0);
                let mut rgb = Vec::with_capacity((3 * w * h) as usize);
                for channel in 0..3 {
                    for y in y0..y1 {
                        rgb.extend((x0..x1).map(|x| image.get_pixel(x, y)[channel] as f32 / 255.0));
                    }
                }
                let input = Tensor::from_vec(rgb, (1, 3, h as usize, w as usize), &Device::Cpu)?;
                let result = self.forward(&input)?.squeeze(0)?.to_vec3::<f32>()?;
                // Copy back only the tile itself, not the padding
                let keep_w = TILE.min(width - tile_x);
                let keep_h = TILE.min(height - tile_y);
                for y in 0..keep_h * s {
                    for x in 0..keep_w * s {
                        let (sx, sy) = (((tile_x - x0) * s + x) as usize, ((tile_y - y0) * s + y) as usize);
                        let pixel = out.get_pixel_mut(tile_x * s + x, tile_y * s + y);
                        for channel in 0..3 {
                            pixel[channel] = (result[channel][sy][sx] * 255.0).round() as u8;
                        }
                    }
                }
            }
        }
        let alpha = GrayImage::from_fn(width, height, |x, y| image::Luma([image.get_pixel(x, y)[3]]));
        if alpha.pixels().all(|p| p[0] == 255) {
            out.pixels_mut().for_each(|p| p[3] = 255);
        } else {
            let alpha_rgba = RgbaImage::from_fn(width, height, |x, y| {
                let a = alpha.get_pixel(x, y)[0];
                image::Rgba([a, a, a, 255])
            });
            let alpha_big = resize_exact(&alpha_rgba, width * s, height * s);
            for (pixel, a) in out.pixels_mut().zip(alpha_big.pixels()) {
                pixel[3] = a[0];
            }
        }
        Ok(out)
    }
}

/// The loaded model, plus the last few results: the preview and the full resolution render
/// often enlarge the same image, and each run takes seconds.
#[derive(Default)]
struct Cache {
    model: Option<Arc<Model>>,
    results: Vec<(Arc<RgbaImage>, u32, Arc<RgbaImage>)>,
}

const CACHED_RESULTS: usize = 2;

static CACHE: Mutex<Cache> = Mutex::new(Cache { model: None, results: Vec::new() });

fn model() -> Result<Arc<Model>> {
    let mut cache = CACHE.lock().unwrap();
    if let Some(model) = &cache.model {
        return Ok(model.clone());
    }
    let path = model_path().filter(|path| path.is_file()).context("the enlarging model is not downloaded yet")?;
    let model = Arc::new(Model::load(&path)?);
    cache.model = Some(model.clone());
    Ok(model)
}

/// Enlarges `factor` times (2 to 4) with the downloaded model.
pub fn upscale(image: &RgbaImage, factor: u32) -> Result<RgbaImage> {
    ensure!((2..=NETWORK_SCALE).contains(&factor), "can only enlarge 2 to {NETWORK_SCALE} times, not {factor}");
    {
        let cache = CACHE.lock().unwrap();
        if let Some((_, _, result)) = cache.results.iter().find(|(input, f, _)| *f == factor && **input == *image) {
            return Ok((**result).clone());
        }
    }
    let model = model()?;
    let enlarged = model.upscale_x4(image)?;
    let (width, height) = image.dimensions();
    let result = if factor == NETWORK_SCALE {
        enlarged
    } else {
        resize_exact(&enlarged, width * factor, height * factor)
    };
    let mut cache = CACHE.lock().unwrap();
    cache.results.insert(0, (Arc::new(image.clone()), factor, Arc::new(result.clone())));
    cache.results.truncate(CACHED_RESULTS);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Needs the weights: `GRANDSHOP_UPSCALE_MODEL=path/to/realesr-general-x4v3.pth cargo test
    /// --release upscale -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn enlarges_the_fixture() {
        let path = std::env::var("GRANDSHOP_UPSCALE_MODEL").expect("GRANDSHOP_UPSCALE_MODEL");
        let model = Model::load(Path::new(&path)).unwrap();
        let photo = crate::io::load_image(Path::new("tests/fixtures/paint.jpg")).unwrap();
        let small = crate::resample::resize_exact(&photo, 300, 200);
        let start = std::time::Instant::now();
        let big = model.upscale_x4(&small).unwrap();
        eprintln!("300×200 → {:?} in {:?}", big.dimensions(), start.elapsed());
        assert_eq!(big.dimensions(), (1200, 800));
        // Close to a plain enlargement overall, just sharper
        let plain = crate::resample::resize_exact(&small, 1200, 800);
        let diff: u64 = big.as_raw().iter().zip(plain.as_raw()).map(|(a, b)| a.abs_diff(*b) as u64).sum();
        let mean = diff as f64 / big.as_raw().len() as f64;
        assert!(mean < 12.0, "mean difference {mean}");
        if let Ok(out) = std::env::var("GRANDSHOP_UPSCALE_OUT") {
            big.save(format!("{out}/ai.png")).unwrap();
            plain.save(format!("{out}/lanczos.png")).unwrap();
        }
    }
}
