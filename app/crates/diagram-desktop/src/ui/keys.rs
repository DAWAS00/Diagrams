//! Keyboard routing at the start of a frame: Escape in priority order, then
//! the catalog shortcuts. It only queues intents.
//!
//! Moved out of `shell::show` unchanged.

use eframe::egui::{self, Key, Modifiers};

use crate::shell::ShellState;
use crate::ui::commands;
use crate::ui::intents::UiIntent;

/// Escape in priority order (showcase, then drawer), then the catalog
/// shortcuts, using the text focus of the previous frame.
pub(crate) fn route_keys(ui: &mut egui::Ui, state: &mut ShellState) {
    // Shortcuts use the text focus of the previous frame: egui drops focus in
    // the frame Escape is pressed, and Escape must first leave the field.
    // An open drawer takes Escape before the canvas selection does.
    let text_had_focus = state.text_had_focus();
    if !state.has_dialog()
        && state.showcase.is_some()
        && !text_had_focus
        && ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape))
    {
        state.queue(UiIntent::CloseShowcase);
    }
    if !state.has_dialog()
        && state.drawer.is_some()
        && !text_had_focus
        && ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape))
    {
        state.queue(UiIntent::CloseDrawer);
    }
    // A dialog owns the keyboard: shortcuts do nothing behind it.
    if !state.has_dialog()
        && let Some(id) = ui.input_mut(|input| commands::take_shortcut(input, text_had_focus))
    {
        state.queue(UiIntent::run(id));
    }
}
