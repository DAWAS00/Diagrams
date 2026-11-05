//! Editor state through `dispatch::apply`: tools and gestures, viewport,
//! selection, drafts, Escape layering, region order, preferences, tabs and
//! the split searches. None of it may change the document by accident.

mod common_state;

use common_state::{apply_all, sample};
use diagram_canvas::{MAX_ZOOM, MIN_ZOOM};
use diagram_core::Point;
use diagram_desktop::app_state::{AppState, BLANK_NAME_WARNING};
use diagram_desktop::dispatch;
use diagram_desktop::editor_state::{Gesture, ShellSupport, Tool};
use diagram_desktop::intent::{
    CanvasPointer, DraftField, PointerKind, Region, ResolveChoice, SearchScope, Surface, Target,
    UiIntent,
};
use diagram_desktop::presentation::{self, DialogView, InspectorKind, NO_MATCHES, NO_OBJECTS};
use diagram_desktop::ui::commands::{self, CommandId};
use eframe::egui::{Modifiers, Pos2};

fn pointer(kind: PointerKind, x: f32, y: f32, shift: bool) -> UiIntent {
    UiIntent::Pointer(CanvasPointer {
        kind,
        position: Pos2::new(x, y),
        modifiers: if shift {
            Modifiers::SHIFT
        } else {
            Modifiers::NONE
        },
        hit: None,
    })
}

fn drag(from: (f32, f32), to: (f32, f32), shift: bool) -> [UiIntent; 3] {
    [
        pointer(PointerKind::Down, from.0, from.1, shift),
        pointer(PointerKind::Move, to.0, to.1, shift),
        pointer(PointerKind::Up, to.0, to.1, shift),
    ]
}

fn command(id: CommandId) -> UiIntent {
    UiIntent::Command(id)
}

fn select(index: usize) -> UiIntent {
    UiIntent::Select {
        target: Target::Class(index),
        toggle: false,
    }
}

fn rename(text: &str) -> UiIntent {
    UiIntent::DraftChanged {
        field: DraftField::ClassName,
        text: text.to_owned(),
    }
}

/// The sample with the new screens' support switched on.
fn new_screens() -> AppState {
    let mut state = sample();
    state.support = ShellSupport::NEW_SCREENS;
    state
}

fn enabled(state: &AppState, id: CommandId) -> bool {
    commands::availability(id, state.command_context()).is_enabled()
}

fn world_under(state: &AppState, screen: (f64, f64)) -> Option<Point> {
    let screen = Point::new(screen.0, screen.1).ok()?;
    state.editor.viewport().to_world(screen).ok()
}

// --- Tools and gestures ---------------------------------------------------

#[test]
fn switching_tool_should_cancel_a_geometric_preview() {
    let mut state = sample();
    state.editor.gesture = Gesture::Placement { at: Point::ORIGIN };
    state.editor.set_tool(Tool::Hand);
    assert_eq!(state.editor.gesture, Gesture::Idle);

    state.editor.gesture = Gesture::MovePreview {
        start: Point::ORIGIN,
        current: Point::ORIGIN,
    };
    apply_all(&mut state, [command(CommandId::SelectTool)]);
    assert_eq!(state.editor.gesture, Gesture::Idle);
    assert_eq!(state.editor.tool, Tool::Select);
}

#[test]
fn hand_drag_should_pan_and_never_move_a_node() {
    let mut state = sample();
    state.editor.tool = Tool::Hand;
    let classes = state.classes.clone();

    // The drag starts on Customer, which is at (48, 48).
    apply_all(&mut state, drag((60.0, 60.0), (100.0, 90.0), false));

    assert_eq!(state.classes, classes);
    assert_eq!(
        state.editor.viewport().pan(),
        Point::new(40.0, 30.0).unwrap_or_default()
    );
    assert!(state.selection.is_empty());
    assert_eq!(state.editor.gesture, Gesture::Idle);
}

