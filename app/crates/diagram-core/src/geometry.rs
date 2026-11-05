//! Finite world geometry in logical units.
//!
//! Every value is checked on construction: coordinates are finite and sizes
//! are positive, so invalid geometry can never reach the document, the scene
//! or a saved file.

use thiserror::Error;

/// Why a geometry value was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum GeometryError {
    /// A coordinate or length was NaN or infinite.
    #[error("geometry must be finite")]
    NotFinite,
    /// A width or height was zero or negative.
    #[error("width and height must be greater than zero")]
    NotPositive,
}

fn finite(value: f64) -> Result<f64, GeometryError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(GeometryError::NotFinite)
    }
}

/// A finite point.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    x: f64,
    y: f64,
}

impl Point {
    /// The origin.
    pub const ORIGIN: Self = Self { x: 0.0, y: 0.0 };

    /// A checked point.
    ///
    /// # Errors
    ///
    /// Returns [`GeometryError::NotFinite`] for NaN or infinite coordinates.
    pub fn new(x: f64, y: f64) -> Result<Self, GeometryError> {
        Ok(Self {
            x: finite(x)?,
            y: finite(y)?,
        })
    }

    /// Horizontal coordinate.
    pub const fn x(self) -> f64 {
        self.x
    }

    /// Vertical coordinate.
    pub const fn y(self) -> f64 {
        self.y
    }

    /// This point moved by `(dx, dy)`.
    ///
    /// # Errors
    ///
    /// Returns [`GeometryError::NotFinite`] if the result overflows.
    pub fn offset(self, dx: f64, dy: f64) -> Result<Self, GeometryError> {
        Self::new(self.x + dx, self.y + dy)
    }
}

/// A finite size with positive width and height.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    width: f64,
    height: f64,
}

impl Size {
    /// One unit by one unit.
    pub const UNIT: Self = Self {
        width: 1.0,
        height: 1.0,
    };

    /// A checked size.
    ///
    /// # Errors
    ///
    /// Returns [`GeometryError`] when a side is not finite or not positive.
    pub fn new(width: f64, height: f64) -> Result<Self, GeometryError> {
        let (width, height) = (finite(width)?, finite(height)?);
        if width <= 0.0 || height <= 0.0 {
            return Err(GeometryError::NotPositive);
        }
        Ok(Self { width, height })
    }

    /// Width.
    pub const fn width(self) -> f64 {
        self.width
    }

    /// Height.
    pub const fn height(self) -> f64 {
        self.height
    }
}

/// A finite rectangle with positive size, by its top-left corner.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    origin: Point,
    size: Size,
}

impl Rect {
    /// A rectangle from its top-left corner and size.
    pub const fn new(origin: Point, size: Size) -> Self {
        Self { origin, size }
    }

    /// A checked rectangle from raw values.
    ///
    /// # Errors
    ///
    /// Returns [`GeometryError`] for non-finite values or a non-positive size.
    pub fn from_xywh(x: f64, y: f64, width: f64, height: f64) -> Result<Self, GeometryError> {
        Ok(Self::new(Point::new(x, y)?, Size::new(width, height)?))
    }

    /// Top-left corner.
    pub const fn origin(self) -> Point {
        self.origin
    }

    /// Size.
    pub const fn size(self) -> Size {
        self.size
    }

    /// Left edge.
    pub const fn left(self) -> f64 {
        self.origin.x
    }

    /// Top edge.
    pub const fn top(self) -> f64 {
        self.origin.y
    }

    /// Right edge.
    pub fn right(self) -> f64 {
        self.origin.x + self.size.width
    }

    /// Bottom edge.
    pub fn bottom(self) -> f64 {
        self.origin.y + self.size.height
    }

    /// Center point.
    pub fn center(self) -> Point {
        Point {
            x: self.origin.x + self.size.width / 2.0,
            y: self.origin.y + self.size.height / 2.0,
        }
    }

    /// The same size with its top-left corner at `origin`.
    pub const fn moved_to(self, origin: Point) -> Self {
        Self {
            origin,
            size: self.size,
        }
    }

    /// True when `point` lies inside or on the edge, grown by `margin` on every side.
    pub fn contains_with_margin(self, point: Point, margin: f64) -> bool {
        point.x >= self.left() - margin
            && point.x <= self.right() + margin
            && point.y >= self.top() - margin
            && point.y <= self.bottom() + margin
    }

    /// True when `other` lies completely inside this rectangle.
    pub fn encloses(self, other: Self) -> bool {
        other.left() >= self.left()
            && other.top() >= self.top()
            && other.right() <= self.right()
            && other.bottom() <= self.bottom()
    }

    /// The smallest rectangle containing both.
    pub fn union(self, other: Self) -> Self {
        let left = self.left().min(other.left());
        let top = self.top().min(other.top());
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        Self {
            origin: Point { x: left, y: top },
            size: Size {
                width: right - left,
                height: bottom - top,
            },
        }
    }

    /// The bounds of several rectangles, or `None` for none.
    pub fn bounds(rects: impl IntoIterator<Item = Self>) -> Option<Self> {
        rects.into_iter().reduce(Self::union)
    }

    /// The rectangle spanned by two corner points in any order, if it has area.
    ///
    /// # Errors
    ///
    /// Returns [`GeometryError::NotPositive`] when the points share an x or y.
    pub fn spanning(first: Point, second: Point) -> Result<Self, GeometryError> {
        Self::from_xywh(
            first.x.min(second.x),
            first.y.min(second.y),
            (first.x - second.x).abs(),
            (first.y - second.y).abs(),
        )
    }
}
