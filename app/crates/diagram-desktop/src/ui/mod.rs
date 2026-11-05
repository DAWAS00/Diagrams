//! Building blocks of the native UI, grown task by task from plan-rust-diagram-editor/UI-DESIGN-AND-INTEGRATION.md.
//!
//! The view modules (`header`, `tabs`, `toolbar`, `navigator`,
//! `canvas_view`, `inspector`, `status`, `dialogs` and `search_view`) draw a read-only presentation and push `UiIntent`s; they
//! never change application state.

pub mod commands;
pub mod components;
pub mod fixtures;
pub mod palette;
pub mod showcase;
pub mod theme;
pub mod welcome;
pub mod workspace;
