//! Text drafts: a field edits a copy, not the model, until it is committed.
//!
//! Policy (functional spec §6, "Draft and focus policy"):
//! 1. Typing changes the draft. Enter, or leaving the field, commits a valid
//!    draft; an invalid draft stays visible and uncommitted.
//! 2. Escape cancels the draft and restores the original value.
//! 3. A selection, tab or tool change that would lose an invalid draft asks
//!    to resolve it first (Return to field / Discard edit).
//!
//! Every commit goes through [`commit_class_name`], the single function M1
//! turns into a `RenameElement` command.

use crate::app_state::{BLANK_NAME_WARNING, PreviewClass};
use crate::intent::{DraftField, Target};

/// An uncommitted edit of one field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Draft {
    /// The field being edited.
    pub field: DraftField,
    /// The object the field belongs to.
    pub target: Target,
    /// The value when editing started; Escape restores it.
    pub original: String,
    /// The text in the field.
    pub text: String,
    /// Enter or a commit was tried while the text was invalid.
    pub commit_refused: bool,
}

impl Draft {
    /// Starts a draft from the current value.
    pub fn new(field: DraftField, target: Target, original: String) -> Self {
        Self {
            field,
            target,
            text: original.clone(),
            original,
            commit_refused: false,
        }
    }

    /// Why the text cannot be committed, if it cannot.
    pub fn problem(&self) -> Option<&'static str> {
        validate(self.field, &self.text)
    }

    /// The text can be committed.
    pub fn is_valid(&self) -> bool {
        self.problem().is_none()
    }

    /// The text differs from the original.
    pub fn is_changed(&self) -> bool {
        self.text != self.original
    }
}

/// The rule for each field.
pub fn validate(field: DraftField, text: &str) -> Option<&'static str> {
    match field {
        DraftField::ClassName => text.trim().is_empty().then_some(BLANK_NAME_WARNING),
    }
}

/// Commits a valid class name. This is the one place a rename reaches the
/// document; M1 replaces the body with a `RenameElement` command.
pub fn commit_class_name(class: &mut PreviewClass, name: &str) {
    if validate(DraftField::ClassName, name).is_none() && class.name != name {
        name.clone_into(&mut class.name);
    }
}
