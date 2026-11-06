//! Native shell prototype: toolbar, sidebar, canvas, inspector and status bar.
//!
//! The preview classes are prototype data owned by the desktop crate. They
//! exercise layout, selection and real text editing before M1 introduces the
//! core model, commands and history.

use eframe::egui::{
    self, Align, Align2, Area, Button, Color32, CornerRadius, Frame, Id, Key, Layout, Modal,
    Modifiers, Order, Pos2, Rect, Role, ScrollArea, Sense, Stroke, StrokeKind, ThemePreference,
    Vec2, WidgetInfo,
};

use crate::ui::commands::{self, Availability, CommandContext, CommandId, CommandSpec, Placement};
use crate::ui::components;
use crate::ui::entry::entry_screens;
use crate::ui::inspector::{self as rules, DraftKey, DraftOutcome, PreviewRelation, RelationField};
use crate::ui::intents::UiIntent;
use crate::ui::navigator::{self, Diagram, OpenTab};
use crate::ui::notation::{self, InspectorSection, NotationId};
use crate::ui::palette::{self, PaletteEntry, PaletteState};
use crate::ui::showcase::{self, ShowcaseState};
use crate::ui::status::status_bar;
use crate::ui::tabs::tab_row;
use crate::ui::theme::{self, Palette, TextRole};
use crate::ui::welcome::EntryState;
use crate::ui::workspace::{self, Drawer, INSPECTOR, LayoutMode, NAVIGATOR, PanelPreferences};

const CLASS_WIDTH: f32 = 180.0;
const HEADER_HEIGHT: f32 = 32.0;
const ATTRIBUTE_HEIGHT: f32 = 20.0;
const TEXT_INSET: f32 = 10.0;

/// Document colors for diagram objects. They do not follow the editor theme,
/// so switching themes never changes how a diagram looks when exported.
const DOCUMENT_FILL: Color32 = Color32::from_rgb(0xFA, 0xFC, 0xFF);
const DOCUMENT_INK: Color32 = Color32::from_rgb(0x22, 0x31, 0x47);

/// Name of the diagram every new project starts with.
pub const FIRST_DIAGRAM: &str = "Class diagram 1";

/// Name of the built-in sample project.
pub const SAMPLE_PROJECT: &str = "Sample project";

/// Message shown when a class name has no visible characters.
pub const BLANK_NAME_WARNING: &str = "A class name must contain a non-whitespace character.";

/// A class box drawn on the prototype canvas.
#[derive(Debug, Clone, PartialEq)]
pub struct PreviewClass {
    /// Editable class name, bound directly to the inspector text field.
    pub name: String,
    /// Attribute lines displayed below the name.
    pub attributes: Vec<String>,
    /// Top-left corner, relative to the canvas origin.
    pub position: Vec2,
}

impl PreviewClass {
    /// Name used in the canvas and accessibility tree; blank names get a placeholder.
    pub fn display_name(&self) -> &str {
        if self.name.trim().is_empty() {
            "Unnamed class"
        } else {
            &self.name
        }
    }

    fn size(&self) -> Vec2 {
        let rows = self.attributes.len().max(1) as f32;
        Vec2::new(CLASS_WIDTH, HEADER_HEIGHT + rows * ATTRIBUTE_HEIGHT + 8.0)
    }
}

