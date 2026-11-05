//! Status bar (spec §2, brief §A8): save state, zoom, grid and snap in
//! words, the selection summary with the tool hint, and the theme chooser.
//! Every element is text, never color alone. Grid, Snap and zoom controls
//! wait for their `CommandView`s (request 7 in AGENT-A-UI.md §6).

use eframe::egui::{self, Align, Layout, RichText, ThemePreference};

use super::components;
use super::theme::TextRole;
use crate::intent::UiIntent;
use crate::presentation::StatusView;

/// The theme chooser's options, in visual order.
pub const THEME_CHOICES: [(ThemePreference, &str); 3] = [
    (ThemePreference::Light, "Light"),
    (ThemePreference::Dark, "Dark"),
    (ThemePreference::System, "System"),
];

/// Draws the status bar. `in_workspace` adds the selection summary, which
/// the welcome screens do not have.
pub fn show(ui: &mut egui::Ui, view: &StatusView, in_workspace: bool, out: &mut Vec<UiIntent>) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(view.save_text).text_style(TextRole::Label.text_style()));
        ui.separator();
        if in_workspace {
            // Grid, snap and zoom in words, never color alone.
            let on_off = |on: bool| if on { "on" } else { "off" };
            let facts = format!(
                "{} · Grid {} · Snap {}",
                view.zoom_text,
                on_off(view.grid),
                on_off(view.snap)
            );
            ui.label(RichText::new(facts).text_style(TextRole::Label.text_style()));
            ui.separator();
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            // Right-to-left layout: add the options in reverse visual order.
            for (preference, label) in THEME_CHOICES.iter().rev() {
                if components::choice(ui, view.theme == *preference, label).clicked() {
                    out.push(UiIntent::SetTheme(*preference));
                }
            }
            components::field_label(ui, "Theme:");
            if in_workspace {
                ui.separator();
                components::truncated_text(ui, &view.selection_text);
                if !view.tool_hint.is_empty() {
                    ui.separator();
                    components::truncated_text(ui, view.tool_hint);
                }
            }
        });
    });
}
