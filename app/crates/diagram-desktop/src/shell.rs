//! Native shell: toolbar, project panel, canvas, inspector and status bar.
//!
//! The window reads the open project's real [`Document`] through
//! [`crate::ui::read_model`] and changes it only by executing `diagram-core`
//! commands in the coordinator ([`apply_intents`]). Selection and open tabs
//! name records by typed ID, never by list position, so an ID that no longer
//! exists is ignored with a message instead of selecting the wrong record.

use std::sync::LazyLock;

use diagram_core::command::Command;
use diagram_core::ids::{DiagramId, ElementId, RelationId};
use diagram_core::model::Document;
use eframe::egui::{
    self, Align, Area, Button, Frame, Id, Key, Layout, Modal, Modifiers, Order, Pos2, Rect, Role,
    ScrollArea, Stroke, ThemePreference, Vec2,
};

use crate::session::ProjectSession;
pub use crate::session::{FIRST_DIAGRAM, SAMPLE_PROJECT};
use crate::ui::canvas::canvas;
use crate::ui::commands::{self, Availability, CommandContext, CommandId, CommandSpec, Placement};
use crate::ui::components;
use crate::ui::dialogs::{self, DialogAnswer, DialogKind, DialogState};
use crate::ui::entry::entry_screens;
use crate::ui::inspector::{self as rules, DraftKey};
use crate::ui::inspector_view::inspector;
use crate::ui::intents::UiIntent;
use crate::ui::navigator::OpenTab;
use crate::ui::notation::{self, NotationId};
use crate::ui::palette::{self, PaletteEntry, PaletteState};
use crate::ui::project_panel::sidebar;
use crate::ui::read_model;
use crate::ui::showcase::{self, ShowcaseState};
use crate::ui::status::status_bar;
use crate::ui::tabs::tab_row;
use crate::ui::theme::{self, Palette};
use crate::ui::welcome::{self, EntryState};
use crate::ui::workspace::{self, Drawer, INSPECTOR, LayoutMode, NAVIGATOR, PanelPreferences};

/// Message shown when a class name has no visible characters.
pub const BLANK_NAME_WARNING: &str = "A class name must contain a non-whitespace character.";

/// Shown while no project is open, so drawing code never needs a project.
static EMPTY_DOCUMENT: LazyLock<Document> = LazyLock::new(Document::default);

/// Editor state: the open project, the selection, the open tabs and the
/// window's own state (panels, theme, dialogs).
#[derive(Debug, Clone, PartialEq)]
pub struct ShellState {
    /// The open project; `None` on the welcome screen.
    project: Option<ProjectSession>,
    /// The selected class, if any (in the active tab).
    selected: Option<ElementId>,
    /// Further classes selected with Ctrl+click (multi-selection).
    also_selected: Vec<ElementId>,
    /// The selected relationship, if any.
    selected_relation: Option<RelationId>,
    /// Uncommitted inspector drafts; widgets edit these, never the records.
    pub drafts: std::collections::BTreeMap<DraftKey, String>,
    /// Whether the inspector's Appearance and Advanced sections are open.
    pub inspector_sections: (bool, bool),
    /// Open diagram tabs, in order.
    tabs: Vec<OpenTab>,
    /// Index into `tabs` of the active tab, if any tab is open.
    active_tab: Option<usize>,
    /// Text in the project panel's search field.
    pub navigator_search: String,
    /// Light, dark or follow the operating system. Light is the default.
    pub theme: ThemePreference,
    /// The command and object search, while it is open.
    pub palette: Option<PaletteState>,
    /// Which docked panels the user wants; kept across window sizes.
    pub panels: PanelPreferences,
    /// The open drawer on narrow windows, if any.
    pub drawer: Option<Drawer>,
    /// Docked or narrow, decided from the window width each frame.
    pub layout: LayoutMode,
    /// Widget to focus when the open drawer closes (usually its toggle).
    drawer_return: Option<Id>,
    /// The welcome or new-project screen, while one is shown instead of the workspace.
    pub entry: Option<EntryState>,
    /// The component showcase window, while it is open.
    pub showcase: Option<ShowcaseState>,
    /// Widget to focus when the showcase closes.
    showcase_return: Option<Id>,
    /// The open dialog, if any.
    dialog: Option<DialogState>,
    /// Widget to focus when the dialog closes.
    dialog_return: Option<Id>,
    /// A sentence about the last thing that was refused or undone, in words.
    notice: Option<String>,
    /// Counts projects opened or closed, so requests queued for an earlier
    /// project are dropped instead of hitting the next one.
    epoch: u64,
    /// Intents queued by widgets this frame, applied after drawing.
    intents: Vec<UiIntent>,
    /// Widget to focus at the start of the next frame, after a dialog closes.
    pending_focus: Option<Id>,
    /// Theme preference currently installed in the egui context.
    applied_theme: Option<ThemePreference>,
    /// Whether a text field (not just any widget) had keyboard focus at the
    /// end of the last frame.
    text_had_focus: bool,
}

