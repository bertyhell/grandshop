//! "Improve faces": finds the faces with SCRFD, turns each upright into a 512×512 crop,
//! restores it with GFPGAN and blends it back over the original.
//!
//! The models are downloaded the first time the operation is used (see `ai::models`).

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

const SCRFD: ModelFile = ModelFile {
    file: "scrfd_2.5g.onnx",
    url: facefusion!("scrfd_2.5g.onnx"),
    sha256: "2c07342347cef21a619c49dd5664fb8c09570ae9eda5bff3e385c11eafc45ada",
    bytes: 3_295_067,
    licence: "SCRFD 2.5G by InsightFace: non-commercial research use",
};
const GFPGAN: ModelFile = ModelFile {
    file: "gfpgan_1.4.onnx",
    url: facefusion!("gfpgan_1.4.onnx"),
    sha256: "accc4757b26bdb89b32b4d3500d4f79c9dff97c1dd7c7104bf9dcb95e3311385",
    bytes: 340_299_087,
    licence: "GFPGAN 1.4 by Tencent ARC: Apache-2.0",
};

/// Everything the operation downloads
pub const MODELS: &[&ModelFile] = &[&SCRFD, &GFPGAN];

pub fn is_installed() -> bool {
    models::is_installed(MODELS)
}

/// Downloads the models, reporting the fraction done. Blocks for as long as it takes.
pub fn install(progress: &dyn Fn(f32)) -> Result<()> {
    models::install(MODELS, progress)
}

/// Side of the square the restorer works on
const CROP: usize = 512;

/// Where the eyes, nose tip and mouth corners go in the crop (the FFHQ alignment), as fractions
const TEMPLATE: [[f32; 2]; 5] = [
    [0.376_916_8, 0.468_646_6],
    [0.622_857, 0.469_128_1],
    [0.501_238_6, 0.613_319],
    [0.393_088_2, 0.725_411],
    [0.611_502, 0.724_904_6],
];

/// A face found in the image, in image pixels
#[derive(Debug, Clone, PartialEq)]
pub struct Face {
    pub score: f32,
    /// Left, top, right, bottom
    pub bbox: [f32; 4],
    /// Eyes, nose tip and mouth corners, as seen in the image (left eye first)
    pub landmarks: [[f32; 2]; 5],
}

/// The last few results: the preview and the full resolution render often improve the same
/// image, and each run takes a while.
type CachedResult = (Arc<RgbaImage>, u32, Arc<RgbaImage>);

static RESULTS: Mutex<Vec<CachedResult>> = Mutex::new(Vec::new());

/// Restores every face in `image`. `strength` (0 to 1) blends between the original and the
/// fully restored face.
pub fn improve_faces(image: &RgbaImage, strength: f32, feedback: &Feedback) -> Result<RgbaImage> {
    let key = (strength * 1000.0).round() as u32;
    {
        let results = RESULTS.lock().unwrap();
        if let Some((_, _, result)) = results.iter().find(|(input, k, _)| *k == key && **input == *image) {
            return Ok((**result).clone());
        }
    }
    ensure!(is_installed(), "the face model is not downloaded yet");
    let detector = runtime::session(&models::path(&SCRFD)?)?;
    let restorer = runtime::session(&models::path(&GFPGAN)?)?;
    let result = improve_with(image, &detector, &restorer, strength, feedback)?;
    let mut results = RESULTS.lock().unwrap();
    results.insert(0, (Arc::new(image.clone()), key, Arc::new(result.clone())));
    results.truncate(2);
    Ok(result)
}

/// `improve_faces` with the given models, for trying other restorers.
pub fn improve_with(
    image: &RgbaImage,
    detector: &SharedSession,
    restorer: &SharedSession,
    strength: f32,
    feedback: &Feedback,
) -> Result<RgbaImage> {
    let faces = detect(image, detector, feedback)?;
    let mut out = image.clone();
    for (index, face) in faces.iter().enumerate() {
        feedback.check()?;
        let to_crop = similarity(&face.landmarks, &TEMPLATE.map(|[x, y]| [x * CROP as f32, y * CROP as f32]));
        let crop = warp_crop(image, &to_crop);
        let restored = keep_colours(&crop, &restore(&crop, restorer)?);
        paste(&mut out, &restored, &to_crop, strength);
        (feedback.progress)((index + 1) as f32 / faces.len() as f32);
    }
    Ok(out)
}

