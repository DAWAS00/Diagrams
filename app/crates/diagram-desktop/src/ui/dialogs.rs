//! Modal dialogs (UI-09): a question the user must answer before going on.
//!
//! A dialog is plain data ([`DialogState`]); the coordinator opens and
//! closes it, and [`show`] only draws it and reports the answer. Escape and
//! a click outside both mean Cancel, the safe answer has focus when it
//! opens, and focus returns to the control that opened it.

use eframe::egui::{self, Id, Modal};

use super::components;
use super::theme;

/// What a dialog asks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogKind {
    /// Closing a project that has changes which would be lost.
    CloseProject {
        /// Name of the project.
        project: String,
    },
}

/// An open dialog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialogState {
    /// The question.
    pub kind: DialogKind,
    /// The safe button has already been given focus.
    pub focused_once: bool,
}

impl DialogState {
    /// A dialog asking `kind`.
    pub const fn new(kind: DialogKind) -> Self {
        Self {
            kind,
            focused_once: false,
        }
    }
}

/// How the user answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogAnswer {
    /// Go ahead with the action.
    Confirm,
    /// Leave everything as it was.
    Cancel,
}

/// Title, explanation and button labels of a dialog.
pub struct DialogText {
    /// Title.
    pub title: &'static str,
    /// Explanation, in words.
    pub body: String,
    /// Label of the button that goes ahead.
    pub confirm: &'static str,
    /// Label of the safe button.
    pub cancel: &'static str,
}

/// The words of `kind`.
pub fn text(kind: &DialogKind) -> DialogText {
    match kind {
        DialogKind::CloseProject { project } => DialogText {
            title: "Close project?",
            body: format!(
                "{project} has changes that are not saved. Saving is not available yet, \
                 so closing discards them."
            ),
            confirm: "Discard changes and close",
            cancel: "Cancel",
        },
    }
}

/// Draws `dialog` and returns the user's answer, if they gave one this frame.
pub fn show(ctx: &egui::Context, dialog: &mut DialogState) -> Option<DialogAnswer> {
    let words = text(&dialog.kind);
    let mut answer = None;
    let modal = Modal::new(Id::unique("dialog")).show(ctx, |ui| {
        ui.set_width(420.0);
        components::panel_title(ui, words.title);
        ui.label(&words.body);
        ui.add_space(theme::GAP);
        ui.horizontal(|ui| {
            let cancel = components::action_button(ui, words.cancel, None, Some("Escape"));
            if !dialog.focused_once {
                cancel.request_focus();
                dialog.focused_once = true;
            }
            if cancel.clicked() {
                answer = Some(DialogAnswer::Cancel);
            }
            if components::action_button(ui, words.confirm, None, None).clicked() {
                answer = Some(DialogAnswer::Confirm);
            }
        });
    });
    if answer.is_none() && modal.should_close() {
        answer = Some(DialogAnswer::Cancel);
    }
    answer
}
