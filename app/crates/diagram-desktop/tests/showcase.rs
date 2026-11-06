//! UI-01: the component showcase shows every reusable control and state.

mod common;
use common::{harness, select_class, selected_name};
use diagram_desktop::shell::ShellState;
use eframe::egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};

fn open_from_search(harness: &mut Harness<'_, ShellState>) {
    harness.key_press_modifiers(Modifiers::COMMAND, Key::K);
    harness.run();
    harness
        .get_by_label("Find a command or object")
        .type_text("showcase");
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
}

#[test]
fn search_should_open_the_showcase() {
    let mut harness = harness();

    open_from_search(&mut harness);

    assert!(harness.state().showcase.is_some());
    harness.get_by_label("Close showcase");
}

#[test]
fn showcase_should_label_itself_as_a_fixture_and_show_every_component() {
    let mut harness = harness();
    open_from_search(&mut harness);

    harness.get_by_label_contains("Design fixture");
    for label in [
        "Page title",
        "Enabled action",
        "Disabled action",
        "First option",
        "Second option",
        "Selected row",
        "Another row",
        "Text field",
        "Field with an error",
        "Disabled field",
    ] {
        harness.get_by_label(label);
    }
    harness.get_by_label_contains("Warning: ");
    harness.get_by_label_contains("Error: A name must contain");
    harness.get_by_label_contains("Error: This failed");
}

#[test]
fn showcase_states_should_be_exposed_to_assistive_technology() {
    let mut harness = harness();
    open_from_search(&mut harness);

    assert!(
        harness
            .get_by_label("Disabled action")
            .accesskit_node()
            .is_disabled()
    );
    assert!(
        harness
            .get_by_label("Disabled field")
            .accesskit_node()
            .is_disabled()
    );
    assert!(
        !harness
            .get_by_label("Enabled action")
            .accesskit_node()
            .is_disabled()
    );
    assert!(
        !harness
            .get_by_label("Field with an error")
            .accesskit_node()
            .is_disabled()
    );
}

#[test]
fn showcase_choices_and_rows_should_respond() {
    let mut harness = harness();
    open_from_search(&mut harness);

    harness.get_by_label("Second option").click();
    harness.get_by_label("Another row").click();
    harness.run();

    let fixture = harness.state().showcase.clone();
    assert_eq!(fixture.as_ref().map(|state| state.choice), Some(1));
    assert_eq!(fixture.as_ref().map(|state| state.row), Some(1));
}

#[test]
fn close_button_and_escape_should_close_the_showcase_only() {
    let mut harness = harness();
    select_class(&mut harness, "Customer");
    harness.run();
    open_from_search(&mut harness);

    harness.get_by_label("Close showcase").click();
    harness.run();
    assert!(harness.state().showcase.is_none());

    open_from_search(&mut harness);
    harness.key_press(Key::Escape);
    harness.run();
    assert!(harness.state().showcase.is_none());
    assert_eq!(
        selected_name(&harness).as_deref(),
        Some("Customer"),
        "Escape closed the showcase only"
    );
}
