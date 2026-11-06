//! The inspector: nothing selected, one class, several classes or a relationship.
//!
//! Moved out of `shell.rs` unchanged. Fields edit drafts; committed drafts are
//! queued as intents.

use eframe::egui::{self, Key};

use crate::shell::{BLANK_NAME_WARNING, PreviewClass, ShellState};
use crate::ui::components;
use crate::ui::inspector::{self as rules, DraftKey, DraftOutcome, RelationField};
use crate::ui::intents::UiIntent;
use crate::ui::navigator;
use crate::ui::notation::{self, InspectorSection};
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
    if let Some(relation) = state.selected_relation {
        relation_inspector(ui, state, relation);
        return;
    }
    let selected = state.selected_classes();
    match selected.as_slice() {
        [] => {
            ui.label("Nothing selected. Select a class on the canvas or in the Model list.");
            if let Some(diagram) = state.active_diagram().and_then(|d| state.diagrams.get(d)) {
                ui.add_space(theme::SPACING);
                components::field_label(ui, "Diagram");
                ui.label(&diagram.name);
                components::field_label(ui, &format!("{} classes shown", diagram.classes.len()));
            }
            components::field_label(ui, "Tip: Ctrl+click adds classes to the selection.");
        }
        [index] => class_inspector(ui, state, *index),
        several => {
            let names: Vec<&str> = several
                .iter()
                .filter_map(|&index| state.classes.get(index))
                .map(PreviewClass::display_name)
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

fn class_inspector(ui: &mut egui::Ui, state: &mut ShellState, index: usize) {
    let Some(class) = state.classes.get(index).cloned() else {
        return;
    };
    components::field_label(ui, "Model: changes apply in every diagram");
    let outcome = draft_field(
        ui,
        &mut state.drafts,
        DraftKey::ClassName(index),
        "Name",
        &class.name,
        rules::class_name_error,
    );
    if let DraftOutcome::Commit(name) = outcome {
        state.queue(UiIntent::RenameClass(index, name));
    }
    if class.name.trim().is_empty() {
        components::warning_text(ui, BLANK_NAME_WARNING);
    }
    // Model versus view: renaming changes the class in every diagram.
    components::field_label(ui, &navigator::shown_in(&state.diagrams, index));

    // Sections come from the active diagram's notation contract.
    let sections = notation::notation(state.active_notation()).element_sections;
    if sections.contains(&InspectorSection::Members) {
        members(ui, &class);
    }
    if sections.contains(&InspectorSection::Appearance)
        || sections.contains(&InspectorSection::Advanced)
    {
        appearance_and_advanced(ui, state, false);
    }
}

/// Attributes and operations of a class, with the unavailable Add buttons.
fn members(ui: &mut egui::Ui, class: &PreviewClass) {
    ui.add_space(theme::SPACING);
    components::field_label(ui, "Attributes");
    if class.attributes.is_empty() {
        ui.label("No attributes.");
    }
    for attribute in &class.attributes {
        components::code_text(ui, attribute);
    }
    components::field_label(ui, "Operations");
    ui.label("No operations.");
    ui.horizontal_wrapped(|ui| {
        for label in ["Add attribute", "Add operation"] {
            let _ = components::action_button(
                ui,
                label,
                Some("Not available yet: members need the project model."),
                None,
            );
        }
    });
}

fn relation_inspector(ui: &mut egui::Ui, state: &mut ShellState, index: usize) {
    let Some(relation) = state.relations.get(index).cloned() else {
        return;
    };
    let name_of = |class: usize| {
        state
            .classes
            .get(class)
            .map(|class| class.display_name().to_owned())
            .unwrap_or_default()
    };
    let (source, target) = (name_of(relation.source), name_of(relation.target));
    components::field_label(ui, "Relationship (model)");
    ui.label(format!("From {source} to {target}"));
    let fields = [
        (RelationField::Name, String::from("Relationship name")),
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
        let outcome = draft_field(
            ui,
            &mut state.drafts,
            DraftKey::Relation(index, field),
            &label,
            field.value(&relation),
            |value| field.error(value),
        );
        if let DraftOutcome::Commit(value) = outcome {
            state.queue(UiIntent::SetRelationField(index, field, value));
        }
    }
    appearance_and_advanced(ui, state, false);
}
