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
use crate::ui::palette::{self, PaletteEntry, PaletteState};
use crate::ui::showcase::{self, ShowcaseState};
use crate::ui::theme::{self, Palette, TextRole};
use crate::ui::workspace::{self, Drawer, INSPECTOR, LayoutMode, NAVIGATOR, PanelPreferences};

const CLASS_WIDTH: f32 = 180.0;
const HEADER_HEIGHT: f32 = 32.0;
const ATTRIBUTE_HEIGHT: f32 = 20.0;
const TEXT_INSET: f32 = 10.0;

/// Document colors for diagram objects. They do not follow the editor theme,
/// so switching themes never changes how a diagram looks when exported.
const DOCUMENT_FILL: Color32 = Color32::from_rgb(0xFA, 0xFC, 0xFF);
const DOCUMENT_INK: Color32 = Color32::from_rgb(0x22, 0x31, 0x47);

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
    /// Index into [`Self::classes`] of the selected class, if any.
    pub selected: Option<usize>,
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
    /// The component showcase window, while it is open.
    pub showcase: Option<ShowcaseState>,
    /// Widget to focus when the showcase closes.
    showcase_return: Option<Id>,
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
            theme: ThemePreference::Light,
            palette: None,
            panels: PanelPreferences::default(),
            drawer: None,
            layout: LayoutMode::Docked,
            drawer_return: None,
            showcase: None,
            showcase_return: None,
            pending_focus: None,
            applied_theme: None,
            text_had_focus: false,
        }
    }
}

impl ShellState {
    /// The currently selected class, if the selection index is valid.
    pub fn selected_class(&self) -> Option<&PreviewClass> {
        self.selected.and_then(|index| self.classes.get(index))
    }

    /// Selects the class at `index`; out-of-range indices clear the selection.
    pub fn select(&mut self, index: usize) {
        self.selected = (index < self.classes.len()).then_some(index);
    }

    /// Clears the selection.
    pub fn clear_selection(&mut self) {
        self.selected = None;
    }

    /// The state that decides which commands are available.
    pub fn command_context(&self) -> CommandContext {
        CommandContext {
            has_selection: self.selected_class().is_some(),
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
        close_showcase(state);
    }
    if state.drawer.is_some()
        && !text_had_focus
        && ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape))
    {
        close_drawer(state);
    }
    if let Some(id) = ui.input_mut(|input| commands::take_shortcut(input, text_had_focus)) {
        run(state, ui.ctx(), id);
    }

    let bar = Frame::side_top_panel(ui.style()).fill(palette.workspace);
    egui::Panel::top("header")
        .frame(bar)
        .exact_size(workspace::HEADER_HEIGHT)
        .show(ui, |ui| header(ui, state));
    egui::Panel::bottom("status")
        .frame(bar)
        .show(ui, |ui| status_bar(ui, state));
    if state.layout == LayoutMode::Docked {
        if state.panels.navigator_visible {
            egui::Panel::left("navigator")
                .resizable(true)
                .default_size(NAVIGATOR.default)
                .size_range(NAVIGATOR.min..=NAVIGATOR.max)
                .show(ui, |ui| scrolling(ui, "navigator", |ui| sidebar(ui, state)));
        }
        if state.panels.inspector_visible {
            egui::Panel::right("inspector")
                .resizable(true)
                .default_size(INSPECTOR.default)
                .size_range(INSPECTOR.min..=INSPECTOR.max)
                .show(ui, |ui| {
                    scrolling(ui, "inspector", |ui| inspector(ui, state))
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
    // Only a focused text field keeps editing keys; a focused button must not
    // swallow Escape (egui_wants_keyboard_input is true for any focus).
    state.text_had_focus = ui.ctx().text_edit_focused();
}

/// A command button with its shortcut as a hover hint; disabled commands
/// explain why.
fn command_button(
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
                |ui| components::truncated_text(ui, "Sample project · not saved"),
            );
        });
    });
    if let Some(id) = clicked {
        run(state, ui.ctx(), id);
    }
}