impl Default for ShellState {
    /// The workspace showing the labelled sample project.
    fn default() -> Self {
        let mut state = Self::welcome();
        state.open_project(ProjectSession::sample());
        state
    }
}

impl ShellState {
    /// State at application start: the welcome screen.
    pub fn welcome() -> Self {
        Self {
            project: None,
            selected: None,
            also_selected: Vec::new(),
            selected_relation: None,
            drafts: std::collections::BTreeMap::new(),
            inspector_sections: (false, false),
            tabs: Vec::new(),
            active_tab: None,
            navigator_search: String::new(),
            theme: ThemePreference::Light,
            palette: None,
            panels: PanelPreferences::default(),
            drawer: None,
            layout: LayoutMode::Docked,
            drawer_return: None,
            entry: Some(EntryState::default()),
            showcase: None,
            showcase_return: None,
            dialog: None,
            dialog_return: None,
            notice: None,
            epoch: 0,
            intents: Vec::new(),
            pending_focus: None,
            applied_theme: None,
            text_had_focus: false,
        }
    }

    /// Makes `project` the open project, with its first diagram in a tab, and
    /// leaves the entry screens.
    pub fn open_project(&mut self, project: ProjectSession) {
        self.epoch += 1;
        self.clear_selection();
        self.drafts.clear();
        self.tabs.clear();
        self.active_tab = None;
        self.navigator_search.clear();
        self.notice = None;
        self.dialog = None;
        self.entry = None;
        let first = project.document().diagram_order.first().copied();
        self.project = Some(project);
        if let Some(diagram) = first {
            self.open_diagram(diagram);
        }
    }

    /// Closes the project and returns to the welcome screen.
    pub fn close_project(&mut self) {
        self.epoch += 1;
        self.project = None;
        self.clear_selection();
        self.drafts.clear();
        self.tabs.clear();
        self.active_tab = None;
        self.notice = None;
        self.dialog = None;
        self.palette = None;
        self.showcase = None;
        self.drawer = None;
        self.entry = Some(EntryState::default());
    }

    /// The open project, if any.
    pub const fn project(&self) -> Option<&ProjectSession> {
        self.project.as_ref()
    }

    /// The open document; empty while no project is open.
    pub fn document(&self) -> &Document {
        self.project
            .as_ref()
            .map_or(&*EMPTY_DOCUMENT, ProjectSession::document)
    }

    /// Name of the open project, empty on the welcome screen.
    pub fn project_name(&self) -> &str {
        self.project.as_ref().map_or("", ProjectSession::name)
    }

    /// The ID of the first class with this name, if there is one.
    pub fn element_id(&self, name: &str) -> Option<ElementId> {
        read_model::element_rows(self.document())
            .into_iter()
            .find(|row| row.name == name)
            .map(|row| row.id)
    }

    /// The ID of the first diagram with this name, if there is one.
    pub fn diagram_id(&self, name: &str) -> Option<DiagramId> {
        read_model::diagram_rows(self.document())
            .into_iter()
            .find(|row| row.name == name)
            .map(|row| row.id)
    }

    /// The selected class, if it still exists.
    pub fn selected_element(&self) -> Option<ElementId> {
        self.selected
            .filter(|id| self.document().elements.contains_key(id))
    }

    /// The selected relationship, if it still exists.
    pub fn selected_relation(&self) -> Option<RelationId> {
        self.selected_relation
            .filter(|id| self.document().relations.contains_key(id))
    }

    /// Every selected class, the primary one first.
    pub fn selected_elements(&self) -> Vec<ElementId> {
        self.selected
            .into_iter()
            .chain(self.also_selected.iter().copied())
            .filter(|id| self.document().elements.contains_key(id))
            .collect()
    }

    /// Open tabs, in order.
    pub fn tabs(&self) -> &[OpenTab] {
        &self.tabs
    }

    /// Index into [`Self::tabs`] of the active tab, if any tab is open.
    pub const fn active_tab(&self) -> Option<usize> {
        self.active_tab
    }

    /// The sentence about the last refused or undone action, if any.
    pub fn notice(&self) -> Option<&str> {
        self.notice.as_deref()
    }

    /// Whether a dialog is open.
    pub const fn has_dialog(&self) -> bool {
        self.dialog.is_some()
    }

    /// Clears the whole selection: classes and relationship.
    pub fn clear_selection(&mut self) {
        self.selected = None;
        self.also_selected.clear();
        self.selected_relation = None;
    }

    /// Selects only `element`. A class that no longer exists is ignored and
    /// the selection stays as it was; the result says whether it was selected.
    pub fn select(&mut self, element: ElementId) -> bool {
        if !self.document().elements.contains_key(&element) {
            return false;
        }
        self.clear_selection();
        self.selected = Some(element);
        true
    }

    /// Adds `element` to the selection, or removes it if already selected.
    /// Returns false for a class that no longer exists.
    pub fn toggle_in_selection(&mut self, element: ElementId) -> bool {
        if !self.document().elements.contains_key(&element) {
            return false;
        }
        self.selected_relation = None;
        if self.selected == Some(element) {
            self.selected = (!self.also_selected.is_empty()).then(|| self.also_selected.remove(0));
        } else if let Some(position) = self.also_selected.iter().position(|&id| id == element) {
            self.also_selected.remove(position);
        } else if self.selected.is_none() {
            self.selected = Some(element);
        } else {
            self.also_selected.push(element);
        }
        true
    }

