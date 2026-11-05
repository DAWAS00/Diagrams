//! Building blocks of the native UI, grown task by task from plan-rust-diagram-editor/UI-DESIGN-AND-INTEGRATION.md.
//!
//! The view modules ([`header`], [`tabs`], [`toolbar`], [`navigator`],
//! [`canvas_view`], [`inspector`] and [`status`]) draw a read-only presentation and push `UiIntent`s; they
//! never change application state.

pub mod canvas_view;
pub mod commands;
pub mod components;
pub mod fixtures;
pub mod header;
pub mod inspector;
pub mod navigator;
pub mod palette;
pub mod showcase;
pub mod status;
pub mod tabs;
pub mod theme;
pub mod toolbar;
pub mod welcome;
pub mod workspace;
