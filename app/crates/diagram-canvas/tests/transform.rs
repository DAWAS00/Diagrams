//! World ↔ screen transform: inverse within tolerance, clamped zoom,
//! anchored zoom and fit that never moves geometry.

use diagram_canvas::{MAX_ZOOM, MIN_ZOOM, ScreenSize, Viewport};
use diagram_core::{GeometryError, Point, Rect};

fn point(x: f64, y: f64) -> Point {
    Point::new(x, y).unwrap_or(Point::ORIGIN)
}

fn close(first: Point, second: Point) -> bool {
    (first.x() - second.x()).abs() < 1e-9 && (first.y() - second.y()).abs() < 1e-9
}

#[test]
fn inverse_should_round_trip_at_10_100_and_400_percent() {
    for zoom in [0.1, 1.0, 4.0] {
        let viewport = viewport_at(point(37.5, -12.25), zoom);
        for world in [point(0.0, 0.0), point(120.5, 48.0), point(-300.0, 9000.0)] {
            let screen = viewport.to_screen(world);
            assert_eq!(
                viewport.to_world(screen).map(|back| close(back, world)),
                Ok(true)
            );
        }
    }
}

/// A viewport from literal values, checked to have the requested zoom.
fn viewport_at(pan: Point, zoom: f64) -> Viewport {
    let viewport = Viewport::new(pan, zoom).unwrap_or_default();
    assert_eq!(viewport.zoom(), zoom);
    viewport
}

#[test]
fn screen_should_equal_pan_plus_world_times_zoom() {
    let viewport = viewport_at(point(10.0, 20.0), 2.0);
    let screen = viewport.to_screen(point(3.0, 4.0));
    assert!(close(screen, point(16.0, 28.0)));
}

#[test]
fn zoom_should_clamp_to_10_to_400_percent() {
    let viewport = Viewport::default();
    let tiny = viewport
        .zoomed_by(0.0001, Point::ORIGIN)
        .map(Viewport::zoom);
    let huge = viewport
        .zoomed_by(1000.0, Point::ORIGIN)
        .map(Viewport::zoom);
    assert_eq!(tiny, Ok(MIN_ZOOM));
    assert_eq!(huge, Ok(MAX_ZOOM));
    assert_eq!(
        Viewport::new(Point::ORIGIN, 9.0).map(Viewport::zoom),
        Ok(MAX_ZOOM)
    );
}

#[test]
fn non_finite_zoom_should_be_rejected() {
    for zoom in [f64::NAN, f64::INFINITY, 0.0, -1.0] {
        assert_eq!(
            Viewport::new(Point::ORIGIN, zoom),
            Err(GeometryError::NotFinite),
            "{zoom}"
        );
        assert!(Viewport::default().zoomed_to(zoom, Point::ORIGIN).is_err());
    }
    assert!(Viewport::default().panned(f64::NAN, 0.0).is_err());
    assert!(Point::new(f64::INFINITY, 0.0).is_err());
}

#[test]
fn anchored_zoom_should_keep_the_world_point_under_the_anchor() {
    let anchor = point(400.0, 250.0);
    for start in [0.1, 0.5, 1.0, 3.0] {
        let viewport = viewport_at(point(-20.0, 15.0), start);
        let before = viewport.to_world(anchor);
        let zoomed = viewport.zoomed_by(1.25, anchor);
        let after = zoomed.and_then(|zoomed| zoomed.to_world(anchor));
        assert_eq!(
            before.and_then(|b| after.map(|a| close(a, b))),
            Ok(true),
            "{start}"
        );
    }
}

#[test]
fn fit_should_show_all_content_inside_the_screen() {
    let content = Rect::from_xywh(48.0, 48.0, 432.0, 120.0).ok();
    let screen = ScreenSize {
        width: 800.0,
        height: 500.0,
    };
    let viewport = Viewport::fit(content, screen);
    let shown = content.and_then(|rect| viewport.rect_to_screen(rect));
    assert!(shown.is_some_and(|rect| rect.left() >= 0.0
        && rect.top() >= 0.0
        && rect.right() <= 800.0
        && rect.bottom() <= 500.0));
    // Centered.
    let center = shown.map(Rect::center);
    assert!(center.is_some_and(|c| close(c, point(400.0, 250.0))));
}

#[test]
fn fit_of_nothing_should_reset_to_100_percent() {
    let viewport = Viewport::fit(
        None,
        ScreenSize {
            width: 800.0,
            height: 500.0,
        },
    );
    assert_eq!(viewport.zoom_percent(), 100);
}

#[test]
fn fit_of_a_tiny_object_should_not_zoom_past_400_percent() {
    let content = Rect::from_xywh(0.0, 0.0, 1.0, 1.0).ok();
    let viewport = Viewport::fit(
        content,
        ScreenSize {
            width: 1280.0,
            height: 800.0,
        },
    );
    assert_eq!(viewport.zoom(), MAX_ZOOM);
}
