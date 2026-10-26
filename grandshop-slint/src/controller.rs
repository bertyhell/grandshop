//! Connects the Slint UI to the image core: the card list, file dialogs, background
//! loading and saving, and feeding render results from the worker into the preview.

use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use grandshop::color::{color_wheel, hsv_to_rgb, rgb_to_hsv};
use grandshop::io::{default_save_file_name, format_file_size, load_image, save_png};
use grandshop::pipeline::{self, available_borders, Modification};
use grandshop::resample::resize_exact;
use grandshop::view::{Region, ViewTransform};
use grandshop::worker::{Document, RenderOutput, Worker};
use image::{Rgba, RgbaImage};
use slint::{Color, ComponentHandle, Image, Model, ModelRc, Rgba8Pixel, SharedPixelBuffer, VecModel};

use crate::toast;
use crate::{AppState, AppWindow, Hsv, ImageSize, ModEntry, ModField, ModKind, ModParams, Strings, ToastKind};

const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "tif", "tiff", "webp", "bmp", "svg"];

/// Rendered at twice the 300 px it is shown at, so it stays sharp on high DPI screens
const COLOR_WHEEL_SIZE: u32 = 600;

/// Wait this long after zooming or panning stops before rendering the sharp tile
const DETAIL_DELAY: Duration = Duration::from_millis(150);
/// Show the sharp tile once the preview is enlarged this much beyond its own pixels
const DETAIL_THRESHOLD: f32 = 1.2;
/// The sharp tile covers this much extra around the visible part, so small pans stay sharp
const DETAIL_MARGIN: f32 = 0.25;
/// Largest side of the sharp tile in pixels
const DETAIL_MAX_SIZE: u32 = 4096;

#[derive(Default)]
struct State {
    path: Option<PathBuf>,
    document: Option<Arc<Document>>,
    modifications: Vec<Modification>,
}

/// Shared with the worker thread
#[derive(Default)]
struct RenderShared {
    /// Generation of the newest render request; older results are ignored
    latest_generation: AtomicU64,
    /// Latest preview image, for the eyedropper
    last_image: Mutex<Option<Arc<RgbaImage>>>,
    /// Whether a "Reduce image" card needs the file size estimate
    estimate_file_size: AtomicBool,
    /// Rotation and mirroring of the preview
    transform: Mutex<ViewTransform>,
    view: Mutex<ViewState>,
    /// Full resolution result with the generation it belongs to, for the sharp tile
    detail: Mutex<Option<(u64, Arc<RgbaImage>)>>,
    /// Proxy pixels per original pixel of the open image
    proxy_scale: Mutex<f32>,
}

/// What part of the preview is visible, as reported by the viewer
#[derive(Clone, Copy)]
struct ViewState {
    region: Region,
    /// Size of the whole shown image on screen in logical pixels
    shown_width: f32,
}

impl Default for ViewState {
    fn default() -> Self {
        Self { region: Region::FULL, shown_width: 0.0 }
    }
}

pub struct Controller {
    ui: slint::Weak<AppWindow>,
    worker: Worker,
    detail_worker: Worker,
    detail_timer: slint::Timer,
    /// Generation of the last full resolution render that was asked for
    detail_requested: Cell<u64>,
    state: RefCell<State>,
    cards: Rc<VecModel<ModEntry>>,
    borders: Vec<String>,
    shared: Arc<RenderShared>,
}

