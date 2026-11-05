//! Project panel (spec §2 and §3, brief §A5): the project's diagrams and
//! model objects. Rows are at least 32 high and shorten long names with an
//! ellipsis while keeping the full name as their accessible name. Clicking a
//! model row selects its object (Shift toggles).
//!
//! Each list has a designed empty state.

use eframe::egui;

use super::components;
use super::theme;
use crate::intent::UiIntent;
use crate::presentation::{NavigatorView, RowView};

/// Shown when the project has no diagrams.
pub const NO_DIAGRAMS: &str = "No diagrams yet.";
/// Shown when the project has no model objects.
pub const NO_MODELS: &str = "No classes yet. Classes you add appear here.";

/// Draws the project panel contents.
pub fn show(ui: &mut egui::Ui, view: &NavigatorView, out: &mut Vec<UiIntent>) {
    components::panel_title(ui, view.title);
    section(ui, "Diagrams", "diagrams", &view.diagrams, NO_DIAGRAMS, out);
    ui.add_space(theme::SPACING);
    section(ui, "Model", "model", &view.model, NO_MODELS, out);
}

fn section(
    ui: &mut egui::Ui,
    title: &str,
    id: &str,
    rows: &[RowView],
    empty: &str,
    out: &mut Vec<UiIntent>,
) {
    components::field_label(ui, title);
    ui.indent(id, |ui| {
        if rows.is_empty() {
            ui.label(empty);
        }
        for row in rows {
            let response = components::list_row(ui, row.selected, &row.label);
            if response.clicked()
                && let Some(target) = row.target
            {
                let toggle = ui.input(|input| input.modifiers.shift);
                out.push(UiIntent::Select { target, toggle });
            }
        }
    });
}