/// Diagram tabs. The tab strip itself (overflow, close, per-tab state) is UI-05.
fn tab_row(ui: &mut egui::Ui, state: &mut ShellState) {
    let context = state.command_context();
    let mut clicked = None;
    ui.horizontal_centered(|ui| {
        let _ = components::choice(ui, true, "Class diagram 1");
        for spec in commands::placed(Placement::TabRow) {
            command_button(ui, spec, context, &mut clicked);
        }
    });
    if let Some(id) = clicked {
        run(state, ui.ctx(), id);
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
        for spec in commands::placed(Placement::ToolRow) {
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
        run(state, ui.ctx(), id);
    }
    if let Some((id, button)) = toggle {
        run(state, ui.ctx(), id);
        // A drawer opened from its button returns focus to that button.
        if state.drawer.is_some() {
            state.drawer_return = Some(button);
        }
    }
}

/// Wraps panel contents in a vertical scroll area.
fn scrolling(ui: &mut egui::Ui, id: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    ScrollArea::vertical()
        .id_salt(id)
        .auto_shrink([false, false])
        .show(ui, add_contents);
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
                scrolling(ui, "drawer", |ui| match which {
                    Drawer::Navigator => sidebar(ui, state),
                    Drawer::Inspector => inspector(ui, state),
                });
            });
        });
    if close {
        close_drawer(state);
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
            run(state, ctx, id);
            true
        }
        Some(PaletteEntry::Object(index)) => {
            state.select(index);
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

fn status_bar(ui: &mut egui::Ui, state: &mut ShellState) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Prototype · nothing is saved")
                .text_style(TextRole::Label.text_style()),
        );
        ui.separator();
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            // Right-to-left layout: add the options in reverse visual order.
            for (preference, label) in [
                (ThemePreference::System, "System"),
                (ThemePreference::Dark, "Dark"),
                (ThemePreference::Light, "Light"),
            ] {
                if components::choice(ui, state.theme == preference, label).clicked() {
                    state.theme = preference;
                }
            }
            components::field_label(ui, "Theme:");
            ui.separator();
            let selection = match state.selected_class() {
                Some(class) => format!("Selected: {}", class.display_name()),
                None => String::from("No selection"),
            };
            components::truncated_text(ui, &selection);
        });
    });
}

fn sidebar(ui: &mut egui::Ui, state: &mut ShellState) {
    components::panel_title(ui, "Project");
    components::field_label(ui, "Diagrams");
    ui.indent("diagrams", |ui| {
        let _ = components::list_row(ui, true, "Class diagram 1");
    });
    ui.add_space(theme::SPACING);
    components::field_label(ui, "Model");
    ui.indent("model", |ui| {
        for index in 0..state.classes.len() {
            let selected = state.selected == Some(index);
            let label = state.classes[index].display_name().to_owned();
            if components::list_row(ui, selected, &label).clicked() {
                state.select(index);
            }
        }
    });
}

fn inspector(ui: &mut egui::Ui, state: &mut ShellState) {
    components::panel_title(ui, "Selection");
    let Some(class) = state
        .selected
        .and_then(|index| state.classes.get_mut(index))
    else {
        ui.label("Nothing selected. Select a class on the canvas or in the Model list.");
        return;
    };

    components::text_field(ui, "Name", &mut class.name);
    if class.name.trim().is_empty() {
        components::warning_text(ui, BLANK_NAME_WARNING);
    }

    ui.add_space(theme::SPACING);
    components::field_label(ui, "Attributes");
    for attribute in &class.attributes {
        components::code_text(ui, attribute);
    }
}

fn canvas(ui: &mut egui::Ui, state: &mut ShellState, palette: &Palette) -> Rect {
    let (background, painter) = ui.allocate_painter(ui.available_size(), Sense::click());
    background.widget_info(|| WidgetInfo::labeled(Role::Canvas, true, "Diagram canvas"));
    painter.rect_filled(background.rect, CornerRadius::ZERO, palette.canvas);
    let origin = background.rect.min;

    let mut clicked_class = None;
    for (index, class) in state.classes.iter().enumerate() {
        let rect = Rect::from_min_size(origin + class.position, class.size());
        let selected = state.selected == Some(index);
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

    match clicked_class {
        Some(index) => state.select(index),
        None if background.clicked() => state.clear_selection(),
        None => {}
    }
    background.rect
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

/// eframe application wrapper around [`ShellState`].
#[derive(Debug, Default)]
pub struct ShellApp {
    state: ShellState,
}

impl eframe::App for ShellApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        show(ui, &mut self.state);
    }
}
