//! H5: `presentation::build` reflects the state the views must draw.
//!
//! The Phase 0 checks prove that building the presentation from the sample
//! state reproduces the labels, availability and selection the shell showed
//! before the contract existed, and that the real shell draws exactly what
//! the presentation says.

mod common_state;

use common_state::{apply_all, sample};
use diagram_desktop::app_state::BLANK_NAME_WARNING;
use diagram_desktop::intent::{Target, UiIntent};
use diagram_desktop::presentation::{
    self, CommandView, FieldView, InspectorKind, NOTHING_SELECTED, Presentation, SAVE_STATE,
};
use diagram_desktop::shell::{self, ShellState};
use diagram_desktop::ui::commands::{self, CATALOG, CommandId, Placement};
use eframe::egui::{self, ThemePreference, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};

fn labels(views: &[CommandView]) -> Vec<&str> {
    views.iter().map(|view| view.label.as_str()).collect()
}

/// The Name field and attributes when the inspector shows one class.
fn class_inspector(kind: &InspectorKind) -> Option<(&FieldView, &[String])> {
    match kind {
        InspectorKind::Class { name, attributes } => Some((name, attributes)),
        InspectorKind::Nothing { .. } => None,
    }
}

fn all_commands(view: &Presentation) -> Vec<&CommandView> {
    view.header
        .commands
        .iter()
        .chain(&view.tabs.commands)
        .chain(&view.tools.tools)
        .chain([&view.tools.navigator_toggle, &view.tools.inspector_toggle])
        .chain(view.header.menu.iter().flat_map(|group| &group.commands))
        .collect()
}

#[test]
fn phase0_build_should_reproduce_the_shell_labels_and_order() {
    let view = presentation::build(&sample());

    assert_eq!(view.header.title, "Sample project · not saved");
    let placed = |placement| {
        commands::placed(placement)
            .map(|spec| spec.label)
            .collect::<Vec<_>>()
    };
    assert_eq!(labels(&view.header.commands), placed(Placement::Header));
    assert_eq!(labels(&view.tools.tools), placed(Placement::ToolRow));
    assert_eq!(labels(&view.tabs.commands), placed(Placement::TabRow));
    assert_eq!(view.tools.navigator_toggle.label, "Project panel");
    assert_eq!(view.tools.inspector_toggle.label, "Inspector");
    assert_eq!(view.tabs.tabs.len(), 1);
    assert!(view.tabs.tabs[0].active);

    let menu: Vec<_> = view
        .header
        .menu
        .iter()
        .map(|group| (group.group, labels(&group.commands)))
        .collect();
    let expected: Vec<_> = commands::menu_groups()
        .into_iter()
        .map(|(group, specs)| (group, specs.iter().map(|spec| spec.label).collect()))
        .collect();
    assert_eq!(menu, expected);
}

#[test]
fn phase0_availability_and_shortcuts_should_equal_the_catalog() {
    let state = sample();
    let view = presentation::build(&state);
    let context = state.command_context();
    let ctx = egui::Context::default();

    for command in all_commands(&view) {
        let spec = command.id.spec();
        assert_eq!(command.label, spec.label);
        assert_eq!(
            command.availability,
            commands::availability(command.id, context),
            "{:?}",
            command.id
        );
        assert_eq!(
            command.shortcut_text,
            spec.shortcut.map(|shortcut| ctx.format_shortcut(&shortcut)),
            "{:?}",
            command.id
        );
    }
    // Every catalog command reaches the screen at least through the menu.
    for spec in &CATALOG {
        assert!(
            all_commands(&view)
                .iter()
                .any(|command| command.id == spec.id),
            "{:?}",
            spec.id
        );
    }
}

#[test]
fn phase0_select_tool_and_docked_panel_toggles_should_show_pressed() {
    let view = presentation::build(&sample());

    let select = view
        .tools
        .tools
        .iter()
        .find(|tool| tool.id == CommandId::SelectTool);
    assert_eq!(select.and_then(|tool| tool.active), Some(true));
    assert_eq!(view.tools.navigator_toggle.active, Some(true));
    assert_eq!(view.tools.inspector_toggle.active, Some(true));
}