/// Editor state for the prototype shell.
#[derive(Debug, Clone, PartialEq)]
pub struct ShellState {
    /// Classes shown on the canvas, in drawing order.
    pub classes: Vec<PreviewClass>,
    /// Index into [`Self::classes`] of the selected class, if any (in the active tab).
    pub selected: Option<usize>,
    /// Further classes selected with Ctrl+click (multi-selection).
    pub also_selected: Vec<usize>,
    /// Relationships between classes.
    pub relations: Vec<PreviewRelation>,
    /// Index into [`Self::relations`] of the selected relationship, if any.
    pub selected_relation: Option<usize>,
    /// Uncommitted inspector drafts; widgets edit these, never the records.
    pub drafts: std::collections::BTreeMap<DraftKey, String>,
    /// Whether the inspector's Appearance and Advanced sections are open.
    pub inspector_sections: (bool, bool),
    /// Diagrams in the project; each shows some of [`Self::classes`].
    pub diagrams: Vec<Diagram>,
    /// Open diagram tabs, in order.
    pub tabs: Vec<OpenTab>,
    /// Index into [`Self::tabs`] of the active tab, if any tab is open.
    pub active_tab: Option<usize>,
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
    /// Name of the open project, shown in the header.
    pub project_name: String,
    /// The component showcase window, while it is open.
    pub showcase: Option<ShowcaseState>,
    /// Widget to focus when the showcase closes.
    showcase_return: Option<Id>,
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
    fn default() -> Self {
        let class = |name: &str, attributes: &[&str], x, y| PreviewClass {
            name: name.to_owned(),
            attributes: attributes.iter().map(|a| (*a).to_owned()).collect(),
            position: Vec2::new(x, y),
        };
        Self {
            classes: vec![
                class("Customer", &["id: CustomerId", "name: String"], 48.0, 48.0),
                class("Order", &["id: OrderId", "total: Money"], 300.0, 96.0),
            ],
            selected: None,
            also_selected: Vec::new(),
            relations: vec![rules::sample_relation()],
            selected_relation: None,
            drafts: std::collections::BTreeMap::new(),
            inspector_sections: (false, false),
            diagrams: vec![
                Diagram::new(FIRST_DIAGRAM, &[0, 1]),
                Diagram::new("Order details", &[1]),
            ],
            tabs: vec![OpenTab {
                diagram: 0,
                selected: None,
            }],
            active_tab: Some(0),
            navigator_search: String::new(),
            theme: ThemePreference::Light,
            palette: None,
            panels: PanelPreferences::default(),
            drawer: None,
            layout: LayoutMode::Docked,
            drawer_return: None,
            entry: None,
            project_name: String::from(SAMPLE_PROJECT),
            showcase: None,
            showcase_return: None,
            intents: Vec::new(),
            pending_focus: None,
            applied_theme: None,
            text_had_focus: false,
        }
    }
}

impl ShellState {
    /// State at application start: the welcome screen.
    pub fn welcome() -> Self {
        Self {
            entry: Some(EntryState::default()),
            ..Self::default()
        }
    }

    /// The currently selected class, if the selection index is valid.
    pub fn selected_class(&self) -> Option<&PreviewClass> {
        self.selected.and_then(|index| self.classes.get(index))
    }

    /// Selects only the class at `index`; out-of-range indices clear the selection.
    pub fn select(&mut self, index: usize) {
        self.clear_selection();
        self.selected = (index < self.classes.len()).then_some(index);
    }

    /// Clears the whole selection: classes and relationship.
    pub fn clear_selection(&mut self) {
        self.selected = None;
        self.also_selected.clear();
        self.selected_relation = None;
    }

    /// Every selected class, the primary one first.
    pub fn selected_classes(&self) -> Vec<usize> {
        self.selected
            .into_iter()
            .chain(self.also_selected.iter().copied())
            .collect()
    }

    /// Adds the class to the selection, or removes it if already selected.
    pub fn toggle_in_selection(&mut self, index: usize) {
        if index >= self.classes.len() {
            return;
        }
        self.selected_relation = None;
        if self.selected == Some(index) {
            self.selected = (!self.also_selected.is_empty()).then(|| self.also_selected.remove(0));
        } else if let Some(position) = self.also_selected.iter().position(|&i| i == index) {
            self.also_selected.remove(position);
        } else if self.selected.is_none() {
            self.selected = Some(index);
        } else {
            self.also_selected.push(index);
        }
    }

    /// Selects only the relationship at `index`.
    pub fn select_relation(&mut self, index: usize) {
        self.clear_selection();
        self.selected_relation = (index < self.relations.len()).then_some(index);
    }

    /// The diagram in the active tab, if a tab is open.
    pub fn active_diagram(&self) -> Option<usize> {
        self.active_tab
            .and_then(|tab| self.tabs.get(tab))
            .map(|tab| tab.diagram)
    }

