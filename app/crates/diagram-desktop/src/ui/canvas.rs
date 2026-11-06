//! The diagram canvas: class boxes, relationship lines and click selection.
//!
//! Painting reads the read model and queues intents; it never changes records.

use eframe::egui::{
    self, Align2, Color32, CornerRadius, Pos2, Rect, Role, Sense, Stroke, StrokeKind, Vec2,
    WidgetInfo,
};

use diagram_core::ids::ElementId;

use crate::shell::ShellState;
use crate::ui::commands::CommandId;
use crate::ui::components;
use crate::ui::intents::UiIntent;
use crate::ui::read_model::{self, ClassCard, EdgeCard};
use crate::ui::theme::{Palette, TextRole};

const CLASS_WIDTH: f32 = 180.0;
const HEADER_HEIGHT: f32 = 32.0;
const ATTRIBUTE_HEIGHT: f32 = 20.0;
const TEXT_INSET: f32 = 10.0;

/// Document colors for diagram objects. They do not follow the editor theme,
/// so switching themes never changes how a diagram looks when exported.
const DOCUMENT_FILL: Color32 = Color32::from_rgb(0xFA, 0xFC, 0xFF);
const DOCUMENT_INK: Color32 = Color32::from_rgb(0x22, 0x31, 0x47);

/// Paints the active diagram and queues selection intents for clicks.
pub(crate) fn canvas(ui: &mut egui::Ui, state: &mut ShellState, palette: &Palette) -> Rect {
    let (background, painter) = ui.allocate_painter(ui.available_size(), Sense::click());
    background.widget_info(|| WidgetInfo::labeled(Role::Canvas, true, "Diagram canvas"));
    painter.rect_filled(background.rect, CornerRadius::ZERO, palette.canvas);
    let model = state
        .active_diagram()
        .map(|diagram| read_model::canvas_model(state.document(), diagram))
        .unwrap_or_default();
    let empty_message = match state.active_diagram() {
        None => Some("No diagram is open. Open one from the project panel."),
        Some(_) if model.classes.is_empty() => {
            Some("This diagram is empty. Adding classes arrives with the Class tool.")
        }
        Some(_) => None,
    };
    if let Some(message) = empty_message {
        painter.text(
            background.rect.center(),
            Align2::CENTER_CENTER,
            message,
            TextRole::Body.font_id(),
            palette.text_secondary,
        );
    }
    let origin = background.rect.min;

    let mut clicked_class = None;
    let chosen = state.selected_elements();
    let rect_of = |card: &ClassCard| {
        Rect::from_min_size(
            origin + Vec2::new(card.position.0, card.position.1),
            card_size(card),
        )
    };
    let rect_for = |element: ElementId| {
        model
            .classes
            .iter()
            .find(|card| card.element == element)
            .map(rect_of)
    };
    // Relationship lines between visible classes, under the boxes.
    for edge in &model.edges {
        let (Some(source), Some(target)) = (rect_for(edge.source), rect_for(edge.target)) else {
            continue;
        };
        paint_relation(
            &painter,
            edge,
            source,
            target,
            state.selected_relation() == Some(edge.relation),
            palette,
        );
    }
    for card in &model.classes {
        let rect = rect_of(card);
        let selected = chosen.contains(&card.element);
        let response = ui.interact(
            rect,
            ui.scope_id().with(("canvas-class", card.element)),
            Sense::click(),
        );
        let name = &card.name;
        response.widget_info(|| {
            WidgetInfo::selected(Role::Button, true, selected, format!("Class {name}"))
        });
        if response.clicked() {
            clicked_class = Some(card.element);
        }
        paint_class(&painter, rect, card, selected, response.hovered(), palette);
    }

    let adding = ui.input(|input| input.modifiers.command);
    match clicked_class {
        Some(element) if adding => state.queue(UiIntent::ToggleClassInSelection(element)),
        Some(element) => state.queue(UiIntent::SelectClass(element)),
        None if background.clicked() => state.queue(UiIntent::run(CommandId::ClearSelection)),
        None => {}
    }
    background.rect
}

/// Size of a class box: fixed width, height from its lines.
pub(crate) fn card_size(card: &ClassCard) -> Vec2 {
    let rows = card.lines.len().max(1) as f32;
    Vec2::new(CLASS_WIDTH, HEADER_HEIGHT + rows * ATTRIBUTE_HEIGHT + 8.0)
}