    /// Selects only `relation`. Returns false for one that no longer exists.
    pub fn select_relation(&mut self, relation: RelationId) -> bool {
        if !self.document().relations.contains_key(&relation) {
            return false;
        }
        self.clear_selection();
        self.selected_relation = Some(relation);
        true
    }

    /// The diagram in the active tab, if a tab is open.
    pub fn active_diagram(&self) -> Option<DiagramId> {
        self.active_tab
            .and_then(|tab| self.tabs.get(tab))
            .map(|tab| tab.diagram)
    }

    /// Notation of the active diagram; Class when no diagram is open.
    pub fn active_notation(&self) -> NotationId {
        self.active_diagram()
            .and_then(|id| self.document().diagrams.get(&id))
            .map_or(NotationId::Class, |diagram| {
                NotationId::from_core(diagram.notation)
            })
    }

    /// Classes shown on the canvas: those in the active diagram.
    pub fn visible_elements(&self) -> Vec<ElementId> {
        self.active_diagram()
            .map(|diagram| read_model::elements_in(self.document(), diagram))
            .unwrap_or_default()
    }

    /// Makes the tab showing `diagram` active. The leaving tab keeps its
    /// selection and the arriving tab restores its own. Returns false when
    /// no open tab shows that diagram.
    pub fn activate_tab(&mut self, diagram: DiagramId) -> bool {
        let Some(index) = self.tabs.iter().position(|tab| tab.diagram == diagram) else {
            return false;
        };
        if self.active_tab == Some(index) {
            return true;
        }
        if let Some(current) = self.active_tab.and_then(|tab| self.tabs.get_mut(tab)) {
            current.selected = self.selected;
        }
        self.active_tab = Some(index);
        let restored = self.tabs[index].selected;
        self.clear_selection();
        self.selected = restored.filter(|id| self.document().elements.contains_key(id));
        true
    }

    /// Shows `diagram` in its tab, opening a tab if it has none. Returns
    /// false for a diagram that no longer exists.
    pub fn open_diagram(&mut self, diagram: DiagramId) -> bool {
        if !self.document().diagrams.contains_key(&diagram) {
            return false;
        }
        if !self.tabs.iter().any(|tab| tab.diagram == diagram) {
            self.tabs.push(OpenTab {
                diagram,
                selected: None,
            });
        }
        self.activate_tab(diagram)
    }

    /// Closes the tab showing `diagram`. Model elements are untouched; a
    /// neighbouring tab becomes active when the active one closes. Returns
    /// false when that diagram has no open tab.
    pub fn close_tab(&mut self, diagram: DiagramId) -> bool {
        let Some(index) = self.tabs.iter().position(|tab| tab.diagram == diagram) else {
            return false;
        };
        if let Some(current) = self.active_tab.and_then(|tab| self.tabs.get_mut(tab)) {
            current.selected = self.selected;
        }
        let was_active = self.active_tab == Some(index);
        self.tabs.remove(index);
        self.active_tab = match self.active_tab {
            _ if self.tabs.is_empty() => None,
            Some(active) if active > index => Some(active - 1),
            Some(active) if was_active => Some(active.min(self.tabs.len() - 1)),
            other => other,
        };
        // Closing a background tab leaves the active tab's selection alone.
        if was_active {
            let restored = self
                .active_tab
                .and_then(|tab| self.tabs.get(tab))
                .and_then(|tab| tab.selected);
            self.clear_selection();
            self.selected = restored.filter(|id| self.document().elements.contains_key(id));
        }
        true
    }

    /// Selects a class from the project panel. If the active diagram does not
    /// show it, a diagram that does is opened first ("reveal"). Returns false
    /// for a class that no longer exists.
    pub fn reveal_class(&mut self, element: ElementId) -> bool {
        if !self.document().elements.contains_key(&element) {
            return false;
        }
        if !self.visible_elements().contains(&element)
            && let Some(&diagram) = read_model::diagrams_showing(self.document(), element).first()
        {
            self.open_diagram(diagram);
        }
        self.select(element)
    }

    /// Queues `intent`; it is applied after the current frame is drawn.
    pub fn queue(&mut self, intent: UiIntent) {
        self.intents.push(intent);
    }

    /// Whether a text field had keyboard focus at the end of the last frame.
    pub(crate) fn text_had_focus(&self) -> bool {
        self.text_had_focus
    }

    /// Asks for keyboard focus on `target` at the start of the next frame.
    pub(crate) fn focus_next_frame(&mut self, target: Option<Id>) {
        self.pending_focus = target;
    }

    /// The state that decides which commands are available.
    pub fn command_context(&self) -> CommandContext {
        CommandContext {
            has_selection: self.selected_element().is_some() || self.selected_relation().is_some(),
            has_project: self.project.is_some(),
            can_undo: self
                .project
                .as_ref()
                .is_some_and(|project| project.undo_label().is_some()),
            can_redo: self
                .project
                .as_ref()
                .is_some_and(|project| project.redo_label().is_some()),
        }
    }

