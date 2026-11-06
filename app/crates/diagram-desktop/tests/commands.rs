//! UI-06: one command catalog feeds the header, tool row, menus, search and
//! shortcuts; disabled commands explain why; text fields keep their keys.

mod common;
use std::collections::HashSet;

use common::{class_id, diagram_id, harness, name_of, select_class, selected_name};
use diagram_core::ids::ElementId;
use diagram_desktop::shell::ShellState;
use diagram_desktop::ui::commands::{
    self, Availability, CATALOG, CommandContext, CommandGroup, CommandId, EffectOwner, Placement,
};
use diagram_desktop::ui::intents::UiIntent;
use diagram_desktop::ui::palette::{self, PaletteEntry};
use eframe::egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};

const SEARCH_FIELD: &str = "Find a command or object";

fn labels(specs: &[&commands::CommandSpec]) -> Vec<&'static str> {
    specs.iter().map(|spec| spec.label).collect()
}

#[test]
fn catalog_should_list_each_command_once_in_id_order() {
    for id in CommandId::ALL {
        assert_eq!(id.spec().id, id);
    }
    let labels: HashSet<_> = CATALOG.iter().map(|spec| spec.label).collect();
    // Primary and alternate shortcuts together: no key combination runs two commands.
    let shortcuts: Vec<_> = CATALOG
        .iter()
        .flat_map(|spec| spec.shortcut.into_iter().chain(spec.alternate_shortcut))
        .collect();
    let unique_shortcuts: HashSet<_> = shortcuts.iter().collect();

    assert_eq!(labels.len(), CATALOG.len());
    assert_eq!(unique_shortcuts.len(), shortcuts.len());
}

#[test]
fn every_disabled_command_should_state_a_reason() {
    for bits in 0..16_u8 {
        let context = CommandContext {
            has_selection: bits & 1 != 0,
            has_project: bits & 2 != 0,
            can_undo: bits & 4 != 0,
            can_redo: bits & 8 != 0,
        };
        for id in CommandId::ALL {
            if let Availability::Disabled(reason) = commands::availability(id, context) {
                assert!(
                    reason.ends_with('.') && reason.len() > 10,
                    "{id:?}: {reason}"
                );
            }
        }
    }
}

#[test]
fn clear_selection_should_need_a_selection() {
    let none = CommandContext {
        has_selection: false,
        ..CommandContext::default()
    };
    let some = CommandContext {
        has_selection: true,
        ..CommandContext::default()
    };

    assert!(!commands::availability(CommandId::ClearSelection, none).is_enabled());
    assert!(commands::availability(CommandId::ClearSelection, some).is_enabled());
}

#[test]
fn header_rows_and_menus_should_come_from_the_catalog() {
    let placed = |placement| commands::placed(placement).collect::<Vec<_>>();
    assert_eq!(
        labels(&placed(Placement::Header)),
        ["Undo", "Redo", "Search", "Save", "Export"]
    );
    assert_eq!(
        labels(&placed(Placement::ToolRow)),
        ["Select", "Pan", "Class", "Association", "Note", "Arrange"]
    );
    assert_eq!(labels(&placed(Placement::TabRow)), ["New diagram"]);
    assert_eq!(
        labels(&placed(Placement::PanelToggle)),
        ["Project panel", "Inspector"]
    );

    let groups = commands::menu_groups();
    let group_names: Vec<_> = groups.iter().map(|(group, _)| *group).collect();
    assert_eq!(group_names, CommandGroup::ALL);
    assert_eq!(
        labels(&groups[0].1),
        ["Save", "Export", "New diagram", "Close project"]
    );
    assert_eq!(labels(&groups[1].1), ["Undo", "Redo", "Clear selection"]);
    assert_eq!(
        labels(&groups[2].1),
        ["Project panel", "Inspector", "Component showcase"]
    );
    let listed: usize = groups.iter().map(|(_, specs)| specs.len()).sum();
    assert_eq!(listed, CATALOG.len());
}

#[test]
fn search_should_match_label_or_group_ignoring_case() {
    assert_eq!(labels(&commands::search("sav")), ["Save"]);
    assert_eq!(
        labels(&commands::search("  EDIT ")),
        ["Undo", "Redo", "Clear selection"]
    );
    assert_eq!(commands::search("").len(), CATALOG.len());
    assert!(commands::search("zzz").is_empty());
}

