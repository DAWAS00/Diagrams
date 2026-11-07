//! The tool row above the canvas: project panel button, the notation's
//! tools, and the inspector button.
//!
//! Moved out of `shell.rs` unchanged.

use eframe::egui::{self, Align, Layout};

use crate::shell::ShellState;
use crate::ui::class_tool::Tool;
use crate::ui::commands;
use crate::ui::commands::CommandId;
use crate::ui::components;
use crate::ui::header::command_button;
use crate::ui::intents::UiIntent;
use crate::ui::notation;
use crate::ui::workspace::{Drawer, LayoutMode};

/// Whether a panel's toggle shows as pressed.
fn panel_shown(state: &ShellState, which: Drawer) -> bool {
    match (state.layout, which) {
        (LayoutMode::Docked, Drawer::Navigator) => state.panels.navigator_visible,
        (LayoutMode::Docked, Drawer::Inspector) => state.panels.inspector_visible,
        (LayoutMode::Narrow, _) => state.drawer == Some(which),
    }
}

/// Tool row: project panel button, tools, inspector button.
pub(crate) fn tool_row(ui: &mut egui::Ui, state: &mut ShellState) {
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
            // The active tool shows as pressed.
            let tool = match spec.id {
                CommandId::SelectTool => Some(Tool::Select),
                CommandId::AddClass => Some(Tool::Class),
                _ => None,
            };
            if let Some(tool) = tool {
                let reason = commands::availability(spec.id, context).reason();
                let pressed = state.active_tool() == tool;
                if components::tool_choice(ui, pressed, spec.label, reason).clicked() {
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