#[test]
fn space_drag_should_pan_temporarily_and_keep_the_tool() {
    let mut state = sample();
    apply_all(&mut state, [UiIntent::TemporaryPan(true)]);
    apply_all(&mut state, drag((300.0, 300.0), (290.0, 280.0), false));
    apply_all(&mut state, [UiIntent::TemporaryPan(false)]);

    assert_eq!(state.editor.tool, Tool::Select);
    assert_eq!(
        state.editor.viewport().pan(),
        Point::new(-10.0, -20.0).unwrap_or_default()
    );
    assert!(state.selection.is_empty(), "no marquee while panning");
}

#[test]
fn dragging_a_node_should_preview_only_and_change_nothing() {
    let mut state = sample();
    let classes = state.classes.clone();
    apply_all(&mut state, [pointer(PointerKind::Down, 60.0, 60.0, false)]);
    assert!(matches!(state.editor.gesture, Gesture::MovePreview { .. }));
    assert_eq!(state.selected, Some(0), "pressing on a node selects it");

    apply_all(
        &mut state,
        [
            pointer(PointerKind::Move, 160.0, 160.0, false),
            pointer(PointerKind::Cancel, 160.0, 160.0, false),
        ],
    );
    assert_eq!(state.editor.gesture, Gesture::Idle);
    assert_eq!(state.classes, classes);
}

// --- Viewport --------------------------------------------------------------

#[test]
fn zoom_should_clamp_to_10_and_400_percent_with_reasons() {
    let mut state = sample();
    for _ in 0..30 {
        apply_all(&mut state, [command(CommandId::ZoomIn)]);
    }
    assert_eq!(state.editor.viewport().zoom(), MAX_ZOOM);
    assert!(!enabled(&state, CommandId::ZoomIn));
    let reason = commands::availability(CommandId::ZoomIn, state.command_context()).reason();
    assert!(reason.is_some_and(|reason| reason.contains("400%")));

    for _ in 0..40 {
        apply_all(&mut state, [command(CommandId::ZoomOut)]);
    }
    assert_eq!(state.editor.viewport().zoom(), MIN_ZOOM);
    assert!(!enabled(&state, CommandId::ZoomOut));

    apply_all(&mut state, [command(CommandId::ActualSize)]);
    assert_eq!(presentation::build(&state).status.zoom_text, "Zoom 100%");
}

#[test]
fn keyboard_zoom_should_keep_the_canvas_center_in_place() {
    let mut state = sample();
    let center = (
        state.editor.canvas_size.width / 2.0,
        state.editor.canvas_size.height / 2.0,
    );
    let before = world_under(&state, center);
    apply_all(&mut state, [command(CommandId::ZoomIn)]);
    let after = world_under(&state, center);
    let moved = before
        .zip(after)
        .map(|(b, a)| (b.x() - a.x()).abs() + (b.y() - a.y()).abs());
    assert!(moved.is_some_and(|moved| moved < 1e-9), "{moved:?}");
    assert_eq!(presentation::build(&state).status.zoom_text, "Zoom 125%");
}

#[test]
fn wheel_zoom_should_anchor_at_the_cursor() {
    let mut state = sample();
    let cursor = (123.0, 77.0);
    let before = world_under(&state, cursor);
    apply_all(
        &mut state,
        [pointer(
            PointerKind::Zoom { delta: 1.5 },
            123.0,
            77.0,
            false,
        )],
    );
    let after = world_under(&state, cursor);
    let moved = before
        .zip(after)
        .map(|(b, a)| (b.x() - a.x()).abs() + (b.y() - a.y()).abs());
    assert!(moved.is_some_and(|moved| moved < 1e-4), "{moved:?}");
    assert_eq!(state.editor.viewport().zoom_percent(), 150);

    // A nonsense wheel delta changes nothing.
    let viewport = state.editor.viewport();
    apply_all(
        &mut state,
        [pointer(
            PointerKind::Zoom { delta: f32::NAN },
            0.0,
            0.0,
            false,
        )],
    );
    assert_eq!(state.editor.viewport(), viewport);
}

#[test]
fn fit_all_should_show_every_node_and_never_move_geometry() {
    let mut state = sample();
    apply_all(
        &mut state,
        [UiIntent::CanvasResized {
            width: 900.0,
            height: 500.0,
        }],
    );
    let classes = state.classes.clone();
    apply_all(&mut state, [command(CommandId::FitAll)]);

    assert_eq!(state.classes, classes);
    for node in presentation::build(&state).canvas.nodes {
        let right = node.position.x + node.size.x;
        let bottom = node.position.y + node.size.y;
        assert!(node.position.x >= 0.0 && node.position.y >= 0.0, "{node:?}");
        assert!(right <= 900.0 && bottom <= 500.0, "{node:?}");
    }
    // Fitted at more than 100%: the boxes are drawn larger.
    assert!(state.editor.viewport().zoom() > 1.0);
}

