//! The AI models are not shipped with the app: each is downloaded the first time an operation
//! needs it, checked against its SHA-256 and kept in the app's data folder.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use anyhow::{bail, ensure, Context, Result};
use sha2::{Digest, Sha256};

/// One downloadable model file
#[derive(Debug)]
pub struct ModelFile {
    pub file: &'static str,
    pub url: &'static str,
    pub sha256: &'static str,
    pub bytes: u64,
    /// Licence of the weights, for the notice
    pub licence: &'static str,
}

/// Folder with the downloaded models, e.g. `~/.local/share/grandshop/models/` on Linux.
pub fn models_dir() -> Option<PathBuf> {
    directories::ProjectDirs::from("com", "grandshop", "grandshop").map(|dirs| dirs.data_dir().join("models"))
}

/// Where `model` is kept, whether it is downloaded or not
pub fn path(model: &ModelFile) -> Result<PathBuf> {
    Ok(models_dir().context("no data folder for this user")?.join(model.file))
}

pub fn is_installed(models: &[&ModelFile]) -> bool {
    models.iter().all(|model| path(model).is_ok_and(|path| path.is_file()))
}

/// Downloads and checks the missing `models`, reporting the fraction done. Blocks for as long
/// as the download takes.
pub fn install(models: &[&ModelFile], progress: &dyn Fn(f32)) -> Result<()> {
    let folder = models_dir().context("no data folder for this user")?;
    std::fs::create_dir_all(&folder).with_context(|| format!("creating {}", folder.display()))?;
    let missing: Vec<&ModelFile> = models.iter().copied().filter(|model| !folder.join(model.file).is_file()).collect();
    let total: u64 = missing.iter().map(|model| model.bytes).sum::<u64>().max(1);
    let mut done = 0;
    for model in missing {
        let report = |bytes: u64| progress((done + bytes.min(model.bytes)) as f32 / total as f32);
        install_one(model, &folder, &report)?;
        done += model.bytes;
    }
    progress(1.0);
    Ok(())
}

fn install_one(model: &ModelFile, folder: &Path, progress: &dyn Fn(u64)) -> Result<()> {
    let path = folder.join(model.file);
    let partial = path.with_extension("part");
    let result = download(model.url, &partial, progress).and_then(|()| {
        let data = std::fs::read(&partial)?;
        let digest: String = Sha256::digest(&data).iter().map(|b| format!("{b:02x}")).collect();
        ensure!(digest == model.sha256, "{} has checksum {digest}, expected {}", model.file, model.sha256);
        std::fs::rename(&partial, &path).with_context(|| format!("moving the model to {}", path.display()))
    });
    if result.is_err() {
        let _ = std::fs::remove_file(&partial);
    }
    result
}

/// Uses the system's `curl` (part of Windows 10 and later, macOS and most Linux systems),
/// which keeps an HTTPS client out of the app. Reports the bytes received so far.
fn download(url: &str, target: &Path, progress: &dyn Fn(u64)) -> Result<()> {
    let mut command = Command::new("curl");
    command.args(["--fail", "--silent", "--show-error", "--location", "--retry", "2", "--output"]);
    command.arg(target).arg(url);
    command.stdout(std::process::Stdio::null()).stderr(std::process::Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // No console window flashing up from the windowed app
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let mut child = command.spawn().context("running curl")?;
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        progress(std::fs::metadata(target).map(|m| m.len()).unwrap_or(0));
        std::thread::sleep(Duration::from_millis(250));
    };
    if !status.success() {
        let mut stderr = String::new();
        if let Some(mut pipe) = child.stderr.take() {
            use std::io::Read;
            let _ = pipe.read_to_string(&mut stderr);
        }
        bail!("downloading {url} failed: {}", stderr.trim());
    }
    Ok(())
}

/// Space taken by the downloaded models, in bytes
pub fn installed_bytes() -> u64 {
    let Some(folder) = models_dir() else { return 0 };
    std::fs::read_dir(folder)
        .map(|entries| entries.flatten().filter_map(|e| e.metadata().ok()).filter(|m| m.is_file()).map(|m| m.len()).sum())
        .unwrap_or(0)
}

/// Removes every downloaded model; they come back when an operation needs them again.
pub fn delete_all() -> Result<()> {
    crate::ai::runtime::forget_sessions();
    let Some(folder) = models_dir() else { return Ok(()) };
    if folder.is_dir() {
        std::fs::remove_dir_all(&folder).with_context(|| format!("removing {}", folder.display()))?;
    }
    Ok(())
}
