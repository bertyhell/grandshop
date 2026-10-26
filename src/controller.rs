//! Connects the Slint UI to the image core: the card list, file dialogs, background
//! loading and saving, and feeding render results from the worker into the preview.

use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use grandshop::color::{color_wheel, hsv_to_rgb, rgb_to_hsv};
use grandshop::crop_aspect::{self, CropBox};
use grandshop::io::{default_save_file_name, format_file_size, load_image, save_png};
use grandshop::pipeline::{self, available_borders, Dimensions, Modification, TextEffect};
use grandshop::resample::resize_exact;
use grandshop::settings::Settings;
use grandshop::{background, colourise, faces, upscale};
use grandshop::view::{Region, ViewTransform};
use grandshop::worker::{Document, RenderOutput, Worker};
use image::{Rgba, RgbaImage};
use slint::{Color, ComponentHandle, Image, Model, ModelRc, Rgba8Pixel, SharedPixelBuffer, VecModel};

use crate::toast;
use crate::{
    AppState, AppWindow, ColorField, FlagField, Hsv, ImageSize, ModEntry, ModField, ModKind, ModParams, Strings,
    ToastKind,
};

const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "tif", "tiff", "webp", "bmp", "svg"];

/// Rendered at twice the 300 px it is shown at, so it stays sharp on high DPI screens
const COLOR_WHEEL_SIZE: u32 = 600;

/// Size of the border style examples in the picker, twice the 64 × 48 they are shown at
const BORDER_THUMBNAIL_SIZE: (u32, u32) = (128, 96);

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
    /// Stack of the last render request, so unchanged stacks aren't rendered again
    rendered: Option<Vec<Modification>>,
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
    /// Number of cards; a render of fewer stops before an open crop card
    card_count: AtomicUsize,
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
    /// The AI operations whose models are being downloaded
    ai_downloads: RefCell<Vec<AiTool>>,
    state: RefCell<State>,
    cards: Rc<VecModel<ModEntry>>,
    borders: Vec<String>,
    shared: Arc<RenderShared>,
}

