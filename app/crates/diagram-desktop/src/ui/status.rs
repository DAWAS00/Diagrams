//! Status bar (moved out of shell.rs; no behaviour change).

use eframe::egui::{self, Align, Layout, ThemePreference};

use super::components;
use super::intents::UiIntent;
use super::theme::TextRole;
use crate::shell::{ShellState, relation_label};

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
                let count = state.selected_classes().len();
                let selection = match (state.selected_relation, state.selected_class()) {
                    (Some(index), _) => state.relations.get(index).map_or_else(
                        || String::from("No selection"),
                        |relation| format!("Selected: {}", relation_label(state, relation)),
                    ),
                    (None, Some(_)) if count > 1 => format!("{count} classes selected"),
                    (None, Some(class)) => format!("Selected: {}", class.display_name()),
                    (None, None) => String::from("No selection"),
                };
                components::truncated_text(ui, &selection);
            }
        });
    });
}
