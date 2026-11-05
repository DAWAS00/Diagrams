//! UI-08: contextual inspector — drafts with explicit commit and cancel,
//! Model versus Appearance, members, relationship endpoints, mixed values.

use diagram_desktop::shell::{self, BLANK_NAME_WARNING, ShellState};
use diagram_desktop::ui::inspector::{self, RelationField};
use eframe::egui::{Key, Modifiers, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};

const RELATION_ROW: &str = "Customer places Order";

fn harness_with(state: ShellState) -> Harness<'static, ShellState> {
    let mut harness = Harness::builder()
        .with_size(Vec2::new(1280.0, 800.0))
        .build_ui_state(shell::show, state);
    harness.run();
    harness
}

fn selected(index: usize) -> Harness<'static, ShellState> {
    let mut state = ShellState::default();
    state.select(index);
    harness_with(state)
}

fn edit(harness: &mut Harness<'_, ShellState>, label: &str, text: &str) {
    harness.get_by_label(label).focus();
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness.get_by_label(label).type_text(text);
    harness.run();
}

#[test]
fn multiplicity_rule_should_accept_uml_forms_only() {
    for valid in ["1", "*", "0..1", "1..*", "2..5", " 0..* "] {
        assert_eq!(inspector::multiplicity_error(valid), None, "{valid}");
    }
    for invalid in ["", "a", "5..2", "0..0", "1..", "..3", "-1", "1..x"] {
        assert!(
            inspector::multiplicity_error(invalid).is_some(),
            "{invalid}"
        );
    }
}

#[test]
fn relation_fields_should_read_and_validate_their_own_values() {
    let relation = inspector::sample_relation();

    assert_eq!(RelationField::Name.value(&relation), "places");
    assert_eq!(RelationField::TargetMultiplicity.value(&relation), "0..*");
    assert!(RelationField::SourceMultiplicity.error("x").is_some());
    assert!(RelationField::SourceRole.error("anything").is_none());
    assert_eq!(
        inspector::class_name_error("  ").as_deref(),
        Some(BLANK_NAME_WARNING)
    );
}

#[test]
fn no_selection_should_describe_the_diagram() {
    let harness = harness_with(ShellState::default());

    harness.get_by_label("Diagram");
    harness.get_by_label("2 classes shown");
    harness.get_by_label_contains("Ctrl+click adds classes");
}

#[test]
fn name_should_stay_a_draft_until_enter() {
    let mut harness = selected(0);

    edit(&mut harness, "Name", "Client");

    assert_eq!(
        harness.state().classes[0].name,
        "Customer",
        "no direct write"
    );
    harness.get_by_label("Class Customer");
    harness.get_by_label_contains("Enter applies the change");
    harness.key_press(Key::Enter);
    harness.run();

    assert_eq!(harness.state().classes[0].name, "Client");
    harness.get_by_label("Class Client");
    assert!(harness.state().drafts.is_empty());
}

#[test]
fn escape_should_cancel_the_draft_and_restore_the_name() {
    let mut harness = selected(0);
    edit(&mut harness, "Name", "Client");

    harness.key_press(Key::Escape);
    harness.run();

    assert_eq!(harness.state().classes[0].name, "Customer");
    assert!(harness.state().drafts.is_empty());
    assert_eq!(
        harness.get_by_label("Name").value().as_deref(),
        Some("Customer")
    );
    assert_eq!(
        harness.state().selected,
        Some(0),
        "first Escape only cancels the edit"
    );
}

#[test]
fn moving_focus_away_should_commit_a_valid_draft() {
    let mut harness = selected(0);
    edit(&mut harness, "Name", "Client");

    harness.key_press(Key::Tab);
    harness.run();

    assert_eq!(harness.state().classes[0].name, "Client");
}

#[test]
fn invalid_draft_should_stay_with_its_error_and_never_commit() {
    let mut harness = selected(0);
    edit(&mut harness, "Name", "   ");

    harness.get_by_label(&format!("Error: {BLANK_NAME_WARNING}"));
    harness.key_press(Key::Enter);
    harness.run();

    assert_eq!(harness.state().classes[0].name, "Customer");
    assert!(
        !harness.state().drafts.is_empty(),
        "the invalid draft stays editable"
    );
    harness.get_by_label(&format!("Error: {BLANK_NAME_WARNING}"));
}

