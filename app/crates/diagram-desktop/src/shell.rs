//! Native shell: header, tabs, tool row, project panel, canvas, inspector and
//! status bar.
//!
//! The shell lays out views drawn from [`presentation::build`]. Views push
//! [`UiIntent`]s, and [`dispatch::apply`] is the only code that changes the
//! state. The preview classes are prototype data until M1 binds the core
//! model, commands and history.

use eframe::egui::{
    self, Align, Align2, Area, Button, Color32, CornerRadius, Frame, Id, Key, Layout, Modal,
    Modifiers, Order, Pos2, Rect, Role, ScrollArea, Sense, Stroke, StrokeKind, ThemePreference,
    Vec2, WidgetInfo,
};

pub use crate::app_state::{
    AppState, BLANK_NAME_WARNING, PreviewClass, SAMPLE_PROJECT, ShellState,
};

use crate::app_state::{ATTRIBUTE_HEIGHT, CLASS_HEADER_HEIGHT};
use crate::dispatch;
use crate::intent::{
    CanvasPointer, DraftField, InputContext, PointerKind, SearchScope, Surface, Target, UiIntent,
};
use crate::presentation::{
    self, CanvasView, CommandView, HeaderView, InspectorKind, InspectorView, NavigatorView,
    NodeView, Presentation, StatusView, TabsView, ToolsView,
};
use crate::ui::commands::{self, Availability};
use crate::ui::components;
use crate::ui::palette::{self, PaletteEntry};
use crate::ui::showcase;
use crate::ui::theme::{self, Palette, TextRole};
use crate::ui::welcome::{self, EntryAction, EntryScreen};
use crate::ui::workspace::{self, Drawer, INSPECTOR, LayoutMode, NAVIGATOR};

const TEXT_INSET: f32 = 10.0;

/// Document colors for diagram objects. They do not follow the editor theme,
/// so switching themes never changes how a diagram looks when exported.
const DOCUMENT_FILL: Color32 = Color32::from_rgb(0xFA, 0xFC, 0xFF);
const DOCUMENT_INK: Color32 = Color32::from_rgb(0x22, 0x31, 0x47);

/// This frame's UI facts, with `opener` as the widget focus returns to.
fn input_context(ctx: &egui::Context, state: &AppState, opener: Option<Id>) -> InputContext {
    InputContext {
        text_had_focus: state.text_had_focus,
        modal_open: ctx.memory(|memory| memory.top_modal_layer()).is_some(),
        opener: opener.or_else(|| ctx.memory(|memory| memory.focused())),
        ..InputContext::default()
    }
}

/// Applies `intents` now and refreshes `view` so the rest of the frame draws
/// the new state.
fn commit(
    ctx: &egui::Context,
    state: &mut AppState,
    view: &mut Presentation,
    intents: Vec<UiIntent>,
    opener: Option<Id>,
) {
    if intents.is_empty() {
        return;
    }
    let input = input_context(ctx, state, opener);
    dispatch::apply(state, &input, intents);
    *view = presentation::build(state);
}

/// The component showcase as a movable window with an explicit Close button.
fn showcase_window(ctx: &egui::Context, state: &mut AppState) {
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
            // The fixture's controls are local widget state of a design fixture.
            ScrollArea::vertical().show(ui, |ui| showcase::show(ui, &mut fixture));
        });
    state.showcase = Some(fixture);
    if close {
        let input = input_context(ctx, state, None);
        dispatch::apply(state, &input, [UiIntent::Close(Surface::Showcase)]);
    }
}

