//! Native shell prototype: toolbar, sidebar, canvas, inspector and status bar.
//!
//! The preview classes are prototype data owned by the desktop crate. They
//! exercise layout, selection and real text editing before M1 introduces the
//! core model, commands and history.

use eframe::egui::{
    self, Align, Area, Button, Frame, Id, Key, Layout, Modal, Modifiers, Order, Pos2, Rect, Role,
    ScrollArea, Stroke, ThemePreference, Vec2,
};

use crate::ui::canvas::{ATTRIBUTE_HEIGHT, CLASS_WIDTH, HEADER_HEIGHT, canvas};
use crate::ui::commands::{self, Availability, CommandContext, CommandId, CommandSpec, Placement};
use crate::ui::components;
use crate::ui::entry::entry_screens;
use crate::ui::inspector::{self as rules, DraftKey, PreviewRelation};
use crate::ui::inspector_view::inspector;
use crate::ui::intents::UiIntent;
use crate::ui::navigator::{self, Diagram, OpenTab};
use crate::ui::notation::{self, NotationId};
use crate::ui::palette::{self, PaletteEntry, PaletteState};
use crate::ui::project_panel::sidebar;
use crate::ui::showcase::{self, ShowcaseState};
use crate::ui::status::status_bar;
use crate::ui::tabs::tab_row;
use crate::ui::theme::{self, Palette};
use crate::ui::welcome::EntryState;
use crate::ui::workspace::{self, Drawer, INSPECTOR, LayoutMode, NAVIGATOR, PanelPreferences};

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

    pub(crate) fn size(&self) -> Vec2 {
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