#[test]
fn class_inspector_should_show_members_and_explain_unavailable_edits() {
    let harness = selected(0);

    harness.get_by_label("Model: changes apply in every diagram");
    harness.get_by_label("id: CustomerId");
    harness.get_by_label("No operations.");
    assert!(
        harness
            .get_by_label("Add attribute")
            .accesskit_node()
            .is_disabled()
    );
    assert!(
        harness
            .get_by_label("Add operation")
            .accesskit_node()
            .is_disabled()
    );
}

#[test]
fn appearance_and_advanced_should_start_collapsed_and_open_on_request() {
    let mut harness = selected(0);
    assert!(harness.query_by_label("Change appearance").is_none());

    harness.get_by_label("Appearance (this view only)").click();
    harness.run();
    harness.get_by_label("Advanced").click();
    harness.run();

    assert!(
        harness
            .get_by_label("Change appearance")
            .accesskit_node()
            .is_disabled()
    );
    harness.get_by_label_contains("Stereotypes, visibility and documentation");
}

#[test]
fn relationship_should_show_both_endpoints_from_the_project_panel() {
    let mut harness = harness_with(ShellState::default());

    harness.get_by_label(RELATION_ROW).click();
    harness.run();

    assert_eq!(harness.state().selected_relation, Some(0));
    for label in [
        "Relationship name",
        "Role at Customer",
        "Multiplicity at Customer",
        "Role at Order",
        "Multiplicity at Order",
    ] {
        harness.get_by_label(label);
    }
    harness.get_by_label("From Customer to Order");
    harness.get_by_label(&format!("Selected: {RELATION_ROW}"));
}

#[test]
fn multiplicity_should_reject_invalid_values_and_commit_valid_ones() {
    let mut harness = harness_with(ShellState::default());
    harness.get_by_label(RELATION_ROW).click();
    harness.run();

    edit(&mut harness, "Multiplicity at Order", "many");
    harness.key_press(Key::Enter);
    harness.run();
    harness.get_by_label_contains("Error: A multiplicity is a number");
    assert_eq!(harness.state().relations[0].target_multiplicity, "0..*");

    edit(&mut harness, "Multiplicity at Order", "1..3");
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(harness.state().relations[0].target_multiplicity, "1..3");
}

#[test]
fn ctrl_click_should_build_a_mixed_selection_without_silent_edits() {
    let mut harness = selected(0);

    harness
        .get_by_label("Order")
        .click_modifiers(Modifiers::COMMAND);
    harness.run();

    assert_eq!(harness.state().selected_classes(), [0, 1]);
    // Shown in the inspector and in the status bar.
    assert_eq!(harness.get_all_by_label("2 classes selected").count(), 2);
    harness.get_by_label("Names (mixed): Customer, Order");
    assert!(
        harness.query_by_label("Name").is_none(),
        "no field edits several classes"
    );
    harness.get_by_label_contains("never apply to several classes silently");

    harness
        .get_by_label("Order")
        .click_modifiers(Modifiers::COMMAND);
    harness.run();
    assert_eq!(harness.state().selected_classes(), [0]);
}

#[test]
fn ctrl_click_on_the_canvas_should_add_to_the_selection() {
    let mut harness = selected(1);

    harness
        .get_by_label("Class Customer")
        .click_modifiers(Modifiers::COMMAND);
    harness.run();

    assert_eq!(harness.state().selected_classes(), [1, 0]);
}

#[test]
fn plain_click_should_replace_a_mixed_selection() {
    let mut harness = selected(0);
    harness
        .get_by_label("Order")
        .click_modifiers(Modifiers::COMMAND);
    harness.run();

    harness.get_by_label("Class Customer").click();
    harness.run();

    assert_eq!(harness.state().selected_classes(), [0]);
}
