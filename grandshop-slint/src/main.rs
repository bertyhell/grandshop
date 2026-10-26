// Hide the console window on Windows release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod controller;
mod toast;

use controller::Controller;
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

/// Applies the saved language and lets the settings dialog change it.
fn setup_settings(app: &AppWindow) {
    let path = Settings::default_path();
    let settings = path.as_deref().map(Settings::load).unwrap_or_default();
    let state = app.global::<AppState>();

    let names: Vec<SharedString> =
        LANGUAGES.iter().map(|(_, english, native)| format!("{english} - {native}").into()).collect();
    state.set_languages(ModelRc::new(VecModel::from(names)));
    select_language(&state, settings.effective_language());

    let weak = app.as_weak();
    state.on_save_settings(move |index| {
        let Some(ui) = weak.upgrade() else { return };
        let Some((code, _, _)) = usize::try_from(index).ok().and_then(|i| LANGUAGES.get(i)) else { return };
        select_language(&ui.global::<AppState>(), code);

        let settings = Settings { language: Some((*code).to_owned()) };
        if let Some(path) = &path
            && let Err(err) = settings.save(path)
        {
            eprintln!("saving settings to {}: {err}", path.display());
        }
    });
}

fn select_language(state: &AppState, code: &str) {
    if let Err(err) = slint::select_bundled_translation(code) {
        eprintln!("selecting language {code}: {err}");
    }
    let index = LANGUAGES.iter().position(|(c, _, _)| *c == code).unwrap_or(0);
    state.set_language_index(index as i32);
}
