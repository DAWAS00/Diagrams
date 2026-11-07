//! Component showcase (UI-01): every reusable control in each state, shown
//! as a clearly labeled design fixture. Nothing here changes the project.

use eframe::egui::{self, Id, ScrollArea};

use super::components;
use super::intents::UiIntent;
use crate::shell::ShellState;

/// Editable values used by the showcase's own controls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShowcaseState {
    /// Value of the normal text field.
    pub text: String,
    /// Value of the field that shows an error.
    pub invalid: String,
    /// Selected option of the segmented choice.
    pub choice: usize,
    /// Selected list row.
    pub row: usize,
}

impl Default for ShowcaseState {
    fn default() -> Self {
        Self {
            text: String::from("Customer"),
            invalid: String::new(),
            choice: 0,
            row: 0,
        }
    }
}

/// Draws every component and state.
pub fn show(ui: &mut egui::Ui, state: &mut ShowcaseState) {
    components::field_label(
        ui,
        "Design fixture: these controls change nothing in the project.",
    );

    components::panel_title(ui, "Text roles");
    components::page_title(ui, "Page title");
    ui.label("Body text explains what something does.");
    components::field_label(ui, "Label text names a field");
    components::code_text(ui, "code: MemberType");
    components::truncated_text(
        ui,
        "Long text shortens with an ellipsis instead of pushing controls away from view",
    );

    components::panel_title(ui, "Buttons");
    ui.horizontal(|ui| {
        let _ = components::action_button(ui, "Enabled action", None, Some("Shortcut: Ctrl+E"));
        let _ = components::action_button(
            ui,
            "Disabled action",
            Some("Not available: disabled actions always explain why."),
            None,
        );
    });

    components::panel_title(ui, "Choices and rows");
    ui.horizontal(|ui| {
        for (index, label) in ["First option", "Second option"].into_iter().enumerate() {
            if components::choice(ui, state.choice == index, label).clicked() {
                state.choice = index;
            }
        }
    });
    for (index, label) in ["Selected row", "Another row"].into_iter().enumerate() {
        if components::list_row(ui, state.row == index, label).clicked() {
            state.row = index;
        }
    }

    components::panel_title(ui, "Fields");
    components::text_field(ui, "Text field", &mut state.text);
    components::text_field_with_error(
        ui,
        "Field with an error",
        &mut state.invalid,
        "A name must contain a non-whitespace character.",
    );
    ui.add_enabled_ui(false, |ui| {
        let mut disabled = String::from("Read-only value");
        components::text_field(ui, "Disabled field", &mut disabled);
    });

    components::panel_title(ui, "Messages");
    components::warning_text(ui, "This needs attention but nothing failed.");
    components::error_text(ui, "This failed; the message says what to do next.");
}

/// The component showcase as a movable window with an explicit Close button.
pub(crate) fn showcase_window(ctx: &egui::Context, state: &mut ShellState) {
    let Some(mut fixture) = state.showcase.take() else {
        return;
    };
    let mut close = false;
    egui::Window::new("Component showcase")
        .id(Id::unique("component-showcase"))
        .collapsible(false)
        .default_size([460.0, 640.0])
        .default_pos([260.0, 60.0])
        .max_height((ctx.content_rect().height() - 80.0).max(200.0))
        .show(ctx, |ui| {
            if components::action_button(ui, "Close showcase", None, Some("Escape")).clicked() {
                close = true;
            }
            ui.separator();
            ScrollArea::vertical().show(ui, |ui| show(ui, &mut fixture));
        });
    state.showcase = Some(fixture);
    if close {
        // Closing goes through the coordinator like every other change.
        state.queue(UiIntent::CloseShowcase);
    }
}