/// Welcome and new-project screens, shown instead of the workspace.
fn entry_screens(ui: &mut egui::Ui, state: &mut AppState, palette: &Palette) {
    let Some(mut entry) = state.entry.take() else {
        return;
    };
    let view = presentation::build(state);
    // Escape leaves the name field first, then cancels the form.
    let cancel_by_escape = entry.screen == EntryScreen::NewProject
        && !state.text_had_focus
        && ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape));

    let bar = Frame::side_top_panel(ui.style()).fill(palette.workspace);
    egui::Panel::top("header")
        .frame(bar)
        .exact_size(workspace::HEADER_HEIGHT)
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| ui.strong(crate::APPLICATION_NAME));
        });
    let mut intents = egui::Panel::bottom("status")
        .frame(bar)
        .show(ui, |ui| status_bar(ui, &view.status, false))
        .inner;
    let action = egui::CentralPanel::default()
        .frame(
            Frame::new()
                .fill(palette.panel)
                .inner_margin(2.0 * theme::SPACING),
        )
        .show(ui, |ui| {
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_max_width(560.0);
                    match entry.screen {
                        EntryScreen::Welcome => welcome::welcome(ui, &mut entry),
                        EntryScreen::NewProject => welcome::new_project(ui, &mut entry),
                    }
                })
                .inner
        })
        .inner;
    let action = if cancel_by_escape {
        Some(EntryAction::CancelNewProject)
    } else {
        action
    };

    // Moving between the entry screens is local form state; opening or
    // creating a project goes through the dispatcher.
    let mut keep_entry = true;
    match action {
        Some(EntryAction::StartNewProject) => {
            entry.screen = EntryScreen::NewProject;
            entry.new_name.clear();
            entry.name_touched = false;
            entry.focus_name = true;
        }
        Some(EntryAction::CancelNewProject) => {
            entry.screen = EntryScreen::Welcome;
            state.pending_focus = entry.new_button;
        }
        Some(EntryAction::CreateProject(name)) => {
            if welcome::name_error(&name).is_none() {
                intents.push(UiIntent::CreateProject { name });
                keep_entry = false;
            } else {
                entry.name_touched = true;
            }
        }
        Some(EntryAction::OpenSample) => {
            intents.push(UiIntent::OpenSampleProject);
            keep_entry = false;
        }
        Some(EntryAction::RemoveRecent(index)) => {
            if index < entry.recent.len() {
                entry.recent.remove(index);
            }
        }
        Some(EntryAction::DismissRecovery) => entry.recovery_visible = false,
        None => {}
    }
    if keep_entry {
        state.entry = Some(entry);
    }
    let input = input_context(ui.ctx(), state, None);
    dispatch::apply(state, &input, intents);
}

/// Draws the complete shell into `ui` and applies the user's interactions to `state`.
pub fn show(ui: &mut egui::Ui, state: &mut AppState) {
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
        state.text_had_focus = ui.ctx().text_edit_focused();
        return;
    }

    dispatch::set_layout(state, workspace::layout_mode(ui.max_rect().width()));
    let ctx = ui.ctx().clone();
    let mut view = presentation::build(state);

    // Shortcuts use the text focus of the previous frame: egui drops focus in
    // the frame Escape is pressed, and Escape must first leave the field.
    // An open drawer takes Escape before the canvas selection does.
    let text_had_focus = state.text_had_focus;
    let mut early = Vec::new();
    if state.showcase.is_some()
        && !text_had_focus
        && ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape))
    {
        early.push(UiIntent::Close(Surface::Showcase));
    }
    if state.drawer.is_some()
        && !text_had_focus
        && ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape))
    {
        early.push(UiIntent::Close(Surface::Drawer));
    }
    if let Some(id) = ui.input_mut(|input| commands::take_shortcut(input, text_had_focus)) {
        early.push(UiIntent::Command(id));
    }
    commit(&ctx, state, &mut view, early, None);

    let bar = Frame::side_top_panel(ui.style()).fill(palette.workspace);
    let intents = egui::Panel::top("header")
        .frame(bar)
        .exact_size(workspace::HEADER_HEIGHT)
        .show(ui, |ui| header(ui, &view.header))
        .inner;
    commit(&ctx, state, &mut view, intents, None);
    let intents = egui::Panel::bottom("status")
        .frame(bar)
        .show(ui, |ui| status_bar(ui, &view.status, true))
        .inner;
    commit(&ctx, state, &mut view, intents, None);
    if state.layout == LayoutMode::Docked {
        if state.panels.navigator_visible {
            let intents = egui::Panel::left("navigator")
                .resizable(true)
                .default_size(NAVIGATOR.default)
                .size_range(NAVIGATOR.min..=NAVIGATOR.max)
                .show(ui, |ui| {
                    scrolling(ui, "navigator", |ui| sidebar(ui, &view.navigator))
                })
                .inner;
            commit(&ctx, state, &mut view, intents, None);
        }
        if state.panels.inspector_visible {
            let intents = egui::Panel::right("inspector")
                .resizable(true)
                .default_size(INSPECTOR.default)
                .size_range(INSPECTOR.min..=INSPECTOR.max)
                .show(ui, |ui| {
                    scrolling(ui, "inspector", |ui| inspector(ui, &view.inspector))
                })
                .inner;
            commit(&ctx, state, &mut view, intents, None);
        }
    }
    let canvas_rect = egui::CentralPanel::default()
        .frame(Frame::new().fill(palette.workspace))
        .show(ui, |ui| {
            let row = Frame::side_top_panel(ui.style()).fill(palette.panel);
            let intents = egui::Panel::top("tabs")
                .frame(row)
                .exact_size(workspace::TAB_ROW_HEIGHT)
                .show(ui, |ui| tab_row(ui, &view.tabs))
                .inner;
            commit(&ctx, state, &mut view, intents, None);
            let (intents, opener) = egui::Panel::top("tools")
                .frame(row)
                .exact_size(workspace::TOOL_ROW_HEIGHT)
                .show(ui, |ui| tool_row(ui, &view.tools))
                .inner;
            commit(&ctx, state, &mut view, intents, opener);
            let (rect, intents) = Frame::new()
                .inner_margin(theme::SPACING)
                .show(ui, |ui| canvas(ui, &view.canvas, &palette))
                .inner;
            commit(&ctx, state, &mut view, intents, None);
            rect
        })
        .inner;
    if let Some(which) = state.drawer {
        let intents = drawer(&ctx, which, &view, canvas_rect, &palette);
        commit(&ctx, state, &mut view, intents, None);
    }
    showcase_window(&ctx, state);
    command_palette(&ctx, state);
    // Only a focused text field keeps editing keys; a focused button must not
    // swallow Escape (egui_wants_keyboard_input is true for any focus).
    state.text_had_focus = ctx.text_edit_focused();
}

