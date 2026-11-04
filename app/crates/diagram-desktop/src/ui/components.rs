//! Reusable widgets built on the design tokens (UI-01).
//!
//! Each component gives its control an accessible name and the planned
//! sizes. Tabs arrive with the diagram tab row in UI-05.

use eframe::egui::{self, Button, Label, Response, RichText, TextEdit, Vec2};

use super::theme::{CONTROL_HEIGHT, ROW_HEIGHT};

/// Panel title such as "Project" or "Selection".
pub fn panel_title(ui: &mut egui::Ui, text: &str) -> Response {
    ui.label(RichText::new(text).heading().strong())
}

/// Small secondary label naming a section or a field.
pub fn field_label(ui: &mut egui::Ui, text: &str) -> Response {
    ui.label(RichText::new(text).weak())
}

/// Labeled single-line text field; the label becomes its accessible name.
pub fn text_field(ui: &mut egui::Ui, label: &str, value: &mut String) -> Response {
    let label = field_label(ui, label);
    ui.add(
        TextEdit::singleline(value)
            .desired_width(f32::INFINITY)
            .min_size(Vec2::new(0.0, CONTROL_HEIGHT)),
    )
    .labelled_by(label.id)
}

/// Selectable list row with at least the minimum hit height.
pub fn list_row(ui: &mut egui::Ui, selected: bool, text: &str) -> Response {
    ui.add(Button::selectable(selected, text).min_size(Vec2::new(0.0, ROW_HEIGHT)))
}

/// Toolbar or dialog action. A disabled action always explains why;
/// `hint` (for example a shortcut) is shown on hover.
pub fn action_button(
    ui: &mut egui::Ui,
    label: &str,
    disabled_reason: Option<&str>,
    hint: Option<&str>,
) -> Response {
    let button = Button::new(label).min_size(Vec2::new(0.0, CONTROL_HEIGHT));
    let response = ui.add_enabled(disabled_reason.is_none(), button);
    let tooltip = |text: &str| match hint {
        Some(hint) => format!("{text} ({hint})"),
        None => text.to_owned(),
    };
    match (disabled_reason, hint) {
        (Some(reason), _) => response.on_disabled_hover_text(tooltip(reason)),
        (None, Some(hint)) => response.on_hover_text(hint),
        (None, None) => response,
    }
}

/// Warning shown as text with a "Warning:" prefix, never by color alone.
pub fn warning_text(ui: &mut egui::Ui, text: &str) -> Response {
    ui.label(RichText::new(format!("Warning: {text}")).color(ui.visuals().warn_fg_color))
}

/// One option of a segmented choice such as the theme selector.
pub fn choice(ui: &mut egui::Ui, selected: bool, text: &str) -> Response {
    ui.add(Button::selectable(selected, text))
}

/// Text that shortens with an ellipsis instead of pushing other controls away.
pub fn truncated_text(ui: &mut egui::Ui, text: &str) -> Response {
    ui.add(Label::new(text).truncate())
}
