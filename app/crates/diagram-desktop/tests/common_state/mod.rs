//! Shared helpers for Agent B's state tests: sample state, applying intents,
//! input contexts and key presses.

#![allow(dead_code, reason = "each test file uses a different subset")]

use diagram_desktop::app_state::AppState;
use diagram_desktop::dispatch;
use diagram_desktop::intent::{InputContext, Region, UiIntent};
use eframe::egui::{self, Key, Modifiers};

/// The sample project with nothing selected.
pub fn sample() -> AppState {
    AppState::default()
}

/// Applies `intents` in order with a plain input context.
pub fn apply_all(state: &mut AppState, intents: impl IntoIterator<Item = UiIntent>) {
    dispatch::apply(state, &InputContext::default(), intents);
}

/// Applies `intents` with the given input context.
pub fn apply_with(
    state: &mut AppState,
    input: &InputContext,
    intents: impl IntoIterator<Item = UiIntent>,
) {
    dispatch::apply(state, input, intents);
}

/// The canvas has focus; nobody is typing.
pub fn canvas_focused() -> InputContext {
    InputContext {
        focused_region: Region::Board,
        ..InputContext::default()
    }
}

/// A text field has focus.
pub fn typing() -> InputContext {
    InputContext {
        text_had_focus: true,
        ..canvas_focused()
    }
}

/// An input method is composing text.
pub fn ime_composing() -> InputContext {
    InputContext {
        ime_composing: true,
        ..canvas_focused()
    }
}

/// A modal dialog is open.
pub fn modal_open() -> InputContext {
    InputContext {
        modal_open: true,
        ..canvas_focused()
    }
}

/// Focus is in `region`.
pub fn in_region(region: Region) -> InputContext {
    InputContext {
        focused_region: region,
        ..InputContext::default()
    }
}

/// A key-down event with modifiers.
pub fn key(modifiers: Modifiers, key: Key) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    }
}

/// An input state holding exactly `events`.
pub fn input_with(events: Vec<egui::Event>) -> egui::InputState {
    let raw = egui::RawInput {
        events,
        ..egui::RawInput::default()
    };
    let mut input = egui::InputState::default();
    input = input.begin_pass(raw, false, 1.0, egui::InputOptions::default());
    input
}
