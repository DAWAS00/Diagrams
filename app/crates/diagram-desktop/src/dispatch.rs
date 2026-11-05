//! The only place that changes [`AppState`] in response to the person.
//!
//! Every button, menu item, shortcut, search result and pointer gesture
//! arrives here as a [`UiIntent`]. A command that is not available now
//! changes nothing. Actions that would lose an invalid draft are held back
//! until the draft is resolved (functional spec §6).

use diagram_canvas::ScreenSize;
use diagram_canvas::picking;
use diagram_core::Point;

use crate::app_state::{AppState, SAMPLE_PROJECT, sample_classes};
use crate::drafts::{self, Draft};
use crate::editor_state::{EditorState, Gesture, Tool};
use crate::intent::{
    CanvasPointer, DraftField, InputContext, PointerKind, Region, ResolveChoice, SearchScope,
    Surface, Target, UiIntent,
};
use crate::ui::commands::{self, CommandId};
use crate::ui::palette::PaletteState;
use crate::ui::showcase::ShowcaseState;
use crate::ui::workspace::{Drawer, LayoutMode};
use crate::viewport;

/// Applies `intents` in order. `input` carries this frame's UI facts, such as
/// the widget focus returns to when a surface opened here closes.
pub fn apply(
    state: &mut AppState,
    input: &InputContext,
    intents: impl IntoIterator<Item = UiIntent>,
) {
    for intent in intents {
        apply_one(state, input, intent);
    }
}

fn apply_one(state: &mut AppState, input: &InputContext, intent: UiIntent) {
    match intent {
        UiIntent::Command(id) => run(state, input, id),
        UiIntent::Select { target, toggle } => select(state, target, toggle),
        UiIntent::DraftChanged { field, text } => draft_changed(state, field, text),
        UiIntent::CommitDraft(field) => {
            commit_draft(state, field);
        }
        UiIntent::CancelDraft(field) => {
            if state
                .editor
                .draft
                .as_ref()
                .is_some_and(|draft| draft.field == field)
            {
                cancel_draft(state);
            }
        }
        UiIntent::Pointer(pointer) => self::pointer(state, pointer),
        UiIntent::Search { scope, query } => search(state, scope, query),
        UiIntent::FocusRegion(region) => {
            if visible_regions(state).contains(&region) {
                state.editor.focus_region = region;
            }
        }
        UiIntent::Close(surface) => close(state, surface),
        UiIntent::SetTheme(preference) => state.theme = preference,
        UiIntent::OpenSampleProject => {
            reset_project(state, String::from(SAMPLE_PROJECT));
            state.classes = sample_classes();
        }
        UiIntent::CreateProject { name } => {
            reset_project(state, name);
            state.classes.clear();
        }
        UiIntent::ActivateTab(index) => switch_tab(state, index),
        UiIntent::TemporaryPan(held) => {
            state.editor.temporary_pan = held;
            if !held && matches!(state.editor.gesture, Gesture::Pan { .. }) {
                state.editor.gesture = Gesture::Idle;
            }
        }
        UiIntent::CanvasResized { width, height } => {
            let (width, height) = (f64::from(width), f64::from(height));
            if width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0 {
                state.editor.canvas_size = ScreenSize { width, height };
            }
        }
        UiIntent::ResolveEdit(choice) => resolve_edit(state, input, choice),
    }
}

/// Replaces the open project; editor state starts fresh.
fn reset_project(state: &mut AppState, name: String) {
    state.project_name = name;
    state.selected = None;
    state.selection.clear();
    state.entry = None;
    state.editor = EditorState::default();
    state.diagrams.truncate(1);
}