#[test]
fn palette_entries_should_list_commands_then_matching_objects() {
    let (customer, order) = (ElementId::from_raw(1), ElementId::from_raw(2));
    let names = [(customer, "Customer"), (order, "Order")];

    assert_eq!(
        palette::entries("cust", &names),
        [PaletteEntry::Object(customer)]
    );
    let all = palette::entries("", &names);
    assert!(!all.contains(&PaletteEntry::Command(CommandId::SearchCommands)));
    assert_eq!(all.len(), CATALOG.len() - 1 + names.len());
    assert_eq!(
        palette::entry_label(PaletteEntry::Command(CommandId::Save), &names),
        "File: Save"
    );
    assert_eq!(
        palette::entry_label(PaletteEntry::Object(order), &names),
        "Class: Order"
    );
}

#[test]
fn highlight_should_stay_within_the_results() {
    assert_eq!(palette::move_highlight(0, 3, false, true), 0);
    assert_eq!(palette::move_highlight(2, 3, true, false), 2);
    assert_eq!(palette::move_highlight(1, 3, true, false), 2);
    assert_eq!(palette::move_highlight(5, 3, false, false), 2);
    assert_eq!(palette::move_highlight(4, 0, true, false), 0);
}

#[test]
fn visible_buttons_should_match_catalog_availability() {
    let harness = harness();

    let buttons = [Placement::Header, Placement::ToolRow, Placement::TabRow]
        .into_iter()
        .flat_map(commands::placed);
    for spec in buttons {
        let enabled = commands::availability(spec.id, CommandContext::default()).is_enabled();
        // A command may appear in several places (New diagram: tab row and
        // project panel); every copy must agree with the catalog.
        let mut copies = 0;
        for node in harness.get_all_by_label(spec.label) {
            copies += 1;
            assert_eq!(
                !node.accesskit_node().is_disabled(),
                enabled,
                "{}",
                spec.label
            );
        }
        assert!(copies >= 1, "{} is not shown", spec.label);
    }
}

#[test]
fn unavailable_shortcuts_should_change_nothing() {
    let mut harness = harness();
    select_class(&mut harness, "Order");
    harness.run();
    let before = harness.state().clone();

    harness.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::S);
    harness.run();

    assert_eq!(
        harness.state().selected_element(),
        before.selected_element()
    );
    assert_eq!(harness.state().document(), before.document());
    assert!(harness.state().palette.is_none());
}

#[test]
fn ctrl_k_should_open_search_with_the_field_focused() {
    let mut harness = harness();

    harness.key_press_modifiers(Modifiers::COMMAND, Key::K);
    harness.run();

    assert!(harness.state().palette.is_some());
    assert!(harness.get_by_label(SEARCH_FIELD).is_focused());
}

#[test]
fn search_button_should_open_search() {
    let mut harness = harness();

    harness.get_by_label("Search").click();
    harness.run();

    assert!(harness.state().palette.is_some());
}

#[test]
fn typing_and_enter_should_select_the_matching_object_and_close() {
    let mut harness = harness();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::K);
    harness.run();

    harness.get_by_label(SEARCH_FIELD).type_text("order");
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();

    assert_eq!(selected_name(&harness).as_deref(), Some("Order"));
    assert!(harness.state().palette.is_none());
}

fn first_available_in_harness(query: &str) -> usize {
    let objects = [
        (ElementId::from_raw(1), "Customer"),
        (ElementId::from_raw(2), "Order"),
    ];
    let results = palette::entries(query, &objects);
    let context = CommandContext {
        has_project: true,
        ..CommandContext::default()
    };
    palette::first_available(&results, |entry| match entry {
        PaletteEntry::Command(id) => commands::availability(id, context).is_enabled(),
        PaletteEntry::Object(_) => true,
    })
}

#[test]
fn highlight_should_start_on_the_first_available_result() {
    // "s" first matches "File: Save", which is unavailable.
    assert!(first_available_in_harness("s") > 0);
    let mut harness = harness();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::K);
    harness.run();

    harness.get_by_label(SEARCH_FIELD).type_text("s");
    harness.run();
    assert_eq!(
        harness
            .state()
            .palette
            .as_ref()
            .map(|search| search.highlighted),
        Some(first_available_in_harness("s"))
    );
    harness.key_press(Key::Enter);
    harness.run();

    // An available result ran, so the search closed.
    assert!(harness.state().palette.is_none());
}

