//! H3: a key press with a given `InputContext` produces the right command,
//! or none (functional spec §8, input dispatch).

mod common_state;

use std::collections::HashSet;

use common_state::{canvas_focused, ime_composing, in_region, input_with, key, modal_open, typing};
use diagram_desktop::intent::{InputContext, Region};
use diagram_desktop::ui::commands::{self, CATALOG, CommandId, ShortcutScope};
use eframe::egui::{self, Key, Modifiers};

fn press(context: &InputContext, modifiers: Modifiers, pressed: Key) -> Option<CommandId> {
    let mut input = input_with(vec![key(modifiers, pressed)]);
    commands::take_shortcut_in(&mut input, context)
}

#[test]
fn every_catalog_shortcut_should_run_its_command_on_the_canvas() {
    for spec in &CATALOG {
        for shortcut in spec.all_shortcuts() {
            assert_eq!(
                press(&canvas_focused(), shortcut.modifiers, shortcut.logical_key),
                Some(spec.id),
                "{:?} for {:?}",
                shortcut,
                spec.id
            );
        }
    }
}

#[test]
fn no_two_commands_should_share_a_key_combination() {
    let mut seen = HashSet::new();
    for spec in &CATALOG {
        for shortcut in spec.all_shortcuts() {
            assert!(seen.insert(shortcut), "{shortcut:?} is used twice");
        }
    }
}

#[test]
fn search_should_open_with_ctrl_k_and_ctrl_shift_p() {
    // Decision D1.
    let context = canvas_focused();
    assert_eq!(
        press(&context, Modifiers::COMMAND, Key::K),
        Some(CommandId::SearchCommands)
    );
    assert_eq!(
        press(&context, Modifiers::COMMAND | Modifiers::SHIFT, Key::P),
        Some(CommandId::SearchCommands)
    );
}

#[test]
fn the_more_specific_shortcut_should_win() {
    let context = canvas_focused();
    let ctrl_shift = Modifiers::COMMAND | Modifiers::SHIFT;
    assert_eq!(press(&context, ctrl_shift, Key::Z), Some(CommandId::Redo));
    assert_eq!(
        press(&context, Modifiers::COMMAND, Key::Z),
        Some(CommandId::Undo)
    );
    assert_eq!(press(&context, ctrl_shift, Key::S), Some(CommandId::SaveAs));
    assert_eq!(
        press(&context, Modifiers::COMMAND, Key::S),
        Some(CommandId::Save)
    );
    assert_eq!(
        press(&context, Modifiers::SHIFT, Key::G),
        Some(CommandId::ToggleSnap)
    );
    assert_eq!(
        press(&context, Modifiers::NONE, Key::G),
        Some(CommandId::ToggleGrid)
    );
    assert_eq!(
        press(&context, ctrl_shift, Key::Tab),
        Some(CommandId::PreviousTab)
    );
}

#[test]
fn ctrl_s_should_never_also_trigger_a_letter_tool() {
    let mut input = input_with(vec![key(Modifiers::COMMAND, Key::S)]);
    assert_eq!(
        commands::take_shortcut_in(&mut input, &canvas_focused()),
        Some(CommandId::Save)
    );
    // The event was consumed once.
    assert_eq!(
        commands::take_shortcut_in(&mut input, &canvas_focused()),
        None
    );

    // Ctrl+C is not the C tool, and Ctrl+V is not the V tool.
    assert_eq!(press(&canvas_focused(), Modifiers::COMMAND, Key::C), None);
    assert_eq!(press(&canvas_focused(), Modifiers::COMMAND, Key::V), None);
}

