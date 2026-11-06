//! Command and object search, shown as a modal over the workspace.
//!
//! Moved out of `shell.rs` unchanged apart from asking for focus through
//! `ShellState::focus_next_frame`.

use eframe::egui::{self, Button, Id, Key, Modal, ScrollArea, Vec2};

use crate::shell::ShellState;
use crate::ui::commands::{self, Availability};
use crate::ui::components;
use crate::ui::intents::UiIntent;
use crate::ui::palette::{self, PaletteEntry};
use crate::ui::read_model;
use crate::ui::theme;
use diagram_core::ids::ElementId;

/// Command and object search. Enter runs the highlighted result, arrows move
/// the highlight, Escape or a click outside closes it, and focus returns to
/// where it was.
pub(crate) fn command_palette(ctx: &egui::Context, state: &mut ShellState) {
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
        state.focus_next_frame(search.restore_focus);
        ctx.request_repaint();
    } else {
        state.palette = Some(search);
    }
}