#[test]
fn arrow_keys_should_move_the_highlight_to_the_next_result() {
    let mut harness = harness();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::K);
    harness.run();
    harness.get_by_label(SEARCH_FIELD).type_text("o");
    harness.run();

    harness.key_press(Key::ArrowDown);
    harness.run();

    assert_eq!(
        harness
            .state()
            .palette
            .as_ref()
            .map(|search| search.highlighted),
        Some(first_available_in_harness("o") + 1)
    );
}

#[test]
fn first_available_should_skip_unavailable_results() {
    let results = [
        PaletteEntry::Command(CommandId::Save),
        PaletteEntry::Object(ElementId::from_raw(1)),
    ];
    let available = |entry| matches!(entry, PaletteEntry::Object(_));

    assert_eq!(palette::first_available(&results, available), 1);
    assert_eq!(palette::first_available(&results[..1], available), 0);
}

#[test]
fn unavailable_result_should_show_its_reason_and_stay_open_on_enter() {
    let mut harness = harness();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::K);
    harness.run();
    harness.get_by_label(SEARCH_FIELD).type_text("save");
    harness.run();

    harness.get_by_label_contains("this prototype cannot save");
    assert!(
        harness
            .get_by_label_contains("File: Save")
            .accesskit_node()
            .is_disabled()
    );
    harness.key_press(Key::Enter);
    harness.run();

    assert!(harness.state().palette.is_some());
}

#[test]
fn escape_should_close_search_keep_selection_and_restore_focus() {
    let mut harness = harness();
    select_class(&mut harness, "Customer");
    harness.run();
    harness.get_by_label("Name").focus();
    harness.run();

    // Ctrl+K is not a text-editing key, so it works while typing.
    harness.key_press_modifiers(Modifiers::COMMAND, Key::K);
    harness.run();
    assert!(harness.state().palette.is_some());
    harness.key_press(Key::Escape);
    harness.run();

    assert!(harness.state().palette.is_none());
    assert_eq!(selected_name(&harness).as_deref(), Some("Customer"));
    assert!(harness.get_by_label("Name").is_focused());
}

/// Source of every integration test file, to check that contracts name real tests.
fn test_sources() -> String {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| std::fs::read_to_string(entry.path()).ok())
        .collect()
}

#[test]
fn every_command_should_have_a_contract_naming_an_existing_test() {
    let sources = test_sources();
    for id in CommandId::ALL {
        let contract = commands::contract(id);
        assert!(!contract.prerequisite.is_empty(), "{id:?} prerequisite");
        assert!(!contract.effect.is_empty(), "{id:?} effect");
        assert!(
            sources.contains(&format!("fn {}(", contract.test)),
            "{id:?} names a test that does not exist: {}",
            contract.test
        );
    }
}

#[test]
fn model_and_worker_commands_should_stay_unavailable_until_their_prerequisites_exist() {
    // Undo and Redo become available only with history, which this context lacks.
    for has_selection in [false, true] {
        let context = CommandContext {
            has_selection,
            ..CommandContext::default()
        };
        for id in CommandId::ALL {
            let owner = commands::contract(id).owner;
            let enabled = commands::availability(id, context).is_enabled();
            if owner != EffectOwner::Editor {
                assert!(!enabled, "{id:?} pretends to change the project");
            }
        }
    }
}

#[test]
fn queued_intents_should_apply_after_the_frame() {
    let mut harness = harness();

    let details = diagram_id(harness.state(), "Order details");
    let order = class_id(harness.state(), "Order");
    harness.state_mut().queue(UiIntent::OpenDiagram(details));
    harness.state_mut().queue(UiIntent::SelectClass(order));
    harness.run();

    assert_eq!(harness.state().active_diagram(), Some(details));
    assert_eq!(harness.state().selected_element(), Some(order));
}

#[test]
fn queued_unavailable_commands_should_do_nothing() {
    let mut harness = harness();
    let before = harness.state().clone();

    for id in [
        CommandId::Save,
        CommandId::Undo,
        CommandId::AddClass,
        CommandId::NewDiagram,
    ] {
        harness.state_mut().queue(UiIntent::run(id));
    }
    harness.run();

    assert_eq!(harness.state().document(), before.document());
    assert_eq!(harness.state().tabs(), before.tabs());
}

#[test]
fn search_result_for_a_class_should_reveal_it() {
    let mut harness = harness();
    let details = diagram_id(harness.state(), "Order details");
    harness.state_mut().queue(UiIntent::OpenDiagram(details));
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::K);
    harness.run();

    harness.get_by_label(SEARCH_FIELD).type_text("customer");
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();

    let first = diagram_id(harness.state(), "Class diagram 1");
    assert_eq!(
        harness.state().active_diagram(),
        Some(first),
        "opened a diagram showing it"
    );
    assert_eq!(selected_name(&harness).as_deref(), Some("Customer"));
}

