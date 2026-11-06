//! The project panel: search, diagrams (views) and model elements.
//!
//! Every row carries the typed ID of the record it shows, taken from the
//! read model, so a click names its target without a list position.

use eframe::egui;

use crate::shell::ShellState;
use crate::ui::components;
use crate::ui::intents::UiIntent;
use crate::ui::navigator;
use crate::ui::read_model;
use crate::ui::theme;

/// Project panel: search, the project's diagrams (views) and its model
/// elements, with distinct "nothing yet" and "no match" states.
pub(crate) fn sidebar(ui: &mut egui::Ui, state: &mut ShellState) {
    components::panel_title(ui, "Project");
    components::text_field(ui, "Search project", &mut state.navigator_search);
    let query = state.navigator_search.clone();
    let searching = !query.trim().is_empty();
    let diagrams = read_model::diagram_rows(state.document());
    let elements = read_model::element_rows(state.document());
    let relations = read_model::relation_rows(state.document());

    ui.add_space(theme::SPACING);
    components::field_label(ui, "Diagrams (views of the model)");
    let diagram_hits: Vec<_> = diagrams
        .iter()
        .filter(|row| navigator::matches(&query, &row.name))
        .collect();
    if diagrams.is_empty() {
        ui.label("No diagrams yet.");
    } else if diagram_hits.is_empty() {
        ui.label(format!("No diagrams match \"{}\".", query.trim()));
    }
    let active = state.active_diagram();
    let mut open = None;
    ui.indent("diagrams", |ui| {
        for row in diagram_hits {
            let response = components::list_row(ui, active == Some(row.id), &row.name)
                .on_hover_text("Open this diagram in a tab");
            if response.clicked() {
                open = Some(row.id);
            }
        }
    });
    if let Some(id) = open {
        state.queue(UiIntent::OpenDiagram(id));
    }

    ui.add_space(theme::SPACING);
    components::field_label(ui, "Model (shared by all diagrams)");
    let class_hits: Vec<_> = elements
        .iter()
        .filter(|row| navigator::matches(&query, &row.name))
        .collect();
    if elements.is_empty() {
        ui.label("No classes yet. Classes appear here when you add them to a diagram.");
    } else if class_hits.is_empty() {
        ui.label(format!("No classes match \"{}\".", query.trim()));
    }
    let mut reveal = None;
    let mut toggle = None;
    let chosen = state.selected_elements();
    let adding = ui.input(|input| input.modifiers.command);
    ui.indent("model", |ui| {
        for row in class_hits {
            let response = components::list_row(ui, chosen.contains(&row.id), &row.name)
                .on_hover_text(format!(
                    "{}. Ctrl+click or Ctrl+Enter adds it to the selection.",
                    read_model::shown_in(state.document(), row.id)
                ));
            if response.clicked() {
                if adding {
                    toggle = Some(row.id);
                } else {
                    reveal = Some(row.id);
                }
            }
        }
    });
    components::field_label(ui, "Relationships");
    let relation_hits: Vec<_> = relations
        .iter()
        .filter(|row| navigator::matches(&query, &row.label))
        .collect();
    if relations.is_empty() {
        ui.label("No relationships yet.");
    } else if relation_hits.is_empty() {
        ui.label(format!("No relationships match \"{}\".", query.trim()));
    }
    let mut relation = None;
    ui.indent("relationships", |ui| {
        for row in relation_hits {
            if components::list_row(ui, state.selected_relation() == Some(row.id), &row.label)
                .clicked()
            {
                relation = Some(row.id);
            }
        }
    });
    if let Some(id) = toggle {
        state.queue(UiIntent::ToggleClassInSelection(id));
    }
    if let Some(id) = relation {
        state.queue(UiIntent::SelectRelation(id));
    }
    if searching {
        ui.add_space(theme::GAP);
        if components::action_button(ui, "Clear search", None, None).clicked() {
            state.navigator_search.clear();
        }
    }
    if let Some(id) = reveal {
        state.queue(UiIntent::RevealClass(id));
    }
}