/// Runs `id` if it is available now; unavailable commands do nothing.
fn run(state: &mut AppState, input: &InputContext, id: CommandId) {
    if !commands::availability(id, state.command_context()).is_enabled() {
        return;
    }
    let held = UiIntent::Command(id);
    match id {
        CommandId::ClearSelection => {
            if resolve_draft(state, held) {
                state.clear_selection();
            }
        }
        CommandId::Cancel => cancel(state),
        CommandId::SearchCommands => {
            if state.palette.is_none() {
                state.palette = Some(PaletteState::new(input.opener));
            }
        }
        CommandId::ToggleNavigator => toggle_panel(state, input, Drawer::Navigator),
        CommandId::ToggleInspector => toggle_panel(state, input, Drawer::Inspector),
        CommandId::ShowComponentShowcase => {
            if state.showcase.is_none() {
                state.showcase_return = input.opener;
                state.showcase = Some(ShowcaseState::default());
            }
        }
        CommandId::SelectTool => set_tool(state, held, Tool::Select),
        CommandId::PanTool => set_tool(state, held, Tool::Hand),
        CommandId::SelectAll => {
            if resolve_draft(state, held) {
                state.set_selection((0..state.classes.len()).collect());
            }
        }
        CommandId::NextTab => {
            let count = state.editor.tabs.len();
            switch_tab(state, (state.editor.active_tab + 1) % count.max(1));
        }
        CommandId::PreviousTab => {
            let count = state.editor.tabs.len().max(1);
            switch_tab(state, (state.editor.active_tab + count - 1) % count);
        }
        CommandId::CloseTab => close_tab(state, held),
        CommandId::ZoomIn => viewport::zoom_in(&mut state.editor),
        CommandId::ZoomOut => viewport::zoom_out(&mut state.editor),
        CommandId::ActualSize => viewport::actual_size(&mut state.editor),
        CommandId::FitAll => {
            let content = viewport::bounds(&state.classes, None);
            viewport::fit(&mut state.editor, content);
        }
        CommandId::FitSelection => {
            let selection = state.selected_indices();
            let content = viewport::bounds(&state.classes, Some(&selection));
            if content.is_some() {
                viewport::fit(&mut state.editor, content);
            }
        }
        CommandId::Reveal => {
            let selection = state.selected_indices();
            if let Some(content) = viewport::bounds(&state.classes, Some(&selection)) {
                viewport::center_on(&mut state.editor, content);
            }
        }
        CommandId::ToggleGrid => state.editor.preferences.grid ^= true,
        CommandId::ToggleSnap => state.editor.preferences.snap ^= true,
        CommandId::FindObject => {
            state.editor.object_search.get_or_insert_with(String::new);
            state.editor.focus_region = Region::Navigator;
        }
        CommandId::NextRegion => step_region(state, true),
        CommandId::PreviousRegion => step_region(state, false),
        // Moves focus to the Name field; the presentation asks for it.
        CommandId::Rename => state.editor.focus_region = Region::Inspector,
        // Unavailable until the project model, storage or new screens exist;
        // `availability` gives the reason and they never reach this point.
        CommandId::Undo
        | CommandId::Redo
        | CommandId::Save
        | CommandId::Export
        | CommandId::AddClass
        | CommandId::Connect
        | CommandId::AddNote
        | CommandId::Arrange
        | CommandId::NewDiagram
        | CommandId::NewProject
        | CommandId::OpenProject
        | CommandId::SaveAs
        | CommandId::CloseProject
        | CommandId::RecoverProject
        | CommandId::RenameDiagram
        | CommandId::DeleteDiagram
        | CommandId::RemoveView
        | CommandId::EditGeometry
        | CommandId::EditAppearance
        | CommandId::AddAttribute
        | CommandId::AddOperation
        | CommandId::MoveMemberUp
        | CommandId::MoveMemberDown
        | CommandId::RemoveMember
        | CommandId::EditRelationEnds
        | CommandId::ShowInDiagram
        | CommandId::DuplicateModel
        | CommandId::DeleteModelEverywhere
        | CommandId::RelationshipKind
        | CommandId::AlignLeft
        | CommandId::AlignCenter
        | CommandId::AlignRight
        | CommandId::AlignTop
        | CommandId::AlignMiddle
        | CommandId::AlignBottom
        | CommandId::DistributeHorizontally
        | CommandId::DistributeVertically
        | CommandId::ShowIssues
        | CommandId::Preferences
        | CommandId::ContextMenu
        | CommandId::HelpShortcuts => {}
    }
}

