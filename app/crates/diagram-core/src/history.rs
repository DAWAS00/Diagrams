//! Undo, redo and saved-state identity (R1-04).
//!
//! - [`SessionId`] names one open-document lifetime.
//! - [`Revision`] increases with every accepted edit, undo and redo, so a
//!   worker result for an older revision is recognisably stale.
//! - [`StateId`] names a document state in history: undoing back to the
//!   saved state makes the project clean again even though the revision
//!   moved on. A new edit after undo creates a new state and drops redo.
//!
//! The history keeps at most [`HISTORY_LIMIT`] whole transactions; the
//! oldest are evicted whole, never split. (The 64 MiB byte budget of the
//! architecture plan is not measured yet.)

use crate::command::{self, ChangeSet, Command};
use crate::model::Document;
use crate::validate::DomainError;

/// Most transactions kept for undo.
pub const HISTORY_LIMIT: usize = 500;

/// One open-document lifetime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId(u64);

impl SessionId {
    /// A session ID from a raw value chosen by the caller.
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }
}

/// Monotonic edit sequence number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Revision(u64);

impl Revision {
    /// The raw value.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Identity of one document state in history.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateId(u64);

/// One undoable user action.
#[derive(Debug, Clone, PartialEq)]
struct Transaction {
    changes: ChangeSet,
    before: StateId,
    after: StateId,
}

/// The document with its history and identities.
#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    document: Document,
    session: SessionId,
    revision: Revision,
    state: StateId,
    saved: Option<StateId>,
    next_state: u64,
    undo: Vec<Transaction>,
    redo: Vec<Transaction>,
}

/// What the last edit, undo or redo did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Outcome {
    /// Label of the action.
    pub label: &'static str,
    /// Revision after it.
    pub revision: Revision,
}

impl Session {
    /// A session for `document`. A document that was just opened from a
    /// file is clean; a new one is unsaved until a save succeeds.
    pub fn new(session: SessionId, document: Document, opened_from_file: bool) -> Self {
        let state = StateId(0);
        Self {
            document,
            session,
            revision: Revision::default(),
            state,
            saved: opened_from_file.then_some(state),
            next_state: 1,
            undo: Vec::new(),
            redo: Vec::new(),
        }
    }

    /// The authoritative document, read-only.
    pub const fn document(&self) -> &Document {
        &self.document
    }

    /// This session's identity.
    pub const fn id(&self) -> SessionId {
        self.session
    }

    /// Current revision.
    pub const fn revision(&self) -> Revision {
        self.revision
    }

    /// Current history state.
    pub const fn state(&self) -> StateId {
        self.state
    }

    /// The project differs from the last successful save (or was never saved).
    pub fn is_dirty(&self) -> bool {
        self.saved != Some(self.state)
    }

    /// Label of the action Undo would reverse.
    pub fn undo_label(&self) -> Option<&'static str> {
        self.undo
            .last()
            .map(|transaction| transaction.changes.label)
    }

    /// Label of the action Redo would repeat.
    pub fn redo_label(&self) -> Option<&'static str> {
        self.redo
            .last()
            .map(|transaction| transaction.changes.label)
    }

    /// Number of transactions Undo can reverse.
    pub fn undo_depth(&self) -> usize {
        self.undo.len()
    }

    /// Validates and applies `command` as one undoable action.
    ///
    /// # Errors
    ///
    /// Returns the [`DomainError`] when the command is rejected; the
    /// document, history and identities are then unchanged.
    pub fn execute(&mut self, command: &Command) -> Result<Outcome, DomainError> {
        let changes = command::execute(&mut self.document, command)?;
        let before = self.state;
        self.state = self.fresh_state();
        self.undo.push(Transaction {
            changes,
            before,
            after: self.state,
        });
        if self.undo.len() > HISTORY_LIMIT {
            self.undo.remove(0);
        }
        // A new edit after Undo starts a new branch: the old future is gone.
        self.redo.clear();
        Ok(self.bump(command.label()))
    }

    /// Reverses the last action. `None` when there is nothing to undo.
    pub fn undo(&mut self) -> Option<Outcome> {
        let transaction = self.undo.pop()?;
        if command::apply(&mut self.document, &transaction.changes.inverse()).is_err() {
            // History always matches the document it was recorded on.
            self.undo.push(transaction);
            return None;
        }
        self.state = transaction.before;
        let label = transaction.changes.label;
        self.redo.push(transaction);
        Some(self.bump(label))
    }

    /// Repeats the last undone action. `None` when there is nothing to redo.
    pub fn redo(&mut self) -> Option<Outcome> {
        let transaction = self.redo.pop()?;
        if command::apply(&mut self.document, &transaction.changes).is_err() {
            self.redo.push(transaction);
            return None;
        }
        self.state = transaction.after;
        let label = transaction.changes.label;
        self.undo.push(transaction);
        Some(self.bump(label))
    }

    /// Records that a save of `state` in this session succeeded. A result for
    /// another session is ignored, and a save of an older state leaves later
    /// edits dirty. Returns whether the result was accepted.
    pub fn mark_saved(&mut self, session: SessionId, state: StateId) -> bool {
        if session != self.session {
            return false;
        }
        self.saved = Some(state);
        true
    }

    fn fresh_state(&mut self) -> StateId {
        let state = StateId(self.next_state);
        self.next_state += 1;
        state
    }

    fn bump(&mut self, label: &'static str) -> Outcome {
        self.revision = Revision(self.revision.0 + 1);
        Outcome {
            label,
            revision: self.revision,
        }
    }
}