fn rename(harness: &mut Harness<'static, ShellState>, from: &str, to: &str) {
    let element = class_id(harness.state(), from);
    harness
        .state_mut()
        .queue(UiIntent::RenameClass(element, to.to_owned()));
    harness.run();
}

#[test]
fn undo_and_redo_should_follow_the_project_history() {
    let mut harness = harness();
    let customer = class_id(harness.state(), "Customer");
    for label in ["Undo", "Redo"] {
        assert!(
            harness.get_by_label(label).accesskit_node().is_disabled(),
            "{label} needs history"
        );
    }

    rename(&mut harness, "Customer", "Client");

    assert_eq!(name_of(&harness, customer), "Client");
    assert!(!harness.get_by_label("Undo").accesskit_node().is_disabled());
    assert!(harness.get_by_label("Redo").accesskit_node().is_disabled());

    harness.get_by_label("Undo").click();
    harness.run();

    assert_eq!(name_of(&harness, customer), "Customer");
    harness.get_by_label_contains("Undid: Rename class.");
    assert!(!harness.get_by_label("Redo").accesskit_node().is_disabled());

    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z);
    harness.run();

    assert_eq!(name_of(&harness, customer), "Client");
    harness.get_by_label_contains("Redid: Rename class.");
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    harness.run();
    assert_eq!(name_of(&harness, customer), "Customer");
}

#[test]
fn close_project_should_close_at_once_when_nothing_changed() {
    let mut harness = harness();

    harness
        .state_mut()
        .queue(UiIntent::run(CommandId::CloseProject));
    harness.run();

    assert!(harness.state().project().is_none());
    assert!(
        harness.state().entry.is_some(),
        "back on the welcome screen"
    );
    assert!(!harness.state().has_dialog());
    harness.get_by_label("New project");
}

#[test]
fn close_project_should_ask_first_when_there_are_unsaved_changes() {
    let mut harness = harness();
    rename(&mut harness, "Customer", "Client");
    harness.get_by_label("Search project").focus();
    harness.run();

    harness
        .state_mut()
        .queue(UiIntent::run(CommandId::CloseProject));
    harness.run();

    assert!(harness.state().has_dialog());
    assert!(harness.state().project().is_some(), "nothing closed yet");
    harness.get_by_label("Close project?");
    harness.get_by_label_contains("has changes that are not saved");
    assert!(
        harness.get_by_label("Cancel").is_focused(),
        "the safe answer has focus"
    );

    // Escape cancels, keeps the project and returns focus to where it was.
    harness.key_press(Key::Escape);
    harness.run();
    harness.run();

    assert!(!harness.state().has_dialog());
    assert!(harness.state().project().is_some());
    assert!(harness.get_by_label("Search project").is_focused());

    // Ask again and confirm: the project closes and the changes are gone.
    harness
        .state_mut()
        .queue(UiIntent::run(CommandId::CloseProject));
    harness.run();
    harness.get_by_label("Discard changes and close").click();
    harness.run();

    assert!(harness.state().project().is_none());
    assert!(harness.state().entry.is_some());
}

#[test]
fn shortcuts_should_do_nothing_behind_an_open_dialog() {
    let mut harness = harness();
    rename(&mut harness, "Customer", "Client");
    harness
        .state_mut()
        .queue(UiIntent::run(CommandId::CloseProject));
    harness.run();
    let before = harness.state().document().clone();

    harness.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    harness.run();

    assert_eq!(harness.state().document(), &before, "Undo must not run");
    assert!(harness.state().has_dialog());
}

#[test]
fn close_project_should_be_listed_in_the_file_menu_and_the_search() {
    assert!(
        commands::search("close project")
            .iter()
            .any(|spec| spec.id == CommandId::CloseProject)
    );
    let groups = commands::menu_groups();
    assert!(groups.iter().any(|(group, specs)| group.label() == "File"
        && specs.iter().any(|spec| spec.id == CommandId::CloseProject)));
}

fn open_changed_project_dialog_from_the_menu(harness: &mut Harness<'static, ShellState>) {
    harness.get_by_label("Menu").click();
    harness.run();
    // Submenus open on hover.
    harness.get_by_label("File").hover();
    harness.run();
    harness.get_by_label_contains("Close project").click();
    harness.run();
}

