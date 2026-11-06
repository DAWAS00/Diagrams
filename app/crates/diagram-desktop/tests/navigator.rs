//! UI-05: project panel (navigator) and diagram tabs.

mod common;
use common::{class_id, diagram_id, harness, harness_with, state_with_many_diagrams};
use diagram_desktop::session::ProjectSession;
use diagram_desktop::shell::ShellState;
use diagram_desktop::ui::navigator;
use diagram_desktop::ui::read_model;
use eframe::egui::Key;
use eframe::egui::accesskit::{Role, Toggled};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};

/// The project-panel row for `label` (below the header, tab and tool rows).
fn panel_row<'a>(harness: &'a Harness<'_, ShellState>, label: &'a str) -> egui_kittest::Node<'a> {
    harness
        .get_all_by_label(label)
        .find(|node| {
            node.accesskit_node()
                .bounding_box()
                .is_some_and(|rect| rect.y0 >= 126.0 && rect.x1 <= 250.0)
        })
        .unwrap_or_else(|| harness.get_by_label(label))
}

fn tab_titles(harness: &Harness<'_, ShellState>) -> Vec<String> {
    let state = harness.state();
    state
        .tabs()
        .iter()
        .filter_map(|tab| state.document().diagrams.get(&tab.diagram))
        .map(|diagram| diagram.name.as_str().to_owned())
        .collect()
}

#[test]
fn matching_should_ignore_case_and_accept_an_empty_query() {
    assert!(navigator::matches("", "Order"));
    assert!(navigator::matches("  ORD ", "Order details"));
    assert!(!navigator::matches("zzz", "Order"));
}

#[test]
fn shown_in_should_name_every_diagram_showing_a_class() {
    let state = ShellState::default();
    let document = state.document();
    let (customer, order) = (class_id(&state, "Customer"), class_id(&state, "Order"));

    assert_eq!(
        read_model::shown_in(document, customer),
        "Shown in Class diagram 1"
    );
    assert_eq!(
        read_model::shown_in(document, order),
        "Shown in Class diagram 1 and Order details"
    );
    assert_eq!(
        read_model::shown_in(document, diagram_core::ids::ElementId::from_raw(u64::MAX)),
        "Not shown in any diagram"
    );
}

#[test]
fn diagrams_showing_should_list_diagrams_in_project_order() {
    let state = ShellState::default();
    let order = class_id(&state, "Order");

    assert_eq!(
        read_model::diagrams_showing(state.document(), order),
        [
            diagram_id(&state, "Class diagram 1"),
            diagram_id(&state, "Order details")
        ]
    );
    assert!(
        read_model::diagrams_showing(
            state.document(),
            diagram_core::ids::ElementId::from_raw(u64::MAX)
        )
        .is_empty()
    );
}

#[test]
fn tabs_that_fit_should_keep_order_and_always_show_the_active_tab() {
    assert_eq!(
        navigator::tabs_that_fit(&[100.0; 3], 400.0, Some(0)),
        (vec![0, 1, 2], vec![])
    );
    assert_eq!(
        navigator::tabs_that_fit(&[100.0; 4], 250.0, Some(0)),
        (vec![0, 1], vec![2, 3])
    );
    // The active tab stays visible even when it comes after the overflow.
    assert_eq!(
        navigator::tabs_that_fit(&[100.0; 4], 250.0, Some(3)),
        (vec![0, 3], vec![1, 2])
    );
}

#[test]
fn panel_should_list_diagrams_and_model_with_view_wording() {
    let harness = harness();

    harness.get_by_label("Diagrams (views of the model)");
    harness.get_by_label("Model (shared by all diagrams)");
    panel_row(&harness, "Order details");
    panel_row(&harness, "Customer");
    assert_eq!(tab_titles(&harness), ["Class diagram 1"]);
}

