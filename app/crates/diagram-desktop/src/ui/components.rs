//! Reusable widgets built on the design tokens (UI-01).
//!
//! Each component gives its control an accessible name and the planned
//! sizes. Tabs arrive with the diagram tab row in UI-05.

use std::sync::Arc;

use eframe::egui::text::{LayoutJob, TextWrapping};
use eframe::egui::{
    self, Button, Color32, FontId, Galley, Label, Response, RichText, TextEdit, Vec2,
};

use super::theme::{CONTROL_HEIGHT, ROW_HEIGHT, TextRole};
use crate::presentation::CommandView;
use crate::ui::commands::CommandId;

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

/// Hover text of a command control: its reason when disabled, otherwise its
/// shortcut. Both texts come from the command catalog.
fn command_hover(response: Response, command: &CommandView) -> Response {
    let shortcut = command
        .shortcut_text
        .as_deref()
        .map(|shortcut| format!("Shortcut: {shortcut}"));
    match (command.availability.reason(), shortcut) {
        (Some(reason), Some(shortcut)) => {
            response.on_disabled_hover_text(format!("{reason} ({shortcut})"))
        }
        (Some(reason), None) => response.on_disabled_hover_text(reason),
        (None, Some(shortcut)) => response.on_hover_text(shortcut),
        (None, None) => response,
    }
}

/// A button for a catalog command. It is drawn pressed when the command is
/// active, keeps its border when not, and explains a disabled state on
/// hover. Returns true when it was activated (click, Enter or Space).
pub fn command_button(ui: &mut egui::Ui, command: &CommandView) -> bool {
    command_button_named(ui, command, &command.label, None)
}

/// [`command_button`] with a different visible text and, optionally, a
/// longer accessible name (for example "Up" read as "Move attribute id up").
pub fn command_button_named(
    ui: &mut egui::Ui,
    command: &CommandView,
    text: &str,
    accessible_name: Option<&str>,
) -> bool {
    let pressed = command.active == Some(true);
    let button = Button::selectable(pressed, text)
        .frame_when_inactive(true)
        .min_size(Vec2::new(0.0, CONTROL_HEIGHT));
    let response = ui.add_enabled(command.is_enabled(), button);
    if let Some(name) = accessible_name {
        let enabled = command.is_enabled();
        let name = name.to_owned();
        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::Role::Button, enabled, pressed, &name)
        });
    }
    command_hover(response, command).clicked()
}

/// A button that opens a menu, with a painted down arrow after its label
/// (the default fonts have no "▾" glyph). The label is the accessible name.
pub fn dropdown_button<R>(
    ui: &mut egui::Ui,
    label: &str,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> (Response, Option<R>) {
    let arrow = egui::AtomKind::closure(|ui, _args| {
        let size = Vec2::new(ui.spacing().icon_width * 0.8, ui.spacing().icon_width);
        egui::IntoSizedResult {
            intrinsic_size: size,
            sized: egui::SizedAtomKind::Paint {
                paint: Arc::new(|ui: &egui::Ui, args: egui::AtomPaintArgs| {
                    let rect = egui::Rect::from_center_size(
                        args.rect.center(),
                        Vec2::new(args.rect.width(), args.rect.width() * 0.6),
                    );
                    ui.painter().add(egui::Shape::convex_polygon(
                        vec![rect.left_top(), rect.right_top(), rect.center_bottom()],
                        args.fallback_text_color,
                        egui::Stroke::NONE,
                    ));
                }),
                size,
            },
        }
    });
    let button = Button::new((label, egui::Atom::from(arrow)))
        .frame_when_inactive(true)
        .min_size(Vec2::new(0.0, CONTROL_HEIGHT));
    let (response, inner) = egui::containers::menu::MenuButton::from_button(button).ui(ui, |ui| {
        ui.set_min_width(220.0);
        add_contents(ui)
    });
    (response, inner.map(|inner| inner.inner))
}

/// Right-hand text of a disabled menu item. The full reason is the item's
/// hover text, and command search lists it as visible text.
pub const UNAVAILABLE: &str = "Unavailable";

/// Menu items for catalog commands, one row each: the label, then on the
/// right the shortcut text, or [`UNAVAILABLE`] for a disabled item, whose
/// reason is its hover text. Rows never grow, so long groups still fit at
/// 800×500. Returns the command chosen and closes the menu when one is.
pub fn menu_items(ui: &mut egui::Ui, items: &[CommandView]) -> Option<CommandId> {
    let mut chosen = None;
    for item in items {
        let mut button = Button::selectable(item.active == Some(true), item.label.as_str())
            .min_size(Vec2::new(ui.available_width(), ROW_HEIGHT));
        let right = if item.is_enabled() {
            item.shortcut_text.as_deref()
        } else {
            Some(UNAVAILABLE)
        };
        if let Some(right) = right {
            button = button.shortcut_text(right);
        }
        let response = ui.add_enabled(item.is_enabled(), button);
        let response = command_hover(response, item);
        if response.clicked() {
            chosen = Some(item.id);
        }
    }
    if chosen.is_some() {
        ui.close();
    }
    chosen
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
