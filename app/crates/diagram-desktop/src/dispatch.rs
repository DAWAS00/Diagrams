//! The only place that changes [`AppState`] in response to the person.
//!
//! Every button, menu item, shortcut, search result and pointer gesture
//! arrives here as a [`UiIntent`]. A command that is not available now
//! changes nothing.

use crate::app_state::{AppState, SAMPLE_PROJECT, sample_classes};
use crate::intent::DraftField;
use crate::intent::{InputContext, PointerKind, Surface, Target, UiIntent};
use crate::ui::commands::{self, CommandId};
use crate::ui::palette::PaletteState;
use crate::ui::showcase::ShowcaseState;
use crate::ui::workspace::{Drawer, LayoutMode};

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
        // The sample name is edited live until drafts exist (B-7), so there
        // is nothing left to commit or restore.
        UiIntent::CommitDraft(_) | UiIntent::CancelDraft(_) => {}
        UiIntent::Pointer(pointer) => {
            // Select is the only tool: a click on empty canvas clears the selection.
            if pointer.kind == PointerKind::Up && pointer.hit.is_none() {
                state.clear_selection();
            }
        }
        // One combined search exists until command and object search are split (B-9).
        UiIntent::Search { query, .. } => {
            if let Some(search) = state.palette.as_mut()
                && search.query != query
            {
                search.query = query;
                search.highlight_pending = true;
            }
        }
        // Region focus is tracked from B-8; the shell keeps egui's own focus until then.
        UiIntent::FocusRegion(_) => {}
        UiIntent::Close(surface) => close(state, surface),
        UiIntent::SetTheme(preference) => state.theme = preference,
        UiIntent::OpenSampleProject => {
            state.project_name = String::from(SAMPLE_PROJECT);
            state.classes = sample_classes();
            state.selected = None;
            state.entry = None;
        }
        UiIntent::CreateProject { name } => {
            state.project_name = name;
            state.classes.clear();
            state.selected = None;
            state.entry = None;
        }
    }
}

/// Runs `id` if it is available now; unavailable commands do nothing.
fn run(state: &mut AppState, input: &InputContext, id: CommandId) {
    if !commands::availability(id, state.command_context()).is_enabled() {
        return;
    }
    match id {
        CommandId::ClearSelection => state.clear_selection(),
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
        // Select is the only tool so far, so it is always the active one.
        CommandId::SelectTool => {}
        // Always unavailable until the project model exists (M1/M2).
        CommandId::Undo
        | CommandId::Redo
        | CommandId::Save
        | CommandId::Export
        | CommandId::PanTool
        | CommandId::AddClass
        | CommandId::Connect
        | CommandId::AddNote
        | CommandId::Arrange
        | CommandId::NewDiagram => {}
    }
}

fn select(state: &mut AppState, target: Target, toggle: bool) {
    let Target::Class(index) = target;
    if toggle && state.selected == Some(index) {
        state.clear_selection();
    } else {
        state.select(index);
    }
}

fn draft_changed(state: &mut AppState, field: DraftField, text: String) {
    match field {
        DraftField::ClassName => {
            if let Some(class) = state
                .selected
                .and_then(|index| state.classes.get_mut(index))
            {
                class.name = text;
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
