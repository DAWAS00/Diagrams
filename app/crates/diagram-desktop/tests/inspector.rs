//! UI-08: contextual inspector — drafts with explicit commit and cancel,
//! Model versus Appearance, members, relationship endpoints, mixed values.
//! UI-13/UI-14: committed drafts are `diagram-core` commands on the real document.

use diagram_core::ids::ElementId;
use diagram_desktop::shell::{BLANK_NAME_WARNING, ShellState};
use diagram_desktop::ui::inspector::{self, RelationField};
use diagram_desktop::ui::read_model;
use eframe::egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};

mod common;
use common::{class_id, harness_with, state_with_generalization};

const RELATION_ROW: &str = "Customer – Order (association)";

fn open(state: ShellState) -> Harness<'static, ShellState> {
    harness_with(common::LARGE, state)
}

fn selected(name: &str) -> Harness<'static, ShellState> {
    let mut state = ShellState::default();
    let element = class_id(&state, name);
    state.select(element);
    open(state)
}

fn name_of(harness: &Harness<'_, ShellState>, element: ElementId) -> String {
    read_model::element_name(harness.state().document(), element)
        .unwrap_or("?")
        .to_owned()
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
    // Blank means "none" and is allowed; every other form follows the core rule.
    for valid in ["", "1", "*", "0..1", "1..*", "2..5", " 0..* "] {
        assert_eq!(inspector::multiplicity_error(valid), None, "{valid}");
    }
    for invalid in ["a", "5..2", "1..", "..3", "-1", "1..x"] {
        assert!(
            inspector::multiplicity_error(invalid).is_some(),
            "{invalid}"
        );
    }
}

#[test]
fn relation_fields_should_read_and_validate_their_own_values() {
    let state = ShellState::default();
    let relation = state.document().relations.values().next();
    assert!(relation.is_some(), "the sample has one association");
    let Some(relation) = relation else { return };

    // The core writes 0..* in its canonical form, `*`.
    assert_eq!(RelationField::TargetMultiplicity.value(relation), "*");
    assert_eq!(RelationField::SourceRole.value(relation), "customer");
    assert!(RelationField::SourceMultiplicity.error("x").is_some());
    assert!(RelationField::SourceRole.error("anything").is_none());
    assert_eq!(
        inspector::class_name_error("  ").as_deref(),
        Some(BLANK_NAME_WARNING)
    );
}

#[test]
fn no_selection_should_describe_the_diagram() {
    let harness = open(ShellState::default());

    harness.get_by_label("Diagram");
    harness.get_by_label("2 classes shown");
    harness.get_by_label_contains("Ctrl+click adds classes");
}

#[test]
fn a_diagram_with_one_class_should_say_class_in_the_singular() {
    let mut state = ShellState::default();
    let details = common::diagram_id(&state, "Order details");
    assert!(state.open_diagram(details));

    let harness = open(state);

    harness.get_by_label("1 class shown");
}

#[test]
fn name_should_stay_a_draft_until_enter() {
    let mut harness = selected("Customer");
    let customer = class_id(harness.state(), "Customer");

    edit(&mut harness, "Name", "Client");

    assert_eq!(name_of(&harness, customer), "Customer", "no direct write");
    harness.get_by_label("Class Customer");
    harness.get_by_label_contains("Enter applies the change");
    harness.key_press(Key::Enter);
    harness.run();

    assert_eq!(name_of(&harness, customer), "Client");
    harness.get_by_label("Class Client");
    assert!(harness.state().drafts.is_empty());
}

#[test]
fn escape_should_cancel_the_draft_and_restore_the_name() {
    let mut harness = selected("Customer");
    let customer = class_id(harness.state(), "Customer");
    edit(&mut harness, "Name", "Client");

    harness.key_press(Key::Escape);
    harness.run();

    assert_eq!(name_of(&harness, customer), "Customer");
    assert!(harness.state().drafts.is_empty());
    assert_eq!(
        harness.get_by_label("Name").value().as_deref(),
        Some("Customer")
    );
    assert_eq!(
        harness.state().selected_element(),
        Some(customer),
        "first Escape only cancels the edit"
    );
}

#[test]
fn moving_focus_away_should_commit_a_valid_draft() {
    let mut harness = selected("Customer");
    let customer = class_id(harness.state(), "Customer");
    edit(&mut harness, "Name", "Client");

    harness.key_press(Key::Tab);
    harness.run();

    assert_eq!(name_of(&harness, customer), "Client");
}

