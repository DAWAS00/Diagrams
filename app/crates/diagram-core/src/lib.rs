//! UI-independent document values and validation for Diagram Studio.
//!
//! Provides a validated [`ProjectName`] and finite world [`geometry`]. Model
//! IDs, commands, history and diagrams are added in M1 (R1-01 to R1-04).
//! This crate must remain independent of graphics, databases and OS dialogs.

#![deny(missing_docs)]

pub mod geometry;
pub mod notation;
mod project_name;

pub use geometry::{GeometryError, Point, Rect, Size};
pub use project_name::{InvalidProjectName, ProjectName};