    /// "Selected: Customer", "2 classes selected" or "No selection", for the status bar.
    pub(crate) fn selection_summary(&self) -> String {
        if let Some(relation) = self
            .selected_relation()
            .and_then(|id| self.document().relations.get(&id))
        {
            return format!(
                "Selected: {}",
                read_model::relation_label(self.document(), relation)
            );
        }
        match self.selected_elements().as_slice() {
            [] => String::from("No selection"),
            [only] => format!(
                "Selected: {}",
                read_model::element_name(self.document(), *only).unwrap_or("?")
            ),
            several => format!("{} classes selected", several.len()),
        }
    }

    /// Executes `command` on the open project. A refusal becomes a notice in
    /// words and leaves the document unchanged. Returns whether it applied.
    fn execute(&mut self, command: &Command) -> bool {
        let Some(project) = self.project.as_mut() else {
            return false;
        };
        match project.execute(command) {
            Ok(_) => {
                self.after_document_change(false);
                true
            }
            Err(error) => {
                self.notice = Some(format!("Error: {error}. Nothing was changed."));
                false
            }
        }
    }

    /// Reverses the last action and says what it was.
    fn undo(&mut self) {
        if let Some(outcome) = self.project.as_mut().and_then(ProjectSession::undo) {
            self.notice = Some(format!("Undid: {}.", outcome.label));
            self.after_document_change(true);
        }
    }

    /// Repeats the last undone action and says what it was.
    fn redo(&mut self) {
        if let Some(outcome) = self.project.as_mut().and_then(ProjectSession::redo) {
            self.notice = Some(format!("Redid: {}.", outcome.label));
            self.after_document_change(true);
        }
    }

    /// Drops selection, tabs and drafts that name records the document no
    /// longer has (after an edit, an undo or a redo). After a history move
    /// (`history_moved`) every draft goes, because the text it started from
    /// may have changed.
    fn after_document_change(&mut self, history_moved: bool) {
        let elements = self.selected_elements();
        let relation = self.selected_relation();
        let stale_tabs: Vec<DiagramId> = self
            .tabs
            .iter()
            .map(|tab| tab.diagram)
            .filter(|id| !self.document().diagrams.contains_key(id))
            .collect();
        self.selected = elements.first().copied();
        self.also_selected = elements.into_iter().skip(1).collect();
        self.selected_relation = relation;
        for diagram in stale_tabs {
            self.close_tab(diagram);
        }
        if history_moved {
            self.drafts.clear();
        } else {
            let document = self.project.as_ref().map(ProjectSession::document);
            self.drafts.retain(|key, _| {
                document.is_some_and(|document| match key {
                    DraftKey::ClassName(id) => document.elements.contains_key(id),
                    DraftKey::Relation(id, _) => document.relations.contains_key(id),
                })
            });
        }
    }

    /// Tells the user that an ID in a request no longer names anything.
    fn stale(&mut self, what: &str) {
        self.notice = Some(format!(
            "That {what} no longer exists, so nothing was changed."
        ));
    }
}

/// The coordinator: applies queued intents in order, after drawing. This is
/// the only place the editor state and the document change.
fn apply_intents(state: &mut ShellState, ctx: &egui::Context) {
    let intents = std::mem::take(&mut state.intents);
    if !intents.is_empty() {
        if !intents
            .iter()
            .all(|intent| matches!(intent, UiIntent::SetTheme(_)))
        {
            state.notice = None;
        }
        // The state changed after this frame was drawn: draw the next one.
        ctx.request_repaint();
    }
    let epoch = state.epoch;
    // A draft committed in the same frame as Undo or Redo was typed against
    // the text before the history moved, whichever way round they were
    // queued; it must not undo the undo or clear the redo.
    let history_in_batch = intents.iter().any(|intent| {
        matches!(
            intent,
            UiIntent::Run {
                command: CommandId::Undo | CommandId::Redo,
                ..
            }
        )
    });
    for intent in intents {
        if state.epoch != epoch {
            // A project was opened or closed: what is left was queued for the
            // old one and must not touch the new one.
            state.notice = Some(String::from(
                "The project changed, so the remaining requests were dropped.",
            ));
            break;
        }
        if history_in_batch
            && matches!(
                intent,
                UiIntent::RenameClass(..) | UiIntent::SetRelationField(..)
            )
        {
            continue;
        }
        match intent {
            UiIntent::Run { command, origin } => {
                let drawer_before = state.drawer;
                let showcase_before = state.showcase.is_some();
                let dialog_before = state.dialog.is_some();
                run(state, ctx, command);
                // Whatever this control opened returns focus to it.
                if let Some(origin) = origin {
                    if state.drawer.is_some() && state.drawer != drawer_before {
                        state.drawer_return = Some(origin);
                    }
                    if state.showcase.is_some() && !showcase_before {
                        state.showcase_return = Some(origin);
                    }
                    if state.dialog.is_some() && !dialog_before {
                        state.dialog_return = Some(origin);
                    }
                }
            }
            UiIntent::SelectClass(id) => {
                if !state.select(id) {
                    state.stale("class");
                }
            }
            UiIntent::ToggleClassInSelection(id) => {
                if !state.toggle_in_selection(id) {
                    state.stale("class");
                }
            }
            UiIntent::SelectRelation(id) => {
                if !state.select_relation(id) {
                    state.stale("relationship");
                }
            }
            UiIntent::RenameClass(id, name) => rename_class(state, id, &name),
            UiIntent::SetRelationField(id, field, value) => {
                set_relation_field(state, id, field, &value);
            }
            UiIntent::RevealClass(id) => {
                if !state.reveal_class(id) {
                    state.stale("class");
                }
            }
            UiIntent::OpenDiagram(id) => {
                if !state.open_diagram(id) {
                    state.stale("diagram");
                }
            }
            UiIntent::ActivateTab(id) => {
                if !state.activate_tab(id) {
                    state.stale("tab");
                }
            }
            UiIntent::CloseTab(id) => {
                if !state.close_tab(id) {
                    state.stale("tab");
                }
            }
            UiIntent::SetTheme(preference) => state.theme = preference,
            UiIntent::CloseDrawer => close_drawer(state),
            UiIntent::CloseShowcase => close_showcase(state),
            UiIntent::CreateProject(name) => {
                if state.project.is_none() && welcome::name_error(&name).is_none() {
                    state.open_project(ProjectSession::blank(&name));
                }
            }
            UiIntent::OpenSample => {
                if state.project.is_none() {
                    state.open_project(ProjectSession::sample());
                }
            }
            UiIntent::AnswerDialog(answer) => answer_dialog(state, answer),
        }
    }
}