#[test]
fn invalid_draft_should_stay_with_its_error_and_never_commit() {
    let mut harness = selected("Customer");
    let customer = class_id(harness.state(), "Customer");
    edit(&mut harness, "Name", "   ");

    harness.get_by_label(&format!("Error: {BLANK_NAME_WARNING}"));
    harness.key_press(Key::Enter);
    harness.run();

    assert_eq!(name_of(&harness, customer), "Customer");
    assert!(
        !harness.state().drafts.is_empty(),
        "the invalid draft stays editable"
    );
    harness.get_by_label(&format!("Error: {BLANK_NAME_WARNING}"));
}

#[test]
fn rename_should_be_one_undoable_command_on_the_real_document() {
    let mut harness = selected("Customer");
    let customer = class_id(harness.state(), "Customer");
    let before = harness.state().document().clone();
    edit(&mut harness, "Name", "Client");
    harness.key_press(Key::Enter);
    harness.run();

    let project = harness.state().project();
    assert_eq!(
        project.and_then(|project| project.undo_label()),
        Some("Rename class")
    );
    assert_ne!(harness.state().document(), &before);

    harness
        .state_mut()
        .queue(diagram_desktop::ui::intents::UiIntent::run(
            diagram_desktop::ui::commands::CommandId::Undo,
        ));
    harness.run();

    assert_eq!(name_of(&harness, customer), "Customer");
    assert_eq!(
        harness.state().document(),
        &before,
        "undo restores the exact document"
    );
}

#[test]
fn class_inspector_should_show_members_and_explain_unavailable_edits() {
    let harness = selected("Customer");

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
    let mut harness = selected("Customer");
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
    let mut harness = open(ShellState::default());

    harness.get_by_label(RELATION_ROW).click();
    harness.run();

    assert!(harness.state().selected_relation().is_some());
    for label in [
        "Role at Customer",
        "Multiplicity at Customer",
        "Role at Order",
        "Multiplicity at Order",
    ] {
        harness.get_by_label(label);
    }
    assert!(
        harness.query_by_label("Relationship name").is_none(),
        "the model has no relationship name; the ends carry roles"
    );
    harness.get_by_label("From Customer to Order");
    harness.get_by_label(&format!("Selected: {RELATION_ROW}"));
}

#[test]
fn multiplicity_should_reject_invalid_values_and_commit_valid_ones() {
    let mut harness = open(ShellState::default());
    harness.get_by_label(RELATION_ROW).click();
    harness.run();
    let target_multiplicity = |harness: &Harness<'_, ShellState>| {
        harness
            .state()
            .document()
            .relations
            .values()
            .next()
            .map(|relation| RelationField::TargetMultiplicity.value(relation))
            .unwrap_or_default()
    };

    edit(&mut harness, "Multiplicity at Order", "many");
    harness.key_press(Key::Enter);
    harness.run();
    harness.get_by_label_contains("Error: A multiplicity is a number");
    assert_eq!(target_multiplicity(&harness), "*");

    edit(&mut harness, "Multiplicity at Order", "1..3");
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(target_multiplicity(&harness), "1..3");
}

#[test]
fn ctrl_click_should_build_a_mixed_selection_without_silent_edits() {
    let mut harness = selected("Customer");
    let (customer, order) = (
        class_id(harness.state(), "Customer"),
        class_id(harness.state(), "Order"),
    );

    harness
        .get_by_label("Order")
        .click_modifiers(Modifiers::COMMAND);
    harness.run();

    assert_eq!(harness.state().selected_elements(), [customer, order]);
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
    assert_eq!(harness.state().selected_elements(), [customer]);
}

#[test]
fn ctrl_click_on_the_canvas_should_add_to_the_selection() {
    let mut harness = selected("Order");
    let (customer, order) = (
        class_id(harness.state(), "Customer"),
        class_id(harness.state(), "Order"),
    );

    harness
        .get_by_label("Class Customer")
        .click_modifiers(Modifiers::COMMAND);
    harness.run();

    assert_eq!(harness.state().selected_elements(), [order, customer]);
}

#[test]
fn plain_click_should_replace_a_mixed_selection() {
    let mut harness = selected("Customer");
    let customer = class_id(harness.state(), "Customer");
    harness
        .get_by_label("Order")
        .click_modifiers(Modifiers::COMMAND);
    harness.run();

    harness.get_by_label("Class Customer").click();
    harness.run();

    assert_eq!(harness.state().selected_elements(), [customer]);
}