impl Controller {
    pub fn new(ui: &AppWindow) -> Rc<Self> {
        let shared = Arc::new(RenderShared::default());
        let worker = Worker::new(result_handler(ui.as_weak(), shared.clone()));
        let detail_worker = Worker::with_step_cache(false, detail_handler(ui.as_weak(), shared.clone()));
        let cards = Rc::new(VecModel::default());
        let borders = available_borders();

        let app_state = ui.global::<AppState>();
        app_state.set_modifications(ModelRc::from(cards.clone()));
        app_state.set_border_styles(ModelRc::new(VecModel::from(
            borders.iter().map(|name| name.replace('_', " ").into()).collect::<Vec<slint::SharedString>>(),
        )));
        app_state.set_color_wheel(to_slint_image(&color_wheel(COLOR_WHEEL_SIZE)));
        app_state.on_hsv_to_color(|hsv| {
            let [r, g, b] = hsv_to_rgb(hsv.hue, hsv.saturation, hsv.value);
            Color::from_rgb_u8(r, g, b)
        });
        app_state.on_color_to_hsv(|color| {
            let (hue, saturation, value) = rgb_to_hsv([color.red(), color.green(), color.blue()]);
            Hsv { hue, saturation, value }
        });
        toast::init(ui);
        {
            let ui = ui.as_weak();
            app_state.on_invalid_number(move || {
                if let Some(ui) = ui.upgrade() {
                    let message = ui.global::<Strings>().get_invalid_number();
                    toast::show(&ui, ToastKind::Warning, &message);
                }
            });
        }

        let controller = Rc::new(Self {
            ui: ui.as_weak(),
            worker,
            detail_worker,
            detail_timer: slint::Timer::default(),
            detail_requested: Cell::new(0),
            state: RefCell::default(),
            cards,
            borders,
            shared,
        });
        controller.connect(&app_state);
        controller
    }

    fn connect(self: &Rc<Self>, app_state: &AppState) {
        macro_rules! on {
            ($setter:ident, |$this:ident $(, $arg:ident)*| $body:expr) => {{
                let $this = self.clone();
                app_state.$setter(move |$($arg),*| $body);
            }};
        }
        on!(on_open_image, |this| this.open_image());
        on!(on_save_image, |this| this.save_image());
        on!(on_add_modification, |this, kind| this.add_card(kind));
        on!(on_toggle_modification, |this, index| this.toggle_card(index as usize));
        on!(on_delete_modification, |this, index| this.delete_card(index as usize));
        on!(on_set_number, |this, index, field, value| {
            this.edit_card(index as usize, |params| set_field(params, field, value))
        });
        on!(on_set_color, |this, index, color| this.edit_card(index as usize, |p| p.color = color));
        on!(on_set_text, |this, index, text| this.edit_card(index as usize, |p| p.text = text));
        on!(on_set_border_style, |this, index, style| {
            this.edit_card(index as usize, |p| p.border_index = style)
        });
        on!(on_preview_clicked, |this, x, y| this.pick_color(x, y));
        on!(on_rotate_left, |this| this.change_view(ViewTransform::rotate_counter_clockwise));
        on!(on_rotate_right, |this| this.change_view(ViewTransform::rotate_clockwise));
        on!(on_flip_horizontal, |this| this.change_view(ViewTransform::flip_horizontal));
        on!(on_flip_vertical, |this| this.change_view(ViewTransform::flip_vertical));
        on!(on_reset_view, |this| this.change_view(|_| ViewTransform::default()));
        on!(on_view_changed, |this, left, top, right, bottom, width, _height| {
            *this.shared.view.lock().unwrap() =
                ViewState { region: Region { left, top, right, bottom }, shown_width: width };
            this.schedule_detail();
        });
    }

    /// Shows a notification with a translated text from the `Strings` global.
    fn notify(&self, kind: ToastKind, message: impl FnOnce(&Strings) -> slint::SharedString) {
        if let Some(ui) = self.ui.upgrade() {
            let text = message(&ui.global::<Strings>());
            toast::show(&ui, kind, &text);
        }
    }

    fn text(&self, get: impl FnOnce(&Strings) -> slint::SharedString) -> String {
        self.ui.upgrade().map(|ui| get(&ui.global::<Strings>()).into()).unwrap_or_default()
    }

    fn with_state<R>(&self, f: impl FnOnce(&AppState) -> R) -> Option<R> {
        self.ui.upgrade().map(|ui| f(&ui.global::<AppState>()))
    }

