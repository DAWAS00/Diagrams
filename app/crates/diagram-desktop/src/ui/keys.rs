//! Keyboard routing at the start of a frame: Escape in priority order, then
//! the catalog shortcuts. It only queues intents.
//!
//! Moved out of `shell::show` unchanged.

use eframe::egui::{self, Key, Modifiers};

use crate::shell::ShellState;
use crate::ui::class_tool::{Phase, Tool};
use crate::ui::commands::{self, CommandId};
use crate::ui::intents::UiIntent;

/// Escape in priority order (showcase, drawer, class naming, Class tool),
/// then the catalog shortcuts, using the text focus of the previous frame.
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
    // Escape while naming a new class cancels it, even before its field has
    // focus; with the field focused, the field itself cancels.
    let naming = state
        .placement()
        .filter(|placement| placement.phase == Phase::Naming)
        .map(|placement| placement.id);
    if let Some(placement) = naming
        && !state.has_dialog()
        && !text_had_focus
        && ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape))
    {
        state.queue(UiIntent::CancelNewClass(placement));
    }
    // Then an armed Class tool goes back to Select, before the selection.
    if state.active_tool() == Tool::Class
        && state.placement().is_none()
        && !state.has_dialog()
        && !text_had_focus
        && ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape))
    {
        state.queue(UiIntent::run(CommandId::SelectTool));
    }
    // A dialog owns the keyboard: shortcuts do nothing behind it. While a
    // class is being named, the keys a text field uses stay with it.
    let text_owns = text_had_focus || naming.is_some();
    if !state.has_dialog()
        && let Some(id) = ui.input_mut(|input| commands::take_shortcut(input, text_owns))
    {
        state.queue(UiIntent::run(id));
    }
}