/// A command button with its shortcut as a hover hint; disabled commands
/// explain why.
fn command_button(ui: &mut egui::Ui, command: &CommandView, out: &mut Vec<UiIntent>) {
    let hint = command
        .shortcut_text
        .as_ref()
        .map(|text| format!("Shortcut: {text}"));
    let reason = command.availability.reason();
    if components::action_button(ui, &command.label, reason, hint.as_deref()).clicked() {
        out.push(UiIntent::Command(command.id));
    }
}

/// The application menu: every catalog command, grouped, with shortcuts.
fn command_menu(ui: &mut egui::Ui, view: &HeaderView, out: &mut Vec<UiIntent>) {
    ui.menu_button("Menu", |ui| {
        for group in &view.menu {
            ui.menu_button(group.label, |ui| {
                for command in &group.commands {
                    let mut item = Button::new(command.label.as_str());
                    if let Some(text) = &command.shortcut_text {
                        item = item.shortcut_text(text.as_str());
                    }
                    let response = ui.add_enabled(command.is_enabled(), item);
                    let response = match command.availability.reason() {
                        Some(reason) => response.on_disabled_hover_text(reason),
                        None => response,
                    };
                    if response.clicked() {
                        out.push(UiIntent::Command(command.id));
                        ui.close();
                    }
                }
            });
        }
    });
}

/// Header: menu, project name, then Undo, Redo, Search, Save and Export.
fn header(ui: &mut egui::Ui, view: &HeaderView) -> Vec<UiIntent> {
    let mut out = Vec::new();
    ui.horizontal_centered(|ui| {
        command_menu(ui, view, &mut out);
        ui.separator();
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            // Right-to-left: add in reverse visual order, with a separator
            // between command groups.
            let mut previous_group = None;
            for command in view.commands.iter().rev() {
                let group = command.id.spec().group;
                if previous_group.is_some_and(|previous| previous != group) {
                    ui.separator();
                }
                previous_group = Some(group);
                command_button(ui, command, &mut out);
            }
            ui.separator();
            ui.allocate_ui_with_layout(
                ui.available_size(),
                Layout::left_to_right(Align::Center),
                |ui| components::truncated_text(ui, &view.title),
            );
        });
    });
    out
}

/// Diagram tabs. The tab strip itself (overflow, close, per-tab state) is UI-05.
fn tab_row(ui: &mut egui::Ui, view: &TabsView) -> Vec<UiIntent> {
    let mut out = Vec::new();
    ui.horizontal_centered(|ui| {
        for tab in &view.tabs {
            let _ = components::choice(ui, tab.active, &tab.label);
        }
        for command in &view.commands {
            command_button(ui, command, &mut out);
        }
    });
    out
}

