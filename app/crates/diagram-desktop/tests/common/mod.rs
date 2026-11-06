//! Helpers shared by the headless UI test files.
//!
//! Each test file declares `mod common;` and imports what it needs. Controls
//! are always found by name or role; never add coordinates here.

#![allow(dead_code)] // each test binary uses only some of these helpers

use diagram_desktop::shell::{self, ShellState};
use eframe::egui::Vec2;
use egui_kittest::Harness;

/// Window size most tests use.
pub const LARGE: (f32, f32) = (1280.0, 800.0);

/// A harness of `size` showing `state`, with the first frame already run.
pub fn harness_with(size: (f32, f32), state: ShellState) -> Harness<'static, ShellState> {
    let mut harness = Harness::builder()
        .with_size(Vec2::new(size.0, size.1))
        .build_ui_state(shell::show, state);
    harness.run();
    harness
}

/// The default sample project at 1280×800.
pub fn harness() -> Harness<'static, ShellState> {
    harness_with(LARGE, ShellState::default())
}

/// The default sample project at `width` × `height`.
pub fn harness_at(width: f32, height: f32) -> Harness<'static, ShellState> {
    harness_with((width, height), ShellState::default())
}

/// The default sample project at 1280×800, built but with no frame run yet.
pub fn harness_unrun() -> Harness<'static, ShellState> {
    Harness::builder()
        .with_size(Vec2::new(LARGE.0, LARGE.1))
        .build_ui_state(shell::show, ShellState::default())
}
