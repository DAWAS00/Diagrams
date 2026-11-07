//! Modal dialogs (UI-09): a question the user must answer before going on.
//!
//! A dialog is plain data ([`DialogState`]); the coordinator opens and
//! closes it, and [`show`] only draws it and reports the answer. Escape and
//! a click outside both mean Cancel, the safe answer has focus when it
//! opens, and focus returns to the control that opened it.

use diagram_core::ids::ElementId;
use eframe::egui::{self, Id, Modal};

use super::class_tool::PlacementId;

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
    /// A new class was given the name of an existing class (D2).
    DuplicateClassName {
        /// The placement being named.
        placement: PlacementId,
        /// The typed name, trimmed.
        name: String,
        /// The class "Show the existing class here" would show, or `None`
        /// when every class with this name is already in the diagram.
        existing: Option<ElementId>,
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
    /// Take the dialog's second way forward.
    Alternative,
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
    /// Label of a second way forward, if the dialog has one.
    pub alternative: Option<&'static str>,
    /// Why the second way forward is unavailable, shown as visible text.
    pub alternative_disabled: Option<String>,
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
            alternative: None,
            alternative_disabled: None,
        },
        DialogKind::DuplicateClassName { name, existing, .. } => DialogText {
            title: "A class with this name exists",
            body: format!(
                "A class named \"{name}\" already exists in this project. Show that class \
                 here, so both diagrams share it, or keep a new, separate class with the same \
                 name."
            ),
            confirm: "Keep a new class",
            cancel: "Cancel",
            alternative: Some("Show the existing class here"),
            alternative_disabled: existing
                .is_none()
                .then(|| format!("Every class named \"{name}\" is already in this diagram.")),
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
        ui.horizontal_wrapped(|ui| {
            let cancel = components::action_button(ui, words.cancel, None, Some("Escape"));
            if !dialog.focused_once {
                cancel.request_focus();
                dialog.focused_once = true;
            }
            if cancel.clicked() {
                answer = Some(DialogAnswer::Cancel);
            }
            if let Some(label) = words.alternative {
                let reason = words.alternative_disabled.as_deref();
                if components::action_button(ui, label, reason, None).clicked() {
                    answer = Some(DialogAnswer::Alternative);
                }
            }
            if components::action_button(ui, words.confirm, None, None).clicked() {
                answer = Some(DialogAnswer::Confirm);
            }
        });
        if let Some(reason) = &words.alternative_disabled {
            components::field_label(ui, reason);
        }
    });
    if answer.is_none() && modal.should_close() {
        answer = Some(DialogAnswer::Cancel);
    }
    answer
}
