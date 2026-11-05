//! UI-05: project panel (navigator) and diagram tabs.

use diagram_desktop::shell::{self, ShellState};
use diagram_desktop::ui::navigator::{self, Diagram, OpenTab};
use eframe::egui::accesskit::{Role, Toggled};
use eframe::egui::{Key, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};

fn harness_with(size: (f32, f32), state: ShellState) -> Harness<'static, ShellState> {
    let mut harness = Harness::builder()
        .with_size(Vec2::new(size.0, size.1))
        .build_ui_state(shell::show, state);
    harness.run();
    harness
}

fn harness() -> Harness<'static, ShellState> {
    harness_with((1280.0, 800.0), ShellState::default())
}

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
        .tabs
        .iter()
        .map(|tab| state.diagrams[tab.diagram].name.clone())
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
    let diagrams = [
        Diagram::new("Overview", &[0, 1]),
        Diagram::new("Details", &[1]),
    ];

    assert_eq!(navigator::shown_in(&diagrams, 0), "Shown in Overview");
    assert_eq!(
        navigator::shown_in(&diagrams, 1),
        "Shown in Overview and Details"
    );
    assert_eq!(
        navigator::shown_in(&diagrams, 2),
        "Not shown in any diagram"
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
    assert_eq!(harness.state().active_tab, Some(1));
    harness.get_by_label("Class Order");
    assert!(harness.query_by_label("Class Customer").is_none());
}

#[test]
fn each_tab_should_keep_its_own_selection() {
    let mut harness = harness();
    harness.state_mut().select(0);
    harness.run();
    panel_row(&harness, "Order details").click();
    harness.run();
    assert_eq!(
        harness.state().selected,
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

    assert_eq!(harness.state().selected, Some(0));
}

#[test]
fn closing_a_tab_should_never_remove_model_elements() {
    let mut harness = harness();

    harness.get_by_label("Close Class diagram 1").click();
    harness.run();

    assert!(harness.state().tabs.is_empty());
    assert_eq!(harness.state().active_tab, None);
    assert_eq!(harness.state().classes.len(), 2);
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

    assert_eq!(harness.state().active_diagram(), Some(0));
    assert_eq!(harness.state().selected, Some(0));
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
    let mut state = ShellState::default();
    state.classes.clear();
    state.diagrams = vec![Diagram::new("Class diagram 1", &[])];
    let harness = harness_with((1280.0, 800.0), state);

    harness.get_by_label_contains("No classes yet");
}

#[test]
fn inspector_should_say_which_diagrams_show_the_class() {
    let mut harness = harness();
    harness.state_mut().select(1);
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

    assert_eq!(harness.state().active_diagram(), Some(1));
}

fn many_open_tabs() -> ShellState {
    let mut state = ShellState::default();
    state.diagrams = (1..=8)
        .map(|number| Diagram::new(&format!("Customer overview part {number}"), &[0, 1]))
        .collect();
    state.tabs = (0..8)
        .map(|diagram| OpenTab {
            diagram,
            selected: None,
        })
        .collect();
    state.active_tab = Some(6);
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

    assert_eq!(harness.state().active_tab, Some(7));
}
