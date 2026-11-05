//! Tool row (spec §2, brief §A4): the project-panel toggle at the start, the
//! tools, and the inspector toggle at the end. There is one tool row only.
//!
//! Tools that do not fit move into a "More tools" menu, so no tool is ever
//! clipped. The active tool and shown panels are drawn pressed (fill and
//! border) and report themselves as selected.

use eframe::egui::{self, Align, Id, Layout, TextStyle};

use super::components;
use crate::intent::UiIntent;
use crate::presentation::{CommandView, ToolsView};

/// Label of the overflow menu.
pub const MORE_TOOLS: &str = "More tools";

/// What the tool row reports besides intents: the toggle that was pressed,
/// so focus can return to it when its drawer closes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ToolRowOutput {
    /// The panel toggle activated this frame, if any.
    pub opener: Option<Id>,
}

/// Width a command button needs, from the button text style and padding.
fn button_width(ui: &egui::Ui, label: &str) -> f32 {
    let font = TextStyle::Button.resolve(ui.style());
    let text = ui.fonts_mut(|fonts| {
        fonts
            .layout_no_wrap(label.to_owned(), font, egui::Color32::PLACEHOLDER)
            .size()
            .x
    });
    text + 2.0 * ui.spacing().button_padding.x + ui.spacing().item_spacing.x
}

/// How many of `tools` fit in `available` points, leaving room for the
/// "More tools" button whenever some do not.
pub fn fitting_tools(ui: &egui::Ui, tools: &[CommandView], available: f32) -> usize {
    let widths: Vec<f32> = tools
        .iter()
        .map(|tool| button_width(ui, &tool.label))
        .collect();
    if widths.iter().sum::<f32>() <= available {
        return tools.len();
    }
    let room = available - button_width(ui, MORE_TOOLS) - ui.spacing().icon_width;
    let mut used = 0.0;
    widths
        .iter()
        .take_while(|width| {
            used += **width;
            used <= room
        })
        .count()
}

fn toggle(ui: &mut egui::Ui, command: &CommandView, out: &mut Vec<UiIntent>) -> Option<Id> {
    let response = components::choice(ui, command.active == Some(true), &command.label);
    if response.clicked() {
        out.push(UiIntent::Command(command.id));
        return Some(response.id);
    }
    None
}

/// Draws the tool row into `ui` (a row [`super::workspace::TOOL_ROW_HEIGHT`]
/// high) and returns the toggle that opened a drawer, if any.
pub fn show(ui: &mut egui::Ui, view: &ToolsView, out: &mut Vec<UiIntent>) -> ToolRowOutput {
    let mut output = ToolRowOutput::default();
    ui.horizontal_centered(|ui| {
        output.opener = toggle(ui, &view.navigator_toggle, out).or(output.opener);
        ui.separator();
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if let Some(id) = toggle(ui, &view.inspector_toggle, out) {
                output.opener = Some(id);
            }
            ui.separator();
            ui.allocate_ui_with_layout(
                ui.available_size(),
                Layout::left_to_right(Align::Center),
                |ui| tools(ui, &view.tools, out),
            );
        });
    });
    output
}

fn tools(ui: &mut egui::Ui, tools: &[CommandView], out: &mut Vec<UiIntent>) {
    let fit = fitting_tools(ui, tools, ui.available_width());
    let (shown, hidden) = tools.split_at(fit);
    for tool in shown {
        if components::command_button(ui, tool) {
            out.push(UiIntent::Command(tool.id));
        }
    }
    if hidden.is_empty() {
        return;
    }
    let (_, chosen) =
        components::dropdown_button(ui, MORE_TOOLS, |ui| components::menu_items(ui, hidden));
    if let Some(Some(id)) = chosen {
        out.push(UiIntent::Command(id));
    }
}
