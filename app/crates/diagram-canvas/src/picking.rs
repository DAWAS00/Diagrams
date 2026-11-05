//! Hit testing in visible stacking order with a screen-sized tolerance.

use diagram_core::{Point, Rect};

use crate::viewport::Viewport;

/// Hit tolerance in screen points: the same on screen at every zoom.
pub const HIT_TOLERANCE: f64 = 4.0;

/// The topmost item under `screen` (relative to the canvas origin).
///
/// `items` are world rectangles in drawing order, so the last match is the
/// one drawn on top. The tolerance is converted from screen points to world
/// units, so it stays usable at 10% and at 400%.
pub fn pick(items: &[Rect], viewport: Viewport, screen: Point) -> Option<usize> {
    let world = viewport.to_world(screen).ok()?;
    let margin = HIT_TOLERANCE / viewport.zoom();
    items
        .iter()
        .rposition(|rect| rect.contains_with_margin(world, margin))
}

/// Items completely inside the marquee spanned by two screen points, in
/// drawing order. A marquee without area encloses nothing.
pub fn enclosed(items: &[Rect], viewport: Viewport, from: Point, to: Point) -> Vec<usize> {
    let (Ok(start), Ok(end)) = (viewport.to_world(from), viewport.to_world(to)) else {
        return Vec::new();
    };
    let Ok(marquee) = Rect::spanning(start, end) else {
        return Vec::new();
    };
    items
        .iter()
        .enumerate()
        .filter(|(_, rect)| marquee.encloses(**rect))
        .map(|(index, _)| index)
        .collect()
}
