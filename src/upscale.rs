//! AI enlarging with Real-ESRGAN, run on the CPU with candle. Two qualities:
//!
//! - Fast: `realesr-general-x4v3` blended half and half with its weak-denoise twin
//!   `realesr-general-wdn-x4v3`, like Real-ESRGAN's own default. The plain model alone smooths
//!   photos into a waxy, painted look.
//! - Best: `RealESRGAN_x4plus`, a far larger network with more natural detail, but many times
//!   slower.
//!
//! The network weights are not shipped with the app: they are downloaded the first time the
//! user asks for that quality and kept in the app's data folder.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

use anyhow::{bail, ensure, Context, Result};
use candle_core::{DType, Device, Tensor};
use image::{GrayImage, RgbaImage};
use sha2::{Digest, Sha256};

use crate::resample::resize_exact;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Quality {
    Fast,
    Best,
}

/// One downloadable weights file
struct Weights {
    url: &'static str,
    sha256: &'static str,
    file: &'static str,
}

const GENERAL: Weights = Weights {
    url: "https://github.com/xinntao/Real-ESRGAN/releases/download/v0.2.5.0/realesr-general-x4v3.pth",
    sha256: "8dc7edb9ac80ccdc30c3a5dca6616509367f05fbc184ad95b731f05bece96292",
    file: "realesr-general-x4v3.pth",
};
const GENERAL_WDN: Weights = Weights {
    url: "https://github.com/xinntao/Real-ESRGAN/releases/download/v0.2.5.0/realesr-general-wdn-x4v3.pth",
    sha256: "1641f8c4464b9f097c9fdda5589273713f67cf59f3d909e0bd688f0cee269dca",
    file: "realesr-general-wdn-x4v3.pth",
};
const X4PLUS: Weights = Weights {
    url: "https://github.com/xinntao/Real-ESRGAN/releases/download/v0.1.0/RealESRGAN_x4plus.pth",
    sha256: "4fa0d38905f75ac06eb49a7951b426670021be3018265fd191d2125df9d682f1",
    file: "RealESRGAN_x4plus.pth",
};

/// How much `realesr-general-x4v3` weighs against its weak-denoise twin (Real-ESRGAN's default).
/// Lower keeps more of the photo's own texture and noise.
const DENOISE_STRENGTH: f64 = 0.5;

/// The networks always enlarge four times; smaller factors downscale their output
const NETWORK_SCALE: u32 = 4;
/// Longest side of an enlarged image, in original-image pixels
pub const MAX_RESULT_EDGE: u32 = 8000;

impl Quality {
    fn weights(self) -> &'static [Weights] {
        match self {
            Quality::Fast => &[GENERAL, GENERAL_WDN],
            Quality::Best => &[X4PLUS],
        }
    }

    /// Input pixels per tile side, and the extra pixels around each tile so tile edges get
    /// the same context as the inside. Feature maps are 64 floats per pixel, and the large
    /// network keeps them at four times the size, so its tiles are smaller to bound memory.
    fn tiling(self) -> (u32, u32) {
        match self {
            Quality::Fast => (256, 16),
            Quality::Best => (128, 16),
        }
    }
}

/// Folder with the downloaded weights, e.g. `~/.local/share/grandshop/models/` on Linux.
pub fn models_dir() -> Option<PathBuf> {
    directories::ProjectDirs::from("com", "grandshop", "grandshop").map(|dirs| dirs.data_dir().join("models"))
}

pub fn is_installed(quality: Quality) -> bool {
    models_dir().is_some_and(|dir| quality.weights().iter().all(|w| dir.join(w.file).is_file()))
}

/// Downloads and checks the weights for `quality`. Blocks for as long as the download takes.
pub fn install(quality: Quality) -> Result<()> {
    let folder = models_dir().context("no data folder for this user")?;
    std::fs::create_dir_all(&folder).with_context(|| format!("creating {}", folder.display()))?;
    for weights in quality.weights() {
        install_weights(weights, &folder)?;
    }
    Ok(())
}