#[test]
fn tabs_should_be_exposed_as_tabs_with_a_named_close_button() {
    let harness = harness();
    let tab = harness
        .get_all_by_label("Class diagram 1")
        .find(|node| node.accesskit_node().role() == Role::Tab);

    assert!(tab.is_some(), "no Tab role for the open diagram");
    harness.get_by_label("Close Class diagram 1");
}

#[test]
fn opening_a_diagram_should_add_a_tab_and_show_only_its_classes() {
    let mut harness = harness();

    panel_row(&harness, "Order details").click();
    harness.run();

    assert_eq!(tab_titles(&harness), ["Class diagram 1", "Order details"]);
    assert_eq!(harness.state().active_tab(), Some(1));
    harness.get_by_label("Class Order");
    assert!(harness.query_by_label("Class Customer").is_none());
}

#[test]
fn each_tab_should_keep_its_own_selection() {
    let mut harness = harness();
    let customer = class_id(harness.state(), "Customer");
    harness.state_mut().select(customer);
    harness.run();
    panel_row(&harness, "Order details").click();
    harness.run();
    assert_eq!(
        harness.state().selected_element(),
        None,
        "new tab starts without a selection"
    );
    harness.get_by_label("Class Order").click();
    harness.run();

    if let Some(tab) = harness
        .get_all_by_label("Class diagram 1")
        .find(|node| node.accesskit_node().role() == Role::Tab)
    {
        tab.click();
    }
    harness.run();

    assert_eq!(harness.state().selected_element(), Some(customer));
}

#[test]
fn closing_a_tab_should_never_remove_model_elements() {
    let mut harness = harness();

    harness.get_by_label("Close Class diagram 1").click();
    harness.run();

    assert!(harness.state().tabs().is_empty());
    assert_eq!(harness.state().active_tab(), None);
    assert_eq!(harness.state().document().elements.len(), 2);
    panel_row(&harness, "Customer");
    assert!(harness.query_by_label("Class Customer").is_none());
}

#[test]
fn selecting_a_class_not_on_this_diagram_should_reveal_it() {
    let mut harness = harness();
    panel_row(&harness, "Order details").click();
    harness.run();

    panel_row(&harness, "Customer").click();
    harness.run();

    let state = harness.state();
    assert_eq!(
        state.active_diagram(),
        Some(diagram_id(state, "Class diagram 1"))
    );
    assert_eq!(state.selected_element(), Some(class_id(state, "Customer")));
    harness.get_by_label("Class Customer");
}

#[test]
fn search_should_filter_both_lists_and_explain_no_matches() {
    let mut harness = harness();
    harness.get_by_label("Search project").focus();
    harness.run();

    harness.get_by_label("Search project").type_text("ord");
    harness.run();
    panel_row(&harness, "Order details");
    assert!(
        harness.query_all_by_label("Customer").all(|node| node
            .accesskit_node()
            .bounding_box()
            .is_none_or(|rect| rect.x1 > 250.0)),
        "Customer still listed"
    );

    harness.get_by_label("Search project").type_text("zzz");
    harness.run();
    harness.get_by_label_contains("No diagrams match");
    harness.get_by_label_contains("No classes match");

    harness.get_by_label("Clear search").click();
    harness.run();
    assert_eq!(harness.state().navigator_search, "");
    panel_row(&harness, "Customer");
}

#[test]
fn empty_project_should_say_there_are_no_classes_yet() {
    let mut state = ShellState::welcome();
    state.open_project(ProjectSession::blank("Empty"));
    let harness = harness_with((1280.0, 800.0), state);

    harness.get_by_label_contains("No classes yet");
}

#[test]
fn inspector_should_say_which_diagrams_show_the_class() {
    let mut harness = harness();
    let order = class_id(harness.state(), "Order");
    harness.state_mut().select(order);
    harness.run();

    harness.get_by_label("Shown in Class diagram 1 and Order details");
}

#[test]
fn keyboard_should_open_a_diagram_from_the_panel() {
    let mut harness = harness();
    panel_row(&harness, "Order details").focus();
    harness.run();

    harness.key_press(Key::Enter);
    harness.run();

    let state = harness.state();
    assert_eq!(
        state.active_diagram(),
        Some(diagram_id(state, "Order details"))
    );
}

