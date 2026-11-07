//! Headless tests of the R0-02 shell, driven through the AccessKit tree the way
//! a screen reader or keyboard user would reach each control.

mod common;
use common::{class_id, harness_unrun, name_of, select_class, selected_name};
use diagram_desktop::shell::{self, ShellState};
use eframe::egui::accesskit::Toggled;
use eframe::egui::{Key, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};

/// egui reports a selected widget to assistive technology as toggled/pressed.
fn is_pressed(harness: &Harness<'_, ShellState>, label: &str) -> bool {
    harness.get_by_label(label).accesskit_node().toggled() == Some(Toggled::True)
}

#[test]
fn shell_should_show_toolbar_sidebar_canvas_inspector_and_status() {
    let harness = harness_unrun();

    for label in [
        "Undo",
        "Redo",
        "Save",
        "Export",
        "Pan",
        "Class",
        "Association",
        "Note",
        "Arrange",
        "New diagram",
    ] {
        harness.get_by_label(label);
    }
    harness.get_by_label("Project");
    // The open diagram is a tab, a project-panel row, and the inspector's
    // "Diagram" detail while nothing is selected.
    assert_eq!(harness.get_all_by_label("Class diagram 1").count(), 3);
    harness.get_by_label("Diagram canvas");
    harness.get_by_label("Selection");
    harness.get_by_label("Prototype · nothing is saved");
    harness.get_by_label("No selection");
}

#[test]
fn unimplemented_toolbar_actions_should_be_disabled() {
    let harness = harness_unrun();

    for label in [
        "Undo",
        "Redo",
        "Save",
        "Export",
        "Pan",
        "Association",
        "Note",
        "Arrange",
        "New diagram",
    ] {
        assert!(
            harness.get_by_label(label).accesskit_node().is_disabled(),
            "{label} is enabled"
        );
    }
}

#[test]
fn inspector_should_explain_how_to_select_when_nothing_is_selected() {
    let harness = harness_unrun();

    harness.get_by_label_contains("Nothing selected");
    assert!(harness.query_by_label("Name").is_none());
}

#[test]
fn clicking_a_canvas_class_should_select_it() {
    let mut harness = harness_unrun();

    harness.get_by_label("Class Order").click();
    harness.run();

    assert_eq!(selected_name(&harness).as_deref(), Some("Order"));
    assert!(is_pressed(&harness, "Class Order"));
    assert!(!is_pressed(&harness, "Class Customer"));
    harness.get_by_label("Selected: Order");
}

#[test]
fn model_list_should_select_a_class_without_the_canvas() {
    let mut harness = harness_unrun();

    harness.get_by_label("Customer").click();
    harness.run();

    assert_eq!(selected_name(&harness).as_deref(), Some("Customer"));
    assert!(is_pressed(&harness, "Class Customer"));
}

#[test]
fn clicking_empty_canvas_should_clear_the_selection() {
    let mut harness = harness_unrun();
    select_class(&mut harness, "Customer");
    harness.run();

    harness.get_by_label("Diagram canvas").click();
    harness.run();

    assert_eq!(selected_name(&harness), None);
}

#[test]
fn escape_should_clear_the_selection_when_not_typing() {
    let mut harness = harness_unrun();
    select_class(&mut harness, "Order");
    harness.run();

    harness.key_press(Key::Escape);
    harness.run();

    assert_eq!(selected_name(&harness), None);
}

#[test]
fn typing_in_the_name_field_should_rename_the_class_everywhere() {
    // The headless test fonts have no Arabic glyphs; shaping and font fallback
    // are validated natively in R0-04. This test checks the data flow only.
    let mut harness = Harness::builder()
        .with_size(Vec2::new(1280.0, 800.0))
        .allow_missing_glyphs()
        .build_ui_state(shell::show, ShellState::default());
    select_class(&mut harness, "Customer");
    let customer = class_id(harness.state(), "Customer");
    harness.run();

    let name = harness.get_by_label("Name");
    name.focus();
    harness.key_press_modifiers(eframe::egui::Modifiers::COMMAND, Key::A);
    name.type_text("Client عميل");
    harness.run();
    // A name is a draft until Enter commits it.
    assert_eq!(name_of(&harness, customer), "Customer");
    harness.key_press(Key::Enter);
    harness.run();

    assert_eq!(name_of(&harness, customer), "Client عميل");
    harness.get_by_label("Class Client عميل");
    harness.get_by_label("Selected: Client عميل");
}

#[test]
fn escape_while_typing_should_keep_the_selection() {
    let mut harness = harness_unrun();
    select_class(&mut harness, "Customer");
    harness.run();

    harness.get_by_label("Name").focus();
    harness.run();
    assert!(harness.get_by_label("Name").is_focused());
    harness.key_press(Key::Escape);
    harness.run();

    assert_eq!(selected_name(&harness).as_deref(), Some("Customer"));
    assert!(!harness.get_by_label("Name").is_focused());

    harness.key_press(Key::Escape);
    harness.run();

    assert_eq!(selected_name(&harness), None);
}

#[test]
fn select_should_ignore_stale_ids_and_keep_the_selection() {
    let mut state = ShellState::default();
    let customer = class_id(&state, "Customer");
    state.select(customer);

    let selected = state.select(diagram_core::ids::ElementId::from_raw(99));

    assert!(!selected, "a class that does not exist cannot be selected");
    assert_eq!(state.selected_element(), Some(customer));
}
