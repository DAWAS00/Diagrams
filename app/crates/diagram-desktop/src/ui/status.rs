//! Status bar (moved out of shell.rs; no behaviour change).

use eframe::egui::{self, Align, Layout, ThemePreference};

use super::class_tool::Tool;
use super::components;
use super::intents::UiIntent;
use super::theme::TextRole;
use crate::shell::ShellState;

/// What to do next while the Class tool is active.
pub const CLASS_TOOL_HINT: &str = "Class tool: click the canvas, or press Enter on it, to place a class. Escape returns to Select.";

/// Status bar. `in_workspace` adds the selection, which entry screens do not have.
pub(crate) fn status_bar(ui: &mut egui::Ui, state: &mut ShellState, in_workspace: bool) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Prototype · nothing is saved")
                .text_style(TextRole::Label.text_style()),
        );
        ui.separator();
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            // Right-to-left layout: add the options in reverse visual order.
            for (preference, label) in [
                (ThemePreference::System, "System"),
                (ThemePreference::Dark, "Dark"),
                (ThemePreference::Light, "Light"),
            ] {
                if components::choice(ui, state.theme == preference, label).clicked() {
                    state.queue(UiIntent::SetTheme(preference));
                }
            }
            components::field_label(ui, "Theme:");
            if in_workspace {
                ui.separator();
                let selection = state.selection_summary();
                components::truncated_text(ui, &selection);
                if state.active_tool() == Tool::Class {
                    ui.separator();
                    components::truncated_text(ui, CLASS_TOOL_HINT).on_hover_text(CLASS_TOOL_HINT);
                }
                if let Some(notice) = state.notice() {
                    ui.separator();
                    components::truncated_text(ui, notice).on_hover_text(notice);
                }
            }
        });
    });
}