// Detection

/// The detector's input square
const DETECT_SIZE: u32 = 640;
/// Large images are also searched in overlapping windows of this size, so small faces in group
/// photos are found too
const WINDOW: u32 = 960;
const MIN_SCORE: f32 = 0.5;

pub fn detect(image: &RgbaImage, detector: &SharedSession, feedback: &Feedback) -> Result<Vec<Face>> {
    let (width, height) = image.dimensions();
    let mut windows = vec![[0, 0, width, height]];
    if width.max(height) > WINDOW * 5 / 4 {
        for y in window_starts(height) {
            for x in window_starts(width) {
                windows.push([x, y, WINDOW.min(width), WINDOW.min(height)]);
            }
        }
    }
    let mut faces = Vec::new();
    for window in windows {
        feedback.check()?;
        faces.extend(detect_in(image, window, detector)?);
    }
    Ok(non_maximum_suppression(faces, 0.4))
}

/// Left edges of windows that cover `length` with some overlap
fn window_starts(length: u32) -> Vec<u32> {
    if length <= WINDOW {
        return vec![0];
    }
    let overlap = WINDOW / 5;
    let count = (length - overlap).div_ceil(WINDOW - overlap);
    (0..count).map(|i| ((length - WINDOW) as u64 * i as u64 / (count - 1).max(1) as u64) as u32).collect()
}

/// Runs SCRFD on one part of the image, `[x, y, width, height]`.
fn detect_in(image: &RgbaImage, window: [u32; 4], detector: &SharedSession) -> Result<Vec<Face>> {
    let [wx, wy, ww, wh] = window;
    let part = image::imageops::crop_imm(image, wx, wy, ww, wh).to_image();
    let scale = DETECT_SIZE as f32 / ww.max(wh) as f32;
    let (sw, sh) = (((ww as f32 * scale).round() as u32).max(1), ((wh as f32 * scale).round() as u32).max(1));
    let small = resize_exact(&part, sw, sh);
    // Normalised RGB, with the unused bottom or right part black
    let size = DETECT_SIZE as usize;
    let mut input = vec![-127.5 / 128.0; 3 * size * size];
    for (x, y, pixel) in small.enumerate_pixels() {
        for channel in 0..3 {
            input[channel * size * size + y as usize * size + x as usize] = (pixel[channel] as f32 - 127.5) / 128.0;
        }
    }
    let tensor = Tensor::from_array(([1usize, 3, size, size], input))?.into_dyn();
    let outputs = runtime::run(detector, tensor, Vec::new())?;
    ensure!(outputs.len() == 9, "face detector gave {} outputs instead of 9", outputs.len());

    let mut faces = Vec::new();
    for (level, stride) in [8usize, 16, 32].into_iter().enumerate() {
        let columns = size / stride;
        let (scores, boxes, points) = (&outputs[level].1, &outputs[level + 3].1, &outputs[level + 6].1);
        for (index, &score) in scores.iter().enumerate() {
            if score < MIN_SCORE {
                continue;
            }
            // Two anchors per location
            let location = index / 2;
            let cx = ((location % columns) * stride) as f32;
            let cy = ((location / columns) * stride) as f32;
            let s = stride as f32;
            // Back from detector pixels to image pixels
            let map = |x: f32, y: f32| [x / scale + wx as f32, y / scale + wy as f32];
            let b = &boxes[index * 4..index * 4 + 4];
            let [left, top] = map(cx - b[0] * s, cy - b[1] * s);
            let [right, bottom] = map(cx + b[2] * s, cy + b[3] * s);
            let k = &points[index * 10..index * 10 + 10];
            let landmarks = std::array::from_fn(|i| map(cx + k[2 * i] * s, cy + k[2 * i + 1] * s));
            faces.push(Face { score, bbox: [left, top, right, bottom], landmarks });
        }
    }
    Ok(faces)
}

fn non_maximum_suppression(mut faces: Vec<Face>, max_overlap: f32) -> Vec<Face> {
    faces.sort_by(|a, b| b.score.total_cmp(&a.score));
    let mut kept: Vec<Face> = Vec::new();
    for face in faces {
        if kept.iter().all(|k| intersection_over_union(&k.bbox, &face.bbox) <= max_overlap) {
            kept.push(face);
        }
    }
    kept
}