    /// Notation of the active diagram; Class when no diagram is open.
    pub fn active_notation(&self) -> NotationId {
        self.active_diagram()
            .and_then(|diagram| self.diagrams.get(diagram))
            .map_or(NotationId::Class, |diagram| diagram.notation)
    }

    /// Classes shown on the canvas: those in the active diagram.
    pub fn visible_classes(&self) -> Vec<usize> {
        self.active_diagram()
            .and_then(|diagram| self.diagrams.get(diagram))
            .map(|diagram| diagram.classes.clone())
            .unwrap_or_default()
    }

    /// Makes tab `index` active. The leaving tab keeps its selection and the
    /// arriving tab restores its own.
    pub fn activate_tab(&mut self, index: usize) {
        if index >= self.tabs.len() || self.active_tab == Some(index) {
            return;
        }
        if let Some(current) = self.active_tab.and_then(|tab| self.tabs.get_mut(tab)) {
            current.selected = self.selected;
        }
        self.active_tab = Some(index);
        let restored = self.tabs[index].selected;
        self.clear_selection();
        self.selected = restored;
    }

    /// Shows `diagram` in its tab, opening a tab if it has none.
    pub fn open_diagram(&mut self, diagram: usize) {
        if diagram >= self.diagrams.len() {
            return;
        }
        match self.tabs.iter().position(|tab| tab.diagram == diagram) {
            Some(index) => self.activate_tab(index),
            None => {
                self.tabs.push(OpenTab {
                    diagram,
                    selected: None,
                });
                self.activate_tab(self.tabs.len() - 1);
            }
        }
    }

    /// Closes tab `index`. Model elements are untouched; a neighbouring tab
    /// becomes active when the active one closes.
    pub fn close_tab(&mut self, index: usize) {
        if index >= self.tabs.len() {
            return;
        }
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
        self.selected = self
            .active_tab
            .and_then(|tab| self.tabs.get(tab))
            .and_then(|tab| tab.selected);
    }

    /// Selects a class from the project panel. If the active diagram does not
    /// show it, a diagram that does is opened first ("reveal").
    pub fn reveal_class(&mut self, class: usize) {
        if class >= self.classes.len() {
            return;
        }
        if !self.visible_classes().contains(&class)
            && let Some(&diagram) = navigator::diagrams_showing(&self.diagrams, class).first()
        {
            self.open_diagram(diagram);
        }
        self.select(class);
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
            has_selection: self.selected_class().is_some() || self.selected_relation.is_some(),
        }
    }
}

/// The coordinator: applies queued intents in order, after drawing.
fn apply_intents(state: &mut ShellState, ctx: &egui::Context) {
    for intent in std::mem::take(&mut state.intents) {
        match intent {
            UiIntent::Run { command, origin } => {
                let drawer_before = state.drawer;
                let showcase_before = state.showcase.is_some();
                run(state, ctx, command);
                // Whatever this control opened returns focus to it.
                if let Some(origin) = origin {
                    if state.drawer.is_some() && state.drawer != drawer_before {
                        state.drawer_return = Some(origin);
                    }
                    if state.showcase.is_some() && !showcase_before {
                        state.showcase_return = Some(origin);
                    }
                }
            }
            UiIntent::SelectClass(index) => state.select(index),
            UiIntent::ToggleClassInSelection(index) => state.toggle_in_selection(index),
            UiIntent::SelectRelation(index) => state.select_relation(index),
            UiIntent::RenameClass(index, name) => {
                if rules::class_name_error(&name).is_none()
                    && let Some(class) = state.classes.get_mut(index)
                {
                    class.name = name;
                }
            }
            UiIntent::SetRelationField(index, field, value) => {
                if field.error(&value).is_none()
                    && let Some(relation) = state.relations.get_mut(index)
                {
                    *field.value_mut(relation) = value.trim().to_owned();
                }
            }
            UiIntent::RevealClass(index) => state.reveal_class(index),
            UiIntent::OpenDiagram(index) => state.open_diagram(index),
            UiIntent::ActivateTab(index) => state.activate_tab(index),
            UiIntent::CloseTab(index) => state.close_tab(index),
            UiIntent::SetTheme(preference) => state.theme = preference,
            UiIntent::CloseDrawer => close_drawer(state),
            UiIntent::CloseShowcase => close_showcase(state),
        }
    }
}