impl Controller {
    pub fn new(ui: &AppWindow) -> Rc<Self> {
        let shared = Arc::new(RenderShared::default());
        let worker = Worker::new(result_handler(ui.as_weak(), shared.clone()));
        {
            let ui = ui.as_weak();
            worker.on_progress(move |_, fraction| {
                let _ = ui.upgrade_in_event_loop(move |ui| ui.global::<AppState>().set_render_progress(fraction));
            });
        }
        let detail_worker = Worker::with_step_cache(false, detail_handler(ui.as_weak(), shared.clone()));
        let cards = Rc::new(VecModel::default());
        let borders = available_borders();

        let app_state = ui.global::<AppState>();
        app_state.set_modifications(ModelRc::from(cards.clone()));
        app_state.set_border_styles(ModelRc::new(VecModel::from(
            borders.iter().map(|name| name.replace('_', " ").into()).collect::<Vec<slint::SharedString>>(),
        )));
        app_state.set_border_thumbnails(ModelRc::new(VecModel::from(
            borders.iter().map(|name| to_slint_image(&border_thumbnail(name))).collect::<Vec<Image>>(),
        )));
        app_state.set_color_wheel(to_slint_image(&color_wheel(COLOR_WHEEL_SIZE)));
        app_state.set_upscale_max_edge(upscale::MAX_RESULT_EDGE as i32);
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
            ai_downloads: RefCell::default(),
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
            this.edit_card(index as usize, |params| {
                set_field(params, field, value);
                // Typing a side by hand gives up the fixed shape
                if matches!(field, ModField::Top | ModField::Right | ModField::Bottom | ModField::Left) {
                    params.aspect_index = 0;
                }
            })
        });
        on!(on_set_aspect, |this, index, preset| this.set_aspect(index as usize, preset));
        on!(on_set_color, |this, index, field, color| {
            this.edit_card(index as usize, |p| *color_field(p, field) = color)
        });
        on!(on_set_flag, |this, index, field, on| {
            if field == FlagField::BestQuality && on {
                this.install_ai_tool(AiTool::Enlarge(upscale::Quality::Best));
            }
            this.edit_card(index as usize, |p| match field {
                FlagField::MirrorHorizontal => p.mirror_horizontal = on,
                FlagField::MirrorVertical => p.mirror_vertical = on,
                FlagField::Enlarge => p.enlarge = on,
                FlagField::BestQuality => p.best_quality = on,
                FlagField::FillBackground => p.fill_background = on,
                FlagField::RotateFill => p.rotate_fill = on,
            })
        });
        on!(on_set_text, |this, index, text| this.edit_card(index as usize, |p| p.text = text));
        on!(on_set_border_style, |this, index, style| {
            this.edit_card(index as usize, |p| p.border_index = style)
        });
        on!(on_set_text_effect, |this, index, effect| {
            this.edit_card(index as usize, |p| p.text_effect = effect)
        });
        on!(on_preview_clicked, |this, x, y| this.pick_color(x, y));
        on!(on_rotate_left, |this| this.change_view(ViewTransform::rotate_counter_clockwise));
        on!(on_rotate_right, |this| this.change_view(ViewTransform::rotate_clockwise));
        on!(on_flip_horizontal, |this| this.change_view(ViewTransform::flip_horizontal));
        on!(on_flip_vertical, |this| this.change_view(ViewTransform::flip_vertical));
        on!(on_reset_view, |this| this.change_view(|_| ViewTransform::default()));
        on!(on_crop_dragged, |this, left, top, right, bottom| {
            this.drag_crop(Region { left, top, right, bottom })
        });
        on!(on_text_dragged, |this, x, y| this.drag_text(x, y));
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
            let mut dialog = rfd::AsyncFileDialog::new()
                .set_title(this.text(|s| s.get_open_dialog_title()))
                .add_filter(this.text(|s| s.get_pictures()), IMAGE_EXTENSIONS)
                .add_filter(this.text(|s| s.get_all_files()), &["*"]);
            if let Some(folder) = Settings::load_default().last_open_folder.filter(|f| f.is_dir()) {
                dialog = dialog.set_directory(folder);
            }
            let Some(file) = dialog.pick_file().await else { return };
            let path = file.path().to_owned();
            if let Some(folder) = path.parent().map(Path::to_owned) {
                Settings::update(|s| s.last_open_folder = Some(folder));
            }
            this.load(path).await;
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
            state.rendered = None;
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
            let folder = Settings::load_default()
                .last_save_folder
                .filter(|f| f.is_dir())
                .or_else(|| path.parent().map(Path::to_owned));
            if let Some(folder) = folder {
                dialog = dialog.set_directory(folder);
            }
            let Some(file) = dialog.save_file().await else { return };
            let target = file.path().to_owned();
            if let Some(folder) = target.parent().map(Path::to_owned) {
                Settings::update(|s| s.last_save_folder = Some(folder));
            }

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
        if kind == ModKind::Upscale {
            self.install_ai_tool(AiTool::Enlarge(upscale::Quality::Fast));
        }
        if kind == ModKind::ImproveFaces {
            self.install_ai_tool(AiTool::Faces);
        }
        if kind == ModKind::RemoveBackground {
            self.install_ai_tool(AiTool::Background);
        }
        if kind == ModKind::Colourise {
            self.install_ai_tool(AiTool::Colourise);
        }

        // The new card works on the current result until the next render reports its size
        self.with_state(|s| {
            let mut sizes: Vec<ImageSize> = s.get_input_sizes().iter().collect();
            sizes.resize(self.cards.row_count() - 1, ImageSize { width, height });
            sizes.push(ImageSize { width, height });
            s.set_input_sizes(ModelRc::new(VecModel::from(sizes)));
        });
        self.cards_changed();
    }

    /// Downloads the models of an AI operation the first time it is asked for: when its card
    /// is added, or for the best enlarging quality when that switch is turned on. Until they
    /// are there, the card falls back to the fast enlarging model or leaves the image as is.
    fn install_ai_tool(self: &Rc<Self>, tool: AiTool) {
        if tool.is_installed() || self.ai_downloads.borrow().contains(&tool) {
            return;
        }
        self.ai_downloads.borrow_mut().push(tool);
        self.show_downloads();
        self.with_state(|s| match tool {
            AiTool::Enlarge(_) => s.set_upscaler_download_progress(0.0),
            AiTool::Faces => s.set_faces_download_progress(0.0),
            AiTool::Background => s.set_background_download_progress(0.0),
            AiTool::Colourise => s.set_colourise_download_progress(0.0),
        });
        self.notify(ToastKind::Info, |s| match tool {
            AiTool::Enlarge(upscale::Quality::Fast) => s.get_upscaler_downloading(),
            AiTool::Enlarge(upscale::Quality::Best) => s.get_upscaler_best_downloading(),
            AiTool::Faces => s.get_faces_downloading(),
            AiTool::Background => s.get_background_downloading(),
            AiTool::Colourise => s.get_colourise_downloading(),
        });
        let this = self.clone();
        let _ = slint::spawn_local(async move {
            let ui = this.ui.clone();
            let installed = run_in_background(move || {
                tool.install(&|fraction| {
                    let _ = ui.upgrade_in_event_loop(move |ui| {
                        let state = ui.global::<AppState>();
                        match tool {
                            AiTool::Enlarge(_) => state.set_upscaler_download_progress(fraction),
                            AiTool::Faces => state.set_faces_download_progress(fraction),
                            AiTool::Background => state.set_background_download_progress(fraction),
                            AiTool::Colourise => state.set_colourise_download_progress(fraction),
                        }
                    });
                })
            })
            .await;
            this.ai_downloads.borrow_mut().retain(|t| *t != tool);
            this.show_downloads();
            this.with_state(crate::show_ai_models_size);
            match installed {
                Ok(()) => {
                    this.notify(ToastKind::Success, |s| match tool {
                        AiTool::Enlarge(_) => s.get_upscaler_ready(),
                        AiTool::Faces => s.get_faces_ready(),
                        AiTool::Background => s.get_background_ready(),
                        AiTool::Colourise => s.get_colourise_ready(),
                    });
                    // The cards now really change the image
                    this.cards_changed();
                }
                Err(err) => {
                    eprintln!("{err:#}");
                    this.notify(ToastKind::Error, |s| match tool {
                        AiTool::Enlarge(_) => s.get_upscaler_failed(),
                        AiTool::Faces => s.get_faces_failed(),
                        AiTool::Background => s.get_background_failed(),
                        AiTool::Colourise => s.get_colourise_failed(),
                    });
                }
            }
        });
    }

    fn show_downloads(&self) {
        let downloads = self.ai_downloads.borrow();
        let enlarging = downloads.iter().any(|t| matches!(t, AiTool::Enlarge(_)));
        let faces = downloads.contains(&AiTool::Faces);
        let background = downloads.contains(&AiTool::Background);
        let colourise = downloads.contains(&AiTool::Colourise);
        self.with_state(|s| {
            s.set_upscaler_downloading(enlarging);
            s.set_faces_downloading(faces);
            s.set_background_downloading(background);
            s.set_colourise_downloading(colourise);
        });
    }

    fn toggle_card(self: &Rc<Self>, index: usize) {
        if let Some(mut card) = self.cards.row_data(index) {
            card.open = !card.open;
            self.cards.set_row_data(index, card);
            self.update_eyedropper();
            self.update_crop_frame();
            // Opening or closing a crop card switches between the image before and after it
            self.request_render();
        }
    }

    fn delete_card(self: &Rc<Self>, index: usize) {
        if index < self.cards.row_count() {
            self.cards.remove(index);
            // The indices after it shift, so drop a pending eyedropper pick
            self.with_state(|s| s.set_eyedropper_index(-1));
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
        self.shared.card_count.store(modifications.len(), Ordering::SeqCst);
        self.state.borrow_mut().modifications = modifications;
        self.update_eyedropper();
        self.update_crop_frame();
        self.request_render();
    }

    /// The stack the preview shows: while a crop card is open, the image going into it, so
    /// the whole image can be seen and the crop frame dragged over it. While a text card is
    /// open, the image up to that card, so the text grip sits on the text.
    fn shown_modifications(&self) -> Vec<Modification> {
        let modifications = &self.state.borrow().modifications;
        let end = shown_end(&*self.cards).min(modifications.len());
        modifications[..end].to_vec()
    }

    fn update_crop_frame(&self) {
        if let Some(ui) = self.ui.upgrade() {
            update_crop_frame(&ui, *self.shared.transform.lock().unwrap());
        }
    }

    /// Dragging a handle of the crop frame; `shown` is the new frame in fractions of the
    /// preview.
    fn drag_crop(self: &Rc<Self>, shown: Region) {
        let Some(index) = editing_crop(&*self.cards) else { return };
        let Some(size) = self.with_state(|s| s.get_input_sizes().row_data(index)).flatten() else { return };
        let (width, height) = (size.width as f32, size.height as f32);
        if width < 2.0 || height < 2.0 {
            return;
        }
        let source = self.shared.transform.lock().unwrap().region_to_source(shown);
        self.edit_card(index, |p| {
            if let (false, Some(ratio)) = (p.enlarge, crop_aspect::ratio(p.aspect_index, size.width as u32, size.height as u32)) {
                let before = CropBox { x0: p.left, y0: p.top, x1: width - p.right, y1: height - p.bottom };
                let after = CropBox {
                    x0: source.left * width,
                    y0: source.top * height,
                    x1: source.right * width,
                    y1: source.bottom * height,
                };
                set_crop_box(p, crop_aspect::constrain(before, after, ratio, width, height), width, height);
                return;
            }
            if p.enlarge {
                p.left = (-source.left * width).round().clamp(0.0, width);
                p.right = ((source.right - 1.0) * width).round().clamp(0.0, width);
                p.top = (-source.top * height).round().clamp(0.0, height);
                p.bottom = ((source.bottom - 1.0) * height).round().clamp(0.0, height);
                return;
            }
            // Keep at least one pixel
            p.left = (source.left * width).round().clamp(0.0, width - 1.0);
            p.right = ((1.0 - source.right) * width).round().clamp(0.0, width - 1.0 - p.left);
            p.top = (source.top * height).round().clamp(0.0, height - 1.0);
            p.bottom = ((1.0 - source.bottom) * height).round().clamp(0.0, height - 1.0 - p.top);
        });
    }

    /// Picks a fixed shape for the crop card: the largest crop of that shape in the middle.
    fn set_aspect(self: &Rc<Self>, index: usize, preset: i32) {
        let Some(size) = self.with_state(|s| s.get_input_sizes().row_data(index)).flatten() else { return };
        let (width, height) = (size.width.max(1) as f32, size.height.max(1) as f32);
        self.edit_card(index, |p| {
            p.aspect_index = preset;
            if let Some(ratio) = crop_aspect::ratio(preset, size.width as u32, size.height as u32) {
                set_crop_box(p, crop_aspect::largest_centred(width, height, ratio), width, height);
            }
        });
    }

    /// Dragging the text grip; `x` and `y` go from 0 to 1 over the shown preview.
    fn drag_text(self: &Rc<Self>, x: f32, y: f32) {
        let Some(index) = editing_text(&*self.cards) else { return };
        let (x, y) = self.shared.transform.lock().unwrap().to_source((x, y));
        self.edit_card(index, |p| {
            p.x_percent = (x * 100.0).clamp(0.0, 100.0);
            p.y_percent = (y * 100.0).clamp(0.0, 100.0);
        });
    }

    /// Open cards with a colour picker, and which of their colours the eyedropper sets
    fn open_color_cards(&self) -> Vec<(usize, ColorField)> {
        (0..self.cards.row_count())
            .filter_map(|index| {
                let card = self.cards.row_data(index).unwrap();
                let field = match card.kind {
                    ModKind::AddBorder => ColorField::Color,
                    ModKind::Annotate => ColorField::TextColor,
                    ModKind::Rotate if !card.params.rotate_fill => ColorField::Background,
                    ModKind::Crop if card.params.enlarge => ColorField::Background,
                    _ => return None,
                };
                card.open.then_some((index, field))
            })
            .collect()
    }

    fn update_eyedropper(&self) {
        let active = !self.open_color_cards().is_empty();
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
        // A pressed eyedropper button picks for its own picker only
        let target = self.with_state(|s| {
            let index = s.get_eyedropper_index();
            s.set_eyedropper_index(-1);
            (index >= 0).then(|| (index as usize, s.get_eyedropper_field()))
        });
        let targets = match target.flatten() {
            Some(target) => vec![target],
            None => self.open_color_cards(),
        };
        for (index, field) in targets {
            self.edit_card(index, |params| *color_field(params, field) = Color::from_rgb_u8(r, g, b));
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
        self.update_crop_frame();
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
            self.detail_worker.render(latest, self.shown_modifications());
        }
    }

    fn request_render(self: &Rc<Self>) {
        let shown = self.shown_modifications();
        {
            let mut state = self.state.borrow_mut();
            if state.document.is_none() || state.rendered.as_ref() == Some(&shown) {
                return;
            }
            state.rendered = Some(shown.clone());
            let has_modifications = !state.modifications.is_empty();
            self.with_state(|s| {
                s.set_rendering(!shown.is_empty());
                s.set_render_progress(-1.0);
                s.set_has_modified_image(has_modifications);
            });
        }
        let generation = self.shared.latest_generation.fetch_add(1, Ordering::SeqCst) + 1;
        self.worker.render(generation, shown);
        // The sharp tile shows the previous stack, so hide it until it is redone
        self.with_state(|s| s.set_detail_visible(false));
        self.schedule_detail();
    }
}

