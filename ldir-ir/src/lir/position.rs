//! LIR positioning types: Point, Size, Rect.
//!
//! All coordinates use 26.6 fixed-point (`Fp266`), per AX-LIR-001.
//! Origin is the top-left corner of the page content area.
//! Y-axis increases downward (PDF/screen convention).

use crate::fp266::Fp266;

/// A 2D point in scaled-point coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Point {
    /// Horizontal position from left edge.
    pub x: Fp266,
    /// Vertical position from top edge.
    pub y: Fp266,
}

impl Point {
    /// Point at origin (0, 0).
    pub const ZERO: Self = Self {
        x: Fp266::ZERO,
        y: Fp266::ZERO,
    };

    /// Create a point from Fp266 values.
    #[inline]
    pub const fn new(x: Fp266, y: Fp266) -> Self {
        Self { x, y }
    }

    /// Create a point from integer values (converted to Fp266).
    #[inline]
    pub const fn from_int(x: i32, y: i32) -> Self {
        Self {
            x: Fp266::from_int(x),
            y: Fp266::from_int(y),
        }
    }
}

/// A 2D size (width × height).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Size {
    /// Horizontal extent.
    pub width: Fp266,
    /// Vertical extent.
    pub height: Fp266,
}

impl Size {
    /// Zero size.
    pub const ZERO: Self = Self {
        width: Fp266::ZERO,
        height: Fp266::ZERO,
    };

    /// Create a size from Fp266 values.
    #[inline]
    pub const fn new(width: Fp266, height: Fp266) -> Self {
        Self { width, height }
    }

    /// Create a size from integer values (converted to Fp266).
    #[inline]
    pub const fn from_int(width: i32, height: i32) -> Self {
        Self {
            width: Fp266::from_int(width),
            height: Fp266::from_int(height),
        }
    }

    /// Check if either dimension is zero.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.width.is_zero() || self.height.is_zero()
    }
}

/// An axis-aligned rectangle defined by origin and size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Rect {
    /// X coordinate of the top-left corner.
    pub x: Fp266,
    /// Y coordinate of the top-left corner.
    pub y: Fp266,
    /// Horizontal extent.
    pub width: Fp266,
    /// Vertical extent.
    pub height: Fp266,
}

impl Rect {
    /// Zero-sized rectangle at origin.
    pub const ZERO: Self = Self {
        x: Fp266::ZERO,
        y: Fp266::ZERO,
        width: Fp266::ZERO,
        height: Fp266::ZERO,
    };

    /// Create a rectangle from Fp266 values.
    #[inline]
    pub const fn new(x: Fp266, y: Fp266, width: Fp266, height: Fp266) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Create a rectangle from integer values (converted to Fp266).
    #[inline]
    pub const fn from_int(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self {
            x: Fp266::from_int(x),
            y: Fp266::from_int(y),
            width: Fp266::from_int(w),
            height: Fp266::from_int(h),
        }
    }

    /// Get the origin point (top-left corner).
    #[inline]
    pub fn origin(&self) -> Point {
        Point::new(self.x, self.y)
    }

    /// Get the size.
    #[inline]
    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }

    /// X coordinate of the right edge (x + width).
    #[inline]
    pub fn right(&self) -> Fp266 {
        self.x + self.width
    }

    /// Y coordinate of the bottom edge (y + height).
    #[inline]
    pub fn bottom(&self) -> Fp266 {
        self.y + self.height
    }

    /// Check if a point lies within this rectangle (half-open: excludes right/bottom).
    pub fn contains_point(&self, point: &Point) -> bool {
        point.x >= self.x
            && point.x < self.x + self.width
            && point.y >= self.y
            && point.y < self.y + self.height
    }

    /// Check if another rectangle is fully contained within this one.
    pub fn contains_rect(&self, other: &Rect) -> bool {
        other.x >= self.x
            && other.y >= self.y
            && other.right() <= self.right()
            && other.bottom() <= self.bottom()
    }

    /// Check if two rectangles overlap (have a non-empty intersection).
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.right()
            && self.right() > other.x
            && self.y < other.bottom()
            && self.bottom() > other.y
    }

    /// Check if either dimension is zero.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.width.is_zero() || self.height.is_zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_from_int() {
        let p = Point::from_int(10, 20);
        assert_eq!(p.x, Fp266::from_int(10));
        assert_eq!(p.y, Fp266::from_int(20));
    }

    #[test]
    fn test_size_from_int() {
        let s = Size::from_int(100, 200);
        assert_eq!(s.width, Fp266::from_int(100));
        assert_eq!(s.height, Fp266::from_int(200));
        assert!(!s.is_empty());
        assert!(Size::ZERO.is_empty());
    }

    #[test]
    fn test_rect_accessors() {
        let r = Rect::from_int(10, 20, 100, 200);
        assert_eq!(r.origin(), Point::from_int(10, 20));
        assert_eq!(r.size(), Size::from_int(100, 200));
        assert_eq!(r.right(), Fp266::from_int(110));
        assert_eq!(r.bottom(), Fp266::from_int(220));
    }

    #[test]
    fn test_rect_contains_point() {
        let r = Rect::from_int(0, 0, 100, 100);
        assert!(r.contains_point(&Point::from_int(50, 50)));
        assert!(r.contains_point(&Point::from_int(0, 0)));
        assert!(!r.contains_point(&Point::from_int(100, 100)));
        assert!(!r.contains_point(&Point::from_int(-1, 0)));
    }

    #[test]
    fn test_rect_intersects() {
        let a = Rect::from_int(0, 0, 100, 100);
        let b = Rect::from_int(50, 50, 100, 100);
        let c = Rect::from_int(200, 200, 100, 100);
        assert!(a.intersects(&b));
        assert!(!a.intersects(&c));
    }

    #[test]
    fn test_rect_contains_rect() {
        let outer = Rect::from_int(0, 0, 200, 200);
        let inner = Rect::from_int(10, 10, 50, 50);
        let outside = Rect::from_int(10, 10, 200, 50);
        assert!(outer.contains_rect(&inner));
        assert!(!outer.contains_rect(&outside));
    }
}
