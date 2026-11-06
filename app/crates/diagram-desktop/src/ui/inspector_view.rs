//! The inspector: nothing selected, one class, several classes or a relationship.
//!
//! Fields edit drafts; committed drafts are queued as intents, which the
//! coordinator turns into `diagram-core` commands.

use eframe::egui::{self, Key};

use diagram_core::ids::{ElementId, RelationId};
use diagram_core::notation::RelationKind;

use crate::shell::ShellState;
use crate::ui::components;
use crate::ui::inspector::{self as rules, DraftKey, DraftOutcome, RelationField};
use crate::ui::intents::UiIntent;
use crate::ui::notation::{self, InspectorSection};
use crate::ui::read_model;
use crate::ui::theme;

/// A text field bound to a draft. Enter or moving focus away commits a valid
/// draft; Escape cancels it; an invalid draft stays with its error.
fn draft_field(
    ui: &mut egui::Ui,
    drafts: &mut std::collections::BTreeMap<DraftKey, String>,
    key: DraftKey,
    label: &str,
    committed: &str,
    error_of: impl Fn(&str) -> Option<String>,
) -> DraftOutcome {
    let editing = drafts.contains_key(&key);
    let mut text = drafts
        .get(&key)
        .cloned()
        .unwrap_or_else(|| committed.to_owned());
    let error = if editing { error_of(&text) } else { None };
    let response = match &error {
        Some(message) => components::text_field_with_error(ui, label, &mut text, message),
        None => components::text_field(ui, label, &mut text),
    };
    if response.changed() {
        drafts.insert(key, text);
    }
    if editing && response.has_focus() {
        components::field_label(ui, "Enter applies the change. Escape cancels it.");
    }
    if !response.lost_focus() {
        return DraftOutcome::Editing;
    }
    if ui.input(|input| input.key_pressed(Key::Escape)) {
        drafts.remove(&key);
        return DraftOutcome::Cancel;
    }
    match drafts.get(&key).cloned() {
        Some(draft) if error_of(&draft).is_none() => {
            drafts.remove(&key);
            if draft == committed {
                DraftOutcome::Editing
            } else {
                DraftOutcome::Commit(draft)
            }
        }
        _ => DraftOutcome::Editing,
    }
}

/// A disclosure button for a collapsed inspector section.
fn section_toggle(ui: &mut egui::Ui, open: &mut bool, title: &str) {
    if components::choice(ui, *open, title).clicked() {
        *open = !*open;
    }
}

/// The view-only sections shared by single and multiple selections.
fn appearance_and_advanced(ui: &mut egui::Ui, state: &mut ShellState, mixed: bool) {
    ui.add_space(theme::SPACING);
    let (mut appearance, mut advanced) = state.inspector_sections;
    section_toggle(ui, &mut appearance, "Appearance (this view only)");
    if appearance {
        let value = if mixed { "Mixed" } else { "Default" };
        components::field_label(ui, &format!("Fill and border: {value}"));
        let _ = components::action_button(
            ui,
            "Change appearance",
            Some("Not available yet: view styles arrive with the project model."),
            None,
        );
    }
    section_toggle(ui, &mut advanced, "Advanced");
    if advanced {
        components::field_label(
            ui,
            "Stereotypes, visibility and documentation arrive with the project model.",
        );
    }
    state.inspector_sections = (appearance, advanced);
}

/// Contextual inspector: nothing selected, one class, several classes, or a
/// relationship. Edits are drafts committed through intents.
pub(crate) fn inspector(ui: &mut egui::Ui, state: &mut ShellState) {
    components::panel_title(ui, "Selection");
    if let Some(relation) = state.selected_relation() {
        relation_inspector(ui, state, relation);
        return;
    }
    let selected = state.selected_elements();
    match selected.as_slice() {
        [] => {
            ui.label("Nothing selected. Select a class on the canvas or in the Model list.");
            if let Some(diagram) = state
                .active_diagram()
                .and_then(|id| state.document().diagrams.get(&id))
            {
                ui.add_space(theme::SPACING);
                components::field_label(ui, "Diagram");
                ui.label(diagram.name.as_str());
                components::field_label(ui, &format!("{} classes shown", diagram.nodes.len()));
            }
            components::field_label(ui, "Tip: Ctrl+click adds classes to the selection.");
        }
        [element] => class_inspector(ui, state, *element),
        several => {
            let names: Vec<&str> = several
                .iter()
                .filter_map(|&id| read_model::element_name(state.document(), id))
                .collect();
            components::field_label(ui, &format!("{} classes selected", several.len()));
            ui.label(format!("Names (mixed): {}", names.join(", ")));
            ui.label(
                "Select one class to rename it. Edits never apply to several classes silently.",
            );
            appearance_and_advanced(ui, state, true);
        }
    }
}

