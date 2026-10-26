// Hide the console window on Windows release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod controller;
mod toast;

use controller::Controller;
use grandshop::ai::{models, runtime};
use grandshop::io::format_file_size;
use grandshop::settings::{Settings, LANGUAGES};
use slint::{ModelRc, SharedString, VecModel};

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let app = AppWindow::new()?;
    let controller = Controller::new(&app);
    setup_settings(&app);

    if let Some(path) = std::env::args_os().nth(1) {
        controller.open_path(path.into());
    }

    app.window().set_maximized(true);
    app.run()
}

/// Applies the saved settings and lets the settings dialog change them.
fn setup_settings(app: &AppWindow) {
    let settings = Settings::load_default();
    let state = app.global::<AppState>();

    let names: Vec<SharedString> =
        LANGUAGES.iter().map(|(_, english, native)| format!("{english} - {native}").into()).collect();
    state.set_languages(ModelRc::new(VecModel::from(names)));
    select_language(&state, settings.effective_language());
    runtime::set_use_gpu(!settings.no_gpu);
    state.set_use_gpu(!settings.no_gpu);
    show_ai_models_size(&state);

    let weak = app.as_weak();
    state.on_save_settings(move |index, use_gpu| {
        let Some(ui) = weak.upgrade() else { return };
        let state = ui.global::<AppState>();
        runtime::set_use_gpu(use_gpu);
        state.set_use_gpu(use_gpu);
        Settings::update(|settings| settings.no_gpu = !use_gpu);
        let Some((code, _, _)) = usize::try_from(index).ok().and_then(|i| LANGUAGES.get(i)) else { return };
        select_language(&state, code);
        Settings::update(|settings| settings.language = Some((*code).to_owned()));
    });

    let weak = app.as_weak();
    state.on_delete_ai_models(move || {
        if let Err(err) = models::delete_all() {
            eprintln!("{err:#}");
        }
        if let Some(ui) = weak.upgrade() {
            show_ai_models_size(&ui.global::<AppState>());
        }
    });
}

pub fn show_ai_models_size(state: &AppState) {
    state.set_ai_models_size(format_file_size(models::installed_bytes()).into());
}

fn select_language(state: &AppState, code: &str) {
    if let Err(err) = slint::select_bundled_translation(code) {
        eprintln!("selecting language {code}: {err}");
    }
    let index = LANGUAGES.iter().position(|(c, _, _)| *c == code).unwrap_or(0);
    state.set_language_index(index as i32);
}
