//! Global shortcuts for the layout editor and overlay visibility, set in Settings.

use std::sync::atomic::Ordering;
use std::sync::Mutex;

use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent, ShortcutState};

use crate::{commands, AppState};

pub const DEFAULT_EDIT: &str = "Ctrl+Shift+F9";
pub const DEFAULT_VISIBILITY: &str = "Ctrl+Shift+F10";

#[derive(Clone, Copy, PartialEq)]
pub enum Action {
    Edit,
    Visibility,
}

impl Action {
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "edit" => Some(Self::Edit),
            "visibility" => Some(Self::Visibility),
            _ => None,
        }
    }
}

/// Shortcuts currently registered for each action.
#[derive(Default)]
pub struct Shortcuts(Mutex<[Option<Shortcut>; 2]>);

fn slot(action: Action) -> usize {
    action as usize
}

pub fn handle(app: &AppHandle, shortcut: &Shortcut, event: ShortcutEvent) {
    if event.state() != ShortcutState::Pressed {
        return;
    }
    let Some(state) = app.try_state::<AppState>() else { return };
    let current = *state.shortcuts.0.lock().unwrap_or_else(|e| e.into_inner());
    if current[slot(Action::Edit)].as_ref() == Some(shortcut) {
        commands::apply_edit_mode(app, !state.edit_mode.load(Ordering::Acquire));
    } else if current[slot(Action::Visibility)].as_ref() == Some(shortcut) {
        commands::apply_overlay_visible(app, !state.overlay_visible.load(Ordering::Acquire));
    }
}

/// Binds `action` to `accelerator` (e.g. "Ctrl+Shift+F9"), replacing its previous shortcut.
/// On failure the previous shortcut stays registered.
pub fn set(app: &AppHandle, action: Action, accelerator: &str) -> Result<(), String> {
    let shortcut: Shortcut = accelerator.parse().map_err(|_| format!("\"{accelerator}\" is not a valid shortcut"))?;
    let state = app.state::<AppState>();
    let mut current = state.shortcuts.0.lock().unwrap_or_else(|e| e.into_inner());
    let other = match action {
        Action::Edit => Action::Visibility,
        Action::Visibility => Action::Edit,
    };
    if current[slot(other)] == Some(shortcut) {
        return Err(format!("{accelerator} is already used by another shortcut"));
    }
    if current[slot(action)] == Some(shortcut) {
        return Ok(());
    }
    let manager = app.global_shortcut();
    manager
        .register(shortcut)
        .map_err(|e| format!("Couldn't register {accelerator}, it may be used by another app: {e}"))?;
    if let Some(old) = current[slot(action)].replace(shortcut) {
        let _ = manager.unregister(old);
    }
    Ok(())
}

/// Registers the stored shortcuts at startup, falling back to the defaults.
pub fn register_initial(app: &AppHandle, edit: Option<&str>, visibility: Option<&str>) {
    for (action, stored, default) in [
        (Action::Edit, edit, DEFAULT_EDIT),
        (Action::Visibility, visibility, DEFAULT_VISIBILITY),
    ] {
        let result = match stored {
            Some(accel) => set(app, action, accel).or_else(|e| {
                eprintln!("shortcut {accel}: {e}; using {default}");
                set(app, action, default)
            }),
            None => set(app, action, default),
        };
        if let Err(e) = result {
            eprintln!("could not register shortcut {default}: {e}");
        }
    }
}