#[test]
fn canvas_letters_should_fire_only_with_canvas_focus_and_no_typing() {
    let canvas_keys: Vec<_> = CATALOG
        .iter()
        .filter(|spec| spec.scope == ShortcutScope::Canvas)
        .collect();
    assert!(
        canvas_keys
            .iter()
            .any(|spec| spec.id == CommandId::SelectTool)
    );

    let blocked = [
        ("typing", typing()),
        ("IME composing", ime_composing()),
        ("modal open", modal_open()),
        ("inspector focused", in_region(Region::Inspector)),
        ("navigator focused", in_region(Region::Navigator)),
    ];
    for (why, context) in blocked {
        for spec in &canvas_keys {
            for shortcut in spec.all_shortcuts() {
                let mut input = input_with(vec![key(shortcut.modifiers, shortcut.logical_key)]);
                assert_eq!(
                    commands::take_shortcut_in(&mut input, &context),
                    None,
                    "{:?} fired while {why}",
                    spec.id
                );
                // The key is left for the focused control.
                assert!(input.key_pressed(shortcut.logical_key), "{:?}", spec.id);
            }
        }
    }
    for letter in [Key::V, Key::H, Key::C, Key::R, Key::G] {
        assert!(
            press(&canvas_focused(), Modifiers::NONE, letter).is_some(),
            "{letter:?}"
        );
    }
}

#[test]
fn text_fields_should_keep_their_editing_keys() {
    let context = typing();
    for (modifiers, pressed) in [
        (Modifiers::COMMAND, Key::A),
        (Modifiers::COMMAND, Key::F),
        (Modifiers::COMMAND, Key::Z),
        (Modifiers::COMMAND | Modifiers::SHIFT, Key::Z),
        (Modifiers::NONE, Key::Escape),
        (Modifiers::NONE, Key::Delete),
    ] {
        assert_eq!(press(&context, modifiers, pressed), None, "{pressed:?}");
    }
    // Keys a text field does not use still work while typing.
    assert_eq!(
        press(&context, Modifiers::COMMAND, Key::K),
        Some(CommandId::SearchCommands)
    );
    assert_eq!(
        press(&context, Modifiers::COMMAND, Key::S),
        Some(CommandId::Save)
    );
}

#[test]
fn a_modal_should_own_every_key() {
    for spec in &CATALOG {
        for shortcut in spec.all_shortcuts() {
            assert_eq!(
                press(&modal_open(), shortcut.modifiers, shortcut.logical_key),
                None,
                "{:?}",
                spec.id
            );
        }
    }
}

#[test]
fn legacy_shell_path_should_leave_ctrl_a_and_delete_to_text_fields() {
    // A field focused in this very frame has text_had_focus == false.
    for (modifiers, pressed) in [(Modifiers::COMMAND, Key::A), (Modifiers::NONE, Key::Delete)] {
        let mut input = input_with(vec![key(modifiers, pressed)]);
        assert_eq!(commands::take_shortcut(&mut input, false), None);
        assert!(input.key_pressed(pressed));
    }
    let mut input = input_with(vec![key(Modifiers::NONE, Key::V)]);
    assert_eq!(
        commands::take_shortcut(&mut input, false),
        Some(CommandId::SelectTool)
    );
    let mut input = input_with(vec![key(Modifiers::NONE, Key::V)]);
    assert_eq!(commands::take_shortcut(&mut input, true), None);
}

#[test]
fn legacy_shell_path_should_not_fire_letters_during_ime_composition() {
    let mut input = input_with(vec![
        egui::Event::Ime(egui::ImeEvent::Preedit {
            text: String::from("ك"),
            active_range_chars: None,
        }),
        key(Modifiers::NONE, Key::C),
    ]);
    assert!(commands::ime_composing(&input));
    assert_eq!(commands::take_shortcut(&mut input, false), None);
}

#[test]
fn every_shortcut_should_have_readable_text() {
    for spec in &CATALOG {
        if let Some(shortcut) = spec.shortcut {
            let text = commands::shortcut_text(&shortcut);
            assert!(!text.is_empty(), "{:?}", spec.id);
        }
    }
    let redo = CommandId::Redo
        .spec()
        .shortcut
        .map(|s| commands::shortcut_text(&s));
    assert!(redo.is_some_and(|text| text.contains("Shift") && text.contains('Z')));
}
