//! The header row: application menu, project name, and the Undo, Redo,
//! Search, Save and Export buttons, all read from the command catalog.
//!
//! Moved out of `shell.rs` unchanged.

use eframe::egui::{self, Align, Button, Id, Layout};

use crate::shell::ShellState;
use crate::ui::commands::{self, CommandContext, CommandId, CommandSpec, Placement};
use crate::ui::components;
use crate::ui::intents::UiIntent;

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
pub(crate) fn header(ui: &mut egui::Ui, state: &mut ShellState) {
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