    // Opening and saving

    pub fn open_image(self: &Rc<Self>) {
        let this = self.clone();
        let _ = slint::spawn_local(async move {
            let Some(file) = rfd::AsyncFileDialog::new()
                .set_title(this.text(|s| s.get_open_dialog_title()))
                .add_filter(this.text(|s| s.get_pictures()), IMAGE_EXTENSIONS)
                .add_filter(this.text(|s| s.get_all_files()), &["*"])
                .pick_file()
                .await
            else {
                return;
            };
            this.load(file.path().to_owned()).await;
        });
    }

    /// Opens `path` without asking, e.g. a file passed on the command line.
    pub fn open_path(self: &Rc<Self>, path: PathBuf) {
        let this = self.clone();
        let _ = slint::spawn_local(async move { this.load(path).await });
    }

    async fn load(self: &Rc<Self>, path: PathBuf) {
        self.with_state(|s| s.set_loading(true));
        let load_path = path.clone();
        let loaded = run_in_background(move || load_image(&load_path).map(Document::new)).await;
        self.with_state(|s| s.set_loading(false));

        match loaded {
            Ok(document) => self.set_document(path, document),
            Err(err) => {
                eprintln!("{err:#}");
                self.notify(ToastKind::Error, |s| s.get_open_failed());
            }
        }
    }

    fn set_document(self: &Rc<Self>, path: PathBuf, document: Document) {
        let document = Arc::new(document);
        *self.shared.last_image.lock().unwrap() = Some(document.proxy.clone());
        *self.shared.transform.lock().unwrap() = ViewTransform::default();
        *self.shared.proxy_scale.lock().unwrap() = document.scale;
        *self.shared.detail.lock().unwrap() = None;
        self.with_state(|s| {
            s.set_has_image(true);
            s.set_detail_visible(false);
            s.set_preview_image(to_slint_image(&document.proxy));
            let size = document.original_dimensions();
            s.set_image_width(size.width as i32);
            s.set_image_height(size.height as i32);
        });
        self.worker.set_document(document.clone());
        self.detail_worker.set_document(Arc::new(Document::full_resolution(document.original.clone())));
        {
            let mut state = self.state.borrow_mut();
            state.path = Some(path);
            state.document = Some(document);
        }
        // Cards stay when another image is opened, so apply them to the new one
        self.request_render();
    }

    pub fn save_image(self: &Rc<Self>) {
        let (document, path, modifications) = {
            let state = self.state.borrow();
            let (Some(document), Some(path)) = (state.document.clone(), state.path.clone()) else {
                return;
            };
            (document, path, state.modifications.clone())
        };
        let this = self.clone();
        let _ = slint::spawn_local(async move {
            let mut dialog = rfd::AsyncFileDialog::new()
                .set_title(this.text(|s| s.get_save_dialog_title()))
                .set_file_name(default_save_file_name(&path, chrono::Local::now().date_naive()))
                .add_filter(this.text(|s| s.get_pictures()), &["png"])
                .add_filter(this.text(|s| s.get_all_files()), &["*"]);
            if let Some(folder) = path.parent() {
                dialog = dialog.set_directory(folder);
            }
            let Some(file) = dialog.save_file().await else { return };
            let target = file.path().to_owned();

            this.with_state(|s| s.set_saving(true));
            let saved = run_in_background(move || {
                let output = pipeline::apply(
                    &document.original,
                    document.original_dimensions(),
                    &modifications,
                    1.0,
                );
                save_png(&output.image, &target)
            })
            .await;
            this.with_state(|s| s.set_saving(false));

            match saved {
                Ok(()) => this.notify(ToastKind::Success, |s| s.get_saved()),
                Err(err) => {
                    eprintln!("{err:#}");
                    this.notify(ToastKind::Error, |s| s.get_save_failed());
                }
            }
        });
    }

    // The card list

