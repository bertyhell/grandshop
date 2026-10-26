//! Background render thread for the live preview.
//!
//! - **Latest wins**: only the newest requested stack is rendered. A request that arrives
//!   while rendering makes the worker abandon the current render after its current step.
//! - **Per-step cache**: the output of every step is kept. When a new stack shares a prefix
//!   with the previous one, rendering resumes after the last unchanged step, so dragging a
//!   slider on the last card only recomputes that card.

use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

use image::RgbaImage;

use crate::pipeline::{apply_step, Dimensions, Modification};

/// An opened image with its preview proxy.
pub struct Document {
    pub original: Arc<RgbaImage>,
    pub proxy: Arc<RgbaImage>,
    /// Proxy pixels per original pixel
    pub scale: f32,
}

impl Document {
    pub fn new(original: RgbaImage) -> Self {
        let proxy = crate::proxy::make_proxy(&original);
        Self { original: Arc::new(original), proxy: Arc::new(proxy.image), scale: proxy.scale }
    }

    /// Works on the original itself, for full resolution renders.
    pub fn full_resolution(original: Arc<RgbaImage>) -> Self {
        Self { proxy: original.clone(), original, scale: 1.0 }
    }

    pub fn original_dimensions(&self) -> Dimensions {
        Dimensions::of(&self.original)
    }
}

pub struct RenderOutput {
    pub generation: u64,
    /// Rendered proxy image
    pub image: Arc<RgbaImage>,
    /// Size of the result at full resolution
    pub logical: Dimensions,
    /// Full resolution size going into each modification
    pub input_sizes: Vec<Dimensions>,
    /// Failed modifications by index, with the error message
    pub errors: Vec<(usize, String)>,
    /// Number of steps actually computed (the rest came from the cache)
    pub steps_computed: usize,
}

struct RenderJob {
    generation: u64,
    modifications: Vec<Modification>,
}

#[derive(Default)]
struct Mailbox {
    document: Option<Arc<Document>>,
    job: Option<RenderJob>,
    shutdown: bool,
}

#[derive(Default)]
struct Shared {
    mailbox: Mutex<Mailbox>,
    wake: Condvar,
}

impl Shared {
    /// A newer request is waiting, so the current render is no longer wanted.
    fn superseded(&self) -> bool {
        let mailbox = self.mailbox.lock().unwrap();
        mailbox.job.is_some() || mailbox.document.is_some() || mailbox.shutdown
    }
}

struct CacheEntry {
    modification: Modification,
    image: Arc<RgbaImage>,
    logical: Dimensions,
    error: Option<String>,
}

pub struct Worker {
    shared: Arc<Shared>,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    /// `on_result` runs on the worker thread for every finished render.
    pub fn new(on_result: impl Fn(RenderOutput) + Send + 'static) -> Self {
        Self::with_step_cache(true, on_result)
    }

    /// Without the step cache only the final result is kept, and any change re-renders the
    /// whole stack. Meant for full resolution renders, where every step costs a lot of memory.
    pub fn with_step_cache(cache_steps: bool, on_result: impl Fn(RenderOutput) + Send + 'static) -> Self {
        let shared = Arc::new(Shared::default());
        let thread_shared = shared.clone();
        let thread = std::thread::Builder::new()
            .name("render".into())
            .spawn(move || run(&thread_shared, cache_steps, on_result))
            .expect("spawning the render thread");
        Self { shared, thread: Some(thread) }
    }

    /// Switches to a new image. Clears the cache and drops any pending render.
    pub fn set_document(&self, document: Arc<Document>) {
        let mut mailbox = self.shared.mailbox.lock().unwrap();
        mailbox.document = Some(document);
        mailbox.job = None;
        self.shared.wake.notify_one();
    }

