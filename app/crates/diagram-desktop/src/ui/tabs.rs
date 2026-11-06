//! Diagram tab row: one tab per open diagram, a labeled Close button, a
//! "More tabs" menu for tabs that do not fit, and New diagram.
//!
//! Moved out of `shell.rs` unchanged.

use eframe::egui::{self, Button, Color32, Role, Vec2, WidgetInfo};

use crate::shell::{ShellState, command_button};
use crate::ui::commands::{self, Placement};
use crate::ui::components;
use crate::ui::intents::UiIntent;
use crate::ui::navigator;
use crate::ui::theme::{self, TextRole};

/// What the user did in the tab row.
enum TabAction {
    Activate(usize),
    Close(usize),
}

/// Space a tab takes: its title, its Close button and the gaps.
fn tab_width(ui: &egui::Ui, title: &str) -> f32 {
    let font = TextRole::Label.font_id();
    let text = ui.fonts_mut(|fonts| {
        fonts
            .layout_no_wrap(title.to_owned(), font.clone(), Color32::BLACK)
            .size()
            .x
    });
    let close = ui.fonts_mut(|fonts| {
        fonts
            .layout_no_wrap(String::from("Close"), font, Color32::BLACK)
            .size()
            .x
    });
    let padding = 2.0 * ui.spacing().button_padding.x;
    text + close + 2.0 * padding + 3.0 * ui.spacing().item_spacing.x
}

/// Diagram tabs: one per open diagram with a labeled Close button, a
/// "More tabs" menu for tabs that do not fit, and New diagram.
pub(crate) fn tab_row(ui: &mut egui::Ui, state: &mut ShellState) {
    let context = state.command_context();
    let mut clicked = None;
    let mut action = None;
    let titles: Vec<String> = state
        .tabs
        .iter()
        .map(|tab| {
            state
                .diagrams
                .get(tab.diagram)
                .map(|diagram| diagram.name.clone())
                .unwrap_or_default()
        })
        .collect();
    ui.horizontal_centered(|ui| {
        // Reserve room for New diagram and the overflow menu.
        let available = ui.available_width() - 260.0;
        let widths: Vec<f32> = titles.iter().map(|title| tab_width(ui, title)).collect();
        let (shown, overflow) = navigator::tabs_that_fit(&widths, available, state.active_tab);
        for index in shown {
            let title = &titles[index];
            let active = state.active_tab == Some(index);
            let tab = components::choice(ui, active, title);
            tab.widget_info(|| WidgetInfo::selected(Role::Tab, true, active, title));
            if tab.clicked() {
                action = Some(TabAction::Activate(index));
            }
            let close = ui
                .add(Button::new("Close").min_size(Vec2::new(0.0, theme::CONTROL_HEIGHT)))
                .on_hover_text(format!("Close {title}. Its classes stay in the project."));
            close.widget_info(|| WidgetInfo::labeled(Role::Button, true, format!("Close {title}")));
            if close.clicked() {
                action = Some(TabAction::Close(index));
            }
            ui.separator();
        }
        if !overflow.is_empty() {
            ui.menu_button(format!("More tabs ({})", overflow.len()), |ui| {
                for index in overflow {
                    if ui.button(&titles[index]).clicked() {
                        action = Some(TabAction::Activate(index));
                        ui.close();
                    }
                }
            });
        }
        for spec in commands::placed(Placement::TabRow) {
            command_button(ui, spec, context, &mut clicked);
        }
    });
    match action {
        Some(TabAction::Activate(index)) => state.queue(UiIntent::ActivateTab(index)),
        Some(TabAction::Close(index)) => state.queue(UiIntent::CloseTab(index)),
        None => {}
    }
    if let Some(id) = clicked {
        state.queue(UiIntent::run(id));
    }
}