    fn add_card(self: &Rc<Self>, kind: ModKind) {
        let number = self.cards.iter().filter(|card| card.kind == kind).count() as i32 + 1;
        let (width, height) = self
            .with_state(|s| (s.get_image_width(), s.get_image_height()))
            .unwrap_or_default();
        // Adding a card collapses the others
        for index in 0..self.cards.row_count() {
            let mut card = self.cards.row_data(index).unwrap();
            if card.open {
                card.open = false;
                self.cards.set_row_data(index, card);
            }
        }
        self.cards.push(ModEntry { kind, number, open: true, params: default_params(kind, width, height) });

        // The new card works on the current result until the next render reports its size
        self.with_state(|s| {
            let mut sizes: Vec<ImageSize> = s.get_input_sizes().iter().collect();
            sizes.resize(self.cards.row_count() - 1, ImageSize { width, height });
            sizes.push(ImageSize { width, height });
            s.set_input_sizes(ModelRc::new(VecModel::from(sizes)));
        });
        self.cards_changed();
    }

    fn toggle_card(&self, index: usize) {
        if let Some(mut card) = self.cards.row_data(index) {
            card.open = !card.open;
            self.cards.set_row_data(index, card);
            self.update_eyedropper();
        }
    }

    fn delete_card(self: &Rc<Self>, index: usize) {
        if index < self.cards.row_count() {
            self.cards.remove(index);
            self.cards_changed();
        }
    }

    fn edit_card(self: &Rc<Self>, index: usize, edit: impl FnOnce(&mut ModParams)) {
        if let Some(mut card) = self.cards.row_data(index) {
            edit(&mut card.params);
            self.cards.set_row_data(index, card);
            self.cards_changed();
        }
    }

    fn cards_changed(self: &Rc<Self>) {
        let modifications: Vec<Modification> =
            self.cards.iter().map(|card| to_modification(&card, &self.borders)).collect();
        let has_resize = modifications.iter().any(|m| matches!(m, Modification::Resize { .. }));
        self.shared.estimate_file_size.store(has_resize, Ordering::SeqCst);
        self.state.borrow_mut().modifications = modifications;
        self.update_eyedropper();
        self.request_render();
        // The sharp tile shows the previous stack, so hide it until it is redone
        self.with_state(|s| s.set_detail_visible(false));
        self.schedule_detail();
    }

    fn open_border_cards(&self) -> Vec<usize> {
        (0..self.cards.row_count())
            .filter(|&index| {
                let card = self.cards.row_data(index).unwrap();
                card.open && card.kind == ModKind::AddBorder
            })
            .collect()
    }

    fn update_eyedropper(&self) {
        let active = !self.open_border_cards().is_empty();
        self.with_state(|s| s.set_eyedropper_active(active));
    }

    /// Eyedropper: `x` and `y` go from 0 to 1 over the shown preview.
    fn pick_color(self: &Rc<Self>, x: f32, y: f32) {
        if !(0.0..=1.0).contains(&x) || !(0.0..=1.0).contains(&y) {
            return;
        }
        let Some(image) = self.shared.last_image.lock().unwrap().clone() else { return };
        let (x, y) = self.shared.transform.lock().unwrap().to_source((x, y));
        let px = ((x * image.width() as f32) as u32).min(image.width() - 1);
        let py = ((y * image.height() as f32) as u32).min(image.height() - 1);
        let Rgba([r, g, b, _]) = *image.get_pixel(px, py);
        for index in self.open_border_cards() {
            self.edit_card(index, |params| params.color = Color::from_rgb_u8(r, g, b));
        }
    }

    fn change_view(self: &Rc<Self>, change: impl FnOnce(ViewTransform) -> ViewTransform) {
        let transform = {
            let mut transform = self.shared.transform.lock().unwrap();
            *transform = change(*transform);
            *transform
        };
        let Some(image) = self.shared.last_image.lock().unwrap().clone() else { return };
        self.with_state(|s| {
            s.set_detail_visible(false);
            s.set_preview_image(to_slint_image(&transform.apply(&image)));
        });
        self.schedule_detail();
    }