/// Runs `id` if it is available now; unavailable commands do nothing.
fn run(state: &mut ShellState, ctx: &egui::Context, id: CommandId) {
    if !commands::availability(id, state.command_context()).is_enabled() {
        return;
    }
    match id {
        CommandId::ClearSelection => state.clear_selection(),
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

/// "Customer places Order".
pub(crate) fn relation_label(state: &ShellState, relation: &PreviewRelation) -> String {
    let name = |class: usize| {
        state
            .classes
            .get(class)
            .map(PreviewClass::display_name)
            .unwrap_or("?")
    };
    format!(
        "{} {} {}",
        name(relation.source),
        relation.name,
        name(relation.target)
    )
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
    if state.showcase.is_some()
        && !text_had_focus
        && ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape))
    {
        state.queue(UiIntent::CloseShowcase);
    }
    if state.drawer.is_some()
        && !text_had_focus
        && ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape))
    {
        state.queue(UiIntent::CloseDrawer);
    }
    if let Some(id) = ui.input_mut(|input| commands::take_shortcut(input, text_had_focus)) {
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
fn command_menu(ui: &mut egui::Ui, context: CommandContext, clicked: &mut Option<CommandId>) {
    ui.menu_button("Menu", |ui| {
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
}

/// Header: menu, project name, then Undo, Redo, Search, Save and Export.
fn header(ui: &mut egui::Ui, state: &mut ShellState) {
    let context = state.command_context();
    let mut clicked = None;
    ui.horizontal_centered(|ui| {
        command_menu(ui, context, &mut clicked);
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
                    let title = format!("{} · not saved", state.project_name);
                    components::truncated_text(ui, &title)
                },
            );
        });
    });
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

