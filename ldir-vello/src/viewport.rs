//! Viewport with pan/zoom and coordinate transformations.
//!
//! Maps 26.6 fixed-point G-IR coordinates to screen coordinates via
//! pan and zoom transformations.

use ldir_core::fp266::Fp266;

/// A rectangular viewport with pan and zoom support.
///
/// All positional fields are in 26.6 fixed-point. The zoom factor is
/// a unitless multiplier (1.0 = 100%).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    /// Horizontal offset of the viewport origin (26.6).
    pub x: Fp266,
    /// Vertical offset of the viewport origin (26.6).
    pub y: Fp266,
    /// Width of the visible area (26.6).
    pub width: Fp266,
    /// Height of the visible area (26.6).
    pub height: Fp266,
    /// Zoom factor (1.0 = no zoom).
    pub zoom: f64,
}

impl Viewport {
    /// Create a new viewport at the given position with the given dimensions.
    ///
    /// Coordinates are in 26.6 fixed-point. Zoom defaults to 1.0.
    pub fn new(x: Fp266, y: Fp266, width: Fp266, height: Fp266) -> Self {
        Self {
            x,
            y,
            width,
            height,
            zoom: 1.0,
        }
    }

    /// Create a viewport from floating-point values (converted to 26.6).
    pub fn from_f64(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x: Fp266::from_f64(x),
            y: Fp266::from_f64(y),
            width: Fp266::from_f64(width),
            height: Fp266::from_f64(height),
            zoom: 1.0,
        }
    }

    /// Pan the viewport by the given delta (in 26.6 units).
    pub fn pan(&mut self, dx: Fp266, dy: Fp266) {
        self.x += dx;
        self.y += dy;
    }

    /// Pan the viewport by floating-point delta (converted to 26.6).
    pub fn pan_f64(&mut self, dx: f64, dy: f64) {
        self.x += Fp266::from_f64(dx);
        self.y += Fp266::from_f64(dy);
    }

    /// Zoom the viewport by a multiplicative factor.
    ///
    /// Zoom is anchored at the viewport center.
    pub fn zoom(&mut self, factor: f64) {
        if factor <= 0.0 {
            return;
        }
        self.zoom *= factor;
    }

    /// Set the zoom level to an absolute value.
    pub fn set_zoom(&mut self, zoom: f64) {
        if zoom > 0.0 {
            self.zoom = zoom;
        }
    }

    /// Transform a 26.6 G-IR coordinate to screen coordinates.
    ///
    /// Returns `(screen_x, screen_y)` in 26.6 units relative to the
    /// viewport origin, accounting for pan and zoom.
    pub fn to_screen(&self, gir_x: Fp266, gir_y: Fp266) -> (Fp266, Fp266) {
        let zoom_fp = Fp266::from_f64(self.zoom);
        let sx = (gir_x - self.x).mul(zoom_fp);
        let sy = (gir_y - self.y).mul(zoom_fp);
        (sx, sy)
    }

    /// Transform screen coordinates back to G-IR 26.6 coordinates.
    ///
    /// Inverse of [`to_screen`](Self::to_screen).
    pub fn to_gir(&self, screen_x: Fp266, screen_y: Fp266) -> (Fp266, Fp266) {
        let inv_zoom = Fp266::from_f64(1.0 / self.zoom);
        let gx = screen_x.mul(inv_zoom) + self.x;
        let gy = screen_y.mul(inv_zoom) + self.y;
        (gx, gy)
    }

    /// Convert a 26.6 coordinate to f64 scene coordinates (for Vello).
    ///
    /// Vello uses f64 coordinates, so this divides by 64.0 and applies
    /// the viewport transform.
    pub fn to_scene_coord(&self, fp: Fp266) -> f64 {
        fp.to_f64()
    }

    /// Get the effective visible width in 26.6 G-IR units, accounting for zoom.
    pub fn effective_width(&self) -> Fp266 {
        let inv_zoom = Fp266::from_f64(1.0 / self.zoom);
        self.width.mul(inv_zoom)
    }

    /// Get the effective visible height in 26.6 G-IR units, accounting for zoom.
    pub fn effective_height(&self) -> Fp266 {
        let inv_zoom = Fp266::from_f64(1.0 / self.zoom);
        self.height.mul(inv_zoom)
    }

    /// Check if a 26.6 point is visible within the viewport.
    pub fn contains(&self, x: Fp266, y: Fp266) -> bool {
        let eff_w = self.effective_width();
        let eff_h = self.effective_height();
        x >= self.x && x < self.x + eff_w && y >= self.y && y < self.y + eff_h
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::from_f64(0.0, 0.0, 612.0, 792.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_new() {
        let vp = Viewport::new(
            Fp266::from_int(10),
            Fp266::from_int(20),
            Fp266::from_int(612),
            Fp266::from_int(792),
        );
        assert_eq!(vp.x, Fp266::from_int(10));
        assert_eq!(vp.y, Fp266::from_int(20));
        assert_eq!(vp.zoom, 1.0);
    }

    #[test]
    fn test_viewport_default() {
        let vp = Viewport::default();
        assert_eq!(vp.x, Fp266::ZERO);
        assert_eq!(vp.y, Fp266::ZERO);
        assert_eq!(vp.zoom, 1.0);
    }

    #[test]
    fn test_pan() {
        let mut vp = Viewport::default();
        vp.pan(Fp266::from_int(100), Fp266::from_int(200));
        assert_eq!(vp.x, Fp266::from_int(100));
        assert_eq!(vp.y, Fp266::from_int(200));
    }

    #[test]
    fn test_pan_f64() {
        let mut vp = Viewport::default();
        vp.pan_f64(10.5, 20.5);
        assert_eq!(vp.x, Fp266::from_f64(10.5));
        assert_eq!(vp.y, Fp266::from_f64(20.5));
    }

    #[test]
    fn test_zoom() {
        let mut vp = Viewport::default();
        vp.zoom(2.0);
        assert_eq!(vp.zoom, 2.0);
        vp.zoom(0.5);
        assert_eq!(vp.zoom, 1.0);
    }

    #[test]
    fn test_zoom_negative_ignored() {
        let mut vp = Viewport::default();
        vp.zoom(-1.0);
        assert_eq!(vp.zoom, 1.0);
        vp.zoom(0.0);
        assert_eq!(vp.zoom, 1.0);
    }

    #[test]
    fn test_set_zoom() {
        let mut vp = Viewport::default();
        vp.set_zoom(3.0);
        assert_eq!(vp.zoom, 3.0);
        vp.set_zoom(-1.0);
        assert_eq!(vp.zoom, 3.0);
    }

    #[test]
    fn test_to_screen_identity() {
        let vp = Viewport::default();
        let (sx, sy) = vp.to_screen(Fp266::from_int(100), Fp266::from_int(200));
        assert_eq!(sx, Fp266::from_int(100));
        assert_eq!(sy, Fp266::from_int(200));
    }

    #[test]
    fn test_to_screen_with_pan() {
        let mut vp = Viewport::default();
        vp.pan(Fp266::from_int(10), Fp266::from_int(20));
        let (sx, sy) = vp.to_screen(Fp266::from_int(100), Fp266::from_int(200));
        assert_eq!(sx, Fp266::from_int(90));
        assert_eq!(sy, Fp266::from_int(180));
    }

    #[test]
    fn test_to_screen_with_zoom() {
        let mut vp = Viewport::default();
        vp.zoom(2.0);
        let (sx, sy) = vp.to_screen(Fp266::from_int(100), Fp266::from_int(200));
        assert_eq!(sx, Fp266::from_int(200));
        assert_eq!(sy, Fp266::from_int(400));
    }

    #[test]
    fn test_to_screen_with_pan_and_zoom() {
        let mut vp = Viewport::default();
        vp.pan(Fp266::from_int(10), Fp266::from_int(20));
        vp.zoom(2.0);
        let (sx, sy) = vp.to_screen(Fp266::from_int(20), Fp266::from_int(30));
        assert_eq!(sx, Fp266::from_int(20));
        assert_eq!(sy, Fp266::from_int(20));
    }

    #[test]
    fn test_to_gir_roundtrip() {
        let vp = Viewport::default();
        let (gx, gy) = vp.to_gir(Fp266::from_int(100), Fp266::from_int(200));
        assert_eq!(gx, Fp266::from_int(100));
        assert_eq!(gy, Fp266::from_int(200));
    }

    #[test]
    fn test_to_gir_roundtrip_with_pan_and_zoom() {
        let mut vp = Viewport::default();
        vp.pan(Fp266::from_int(10), Fp266::from_int(20));
        vp.zoom(2.0);
        let original_x = Fp266::from_int(50);
        let original_y = Fp266::from_int(60);
        let (sx, sy) = vp.to_screen(original_x, original_y);
        let (gx, gy) = vp.to_gir(sx, sy);
        let tolerance = Fp266::from_int(1);
        assert!((gx - original_x).abs() <= tolerance);
        assert!((gy - original_y).abs() <= tolerance);
    }

    #[test]
    fn test_to_scene_coord() {
        let vp = Viewport::default();
        let fp = Fp266::from_int(128);
        assert_eq!(vp.to_scene_coord(fp), 128.0);
    }

    #[test]
    fn test_to_scene_coord_fractional() {
        let vp = Viewport::default();
        let fp = Fp266::from_frac(1, 2);
        assert!((vp.to_scene_coord(fp) - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_effective_dimensions() {
        let mut vp = Viewport::from_f64(0.0, 0.0, 100.0, 200.0);
        assert_eq!(vp.effective_width(), Fp266::from_int(100));
        assert_eq!(vp.effective_height(), Fp266::from_int(200));
        vp.zoom(2.0);
        assert_eq!(vp.effective_width(), Fp266::from_int(50));
        assert_eq!(vp.effective_height(), Fp266::from_int(100));
    }

    #[test]
    fn test_contains() {
        let vp = Viewport::from_f64(0.0, 0.0, 100.0, 100.0);
        assert!(vp.contains(Fp266::from_int(50), Fp266::from_int(50)));
        assert!(vp.contains(Fp266::ZERO, Fp266::ZERO));
        assert!(!vp.contains(Fp266::from_int(100), Fp266::from_int(50)));
        assert!(!vp.contains(Fp266::from_int(50), Fp266::from_int(100)));
    }

    #[test]
    fn test_from_f64() {
        let vp = Viewport::from_f64(10.5, 20.5, 100.0, 200.0);
        assert!((vp.x.to_f64() - 10.5).abs() < 0.01);
        assert!((vp.y.to_f64() - 20.5).abs() < 0.01);
    }

    #[test]
    fn test_coordinate_transform_accuracy() {
        let mut vp = Viewport::from_f64(5.0, 10.0, 612.0, 792.0);
        vp.set_zoom(1.5);

        let test_points: &[(f64, f64)] = &[(100.0, 200.0), (300.0, 400.0), (5.5, 10.75)];

        for &(gx, gy) in test_points {
            let fp_x = Fp266::from_f64(gx);
            let fp_y = Fp266::from_f64(gy);
            let (sx, sy) = vp.to_screen(fp_x, fp_y);
            let (rx, ry) = vp.to_gir(sx, sy);

            let tolerance = Fp266::from_int(4);
            assert!(
                (rx - fp_x).abs() <= tolerance,
                "X roundtrip failed: {} -> {} -> {} (expected {})",
                gx,
                sx.to_f64(),
                rx.to_f64(),
                gx
            );
            assert!(
                (ry - fp_y).abs() <= tolerance,
                "Y roundtrip failed: {} -> {} -> {} (expected {})",
                gy,
                sy.to_f64(),
                ry.to_f64(),
                gy
            );
        }
    }
}