fn install_weights(weights: &Weights, folder: &Path) -> Result<()> {
    let path = folder.join(weights.file);
    if path.is_file() {
        return Ok(());
    }
    let partial = path.with_extension("part");
    let result = download(weights.url, &partial).and_then(|()| {
        let data = std::fs::read(&partial)?;
        let digest: String = Sha256::digest(&data).iter().map(|b| format!("{b:02x}")).collect();
        ensure!(digest == weights.sha256, "{} has checksum {digest}, expected {}", weights.file, weights.sha256);
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

/// Reads the tensors stored under `key` in a PyTorch checkpoint, as f32.
fn read_tensors(path: &Path, key: &str) -> Result<HashMap<String, Tensor>> {
    candle_core::pickle::read_all_with_key(path, Some(key))
        .with_context(|| format!("reading {}", path.display()))?
        .into_iter()
        .map(|(name, tensor)| Ok((name, tensor.to_dtype(DType::F32)?)))
        .collect()
}

fn leaky_relu(x: &Tensor) -> Result<Tensor> {
    Ok(x.maximum(&x.affine(0.2, 0.0)?)?)
}

/// A 3×3 convolution that keeps the size
struct Conv {
    weight: Tensor,
    bias: Tensor,
}

impl Conv {
    fn load(tensors: &HashMap<String, Tensor>, name: &str) -> Result<Self> {
        let get = |part: &str| tensors.get(&format!("{name}.{part}")).with_context(|| format!("model has no {name}.{part}"));
        Ok(Self { weight: get("weight")?.clone(), bias: get("bias")?.reshape((1, (), 1, 1))? })
    }

    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        Ok(x.conv2d(&self.weight, 1, 1, 1, 1)?.broadcast_add(&self.bias)?)
    }
}

/// SRVGGNetCompact: a plain stack of convolutions with a PReLU after all but the last, a pixel
/// shuffle to four times the size, plus the input enlarged with nearest neighbour.
struct Compact {
    layers: Vec<(Conv, Option<Tensor>)>,
}

impl Compact {
    /// Loads `general` and `wdn` and mixes their weights: `strength` of the first.
    fn load_blended(general: &Path, wdn: &Path, strength: f64) -> Result<Self> {
        let a = read_tensors(general, "params")?;
        let b = read_tensors(wdn, "params")?;
        let tensors = a
            .into_iter()
            .map(|(name, a)| {
                let b = b.get(&name).with_context(|| format!("{} has no {name}", wdn.display()))?;
                Ok((name, (a.affine(strength, 0.0)? + b.affine(1.0 - strength, 0.0)?)?))
            })
            .collect::<Result<HashMap<_, _>>>()?;
        // body.0 conv, body.1 prelu, body.2 conv, ..., ending with a conv
        let mut layers = Vec::new();
        let mut index = 0;
        while tensors.contains_key(&format!("body.{index}.weight")) {
            let conv = Conv::load(&tensors, &format!("body.{index}"))?;
            let prelu = tensors.get(&format!("body.{}.weight", index + 1)).filter(|t| t.rank() == 1);
            let prelu = prelu.map(|t| t.reshape((1, (), 1, 1))).transpose()?;
            index += if prelu.is_some() { 2 } else { 1 };
            layers.push((conv, prelu));
        }
        ensure!(layers.len() > 1, "{} is not an SRVGGNetCompact model", general.display());
        Ok(Self { layers })
    }

    fn forward(&self, input: &Tensor) -> Result<Tensor> {
        let mut x = input.clone();
        for (conv, prelu) in &self.layers {
            x = conv.forward(&x)?;
            if let Some(slope) = prelu {
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
        Ok((shuffled + base)?)
    }
}

/// Five densely connected convolutions: each sees the input plus all earlier outputs.
struct DenseBlock {
    convs: Vec<Conv>,
}

impl DenseBlock {
    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let mut features = x.clone();
        for conv in &self.convs[..4] {
            let out = leaky_relu(&conv.forward(&features)?)?;
            features = Tensor::cat(&[&features, &out], 1)?;
        }
        let out = self.convs[4].forward(&features)?;
        Ok((out.affine(0.2, 0.0)? + x)?)
    }
}

/// RRDBNet (ESRGAN): 23 residual-in-residual dense blocks at the input size, then two
/// nearest neighbour doublings, each followed by a convolution.
struct Rrdb {
    first: Conv,
    /// Three dense blocks each
    blocks: Vec<[DenseBlock; 3]>,
    body: Conv,
    up1: Conv,
    up2: Conv,
    hr: Conv,
    last: Conv,
}

impl Rrdb {
    fn load(path: &Path) -> Result<Self> {
        let tensors = read_tensors(path, "params_ema")?;
        let conv = |name: &str| Conv::load(&tensors, name);
        let dense = |name: String| -> Result<DenseBlock> {
            Ok(DenseBlock { convs: (1..=5).map(|i| conv(&format!("{name}.conv{i}"))).collect::<Result<_>>()? })
        };
        let mut blocks = Vec::new();
        while tensors.contains_key(&format!("body.{}.rdb1.conv1.weight", blocks.len())) {
            let name = format!("body.{}", blocks.len());
            blocks.push([dense(format!("{name}.rdb1"))?, dense(format!("{name}.rdb2"))?, dense(format!("{name}.rdb3"))?]);
        }
        ensure!(!blocks.is_empty(), "{} is not an RRDBNet model", path.display());
        Ok(Self {
            first: conv("conv_first")?,
            blocks,
            body: conv("conv_body")?,
            up1: conv("conv_up1")?,
            up2: conv("conv_up2")?,
            hr: conv("conv_hr")?,
            last: conv("conv_last")?,
        })
    }

    fn forward(&self, input: &Tensor) -> Result<Tensor> {
        let features = self.first.forward(input)?;
        let mut x = features.clone();
        for [a, b, c] in &self.blocks {
            let out = c.forward(&b.forward(&a.forward(&x)?)?)?;
            x = (out.affine(0.2, 0.0)? + x)?;
        }
        let x = (features + self.body.forward(&x)?)?;
        let (_, _, height, width) = x.dims4()?;
        let x = leaky_relu(&self.up1.forward(&x.upsample_nearest2d(height * 2, width * 2)?)?)?;
        let x = leaky_relu(&self.up2.forward(&x.upsample_nearest2d(height * 4, width * 4)?)?)?;
        self.last.forward(&leaky_relu(&self.hr.forward(&x)?)?)
    }
}

enum Network {
    Compact(Compact),
    Rrdb(Rrdb),
}

pub struct Model {
    network: Network,
    tile: u32,
    tile_pad: u32,
}

impl Model {
    /// Loads the weights for `quality` from `folder`.
    pub fn load(quality: Quality, folder: &Path) -> Result<Self> {
        let network = match quality {
            Quality::Fast => Network::Compact(Compact::load_blended(
                &folder.join(GENERAL.file),
                &folder.join(GENERAL_WDN.file),
                DENOISE_STRENGTH,
            )?),
            Quality::Best => Network::Rrdb(Rrdb::load(&folder.join(X4PLUS.file))?),
        };
        let (tile, tile_pad) = quality.tiling();
        Ok(Self { network, tile, tile_pad })
    }

    /// Enlarges an RGB tile four times. Input and output are `(1, 3, h, w)` in 0..1.
    fn forward(&self, input: &Tensor) -> Result<Tensor> {
        let output = match &self.network {
            Network::Compact(network) => network.forward(input)?,
            Network::Rrdb(network) => network.forward(input)?,
        };
        Ok(output.clamp(0.0, 1.0)?)
    }

    /// Enlarges four times, tile by tile. Alpha is enlarged with Lanczos.
    pub fn upscale_x4(&self, image: &RgbaImage) -> Result<RgbaImage> {
        let (width, height) = image.dimensions();
        let s = NETWORK_SCALE;
        let (tile, pad) = (self.tile, self.tile_pad);
        let mut out = RgbaImage::new(width * s, height * s);
        for tile_y in (0..height).step_by(tile as usize) {
            for tile_x in (0..width).step_by(tile as usize) {
                let x0 = tile_x.saturating_sub(pad);
                let y0 = tile_y.saturating_sub(pad);
                let x1 = (tile_x + tile + pad).min(width);
                let y1 = (tile_y + tile + pad).min(height);
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
                let keep_w = tile.min(width - tile_x);
                let keep_h = tile.min(height - tile_y);
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

/// The loaded models, plus the last few results: the preview and the full resolution render
/// often enlarge the same image, and each run takes seconds.
#[derive(Default)]
struct Cache {
    models: Vec<(Quality, Arc<Model>)>,
    results: Vec<(Arc<RgbaImage>, u32, Quality, Arc<RgbaImage>)>,
}

const CACHED_RESULTS: usize = 2;

static CACHE: Mutex<Cache> = Mutex::new(Cache { models: Vec::new(), results: Vec::new() });

fn model(quality: Quality) -> Result<Arc<Model>> {
    let mut cache = CACHE.lock().unwrap();
    if let Some((_, model)) = cache.models.iter().find(|(q, _)| *q == quality) {
        return Ok(model.clone());
    }
    ensure!(is_installed(quality), "the enlarging model is not downloaded yet");
    let folder = models_dir().context("no data folder for this user")?;
    let model = Arc::new(Model::load(quality, &folder)?);
    cache.models.push((quality, model.clone()));
    Ok(model)
}

/// Enlarges `factor` times (2 to 4) with the downloaded model for `quality`.
pub fn upscale(image: &RgbaImage, factor: u32, quality: Quality) -> Result<RgbaImage> {
    ensure!((2..=NETWORK_SCALE).contains(&factor), "can only enlarge 2 to {NETWORK_SCALE} times, not {factor}");
    {
        let cache = CACHE.lock().unwrap();
        let cached = cache.results.iter().find(|(input, f, q, _)| *f == factor && *q == quality && **input == *image);
        if let Some((_, _, _, result)) = cached {
            return Ok((**result).clone());
        }
    }
    let model = model(quality)?;
    let enlarged = model.upscale_x4(image)?;
    let (width, height) = image.dimensions();
    let result = if factor == NETWORK_SCALE {
        enlarged
    } else {
        resize_exact(&enlarged, width * factor, height * factor)
    };
    let mut cache = CACHE.lock().unwrap();
    cache.results.insert(0, (Arc::new(image.clone()), factor, quality, Arc::new(result.clone())));
    cache.results.truncate(CACHED_RESULTS);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Downloads into the real data folder: `cargo test upscale -- --ignored`
    #[test]
    #[ignore]
    fn downloads_and_verifies_the_models() {
        for quality in [Quality::Fast, Quality::Best] {
            install(quality).unwrap();
            assert!(is_installed(quality));
            let image = RgbaImage::from_pixel(20, 10, image::Rgba([10, 200, 30, 128]));
            let big = upscale(&image, 3, quality).unwrap();
            assert_eq!(big.dimensions(), (60, 30));
            assert!(big.pixels().all(|p| p[3].abs_diff(128) <= 2));
        }
    }

    /// Needs the weights, downloaded or in `GRANDSHOP_MODELS`:
    /// `cargo test --release upscale -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn enlarges_the_fixture() {
        let folder = std::env::var_os("GRANDSHOP_MODELS").map(PathBuf::from).or_else(models_dir).unwrap();
        let photo = crate::io::load_image(Path::new("tests/fixtures/paint.jpg")).unwrap();
        let small = crate::resample::resize_exact(&photo, 300, 200);
        // Close to a plain enlargement overall, just sharper
        let plain = crate::resample::resize_exact(&small, 1200, 800);
        for (quality, name) in [(Quality::Fast, "fast"), (Quality::Best, "best")] {
            let model = Model::load(quality, &folder).unwrap();
            let start = std::time::Instant::now();
            let big = model.upscale_x4(&small).unwrap();
            eprintln!("{name}: 300×200 → {:?} in {:?}", big.dimensions(), start.elapsed());
            assert_eq!(big.dimensions(), (1200, 800));
            let diff: u64 = big.as_raw().iter().zip(plain.as_raw()).map(|(a, b)| a.abs_diff(*b) as u64).sum();
            let mean = diff as f64 / big.as_raw().len() as f64;
            assert!(mean < 12.0, "{name}: mean difference {mean}");
            if let Ok(out) = std::env::var("GRANDSHOP_UPSCALE_OUT") {
                big.save(format!("{out}/{name}.png")).unwrap();
            }
        }
        if let Ok(out) = std::env::var("GRANDSHOP_UPSCALE_OUT") {
            plain.save(format!("{out}/lanczos.png")).unwrap();
        }
    }
}