/// Renames a class as one undoable command, after checking the name again.
fn rename_class(state: &mut ShellState, id: ElementId, name: &str) {
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
fn set_relation_field(
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
fn answer_dialog(state: &mut ShellState, answer: DialogAnswer) {
    let Some(dialog) = state.dialog.take() else {
        return;
    };
    state.pending_focus = state.dialog_return.take();
    match (dialog.kind, answer) {
        (DialogKind::CloseProject { .. }, DialogAnswer::Confirm) => state.close_project(),
        (DialogKind::CloseProject { .. }, DialogAnswer::Cancel) => {}
    }
}

/// Runs `id` if it is available now; unavailable commands do nothing.
fn run(state: &mut ShellState, ctx: &egui::Context, id: CommandId) {
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
        // Select is the only tool so far, so it is always the active one.
        CommandId::SelectTool => {}
        // Always unavailable until later slices build them.
        CommandId::Save
        | CommandId::Export
        | CommandId::PanTool
        | CommandId::AddClass
        | CommandId::Connect
        | CommandId::AddNote
        | CommandId::Arrange
        | CommandId::NewDiagram => {}
    }
}

/// Closes the project now, or asks first when that would lose work.
fn close_project_command(state: &mut ShellState, ctx: &egui::Context) {
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
fn toggle_panel(state: &mut ShellState, ctx: &egui::Context, which: Drawer) {
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
fn close_drawer(state: &mut ShellState) {
    state.drawer = None;
    state.pending_focus = state.drawer_return.take();
}

/// Closes the component showcase and returns focus to where it came from.
fn close_showcase(state: &mut ShellState) {
    state.showcase = None;
    state.pending_focus = state.showcase_return.take();
}

/// The component showcase as a movable window with an explicit Close button.
fn showcase_window(ctx: &egui::Context, state: &mut ShellState) {
    let Some(mut fixture) = state.showcase.take() else {
        return;
    };
    let mut close = false;
    egui::Window::new("Component showcase")
        .id(Id::unique("component-showcase"))
        .collapsible(false)
        .default_size([460.0, 640.0])
        .default_pos([260.0, 60.0])
        .max_height((ctx.content_rect().height() - 80.0).max(200.0))
        .show(ctx, |ui| {
            if components::action_button(ui, "Close showcase", None, Some("Escape")).clicked() {
                close = true;
            }
            ui.separator();
            ScrollArea::vertical().show(ui, |ui| showcase::show(ui, &mut fixture));
        });
    state.showcase = Some(fixture);
    if close {
        close_showcase(state);
    }
}

/// Whether a panel's toggle shows as pressed.
fn panel_shown(state: &ShellState, which: Drawer) -> bool {
    match (state.layout, which) {
        (LayoutMode::Docked, Drawer::Navigator) => state.panels.navigator_visible,
        (LayoutMode::Docked, Drawer::Inspector) => state.panels.inspector_visible,
        (LayoutMode::Narrow, _) => state.drawer == Some(which),
    }
}

/// Draws the complete shell into `ui` and applies the user's interactions to `state`.
pub fn show(ui: &mut egui::Ui, state: &mut ShellState) {
    if state.applied_theme != Some(state.theme) {
        theme::apply(ui.ctx(), state.theme);
        // This frame's root Ui was created before the styles were installed;
        // update it too, or named text roles (page title) are missing on the
        // first frame and egui panics.
        ui.set_style(ui.ctx().global_style());
        state.applied_theme = Some(state.theme);
    }
    let palette = *theme::palette(ui.ctx().theme());
    // egui keeps the closed modal's layer on top for one more frame, and
    // widgets behind a modal give up focus. Return focus once it is gone.
    if let Some(id) = state.pending_focus {
        if ui.ctx().memory(|memory| memory.top_modal_layer()).is_none() {
            ui.ctx().memory_mut(|memory| memory.request_focus(id));
            state.pending_focus = None;
        } else {
            ui.ctx().request_repaint();
        }
    }

    if state.entry.is_some() {
        entry_screens(ui, state, &palette);
        apply_intents(state, ui.ctx());
        state.text_had_focus = ui.ctx().text_edit_focused();
        return;
    }

    let layout = workspace::layout_mode(ui.max_rect().width());
    if layout != state.layout {
        state.layout = layout;
        state.drawer = None;
        state.drawer_return = None;
    }

    // Shortcuts use the text focus of the previous frame: egui drops focus in
    // the frame Escape is pressed, and Escape must first leave the field.
    // An open drawer takes Escape before the canvas selection does.
    let text_had_focus = state.text_had_focus;
    if state.dialog.is_none()
        && state.showcase.is_some()
        && !text_had_focus
        && ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape))
    {
        state.queue(UiIntent::CloseShowcase);
    }
    if state.dialog.is_none()
        && state.drawer.is_some()
        && !text_had_focus
        && ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape))
    {
        state.queue(UiIntent::CloseDrawer);
    }
    // A dialog owns the keyboard: shortcuts do nothing behind it.
    if state.dialog.is_none()
        && let Some(id) = ui.input_mut(|input| commands::take_shortcut(input, text_had_focus))
    {
        state.queue(UiIntent::run(id));
    }

    let bar = Frame::side_top_panel(ui.style()).fill(palette.workspace);
    egui::Panel::top("header")
        .frame(bar)
        .exact_size(workspace::HEADER_HEIGHT)
        .show(ui, |ui| header(ui, state));
    egui::Panel::bottom("status")
        .frame(bar)
        .show(ui, |ui| status_bar(ui, state, true));
    if state.layout == LayoutMode::Docked {
        if state.panels.navigator_visible {
            egui::Panel::left("navigator")
                .resizable(true)
                .default_size(NAVIGATOR.default)
                .size_range(NAVIGATOR.min..=NAVIGATOR.max)
                .show(ui, |ui| {
                    scrolling(ui, "navigator", "Project panel contents", |ui| {
                        sidebar(ui, state)
                    })
                });
        }
        if state.panels.inspector_visible {
            egui::Panel::right("inspector")
                .resizable(true)
                .default_size(INSPECTOR.default)
                .size_range(INSPECTOR.min..=INSPECTOR.max)
                .show(ui, |ui| {
                    scrolling(ui, "inspector", "Inspector contents", |ui| {
                        inspector(ui, state)
                    })
                });
        }
    }
    let canvas_rect = egui::CentralPanel::default()
        .frame(Frame::new().fill(palette.workspace))
        .show(ui, |ui| {
            let row = Frame::side_top_panel(ui.style()).fill(palette.panel);
            egui::Panel::top("tabs")
                .frame(row)
                .exact_size(workspace::TAB_ROW_HEIGHT)
                .show(ui, |ui| tab_row(ui, state));
            egui::Panel::top("tools")
                .frame(row)
                .exact_size(workspace::TOOL_ROW_HEIGHT)
                .show(ui, |ui| tool_row(ui, state));
            Frame::new()
                .inner_margin(theme::SPACING)
                .show(ui, |ui| canvas(ui, state, &palette))
                .inner
        })
        .inner;
    drawer(ui.ctx(), state, canvas_rect, &palette);
    showcase_window(ui.ctx(), state);
    command_palette(ui.ctx(), state);
    dialog_window(ui.ctx(), state);
    apply_intents(state, ui.ctx());
    // Only a focused text field keeps editing keys; a focused button must not
    // swallow Escape (egui_wants_keyboard_input is true for any focus).
    state.text_had_focus = ui.ctx().text_edit_focused();
}