fn intersection_over_union(a: &[f32; 4], b: &[f32; 4]) -> f32 {
    let width = (a[2].min(b[2]) - a[0].max(b[0])).max(0.0);
    let height = (a[3].min(b[3]) - a[1].max(b[1])).max(0.0);
    let intersection = width * height;
    let area = |r: &[f32; 4]| (r[2] - r[0]).max(0.0) * (r[3] - r[1]).max(0.0);
    intersection / (area(a) + area(b) - intersection).max(f32::EPSILON)
}

// Alignment

/// Turn, uniform scale and shift: `x' = a·x − b·y + tx`, `y' = b·x + a·y + ty`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Similarity {
    a: f32,
    b: f32,
    tx: f32,
    ty: f32,
}

impl Similarity {
    fn apply(&self, [x, y]: [f32; 2]) -> [f32; 2] {
        [self.a * x - self.b * y + self.tx, self.b * x + self.a * y + self.ty]
    }

    fn inverse(&self) -> Self {
        let d = self.a * self.a + self.b * self.b;
        let (a, b) = (self.a / d, -self.b / d);
        Self { a, b, tx: -(a * self.tx - b * self.ty), ty: -(b * self.tx + a * self.ty) }
    }
}

/// The similarity that best maps `from` onto `to` (least squares).
pub fn similarity(from: &[[f32; 2]; 5], to: &[[f32; 2]; 5]) -> Similarity {
    let mean = |points: &[[f32; 2]; 5]| {
        let [sx, sy] = points.iter().fold([0.0, 0.0], |[x, y], p| [x + p[0], y + p[1]]);
        [sx / 5.0, sy / 5.0]
    };
    let (mf, mt) = (mean(from), mean(to));
    let (mut dot, mut cross, mut norm) = (0.0, 0.0, 0.0);
    for (f, t) in from.iter().zip(to) {
        let (fx, fy, tx, ty) = (f[0] - mf[0], f[1] - mf[1], t[0] - mt[0], t[1] - mt[1]);
        dot += fx * tx + fy * ty;
        cross += fx * ty - fy * tx;
        norm += fx * fx + fy * fy;
    }
    let (a, b) = (dot / norm.max(f32::EPSILON), cross / norm.max(f32::EPSILON));
    Similarity { a, b, tx: mt[0] - (a * mf[0] - b * mf[1]), ty: mt[1] - (b * mf[0] + a * mf[1]) }
}

/// Bilinear sample, with coordinates outside the image clamped to its edge
fn sample(image: &RgbaImage, x: f32, y: f32) -> [f32; 4] {
    let (w, h) = (image.width() as f32, image.height() as f32);
    let x = x.clamp(0.0, w - 1.0);
    let y = y.clamp(0.0, h - 1.0);
    let (x0, y0) = (x.floor() as u32, y.floor() as u32);
    let (x1, y1) = ((x0 + 1).min(image.width() - 1), (y0 + 1).min(image.height() - 1));
    let (fx, fy) = (x - x0 as f32, y - y0 as f32);
    let p = |x, y| image.get_pixel(x, y).0.map(f32::from);
    let (a, b, c, d) = (p(x0, y0), p(x1, y0), p(x0, y1), p(x1, y1));
    std::array::from_fn(|i| {
        let top = a[i] + (b[i] - a[i]) * fx;
        let bottom = c[i] + (d[i] - c[i]) * fx;
        top + (bottom - top) * fy
    })
}

/// The upright face crop: crop pixel (u, v) comes from image point `to_crop⁻¹(u, v)`.
fn warp_crop(image: &RgbaImage, to_crop: &Similarity) -> RgbaImage {
    let from_crop = to_crop.inverse();
    RgbaImage::from_fn(CROP as u32, CROP as u32, |u, v| {
        let [x, y] = from_crop.apply([u as f32 + 0.5, v as f32 + 0.5]);
        Rgba(sample(image, x - 0.5, y - 0.5).map(|c| c.round() as u8))
    })
}

/// CodeFormer's balance between following the input face (1) and inventing detail (0)
const CODEFORMER_FIDELITY: f64 = 0.7;