/// Tool row: project panel button, tools, inspector button. Returns the
/// intents and, for a panel toggle, the button focus returns to.
fn tool_row(ui: &mut egui::Ui, view: &ToolsView) -> (Vec<UiIntent>, Option<Id>) {
    let mut out = Vec::new();
    let mut opener = None;
    ui.horizontal_centered(|ui| {
        let toggle = &view.navigator_toggle;
        let navigator = components::choice(ui, toggle.active == Some(true), &toggle.label);
        if navigator.clicked() {
            out.push(UiIntent::Command(toggle.id));
            opener = Some(navigator.id);
        }
        ui.separator();
        for tool in &view.tools {
            match tool.active {
                // The active tool shows as pressed.
                Some(true) if tool.is_enabled() => {
                    if components::choice(ui, true, &tool.label).clicked() {
                        out.push(UiIntent::Command(tool.id));
                    }
                }
                _ => command_button(ui, tool, &mut out),
            }
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let toggle = &view.inspector_toggle;
            let inspector = components::choice(ui, toggle.active == Some(true), &toggle.label);
            if inspector.clicked() {
                out.push(UiIntent::Command(toggle.id));
                opener = Some(inspector.id);
            }
        });
    });
    (out, opener)
}

/// Wraps panel contents in a vertical scroll area.
fn scrolling<R>(ui: &mut egui::Ui, id: &str, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    ScrollArea::vertical()
        .id_salt(id)
        .auto_shrink([false, false])
        .show(ui, add_contents)
        .inner
}

/// The open drawer, over the canvas only, never over the header, rows or
/// status bar.
fn drawer(
    ctx: &egui::Context,
    which: Drawer,
    view: &Presentation,
    canvas_rect: Rect,
    palette: &Palette,
) -> Vec<UiIntent> {
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
    let mut out = Vec::new();
    Area::new(Id::unique("drawer"))
        .order(Order::Foreground)
        .fixed_pos(rect.min)
        .show(ctx, |ui| {
            frame.show(ui, |ui| {
                ui.set_min_size(inner_size);
                ui.set_max_size(inner_size);
                let close = components::action_button(ui, close_label, None, Some("Escape"));
                ui.separator();
                out = scrolling(ui, "drawer", |ui| match which {
                    Drawer::Navigator => sidebar(ui, &view.navigator),
                    Drawer::Inspector => inspector(ui, &view.inspector),
                });
                if close.clicked() {
                    out.push(UiIntent::Close(Surface::Drawer));
                }
            });
        });
    out
}

/// Command and object search. Enter runs the highlighted result, arrows move
/// the highlight, Escape or a click outside closes it, and focus returns to
/// where it was.
fn command_palette(ctx: &egui::Context, state: &mut AppState) {
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
    // The highlight is the dialog's own widget state.
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

    let mut intents = Vec::new();
    let mut query = search.query.clone();
    let modal = Modal::new(Id::unique("command-palette")).show(ctx, |ui| {
        ui.set_width(480.0);
        components::panel_title(ui, "Search commands and objects");
        let field = components::text_field(ui, "Find a command or object", &mut query);
        if !search.focused_once {
            field.request_focus();
            search.focused_once = true;
        }
        if field.changed() {
            intents.push(UiIntent::Search {
                scope: SearchScope::Commands,
                query: query.clone(),
            });
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
                    row = row.shortcut_text(commands::shortcut_text(&shortcut));
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
            intents.push(UiIntent::Command(id));
            true
        }
        Some(PaletteEntry::Object(index)) => {
            intents.push(UiIntent::Select {
                target: Target::Class(index),
                toggle: false,
            });
            true
        }
        None => false,
    };
    if ran || close_requested {
        intents.push(UiIntent::Close(Surface::CommandSearch));
        ctx.request_repaint();
    }
    state.palette = Some(search);
    let input = input_context(ctx, state, None);
    dispatch::apply(state, &input, intents);
}

/// Status bar. `in_workspace` adds the selection, which entry screens do not have.
fn status_bar(ui: &mut egui::Ui, view: &StatusView, in_workspace: bool) -> Vec<UiIntent> {
    let mut out = Vec::new();
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(view.save_text).text_style(TextRole::Label.text_style()));
        ui.separator();
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            // Right-to-left layout: add the options in reverse visual order.
            for (preference, label) in [
                (ThemePreference::System, "System"),
                (ThemePreference::Dark, "Dark"),
                (ThemePreference::Light, "Light"),
            ] {
                if components::choice(ui, view.theme == preference, label).clicked() {
                    out.push(UiIntent::SetTheme(preference));
                }
            }
            components::field_label(ui, "Theme:");
            if in_workspace {
                ui.separator();
                components::truncated_text(ui, &view.selection_text);
            }
        });
    });
    out
}

