//! A-4: project panel (brief §A5), for the rows B presents today.

mod common_ui;

use common_ui::{audit_view_everywhere, bounds, in_side_panel, light, pressed, shows_text};
use diagram_desktop::intent::UiIntent;
use diagram_desktop::presentation::Presentation;
use diagram_desktop::ui::{fixtures, navigator};
use eframe::egui;
use egui_kittest::kittest::Queryable;

fn draw(ui: &mut egui::Ui, view: &Presentation, out: &mut Vec<UiIntent>) {
    in_side_panel(ui, 240.0, |ui| navigator::show(ui, &view.navigator, out));
}

#[test]
fn rows_should_show_diagrams_and_models_with_the_selection_pressed() {
    let view = fixtures::class_selected();
    let harness = light(view.clone(), draw);

    harness.get_by_label(view.navigator.title);
    for row in view.navigator.diagrams.iter().chain(&view.navigator.model) {
        assert_eq!(pressed(&harness, &row.label), row.selected, "{}", row.label);
    }
}

#[test]
fn long_names_should_stay_on_one_row_inside_the_panel_with_the_full_name() {
    let harness = light(fixtures::long_name(), draw);

    let row = bounds(&harness, fixtures::LONG_NAME);
    assert!(row.x1 <= 240.5, "{row:?}");
    assert!(row.height() < 40.0, "{row:?} wraps");
}

#[test]
fn empty_project_should_say_there_are_no_classes() {
    let harness = light(fixtures::empty_diagram(), draw);

    assert!(shows_text(&harness, navigator::NO_MODELS));
}

#[test]
fn navigator_should_pass_the_audit_at_every_size_and_theme() {
    for (name, view) in [
        ("navigator", fixtures::class_selected()),
        ("navigator, long name", fixtures::long_name()),
        ("navigator, empty", fixtures::empty_diagram()),
    ] {
        audit_view_everywhere(name, &view, draw, &[]);
    }
}