    /// Requests a render, replacing any request that hasn't started yet.
    pub fn render(&self, generation: u64, modifications: Vec<Modification>) {
        let mut mailbox = self.shared.mailbox.lock().unwrap();
        mailbox.job = Some(RenderJob { generation, modifications });
        self.shared.wake.notify_one();
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.shared.mailbox.lock().unwrap().shutdown = true;
        self.shared.wake.notify_one();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn run(shared: &Shared, cache_steps: bool, on_result: impl Fn(RenderOutput)) {
    let mut document: Option<Arc<Document>> = None;
    let mut cache: Vec<CacheEntry> = Vec::new();

    loop {
        let job = {
            let mut mailbox = shared.mailbox.lock().unwrap();
            loop {
                if mailbox.shutdown {
                    return;
                }
                if let Some(new_document) = mailbox.document.take() {
                    document = Some(new_document);
                    cache.clear();
                }
                if let Some(job) = mailbox.job.take() {
                    break job;
                }
                mailbox = shared.wake.wait(mailbox).unwrap();
            }
        };
        let Some(document) = document.as_ref() else { continue };
        if let Some(output) = render(shared, document, &mut cache, cache_steps, job) {
            on_result(output);
        }
    }
}

/// Returns `None` when a newer request arrived before the render finished.
fn render(
    shared: &Shared,
    document: &Document,
    cache: &mut Vec<CacheEntry>,
    cache_steps: bool,
    job: RenderJob,
) -> Option<RenderOutput> {
    let mut reusable = cache
        .iter()
        .zip(&job.modifications)
        .take_while(|(entry, modification)| entry.modification == **modification)
        .count();
    if !cache_steps && (reusable < job.modifications.len() || cache.len() != reusable) {
        // Only the final image is kept, so anything but the exact same stack starts over
        reusable = 0;
    }
    cache.truncate(reusable);

    let mut steps_computed = 0;
    for modification in &job.modifications[reusable..] {
        if steps_computed > 0 && shared.superseded() {
            return None;
        }
        let (input, logical) = match cache.last() {
            Some(entry) => (entry.image.clone(), entry.logical),
            None => (document.proxy.clone(), document.original_dimensions()),
        };
        let entry = match apply_step(&input, logical, modification, document.scale) {
            Ok((image, logical)) => CacheEntry {
                modification: modification.clone(),
                image: Arc::new(image),
                logical,
                error: None,
            },
            // A failing step is skipped: its output is its input
            Err(err) => CacheEntry {
                modification: modification.clone(),
                image: input,
                logical,
                error: Some(format!("{err:#}")),
            },
        };
        cache.push(entry);
        steps_computed += 1;
        if !cache_steps && cache.len() >= 2 {
            let previous = cache.len() - 2;
            cache[previous].image = Arc::new(RgbaImage::new(0, 0));
        }
    }

    let (image, logical) = match cache.last() {
        Some(entry) => (entry.image.clone(), entry.logical),
        None => (document.proxy.clone(), document.original_dimensions()),
    };
    let input_sizes = std::iter::once(document.original_dimensions())
        .chain(cache.iter().map(|entry| entry.logical))
        .take(cache.len())
        .collect();
    let errors = cache
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| Some((index, entry.error.clone()?)))
        .collect();
    Some(RenderOutput { generation: job.generation, image, logical, input_sizes, errors, steps_computed })
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::time::Duration;

    use image::Rgba;

    use super::*;

    fn worker() -> (Worker, mpsc::Receiver<RenderOutput>) {
        let (sender, receiver) = mpsc::channel();
        let worker = Worker::new(move |output| {
            let _ = sender.send(output);
        });
        (worker, receiver)
    }

    fn document(width: u32, height: u32) -> Arc<Document> {
        Arc::new(Document::new(RgbaImage::from_pixel(width, height, Rgba([100, 150, 200, 255]))))
    }

    fn next(receiver: &mpsc::Receiver<RenderOutput>) -> RenderOutput {
        receiver.recv_timeout(Duration::from_secs(10)).expect("render result")
    }

    fn border(thickness: u32) -> Modification {
        Modification::AddBorder { thickness, color: Rgba([0, 0, 0, 255]) }
    }

    #[test]
    fn renders_the_stack() {
        let (worker, results) = worker();
        worker.set_document(document(200, 100));
        worker.render(1, vec![border(10), Modification::Sepia { strength: 1.0 }]);
        let output = next(&results);
        assert_eq!(output.generation, 1);
        assert_eq!(output.logical, Dimensions { width: 220, height: 120 });
        assert_eq!(output.image.dimensions(), (220, 120));
        assert_eq!(output.steps_computed, 2);
        assert_eq!(
            output.input_sizes,
            vec![Dimensions { width: 200, height: 100 }, Dimensions { width: 220, height: 120 }]
        );
    }

    #[test]
    fn empty_stack_returns_the_proxy() {
        let (worker, results) = worker();
        let doc = document(6000, 3000);
        worker.set_document(doc.clone());
        worker.render(1, vec![]);
        let output = next(&results);
        assert!(Arc::ptr_eq(&output.image, &doc.proxy));
        assert_eq!(output.logical, Dimensions { width: 6000, height: 3000 });
    }

    #[test]
    fn only_changed_steps_are_recomputed() {
        let (worker, results) = worker();
        worker.set_document(document(200, 100));
        worker.render(1, vec![border(10), Modification::Sepia { strength: 1.0 }, border(5)]);
        assert_eq!(next(&results).steps_computed, 3);

        worker.render(2, vec![border(10), Modification::Sepia { strength: 1.0 }, border(7)]);
        let output = next(&results);
        assert_eq!(output.steps_computed, 1);
        assert_eq!(output.logical, Dimensions { width: 234, height: 134 });

        // Removing the last card needs no work at all
        worker.render(3, vec![border(10), Modification::Sepia { strength: 1.0 }]);
        let output = next(&results);
        assert_eq!(output.steps_computed, 0);
        assert_eq!(output.logical, Dimensions { width: 220, height: 120 });
    }

    #[test]
    fn new_document_clears_the_cache() {
        let (worker, results) = worker();
        worker.set_document(document(200, 100));
        worker.render(1, vec![border(10)]);
        next(&results);
        worker.set_document(document(50, 50));
        worker.render(2, vec![border(10)]);
        let output = next(&results);
        assert_eq!(output.steps_computed, 1);
        assert_eq!(output.logical, Dimensions { width: 70, height: 70 });
    }

    #[test]
    fn failing_steps_are_reported_and_skipped() {
        let (worker, results) = worker();
        worker.set_document(document(100, 100));
        worker.render(1, vec![Modification::Crop { top: 80, right: 0, bottom: 80, left: 0 }, border(1)]);
        let output = next(&results);
        assert_eq!(output.errors.len(), 1);
        assert_eq!(output.errors[0].0, 0);
        assert_eq!(output.logical, Dimensions { width: 102, height: 102 });
    }

    #[test]
    fn without_step_cache_only_the_same_stack_is_reused() {
        let (sender, results) = mpsc::channel();
        let worker = Worker::with_step_cache(false, move |output| {
            let _ = sender.send(output);
        });
        worker.set_document(document(200, 100));
        worker.render(1, vec![border(10), Modification::Sepia { strength: 1.0 }]);
        assert_eq!(next(&results).steps_computed, 2);
        worker.render(2, vec![border(10), Modification::Sepia { strength: 1.0 }]);
        let output = next(&results);
        assert_eq!(output.steps_computed, 0);
        assert_eq!(output.image.dimensions(), (220, 120));
        worker.render(3, vec![border(10), border(5)]);
        assert_eq!(next(&results).steps_computed, 2);
        worker.render(4, vec![border(10)]);
        let output = next(&results);
        assert_eq!(output.steps_computed, 1);
        assert_eq!(output.image.dimensions(), (220, 120));
    }

    #[test]
    fn latest_request_wins() {
        let (worker, results) = worker();
        worker.set_document(document(2500, 2500));
        // Slow steps, requested much faster than they can be rendered
        let stack = |factor: f32| vec![Modification::Sepia { strength: 1.0 }, Modification::Brightness { factor, contrast: 1.0, saturation: 1.0 }, Modification::Sepia { strength: 1.0 }];
        for generation in 1..=50 {
            worker.render(generation, stack(1.0 + generation as f32 / 100.0));
        }
        let mut last = next(&results);
        let mut delivered = 1;
        while let Ok(output) = results.recv_timeout(Duration::from_millis(500)) {
            last = output;
            delivered += 1;
        }
        assert_eq!(last.generation, 50);
        assert!(delivered < 50, "stale renders should be dropped, got {delivered}");
    }
}
