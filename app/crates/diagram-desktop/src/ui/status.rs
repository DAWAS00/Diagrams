//! Status bar (spec §2, brief §A8): save state in words, the selection
//! summary, and the theme chooser. Every element is text, never color alone.

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
            }
        });
    });
}