fn sidebar(ui: &mut egui::Ui, view: &NavigatorView) -> Vec<UiIntent> {
    let mut out = Vec::new();
    components::panel_title(ui, view.title);
    components::field_label(ui, "Diagrams");
    ui.indent("diagrams", |ui| {
        for row in &view.diagrams {
            let _ = components::list_row(ui, row.selected, &row.label);
        }
    });
    ui.add_space(theme::SPACING);
    components::field_label(ui, "Model");
    ui.indent("model", |ui| {
        for row in &view.model {
            if components::list_row(ui, row.selected, &row.label).clicked()
                && let Some(target) = row.target
            {
                out.push(UiIntent::Select {
                    target,
                    toggle: false,
                });
            }
        }
    });
    out
}

fn inspector(ui: &mut egui::Ui, view: &InspectorView) -> Vec<UiIntent> {
    let mut out = Vec::new();
    components::panel_title(ui, view.title);
    let (name, attributes) = match &view.kind {
        InspectorKind::Nothing { guidance } => {
            ui.label(*guidance);
            return out;
        }
        InspectorKind::Class { name, attributes } => (name, attributes),
    };

    let mut text = name.text.clone();
    let changed = components::text_field(ui, name.label, &mut text).changed();
    if changed {
        out.push(UiIntent::DraftChanged {
            field: DraftField::ClassName,
            text: text.clone(),
        });
    }
    // Show the problem for the text in the field, so it appears in the same
    // frame as the keystroke that caused it.
    let problem = if changed {
        text.trim().is_empty().then_some(BLANK_NAME_WARNING)
    } else {
        name.problem.as_deref()
    };
    if let Some(problem) = problem {
        components::warning_text(ui, problem);
    }

    ui.add_space(theme::SPACING);
    components::field_label(ui, "Attributes");
    for attribute in attributes {
        components::code_text(ui, attribute);
    }
    out
}

fn canvas(ui: &mut egui::Ui, view: &CanvasView, palette: &Palette) -> (Rect, Vec<UiIntent>) {
    let mut out = Vec::new();
    let (background, painter) = ui.allocate_painter(ui.available_size(), Sense::click());
    background.widget_info(|| WidgetInfo::labeled(Role::Canvas, true, view.accessible_name));
    painter.rect_filled(background.rect, CornerRadius::ZERO, palette.canvas);
    if let Some(text) = view.empty_text {
        painter.text(
            background.rect.center(),
            Align2::CENTER_CENTER,
            text,
            TextRole::Body.font_id(),
            palette.text_secondary,
        );
    }
    let origin = background.rect.min;

    let mut clicked_node = None;
    for (index, node) in view.nodes.iter().enumerate() {
        let rect = Rect::from_min_size(origin + node.position.to_vec2(), node.size);
        let response = ui.interact(
            rect,
            ui.scope_id().with(("canvas-class", index)),
            Sense::click(),
        );
        response.widget_info(|| {
            WidgetInfo::selected(Role::Button, true, node.selected, &node.accessible_name)
        });
        if response.clicked() {
            clicked_node = Some(node.target);
        }
        paint_class(&painter, rect, node, response.hovered(), palette);
    }

    match clicked_node {
        Some(target) => out.push(UiIntent::Select {
            target,
            toggle: false,
        }),
        None if background.clicked() => {
            let position = background.interact_pointer_pos().unwrap_or(origin);
            let modifiers = ui.input(|input| input.modifiers);
            for kind in [PointerKind::Down, PointerKind::Up] {
                out.push(UiIntent::Pointer(CanvasPointer {
                    kind,
                    position,
                    modifiers,
                    hit: None,
                }));
            }
        }
        None => {}
    }
    (background.rect, out)
}

fn paint_class(
    painter: &egui::Painter,
    rect: Rect,
    node: &NodeView,
    hovered: bool,
    palette: &Palette,
) {
    let selected = node.selected;
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
    let header_bottom = rect.top() + CLASS_HEADER_HEIGHT;
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
        &node.name,
        TextRole::Body.font_id(),
        DOCUMENT_INK,
        text_width,
    );
    let name_origin = Align2::CENTER_CENTER
        .anchor_size(
            Pos2::new(rect.center().x, rect.top() + CLASS_HEADER_HEIGHT / 2.0),
            name.size(),
        )
        .min;
    painter.galley(name_origin, name, DOCUMENT_INK);
    for (row, attribute) in node.attributes.iter().enumerate() {
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

/// eframe application wrapper around [`AppState`]; starts on the welcome screen.
#[derive(Debug)]
pub struct ShellApp {
    state: AppState,
}

impl Default for ShellApp {
    fn default() -> Self {
        Self {
            state: AppState::welcome(),
        }
    }
}

impl eframe::App for ShellApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        show(ui, &mut self.state);
    }
}
