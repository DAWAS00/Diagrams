//! H6: whole journeys through the real `shell::show`, by accessible names.
//!
//! Until Agent A's new screens exist, each feature that is already real is
//! proved in the shell that exists now (AGENT-B §3, relaxed model gate).
//! The full journeys J1–J7 of TASKS-AND-TESTS §5 are added as their
//! screens land.

use diagram_desktop::shell::{self, ShellState};
use eframe::egui::accesskit::Rect;
use eframe::egui::{Key, Modifiers, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};

const SEARCH_FIELD: &str = "Find a command or object";

fn harness_at(width: f32, height: f32) -> Harness<'static, ShellState> {
    let mut harness = Harness::builder()
        .with_size(Vec2::new(width, height))
        .build_ui_state(shell::show, ShellState::default());
    harness.run();
    harness
}

fn bounds(harness: &Harness<'_, ShellState>, label: &str) -> Rect {
    harness
        .get_by_label(label)
        .accesskit_node()
        .bounding_box()
        .unwrap_or_default()
}

fn run_from_search(harness: &mut Harness<'_, ShellState>, query: &str) {
    harness.key_press_modifiers(Modifiers::COMMAND, Key::K);
    harness.run();
    harness.get_by_label(SEARCH_FIELD).type_text(query);
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
}

fn run_from_menu(harness: &mut Harness<'_, ShellState>, group: &str, item: &str) {
    harness.get_by_label("Menu").click();
    harness.run();
    harness.get_by_label(group).hover();
    harness.run();
    harness.get_by_label_contains(item).click();
    harness.run();
}

#[test]
fn zoom_in_from_the_menu_should_enlarge_the_class_boxes() {
    for (width, height) in [(1280.0, 800.0), (800.0, 500.0)] {
        let mut harness = harness_at(width, height);
        let before = bounds(&harness, "Class Customer");

        run_from_menu(&mut harness, "View", "Zoom in");

        let after = bounds(&harness, "Class Customer");
        assert!(
            (after.width() / before.width() - 1.25).abs() < 0.01,
            "{before:?} -> {after:?}"
        );
        // The model did not change.
        assert_eq!(harness.state().classes, ShellState::default().classes);
    }
}

#[test]
fn fit_all_from_search_should_fit_every_class_and_change_no_name() {
    let mut harness = harness_at(1280.0, 800.0);
    run_from_search(&mut harness, "fit all");

    assert!(harness.state().palette.is_none());
    let canvas = bounds(&harness, "Diagram canvas");
    for name in ["Class Customer", "Class Order"] {
        let node = bounds(&harness, name);
        assert!(
            node.x0 >= canvas.x0 && node.y0 >= canvas.y0 && node.x1 <= canvas.x1,
            "{name} {node:?} outside {canvas:?}"
        );
    }
    assert_eq!(harness.state().classes, ShellState::default().classes);
}

#[test]
fn actual_size_shortcut_should_return_to_100_percent() {
    let mut harness = harness_at(1280.0, 800.0);
    let before = bounds(&harness, "Class Order");
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Plus);
    harness.run();
    assert!(bounds(&harness, "Class Order").width() > before.width());

    harness.key_press_modifiers(Modifiers::COMMAND, Key::Num0);
    harness.run();
    assert!((bounds(&harness, "Class Order").width() - before.width()).abs() < 0.5);
}

#[test]
fn ctrl_shift_p_should_open_command_search_with_the_field_focused() {
    let mut harness = harness_at(1280.0, 800.0);
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::P);
    harness.run();

    assert!(harness.state().palette.is_some());
    assert!(harness.get_by_label(SEARCH_FIELD).is_focused());
}

#[test]
fn select_all_from_the_menu_should_select_both_classes() {
    let mut harness = harness_at(1280.0, 800.0);
    run_from_menu(&mut harness, "Edit", "Select all");

    assert_eq!(harness.state().selection, [0, 1]);
    harness.get_by_label("2 selected");
}

#[test]
fn escape_after_typing_a_valid_name_should_keep_it() {
    let mut harness = harness_at(1280.0, 800.0);
    harness.get_by_label("Customer").click();
    harness.run();
    let name = harness.get_by_label("Name");
    name.focus();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness.get_by_label("Name").type_text("Client");
    harness.run();

    // Leave the field, then clear the selection.
    harness.key_press(Key::Escape);
    harness.run();
    harness.key_press(Key::Escape);
    harness.run();

    assert_eq!(harness.state().selected, None);
    assert_eq!(harness.state().classes[0].name, "Client");
    harness.get_by_label("Class Client");
}

#[test]
fn a_blank_name_should_hold_the_selection_until_escape_restores_it() {
    let mut harness = harness_at(1280.0, 800.0);
    harness.get_by_label("Customer").click();
    harness.run();
    let name = harness.get_by_label("Name");
    name.focus();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness.key_press(Key::Backspace);
    harness.run();
    assert!(
        harness.get_by_label("Name").is_focused(),
        "focus stays in the field"
    );

    // The canvas still shows the committed name, and Order cannot be
    // selected while the edit is unresolved.
    harness.get_by_label("Class Customer").click();
    harness.run();
    harness.get_by_label("Class Order").click();
    harness.run();
    assert_eq!(harness.state().selected, Some(0));
    harness.get_by_label_contains("press Escape to discard the edit");

    // Clicking the canvas moved focus out of the field, so one Escape
    // discards the edit and keeps the selection (one layer per press).
    harness.key_press(Key::Escape);
    harness.run();
    assert_eq!(harness.state().classes[0].name, "Customer");
    assert_eq!(harness.state().selected, Some(0));
    harness.get_by_label("Class Order").click();
    harness.run();
    assert_eq!(harness.state().selected, Some(1));
}
