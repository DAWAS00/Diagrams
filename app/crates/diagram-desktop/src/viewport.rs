//! Zoom, fit and pan for the active tab, on top of [`diagram_canvas::Viewport`].
//!
//! Keyboard and menu zoom anchor at the canvas center; pointer zoom anchors
//! at the cursor. None of these change the document: they only replace the
//! active tab's viewport.

use diagram_canvas::{Viewport, ZOOM_STEP};
use diagram_core::{Point, Rect};

use crate::app_state::PreviewClass;
use crate::editor_state::EditorState;

/// A class box's bounds in world units, if its geometry is valid.
pub fn class_rect(class: &PreviewClass) -> Option<Rect> {
    let size = class.size();
    Rect::from_xywh(
        f64::from(class.position.x),
        f64::from(class.position.y),
        f64::from(size.x),
        f64::from(size.y),
    )
    .ok()
}

/// World bounds of the classes at `indices` (all classes when `None`).
pub fn bounds(classes: &[PreviewClass], indices: Option<&[usize]>) -> Option<Rect> {
    let rects = classes.iter().enumerate().filter_map(|(index, class)| {
        let wanted = indices.is_none_or(|indices| indices.contains(&index));
        wanted.then(|| class_rect(class)).flatten()
    });
    Rect::bounds(rects)
}

/// Applies `change` to the active viewport; a failed change leaves it as it was.
fn update(editor: &mut EditorState, change: impl FnOnce(Viewport) -> Option<Viewport>) {
    if let Some(viewport) = change(editor.viewport()) {
        editor.set_viewport(viewport);
    }
}

/// One zoom step in, around the canvas center.
pub fn zoom_in(editor: &mut EditorState) {
    let center = editor.canvas_center();
    update(editor, |viewport| {
        viewport.zoomed_by(ZOOM_STEP, center).ok()
    });
}

/// One zoom step out, around the canvas center.
pub fn zoom_out(editor: &mut EditorState) {
    let center = editor.canvas_center();
    update(editor, |viewport| {
        viewport.zoomed_by(1.0 / ZOOM_STEP, center).ok()
    });
}

/// 100%, keeping the world point at the canvas center in place.
pub fn actual_size(editor: &mut EditorState) {
    let center = editor.canvas_center();
    update(editor, |viewport| viewport.zoomed_to(1.0, center).ok());
}

/// Pointer zoom by `factor`, anchored at `anchor` (relative to the canvas origin).
pub fn zoom_at(editor: &mut EditorState, factor: f32, anchor: Point) {
    update(editor, |viewport| {
        viewport.zoomed_by(f64::from(factor), anchor).ok()
    });
}

/// Fits `content` (world bounds) in the canvas.
pub fn fit(editor: &mut EditorState, content: Option<Rect>) {
    let viewport = Viewport::fit(content, editor.canvas_size);
    editor.set_viewport(viewport);
}

/// Pans so `content` is centered, without changing the zoom.
pub fn center_on(editor: &mut EditorState, content: Rect) {
    let center = editor.canvas_center();
    update(editor, |viewport| {
        let shown = viewport.to_screen(content.center());
        viewport
            .panned(center.x() - shown.x(), center.y() - shown.y())
            .ok()
    });
}

/// Pans by a screen delta.
pub fn pan_by(editor: &mut EditorState, dx: f64, dy: f64) {
    update(editor, |viewport| viewport.panned(dx, dy).ok());
}
