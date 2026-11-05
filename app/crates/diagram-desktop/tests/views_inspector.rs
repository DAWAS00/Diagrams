//! A-6: inspector (brief §A7), for the selection kinds B presents today.

mod common_ui;

use common_ui::{audit_view_everywhere, in_side_panel, is_focused, light, shows_text};
use diagram_desktop::intent::UiIntent;
use diagram_desktop::presentation::{InspectorKind, Presentation};
use diagram_desktop::shell::BLANK_NAME_WARNING;
use diagram_desktop::ui::{fixtures, inspector};
use eframe::egui;
use egui_kittest::kittest::Queryable;

fn draw(ui: &mut egui::Ui, view: &Presentation, out: &mut Vec<UiIntent>) {
    in_side_panel(ui, 300.0, |ui| inspector::show(ui, &view.inspector, out));
}

#[test]
fn nothing_selected_should_show_guidance_and_no_fields() {
    let view = fixtures::sample();
    let harness = light(view.clone(), draw);

    let guidance = match view.inspector.kind {
        InspectorKind::Nothing { guidance } => guidance,
        InspectorKind::Class { .. } => "",
    };
    assert!(!guidance.is_empty(), "the sample has no selection");
    assert!(shows_text(&harness, guidance));
    assert!(harness.query_by_label("Name").is_none());
}

#[test]
fn class_should_show_its_scope_name_field_and_attributes() {
    let harness = light(fixtures::class_selected(), draw);

    assert!(shows_text(&harness, inspector::SHARED_MODEL_SCOPE));
    harness.get_by_label("Name");
    assert!(shows_text(&harness, "id: CustomerId"));
}

#[test]
fn blank_name_should_show_the_problem_in_words() {
    let harness = light(fixtures::blank_name(), draw);

    assert!(shows_text(
        &harness,
        &format!("Warning: {BLANK_NAME_WARNING}")
    ));
}

#[test]
fn name_field_should_keep_focus_when_its_problem_appears() {
    let mut harness = light(fixtures::class_selected(), draw);
    harness.get_by_label("Name").focus();
    harness.run();
    assert!(is_focused(&harness, "Name"));

    // B answers the edit with a presentation that carries the problem.
    harness.state_mut().view = fixtures::blank_name();
    harness.run();

    assert!(shows_text(&harness, BLANK_NAME_WARNING));
    assert!(is_focused(&harness, "Name"), "the field lost focus");
}

#[test]
fn inspector_should_pass_the_audit_at_every_size_and_theme() {
    for (name, view) in [
        ("inspector, nothing", fixtures::sample()),
        ("inspector, class", fixtures::class_selected()),
        ("inspector, blank name", fixtures::blank_name()),
        ("inspector, long name", fixtures::long_name()),
    ] {
        audit_view_everywhere(name, &view, draw, &[]);
    }
}