fn class_inspector(ui: &mut egui::Ui, state: &mut ShellState, element: ElementId) {
    let Some(name) = read_model::element_name(state.document(), element).map(str::to_owned) else {
        return;
    };
    components::field_label(ui, "Model: changes apply in every diagram");
    let outcome = draft_field(
        ui,
        &mut state.drafts,
        DraftKey::ClassName(element),
        "Name",
        &name,
        rules::class_name_error,
    );
    if let DraftOutcome::Commit(new_name) = outcome {
        state.queue(UiIntent::RenameClass(element, new_name));
    }
    // Model versus view: renaming changes the class in every diagram.
    components::field_label(ui, &read_model::shown_in(state.document(), element));

    // Sections come from the active diagram's notation contract.
    let sections = notation::notation(state.active_notation()).element_sections;
    if sections.contains(&InspectorSection::Members) {
        members(ui, state, element);
    }
    if sections.contains(&InspectorSection::Appearance)
        || sections.contains(&InspectorSection::Advanced)
    {
        appearance_and_advanced(ui, state, false);
    }
}

/// Attributes and operations of a class, with the unavailable Add buttons.
fn members(ui: &mut egui::Ui, state: &ShellState, element: ElementId) {
    ui.add_space(theme::SPACING);
    components::field_label(ui, "Attributes");
    let attributes = read_model::attribute_lines(state.document(), element);
    if attributes.is_empty() {
        ui.label("No attributes.");
    }
    for attribute in &attributes {
        components::code_text(ui, attribute);
    }
    components::field_label(ui, "Operations");
    let operations = read_model::operation_lines(state.document(), element);
    if operations.is_empty() {
        ui.label("No operations.");
    }
    for operation in &operations {
        components::code_text(ui, operation);
    }
    ui.horizontal_wrapped(|ui| {
        for label in ["Add attribute", "Add operation"] {
            let _ = components::action_button(
                ui,
                label,
                Some("Not available yet: editing members arrives with the member editor."),
                None,
            );
        }
    });
}

fn relation_inspector(ui: &mut egui::Ui, state: &mut ShellState, id: RelationId) {
    let Some(relation) = state.document().relations.get(&id).cloned() else {
        return;
    };
    let name_of = |element| {
        read_model::element_name(state.document(), element)
            .unwrap_or("?")
            .to_owned()
    };
    let (source, target) = (name_of(relation.source), name_of(relation.target));
    components::field_label(ui, "Relationship (model)");
    ui.label(format!("From {source} to {target}"));
    ui.label(format!("Kind: {}", read_model::kind_word(relation.kind)));
    if relation.kind == RelationKind::Association {
        // The core refuses a class related to itself, so the two ends always
        // have different names and every accessible name below is distinct.
        let fields = [
            (RelationField::SourceRole, format!("Role at {source}")),
            (
                RelationField::SourceMultiplicity,
                format!("Multiplicity at {source}"),
            ),
            (RelationField::TargetRole, format!("Role at {target}")),
            (
                RelationField::TargetMultiplicity,
                format!("Multiplicity at {target}"),
            ),
        ];
        for (field, label) in fields {
            let committed = field.value(&relation);
            let outcome = draft_field(
                ui,
                &mut state.drafts,
                DraftKey::Relation(id, field),
                &label,
                &committed,
                |value| field.error(value),
            );
            if let DraftOutcome::Commit(value) = outcome {
                state.queue(UiIntent::SetRelationField(id, field, value));
            }
        }
    }
    if relation.kind != RelationKind::Association {
        components::field_label(ui, "Roles and multiplicities apply to associations only.");
    }
    appearance_and_advanced(ui, state, false);
}
