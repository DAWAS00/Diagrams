//! Inspector (spec §6, brief §A7): one layout per selection kind.
//!
//! The Name field edits B's draft: typing pushes `DraftChanged`, Enter
//! pushes `CommitDraft`, Escape pushes `CancelDraft`. The field has a fixed
//! id, so it keeps keyboard focus when its problem text appears beside it
//! (UI rule 1.4). Every section carries a persistent scope label.

use eframe::egui::{self, Key, TextEdit, Vec2};

use super::components;
use super::theme::{self, CONTROL_HEIGHT};
use crate::intent::{DraftField, UiIntent};
use crate::presentation::{FieldView, InspectorKind, InspectorView};

/// Scope label of sections that edit the shared model.
pub const SHARED_MODEL_SCOPE: &str = "Shared model — affects all views";
/// Scope label of sections that edit this diagram's view only.
pub const THIS_VIEW_SCOPE: &str = "This diagram view only";

/// Draws the inspector contents.
pub fn show(ui: &mut egui::Ui, view: &InspectorView, out: &mut Vec<UiIntent>) {
    components::panel_title(ui, view.title);
    match &view.kind {
        InspectorKind::Nothing { guidance } => {
            ui.label(*guidance);
        }
        InspectorKind::Class { name, attributes } => {
            components::field_label(ui, "Model");
            scope_label(ui, SHARED_MODEL_SCOPE);
            draft_field(ui, DraftField::ClassName, name, out);
            ui.add_space(theme::SPACING);
            components::field_label(ui, "Attributes");
            if attributes.is_empty() {
                ui.label("No attributes.");
            }
            for attribute in attributes {
                components::code_text(ui, attribute);
            }
        }
    }
}

/// A persistent label saying what a section changes.
fn scope_label(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(format!("Scope: {text}"))
            .text_style(theme::TextRole::Label.text_style())
            .weak(),
    );
}

/// A labeled field bound to a draft, with its problem beside it.
fn draft_field(ui: &mut egui::Ui, field: DraftField, view: &FieldView, out: &mut Vec<UiIntent>) {
    let label = components::field_label(ui, view.label);
    let mut text = view.text.clone();
    let danger = ui.visuals().error_fg_color;
    let saved = ui.visuals().widgets.clone();
    if view.problem.is_some() {
        // Changed in place, not in a nested scope: a scope would give the
        // field a new id and it would lose focus when the problem appears.
        let widgets = &mut ui.visuals_mut().widgets;
        for state in [&mut widgets.inactive, &mut widgets.hovered] {
            state.bg_stroke = egui::Stroke::new(1.5, danger);
        }
    }
    let response = ui
        .add(
            TextEdit::singleline(&mut text)
                .id_salt(("draft-field", field))
                .desired_width(f32::INFINITY)
                .min_size(Vec2::new(0.0, CONTROL_HEIGHT)),
        )
        .labelled_by(label.id);
    ui.visuals_mut().widgets = saved;
    if response.changed() {
        out.push(UiIntent::DraftChanged {
            field,
            text: text.clone(),
        });
    }
    // `lost_focus` can be reported on more than one run for a single focus
    // change, so remember (local widget state) whether this focus session
    // has already ended, and end it exactly once.
    let editing = response.id.with("editing");
    if response.has_focus() {
        ui.data_mut(|data| data.insert_temp(editing, true));
    }
    let was_editing = ui.data(|data| data.get_temp::<bool>(editing).unwrap_or(false));
    if response.lost_focus() && was_editing {
        ui.data_mut(|data| data.insert_temp(editing, false));
        if ui.input(|input| input.key_pressed(Key::Escape)) {
            out.push(UiIntent::CancelDraft(field));
        } else {
            out.push(UiIntent::CommitDraft(field));
        }
    }
    if let Some(problem) = &view.problem {
        components::warning_text(ui, problem);
    }
}