/// A command button with its shortcut as a hover hint; disabled commands
/// explain why.
pub(crate) fn command_button(
    ui: &mut egui::Ui,
    spec: &CommandSpec,
    context: CommandContext,
    clicked: &mut Option<CommandId>,
) {
    let hint = spec
        .shortcut
        .map(|shortcut| format!("Shortcut: {}", ui.ctx().format_shortcut(&shortcut)));
    let availability = commands::availability(spec.id, context);
    if components::action_button(ui, spec.label, availability.reason(), hint.as_deref()).clicked() {
        *clicked = Some(spec.id);
    }
}

/// The application menu: every catalog command, grouped, with shortcuts.
fn command_menu(ui: &mut egui::Ui, context: CommandContext, clicked: &mut Option<CommandId>) -> Id {
    let menu = ui.menu_button("Menu", |ui| {
        for (group, specs) in commands::menu_groups() {
            ui.menu_button(group.label(), |ui| {
                for spec in specs {
                    let availability = commands::availability(spec.id, context);
                    let mut item = Button::new(spec.label);
                    if let Some(shortcut) = spec.shortcut {
                        item = item.shortcut_text(ui.ctx().format_shortcut(&shortcut));
                    }
                    let response = ui.add_enabled(availability.is_enabled(), item);
                    let response = match availability.reason() {
                        Some(reason) => response.on_disabled_hover_text(reason),
                        None => response,
                    };
                    if response.clicked() {
                        *clicked = Some(spec.id);
                        ui.close();
                    }
                }
            });
        }
    });
    menu.response.id
}

