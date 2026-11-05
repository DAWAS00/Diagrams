//! Routing hops H1 and H2 (TASKS-AND-TESTS.md §1): every command a view
//! places renders a control with the same label and accessible name; a
//! click or Enter emits `Command(id)` exactly once; a disabled control emits
//! nothing and shows its reason. The tests loop over the catalog
//! (`CommandId::ALL`), so a new command is covered without editing a list.

mod common_ui;

use common_ui::{LARGE, click, harness, is_disabled, light, press_enter_on};
use diagram_desktop::ui::components;
use diagram_desktop::ui::fixtures::{self, CommandId, CommandView, UiIntent};
use eframe::egui::{self, ThemePreference};
use egui_kittest::kittest::Queryable;

fn one_button(ui: &mut egui::Ui, view: &CommandView, out: &mut Vec<UiIntent>) {
    if components::command_button(ui, view) {
        out.push(UiIntent::Command(view.id));
    }
}

#[test]
fn every_enabled_command_button_should_emit_its_command_once_by_click_and_enter() {
    for id in CommandId::ALL {
        let view = fixtures::enabled(id);
        let label = view.label.clone();
        let mut harness = light(view, one_button);

        assert_eq!(
            click(&mut harness, &label),
            [UiIntent::Command(id)],
            "{id:?} click"
        );
        assert_eq!(
            press_enter_on(&mut harness, &label),
            [UiIntent::Command(id)],
            "{id:?} Enter"
        );
    }
}

#[test]
fn every_disabled_command_button_should_emit_nothing_and_explain_why() {
    for id in CommandId::ALL {
        let view = fixtures::disabled(id);
        let label = view.label.clone();
        let reason = view.availability.reason().unwrap_or_default().to_owned();
        let mut harness = harness(LARGE, ThemePreference::Dark, view, one_button);

        assert!(is_disabled(&harness, &label), "{id:?}");
        assert!(click(&mut harness, &label).is_empty(), "{id:?}");
        harness.get_by_label(&label).hover();
        harness.run();
        assert!(
            harness.query_by_label_contains(&reason).is_some(),
            "{id:?}: reason not shown on hover"
        );
    }
}