#[test]
fn fit_selection_should_need_a_selection() {
    let mut state = sample();
    assert!(!enabled(&state, CommandId::FitSelection));
    let before = state.editor.viewport();
    apply_all(&mut state, [command(CommandId::FitSelection)]);
    assert_eq!(state.editor.viewport(), before);

    apply_all(&mut state, [select(1), command(CommandId::FitSelection)]);
    assert_ne!(state.editor.viewport(), before);
}

#[test]
fn reveal_should_center_the_selection_without_zooming() {
    let mut state = sample();
    apply_all(&mut state, [select(1), command(CommandId::Reveal)]);
    let view = presentation::build(&state);
    let node = &view.canvas.nodes[1];
    let center_x = f64::from(node.position.x + node.size.x / 2.0);
    assert!((center_x - state.editor.canvas_size.width / 2.0).abs() < 0.01);
    assert_eq!(state.editor.viewport().zoom(), 1.0);
}

#[test]
fn canvas_size_should_ignore_invalid_reports() {
    let mut state = sample();
    let size = state.editor.canvas_size;
    apply_all(
        &mut state,
        [UiIntent::CanvasResized {
            width: f32::NAN,
            height: 10.0,
        }],
    );
    apply_all(
        &mut state,
        [UiIntent::CanvasResized {
            width: 0.0,
            height: 10.0,
        }],
    );
    assert_eq!(state.editor.canvas_size, size);
}

// --- Selection ---------------------------------------------------------------

#[test]
fn click_shift_click_and_select_all_should_build_the_selection() {
    let mut state = sample();
    apply_all(&mut state, [select(0)]);
    assert_eq!(state.selection, [0]);
    apply_all(
        &mut state,
        [UiIntent::Select {
            target: Target::Class(1),
            toggle: true,
        }],
    );
    assert_eq!(state.selection, [0, 1]);
    assert_eq!(
        presentation::build(&state).status.selection_text,
        "2 selected"
    );
    apply_all(
        &mut state,
        [UiIntent::Select {
            target: Target::Class(0),
            toggle: true,
        }],
    );
    assert_eq!(state.selection, [1]);
    assert_eq!(state.selected, Some(1));

    apply_all(&mut state, [command(CommandId::ClearSelection)]);
    apply_all(&mut state, [command(CommandId::SelectAll)]);
    assert_eq!(state.selection, [0, 1]);
}

#[test]
fn marquee_should_select_fully_enclosed_nodes_and_click_should_clear() {
    let mut state = sample();
    // Customer spans (48, 48)-(228, 128); Order starts at x 300.
    apply_all(&mut state, drag((20.0, 20.0), (250.0, 200.0), false));
    assert_eq!(state.selection, [0]);

    // Shift adds.
    apply_all(&mut state, drag((280.0, 80.0), (500.0, 200.0), true));
    assert_eq!(state.selection, [0, 1]);

    // A click on empty canvas clears; a Shift-click on it keeps the selection.
    apply_all(&mut state, drag((600.0, 400.0), (600.0, 400.0), true));
    assert_eq!(state.selection, [0, 1]);
    apply_all(&mut state, drag((600.0, 400.0), (600.0, 400.0), false));
    assert!(state.selection.is_empty());
}

#[test]
fn marquee_should_be_presented_while_dragging() {
    let mut state = sample();
    apply_all(
        &mut state,
        [
            pointer(PointerKind::Down, 10.0, 10.0, false),
            pointer(PointerKind::Move, 50.0, 40.0, false),
        ],
    );
    assert_eq!(
        presentation::build(&state).canvas.marquee,
        Some((Pos2::new(10.0, 10.0), Pos2::new(50.0, 40.0)))
    );
}

// --- Drafts ------------------------------------------------------------------

