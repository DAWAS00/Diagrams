//! Header row (spec §2, brief §A2): Project menu, project name with its save
//! state, then the header commands (Undo, Redo, Search, Save, Export).
//!
//! The view draws [`HeaderView`] and pushes [`UiIntent`]s; labels, shortcut
//! text and reasons all come from the presentation.

use eframe::egui::{self, Align, Layout};

use super::components;
use crate::intent::UiIntent;
use crate::presentation::HeaderView;

/// Label of the menu button that lists every catalog command.
pub const PROJECT_MENU: &str = "Project menu";

/// Draws the header into `ui` (a row [`super::workspace::HEADER_HEIGHT`] high).
pub fn show(ui: &mut egui::Ui, view: &HeaderView, out: &mut Vec<UiIntent>) {
    ui.horizontal_centered(|ui| {
        project_menu(ui, view, out);
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
                if components::command_button(ui, command) {
                    out.push(UiIntent::Command(command.id));
                }
            }
            ui.separator();
            // The title takes the remaining width and shortens with an
            // ellipsis; the full title stays its accessible name.
            ui.allocate_ui_with_layout(
                ui.available_size(),
                Layout::left_to_right(Align::Center),
                |ui| components::truncated_text(ui, &view.title),
            );
        });
    });
}

/// The Project menu: one submenu per command group, each item with its
/// shortcut, and disabled items with their reason as visible text.
fn project_menu(ui: &mut egui::Ui, view: &HeaderView, out: &mut Vec<UiIntent>) {
    let (_, chosen) = components::dropdown_button(ui, PROJECT_MENU, |ui| {
        let mut chosen = None;
        for group in &view.menu {
            ui.menu_button(group.label, |ui| {
                ui.set_min_width(260.0);
                if let Some(id) = components::menu_items(ui, &group.commands) {
                    chosen = Some(id);
                }
            });
        }
        chosen
    });
    if let Some(Some(id)) = chosen {
        out.push(UiIntent::Command(id));
    }
}