    /// Re-checks the sharp tile once zooming or panning has stopped.
    fn schedule_detail(self: &Rc<Self>) {
        let this = Rc::downgrade(self);
        self.detail_timer.start(slint::TimerMode::SingleShot, DETAIL_DELAY, move || {
            if let Some(this) = this.upgrade() {
                this.update_detail();
            }
        });
    }

    fn update_detail(&self) {
        let Some(ui) = self.ui.upgrade() else { return };
        if !detail_needed(&ui, &self.shared) {
            ui.global::<AppState>().set_detail_visible(false);
            return;
        }
        let latest = self.shared.latest_generation.load(Ordering::SeqCst);
        let ready = matches!(&*self.shared.detail.lock().unwrap(), Some((generation, _)) if *generation == latest);
        if ready {
            show_detail(&ui, &self.shared);
        } else if self.detail_requested.get() != latest {
            self.detail_requested.set(latest);
            self.detail_worker.render(latest, self.state.borrow().modifications.clone());
        }
    }

    fn request_render(self: &Rc<Self>) {
        let state = self.state.borrow();
        if state.document.is_none() {
            return;
        }
        let generation = self.shared.latest_generation.fetch_add(1, Ordering::SeqCst) + 1;
        let has_modifications = !state.modifications.is_empty();
        self.with_state(|s| {
            s.set_rendering(has_modifications);
            s.set_has_modified_image(has_modifications);
        });
        self.worker.render(generation, state.modifications.clone());
    }
}

fn default_params(kind: ModKind, image_width: i32, image_height: i32) -> ModParams {
    let mut params = ModParams { border_index: -1, ..Default::default() };
    match kind {
        ModKind::AddBorder => {
            params.thickness = 100.0;
            params.color = Color::from_rgb_u8(0x22, 0x5e, 0xff);
        }
        ModKind::AddImageBorder => params.border_width = 100.0,
        ModKind::Brightness => params.factor = 1.1,
        ModKind::Sepia | ModKind::Crop => {}
        ModKind::Resize => params.size = image_width.max(image_height).min(10_000) as f32,
        ModKind::Annotate => {
            params.font_size = 30.0;
            params.x_percent = 50.0;
            params.y_percent = 95.0;
        }
    }
    params
}

fn set_field(params: &mut ModParams, field: ModField, value: f32) {
    let target = match field {
        ModField::Thickness => &mut params.thickness,
        ModField::BorderWidth => &mut params.border_width,
        ModField::Factor => &mut params.factor,
        ModField::Size => &mut params.size,
        ModField::FontSize => &mut params.font_size,
        ModField::XPercent => &mut params.x_percent,
        ModField::YPercent => &mut params.y_percent,
        ModField::Top => &mut params.top,
        ModField::Right => &mut params.right,
        ModField::Bottom => &mut params.bottom,
        ModField::Left => &mut params.left,
    };
    *target = value;
}

fn to_modification(card: &ModEntry, borders: &[String]) -> Modification {
    let p = &card.params;
    let px = |value: f32| value.round().max(0.0) as u32;
    match card.kind {
        ModKind::AddBorder => Modification::AddBorder {
            thickness: px(p.thickness),
            color: Rgba([p.color.red(), p.color.green(), p.color.blue(), 255]),
        },
        ModKind::AddImageBorder => Modification::AddImageBorder {
            border: usize::try_from(p.border_index).ok().and_then(|i| borders.get(i)).cloned(),
            width: px(p.border_width),
        },
        ModKind::Brightness => Modification::Brightness { factor: p.factor },
        ModKind::Sepia => Modification::Sepia,
        ModKind::Resize => Modification::Resize { size: px(p.size) },
        ModKind::Annotate => Modification::Annotate {
            text: p.text.to_string(),
            font_size: p.font_size,
            x_percent: p.x_percent,
            y_percent: p.y_percent,
        },
        ModKind::Crop => Modification::Crop {
            top: px(p.top),
            right: px(p.right),
            bottom: px(p.bottom),
            left: px(p.left),
        },
    }
}