/// Runs GFPGAN-style restorers: RGB in and out, scaled to −1..1.
fn restore(crop: &RgbaImage, restorer: &SharedSession) -> Result<RgbaImage> {
    let mut input = vec![0.0; 3 * CROP * CROP];
    for (x, y, pixel) in crop.enumerate_pixels() {
        for channel in 0..3 {
            input[channel * CROP * CROP + y as usize * CROP + x as usize] = pixel[channel] as f32 / 127.5 - 1.0;
        }
    }
    let tensor = Tensor::from_array(([1usize, 3, CROP, CROP], input))?.into_dyn();
    // CodeFormer also takes how closely to follow the input face
    let has_weight = restorer.lock().unwrap().inputs().iter().any(|input| input.name() == "weight");
    let extra = if has_weight {
        vec![("weight", Tensor::from_array(([0usize; 0], vec![CODEFORMER_FIDELITY]))?.into_dyn())]
    } else {
        Vec::new()
    };
    let outputs = runtime::run(restorer, tensor, extra)?;
    let output = &outputs[0].1;
    ensure!(output.len() == 3 * CROP * CROP, "face restorer gave {} values", output.len());
    Ok(RgbaImage::from_fn(CROP as u32, CROP as u32, |x, y| {
        let index = y as usize * CROP + x as usize;
        let channel = |c: usize| ((output[c * CROP * CROP + index].clamp(-1.0, 1.0) + 1.0) * 127.5).round() as u8;
        Rgba([channel(0), channel(1), channel(2), 255])
    }))
}

/// The lightness of `restored` with the colours of `original`. The restorer adds skin tones,
/// which would tint black-and-white and sepia photos; the detail is in the lightness anyway.
fn keep_colours(original: &RgbaImage, restored: &RgbaImage) -> RgbaImage {
    let luma = |p: &Rgba<u8>| 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32;
    RgbaImage::from_fn(restored.width(), restored.height(), |x, y| {
        let source = original.get_pixel(x, y);
        let y_new = luma(restored.get_pixel(x, y));
        // Same colour differences, new lightness
        let shift = y_new - luma(source);
        let channel = |c: usize| (source[c] as f32 + shift).round().clamp(0.0, 255.0) as u8;
        Rgba([channel(0), channel(1), channel(2), 255])
    })
}

/// Width of the soft edge of the pasted crop, in crop pixels
const FEATHER: f32 = 48.0;

/// Blends the restored crop back over the face it came from, fading out towards its edges.
fn paste(out: &mut RgbaImage, restored: &RgbaImage, to_crop: &Similarity, strength: f32) {
    let from_crop = to_crop.inverse();
    let corners = [[0.0, 0.0], [CROP as f32, 0.0], [0.0, CROP as f32], [CROP as f32, CROP as f32]].map(|p| from_crop.apply(p));
    let (w, h) = (out.width() as f32, out.height() as f32);
    let left = corners.iter().map(|p| p[0]).fold(f32::MAX, f32::min).floor().clamp(0.0, w) as u32;
    let right = corners.iter().map(|p| p[0]).fold(f32::MIN, f32::max).ceil().clamp(0.0, w) as u32;
    let top = corners.iter().map(|p| p[1]).fold(f32::MAX, f32::min).floor().clamp(0.0, h) as u32;
    let bottom = corners.iter().map(|p| p[1]).fold(f32::MIN, f32::max).ceil().clamp(0.0, h) as u32;
    for y in top..bottom {
        for x in left..right {
            let [u, v] = to_crop.apply([x as f32 + 0.5, y as f32 + 0.5]);
            let edge = u.min(v).min(CROP as f32 - u).min(CROP as f32 - v);
            if edge <= 0.0 {
                continue;
            }
            let t = (edge / FEATHER).min(1.0);
            let weight = t * t * (3.0 - 2.0 * t) * strength;
            let face = sample(restored, u - 0.5, v - 0.5);
            let pixel = out.get_pixel_mut(x, y);
            for channel in 0..3 {
                let value = pixel[channel] as f32 + (face[channel] - pixel[channel] as f32) * weight;
                pixel[channel] = value.round().clamp(0.0, 255.0) as u8;
            }
        }
    }
}

