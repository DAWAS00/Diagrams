//! The project panel: search, diagrams (views) and model elements.
//!
//! Moved out of `shell.rs` unchanged.

use eframe::egui;

use crate::shell::{ShellState, relation_label};
use crate::ui::components;
use crate::ui::intents::UiIntent;
use crate::ui::navigator;
use crate::ui::theme;

/// Project panel: search, the project's diagrams (views) and its model
/// elements, with distinct "nothing yet" and "no match" states.
pub(crate) fn sidebar(ui: &mut egui::Ui, state: &mut ShellState) {
    components::panel_title(ui, "Project");
    components::text_field(ui, "Search project", &mut state.navigator_search);
    let query = state.navigator_search.clone();
    let searching = !query.trim().is_empty();

    ui.add_space(theme::SPACING);
    components::field_label(ui, "Diagrams (views of the model)");
    let diagram_hits: Vec<usize> = (0..state.diagrams.len())
        .filter(|&index| navigator::matches(&query, &state.diagrams[index].name))
        .collect();
    if state.diagrams.is_empty() {
        ui.label("No diagrams yet.");
    } else if diagram_hits.is_empty() {
        ui.label(format!("No diagrams match \"{}\".", query.trim()));
    }
    let active = state.active_diagram();
    let mut open = None;
    ui.indent("diagrams", |ui| {
        for index in diagram_hits {
            let name = &state.diagrams[index].name;
            let row = components::list_row(ui, active == Some(index), name)
                .on_hover_text("Open this diagram in a tab");
            if row.clicked() {
                open = Some(index);
            }
        }
    });
    if let Some(index) = open {
        state.queue(UiIntent::OpenDiagram(index));
    }

    ui.add_space(theme::SPACING);
    components::field_label(ui, "Model (shared by all diagrams)");
    let class_hits: Vec<usize> = (0..state.classes.len())
        .filter(|&index| navigator::matches(&query, state.classes[index].display_name()))
        .collect();
    if state.classes.is_empty() {
        ui.label("No classes yet. Classes appear here when you add them to a diagram.");
    } else if class_hits.is_empty() {
        ui.label(format!("No classes match \"{}\".", query.trim()));
    }
    let mut reveal = None;
    let mut toggle = None;
    let chosen = state.selected_classes();
    let adding = ui.input(|input| input.modifiers.command);
    ui.indent("model", |ui| {
        for index in class_hits {
            let label = state.classes[index].display_name().to_owned();
            let row =
                components::list_row(ui, chosen.contains(&index), &label).on_hover_text(format!(
                    "{}. Ctrl+click or Ctrl+Enter adds it to the selection.",
                    navigator::shown_in(&state.diagrams, index)
                ));
            if row.clicked() {
                if adding {
                    toggle = Some(index);
                } else {
                    reveal = Some(index);
                }
            }
        }
    });
    components::field_label(ui, "Relationships");
    let relation_hits: Vec<(usize, String)> = state
        .relations
        .iter()
        .enumerate()
        .map(|(index, relation)| (index, relation_label(state, relation)))
        .filter(|(_, label)| navigator::matches(&query, label))
        .collect();
    if state.relations.is_empty() {
        ui.label("No relationships yet.");
    } else if relation_hits.is_empty() {
        ui.label(format!("No relationships match \"{}\".", query.trim()));
    }
    let mut relation = None;
    ui.indent("relationships", |ui| {
        for (index, label) in relation_hits {
            if components::list_row(ui, state.selected_relation == Some(index), &label).clicked() {
                relation = Some(index);
            }
        }
    });
    if let Some(index) = toggle {
        state.queue(UiIntent::ToggleClassInSelection(index));
    }
    if let Some(index) = relation {
        state.queue(UiIntent::SelectRelation(index));
    }
    if searching {
        ui.add_space(theme::GAP);
        if components::action_button(ui, "Clear search", None, None).clicked() {
            state.navigator_search.clear();
        }
    }
    if let Some(index) = reveal {
        state.queue(UiIntent::RevealClass(index));
    }
}