fn default_params(kind: ModKind, image_width: i32, image_height: i32) -> ModParams {
    let mut params = ModParams { border_index: -1, ..Default::default() };
    match kind {
        ModKind::AddBorder => {
            params.thickness = 5.0;
            params.color = Color::from_rgb_u8(0x22, 0x5e, 0xff);
        }
        ModKind::AddImageBorder => params.border_width = 5.0,
        ModKind::Brightness => {
            params.factor = 1.1;
            params.contrast = 1.1;
            params.saturation = 1.0;
        }
        ModKind::Sepia | ModKind::Greyscale => params.strength = 100.0,
        ModKind::Mirror => {}
        ModKind::Upscale => params.upscale_factor = 2.0,
        ModKind::ImproveFaces => params.strength = 80.0,
        ModKind::Colourise => params.strength = 100.0,
        ModKind::RemoveBackground => params.background = Color::from_rgb_u8(255, 255, 255),
        ModKind::Rotate | ModKind::Crop => params.background = Color::from_rgb_u8(255, 255, 255),
        ModKind::Resize => params.size = image_width.max(image_height).min(10_000) as f32,
        ModKind::Annotate => {
            params.font_size = 4.0;
            params.x_percent = 50.0;
            params.y_percent = 95.0;
            params.text_color = Color::from_rgb_u8(0, 0, 0);
        }
    }
    params
}

