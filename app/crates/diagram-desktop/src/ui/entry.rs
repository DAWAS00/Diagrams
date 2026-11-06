//! Welcome and new-project screens, shown instead of the workspace.
//!
//! The screens themselves live in `ui/welcome.rs`; this module turns their
//! `EntryAction`s into intents.

use eframe::egui::{self, Frame, Key, Modifiers, ScrollArea};

use crate::shell::ShellState;
use crate::ui::intents::UiIntent;
use crate::ui::status::status_bar;
use crate::ui::theme::{self, Palette};
use crate::ui::welcome::{self, EntryAction, EntryScreen};
use crate::ui::workspace;

/// Welcome and new-project screens, shown instead of the workspace.
pub(crate) fn entry_screens(ui: &mut egui::Ui, state: &mut ShellState, palette: &Palette) {
    let Some(mut entry) = state.entry.take() else {
        return;
    };
    // Escape leaves the name field first, then cancels the form.
    let cancel_by_escape = entry.screen == EntryScreen::NewProject
        && !state.text_had_focus()
        && ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape));

    let bar = Frame::side_top_panel(ui.style()).fill(palette.workspace);
    egui::Panel::top("header")
        .frame(bar)
        .exact_size(workspace::HEADER_HEIGHT)
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| ui.strong(crate::APPLICATION_NAME));
        });
    egui::Panel::bottom("status")
        .frame(bar)
        .show(ui, |ui| status_bar(ui, state, false));
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

    match action {
        Some(EntryAction::StartNewProject) => {
            entry.screen = EntryScreen::NewProject;
            entry.new_name.clear();
            entry.name_touched = false;
            entry.focus_name = true;
        }
        Some(EntryAction::CancelNewProject) => {
            entry.screen = EntryScreen::Welcome;
            state.focus_next_frame(entry.new_button);
        }
        Some(EntryAction::CreateProject(name)) => {
            if welcome::name_error(&name).is_none() {
                state.queue(UiIntent::CreateProject(name));
            } else {
                entry.name_touched = true;
            }
        }
        Some(EntryAction::OpenSample) => state.queue(UiIntent::OpenSample),
        Some(EntryAction::RemoveRecent(index)) => {
            if index < entry.recent.len() {
                entry.recent.remove(index);
            }
        }
        Some(EntryAction::DismissRecovery) => entry.recovery_visible = false,
        None => {}
    }
    state.entry = Some(entry);
}