/// Escape: one layer per press — the Resolve edit question, then the draft,
/// then the gesture, then the tool, then the selection.
fn cancel(state: &mut AppState) {
    let editor = &mut state.editor;
    if editor.pending.take().is_some() {
        return;
    }
    if editor.draft.is_some() {
        cancel_draft(state);
    } else if editor.gesture != Gesture::Idle {
        editor.gesture = Gesture::Idle;
    } else if editor.tool != Tool::Select {
        editor.set_tool(Tool::Select);
    } else {
        state.clear_selection();
    }
}

fn set_tool(state: &mut AppState, held: UiIntent, tool: Tool) {
    if state.editor.tool != tool && !resolve_draft(state, held) {
        return;
    }
    state.editor.set_tool(tool);
}

fn select(state: &mut AppState, target: Target, toggle: bool) {
    let Target::Class(index) = target;
    if index >= state.classes.len() {
        return;
    }
    let same_object = state
        .editor
        .draft
        .as_ref()
        .is_some_and(|draft| draft.target == target);
    if !(same_object && !toggle) && !resolve_draft(state, UiIntent::Select { target, toggle }) {
        return;
    }
    if toggle {
        state.toggle_selected(index);
    } else {
        state.select(index);
    }
}

// --- Drafts ---------------------------------------------------------------

fn draft_changed(state: &mut AppState, field: DraftField, text: String) {
    match field {
        DraftField::ClassName => {
            let Some(index) = state.selected else {
                return;
            };
            let Some(class) = state.classes.get_mut(index) else {
                return;
            };
            let target = Target::Class(index);
            let draft = match &mut state.editor.draft {
                Some(draft) if draft.field == field && draft.target == target => draft,
                slot => slot.insert(Draft::new(field, target, class.name.clone())),
            };
            draft.text = text;
            draft.commit_refused = false;
            // A shell without draft events never sends Enter or blur, so a
            // valid name commits as it is typed; an invalid one waits.
            if !state.support.draft_events && draft.is_valid() {
                drafts::commit_class_name(class, &draft.text);
                state.editor.draft = None;
            }
        }
    }
}

/// Commits the draft for `field` if it is valid. Returns whether nothing is
/// left uncommitted.
fn commit_draft(state: &mut AppState, field: DraftField) -> bool {
    let Some(draft) = state.editor.draft.as_mut() else {
        return true;
    };
    if draft.field != field {
        return false;
    }
    if !draft.is_valid() {
        draft.commit_refused = true;
        return false;
    }
    let Target::Class(index) = draft.target;
    if let Some(class) = state.classes.get_mut(index) {
        drafts::commit_class_name(class, &draft.text);
    }
    state.editor.draft = None;
    state.editor.pending = None;
    true
}

/// Restores the original value and drops the draft.
fn cancel_draft(state: &mut AppState) {
    if let Some(draft) = state.editor.draft.take() {
        let Target::Class(index) = draft.target;
        if let Some(class) = state.classes.get_mut(index) {
            class.name = draft.original;
        }
    }
    state.editor.pending = None;
}

/// Makes way for `held`: a valid draft commits (as leaving the field would);
/// an invalid one stays and `held` waits for the Resolve edit answer.
/// Returns whether `held` may go ahead now.
fn resolve_draft(state: &mut AppState, held: UiIntent) -> bool {
    let Some(field) = state.editor.draft.as_ref().map(|draft| draft.field) else {
        return true;
    };
    if commit_draft(state, field) {
        return true;
    }
    if state.support.resolve_dialog {
        state.editor.pending = Some(Box::new(held));
    }
    false
}

fn resolve_edit(state: &mut AppState, input: &InputContext, choice: ResolveChoice) {
    let Some(held) = state.editor.pending.take() else {
        return;
    };
    match choice {
        ResolveChoice::ReturnToField => state.editor.focus_region = Region::Inspector,
        ResolveChoice::DiscardEdit => {
            cancel_draft(state);
            apply_one(state, input, *held);
        }
    }
}

// --- Pointer gestures -----------------------------------------------------