fn set_field(params: &mut ModParams, field: ModField, value: f32) {
    let target = match field {
        ModField::Thickness => &mut params.thickness,
        ModField::BorderWidth => &mut params.border_width,
        ModField::Factor => &mut params.factor,
        ModField::Contrast => &mut params.contrast,
        ModField::Angle => &mut params.angle,
        ModField::UpscaleFactor => &mut params.upscale_factor,
        ModField::Strength => &mut params.strength,
        ModField::Saturation => &mut params.saturation,
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

/// Sets the sides of a crop card from the part of a `width` × `height` image it keeps.
fn set_crop_box(p: &mut ModParams, crop: CropBox, width: f32, height: f32) {
    p.left = crop.x0.round().clamp(0.0, width - 1.0);
    p.top = crop.y0.round().clamp(0.0, height - 1.0);
    p.right = (width - crop.x1).round().clamp(0.0, width - 1.0 - p.left);
    p.bottom = (height - crop.y1).round().clamp(0.0, height - 1.0 - p.top);
}

fn color_field(params: &mut ModParams, field: ColorField) -> &mut Color {
    match field {
        ColorField::Color => &mut params.color,
        ColorField::TextColor => &mut params.text_color,
        ColorField::Background => &mut params.background,
    }
}

fn to_modification(card: &ModEntry, borders: &[String]) -> Modification {
    let p = &card.params;
    let px = |value: f32| value.round().max(0.0) as u32;
    let rgba = |color: Color| Rgba([color.red(), color.green(), color.blue(), 255]);
    match card.kind {
        ModKind::AddBorder => Modification::AddBorder {
            percent: p.thickness.max(0.0),
            color: rgba(p.color),
        },
        ModKind::AddImageBorder => Modification::AddImageBorder {
            border: usize::try_from(p.border_index).ok().and_then(|i| borders.get(i)).cloned(),
            percent: p.border_width.max(0.0),
        },
        ModKind::Brightness => {
            Modification::Brightness { factor: p.factor, contrast: p.contrast, saturation: p.saturation }
        }
        ModKind::Sepia => Modification::Sepia { strength: p.strength / 100.0 },
        ModKind::Greyscale => Modification::Greyscale { strength: p.strength / 100.0 },
        ModKind::Rotate => {
            Modification::Rotate { angle: p.angle, background: rgba(p.background), fill: p.rotate_fill }
        }
        ModKind::Mirror => Modification::Mirror { horizontal: p.mirror_horizontal, vertical: p.mirror_vertical },
        // Leaves the image as is until the models are downloaded
        ModKind::ImproveFaces if !faces::is_installed() => Modification::ImproveFaces { strength: 0.0 },
        ModKind::ImproveFaces => Modification::ImproveFaces { strength: (p.strength / 100.0).clamp(0.0, 1.0) },
        ModKind::Colourise if !colourise::is_installed() => Modification::Colourise { strength: 0.0 },
        ModKind::Colourise => Modification::Colourise { strength: (p.strength / 100.0).clamp(0.0, 1.0) },
        ModKind::RemoveBackground => Modification::RemoveBackground {
            fill: p.fill_background.then(|| rgba(p.background)),
            downloaded: background::is_installed(),
        },
        ModKind::Upscale => {
            let wanted = if p.best_quality { upscale::Quality::Best } else { upscale::Quality::Fast };
            // Uses the fast model while the best one downloads, and leaves the image as is
            // until either is there
            match [wanted, upscale::Quality::Fast].into_iter().find(|q| upscale::is_installed(*q)) {
                Some(quality) => {
                    Modification::Upscale { factor: p.upscale_factor.round().clamp(2.0, 4.0) as u32, quality }
                }
                None => Modification::Upscale { factor: 1, quality: wanted },
            }
        }
        ModKind::Resize => Modification::Resize { size: px(p.size) },
        ModKind::Annotate => Modification::Annotate {
            text: p.text.to_string(),
            font_percent: p.font_size,
            x_percent: p.x_percent,
            y_percent: p.y_percent,
            color: rgba(p.text_color),
            effect: match p.text_effect {
                1 => TextEffect::Outline,
                2 => TextEffect::Shadow,
                _ => TextEffect::None,
            },
        },
        ModKind::Crop if p.enlarge => Modification::Extend {
            top: px(p.top),
            right: px(p.right),
            bottom: px(p.bottom),
            left: px(p.left),
            background: rgba(p.background),
        },
        ModKind::Crop => Modification::Crop {
            top: px(p.top),
            right: px(p.right),
            bottom: px(p.bottom),
            left: px(p.left),
        },
    }
}

/// An AI operation, for downloading its models
#[derive(Debug, Clone, Copy, PartialEq)]
enum AiTool {
    Enlarge(upscale::Quality),
    Faces,
    Background,
    Colourise,
}

impl AiTool {
    fn is_installed(self) -> bool {
        match self {
            AiTool::Enlarge(quality) => upscale::is_installed(quality),
            AiTool::Faces => faces::is_installed(),
            AiTool::Background => background::is_installed(),
            AiTool::Colourise => colourise::is_installed(),
        }
    }

    fn install(self, progress: &dyn Fn(f32)) -> anyhow::Result<()> {
        match self {
            AiTool::Enlarge(quality) => upscale::install(quality, progress),
            AiTool::Faces => faces::install(progress),
            AiTool::Background => background::install(progress),
            AiTool::Colourise => colourise::install(progress),
        }
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
        // Stopped before an open crop card: the result is not the final image
        let partial = output.input_sizes.len() < shared.card_count.load(Ordering::SeqCst);
        if shared.estimate_file_size.load(Ordering::SeqCst) && !partial {
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
            let to_size = |size: &Dimensions| ImageSize { width: size.width as i32, height: size.height as i32 };
            let mut sizes: Vec<ImageSize> = output.input_sizes.iter().map(to_size).collect();
            if partial {
                // The cards after the crop keep their sizes from the last full render
                sizes.push(to_size(&output.logical));
                sizes.extend(state.get_input_sizes().iter().skip(sizes.len()));
            } else {
                state.set_image_width(output.logical.width as i32);
                state.set_image_height(output.logical.height as i32);
            }
            state.set_input_sizes(ModelRc::new(VecModel::from(sizes)));
            state.set_rendering(false);
            state.set_render_progress(-1.0);
            update_crop_frame(&ui, *shared.transform.lock().unwrap());
        });
    }
}

/// The first open crop card, whose frame is shown over the preview
fn editing_crop(cards: &impl Model<Data = ModEntry>) -> Option<usize> {
    cards.iter().position(|card| card.open && card.kind == ModKind::Crop)
}

/// The first open text card, whose grip is shown over the preview
fn editing_text(cards: &impl Model<Data = ModEntry>) -> Option<usize> {
    cards.iter().position(|card| card.open && card.kind == ModKind::Annotate)
}

/// How many cards the preview shows: up to an open crop card, or up to and including an
/// open text card
fn shown_end(cards: &impl Model<Data = ModEntry>) -> usize {
    let crop = editing_crop(cards).unwrap_or(usize::MAX);
    let text = editing_text(cards).map_or(usize::MAX, |index| index + 1);
    crop.min(text).min(cards.row_count())
}

/// Shows the frame of the crop card being edited and the grip of the text card being edited,
/// in fractions of the preview.
fn update_crop_frame(ui: &AppWindow, transform: ViewTransform) {
    let state = ui.global::<AppState>();
    let cards = state.get_modifications();
    // Hidden when an open text card before it stops the preview earlier
    let frame = editing_crop(&cards).filter(|&index| index == shown_end(&cards)).and_then(|index| {
        let size = state.get_input_sizes().row_data(index)?;
        let params = cards.row_data(index)?.params;
        let (width, height) = (size.width.max(1) as f32, size.height.max(1) as f32);
        // Enlarging puts the frame outside the image, at most one image size away
        let source = if params.enlarge {
            Region {
                left: -(params.left / width).clamp(0.0, 1.0),
                top: -(params.top / height).clamp(0.0, 1.0),
                right: 1.0 + (params.right / width).clamp(0.0, 1.0),
                bottom: 1.0 + (params.bottom / height).clamp(0.0, 1.0),
            }
        } else {
            Region {
                left: (params.left / width).clamp(0.0, 1.0),
                top: (params.top / height).clamp(0.0, 1.0),
                right: (1.0 - params.right / width).clamp(0.0, 1.0),
                bottom: (1.0 - params.bottom / height).clamp(0.0, 1.0),
            }
        };
        state.set_crop_enlarge(params.enlarge);
        state.set_crop_background(params.background);
        Some(transform.region_from_source(source))
    });
    state.set_crop_active(frame.is_some());
    if let Some(frame) = frame {
        state.set_crop_left(frame.left);
        state.set_crop_top(frame.top);
        state.set_crop_right(frame.right);
        state.set_crop_bottom(frame.bottom);
    }

    // The text grip, unless a crop card before it hides the text
    let text = editing_text(&cards).filter(|&index| index < shown_end(&cards)).and_then(|index| {
        let params = cards.row_data(index)?.params;
        let (x, y) = (params.x_percent / 100.0, params.y_percent / 100.0);
        Some(transform.region_from_source(Region { left: x, top: y, right: x, bottom: y }))
    });
    state.set_text_active(text.is_some());
    if let Some(point) = text {
        state.set_text_x(point.left);
        state.set_text_y(point.top);
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

/// Border style `name` around a plain picture, at twice the size it is shown at
fn border_thumbnail(name: &str) -> RgbaImage {
    let (width, height) = BORDER_THUMBNAIL_SIZE;
    let plain = RgbaImage::from_pixel(width, height, Rgba([0xdd, 0xe6, 0xee, 255]));
    pipeline::image_border::add_image_border(&plain, name, height as f32 * 0.3).unwrap_or(plain)
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