#[test]
fn a_draft_should_change_the_model_only_when_committed() {
    let mut state = new_screens();
    apply_all(&mut state, [select(0), rename("Client")]);
    assert_eq!(
        state.classes[0].name, "Customer",
        "typing edits the draft only"
    );
    let view = presentation::build(&state);
    assert!(matches!(
        &view.inspector.kind,
        InspectorKind::Class { name, .. } if name.text == "Client"
    ));
    assert_eq!(view.canvas.nodes[0].name, "Customer");

    apply_all(&mut state, [UiIntent::CommitDraft(DraftField::ClassName)]);
    assert_eq!(state.classes[0].name, "Client");
    assert!(state.editor.draft.is_none());
}

#[test]
fn cancel_should_restore_the_original_and_keep_the_selection() {
    let mut state = new_screens();
    apply_all(
        &mut state,
        [
            select(0),
            rename("Client"),
            UiIntent::CancelDraft(DraftField::ClassName),
        ],
    );
    assert_eq!(state.classes[0].name, "Customer");
    assert!(state.editor.draft.is_none());
    assert_eq!(state.selected, Some(0));
}

#[test]
fn an_invalid_draft_should_stay_and_explain_itself() {
    let mut state = new_screens();
    apply_all(
        &mut state,
        [
            select(0),
            rename("  "),
            UiIntent::CommitDraft(DraftField::ClassName),
        ],
    );
    assert_eq!(state.classes[0].name, "Customer");
    let draft = state.editor.draft.as_ref();
    assert!(draft.is_some_and(|draft| draft.text == "  " && draft.commit_refused));
    let view = presentation::build(&state);
    assert!(matches!(
        &view.inspector.kind,
        InspectorKind::Class { name, .. }
            if name.problem.as_deref().is_some_and(|problem| problem.contains(BLANK_NAME_WARNING))
    ));
}

#[test]
fn leaving_with_a_valid_draft_should_commit_it() {
    let mut state = new_screens();
    apply_all(&mut state, [select(0), rename("Client"), select(1)]);
    assert_eq!(state.classes[0].name, "Client");
    assert_eq!(state.selected, Some(1));
}

#[test]
fn leaving_with_an_invalid_draft_should_ask_to_resolve_it() {
    let mut state = new_screens();
    apply_all(&mut state, [select(0), rename(""), select(1)]);

    // Held back: still on Customer, with the question shown.
    assert_eq!(state.selected, Some(0));
    let view = presentation::build(&state);
    assert!(matches!(view.dialog, Some(DialogView::ResolveEdit { .. })));

    // Return to field: nothing else happens.
    apply_all(
        &mut state,
        [UiIntent::ResolveEdit(ResolveChoice::ReturnToField)],
    );
    assert_eq!(state.selected, Some(0));
    assert!(state.editor.draft.is_some());
    assert_eq!(state.editor.focus_region, Region::Inspector);
    assert!(presentation::build(&state).dialog.is_none());

    // Discard: the original comes back and the held selection happens.
    apply_all(
        &mut state,
        [select(1), UiIntent::ResolveEdit(ResolveChoice::DiscardEdit)],
    );
    assert_eq!(state.classes[0].name, "Customer");
    assert!(state.editor.draft.is_none());
    assert_eq!(state.selected, Some(1));
}

#[test]
fn an_invalid_draft_should_also_hold_back_tab_and_tool_changes() {
    let mut state = AppState::with_second_diagram();
    state.support = ShellSupport::NEW_SCREENS;
    apply_all(
        &mut state,
        [select(0), rename(" "), UiIntent::ActivateTab(1)],
    );
    assert_eq!(state.editor.active_tab, 0);
    assert_eq!(
        state.editor.pending.as_deref(),
        Some(&UiIntent::ActivateTab(1))
    );

    apply_all(
        &mut state,
        [UiIntent::ResolveEdit(ResolveChoice::DiscardEdit)],
    );
    assert_eq!(state.editor.active_tab, 1);
}

