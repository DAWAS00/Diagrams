//! Reusable widgets built on the design tokens (UI-01).
//!
//! Each component gives its control an accessible name and the planned
//! sizes. Tabs arrive with the diagram tab row in UI-05.

use std::sync::Arc;

use eframe::egui::text::{LayoutJob, TextWrapping};
use eframe::egui::{
    self, Button, Color32, FontId, Galley, Label, Response, RichText, Role, ScrollArea, TextEdit,
    Vec2,
};

use super::theme::{CONTROL_HEIGHT, ROW_HEIGHT, TextRole};

/// Page title such as the welcome screen heading.
pub fn page_title(ui: &mut egui::Ui, text: &str) -> Response {
    ui.label(
        RichText::new(text)
            .text_style(TextRole::PageTitle.text_style())
            .strong(),
    )
}

/// Panel title such as "Project" or "Selection".
pub fn panel_title(ui: &mut egui::Ui, text: &str) -> Response {
    ui.label(
        RichText::new(text)
            .text_style(TextRole::PanelTitle.text_style())
            .strong(),
    )
}

/// Small secondary label naming a section or a field.
pub fn field_label(ui: &mut egui::Ui, text: &str) -> Response {
    ui.label(
        RichText::new(text)
            .text_style(TextRole::Label.text_style())
            .weak(),
    )
}

/// Code-like text such as a member line.
pub fn code_text(ui: &mut egui::Ui, text: &str) -> Response {
    ui.label(RichText::new(text).text_style(TextRole::Code.text_style()))
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

/// Selectable list row with at least the minimum hit height. Long text is
/// shortened with an ellipsis on one line; the full text stays the row's
/// accessible name.
pub fn list_row(ui: &mut egui::Ui, selected: bool, text: &str) -> Response {
    ui.add(
        Button::selectable(selected, text)
            .truncate()
            .min_size(Vec2::new(0.0, ROW_HEIGHT)),
    )
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

/// Labeled text field with a problem: a danger-colored border and the
/// message as text below it. The field keeps its normal focus highlight.
///
/// The border is changed in place rather than in a nested scope: a scope
/// would give the field a different id, so it would lose keyboard focus the
/// moment the error appears.
pub fn text_field_with_error(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut String,
    error: &str,
) -> Response {
    let danger = ui.visuals().error_fg_color;
    let saved = ui.visuals().widgets.clone();
    {
        let widgets = &mut ui.visuals_mut().widgets;
        for state in [&mut widgets.inactive, &mut widgets.hovered] {
            state.bg_stroke = egui::Stroke::new(1.5, danger);
        }
    }
    let response = text_field(ui, label, value);
    ui.visuals_mut().widgets = saved;
    error_text(ui, error);
    response
}

/// Error shown as text with an "Error:" prefix, never by color alone.
pub fn error_text(ui: &mut egui::Ui, text: &str) -> Response {
    ui.label(RichText::new(format!("Error: {text}")).color(ui.visuals().error_fg_color))
}

/// Warning shown as text with a "Warning:" prefix, never by color alone.
pub fn warning_text(ui: &mut egui::Ui, text: &str) -> Response {
    ui.label(RichText::new(format!("Warning: {text}")).color(ui.visuals().warn_fg_color))
}

/// A toggle or one option of a segmented choice, such as a panel button or
/// the theme selector. It keeps its control border when not selected, so it
/// never looks like plain text.
pub fn choice(ui: &mut egui::Ui, selected: bool, text: &str) -> Response {
    ui.add(Button::selectable(selected, text).frame_when_inactive(true))
}

/// Text that shortens with an ellipsis instead of pushing other controls away.
pub fn truncated_text(ui: &mut egui::Ui, text: &str) -> Response {
    ui.add(Label::new(RichText::new(text).text_style(TextRole::Label.text_style())).truncate())
}

/// Lays out painted text on one line no wider than `max_width`, ending in an
/// ellipsis when shortened. Use it for text drawn inside fixed shapes; keep the
/// full text in the shape's accessible name.
pub fn one_line_galley(
    painter: &egui::Painter,
    text: &str,
    font: FontId,
    color: Color32,
    max_width: f32,
) -> Arc<Galley> {
    let mut job = LayoutJob::simple_singleline(text.to_owned(), font, color);
    job.wrap = TextWrapping::truncate_at_width(max_width);
    painter.layout_job(job)
}

/// Wraps panel contents in a vertical scroll area that assistive technology
/// (and the usability audit) sees as a named, clipping scroll region.
pub(crate) fn scrolling(
    ui: &mut egui::Ui,
    id: &str,
    label: &str,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    ui.scope(|ui| {
        let region = ui.unique_id();
        let output = ScrollArea::vertical()
            .id_salt(id)
            .auto_shrink([false, false])
            .show(ui, add_contents);
        // Same point coordinates egui uses for widget bounds.
        let visible = output.inner_rect;
        ui.ctx().accesskit_node_builder(region, |node| {
            node.set_role(Role::ScrollView);
            node.set_label(label);
            node.set_clips_children();
            node.set_bounds(egui::accesskit::Rect {
                x0: visible.min.x.into(),
                y0: visible.min.y.into(),
                x1: visible.max.x.into(),
                y1: visible.max.y.into(),
            });
        });
    });
}
