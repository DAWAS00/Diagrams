//! Diagram tab row (spec §2, brief §A3): one tab per open diagram, then the
//! tab-row commands ("New diagram").
//!
//! Only whole tabs are drawn. When they do not all fit, the row shows a
//! window of consecutive tabs and a "More tabs" menu listing every tab;
//! choosing one moves the window so that tab is shown, and focuses it. The
//! window position is local widget state. (A hidden scroll area was tried
//! first: its clipped tabs kept their full rectangles in the accessibility
//! tree, under "New diagram" and outside the window. The audit found it.)
//!
//! The active tab is drawn pressed *and* underlined, so it is shown by more
//! than color, and reports itself as selected. Clicking another tab pushes
//! `ActivateTab`; closing tabs needs an intent from Agent B (request 2 in
//! AGENT-A-UI.md §6).

use eframe::egui::{self, Align, Id, Layout, Stroke, TextStyle, Vec2};

use super::components;
use super::theme::{CONTROL_HEIGHT, GAP, SPACING};
use crate::intent::UiIntent;
use crate::presentation::{TabView, TabsView};

/// Label of the overflow menu.
pub const MORE_TABS: &str = "More tabs";

/// The row's local state: first tab of the window, and a tab to focus.
#[derive(Debug, Clone, Copy, Default)]
struct Strip {
    first: Option<usize>,
    focus: Option<usize>,
}

fn tab_width(ui: &egui::Ui, label: &str) -> f32 {
    let font = TextStyle::Button.resolve(ui.style());
    let text = ui.fonts_mut(|fonts| {
        fonts
            .layout_no_wrap(label.to_owned(), font, egui::Color32::PLACEHOLDER)
            .size()
            .x
    });
    // Active tabs are bold; measure generously so they never clip.
    text * 1.08 + 2.0 * ui.spacing().button_padding.x + GAP
}

/// End (exclusive) of the window starting at `first` within `available`.
/// Always shows at least one tab.
fn window_end(widths: &[f32], first: usize, available: f32) -> usize {
    let mut used = 0.0;
    let mut end = first;
    while end < widths.len() && (end == first || used + widths[end] <= available) {
        used += widths[end];
        end += 1;
    }
    end
}

/// First index of a window that includes `index`, moving as little as
/// possible from `first`.
fn reveal(widths: &[f32], mut first: usize, index: usize, available: f32) -> usize {
    if index < first {
        return index;
    }
    while window_end(widths, first, available) <= index {
        first += 1;
    }
    first
}

/// Draws the tab row into `ui` (a row [`super::workspace::TAB_ROW_HEIGHT`] high).
pub fn show(ui: &mut egui::Ui, view: &TabsView, out: &mut Vec<UiIntent>) {
    let strip_id = ui.scope_id().with("tab-strip");
    let mut strip: Strip = ui.data(|data| data.get_temp(strip_id).unwrap_or_default());
    ui.horizontal_centered(|ui| {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            for command in view.commands.iter().rev() {
                if components::command_button(ui, command) {
                    out.push(UiIntent::Command(command.id));
                }
            }
            let widths: Vec<f32> = view
                .tabs
                .iter()
                .map(|tab| tab_width(ui, &tab.label))
                .collect();
            let all_fit = widths.iter().sum::<f32>() <= ui.available_width();
            let available = if all_fit {
                ui.available_width()
            } else {
                ui.available_width() - tab_width(ui, MORE_TABS) - ui.spacing().icon_width - SPACING
            };
            let active = view.tabs.iter().position(|tab| tab.active).unwrap_or(0);
            let mut first = strip
                .first
                .unwrap_or_else(|| reveal(&widths, 0, active, available));
            if !all_fit {
                let (_, chosen) = components::dropdown_button(ui, MORE_TABS, |ui| {
                    let mut chosen = None;
                    for (index, tab) in view.tabs.iter().enumerate() {
                        if components::list_row(ui, tab.active, &tab.label).clicked() {
                            chosen = Some(index);
                            ui.close();
                        }
                    }
                    chosen
                });
                if let Some(Some(index)) = chosen {
                    first = reveal(&widths, first, index, available);
                    strip.focus = Some(index);
                }
            } else {
                first = 0;
            }
            first = first.min(view.tabs.len().saturating_sub(1));
            let end = window_end(&widths, first, available);
            strip.first = Some(first);
            ui.allocate_ui_with_layout(
                ui.available_size(),
                Layout::left_to_right(Align::Center),
                |ui| {
                    ui.spacing_mut().item_spacing.x = GAP;
                    for (index, tab) in view.tabs.iter().enumerate().take(end).skip(first) {
                        let focus = strip.focus == Some(index);
                        if tab_button(ui, strip_id, index, tab, focus) && !tab.active {
                            out.push(UiIntent::ActivateTab(index));
                        }
                        if focus {
                            strip.focus = None;
                        }
                    }
                },
            );
        });
    });
    ui.data_mut(|data| data.insert_temp(strip_id, strip));
}

/// One tab. The active tab is bold, pressed and underlined in the accent color.
fn tab_button(ui: &mut egui::Ui, strip: Id, index: usize, tab: &TabView, focus: bool) -> bool {
    let text = if tab.active {
        egui::RichText::new(&tab.label).strong()
    } else {
        egui::RichText::new(&tab.label)
    };
    let button = egui::Button::selectable(tab.active, text)
        .frame_when_inactive(true)
        .truncate()
        .min_size(Vec2::new(0.0, CONTROL_HEIGHT));
    let response = ui.push_id(strip.with(index), |ui| ui.add(button)).inner;
    if tab.active {
        let rect = response.rect;
        ui.painter().hline(
            rect.x_range().shrink(SPACING / 2.0),
            rect.bottom() - 1.5,
            Stroke::new(3.0, ui.visuals().hyperlink_color),
        );
    }
    if focus {
        response.request_focus();
    }
    response.clicked()
}