#[test]
fn todays_shell_should_commit_valid_typing_and_hold_invalid_typing() {
    // Today's inspector sends no Enter or blur, so valid names commit as typed.
    let mut state = sample();
    apply_all(&mut state, [select(0), rename("Client")]);
    assert_eq!(state.classes[0].name, "Client");
    assert!(state.editor.draft.is_none());

    // A blank name is not committed; selection changes wait; Escape restores.
    apply_all(&mut state, [rename(""), select(1)]);
    assert_eq!(state.classes[0].name, "Client");
    assert_eq!(state.selected, Some(0));
    assert!(
        state.editor.pending.is_none(),
        "no dialog this shell cannot draw"
    );
    let view = presentation::build(&state);
    assert!(matches!(
        &view.inspector.kind,
        InspectorKind::Class { name, .. }
            if name.problem.as_deref().is_some_and(|p| p.contains("press Escape"))
    ));
    apply_all(&mut state, [command(CommandId::Cancel)]);
    assert_eq!(state.classes[0].name, "Client");
    assert!(state.editor.draft.is_none());
    apply_all(&mut state, [select(1)]);
    assert_eq!(state.selected, Some(1));
}

// --- Escape layering -------------------------------------------------------------

#[test]
fn escape_should_peel_one_layer_per_press() {
    let mut state = new_screens();
    apply_all(&mut state, [select(0), rename(""), select(1)]);
    state.editor.tool = Tool::Hand;
    assert!(state.editor.pending.is_some());

    // 1: the Resolve edit question (same as Return to field).
    apply_all(&mut state, [command(CommandId::Cancel)]);
    assert!(state.editor.pending.is_none());
    assert!(state.editor.draft.is_some());
    // 2: the draft.
    apply_all(&mut state, [command(CommandId::Cancel)]);
    assert!(state.editor.draft.is_none());
    assert_eq!(state.classes[0].name, "Customer");
    // 3: the gesture.
    state.editor.gesture = Gesture::Pan {
        last: Point::ORIGIN,
    };
    apply_all(&mut state, [command(CommandId::Cancel)]);
    assert_eq!(state.editor.gesture, Gesture::Idle);
    assert_eq!(state.editor.tool, Tool::Hand);
    // 4: the tool.
    apply_all(&mut state, [command(CommandId::Cancel)]);
    assert_eq!(state.editor.tool, Tool::Select);
    assert_eq!(state.selected, Some(0));
    // 5: the selection.
    apply_all(&mut state, [command(CommandId::Cancel)]);
    assert_eq!(state.selected, None);
    // Then nothing is left.
    assert!(!enabled(&state, CommandId::Cancel));
}

// --- Regions, preferences, tabs and searches -----------------------------------

#[test]
fn f6_should_cycle_visible_regions_and_skip_hidden_ones() {
    let mut state = new_screens();
    let mut order = Vec::new();
    for _ in 0..5 {
        apply_all(&mut state, [command(CommandId::NextRegion)]);
        order.push(state.editor.focus_region);
    }
    assert_eq!(
        order,
        [
            Region::Inspector,
            Region::Status,
            Region::Header,
            Region::Navigator,
            Region::Board
        ]
    );

    apply_all(&mut state, [command(CommandId::ToggleNavigator)]);
    let regions = dispatch::visible_regions(&state);
    assert!(!regions.contains(&Region::Navigator));
    // From the board, the hidden project panel is skipped.
    apply_all(&mut state, [command(CommandId::PreviousRegion)]);
    assert_eq!(state.editor.focus_region, Region::Header);
    assert_eq!(presentation::build(&state).focus_region, Region::Header);
}

#[test]
fn region_and_grid_commands_should_explain_why_todays_shell_cannot_use_them() {
    let state = sample();
    for id in [
        CommandId::NextRegion,
        CommandId::ToggleGrid,
        CommandId::ToggleSnap,
        CommandId::FindObject,
    ] {
        let reason = commands::availability(id, state.command_context()).reason();
        assert!(
            reason.is_some_and(|reason| reason.starts_with("Not available yet")),
            "{id:?}"
        );
    }
}