fn pointer(state: &mut AppState, event: CanvasPointer) {
    let Ok(position) = Point::new(f64::from(event.position.x), f64::from(event.position.y)) else {
        return;
    };
    match event.kind {
        PointerKind::Down => pointer_down(state, event, position),
        PointerKind::Move => match &mut state.editor.gesture {
            Gesture::Pan { last } => {
                let (dx, dy) = (position.x() - last.x(), position.y() - last.y());
                *last = position;
                viewport::pan_by(&mut state.editor, dx, dy);
            }
            Gesture::Marquee { current, .. }
            | Gesture::MovePreview { current, .. }
            | Gesture::ResizePreview { current, .. } => *current = position,
            Gesture::Idle
            | Gesture::Placement { .. }
            | Gesture::ConnectSource
            | Gesture::ConnectTarget { .. }
            | Gesture::InlineEdit => {}
        },
        PointerKind::Up => pointer_up(state),
        PointerKind::Cancel => {
            // Lost capture: discard the preview; geometry never changed.
            if state.editor.gesture.is_preview() {
                state.editor.gesture = Gesture::Idle;
            }
        }
        PointerKind::Zoom { delta } => {
            if delta.is_finite() && delta > 0.0 {
                viewport::zoom_at(&mut state.editor, delta, position);
            }
        }
    }
}

fn pointer_down(state: &mut AppState, event: CanvasPointer, position: Point) {
    if state.editor.gesture != Gesture::Idle {
        return;
    }
    if state.editor.effective_tool() == Tool::Hand {
        // A drag that starts in Hand mode never moves a node.
        state.editor.gesture = Gesture::Pan { last: position };
        return;
    }
    let hit = event.hit.or_else(|| {
        let rects: Vec<_> = state
            .classes
            .iter()
            .filter_map(viewport::class_rect)
            .collect();
        picking::pick(&rects, state.editor.viewport(), position).map(Target::Class)
    });
    match hit {
        Some(target) => {
            let Target::Class(index) = target;
            if !state.selected_indices().contains(&index) || event.modifiers.shift {
                select(state, target, event.modifiers.shift);
            }
            if state.selected_indices().contains(&index) {
                state.editor.gesture = Gesture::MovePreview {
                    start: position,
                    current: position,
                };
            }
        }
        None => {
            state.editor.gesture = Gesture::Marquee {
                start: position,
                current: position,
                additive: event.modifiers.shift,
            };
        }
    }
}

fn pointer_up(state: &mut AppState) {
    let gesture = std::mem::take(&mut state.editor.gesture);
    match gesture {
        Gesture::Marquee {
            start,
            current,
            additive,
        } => {
            let rects: Vec<_> = state
                .classes
                .iter()
                .filter_map(viewport::class_rect)
                .collect();
            let enclosed = picking::enclosed(&rects, state.editor.viewport(), start, current);
            let moved = start != current;
            let mut selection = if additive {
                state.selected_indices()
            } else {
                Vec::new()
            };
            selection.extend(enclosed);
            // A click on empty canvas clears the selection; Shift keeps it.
            if (moved || !additive)
                && resolve_draft(state, UiIntent::Command(CommandId::ClearSelection))
            {
                state.set_selection(selection);
            }
        }
        // Moving views needs the MoveViews command (R1-03); until then the
        // preview ends with geometry unchanged and nothing recorded.
        Gesture::MovePreview { .. } | Gesture::ResizePreview { .. } | Gesture::Pan { .. } => {}
        other @ (Gesture::Idle
        | Gesture::Placement { .. }
        | Gesture::ConnectSource
        | Gesture::ConnectTarget { .. }
        | Gesture::InlineEdit) => state.editor.gesture = other,
    }
}

// --- Tabs, regions, searches and surfaces ---------------------------------

