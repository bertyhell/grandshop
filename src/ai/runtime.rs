//! ONNX Runtime sessions, on the graphics card where the platform has a driver for it
//! (DirectX 12 on Windows, Core ML on macOS), else on the CPU.

use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use anyhow::{anyhow, ensure, Context, Result};
use ort::environment::Environment;
use ort::session::{Session, SessionInputValue};
use ort::value::{DynValue, Tensor};

static USE_GPU: AtomicBool = AtomicBool::new(true);

pub type SharedSession = Arc<Mutex<Session>>;

/// Loaded models by file. Loading takes a while, so they stay loaded.
static SESSIONS: Mutex<Vec<(PathBuf, SharedSession)>> = Mutex::new(Vec::new());

/// Switches the graphics card on or off for models loaded from now on. Off is the way out
/// when a graphics driver misbehaves.
pub fn set_use_gpu(on: bool) {
    if USE_GPU.swap(on, Ordering::SeqCst) != on {
        forget_sessions();
    }
}

pub fn forget_sessions() {
    SESSIONS.lock().unwrap().clear();
}

fn environment() -> Result<&'static Environment> {
    static ENVIRONMENT: OnceLock<std::result::Result<Environment, String>> = OnceLock::new();
    ENVIRONMENT
        .get_or_init(|| ort::init().with_name("grandshop").with_telemetry(false).build().map_err(|e| e.to_string()))
        .as_ref()
        .map_err(|err| anyhow!("starting ONNX Runtime: {err}"))
}

/// The model at `path`, loaded once.
pub fn session(path: &Path) -> Result<SharedSession> {
    let mut sessions = SESSIONS.lock().unwrap();
    if let Some((_, session)) = sessions.iter().find(|(p, _)| p == path) {
        return Ok(session.clone());
    }
    let session = Arc::new(Mutex::new(load(path).with_context(|| format!("loading {}", path.display()))?));
    sessions.push((path.to_owned(), session.clone()));
    Ok(session)
}

fn load(path: &Path) -> Result<Session> {
    let mut builder = Session::builder(environment()?)?;
    if USE_GPU.load(Ordering::SeqCst) {
        // A provider that fails to start is skipped, so the model then runs on the CPU
        #[cfg(windows)]
        {
            // DirectML needs these off
            builder = builder.with_memory_pattern(false).map_err(ort::Error::<()>::from)?;
            builder = builder.with_parallel_execution(false).map_err(ort::Error::<()>::from)?;
            builder = builder
                .with_execution_providers([ort::ep::DirectML::default().build()])
                .map_err(ort::Error::<()>::from)?;
        }
        #[cfg(target_os = "macos")]
        {
            let mut coreml = ort::ep::CoreML::default();
            if let Some(cache) = super::models::models_dir().map(|dir| dir.join("coreml-cache")) {
                coreml = coreml.with_model_cache_dir(cache.display());
            }
            builder = builder.with_execution_providers([coreml.build()]).map_err(ort::Error::<()>::from)?;
        }
    }
    Ok(builder.commit_from_file(path)?)
}

/// Runs a network with one `(1, 3, size, size)` float input and returns its first output.
pub fn run_image(session: &SharedSession, input: Vec<f32>, size: u32) -> Result<(Vec<usize>, Vec<f32>)> {
    let size = size as usize;
    ensure!(input.len() == 3 * size * size, "input has {} values, not 3×{size}×{size}", input.len());
    let tensor = Tensor::from_array(([1usize, 3, size, size], input))?.into_dyn();
    let mut outputs = run(session, tensor, Vec::new())?;
    Ok(outputs.swap_remove(0))
}

/// Runs a network on `input` (its first input) plus `extra` inputs by name, and returns every
/// float output in order, as shape and values.
pub fn run(
    session: &SharedSession,
    input: DynValue,
    extra: Vec<(&str, DynValue)>,
) -> Result<Vec<(Vec<usize>, Vec<f32>)>> {
    let mut session = session.lock().unwrap();
    let name = session.inputs()[0].name().to_owned();
    let mut inputs: Vec<(Cow<str>, SessionInputValue)> = vec![(name.into(), input.into())];
    inputs.extend(extra.into_iter().map(|(name, value)| (Cow::Owned(name.to_owned()), value.into())));
    let outputs = session.run(inputs)?;
    let mut result = Vec::new();
    for (_, value) in outputs.iter() {
        if let Ok((shape, data)) = value.try_extract_tensor::<f32>() {
            result.push((shape.iter().map(|&d| d as usize).collect(), data.to_vec()));
        }
    }
    ensure!(!result.is_empty(), "the model gave no float output");
    Ok(result)
}