#[test]
fn preferences_should_never_touch_the_document() {
    let mut state = new_screens();
    let classes = state.classes.clone();
    let project = state.project_name.clone();
    apply_all(
        &mut state,
        [
            command(CommandId::ToggleGrid),
            command(CommandId::ToggleSnap),
            UiIntent::SetTheme(eframe::egui::ThemePreference::Dark),
            command(CommandId::ZoomIn),
            command(CommandId::FitAll),
        ],
    );
    assert!(state.editor.preferences.grid && state.editor.preferences.snap);
    assert_eq!(state.classes, classes);
    assert_eq!(state.project_name, project);
    let view = presentation::build(&state);
    assert!(view.status.grid && view.status.snap && view.canvas.grid);
}

#[test]
fn tabs_should_keep_their_own_viewport_and_selection() {
    let mut state = AppState::with_second_diagram();
    assert!(enabled(&state, CommandId::NextTab));
    apply_all(&mut state, [select(0), command(CommandId::ZoomIn)]);
    let first_viewport = state.editor.viewport();

    apply_all(&mut state, [command(CommandId::NextTab)]);
    assert_eq!(state.editor.active_tab, 1);
    assert!(state.selection.is_empty());
    assert_eq!(state.editor.viewport().zoom(), 1.0);

    apply_all(&mut state, [command(CommandId::PreviousTab)]);
    assert_eq!(state.editor.viewport(), first_viewport);
    assert_eq!(state.selection, [0]);
    let tabs = presentation::build(&state).tabs.tabs;
    assert!(tabs[0].active && !tabs[1].active);
}

#[test]
fn closing_a_tab_should_never_delete_a_diagram() {
    let mut state = AppState::with_second_diagram();
    let diagrams = state.diagrams.clone();
    let classes = state.classes.clone();
    apply_all(
        &mut state,
        [UiIntent::ActivateTab(1), command(CommandId::CloseTab)],
    );
    assert_eq!(state.editor.tabs.len(), 1);
    assert_eq!(state.diagrams, diagrams);
    assert_eq!(state.classes, classes);
    assert_eq!(presentation::build(&state).navigator.diagrams.len(), 2);
    // The last tab stays, with a reason.
    assert!(!enabled(&state, CommandId::CloseTab));
    assert!(!enabled(&state, CommandId::NextTab));
}

#[test]
fn object_search_and_command_search_should_have_separate_results() {
    let mut state = new_screens();
    apply_all(
        &mut state,
        [
            command(CommandId::FindObject),
            UiIntent::Search {
                scope: SearchScope::Objects,
                query: String::from("ord"),
            },
        ],
    );
    let view = presentation::build(&state);
    assert!(view.object_search.open);
    let labels: Vec<_> = view
        .object_search
        .results
        .iter()
        .map(|r| &r.label)
        .collect();
    assert_eq!(labels, ["Class: Order"]);
    assert_eq!(view.object_search.results[0].location, "Class diagram 1");
    assert_eq!(view.focus_region, Region::Navigator);
    assert!(!view.command_search.open);

    apply_all(
        &mut state,
        [UiIntent::Search {
            scope: SearchScope::Objects,
            query: String::from("zzz"),
        }],
    );
    assert_eq!(
        presentation::build(&state).object_search.empty_text,
        Some(NO_MATCHES)
    );

    // Command search lists commands only, never objects or itself.
    apply_all(
        &mut state,
        [
            UiIntent::Close(Surface::ObjectSearch),
            command(CommandId::SearchCommands),
            UiIntent::Search {
                scope: SearchScope::Commands,
                query: String::from("fit"),
            },
        ],
    );
    let view = presentation::build(&state);
    assert!(!view.object_search.open);
    let ids: Vec<_> = view.command_search.results.iter().map(|c| c.id).collect();
    assert!(ids.contains(&CommandId::FitAll) && ids.contains(&CommandId::FitSelection));
    assert!(!ids.contains(&CommandId::SearchCommands));
}

#[test]
fn object_search_should_say_when_the_project_is_empty() {
    let mut state = new_screens();
    apply_all(
        &mut state,
        [
            UiIntent::CreateProject {
                name: String::from("Empty"),
            },
            command(CommandId::FindObject),
        ],
    );
    assert_eq!(
        presentation::build(&state).object_search.empty_text,
        Some(NO_OBJECTS)
    );
}

#[test]
fn notation_should_offer_only_the_class_diagram() {
    let view = presentation::build(&sample());
    assert_eq!(view.diagram_types, ["Class diagram"]);
}
