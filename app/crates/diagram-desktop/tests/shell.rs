//! Headless tests of the R0-02 shell, driven through the AccessKit tree the way
//! a screen reader or keyboard user would reach each control.

use diagram_desktop::shell::{self, BLANK_NAME_WARNING, ShellState};
use diagram_desktop::ui::commands::{self, CommandContext, CommandSpec, Placement};
use eframe::egui::accesskit::Toggled;
use eframe::egui::{Key, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};

/// Commands with a button in the header, tab row or tool row, from the catalog.
fn placed_commands() -> impl Iterator<Item = &'static CommandSpec> {
    [Placement::Header, Placement::TabRow, Placement::ToolRow]
        .into_iter()
        .flat_map(commands::placed)
}

fn harness() -> Harness<'static, ShellState> {
    Harness::builder()
        .with_size(Vec2::new(1280.0, 800.0))
        .build_ui_state(shell::show, ShellState::default())
}

/// egui reports a selected widget to assistive technology as toggled/pressed.
fn is_pressed(harness: &Harness<'_, ShellState>, label: &str) -> bool {
    harness.get_by_label(label).accesskit_node().toggled() == Some(Toggled::True)
}

#[test]
fn shell_should_show_toolbar_sidebar_canvas_inspector_and_status() {
    let harness = harness();

    for spec in placed_commands() {
        harness.get_by_label(spec.label);
    }
    harness.get_by_label("Project");
    // The open diagram is both a tab and a row in the project panel.
    assert_eq!(harness.get_all_by_label("Class diagram 1").count(), 2);
    harness.get_by_label("Diagram canvas");
    harness.get_by_label("Selection");
    harness.get_by_label("Prototype · nothing is saved");
    harness.get_by_label("No selection");
}

#[test]
fn unimplemented_toolbar_actions_should_be_disabled() {
    let harness = harness();

    // The catalog decides which commands are unavailable; every one of them
    // must be drawn disabled, and every available one enabled.
    for spec in placed_commands() {
        let disabled = !commands::availability(spec.id, CommandContext::default()).is_enabled();
        assert_eq!(
            harness
                .get_by_label(spec.label)
                .accesskit_node()
                .is_disabled(),
            disabled,
            "{}",
            spec.label
        );
    }
}

#[test]
fn inspector_should_explain_how_to_select_when_nothing_is_selected() {
    let harness = harness();

    harness.get_by_label_contains("Nothing selected");
    assert!(harness.query_by_label("Name").is_none());
}

#[test]
fn clicking_a_canvas_class_should_select_it() {
    let mut harness = harness();

    harness.get_by_label("Class Order").click();
    harness.run();

    assert_eq!(harness.state().selected, Some(1));
    assert!(is_pressed(&harness, "Class Order"));
    assert!(!is_pressed(&harness, "Class Customer"));
    harness.get_by_label("Selected: Order");
}

#[test]
fn model_list_should_select_a_class_without_the_canvas() {
    let mut harness = harness();

    harness.get_by_label("Customer").click();
    harness.run();

    assert_eq!(harness.state().selected, Some(0));
    assert!(is_pressed(&harness, "Class Customer"));
}

#[test]
fn clicking_empty_canvas_should_clear_the_selection() {
    let mut harness = harness();
    harness.state_mut().select(0);
    harness.run();

    harness.get_by_label("Diagram canvas").click();
    harness.run();

    assert_eq!(harness.state().selected, None);
}

#[test]
fn escape_should_clear_the_selection_when_not_typing() {
    let mut harness = harness();
    harness.state_mut().select(1);
    harness.run();

    harness.key_press(Key::Escape);
    harness.run();

    assert_eq!(harness.state().selected, None);
}

#[test]
fn typing_in_the_name_field_should_rename_the_class_everywhere() {
    // The headless test fonts have no Arabic glyphs; shaping and font fallback
    // are validated natively in R0-04. This test checks the data flow only.
    let mut harness = Harness::builder()
        .with_size(Vec2::new(1280.0, 800.0))
        .allow_missing_glyphs()
        .build_ui_state(shell::show, ShellState::default());
    harness.state_mut().select(0);
    harness.run();

    let name = harness.get_by_label("Name");
    name.focus();
    harness.key_press_modifiers(eframe::egui::Modifiers::COMMAND, Key::A);
    name.type_text("Client عميل");
    harness.run();

    assert_eq!(harness.state().classes[0].name, "Client عميل");
    harness.get_by_label("Class Client عميل");
    harness.get_by_label("Selected: Client عميل");
}

#[test]
fn escape_while_typing_should_keep_the_selection() {
    let mut harness = harness();
    harness.state_mut().select(0);
    harness.run();

    harness.get_by_label("Name").focus();
    harness.run();
    assert!(harness.get_by_label("Name").is_focused());
    harness.key_press(Key::Escape);
    harness.run();

    assert_eq!(harness.state().selected, Some(0));
    assert!(!harness.get_by_label("Name").is_focused());

    harness.key_press(Key::Escape);
    harness.run();

    assert_eq!(harness.state().selected, None);
}

#[test]
fn blank_name_should_show_a_text_warning_and_placeholder() {
    let mut harness = harness();
    harness.state_mut().select(1);
    harness.state_mut().classes[1].name = String::from("   ");
    harness.run();

    harness.get_by_label_contains(BLANK_NAME_WARNING);
    harness.get_by_label("Class Unnamed class");
    harness.get_by_label("Selected: Unnamed class");
}

#[test]
fn select_should_ignore_out_of_range_indices() {
    let mut state = ShellState::default();
    state.select(0);

    state.select(99);

    assert_eq!(state.selected, None);
    assert_eq!(state.selected_class(), None);
}
