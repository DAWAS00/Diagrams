//! Native shell: header, tabs, tool row, project panel, canvas, inspector and
//! status bar.
//!
//! The shell only composes the layout (A-10): every region is drawn by a
//! view module in [`crate::ui`] from [`presentation::build`]. Views push
//! [`UiIntent`]s, and [`dispatch::apply`] is the only code that changes the
//! state. The preview classes are prototype data until M1 binds the core
//! model, commands and history.

use eframe::egui::{
    self, Area, Button, Frame, Id, Key, Modal, Modifiers, Order, Pos2, Rect, ScrollArea, Stroke,
    Vec2,
};

pub use crate::app_state::{
    AppState, BLANK_NAME_WARNING, PreviewClass, SAMPLE_PROJECT, ShellState,
};

use crate::dispatch;
use crate::editor_state::ShellSupport;
use crate::intent::{InputContext, SearchScope, Surface, Target, UiIntent};
use crate::presentation::{self, Presentation};
use crate::ui::commands::{self, Availability};
use crate::ui::components;
use crate::ui::palette::{self, PaletteEntry};
use crate::ui::showcase;
use crate::ui::theme::{self, Palette, TextRole};
use crate::ui::welcome::{self, EntryAction, EntryScreen};
use crate::ui::workspace::{self, Drawer, INSPECTOR, LayoutMode, NAVIGATOR};
use crate::ui::{
    canvas_view, dialogs, header, inspector, navigator, search_view, status, tabs, toolbar,
};

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
        .show(ui, |ui| {
            let mut out = Vec::new();
            status::show(ui, &view.status, false, &mut out);
            out
        })
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

    // What this shell really draws, so B enables the matching commands.
    state.support = SUPPORT;
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
        .show(ui, |ui| collect(|out| header::show(ui, &view.header, out)))
        .inner;
    commit(&ctx, state, &mut view, intents, None);
    let intents = egui::Panel::bottom("status")
        .frame(bar)
        .show(ui, |ui| {
            collect(|out| status::show(ui, &view.status, true, out))
        })
        .inner;
    commit(&ctx, state, &mut view, intents, None);
    if state.layout == LayoutMode::Docked {
        if state.panels.navigator_visible {
            let intents = egui::Panel::left("navigator")
                .resizable(true)
                .default_size(NAVIGATOR.default)
                .size_range(NAVIGATOR.min..=NAVIGATOR.max)
                .show(ui, |ui| {
                    scrolling(ui, "navigator", |ui| {
                        collect(|out| project_panel(ui, &view, out))
                    })
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
                    scrolling(ui, "inspector", |ui| {
                        collect(|out| inspector::show(ui, &view.inspector, out))
                    })
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
                .show(ui, |ui| collect(|out| tabs::show(ui, &view.tabs, out)))
                .inner;
            commit(&ctx, state, &mut view, intents, None);
            let (intents, opener) = egui::Panel::top("tools")
                .frame(row)
                .exact_size(workspace::TOOL_ROW_HEIGHT)
                .show(ui, |ui| {
                    let mut out = Vec::new();
                    let row = toolbar::show(ui, &view.tools, &mut out);
                    (out, row.opener)
                })
                .inner;
            commit(&ctx, state, &mut view, intents, opener);
            let (rect, intents) = Frame::new()
                .inner_margin(theme::SPACING)
                .show(ui, |ui| {
                    let rect = ui.available_rect_before_wrap();
                    (
                        rect,
                        collect(|out| canvas_view::show(ui, &view.canvas, out)),
                    )
                })
                .inner;
            commit(&ctx, state, &mut view, intents, None);
            rect
        })
        .inner;
    if let Some(which) = state.drawer {
        let intents = drawer(&ctx, which, &view, canvas_rect, &palette);
        commit(&ctx, state, &mut view, intents, None);
    }
    let intents = collect(|out| dialogs::show(ui, view.dialog.as_ref(), out));
    commit(&ctx, state, &mut view, intents, None);
    showcase_window(&ctx, state);
    command_palette(&ctx, state);
    // Only a focused text field keeps editing keys; a focused button must not
    // swallow Escape (egui_wants_keyboard_input is true for any focus).
    state.text_had_focus = ctx.text_edit_focused();
}

/// The surfaces this shell draws (B enables their commands from these).
///
/// - `draft_events` and `resolve_dialog` stay off although the inspector
///   sends CommitDraft/CancelDraft and `ui::dialogs` draws Resolve edit:
///   Agent B's `journeys.rs` (J2) and A's older shell tests still expect a
///   name to commit as it is typed. They switch on together with those tests.
/// - `region_focus` is off: the shell keeps egui's own focus.
const SUPPORT: ShellSupport = ShellSupport {
    draft_events: false,
    resolve_dialog: false,
    grid: true,
    object_search: true,
    region_focus: false,
};

/// The project panel: the object search while it is open, then the lists.
fn project_panel(ui: &mut egui::Ui, view: &Presentation, out: &mut Vec<UiIntent>) {
    search_view::object_search(ui, &view.object_search, out);
    navigator::show(ui, &view.navigator, out);
}

/// Height of the command search's result list.
const SEARCH_LIST_HEIGHT: f32 = 280.0;

/// Runs a view and returns the intents it pushed.
fn collect(draw: impl FnOnce(&mut Vec<UiIntent>)) -> Vec<UiIntent> {
    let mut out = Vec::new();
    draw(&mut out);
    out
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
                out = scrolling(ui, "drawer", |ui| {
                    collect(|out| match which {
                        Drawer::Navigator => project_panel(ui, view, out),
                        Drawer::Inspector => inspector::show(ui, &view.inspector, out),
                    })
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
        // Only the rows in view are laid out (show_rows), so rows scrolled
        // out of the list never sit in the accessibility tree at positions
        // outside the window. Every row reserves a line for its reason, so
        // all rows have the same height.
        let reason_height = ui.text_style_height(&TextRole::Label.text_style());
        let spacing = ui.spacing().item_spacing.y;
        let row_height = theme::ROW_HEIGHT + spacing + reason_height;
        let offset_id = Id::unique("command-search-offset");
        let mut scroll = ScrollArea::vertical()
            .id_salt("command-search-results")
            .max_height(SEARCH_LIST_HEIGHT);
        if down || up {
            let offset: f32 = ui.data(|data| data.get_temp(offset_id).unwrap_or(0.0));
            let top = search.highlighted as f32 * (row_height + spacing);
            let bottom = top + row_height;
            if top < offset {
                scroll = scroll.vertical_scroll_offset(top);
            } else if bottom > offset + SEARCH_LIST_HEIGHT {
                scroll = scroll.vertical_scroll_offset(bottom - SEARCH_LIST_HEIGHT);
            }
        }
        let output = scroll.show_rows(ui, row_height, results.len(), |ui, range| {
            for index in range {
                let Some(entry) = results.get(index).copied() else {
                    continue;
                };
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
                let response = match availability.reason() {
                    Some(reason) => response.on_disabled_hover_text(reason),
                    None => response,
                };
                // The reason is visible text, on one line; the full reason
                // is also the row's hover text.
                ui.allocate_ui(Vec2::new(ui.available_width(), reason_height), |ui| {
                    if let Some(reason) = availability.reason() {
                        components::truncated_text(ui, reason);
                    }
                });
                if response.clicked() {
                    chosen = Some(entry);
                }
            }
        });
        ui.data_mut(|data| data.insert_temp(offset_id, output.state.clamped_offset().y));
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