fn switch_tab(state: &mut AppState, index: usize) {
    if index == state.editor.active_tab || index >= state.editor.tabs.len() {
        return;
    }
    if !resolve_draft(state, UiIntent::ActivateTab(index)) {
        return;
    }
    let selection = state.selected_indices();
    if let Some(tab) = state.editor.tabs.get_mut(state.editor.active_tab) {
        tab.selection = selection;
    }
    state.editor.active_tab = index;
    state.editor.gesture = Gesture::Idle;
    let restored = state
        .editor
        .tabs
        .get(index)
        .map(|tab| tab.selection.clone())
        .unwrap_or_default();
    state.set_selection(restored);
}

/// Closes the active tab only; the diagram stays in the project.
fn close_tab(state: &mut AppState, held: UiIntent) {
    if state.editor.tabs.len() < 2 || !resolve_draft(state, held) {
        return;
    }
    let closing = state.editor.active_tab;
    state.editor.tabs.remove(closing);
    state.editor.active_tab = closing.min(state.editor.tabs.len() - 1);
    state.editor.gesture = Gesture::Idle;
    let restored = state
        .editor
        .tabs
        .get(state.editor.active_tab)
        .map(|tab| tab.selection.clone())
        .unwrap_or_default();
    state.set_selection(restored);
}

/// Regions in F6 order that are shown now.
pub fn visible_regions(state: &AppState) -> Vec<Region> {
    Region::ALL
        .into_iter()
        .filter(|region| match region {
            Region::Navigator => state.panel_shown(Drawer::Navigator),
            Region::Inspector => state.panel_shown(Drawer::Inspector),
            // The validation drawer does not exist yet.
            Region::Drawer => false,
            Region::Header | Region::Board | Region::Status => true,
        })
        .collect()
}

fn step_region(state: &mut AppState, forward: bool) {
    let regions = visible_regions(state);
    let current = regions
        .iter()
        .position(|region| *region == state.editor.focus_region);
    let count = regions.len();
    let next = match (current, forward) {
        (Some(index), true) => (index + 1) % count,
        (Some(index), false) => (index + count - 1) % count,
        (None, _) => regions
            .iter()
            .position(|region| *region == Region::Board)
            .unwrap_or(0),
    };
    if let Some(region) = regions.get(next) {
        state.editor.focus_region = *region;
    }
}

fn search(state: &mut AppState, scope: SearchScope, query: String) {
    match scope {
        SearchScope::Commands => {
            if let Some(search) = state.palette.as_mut()
                && search.query != query
            {
                search.query = query;
                search.highlight_pending = true;
            }
        }
        SearchScope::Objects => {
            if let Some(current) = state.editor.object_search.as_mut() {
                *current = query;
            }
        }
    }
}

/// Shows or hides a docked panel, or opens and closes its drawer on narrow windows.
fn toggle_panel(state: &mut AppState, input: &InputContext, which: Drawer) {
    match state.layout {
        LayoutMode::Docked => match which {
            Drawer::Navigator => state.panels.navigator_visible ^= true,
            Drawer::Inspector => state.panels.inspector_visible ^= true,
        },
        LayoutMode::Narrow if state.drawer == Some(which) => close(state, Surface::Drawer),
        LayoutMode::Narrow => {
            state.drawer_return = input.opener;
            state.drawer = Some(which);
        }
    }
    if !visible_regions(state).contains(&state.editor.focus_region) {
        state.editor.focus_region = Region::Board;
    }
}

/// Closes a surface and returns focus to the widget that opened it.
fn close(state: &mut AppState, surface: Surface) {
    match surface {
        Surface::Drawer => {
            if state.drawer.take().is_some() {
                state.pending_focus = state.drawer_return.take();
            }
        }
        Surface::Showcase => {
            if state.showcase.take().is_some() {
                state.pending_focus = state.showcase_return.take();
            }
        }
        Surface::CommandSearch => {
            if let Some(search) = state.palette.take() {
                state.pending_focus = search.restore_focus;
            }
        }
        Surface::ObjectSearch => state.editor.object_search = None,
    }
}

/// Records the window layout chosen from the window width. A change of
/// layout closes any drawer, without returning focus (its opener may be gone).
pub fn set_layout(state: &mut AppState, layout: LayoutMode) {
    if layout != state.layout {
        state.layout = layout;
        state.drawer = None;
        state.drawer_return = None;
    }
}