/// Header: menu, project name, then Undo, Redo, Search, Save and Export.
fn header(ui: &mut egui::Ui, state: &mut ShellState) {
    let context = state.command_context();
    let mut clicked = None;
    let mut from_menu = None;
    let mut menu_button = None;
    ui.horizontal_centered(|ui| {
        menu_button = Some(command_menu(ui, context, &mut from_menu));
        ui.separator();
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            // Right-to-left: add in reverse visual order, with a separator
            // between command groups.
            let mut previous_group = None;
            for spec in commands::placed(Placement::Header)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
            {
                if previous_group.is_some_and(|group| group != spec.group) {
                    ui.separator();
                }
                previous_group = Some(spec.group);
                command_button(ui, spec, context, &mut clicked);
            }
            ui.separator();
            ui.allocate_ui_with_layout(
                ui.available_size(),
                Layout::left_to_right(Align::Center),
                |ui| {
                    let title = format!("{} · not saved", state.project_name());
                    components::truncated_text(ui, &title)
                },
            );
        });
    });
    if let Some(command) = from_menu {
        // A dialog the command opens returns focus to the Menu button.
        state.queue(UiIntent::Run {
            command,
            origin: menu_button,
        });
    }
    if let Some(id) = clicked {
        state.queue(UiIntent::run(id));
    }
}

/// Tool row: project panel button, tools, inspector button.
fn tool_row(ui: &mut egui::Ui, state: &mut ShellState) {
    let context = state.command_context();
    let mut clicked = None;
    let mut toggle = None;
    ui.horizontal_centered(|ui| {
        let navigator = components::choice(
            ui,
            panel_shown(state, Drawer::Navigator),
            CommandId::ToggleNavigator.spec().label,
        );
        if navigator.clicked() {
            toggle = Some((CommandId::ToggleNavigator, navigator.id));
        }
        ui.separator();
        // Tools come from the active diagram's notation contract.
        for spec in notation::tool_commands(state.active_notation()).map(CommandId::spec) {
            if spec.id == CommandId::SelectTool {
                // The active tool shows as pressed.
                if components::choice(ui, true, spec.label).clicked() {
                    clicked = Some(spec.id);
                }
            } else {
                command_button(ui, spec, context, &mut clicked);
            }
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let inspector = components::choice(
                ui,
                panel_shown(state, Drawer::Inspector),
                CommandId::ToggleInspector.spec().label,
            );
            if inspector.clicked() {
                toggle = Some((CommandId::ToggleInspector, inspector.id));
            }
        });
    });
    if let Some(id) = clicked {
        state.queue(UiIntent::run(id));
    }
    if let Some((id, button)) = toggle {
        // A drawer opened from its button returns focus to that button.
        state.queue(UiIntent::Run {
            command: id,
            origin: Some(button),
        });
    }
}

/// Wraps panel contents in a vertical scroll area that assistive technology
/// (and the usability audit) sees as a named, clipping scroll region.
fn scrolling(ui: &mut egui::Ui, id: &str, label: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    ui.scope(|ui| {
        let region = ui.unique_id();
        let output = ScrollArea::vertical()
            .id_salt(id)
            .auto_shrink([false, false])
            .show(ui, add_contents);
        // Same point coordinates egui uses for widget bounds.
        let visible = output.inner_rect;
        ui.ctx().accesskit_node_builder(region, |node| {
            node.set_role(Role::ScrollView);
            node.set_label(label);
            node.set_clips_children();
            node.set_bounds(egui::accesskit::Rect {
                x0: visible.min.x.into(),
                y0: visible.min.y.into(),
                x1: visible.max.x.into(),
                y1: visible.max.y.into(),
            });
        });
    });
}

