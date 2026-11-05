//! Board drawing (spec §4, brief §A6): painting and hit regions only.
//!
//! The view paints the class boxes it is given, at the screen rectangles in
//! [`CanvasView`] (relative to the board's top-left corner), and publishes an
//! accessibility node for the board and for every box. A click on a box
//! pushes `Select` (Shift toggles); everything else on the board is reported
//! as `Pointer` events. Hit testing beyond "which box rectangle is under the
//! pointer" and all transforms are Agent B's.
//!
//! Diagram objects use document colors ([`theme::DOCUMENT_DEFAULT_FILL`] and
//! [`theme::DOCUMENT_DEFAULT_INK`] until the presentation carries a document
//! style); selection and hover are editor overlays in the theme accent.

use eframe::egui::{
    self, Align2, CornerRadius, Pos2, Rect, Role, Sense, Stroke, StrokeKind, Vec2, WidgetInfo,
};

use super::components;
use super::theme::{self, Palette, TextRole};
use crate::app_state::{ATTRIBUTE_HEIGHT, CLASS_HEADER_HEIGHT};
use crate::intent::{CanvasPointer, PointerKind, Target, UiIntent};
use crate::presentation::{CanvasView, NodeView};

const TEXT_INSET: f32 = 10.0;
/// Size of a selection handle. Handles mark selection by shape, not color alone.
const HANDLE: f32 = 8.0;

/// Draws the board into all of `ui`'s available space.
pub fn show(ui: &mut egui::Ui, view: &CanvasView, out: &mut Vec<UiIntent>) {
    let palette = *theme::palette(ui.ctx().theme());
    let (board, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
    board.widget_info(|| WidgetInfo::labeled(Role::Canvas, true, view.accessible_name));
    painter.rect_filled(board.rect, CornerRadius::ZERO, palette.canvas);
    if let Some(text) = view.empty_text {
        let galley = components::one_line_galley(
            &painter,
            text,
            TextRole::Body.font_id(),
            palette.text_secondary,
            board.rect.width() - 2.0 * theme::SPACING,
        );
        let origin = Align2::CENTER_CENTER
            .anchor_size(board.rect.center(), galley.size())
            .min;
        painter.galley(origin, galley, palette.text_secondary);
    }
    let origin = board.rect.min;
    let node_rect =
        |node: &NodeView| Rect::from_min_size(origin + node.position.to_vec2(), node.size);

    let mut clicked = None;
    for (index, node) in view.nodes.iter().enumerate() {
        let rect = node_rect(node);
        let response = ui.interact(
            rect,
            ui.scope_id().with(("board-node", index)),
            Sense::click(),
        );
        response.widget_info(|| {
            WidgetInfo::selected(Role::Button, true, node.selected, &node.accessible_name)
        });
        if response.clicked() {
            clicked = Some(node.target);
        }
        paint_node(&painter, rect, node, response.hovered(), &palette);
    }

    let modifiers = ui.input(|input| input.modifiers);
    let hit_at = |position: Pos2| -> Option<Target> {
        view.nodes
            .iter()
            .rev()
            .find(|node| node_rect(node).contains(position))
            .map(|node| node.target)
    };
    let mut events: Vec<(PointerKind, Pos2)> = Vec::new();
    let mut pointer = |kind, position: Pos2| events.push((kind, position));
    let position = board.interact_pointer_pos();
    if let Some(target) = clicked {
        out.push(UiIntent::Select {
            target,
            toggle: modifiers.shift,
        });
    } else if board.clicked() {
        let at = position.unwrap_or(origin);
        pointer(PointerKind::Down, at);
        pointer(PointerKind::Up, at);
    }
    // egui recognizes a drag only after the pointer has moved past a
    // threshold; report Down where the button went down, or a pan loses its
    // first step.
    let press_origin = ui.input(|input| input.pointer.press_origin());
    if board.drag_started()
        && let Some(at) = press_origin.or(position)
    {
        pointer(PointerKind::Down, at);
    } else if board.dragged()
        && let Some(at) = position
    {
        pointer(PointerKind::Move, at);
    }
    if board.drag_stopped() {
        pointer(PointerKind::Up, position.unwrap_or(origin));
    }
    if board.hovered() && modifiers.command {
        let zoom = ui.input(|input| input.zoom_delta());
        if (zoom - 1.0).abs() > f32::EPSILON {
            let at = ui
                .input(|input| input.pointer.hover_pos())
                .unwrap_or(origin);
            pointer(PointerKind::Zoom { delta: zoom }, at);
        }
    }
    out.extend(events.into_iter().map(|(kind, position)| {
        UiIntent::Pointer(CanvasPointer {
            kind,
            position,
            modifiers,
            hit: hit_at(position),
        })
    }));
}

fn paint_node(
    painter: &egui::Painter,
    rect: Rect,
    node: &NodeView,
    hovered: bool,
    palette: &Palette,
) {
    let ink = theme::DOCUMENT_DEFAULT_INK;
    let border = if node.selected || hovered {
        palette.accent
    } else {
        ink
    };
    let width = if node.selected { 2.5 } else { 1.0 };
    painter.rect(
        rect,
        CornerRadius::same(4),
        theme::DOCUMENT_DEFAULT_FILL,
        Stroke::new(width, border),
        StrokeKind::Inside,
    );
    let header_bottom = rect.top() + CLASS_HEADER_HEIGHT;
    painter.hline(rect.x_range(), header_bottom, Stroke::new(1.0, ink));
    // Text stays inside the box on one line; the full name is the box's
    // accessible name and is editable in the inspector.
    let text_width = rect.width() - 2.0 * TEXT_INSET;
    let name = components::one_line_galley(
        painter,
        &node.name,
        TextRole::Body.font_id(),
        ink,
        text_width,
    );
    let name_origin = Align2::CENTER_CENTER
        .anchor_size(
            Pos2::new(rect.center().x, rect.top() + CLASS_HEADER_HEIGHT / 2.0),
            name.size(),
        )
        .min;
    painter.galley(name_origin, name, ink);
    for (row, attribute) in node.attributes.iter().enumerate() {
        let line = components::one_line_galley(
            painter,
            attribute,
            TextRole::Code.font_id(),
            ink,
            text_width,
        );
        let top = header_bottom + 6.0 + row as f32 * ATTRIBUTE_HEIGHT;
        painter.galley(Pos2::new(rect.left() + TEXT_INSET, top), line, ink);
    }
    if node.selected {
        for corner in [
            rect.left_top(),
            rect.right_top(),
            rect.left_bottom(),
            rect.right_bottom(),
        ] {
            painter.rect(
                Rect::from_center_size(corner, Vec2::splat(HANDLE)),
                CornerRadius::ZERO,
                palette.panel,
                Stroke::new(1.5, palette.accent),
                StrokeKind::Middle,
            );
        }
    }
}
