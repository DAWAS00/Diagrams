//! The contract between the views and the functionality stays complete:
//! every command is fully described, every disabled reason is words, and
//! every intent variant is handled by `dispatch::apply`.

mod common_state;

use common_state::apply_all;
use diagram_desktop::app_state::AppState;
use diagram_desktop::intent::{
    CanvasPointer, DraftField, PointerKind, Region, SearchScope, Surface, Target, UiIntent,
};
use diagram_desktop::ui::commands::{self, CATALOG, CommandContext, CommandGroup, CommandId};
use eframe::egui::{Modifiers, Pos2, ThemePreference};

/// One example of every `UiIntent` variant. The exhaustive match below makes
/// this test stop compiling when a variant is added without an example.
fn one_of_each_intent() -> Vec<UiIntent> {
    let examples = vec![
        UiIntent::Command(CommandId::SearchCommands),
        UiIntent::Select {
            target: Target::Class(0),
            toggle: true,
        },
        UiIntent::DraftChanged {
            field: DraftField::ClassName,
            text: String::from("Client"),
        },
        UiIntent::CommitDraft(DraftField::ClassName),
        UiIntent::CancelDraft(DraftField::ClassName),
        UiIntent::Pointer(CanvasPointer {
            kind: PointerKind::Down,
            position: Pos2::new(5.0, 5.0),
            modifiers: Modifiers::NONE,
            hit: None,
        }),
        UiIntent::Search {
            scope: SearchScope::Objects,
            query: String::from("ord"),
        },
        UiIntent::FocusRegion(Region::Inspector),
        UiIntent::Close(Surface::Drawer),
        UiIntent::SetTheme(ThemePreference::Dark),
        UiIntent::OpenSampleProject,
        UiIntent::CreateProject {
            name: String::from("Shop"),
        },
    ];
    for example in &examples {
        // No wildcard: a new variant must be listed here.
        match example {
            UiIntent::Command(_)
            | UiIntent::Select { .. }
            | UiIntent::DraftChanged { .. }
            | UiIntent::CommitDraft(_)
            | UiIntent::CancelDraft(_)
            | UiIntent::Pointer(_)
            | UiIntent::Search { .. }
            | UiIntent::FocusRegion(_)
            | UiIntent::Close(_)
            | UiIntent::SetTheme(_)
            | UiIntent::OpenSampleProject
            | UiIntent::CreateProject { .. } => {}
        }
    }
    examples
}

#[test]
fn every_command_should_have_a_label_group_and_placement() {
    for id in CommandId::ALL {
        let spec = id.spec();
        assert_eq!(spec.id, id);
        assert!(!spec.label.trim().is_empty(), "{id:?}");
        assert!(CommandGroup::ALL.contains(&spec.group), "{id:?}");
        let listed_in_menu = commands::menu_groups()
            .iter()
            .any(|(_, specs)| specs.iter().any(|listed| listed.id == id));
        assert!(listed_in_menu, "{id:?} is missing from the menu");
    }
    assert_eq!(CATALOG.len(), CommandId::ALL.len());
}

#[test]
fn every_disabled_reason_should_be_a_sentence_in_words() {
    for has_selection in [false, true] {
        let context = CommandContext { has_selection };
        for id in CommandId::ALL {
            if let Some(reason) = commands::availability(id, context).reason() {
                let first = reason.chars().next();
                assert!(
                    first.is_some_and(char::is_uppercase),
                    "{id:?}: reason must start with a word: {reason:?}"
                );
                assert!(reason.ends_with('.'), "{id:?}: {reason:?}");
                assert!(reason.split_whitespace().count() >= 3, "{id:?}: {reason:?}");
            }
        }
    }
}

#[test]
fn every_intent_variant_should_be_handled_without_panicking() {
    for start in [AppState::default(), AppState::welcome()] {
        for intent in one_of_each_intent() {
            let mut state = start.clone();
            apply_all(&mut state, [intent]);
        }
        let mut state = start.clone();
        apply_all(&mut state, one_of_each_intent());
    }
}

#[test]
fn dispatch_should_have_no_placeholder_or_catch_all_arm() {
    let source = include_str!("../src/dispatch.rs");
    for forbidden in ["todo!(", "unimplemented!(", "unreachable!("] {
        assert!(!source.contains(forbidden), "dispatch.rs uses {forbidden}");
    }
    let body = source
        .split("fn apply_one")
        .nth(1)
        .and_then(|rest| rest.split("\nfn ").next())
        .unwrap_or_default();
    assert!(!body.is_empty(), "apply_one not found");
    assert!(
        !body.contains("_ =>"),
        "apply_one must handle every intent variant explicitly"
    );
}

#[test]
fn disabled_commands_should_leave_the_state_unchanged() {
    let start = AppState::default();
    for id in CommandId::ALL {
        if commands::availability(id, start.command_context()).is_enabled() {
            continue;
        }
        let mut state = start.clone();
        apply_all(&mut state, [UiIntent::Command(id)]);
        assert_eq!(state, start, "{id:?}");
    }
}