#[test]
fn closing_from_the_menu_should_return_focus_to_the_menu_button() {
    let mut harness = harness();
    rename(&mut harness, "Customer", "Client");

    open_changed_project_dialog_from_the_menu(&mut harness);
    assert!(harness.state().has_dialog());
    harness.key_press(Key::Escape);
    harness.run();
    harness.run();

    assert!(!harness.state().has_dialog());
    assert!(harness.get_by_label("Menu").is_focused());
}

#[test]
fn closing_from_the_search_should_return_focus_to_where_it_was() {
    let mut harness = harness();
    rename(&mut harness, "Customer", "Client");
    harness.get_by_label("Search project").focus();
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::K);
    harness.run();
    harness
        .get_by_label(SEARCH_FIELD)
        .type_text("close project");
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    assert!(harness.state().has_dialog());

    harness.key_press(Key::Escape);
    harness.run();
    harness.run();

    assert!(!harness.state().has_dialog());
    assert!(harness.get_by_label("Search project").is_focused());
}

#[test]
fn escape_should_close_the_dialog_before_a_drawer() {
    let mut harness = common::harness_at(800.0, 500.0);
    rename(&mut harness, "Customer", "Client");
    harness.get_by_label("Inspector").click();
    harness.run();
    assert!(harness.state().drawer.is_some());
    harness
        .state_mut()
        .queue(UiIntent::run(CommandId::CloseProject));
    harness.run();
    assert!(harness.state().has_dialog());

    harness.key_press(Key::Escape);
    harness.run();

    assert!(
        !harness.state().has_dialog(),
        "the dialog takes Escape first"
    );
    assert!(harness.state().drawer.is_some(), "the drawer stays open");
    harness.key_press(Key::Escape);
    harness.run();
    assert!(harness.state().drawer.is_none());
}

#[test]
fn dialog_should_work_from_the_keyboard() {
    let mut harness = harness();
    rename(&mut harness, "Customer", "Client");
    harness
        .state_mut()
        .queue(UiIntent::run(CommandId::CloseProject));
    harness.run();
    assert!(harness.get_by_label("Cancel").is_focused());

    harness.key_press(Key::Tab);
    harness.run();
    assert!(
        harness
            .get_by_label("Discard changes and close")
            .is_focused()
    );
    harness.key_press(Key::Enter);
    harness.run();
    harness.run();

    assert!(harness.state().project().is_none());
}

#[test]
fn requests_queued_for_an_old_project_should_not_touch_the_next_one() {
    // IDs restart in every project, so an old project's Customer ID names the
    // new sample's Customer too. A rename queued before the sample opened
    // must be dropped, not applied to it.
    let old_customer = {
        let state = ShellState::default();
        class_id(&state, "Customer")
    };
    let mut harness = common::harness_with(common::LARGE, ShellState::welcome());

    harness.state_mut().queue(UiIntent::OpenSample);
    harness
        .state_mut()
        .queue(UiIntent::RenameClass(old_customer, String::from("Ghost")));
    harness.run();
    harness.run();

    assert!(harness.state().project().is_some(), "the sample opened");
    assert_eq!(name_of(&harness, old_customer), "Customer");
    assert_eq!(
        harness
            .state()
            .project()
            .and_then(|project| project.undo_label()),
        None,
        "no edit reached the new project's history"
    );
    harness.get_by_label_contains("remaining requests were dropped");
}

#[test]
fn ctrl_y_should_redo_like_ctrl_shift_z() {
    let mut harness = harness();
    let customer = class_id(harness.state(), "Customer");
    rename(&mut harness, "Customer", "Client");
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    harness.run();
    assert_eq!(name_of(&harness, customer), "Customer");

    harness.key_press_modifiers(Modifiers::COMMAND, Key::Y);
    harness.run();

    assert_eq!(name_of(&harness, customer), "Client");
    harness.get_by_label_contains("Redid: Rename class.");
}

#[test]
fn ctrl_y_in_a_text_field_should_be_left_to_the_field() {
    let mut harness = harness();
    let customer = class_id(harness.state(), "Customer");
    rename(&mut harness, "Customer", "Client");
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    harness.run();
    harness.get_by_label("Search project").click();
    harness.run();

    harness.key_press_modifiers(Modifiers::COMMAND, Key::Y);
    harness.run();

    assert_eq!(name_of(&harness, customer), "Customer", "history untouched");
    assert!(!harness.get_by_label("Redo").accesskit_node().is_disabled());
}
