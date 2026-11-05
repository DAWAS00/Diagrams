//! H4: every route to a command (button, menu item, shortcut, command search)
//! leaves the same `AppState`, and a disabled command changes nothing.
//! The tests loop over the catalog, so new commands are covered automatically.

mod common_state;

use common_state::{apply_all, apply_with, canvas_focused, input_with, key, sample};
use diagram_desktop::app_state::AppState;
use diagram_desktop::intent::{Surface, UiIntent};
use diagram_desktop::presentation;
use diagram_desktop::ui::commands::{self, CommandId};
use diagram_desktop::ui::palette::{self, PaletteEntry};

/// States each command is routed from.
fn starting_states() -> Vec<(&'static str, AppState)> {
    let mut selected = sample();
    selected.select(1);
    let mut empty = sample();
    empty.classes.clear();
    vec![
        ("nothing selected", sample()),
        ("Order selected", selected),
        ("empty diagram", empty),
    ]
}

/// A button or menu item: both push the same intent.
fn by_control(start: &AppState, id: CommandId) -> AppState {
    let mut state = start.clone();
    apply_all(&mut state, [UiIntent::Command(id)]);
    state
}

/// The shortcut, through the real key-dispatch function.
fn by_shortcut(start: &AppState, id: CommandId) -> Option<AppState> {
    let shortcut = id.spec().shortcut?;
    let mut input = input_with(vec![key(shortcut.modifiers, shortcut.logical_key)]);
    let taken = commands::take_shortcut_in(&mut input, &canvas_focused())?;
    let mut state = start.clone();
    apply_with(&mut state, &canvas_focused(), [UiIntent::Command(taken)]);
    Some(state)
}

/// Command search: open it, find the command by its label, activate it.
fn by_search(start: &AppState, id: CommandId) -> Option<AppState> {
    let mut state = start.clone();
    apply_all(&mut state, [UiIntent::Command(CommandId::SearchCommands)]);
    state.palette.as_ref()?;
    let names: Vec<&str> = start
        .classes
        .iter()
        .map(|class| class.display_name())
        .collect();
    let found = palette::entries(id.spec().label, &names)
        .into_iter()
        .any(|entry| entry == PaletteEntry::Command(id));
    if !found {
        return None;
    }
    apply_all(
        &mut state,
        [
            UiIntent::Command(id),
            UiIntent::Close(Surface::CommandSearch),
        ],
    );
    Some(state)
}

#[test]
fn every_command_should_be_found_by_command_search() {
    for id in CommandId::ALL {
        if id == CommandId::SearchCommands {
            continue;
        }
        let results = palette::entries(id.spec().label, &[]);
        assert!(
            results.contains(&PaletteEntry::Command(id)),
            "{id:?} is not found by its label"
        );
    }
}

#[test]
fn shortcut_should_leave_the_same_state_as_the_control() {
    for (name, start) in starting_states() {
        for id in CommandId::ALL {
            if let Some(state) = by_shortcut(&start, id) {
                assert_eq!(state, by_control(&start, id), "{id:?} from {name}");
            }
        }
    }
}

#[test]
fn search_activation_should_leave_the_same_state_as_the_control() {
    for (name, start) in starting_states() {
        for id in CommandId::ALL {
            // Opening search from search is not offered as a result.
            if id == CommandId::SearchCommands {
                continue;
            }
            let Some(state) = by_search(&start, id) else {
                continue;
            };
            assert_eq!(state, by_control(&start, id), "{id:?} from {name}");
        }
    }
}

#[test]
fn disabled_commands_should_leave_the_state_unchanged_from_every_route() {
    for (name, start) in starting_states() {
        for id in CommandId::ALL {
            if commands::availability(id, start.command_context()).is_enabled() {
                continue;
            }
            assert_eq!(by_control(&start, id), start, "{id:?} from {name}");
            if let Some(state) = by_shortcut(&start, id) {
                assert_eq!(state, start, "{id:?} shortcut from {name}");
            }
        }
    }
}

#[test]
fn presentation_availability_should_equal_catalog_availability() {
    for (name, start) in starting_states() {
        let view = presentation::build(&start);
        let context = start.command_context();
        for group in &view.header.menu {
            for command in &group.commands {
                assert_eq!(
                    command.availability,
                    commands::availability(command.id, context),
                    "{:?} from {name}",
                    command.id
                );
            }
        }
        let listed = view
            .header
            .menu
            .iter()
            .flat_map(|group| &group.commands)
            .count();
        assert_eq!(listed, CommandId::ALL.len(), "{name}");
    }
}
