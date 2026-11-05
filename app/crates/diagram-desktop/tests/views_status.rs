//! A-7: status bar (brief §A8), for the fields B presents today.

mod common_ui;

use common_ui::{audit_view_everywhere, light, pressed, shows_text};
use diagram_desktop::intent::UiIntent;
use diagram_desktop::presentation::Presentation;
use diagram_desktop::ui::{fixtures, status};
use eframe::egui;

fn draw(ui: &mut egui::Ui, view: &Presentation, out: &mut Vec<UiIntent>) {
    egui::Panel::bottom("status-under-test")
        .show(ui, |ui| status::show(ui, &view.status, true, out));
}

#[test]
fn status_should_show_save_state_selection_and_the_theme_in_words() {
    let view = fixtures::class_selected();
    let harness = light(view.clone(), draw);

    assert!(shows_text(&harness, view.status.save_text));
    assert!(shows_text(&harness, &view.status.selection_text));
    for (preference, label) in status::THEME_CHOICES {
        assert_eq!(
            pressed(&harness, label),
            view.status.theme == preference,
            "{label}"
        );
    }
}

#[test]
fn status_should_pass_the_audit_at_every_size_and_theme() {
    audit_view_everywhere("status", &fixtures::sample(), draw, &[]);
    audit_view_everywhere("status, long selection", &fixtures::long_name(), draw, &[]);
}

#[test]
fn zoom_grid_and_snap_should_be_shown_in_words() {
    let view = fixtures::sample();
    let harness = light(view.clone(), draw);

    assert!(shows_text(&harness, &view.status.zoom_text));
    let grid = if view.status.grid {
        "Grid on"
    } else {
        "Grid off"
    };
    let snap = if view.status.snap {
        "Snap on"
    } else {
        "Snap off"
    };
    assert!(shows_text(&harness, grid));
    assert!(shows_text(&harness, snap));
}