#[test]
fn phase0_nothing_selected_should_show_guidance_and_no_selection() {
    let view = presentation::build(&sample());

    assert_eq!(
        view.inspector.kind,
        InspectorKind::Nothing {
            guidance: NOTHING_SELECTED
        }
    );
    assert_eq!(view.status.selection_text, "No selection");
    assert_eq!(view.status.save_text, SAVE_STATE);
    assert_eq!(view.status.theme, ThemePreference::Light);
    assert!(view.navigator.model.iter().all(|row| !row.selected));
    assert!(view.canvas.nodes.iter().all(|node| !node.selected));
    let names: Vec<_> = view.navigator.model.iter().map(|row| &row.label).collect();
    assert_eq!(names, ["Customer", "Order"]);
    assert_eq!(view.canvas.empty_text, None);
    assert!(view.dialog.is_none());
}

#[test]
fn selecting_a_class_should_update_every_view_of_it() {
    let mut state = sample();
    apply_all(
        &mut state,
        [UiIntent::Select {
            target: Target::Class(1),
            toggle: false,
        }],
    );
    let view = presentation::build(&state);

    let selected: Vec<_> = view
        .navigator
        .model
        .iter()
        .map(|row| row.selected)
        .collect();
    assert_eq!(selected, [false, true]);
    let nodes: Vec<_> = view.canvas.nodes.iter().map(|node| node.selected).collect();
    assert_eq!(nodes, [false, true]);
    assert_eq!(view.canvas.nodes[1].accessible_name, "Class Order");
    assert_eq!(view.status.selection_text, "Selected: Order");
    let (name, attributes) = class_inspector(&view.inspector.kind).unzip();
    assert_eq!(name.map(|name| name.label), Some("Name"));
    assert_eq!(name.map(|name| name.text.as_str()), Some("Order"));
    assert_eq!(name.and_then(|name| name.problem.clone()), None);
    assert_eq!(
        attributes,
        Some(&[String::from("id: OrderId"), String::from("total: Money")][..])
    );
    // Clear selection becomes available once something is selected.
    let clear = view
        .header
        .menu
        .iter()
        .flat_map(|group| &group.commands)
        .find(|command| command.id == CommandId::ClearSelection);
    assert!(clear.is_some_and(CommandView::is_enabled));
}

#[test]
fn blank_name_should_present_the_warning_beside_the_field() {
    let mut state = sample();
    state.select(0);
    state.classes[0].name = String::from("   ");
    let view = presentation::build(&state);

    let problem = class_inspector(&view.inspector.kind).and_then(|(name, _)| name.problem.clone());
    assert_eq!(problem.as_deref(), Some(BLANK_NAME_WARNING));
    assert_eq!(view.canvas.nodes[0].name, "Unnamed class");
}

#[test]
fn empty_project_should_present_the_empty_canvas_text() {
    let mut state = sample();
    apply_all(
        &mut state,
        [UiIntent::CreateProject {
            name: String::from("Shop"),
        }],
    );
    let view = presentation::build(&state);

    assert_eq!(view.header.title, "Shop · not saved");
    assert!(view.canvas.nodes.is_empty());
    assert_eq!(view.canvas.empty_text, Some(presentation::EMPTY_CANVAS));
}

#[test]
fn open_search_should_present_its_dialog_and_query() {
    let mut state = sample();
    apply_all(&mut state, [UiIntent::Command(CommandId::SearchCommands)]);
    apply_all(
        &mut state,
        [UiIntent::Search {
            scope: diagram_desktop::intent::SearchScope::Commands,
            query: String::from("ord"),
        }],
    );
    let view = presentation::build(&state);

    assert_eq!(view.dialog, Some(presentation::DialogView::CommandSearch));
    assert!(view.command_search.open);
    assert_eq!(view.command_search.query, "ord");
}

#[test]
fn phase0_shell_should_draw_exactly_the_presented_commands() {
    let mut harness = Harness::builder()
        .with_size(Vec2::new(1280.0, 800.0))
        .build_ui_state(shell::show, ShellState::default());
    harness.run();
    let view = presentation::build(harness.state());

    for command in view
        .header
        .commands
        .iter()
        .chain(&view.tabs.commands)
        .chain(&view.tools.tools)
    {
        let node = harness.get_by_label(&command.label);
        assert_eq!(
            !node.accesskit_node().is_disabled(),
            command.is_enabled(),
            "{}",
            command.label
        );
    }
    harness.get_by_label(&view.header.title);
    harness.get_by_label(&view.status.selection_text);
    harness.get_by_label(view.navigator.title);
}