/// Runs on the worker thread: hands finished renders to the UI thread.
fn result_handler(
    ui: slint::Weak<AppWindow>,
    shared: Arc<RenderShared>,
) -> impl Fn(RenderOutput) + Send + 'static {
    // Only report a failing card once, not on every slider change
    let reported_errors = Mutex::new(Vec::<usize>::new());
    move |output| {
        if output.generation != shared.latest_generation.load(Ordering::SeqCst) {
            return;
        }
        let failed: Vec<usize> = output.errors.iter().map(|(index, _)| *index).collect();
        {
            let mut reported = reported_errors.lock().unwrap();
            if failed.iter().any(|index| !reported.contains(index)) {
                for (index, message) in &output.errors {
                    eprintln!("modification {index} failed: {message}");
                }
                let _ = ui.upgrade_in_event_loop(|ui| {
                    let message = ui.global::<Strings>().get_apply_failed();
                    toast::show(&ui, ToastKind::Error, &message);
                });
            }
            *reported = failed;
        }
        *shared.last_image.lock().unwrap() = Some(output.image.clone());
        if shared.estimate_file_size.load(Ordering::SeqCst) {
            estimate_file_size(ui.clone(), shared.clone(), &output);
        }

        // Rotate and copy into Slint's buffer here, off the UI thread
        let transform = *shared.transform.lock().unwrap();
        let buffer = if transform.is_identity() {
            to_pixel_buffer(&output.image)
        } else {
            to_pixel_buffer(&transform.apply(&output.image))
        };
        let shared = shared.clone();
        let _ = ui.upgrade_in_event_loop(move |ui| {
            if output.generation != shared.latest_generation.load(Ordering::SeqCst) {
                return;
            }
            let state = ui.global::<AppState>();
            state.set_preview_image(Image::from_rgba8(buffer));
            state.set_image_width(output.logical.width as i32);
            state.set_image_height(output.logical.height as i32);
            let sizes: Vec<ImageSize> = output
                .input_sizes
                .iter()
                .map(|size| ImageSize { width: size.width as i32, height: size.height as i32 })
                .collect();
            state.set_input_sizes(ModelRc::new(VecModel::from(sizes)));
            state.set_rendering(false);
        });
    }
}

/// Runs on the full resolution worker thread: keeps the result and shows the sharp tile.
fn detail_handler(
    ui: slint::Weak<AppWindow>,
    shared: Arc<RenderShared>,
) -> impl Fn(RenderOutput) + Send + 'static {
    move |output| {
        if output.generation != shared.latest_generation.load(Ordering::SeqCst) {
            return;
        }
        *shared.detail.lock().unwrap() = Some((output.generation, output.image));
        let shared = shared.clone();
        let _ = ui.upgrade_in_event_loop(move |ui| {
            if detail_needed(&ui, &shared) {
                show_detail(&ui, &shared);
            }
        });
    }
}

/// Whether the preview is enlarged so much that its proxy pixels show.
fn detail_needed(ui: &AppWindow, shared: &RenderShared) -> bool {
    if *shared.proxy_scale.lock().unwrap() >= 1.0 {
        return false;
    }
    let shown_pixels = shared.view.lock().unwrap().shown_width * ui.window().scale_factor();
    let preview_pixels = ui.global::<AppState>().get_preview_image().size().width as f32;
    preview_pixels > 0.0 && shown_pixels > preview_pixels * DETAIL_THRESHOLD
}

