//! UI-06: one command catalog feeds the header, tool row, menus, search and
//! shortcuts; disabled commands explain why; text fields keep their keys.

mod common;
use std::collections::HashSet;

use common::harness;
use diagram_desktop::ui::commands::{
    self, Availability, CATALOG, CommandContext, CommandGroup, CommandId, EffectOwner, Placement,
};
use diagram_desktop::ui::intents::UiIntent;
use diagram_desktop::ui::palette::{self, PaletteEntry};
use eframe::egui::{Key, Modifiers};
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
    let shortcuts: Vec<_> = CATALOG.iter().filter_map(|spec| spec.shortcut).collect();
    let unique_shortcuts: HashSet<_> = shortcuts.iter().collect();

    assert_eq!(labels.len(), CATALOG.len());
    assert_eq!(unique_shortcuts.len(), shortcuts.len());
}

#[test]
fn every_disabled_command_should_state_a_reason() {
    for has_selection in [false, true] {
        let context = CommandContext { has_selection };
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
    };
    let some = CommandContext {
        has_selection: true,
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
    assert_eq!(labels(&groups[0].1), ["Save", "Export", "New diagram"]);
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
    let names = ["Customer", "Order"];

    assert_eq!(palette::entries("cust", &names), [PaletteEntry::Object(0)]);
    let all = palette::entries("", &names);
    assert!(!all.contains(&PaletteEntry::Command(CommandId::SearchCommands)));
    assert_eq!(all.len(), CATALOG.len() - 1 + names.len());
    assert_eq!(
        palette::entry_label(PaletteEntry::Command(CommandId::Save), &names),
        "File: Save"
    );
    assert_eq!(
        palette::entry_label(PaletteEntry::Object(1), &names),
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
    harness.state_mut().select(1);
    harness.run();
    let before = harness.state().clone();

    harness.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::S);
    harness.run();

    assert_eq!(harness.state().selected, before.selected);
    assert_eq!(harness.state().classes, before.classes);
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

    assert_eq!(harness.state().selected, Some(1));
    assert!(harness.state().palette.is_none());
}

fn first_available_in_harness(query: &str) -> usize {
    let results = palette::entries(query, &["Customer", "Order"]);
    palette::first_available(&results, |entry| match entry {
        PaletteEntry::Command(id) => {
            commands::availability(id, CommandContext::default()).is_enabled()
        }
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
        PaletteEntry::Object(0),
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
    harness.state_mut().select(0);
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
    assert_eq!(harness.state().selected, Some(0));
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
fn model_and_worker_commands_should_stay_unavailable_until_the_model_exists() {
    for has_selection in [false, true] {
        let context = CommandContext { has_selection };
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

    harness.state_mut().queue(UiIntent::OpenDiagram(1));
    harness.state_mut().queue(UiIntent::SelectClass(1));
    harness.run();

    assert_eq!(harness.state().active_diagram(), Some(1));
    assert_eq!(harness.state().selected, Some(1));
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

    assert_eq!(harness.state().classes, before.classes);
    assert_eq!(harness.state().diagrams, before.diagrams);
    assert_eq!(harness.state().tabs, before.tabs);
}

#[test]
fn search_result_for_a_class_should_reveal_it() {
    let mut harness = harness();
    harness.state_mut().queue(UiIntent::OpenDiagram(1));
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::K);
    harness.run();

    harness.get_by_label(SEARCH_FIELD).type_text("customer");
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();

    assert_eq!(
        harness.state().active_diagram(),
        Some(0),
        "opened a diagram showing it"
    );
    assert_eq!(harness.state().selected, Some(0));
}