fn many_open_tabs() -> ShellState {
    let mut state = state_with_many_diagrams(8);
    let seventh = diagram_id(&state, "Customer overview part 7");
    state.activate_tab(seventh);
    state
}

#[test]
fn tabs_that_do_not_fit_should_move_to_the_more_tabs_menu() {
    let mut harness = harness_with((800.0, 500.0), many_open_tabs());

    let more = harness.get_by_label_contains("More tabs (");
    let active = harness
        .get_all_by_label("Customer overview part 7")
        .find(|node| node.accesskit_node().role() == Role::Tab)
        .map(|node| node.accesskit_node().toggled());
    assert_eq!(
        active,
        Some(Some(Toggled::True)),
        "active tab must stay visible"
    );
    more.click();
    harness.run();
    harness.get_by_label("Customer overview part 8").click();
    harness.run();

    let state = harness.state();
    assert_eq!(
        state.active_diagram(),
        Some(diagram_id(state, "Customer overview part 8"))
    );
}

#[test]
fn stale_ids_should_be_ignored_with_a_message_and_never_change_the_selection() {
    use diagram_core::ids::{DiagramId, ElementId, RelationId};
    use diagram_desktop::ui::intents::UiIntent;

    let mut harness = harness();
    let customer = class_id(harness.state(), "Customer");
    harness.state_mut().select(customer);
    harness.run();

    let stale = [
        (UiIntent::SelectClass(ElementId::from_raw(999)), "class"),
        (UiIntent::RevealClass(ElementId::from_raw(999)), "class"),
        (
            UiIntent::SelectRelation(RelationId::from_raw(999)),
            "relationship",
        ),
        (UiIntent::OpenDiagram(DiagramId::from_raw(999)), "diagram"),
        (UiIntent::ActivateTab(DiagramId::from_raw(999)), "tab"),
        (UiIntent::CloseTab(DiagramId::from_raw(999)), "tab"),
    ];
    for (intent, what) in stale {
        let tabs_before = harness.state().tabs().len();
        harness.state_mut().queue(intent);
        harness.run();

        assert_eq!(
            harness.state().selected_element(),
            Some(customer),
            "a stale {what} must not change the selection"
        );
        assert_eq!(harness.state().tabs().len(), tabs_before);
        harness.get_by_label_contains(&format!("That {what} no longer exists"));
    }
}

#[test]
fn a_stale_rename_should_change_nothing() {
    use diagram_core::ids::ElementId;
    use diagram_desktop::ui::intents::UiIntent;

    let mut harness = harness();
    let before = harness.state().document().clone();

    harness.state_mut().queue(UiIntent::RenameClass(
        ElementId::from_raw(999),
        String::from("Ghost"),
    ));
    harness.run();

    assert_eq!(harness.state().document(), &before);
    harness.get_by_label_contains("That class no longer exists");
}

#[test]
fn stale_toggle_and_relation_field_intents_should_be_ignored_with_a_message() {
    use diagram_core::ids::{ElementId, RelationId};
    use diagram_desktop::ui::inspector::RelationField;
    use diagram_desktop::ui::intents::UiIntent;

    let mut harness = harness();
    let before = harness.state().document().clone();
    let customer = class_id(harness.state(), "Customer");
    harness.state_mut().select(customer);
    harness.run();

    harness
        .state_mut()
        .queue(UiIntent::ToggleClassInSelection(ElementId::from_raw(999)));
    harness.run();
    assert_eq!(harness.state().selected_elements(), [customer]);
    harness.get_by_label_contains("That class no longer exists");

    harness.state_mut().queue(UiIntent::SetRelationField(
        RelationId::from_raw(999),
        RelationField::SourceRole,
        String::from("x"),
    ));
    harness.run();
    assert_eq!(harness.state().document(), &before);
    harness.get_by_label_contains("That relationship no longer exists");
}

