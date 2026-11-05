//! A-1: header view (brief §A2).

mod common_ui;

use common_ui::{SMALL, audit_view_everywhere, bounds, harness, in_top_row, light};
use diagram_desktop::intent::UiIntent;
use diagram_desktop::presentation::Presentation;
use diagram_desktop::ui::{fixtures, header, workspace};
use eframe::egui::{self, ThemePreference};
use egui_kittest::kittest::Queryable;

fn draw(ui: &mut egui::Ui, view: &Presentation, out: &mut Vec<UiIntent>) {
    in_top_row(ui, workspace::HEADER_HEIGHT, |ui| {
        header::show(ui, &view.header, out);
    });
}

fn long_project() -> Presentation {
    let mut state = fixtures::sample_state();
    state.project_name = fixtures::LONG_NAME.to_owned();
    diagram_desktop::presentation::build(&state)
}

#[test]
fn header_should_show_menu_title_and_every_header_command() {
    let view = fixtures::sample();
    let harness = light(view.clone(), draw);

    harness.get_by_label(header::PROJECT_MENU);
    harness.get_by_label(&view.header.title);
    for command in &view.header.commands {
        harness.get_by_label(&command.label);
    }
}

#[test]
fn header_should_pass_the_audit_at_every_size_and_theme() {
    audit_view_everywhere("header", &fixtures::sample(), draw, &[]);
    audit_view_everywhere("header, long project name", &long_project(), draw, &[]);
}

#[test]
fn long_project_name_should_shorten_and_keep_its_full_accessible_name() {
    let view = long_project();
    let harness = harness(SMALL, ThemePreference::Light, view.clone(), draw);

    let title = bounds(&harness, &view.header.title);
    let first_command = bounds(&harness, &view.header.commands[0].label);
    assert!(
        title.x1 <= first_command.x0,
        "title {title:?} runs into {first_command:?}"
    );
}

#[test]
fn project_menu_should_list_every_command_group() {
    let view = fixtures::sample();
    let mut harness = light(view.clone(), draw);

    harness.get_by_label(header::PROJECT_MENU).click();
    harness.run();
    for group in &view.header.menu {
        harness.get_by_label(group.label);
    }
}
