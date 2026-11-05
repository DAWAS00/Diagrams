//! The contract between the views and the functionality stays complete:
//! every command is fully described, every disabled reason is words, and
//! every intent variant is handled by `dispatch::apply`.

mod common_state;

use common_state::apply_all;
use diagram_desktop::app_state::AppState;
use diagram_desktop::intent::{
    CanvasPointer, DraftField, PointerKind, Region, ResolveChoice, SearchScope, Surface, Target,
    UiIntent,
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
        UiIntent::ActivateTab(1),
        UiIntent::TemporaryPan(true),
        UiIntent::CanvasResized {
            width: 900.0,
            height: 600.0,
        },
        UiIntent::ResolveEdit(ResolveChoice::DiscardEdit),
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
            | UiIntent::CreateProject { .. }
            | UiIntent::ActivateTab(_)
            | UiIntent::TemporaryPan(_)
            | UiIntent::CanvasResized { .. }
            | UiIntent::ResolveEdit(_) => {}
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
        let context = CommandContext {
            has_selection,
            ..CommandContext::default()
        };
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

/// The catalog command that carries each action of the function map.
fn function_map_command(action: &str) -> Option<CommandId> {
    Some(match action {
        "New project" => CommandId::NewProject,
        "Open project" => CommandId::OpenProject,
        "Add diagram" => CommandId::NewDiagram,
        "Find object" => CommandId::FindObject,
        "Switch diagram" => CommandId::NextTab,
        "Add class" => CommandId::AddClass,
        "Rename class" => CommandId::Rename,
        "Add attribute" => CommandId::AddAttribute,
        "Add operation" => CommandId::AddOperation,
        "Edit/reorder member" => CommandId::MoveMemberUp,
        "Connect association" => CommandId::Connect,
        "Add generalization" => CommandId::RelationshipKind,
        "Edit relation ends" => CommandId::EditRelationEnds,
        "Move/resize views" => CommandId::EditGeometry,
        "Change appearance" => CommandId::EditAppearance,
        "Align selected views" => CommandId::AlignLeft,
        "Show existing model in diagram" => CommandId::ShowInDiagram,
        "Duplicate model" => CommandId::DuplicateModel,
        "Remove view" => CommandId::RemoveView,
        "Delete model everywhere" => CommandId::DeleteModelEverywhere,
        "Undo/Redo" => CommandId::Undo,
        "Pan/zoom/fit" => CommandId::FitAll,
        "Save/Save As" => CommandId::Save,
        "Export SVG" => CommandId::Export,
        "Reveal validation issue" => CommandId::ShowIssues,
        "Close dirty project" => CommandId::CloseProject,
        "Recover project" => CommandId::RecoverProject,
        "Theme/text scale/shortcuts" => CommandId::Preferences,
        _ => return None,
    })
}

#[test]
fn every_function_map_row_should_name_a_catalog_command() {
    let csv = include_str!("../../../docs/ui/new-design/UI-FUNCTION-MAP.csv");
    let mut rows = 0;
    for line in csv.trim_start_matches('\u{feff}').lines().skip(1) {
        let Some(action) = line.split(',').next().filter(|action| !action.is_empty()) else {
            continue;
        };
        rows += 1;
        let command = function_map_command(action);
        assert!(
            command.is_some(),
            "function map row {action:?} has no command"
        );
        if let Some(id) = command {
            // Enabled, or disabled with a reason; either way it is in the
            // catalog and reachable from command search.
            assert!(CommandId::ALL.contains(&id));
            assert!(!commands::search(id.spec().label).is_empty(), "{action}");
        }
    }
    assert_eq!(rows, 28, "the function map changed; update the mapping");
}

#[test]
fn search_aliases_should_reach_one_canonical_label() {
    for (word, expected) in [
        ("attribute", CommandId::AddAttribute),
        ("reuse", CommandId::ShowInDiagram),
        ("fit", CommandId::FitAll),
        ("issues", CommandId::ShowIssues),
        ("hand", CommandId::PanTool),
        ("svg", CommandId::Export),
    ] {
        let results = commands::search(word);
        assert!(
            results.iter().any(|spec| spec.id == expected),
            "{word:?} does not find {expected:?}"
        );
    }
    for spec in &CATALOG {
        for alias in spec.aliases {
            assert_eq!(alias.to_lowercase(), *alias, "{:?}", spec.id);
        }
    }
}

#[test]
fn every_notation_tool_should_be_a_tool_row_command() {
    use diagram_core::notation::{self, Notation, NotationId};
    use diagram_desktop::ui::commands::Placement;

    let class = Notation::get(NotationId::Class);
    for tool in class.tools {
        let id = commands::tool_command(*tool);
        assert_eq!(id.spec().placement, Placement::ToolRow, "{tool:?}");
    }
    // Only delivered notations can be created; planned ones list what they need.
    let creatable: Vec<_> = notation::creatable().map(|n| n.id).collect();
    assert_eq!(creatable, [NotationId::Class]);
    for planned in notation::NOTATIONS.iter().filter(|n| !n.delivered) {
        assert!(planned.tools.is_empty(), "{:?}", planned.id);
        assert!(!planned.open_requirements.is_empty(), "{:?}", planned.id);
    }
    assert!(
        Notation::get(NotationId::Sequence)
            .open_requirements
            .contains("interaction strategy")
    );
}