/// A relationship as a line between box edges, with multiplicities near each end. Selected relationships are thicker and
/// use the accent colour; they are selected from the project panel.
fn paint_relation(
    painter: &egui::Painter,
    connector: &EdgeCard,
    source: Rect,
    target: Rect,
    selected: bool,
    palette: &Palette,
) {
    let (from, to) = (source.center(), target.center());
    let edge = |rect: Rect, toward: Pos2| {
        let direction = (toward - rect.center()).normalized();
        let scale_x = if direction.x.abs() > f32::EPSILON {
            rect.width() / 2.0 / direction.x.abs()
        } else {
            f32::INFINITY
        };
        let scale_y = if direction.y.abs() > f32::EPSILON {
            rect.height() / 2.0 / direction.y.abs()
        } else {
            f32::INFINITY
        };
        rect.center() + direction * scale_x.min(scale_y)
    };
    let (start, end) = (edge(source, to), edge(target, from));
    let (color, width) = if selected {
        (palette.accent, 2.5)
    } else {
        (palette.diagram_line, 1.0)
    };
    painter.line_segment([start, end], Stroke::new(width, color));
    // Multiplicities below the line near each end, so they never collide with it.
    let direction = (end - start).normalized();
    let normal = Vec2::new(-direction.y, direction.x);
    let above = if normal.y > 0.0 { -normal } else { normal };
    let font = TextRole::Code.font_id();
    let below = -above;
    painter.text(
        start + direction * 10.0 + below * 4.0,
        Align2::LEFT_TOP,
        &connector.source_multiplicity,
        font.clone(),
        color,
    );
    painter.text(
        end - direction * 10.0 + below * 4.0,
        Align2::RIGHT_TOP,
        &connector.target_multiplicity,
        font,
        color,
    );
}

fn paint_class(
    painter: &egui::Painter,
    rect: Rect,
    card: &ClassCard,
    selected: bool,
    hovered: bool,
    palette: &Palette,
) {
    // Selection and hover are editor overlays in the theme accent; the box
    // itself keeps its document colors.
    let border = if selected || hovered {
        palette.accent
    } else {
        DOCUMENT_INK
    };
    let width = if selected { 2.5 } else { 1.0 };

    painter.rect(
        rect,
        CornerRadius::same(4),
        DOCUMENT_FILL,
        Stroke::new(width, border),
        StrokeKind::Inside,
    );
    let header_bottom = rect.top() + HEADER_HEIGHT;
    painter.hline(
        rect.x_range(),
        header_bottom,
        Stroke::new(1.0, DOCUMENT_INK),
    );
    // Text stays inside the box; how boxes grow for long names is decided
    // with the canvas states in UI-07.
    let text_width = rect.width() - 2.0 * TEXT_INSET;
    let name = components::one_line_galley(
        painter,
        &card.name,
        TextRole::Body.font_id(),
        DOCUMENT_INK,
        text_width,
    );
    let name_origin = Align2::CENTER_CENTER
        .anchor_size(
            Pos2::new(rect.center().x, rect.top() + HEADER_HEIGHT / 2.0),
            name.size(),
        )
        .min;
    painter.galley(name_origin, name, DOCUMENT_INK);
    for (row, attribute) in card.lines.iter().enumerate() {
        let line = components::one_line_galley(
            painter,
            attribute,
            TextRole::Code.font_id(),
            DOCUMENT_INK,
            text_width,
        );
        let top = header_bottom + 6.0 + row as f32 * ATTRIBUTE_HEIGHT;
        painter.galley(Pos2::new(rect.left() + TEXT_INSET, top), line, DOCUMENT_INK);
    }

    if selected {
        // Corner handles mark selection without relying on colour alone.
        for corner in [
            rect.left_top(),
            rect.right_top(),
            rect.left_bottom(),
            rect.right_bottom(),
        ] {
            let handle = Rect::from_center_size(corner, Vec2::splat(8.0));
            painter.rect(
                handle,
                CornerRadius::ZERO,
                Color32::WHITE,
                Stroke::new(1.5, palette.accent),
                StrokeKind::Middle,
            );
        }
    }
}
