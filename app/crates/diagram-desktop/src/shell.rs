//! Native shell: toolbar, project panel, canvas, inspector and status bar.
//!
//! The window reads the open project's real [`Document`] through
//! [`crate::ui::read_model`] and changes it only by executing `diagram-core`
//! commands in the coordinator (`apply_intents`). Selection and open tabs
//! name records by typed ID, never by list position, so an ID that no longer
//! exists is ignored with a message instead of selecting the wrong record.

use std::sync::LazyLock;

use diagram_core::command::Command;
use diagram_core::ids::{DiagramId, ElementId, RelationId};
use diagram_core::model::Document;
use eframe::egui::{self, Frame, Id, ThemePreference};

use crate::session::ProjectSession;
pub use crate::session::{FIRST_DIAGRAM, SAMPLE_PROJECT};
use crate::ui::canvas::canvas;
use crate::ui::commands::{self, CommandContext, CommandId};
use crate::ui::components;
use crate::ui::dialogs::{self, DialogAnswer, DialogKind, DialogState};
use crate::ui::drawer::drawer;
use crate::ui::entry::entry_screens;
use crate::ui::header::header;
use crate::ui::inspector::{self as rules, DraftKey};
use crate::ui::inspector_view::inspector;
use crate::ui::intents::UiIntent;
use crate::ui::keys::route_keys;
use crate::ui::navigator::OpenTab;
use crate::ui::notation::NotationId;
use crate::ui::palette::PaletteState;
use crate::ui::project_panel::sidebar;
use crate::ui::read_model;
use crate::ui::search::command_palette;
use crate::ui::showcase::{ShowcaseState, showcase_window};
use crate::ui::status::status_bar;
use crate::ui::tabs::tab_row;
use crate::ui::theme;
use crate::ui::tool_row::tool_row;
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
        // A background tab remembers its own selection; forget classes that
        // are gone, so a later Redo cannot bring a dropped selection back.
        if let Some(project) = self.project.as_ref() {
            let document = project.document();
            for tab in &mut self.tabs {
                tab.selected = tab.selected.filter(|id| document.elements.contains_key(id));
            }
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

    route_keys(ui, state);

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
                    components::scrolling(ui, "navigator", "Project panel contents", |ui| {
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
                    components::scrolling(ui, "inspector", "Inspector contents", |ui| {
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