/// Command and object search. Enter runs the highlighted result, arrows move
/// the highlight, Escape or a click outside closes it, and focus returns to
/// where it was.
fn command_palette(ctx: &egui::Context, state: &mut ShellState) {
    let Some(mut search) = state.palette.take() else {
        return;
    };
    let names: Vec<String> = state
        .classes
        .iter()
        .map(|class| class.display_name().to_owned())
        .collect();
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
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
        Some(PaletteEntry::Object(index)) => {
            state.queue(UiIntent::RevealClass(index));
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

/// Project panel: search, the project's diagrams (views) and its model
/// elements, with distinct "nothing yet" and "no match" states.
fn sidebar(ui: &mut egui::Ui, state: &mut ShellState) {
    components::panel_title(ui, "Project");
    components::text_field(ui, "Search project", &mut state.navigator_search);
    let query = state.navigator_search.clone();
    let searching = !query.trim().is_empty();

    ui.add_space(theme::SPACING);
    components::field_label(ui, "Diagrams (views of the model)");
    let diagram_hits: Vec<usize> = (0..state.diagrams.len())
        .filter(|&index| navigator::matches(&query, &state.diagrams[index].name))
        .collect();
    if state.diagrams.is_empty() {
        ui.label("No diagrams yet.");
    } else if diagram_hits.is_empty() {
        ui.label(format!("No diagrams match \"{}\".", query.trim()));
    }
    let active = state.active_diagram();
    let mut open = None;
    ui.indent("diagrams", |ui| {
        for index in diagram_hits {
            let name = &state.diagrams[index].name;
            let row = components::list_row(ui, active == Some(index), name)
                .on_hover_text("Open this diagram in a tab");
            if row.clicked() {
                open = Some(index);
            }
        }
    });
    if let Some(index) = open {
        state.queue(UiIntent::OpenDiagram(index));
    }

    ui.add_space(theme::SPACING);
    components::field_label(ui, "Model (shared by all diagrams)");
    let class_hits: Vec<usize> = (0..state.classes.len())
        .filter(|&index| navigator::matches(&query, state.classes[index].display_name()))
        .collect();
    if state.classes.is_empty() {
        ui.label("No classes yet. Classes appear here when you add them to a diagram.");
    } else if class_hits.is_empty() {
        ui.label(format!("No classes match \"{}\".", query.trim()));
    }
    let mut reveal = None;
    let mut toggle = None;
    let chosen = state.selected_classes();
    let adding = ui.input(|input| input.modifiers.command);
    ui.indent("model", |ui| {
        for index in class_hits {
            let label = state.classes[index].display_name().to_owned();
            let row =
                components::list_row(ui, chosen.contains(&index), &label).on_hover_text(format!(
                    "{}. Ctrl+click or Ctrl+Enter adds it to the selection.",
                    navigator::shown_in(&state.diagrams, index)
                ));
            if row.clicked() {
                if adding {
                    toggle = Some(index);
                } else {
                    reveal = Some(index);
                }
            }
        }
    });
    components::field_label(ui, "Relationships");
    let relation_hits: Vec<(usize, String)> = state
        .relations
        .iter()
        .enumerate()
        .map(|(index, relation)| (index, relation_label(state, relation)))
        .filter(|(_, label)| navigator::matches(&query, label))
        .collect();
    if state.relations.is_empty() {
        ui.label("No relationships yet.");
    } else if relation_hits.is_empty() {
        ui.label(format!("No relationships match \"{}\".", query.trim()));
    }
    let mut relation = None;
    ui.indent("relationships", |ui| {
        for (index, label) in relation_hits {
            if components::list_row(ui, state.selected_relation == Some(index), &label).clicked() {
                relation = Some(index);
            }
        }
    });
    if let Some(index) = toggle {
        state.queue(UiIntent::ToggleClassInSelection(index));
    }
    if let Some(index) = relation {
        state.queue(UiIntent::SelectRelation(index));
    }
    if searching {
        ui.add_space(theme::GAP);
        if components::action_button(ui, "Clear search", None, None).clicked() {
            state.navigator_search.clear();
        }
    }
    if let Some(index) = reveal {
        state.queue(UiIntent::RevealClass(index));
    }
}

/// A text field bound to a draft. Enter or moving focus away commits a valid
/// draft; Escape cancels it; an invalid draft stays with its error.
fn draft_field(
    ui: &mut egui::Ui,
    drafts: &mut std::collections::BTreeMap<DraftKey, String>,
    key: DraftKey,
    label: &str,
    committed: &str,
    error_of: impl Fn(&str) -> Option<String>,
) -> DraftOutcome {
    let editing = drafts.contains_key(&key);
    let mut text = drafts
        .get(&key)
        .cloned()
        .unwrap_or_else(|| committed.to_owned());
    let error = if editing { error_of(&text) } else { None };
    let response = match &error {
        Some(message) => components::text_field_with_error(ui, label, &mut text, message),
        None => components::text_field(ui, label, &mut text),
    };
    if response.changed() {
        drafts.insert(key, text);
    }
    if editing && response.has_focus() {
        components::field_label(ui, "Enter applies the change. Escape cancels it.");
    }
    if !response.lost_focus() {
        return DraftOutcome::Editing;
    }
    if ui.input(|input| input.key_pressed(Key::Escape)) {
        drafts.remove(&key);
        return DraftOutcome::Cancel;
    }
    match drafts.get(&key).cloned() {
        Some(draft) if error_of(&draft).is_none() => {
            drafts.remove(&key);
            if draft == committed {
                DraftOutcome::Editing
            } else {
                DraftOutcome::Commit(draft)
            }
        }
        _ => DraftOutcome::Editing,
    }
}

/// A disclosure button for a collapsed inspector section.
fn section_toggle(ui: &mut egui::Ui, open: &mut bool, title: &str) {
    if components::choice(ui, *open, title).clicked() {
        *open = !*open;
    }
}

/// The view-only sections shared by single and multiple selections.
fn appearance_and_advanced(ui: &mut egui::Ui, state: &mut ShellState, mixed: bool) {
    ui.add_space(theme::SPACING);
    let (mut appearance, mut advanced) = state.inspector_sections;
    section_toggle(ui, &mut appearance, "Appearance (this view only)");
    if appearance {
        let value = if mixed { "Mixed" } else { "Default" };
        components::field_label(ui, &format!("Fill and border: {value}"));
        let _ = components::action_button(
            ui,
            "Change appearance",
            Some("Not available yet: view styles arrive with the project model."),
            None,
        );
    }
    section_toggle(ui, &mut advanced, "Advanced");
    if advanced {
        components::field_label(
            ui,
            "Stereotypes, visibility and documentation arrive with the project model.",
        );
    }
    state.inspector_sections = (appearance, advanced);
}

/// Contextual inspector: nothing selected, one class, several classes, or a
/// relationship. Edits are drafts committed through intents.
fn inspector(ui: &mut egui::Ui, state: &mut ShellState) {
    components::panel_title(ui, "Selection");
    if let Some(relation) = state.selected_relation {
        relation_inspector(ui, state, relation);
        return;
    }
    let selected = state.selected_classes();
    match selected.as_slice() {
        [] => {
            ui.label("Nothing selected. Select a class on the canvas or in the Model list.");
            if let Some(diagram) = state.active_diagram().and_then(|d| state.diagrams.get(d)) {
                ui.add_space(theme::SPACING);
                components::field_label(ui, "Diagram");
                ui.label(&diagram.name);
                components::field_label(ui, &format!("{} classes shown", diagram.classes.len()));
            }
            components::field_label(ui, "Tip: Ctrl+click adds classes to the selection.");
        }
        [index] => class_inspector(ui, state, *index),
        several => {
            let names: Vec<&str> = several
                .iter()
                .filter_map(|&index| state.classes.get(index))
                .map(PreviewClass::display_name)
                .collect();
            components::field_label(ui, &format!("{} classes selected", several.len()));
            ui.label(format!("Names (mixed): {}", names.join(", ")));
            ui.label(
                "Select one class to rename it. Edits never apply to several classes silently.",
            );
            appearance_and_advanced(ui, state, true);
        }
    }
}

fn class_inspector(ui: &mut egui::Ui, state: &mut ShellState, index: usize) {
    let Some(class) = state.classes.get(index).cloned() else {
        return;
    };
    components::field_label(ui, "Model: changes apply in every diagram");
    let outcome = draft_field(
        ui,
        &mut state.drafts,
        DraftKey::ClassName(index),
        "Name",
        &class.name,
        rules::class_name_error,
    );
    if let DraftOutcome::Commit(name) = outcome {
        state.queue(UiIntent::RenameClass(index, name));
    }
    if class.name.trim().is_empty() {
        components::warning_text(ui, BLANK_NAME_WARNING);
    }
    // Model versus view: renaming changes the class in every diagram.
    components::field_label(ui, &navigator::shown_in(&state.diagrams, index));

    // Sections come from the active diagram's notation contract.
    let sections = notation::notation(state.active_notation()).element_sections;
    if sections.contains(&InspectorSection::Members) {
        members(ui, &class);
    }
    if sections.contains(&InspectorSection::Appearance)
        || sections.contains(&InspectorSection::Advanced)
    {
        appearance_and_advanced(ui, state, false);
    }
}

/// Attributes and operations of a class, with the unavailable Add buttons.
fn members(ui: &mut egui::Ui, class: &PreviewClass) {
    ui.add_space(theme::SPACING);
    components::field_label(ui, "Attributes");
    if class.attributes.is_empty() {
        ui.label("No attributes.");
    }
    for attribute in &class.attributes {
        components::code_text(ui, attribute);
    }
    components::field_label(ui, "Operations");
    ui.label("No operations.");
    ui.horizontal_wrapped(|ui| {
        for label in ["Add attribute", "Add operation"] {
            let _ = components::action_button(
                ui,
                label,
                Some("Not available yet: members need the project model."),
                None,
            );
        }
    });
}

fn relation_inspector(ui: &mut egui::Ui, state: &mut ShellState, index: usize) {
    let Some(relation) = state.relations.get(index).cloned() else {
        return;
    };
    let name_of = |class: usize| {
        state
            .classes
            .get(class)
            .map(|class| class.display_name().to_owned())
            .unwrap_or_default()
    };
    let (source, target) = (name_of(relation.source), name_of(relation.target));
    components::field_label(ui, "Relationship (model)");
    ui.label(format!("From {source} to {target}"));
    let fields = [
        (RelationField::Name, String::from("Relationship name")),
        (RelationField::SourceRole, format!("Role at {source}")),
        (
            RelationField::SourceMultiplicity,
            format!("Multiplicity at {source}"),
        ),
        (RelationField::TargetRole, format!("Role at {target}")),
        (
            RelationField::TargetMultiplicity,
            format!("Multiplicity at {target}"),
        ),
    ];
    for (field, label) in fields {
        let outcome = draft_field(
            ui,
            &mut state.drafts,
            DraftKey::Relation(index, field),
            &label,
            field.value(&relation),
            |value| field.error(value),
        );
        if let DraftOutcome::Commit(value) = outcome {
            state.queue(UiIntent::SetRelationField(index, field, value));
        }
    }
    appearance_and_advanced(ui, state, false);
}

fn canvas(ui: &mut egui::Ui, state: &mut ShellState, palette: &Palette) -> Rect {
    let (background, painter) = ui.allocate_painter(ui.available_size(), Sense::click());
    background.widget_info(|| WidgetInfo::labeled(Role::Canvas, true, "Diagram canvas"));
    painter.rect_filled(background.rect, CornerRadius::ZERO, palette.canvas);
    let empty_message = match state.active_diagram() {
        None => Some("No diagram is open. Open one from the project panel."),
        Some(_) if state.visible_classes().is_empty() => {
            Some("This diagram is empty. Adding classes arrives with the project model.")
        }
        Some(_) => None,
    };
    if let Some(message) = empty_message {
        painter.text(
            background.rect.center(),
            Align2::CENTER_CENTER,
            message,
            TextRole::Body.font_id(),
            palette.text_secondary,
        );
    }
    let origin = background.rect.min;

    let mut clicked_class = None;
    let visible = state.visible_classes();
    let chosen = state.selected_classes();
    let rect_of = |class: &PreviewClass| Rect::from_min_size(origin + class.position, class.size());
    // Relationship lines between visible classes, under the boxes.
    for (index, relation) in state.relations.iter().enumerate() {
        let (Some(source), Some(target)) = (
            state.classes.get(relation.source),
            state.classes.get(relation.target),
        ) else {
            continue;
        };
        if !visible.contains(&relation.source) || !visible.contains(&relation.target) {
            continue;
        }
        paint_relation(
            &painter,
            relation,
            rect_of(source),
            rect_of(target),
            state.selected_relation == Some(index),
            palette,
        );
    }
    for (index, class) in state
        .classes
        .iter()
        .enumerate()
        .filter(|(index, _)| visible.contains(index))
    {
        let rect = rect_of(class);
        let selected = chosen.contains(&index);
        let response = ui.interact(
            rect,
            ui.scope_id().with(("canvas-class", index)),
            Sense::click(),
        );
        let name = class.display_name();
        response.widget_info(|| {
            WidgetInfo::selected(Role::Button, true, selected, format!("Class {name}"))
        });
        if response.clicked() {
            clicked_class = Some(index);
        }
        paint_class(&painter, rect, class, selected, response.hovered(), palette);
    }

    let adding = ui.input(|input| input.modifiers.command);
    match clicked_class {
        Some(index) if adding => state.queue(UiIntent::ToggleClassInSelection(index)),
        Some(index) => state.queue(UiIntent::SelectClass(index)),
        None if background.clicked() => state.queue(UiIntent::run(CommandId::ClearSelection)),
        None => {}
    }
    background.rect
}

/// A relationship as a line between box edges, with its name in the middle
/// and multiplicities near each end. Selected relationships are thicker and
/// use the accent colour; they are selected from the project panel.
fn paint_relation(
    painter: &egui::Painter,
    relation: &PreviewRelation,
    source: Rect,
    target: Rect,
    selected: bool,
    palette: &Palette,
) {
    let (from, to) = (source.center(), target.center());
    let edge = |rect: Rect, toward: Pos2| {
        let direction = (toward - rect.center()).normalized();
        let scale_x = if direction.x.abs() > f32::EPSILON {
            rect.width() / 2.0 / direction.x.abs()
        } else {
            f32::INFINITY
        };
        let scale_y = if direction.y.abs() > f32::EPSILON {
            rect.height() / 2.0 / direction.y.abs()
        } else {
            f32::INFINITY
        };
        rect.center() + direction * scale_x.min(scale_y)
    };
    let (start, end) = (edge(source, to), edge(target, from));
    let (color, width) = if selected {
        (palette.accent, 2.5)
    } else {
        (DOCUMENT_INK, 1.0)
    };
    painter.line_segment([start, end], Stroke::new(width, color));
    // Name above the middle, multiplicities below the line near each end, so
    // the three labels do not collide even on short lines.
    let direction = (end - start).normalized();
    let normal = Vec2::new(-direction.y, direction.x);
    let above = if normal.y > 0.0 { -normal } else { normal };
    let font = TextRole::Code.font_id();
    let middle = start + (end - start) / 2.0;
    painter.text(
        middle + above * 4.0,
        Align2::CENTER_BOTTOM,
        &relation.name,
        font.clone(),
        color,
    );
    let below = -above;
    painter.text(
        start + direction * 10.0 + below * 4.0,
        Align2::LEFT_TOP,
        &relation.source_multiplicity,
        font.clone(),
        color,
    );
    painter.text(
        end - direction * 10.0 + below * 4.0,
        Align2::RIGHT_TOP,
        &relation.target_multiplicity,
        font,
        color,
    );
}

fn paint_class(
    painter: &egui::Painter,
    rect: Rect,
    class: &PreviewClass,
    selected: bool,
    hovered: bool,
    palette: &Palette,
) {
    // Selection and hover are editor overlays in the theme accent; the box
    // itself keeps its document colors.
    let border = if selected || hovered {
        palette.accent
    } else {
        DOCUMENT_INK
    };
    let width = if selected { 2.5 } else { 1.0 };

    painter.rect(
        rect,
        CornerRadius::same(4),
        DOCUMENT_FILL,
        Stroke::new(width, border),
        StrokeKind::Inside,
    );
    let header_bottom = rect.top() + HEADER_HEIGHT;
    painter.hline(
        rect.x_range(),
        header_bottom,
        Stroke::new(1.0, DOCUMENT_INK),
    );
    // Text stays inside the box; how boxes grow for long names is decided
    // with the canvas states in UI-07.
    let text_width = rect.width() - 2.0 * TEXT_INSET;
    let name = components::one_line_galley(
        painter,
        class.display_name(),
        TextRole::Body.font_id(),
        DOCUMENT_INK,
        text_width,
    );
    let name_origin = Align2::CENTER_CENTER
        .anchor_size(
            Pos2::new(rect.center().x, rect.top() + HEADER_HEIGHT / 2.0),
            name.size(),
        )
        .min;
    painter.galley(name_origin, name, DOCUMENT_INK);
    for (row, attribute) in class.attributes.iter().enumerate() {
        let line = components::one_line_galley(
            painter,
            attribute,
            TextRole::Code.font_id(),
            DOCUMENT_INK,
            text_width,
        );
        let top = header_bottom + 6.0 + row as f32 * ATTRIBUTE_HEIGHT;
        painter.galley(Pos2::new(rect.left() + TEXT_INSET, top), line, DOCUMENT_INK);
    }

    if selected {
        // Corner handles mark selection without relying on colour alone.
        for corner in [
            rect.left_top(),
            rect.right_top(),
            rect.left_bottom(),
            rect.right_bottom(),
        ] {
            let handle = Rect::from_center_size(corner, Vec2::splat(8.0));
            painter.rect(
                handle,
                CornerRadius::ZERO,
                Color32::WHITE,
                Stroke::new(1.5, palette.accent),
                StrokeKind::Middle,
            );
        }
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
