//! UI-independent document values and validation for Diagram Studio.
//!
//! - [`ids`]: typed identities that cannot be interchanged (R1-01).
//! - [`model`]: the shared model (elements, members, relations) kept apart
//!   from diagram views, with finite [`geometry`] (R1-01).
//! - [`validate`]: typed domain errors and whole-document checks,
//!   including generalization cycles (R1-02).
//! - [`command`]: atomic, reversible commands with labels (R1-03).
//! - [`history`]: undo, redo, revisions and saved-state identity (R1-04).
//! - [`notation`]: what each diagram notation offers (UI-11).
//!
//! This crate must remain independent of graphics, databases and OS dialogs.

#![deny(missing_docs)]

pub mod command;
pub mod geometry;
pub mod history;
pub mod ids;
pub mod model;
pub mod notation;
mod project_name;
pub mod validate;

pub use geometry::{GeometryError, Point, Rect, Size};
pub use project_name::{InvalidProjectName, ProjectName};
