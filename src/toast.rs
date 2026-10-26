//! Notifications shown in the bottom left corner for a few seconds.

use std::time::Duration;

use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::{AppState, AppWindow, Toast, ToastKind};

const VISIBLE_FOR: Duration = Duration::from_secs(5);

/// Gives the window an empty toast list. Call once at startup.
pub fn init(ui: &AppWindow) {
    ui.global::<AppState>().set_toasts(ModelRc::new(VecModel::<Toast>::default()));
}

pub fn show(ui: &AppWindow, kind: ToastKind, message: &str) {
    let toasts = ui.global::<AppState>().get_toasts();
    let Some(list) = toasts.as_any().downcast_ref::<VecModel<Toast>>() else { return };
    let id = list.iter().map(|toast| toast.id).max().unwrap_or(0) + 1;
    list.push(Toast { id, kind, message: message.into() });

    let ui = ui.as_weak();
    slint::Timer::single_shot(VISIBLE_FOR, move || {
        let Some(ui) = ui.upgrade() else { return };
        let toasts = ui.global::<AppState>().get_toasts();
        if let Some(list) = toasts.as_any().downcast_ref::<VecModel<Toast>>()
            && let Some(index) = list.iter().position(|toast| toast.id == id)
        {
            list.remove(index);
        }
    });
}