/// Opens a model for `improve_with`.
pub fn load(path: &Path) -> Result<SharedSession> {
    runtime::session(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Downloads the models into the real data folder and improves the faces in the photo at
    /// `GRANDSHOP_FACES_PHOTO`, writing `faces.png` next to it:
    /// `cargo test --release faces -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn improves_the_faces_in_a_photo() {
        install(&|_| {}).unwrap();
        let photo = std::path::PathBuf::from(std::env::var_os("GRANDSHOP_FACES_PHOTO").unwrap());
        let image = crate::io::load_image(&photo).unwrap();
        let detector = runtime::session(&models::path(&SCRFD).unwrap()).unwrap();
        let start = std::time::Instant::now();
        let faces = detect(&image, &detector, &Feedback::NONE).unwrap();
        eprintln!("{} faces in {:?}", faces.len(), start.elapsed());
        assert!(!faces.is_empty());
        let start = std::time::Instant::now();
        let improved = improve_faces(&image, 0.8, &Feedback::NONE).unwrap();
        eprintln!("improved in {:?}", start.elapsed());
        assert_eq!(improved.dimensions(), image.dimensions());
        assert_ne!(improved, image);
        improved.save(photo.with_file_name("faces.png")).unwrap();
    }

    #[test]
    fn similarity_recovers_a_known_transform() {
        let known = Similarity { a: 0.8, b: 0.3, tx: 12.0, ty: -7.0 };
        let from = [[10.0, 20.0], [50.0, 22.0], [30.0, 40.0], [15.0, 60.0], [45.0, 61.0]];
        let to = from.map(|p| known.apply(p));
        let found = similarity(&from, &to);
        for (p, q) in from.iter().zip(&to) {
            let r = found.apply(*p);
            assert!((r[0] - q[0]).abs() < 1e-3 && (r[1] - q[1]).abs() < 1e-3);
        }
        let back = found.inverse().apply(found.apply([3.0, 4.0]));
        assert!((back[0] - 3.0).abs() < 1e-3 && (back[1] - 4.0).abs() < 1e-3);
    }

    #[test]
    fn windows_cover_the_whole_length_with_overlap() {
        assert_eq!(window_starts(800), vec![0]);
        let starts = window_starts(3000);
        assert_eq!(starts.first(), Some(&0));
        assert_eq!(starts.last(), Some(&(3000 - WINDOW)));
        assert!(starts.windows(2).all(|w| w[1] - w[0] < WINDOW));
    }

    #[test]
    fn overlapping_detections_keep_the_best() {
        let face = |score, left| Face { score, bbox: [left, 0.0, left + 10.0, 10.0], landmarks: [[0.0; 2]; 5] };
        let kept = non_maximum_suppression(vec![face(0.6, 0.0), face(0.9, 1.0), face(0.7, 30.0)], 0.4);
        assert_eq!(kept, vec![face(0.9, 1.0), face(0.7, 30.0)]);
    }

    #[test]
    fn grey_faces_stay_grey() {
        let original = RgbaImage::from_pixel(4, 4, Rgba([100, 100, 100, 255]));
        let restored = RgbaImage::from_pixel(4, 4, Rgba([180, 140, 120, 255]));
        let kept = keep_colours(&original, &restored);
        let pixel = kept.get_pixel(1, 1);
        assert!(pixel[0] == pixel[1] && pixel[1] == pixel[2]);
        // The lightness comes from the restored face
        assert_eq!(pixel[0], 150);
    }

    #[test]
    fn pasting_at_zero_strength_changes_nothing() {
        let mut out = RgbaImage::from_pixel(100, 100, Rgba([10, 20, 30, 255]));
        let before = out.clone();
        let restored = RgbaImage::from_pixel(CROP as u32, CROP as u32, Rgba([200, 200, 200, 255]));
        let to_crop = Similarity { a: 10.0, b: 0.0, tx: -200.0, ty: -200.0 };
        paste(&mut out, &restored, &to_crop, 0.0);
        assert_eq!(out, before);
        paste(&mut out, &restored, &to_crop, 1.0);
        // The middle of the face is replaced, the far corner untouched
        assert_eq!(out.get_pixel(45, 45), &Rgba([200, 200, 200, 255]));
        assert_eq!(out.get_pixel(99, 99), &Rgba([10, 20, 30, 255]));
    }
}