/// The open drawer, over the canvas only, never over the header, rows or
/// status bar.
fn drawer(ctx: &egui::Context, state: &mut ShellState, canvas_rect: Rect, palette: &Palette) {
    let Some(which) = state.drawer else {
        return;
    };
    let width = which.width(canvas_rect.width());
    let rect = match which {
        Drawer::Navigator => {
            Rect::from_min_size(canvas_rect.min, Vec2::new(width, canvas_rect.height()))
        }
        Drawer::Inspector => Rect::from_min_max(
            Pos2::new(canvas_rect.max.x - width, canvas_rect.min.y),
            canvas_rect.max,
        ),
    };
    let close_label = match which {
        Drawer::Navigator => "Close project panel",
        Drawer::Inspector => "Close inspector",
    };
    let frame = Frame::new()
        .fill(palette.panel)
        .stroke(Stroke::new(1.0, palette.separator))
        .inner_margin(theme::SPACING);
    let inner_size = rect.size() - Vec2::splat(2.0 * (theme::SPACING + 1.0));
    let mut close = false;
    Area::new(Id::unique("drawer"))
        .order(Order::Foreground)
        .fixed_pos(rect.min)
        .show(ctx, |ui| {
            frame.show(ui, |ui| {
                ui.set_min_size(inner_size);
                ui.set_max_size(inner_size);
                if components::action_button(ui, close_label, None, Some("Escape")).clicked() {
                    close = true;
                }
                ui.separator();
                scrolling(ui, "drawer", "Drawer contents", |ui| match which {
                    Drawer::Navigator => sidebar(ui, state),
                    Drawer::Inspector => inspector(ui, state),
                });
            });
        });
    if close {
        state.queue(UiIntent::CloseDrawer);
    }
}

/// The open dialog, if any. Its answer is queued as an intent.
fn dialog_window(ctx: &egui::Context, state: &mut ShellState) {
    let Some(mut dialog) = state.dialog.take() else {
        return;
    };
    match dialogs::show(ctx, &mut dialog) {
        Some(answer) => {
            state.dialog = Some(dialog);
            state.queue(UiIntent::AnswerDialog(answer));
        }
        None => state.dialog = Some(dialog),
    }
}

/// Command and object search. Enter runs the highlighted result, arrows move
/// the highlight, Escape or a click outside closes it, and focus returns to
/// where it was.
fn command_palette(ctx: &egui::Context, state: &mut ShellState) {
    let Some(mut search) = state.palette.take() else {
        return;
    };
    let rows = read_model::element_rows(state.document());
    let names: Vec<(ElementId, &str)> =
        rows.iter().map(|row| (row.id, row.name.as_str())).collect();
    let context = state.command_context();
    let results = palette::entries(&search.query, &names);
    let (down, up, enter) = ctx.input(|input| {
        (
            input.key_pressed(Key::ArrowDown),
            input.key_pressed(Key::ArrowUp),
            input.key_pressed(Key::Enter),
        )
    });
    let entry_availability = |entry: PaletteEntry| match entry {
        PaletteEntry::Command(id) => commands::availability(id, context),
        PaletteEntry::Object(_) => Availability::Enabled,
    };
    if search.highlight_pending {
        search.highlighted =
            palette::first_available(&results, |entry| entry_availability(entry).is_enabled());
        search.highlight_pending = false;
    }
    search.highlighted = palette::move_highlight(search.highlighted, results.len(), down, up);
    let mut chosen = if enter {
        results.get(search.highlighted).copied()
    } else {
        None
    };

    let modal = Modal::new(Id::unique("command-palette")).show(ctx, |ui| {
        ui.set_width(480.0);
        components::panel_title(ui, "Search commands and objects");
        let field = components::text_field(ui, "Find a command or object", &mut search.query);
        if !search.focused_once {
            field.request_focus();
            search.focused_once = true;
        }
        if field.changed() {
            search.highlight_pending = true;
        }
        ui.add_space(theme::GAP);
        if results.is_empty() {
            ui.label("No commands or objects match.");
        }
        ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
            for (index, entry) in results.iter().copied().enumerate() {
                let availability = entry_availability(entry);
                let mut row = Button::selectable(
                    index == search.highlighted,
                    palette::entry_label(entry, &names),
                )
                .truncate()
                .min_size(Vec2::new(ui.available_width(), theme::ROW_HEIGHT));
                if let PaletteEntry::Command(id) = entry
                    && let Some(shortcut) = id.spec().shortcut
                {
                    row = row.shortcut_text(ctx.format_shortcut(&shortcut));
                }
                let response = ui.add_enabled(availability.is_enabled(), row);
                if index == search.highlighted && (down || up) {
                    response.scroll_to_me(None);
                }
                if let Some(reason) = availability.reason() {
                    components::field_label(ui, reason);
                }
                if response.clicked() {
                    chosen = Some(entry);
                }
            }
        });
    });

    let close_requested = modal.should_close();
    let ran = match chosen {
        Some(entry) if !entry_availability(entry).is_enabled() => false,
        Some(PaletteEntry::Command(id)) => {
            state.queue(UiIntent::run(id));
            true
        }
        Some(PaletteEntry::Object(id)) => {
            state.queue(UiIntent::RevealClass(id));
            true
        }
        None => false,
    };
    if ran || close_requested {
        state.pending_focus = search.restore_focus;
        ctx.request_repaint();
    } else {
        state.palette = Some(search);
    }
}

/// eframe application wrapper around [`ShellState`]; starts on the welcome screen.
#[derive(Debug)]
pub struct ShellApp {
    state: ShellState,
}

impl Default for ShellApp {
    fn default() -> Self {
        Self {
            state: ShellState::welcome(),
        }
    }
}

impl eframe::App for ShellApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        show(ui, &mut self.state);
    }
}
