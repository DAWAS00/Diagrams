//! UI-independent document values and validation for Diagram Studio.
//!
//! The initial foundation provides a validated [`ProjectName`]. Model IDs,
//! commands, history and diagrams will be added in M1 after the native GUI gate.
//! This crate must remain independent of graphics, databases and OS dialogs.

#![deny(missing_docs)]

mod project_name;

pub use project_name::{InvalidProjectName, ProjectName};
