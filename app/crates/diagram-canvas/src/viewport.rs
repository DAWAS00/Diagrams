//! World ↔ screen transform: `screen = origin + pan + world * zoom`.
//!
//! `pan` is in screen units, relative to the canvas origin. Zoom is always
//! finite and clamped to [`MIN_ZOOM`]..=[`MAX_ZOOM`], so the inverse used for
//! picking always exists. Changing the viewport never changes the document.

use diagram_core::{GeometryError, Point, Rect};

/// Smallest zoom: 10%.
pub const MIN_ZOOM: f64 = 0.1;
/// Largest zoom: 400%.
pub const MAX_ZOOM: f64 = 4.0;
/// Factor of one zoom-in or zoom-out step.
pub const ZOOM_STEP: f64 = 1.25;
/// Screen padding around content when fitting.
pub const FIT_PADDING: f64 = 32.0;

/// A size on screen, in points. Both sides are finite and positive.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenSize {
    /// Width in points.
    pub width: f64,
    /// Height in points.
    pub height: f64,
}

/// Pan and zoom of one diagram tab.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pan: Point,
    zoom: f64,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            pan: Point::ORIGIN,
            zoom: 1.0,
        }
    }
}

fn clamp_zoom(zoom: f64) -> Result<f64, GeometryError> {
    if zoom.is_finite() && zoom > 0.0 {
        Ok(zoom.clamp(MIN_ZOOM, MAX_ZOOM))
    } else {
        Err(GeometryError::NotFinite)
    }
}

impl Viewport {
    /// A viewport; the zoom is clamped to the allowed range.
    ///
    /// # Errors
    ///
    /// Returns [`GeometryError::NotFinite`] for a NaN, infinite or
    /// non-positive zoom.
    pub fn new(pan: Point, zoom: f64) -> Result<Self, GeometryError> {
        Ok(Self {
            pan,
            zoom: clamp_zoom(zoom)?,
        })
    }

    /// Screen offset of the world origin, relative to the canvas origin.
    pub const fn pan(self) -> Point {
        self.pan
    }

    /// Scale from world units to screen points.
    pub const fn zoom(self) -> f64 {
        self.zoom
    }

    /// Zoom as a whole percentage, such as 125.
    pub fn zoom_percent(self) -> u32 {
        // Zoom is clamped to 10–400%, so the cast cannot truncate.
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "zoom is clamped to 0.1..=4.0"
        )]
        let percent = (self.zoom * 100.0).round() as u32;
        percent
    }

    /// Where `world` appears, relative to the canvas origin.
    pub fn to_screen(self, world: Point) -> Point {
        // Finite inputs and a bounded zoom give a finite result for any
        // realistic coordinate; fall back to the pan if they do not.
        Point::new(
            self.pan.x() + world.x() * self.zoom,
            self.pan.y() + world.y() * self.zoom,
        )
        .unwrap_or(self.pan)
    }

    /// The world point under `screen` (relative to the canvas origin).
    ///
    /// # Errors
    ///
    /// Returns [`GeometryError::NotFinite`] if the result is not finite.
    pub fn to_world(self, screen: Point) -> Result<Point, GeometryError> {
        Point::new(
            (screen.x() - self.pan.x()) / self.zoom,
            (screen.y() - self.pan.y()) / self.zoom,
        )
    }

    /// A world rectangle on screen.
    pub fn rect_to_screen(self, world: Rect) -> Option<Rect> {
        let origin = self.to_screen(world.origin());
        Rect::from_xywh(
            origin.x(),
            origin.y(),
            world.size().width() * self.zoom,
            world.size().height() * self.zoom,
        )
        .ok()
    }

    /// Moves the view by a screen delta.
    ///
    /// # Errors
    ///
    /// Returns [`GeometryError::NotFinite`] for a non-finite delta.
    pub fn panned(self, dx: f64, dy: f64) -> Result<Self, GeometryError> {
        Ok(Self {
            pan: self.pan.offset(dx, dy)?,
            ..self
        })
    }

    /// The viewport at `zoom` (clamped), keeping the world point under
    /// `anchor` (a screen point relative to the canvas origin) in place.
    ///
    /// # Errors
    ///
    /// Returns [`GeometryError::NotFinite`] for a non-finite zoom or anchor.
    pub fn zoomed_to(self, zoom: f64, anchor: Point) -> Result<Self, GeometryError> {
        let zoom = clamp_zoom(zoom)?;
        let world = self.to_world(anchor)?;
        let pan = Point::new(anchor.x() - world.x() * zoom, anchor.y() - world.y() * zoom)?;
        Ok(Self { pan, zoom })
    }

    /// The viewport zoomed by `factor`, anchored at `anchor`.
    ///
    /// # Errors
    ///
    /// Returns [`GeometryError::NotFinite`] for a non-finite factor or anchor.
    pub fn zoomed_by(self, factor: f64, anchor: Point) -> Result<Self, GeometryError> {
        self.zoomed_to(self.zoom * factor, anchor)
    }

    /// A viewport that shows `content` centered in `screen`, with padding,
    /// never zooming beyond the allowed range. Empty content resets to 100%
    /// with the world origin at the top-left padding.
    pub fn fit(content: Option<Rect>, screen: ScreenSize) -> Self {
        let Some(content) = content else {
            return Self {
                pan: Point::new(FIT_PADDING, FIT_PADDING).unwrap_or(Point::ORIGIN),
                zoom: 1.0,
            };
        };
        let usable_width = (screen.width - 2.0 * FIT_PADDING).max(1.0);
        let usable_height = (screen.height - 2.0 * FIT_PADDING).max(1.0);
        let zoom = (usable_width / content.size().width())
            .min(usable_height / content.size().height())
            .clamp(MIN_ZOOM, MAX_ZOOM);
        let center = content.center();
        let pan = Point::new(
            screen.width / 2.0 - center.x() * zoom,
            screen.height / 2.0 - center.y() * zoom,
        )
        .unwrap_or(Point::ORIGIN);
        Self { pan, zoom }
    }
}