#[test]
fn undoing_a_creation_should_drop_the_selection_and_stale_requests_after_it() {
    use diagram_core::command::Command;
    use diagram_desktop::ui::commands::CommandId;
    use diagram_desktop::ui::intents::UiIntent;

    // History that already holds "add class": Undo removes it for real.
    let mut session = ProjectSession::sample();
    let diagram = session.document().diagram_order[0];
    common::run(
        &mut session,
        Command::CreateClass {
            diagram,
            name: String::from("Temp"),
            bounds: common::bounds(400.0, 300.0),
        },
    );
    let mut state = common::state_with(session);
    let temp = class_id(&state, "Temp");
    state.select(temp);
    let mut harness = harness_with((1280.0, 800.0), state);
    harness.get_by_label("Class Temp");

    harness.state_mut().queue(UiIntent::run(CommandId::Undo));
    harness.run();
    harness.run();

    assert_eq!(
        harness.state().selected_element(),
        None,
        "the selection was pruned"
    );
    assert!(harness.query_by_label("Class Temp").is_none());
    // Redo brings the class back with the same ID; the selection must not
    // come back with it (it was dropped, not just hidden).
    harness.state_mut().queue(UiIntent::run(CommandId::Redo));
    harness.run();
    harness.run();
    harness.get_by_label("Class Temp");
    assert_eq!(
        harness.state().selected_element(),
        None,
        "a pruned selection stays pruned after Redo"
    );
    harness.state_mut().queue(UiIntent::run(CommandId::Undo));
    harness.run();
    harness.run();
    harness
        .state_mut()
        .queue(UiIntent::RenameClass(temp, String::from("Ghost")));
    harness.run();
    harness.get_by_label_contains("That class no longer exists");
}

#[test]
fn closing_a_background_tab_should_keep_the_active_tabs_selection() {
    use diagram_desktop::ui::intents::UiIntent;

    let mut harness = harness();
    let (customer, order) = (
        class_id(harness.state(), "Customer"),
        class_id(harness.state(), "Order"),
    );
    let details = diagram_id(harness.state(), "Order details");
    let first = diagram_id(harness.state(), "Class diagram 1");
    harness.state_mut().queue(UiIntent::OpenDiagram(details));
    harness.run();
    harness.state_mut().queue(UiIntent::ActivateTab(first));
    harness.run();
    harness.state_mut().select(customer);
    harness.state_mut().toggle_in_selection(order);
    harness.run();

    harness.state_mut().queue(UiIntent::CloseTab(details));
    harness.run();

    assert_eq!(harness.state().selected_elements(), [customer, order]);
}

#[test]
fn a_background_tabs_selection_should_not_come_back_with_redo() {
    use diagram_core::command::Command;
    use diagram_desktop::ui::commands::CommandId;
    use diagram_desktop::ui::intents::UiIntent;

    let mut session = ProjectSession::sample();
    let first = session.document().diagram_order[0];
    common::run(
        &mut session,
        Command::CreateClass {
            diagram: first,
            name: String::from("Temp"),
            bounds: common::bounds(400.0, 300.0),
        },
    );
    let mut state = common::state_with(session);
    let (temp, details) = (
        class_id(&state, "Temp"),
        diagram_id(&state, "Order details"),
    );
    state.select(temp);
    state.open_diagram(details);
    let mut harness = harness_with((1280.0, 800.0), state);

    // Tab "Class diagram 1" remembers Temp as selected while another tab is active.
    harness.state_mut().queue(UiIntent::run(CommandId::Undo));
    harness.run();
    harness.state_mut().queue(UiIntent::run(CommandId::Redo));
    harness.run();
    harness.state_mut().queue(UiIntent::ActivateTab(first));
    harness.run();
    harness.run();

    harness.get_by_label("Class Temp");
    assert_eq!(
        harness.state().selected_element(),
        None,
        "the selection dropped by Undo stays dropped in the other tab"
    );
}