#[test]
fn a_draft_committed_in_the_frame_of_an_undo_should_not_undo_the_undo() {
    let mut harness = selected("Customer");
    let customer = class_id(harness.state(), "Customer");
    edit(&mut harness, "Name", "Client");
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(name_of(&harness, customer), "Client");

    // A new, uncommitted draft; clicking Undo moves focus away from the
    // field, which would commit it in the same frame.
    edit(&mut harness, "Name", "Zed");
    harness.get_by_label("Undo").click();
    harness.run();
    harness.run();

    assert_eq!(
        name_of(&harness, customer),
        "Customer",
        "Undo restored the old name and the stale draft did not replace it"
    );
    assert!(harness.state().drafts.is_empty());
    assert!(
        !harness.get_by_label("Redo").accesskit_node().is_disabled(),
        "Redo is still available"
    );
}

#[test]
fn committing_the_same_name_or_multiplicity_should_change_nothing() {
    let mut harness = selected("Customer");
    edit(&mut harness, "Name", "Customer ");
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(
        harness
            .state()
            .project()
            .and_then(|project| project.undo_label()),
        None,
        "a name that differs only by trailing space is not an edit"
    );
    assert_eq!(
        harness.state().notice(),
        None,
        "and it is not reported as a refusal either"
    );

    let mut harness = open(ShellState::default());
    harness.get_by_label(RELATION_ROW).click();
    harness.run();
    edit(&mut harness, "Multiplicity at Order", "0..*");
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(
        harness
            .state()
            .project()
            .and_then(|project| project.undo_label()),
        None,
        "0..* means the same as the stored *"
    );
    assert_eq!(harness.state().notice(), None);
}

#[test]
fn names_should_be_stored_without_surrounding_spaces() {
    let mut harness = selected("Customer");
    let customer = class_id(harness.state(), "Customer");
    edit(&mut harness, "Name", "  Client  ");
    harness.key_press(Key::Enter);
    harness.run();

    assert_eq!(name_of(&harness, customer), "Client");
}

#[test]
fn a_draft_queued_before_undo_in_the_same_batch_should_also_be_dropped() {
    use diagram_desktop::ui::commands::CommandId;
    use diagram_desktop::ui::intents::UiIntent;

    let mut harness = selected("Customer");
    let customer = class_id(harness.state(), "Customer");
    edit(&mut harness, "Name", "Client");
    harness.key_press(Key::Enter);
    harness.run();

    // Draft first, then Undo: the undo still restores "Customer", and Redo
    // brings back "Client", not the draft.
    harness
        .state_mut()
        .queue(UiIntent::RenameClass(customer, String::from("Zed")));
    harness.state_mut().queue(UiIntent::run(CommandId::Undo));
    harness.run();
    assert_eq!(name_of(&harness, customer), "Customer");

    // Redo first, then a draft: the redo stands and the draft is dropped.
    harness.state_mut().queue(UiIntent::run(CommandId::Redo));
    harness
        .state_mut()
        .queue(UiIntent::RenameClass(customer, String::from("Zed")));
    harness.run();
    assert_eq!(name_of(&harness, customer), "Client");
}

#[test]
fn every_association_end_should_be_editable_from_the_keyboard() {
    let mut harness = open(ShellState::default());
    harness.get_by_label(RELATION_ROW).click();
    harness.run();
    let relation_field = |harness: &Harness<'_, ShellState>, field: RelationField| {
        harness
            .state()
            .document()
            .relations
            .values()
            .next()
            .map(|relation| field.value(relation))
            .unwrap_or_default()
    };

    for (label, field, text, expected) in [
        (
            "Role at Customer",
            RelationField::SourceRole,
            "buyer",
            "buyer",
        ),
        (
            "Multiplicity at Customer",
            RelationField::SourceMultiplicity,
            "1..2",
            "1..2",
        ),
        (
            "Role at Order",
            RelationField::TargetRole,
            "purchases",
            "purchases",
        ),
        (
            "Multiplicity at Order",
            RelationField::TargetMultiplicity,
            "3",
            "3",
        ),
    ] {
        edit(&mut harness, label, text);
        harness.key_press(Key::Enter);
        harness.run();
        assert_eq!(relation_field(&harness, field), expected, "{label}");
    }
}

#[test]
fn a_generalization_should_explain_that_it_has_no_ends_to_edit() {
    let mut harness = open(state_with_generalization());

    harness
        .get_by_label("Premium – Customer (generalization)")
        .click();
    harness.run();

    harness.get_by_label("From Premium to Customer");
    harness.get_by_label("Kind: generalization");
    harness.get_by_label_contains("apply to associations only");
    assert!(harness.query_by_label("Role at Premium").is_none());
    assert!(harness.query_by_label("Multiplicity at Customer").is_none());
}