/// Cuts the visible part (plus a margin) out of the full resolution result and lays it
/// over the preview.
fn show_detail(ui: &AppWindow, shared: &RenderShared) {
    let state = ui.global::<AppState>();
    let latest = shared.latest_generation.load(Ordering::SeqCst);
    let Some(full) = shared.detail.lock().unwrap().as_ref().filter(|(g, _)| *g == latest).map(|(_, image)| image.clone())
    else {
        return;
    };
    let transform = *shared.transform.lock().unwrap();
    let visible = shared.view.lock().unwrap().region;

    let margin_x = (visible.right - visible.left) * DETAIL_MARGIN;
    let margin_y = (visible.bottom - visible.top) * DETAIL_MARGIN;
    let wanted = Region {
        left: (visible.left - margin_x).max(0.0),
        top: (visible.top - margin_y).max(0.0),
        right: (visible.right + margin_x).min(1.0),
        bottom: (visible.bottom + margin_y).min(1.0),
    };
    let source = transform.region_to_source(wanted);
    let (width, height) = (full.width() as f32, full.height() as f32);
    let x0 = (source.left * width).floor() as u32;
    let y0 = (source.top * height).floor() as u32;
    let x1 = ((source.right * width).ceil() as u32).min(full.width());
    let y1 = ((source.bottom * height).ceil() as u32).min(full.height());
    if x1 <= x0 || y1 <= y0 {
        state.set_detail_visible(false);
        return;
    }

    let mut tile = image::imageops::crop_imm(&*full, x0, y0, x1 - x0, y1 - y0).to_image();
    let largest = tile.width().max(tile.height());
    if largest > DETAIL_MAX_SIZE {
        let shrink = DETAIL_MAX_SIZE as f32 / largest as f32;
        tile = resize_exact(&tile, (tile.width() as f32 * shrink) as u32, (tile.height() as f32 * shrink) as u32);
    }
    // Where the whole-pixel crop ends up on screen
    let shown = transform.region_from_source(Region {
        left: x0 as f32 / width,
        top: y0 as f32 / height,
        right: x1 as f32 / width,
        bottom: y1 as f32 / height,
    });
    state.set_detail_image(to_slint_image(&transform.apply(&tile)));
    state.set_detail_left(shown.left);
    state.set_detail_top(shown.top);
    state.set_detail_right(shown.right);
    state.set_detail_bottom(shown.bottom);
    state.set_detail_visible(true);
}

/// Encodes the preview as PNG and scales the size up to full resolution. Waits a little
/// first, so dragging a slider doesn't encode every intermediate result.
fn estimate_file_size(ui: slint::Weak<AppWindow>, shared: Arc<RenderShared>, output: &RenderOutput) {
    let generation = output.generation;
    let image = output.image.clone();
    let full_pixels = output.logical.width as f64 * output.logical.height as f64;
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(250));
        if generation != shared.latest_generation.load(Ordering::SeqCst) {
            return;
        }
        let Ok(png) = grandshop::io::encode_png(&image) else { return };
        let proxy_pixels = image.width() as f64 * image.height() as f64;
        let estimate = (png.len() as f64 * full_pixels / proxy_pixels) as u64;
        let _ = ui.upgrade_in_event_loop(move |ui| {
            if generation == shared.latest_generation.load(Ordering::SeqCst) {
                ui.global::<AppState>().set_estimated_file_size(format_file_size(estimate).into());
            }
        });
    });
}

fn to_pixel_buffer(image: &RgbaImage) -> SharedPixelBuffer<Rgba8Pixel> {
    SharedPixelBuffer::clone_from_slice(image.as_raw(), image.width(), image.height())
}

fn to_slint_image(image: &RgbaImage) -> Image {
    Image::from_rgba8(to_pixel_buffer(image))
}

/// Runs `work` on a new thread and resolves on the UI thread when it is done.
async fn run_in_background<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> T {
    let (sender, receiver) = futures_channel::oneshot::channel();
    std::thread::spawn(move || {
        let _ = sender.send(work());
    });
    receiver.await.expect("background work panicked")
}
