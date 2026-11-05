//! Picking: screen-sized tolerance, stacking order, marquee enclosure.

use diagram_canvas::Viewport;
use diagram_canvas::picking::{self, HIT_TOLERANCE};
use diagram_core::{Point, Rect, Size};

/// A rectangle from literal values; every literal in these tests is valid.
fn rect(x: f64, y: f64, width: f64, height: f64) -> Rect {
    Rect::from_xywh(x, y, width, height).unwrap_or(Rect::new(Point::ORIGIN, Size::UNIT))
}

fn point(x: f64, y: f64) -> Point {
    Point::new(x, y).unwrap_or(Point::ORIGIN)
}

fn at_zoom(zoom: f64) -> Viewport {
    Viewport::new(Point::ORIGIN, zoom).unwrap_or_default()
}

#[test]
fn the_topmost_overlapping_item_should_win() {
    let items = [rect(0.0, 0.0, 100.0, 100.0), rect(50.0, 50.0, 100.0, 100.0)];
    assert_eq!(
        picking::pick(&items, at_zoom(1.0), point(75.0, 75.0)),
        Some(1)
    );
    assert_eq!(
        picking::pick(&items, at_zoom(1.0), point(10.0, 10.0)),
        Some(0)
    );
    assert_eq!(
        picking::pick(&items, at_zoom(1.0), point(500.0, 500.0)),
        None
    );
}

#[test]
fn tolerance_should_be_the_same_in_screen_points_at_every_zoom() {
    let items = [rect(100.0, 100.0, 50.0, 50.0)];
    for zoom in [0.1, 1.0, 4.0] {
        let viewport = at_zoom(zoom);
        // Just outside the left edge on screen, within the tolerance.
        let edge = viewport.to_screen(point(100.0, 120.0));
        let near = point(edge.x() - (HIT_TOLERANCE - 1.0), edge.y());
        let far = point(edge.x() - (HIT_TOLERANCE + 1.0), edge.y());
        assert_eq!(picking::pick(&items, viewport, near), Some(0), "{zoom}");
        assert_eq!(picking::pick(&items, viewport, far), None, "{zoom}");
    }
}

#[test]
fn marquee_should_select_only_fully_enclosed_items() {
    let items = [
        rect(10.0, 10.0, 20.0, 20.0),
        rect(40.0, 10.0, 20.0, 20.0),
        rect(200.0, 200.0, 20.0, 20.0),
    ];
    let viewport = at_zoom(1.0);
    // Dragged right-to-left: corner order does not matter.
    let hits = picking::enclosed(&items, viewport, point(70.0, 0.0), point(0.0, 40.0));
    assert_eq!(hits, [0, 1]);
    // Partly covering the third item is not enough.
    let partial = picking::enclosed(&items, viewport, point(190.0, 190.0), point(210.0, 210.0));
    assert!(partial.is_empty());
    // A click (no area) encloses nothing.
    assert!(picking::enclosed(&items, viewport, point(5.0, 5.0), point(5.0, 5.0)).is_empty());
}

#[test]
fn marquee_should_work_in_world_units_when_zoomed() {
    let items = [rect(100.0, 100.0, 50.0, 50.0)];
    let viewport = at_zoom(0.5);
    // On screen the item spans 50..75.
    assert_eq!(
        picking::enclosed(&items, viewport, point(45.0, 45.0), point(80.0, 80.0)),
        [0]
    );
}
