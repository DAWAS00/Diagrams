//! Coordinator helpers that apply one kind of request each. Called only
//! from `apply_intents`. Moved out of `shell.rs` unchanged.

#[allow(clippy::wildcard_imports)]
use super::*;

/// Renames a class as one undoable command, after checking the name again.
pub(super) fn rename_class(state: &mut ShellState, id: ElementId, name: &str) {
    if let Some(message) = rules::class_name_error(name) {
        state.notice = Some(message);
    } else if !state.document().elements.contains_key(&id) {
        state.stale("class");
    } else if read_model::element_name(state.document(), id) != Some(name.trim()) {
        state.execute(&Command::RenameElement {
            element: id,
            name: name.trim().to_owned(),
        });
    }
}

/// Sets one field of an association's ends as one undoable command.
pub(super) fn set_relation_field(
    state: &mut ShellState,
    id: RelationId,
    field: rules::RelationField,
    value: &str,
) {
    if let Some(message) = field.error(value) {
        state.notice = Some(message);
        return;
    }
    let Some(relation) = state.document().relations.get(&id).cloned() else {
        state.stale("relationship");
        return;
    };
    if field.unchanged(&relation, value) {
        return;
    }
    let (source, target) = field.ends_with(&relation, value);
    state.execute(&Command::SetEnds {
        relation: id,
        source,
        target,
    });
}

/// Applies the user's answer to the open dialog.
pub(super) fn answer_dialog(state: &mut ShellState, answer: DialogAnswer) {
    let Some(dialog) = state.dialog.take() else {
        return;
    };
    state.pending_focus = state.dialog_return.take();
    match (dialog.kind, answer) {
        (DialogKind::CloseProject { .. }, DialogAnswer::Confirm) => state.close_project(),
        (DialogKind::CloseProject { .. }, DialogAnswer::Cancel | DialogAnswer::Alternative) => {}
        (
            DialogKind::DuplicateClassName {
                placement,
                name,
                existing,
            },
            answer,
        ) => state.answer_duplicate(placement, &name, existing, answer),
    }
}

/// Runs `id` if it is available now; unavailable commands do nothing.
pub(super) fn run(state: &mut ShellState, ctx: &egui::Context, id: CommandId) {
    if !commands::availability(id, state.command_context()).is_enabled() {
        return;
    }
    match id {
        CommandId::ClearSelection => state.clear_selection(),
        CommandId::Undo => state.undo(),
        CommandId::Redo => state.redo(),
        CommandId::CloseProject => close_project_command(state, ctx),
        CommandId::SearchCommands => {
            if state.palette.is_none() {
                let focused = ctx.memory(|memory| memory.focused());
                state.palette = Some(PaletteState::new(focused));
            }
        }
        CommandId::ToggleNavigator => toggle_panel(state, ctx, Drawer::Navigator),
        CommandId::ToggleInspector => toggle_panel(state, ctx, Drawer::Inspector),
        CommandId::ShowComponentShowcase => {
            if state.showcase.is_none() {
                state.showcase_return = ctx.memory(|memory| memory.focused());
                state.showcase = Some(ShowcaseState::default());
            }
        }
        CommandId::SelectTool => state.class_tool.arm(Tool::Select),
        CommandId::AddClass => state.class_tool.arm(Tool::Class),
        // Always unavailable until later slices build them.
        CommandId::Save
        | CommandId::Export
        | CommandId::PanTool
        | CommandId::Connect
        | CommandId::AddNote
        | CommandId::Arrange
        | CommandId::NewDiagram => {}
    }
}

/// Closes the project now, or asks first when that would lose work.
pub(super) fn close_project_command(state: &mut ShellState, ctx: &egui::Context) {
    let Some(project) = state.project.as_ref() else {
        return;
    };
    if state.dialog.is_some() {
        return;
    }
    if project.has_unsaved_changes() {
        let name = project.name().to_owned();
        // Focus goes back to what had it before: a search that just closed
        // names it in `pending_focus`; otherwise the focused widget.
        state.dialog_return = state
            .pending_focus
            .take()
            .or_else(|| ctx.memory(|memory| memory.focused()));
        state.dialog = Some(DialogState::new(DialogKind::CloseProject { project: name }));
    } else {
        state.close_project();
    }
}

/// Shows or hides a docked panel, or opens and closes its drawer on narrow windows.
pub(super) fn toggle_panel(state: &mut ShellState, ctx: &egui::Context, which: Drawer) {
    match state.layout {
        LayoutMode::Docked => match which {
            Drawer::Navigator => state.panels.navigator_visible ^= true,
            Drawer::Inspector => state.panels.inspector_visible ^= true,
        },
        LayoutMode::Narrow if state.drawer == Some(which) => close_drawer(state),
        LayoutMode::Narrow => {
            state.drawer_return = ctx.memory(|memory| memory.focused());
            state.drawer = Some(which);
        }
    }
}

/// Closes the open drawer and returns focus to where it came from.
pub(super) fn close_drawer(state: &mut ShellState) {
    state.drawer = None;
    state.pending_focus = state.drawer_return.take();
}

/// Closes the component showcase and returns focus to where it came from.
pub(super) fn close_showcase(state: &mut ShellState) {
    state.showcase = None;
    state.pending_focus = state.showcase_return.take();
}
