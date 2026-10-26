//! Shared code for the AI operations: downloading the models, running them with ONNX Runtime
//! and feeding them large images in tiles.

pub mod models;
pub mod runtime;
pub mod tiling;

/// Lets a long AI step report how far it is and stop early when its result is no longer wanted.
pub struct Feedback<'a> {
    pub cancelled: &'a (dyn Fn() -> bool + Sync),
    /// Fraction done, 0 to 1
    pub progress: &'a (dyn Fn(f32) + Sync),
}

fn never() -> bool {
    false
}

fn ignore(_: f32) {}

impl Feedback<'static> {
    /// Runs to the end without reporting anything
    pub const NONE: Feedback<'static> = Feedback { cancelled: &never, progress: &ignore };
}

impl Feedback<'_> {
    pub fn check(&self) -> anyhow::Result<()> {
        if (self.cancelled)() {
            return Err(Cancelled.into());
        }
        Ok(())
    }
}

/// Error of a step that stopped because a newer render was asked for
#[derive(Debug)]
pub struct Cancelled;

impl std::fmt::Display for Cancelled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("cancelled")
    }
}

impl std::error::Error for Cancelled {}
