//! Glyph outline cache for the HarfBuzz shaping pipeline.
//!
//! Avoids repeated outline extraction for glyphs that appear multiple times
//! in a document. Outlines are identified by `(face_id, glyph_id)` pairs,
//! making the cache generic enough for both HarfBuzz and ttf-parser backends.

use std::collections::HashMap;

/// A single point in a glyph outline.
#[derive(Debug, Clone)]
pub struct OutlinePoint {
    pub x: f32,
    pub y: f32,
    pub kind: OutlinePointKind,
}

/// Classification of an outline point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutlinePointKind {
    OnCurve,
    OffCurveQuad,
    OffCurveCubic,
}

/// A contour (closed or open path) within a glyph outline.
#[derive(Debug, Clone)]
pub struct Contour {
    pub points: Vec<OutlinePoint>,
    pub is_closed: bool,
}

#[derive(Debug, Clone, Default)]
pub struct GlyphOutline {
    pub contours: Vec<Contour>,
    pub has_bezier: bool,
}

/// LRU cache for glyph outlines.
///
/// Uses `HashMap` for O(1) lookup and a `VecDeque`-style access order
/// for LRU eviction. Default capacity of 8192 covers typical documents.
pub struct GlyphOutlineCache {
    outlines: HashMap<(u64, u32), GlyphOutline>,
    access_order: Vec<(u64, u32)>,
    max_entries: usize,
    hits: u64,
    misses: u64,
}

const DEFAULT_CAPACITY: usize = 8192;

impl GlyphOutlineCache {
    /// Creates a new cache with the default capacity (8192).
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    /// Creates a new cache with the given maximum number of entries.
    pub fn with_capacity(max_entries: usize) -> Self {
        Self {
            outlines: HashMap::with_capacity(max_entries),
            access_order: Vec::with_capacity(max_entries),
            max_entries: max_entries.max(1),
            hits: 0,
            misses: 0,
        }
    }

    /// Retrieves a cached outline or computes it via the provided closure.
    ///
    /// On cache hit, the entry is promoted to most-recently-used.
    /// On cache miss, the closure is called with the `glyph_id` and the result
    /// is stored. If the cache is at capacity, the least-recently-used entry
    /// is evicted first.
    ///
    /// Returns a reference to the cached outline. Panics if the compute
    /// closure returns `None` for a previously-uncached key.
    pub fn get_or_compute<F>(&mut self, face_id: u64, glyph_id: u32, compute: F) -> &GlyphOutline
    where
        F: FnOnce(u32) -> Option<GlyphOutline>,
    {
        let key = (face_id, glyph_id);

        if let Some(pos) = self.access_order.iter().position(|k| *k == key) {
            self.access_order.remove(pos);
            self.access_order.push(key);
            self.hits += 1;
            // SAFETY: access_order and outlines are kept in sync.
            // When we insert, we add to both. When we evict, we remove
            // from both. The key was found in access_order, so it must
            // exist in outlines.
            #[allow(clippy::indexing_slicing)]
            return self
                .outlines
                .get(&key)
                .unwrap_or_else(|| unreachable!("outlines contains every key in access_order"));
        }

        self.misses += 1;
        let outline = compute(glyph_id).unwrap_or_default();
        self.evict_if_needed();
        self.outlines.insert(key, outline);
        self.access_order.push(key);

        self.outlines
            .get(&key)
            .unwrap_or_else(|| unreachable!("key was just inserted above"))
    }

    /// Returns a cached outline without computing if absent.
    pub fn get(&self, face_id: u64, glyph_id: u32) -> Option<&GlyphOutline> {
        self.outlines.get(&(face_id, glyph_id))
    }

    /// Inserts an outline into the cache, evicting if at capacity.
    pub fn insert(&mut self, face_id: u64, glyph_id: u32, outline: GlyphOutline) {
        let key = (face_id, glyph_id);
        if self.outlines.contains_key(&key) {
            if let Some(pos) = self.access_order.iter().position(|k| *k == key) {
                self.access_order.remove(pos);
            }
            self.outlines.insert(key, outline);
            self.access_order.push(key);
        } else {
            self.evict_if_needed();
            self.outlines.insert(key, outline);
            self.access_order.push(key);
        }
    }

    fn evict_if_needed(&mut self) {
        while self.outlines.len() >= self.max_entries {
            if let Some(oldest) = self.access_order.first().copied() {
                self.access_order.remove(0);
                self.outlines.remove(&oldest);
            } else {
                break;
            }
        }
    }

    /// Cache hit rate as a value between 0.0 and 1.0.
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            1.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Returns (hits, misses) statistics.
    pub fn stats(&self) -> (u64, u64) {
        (self.hits, self.misses)
    }

    /// Resets hit/miss counters without clearing cached entries.
    pub fn reset_stats(&mut self) {
        self.hits = 0;
        self.misses = 0;
    }

    /// Number of entries currently in the cache.
    pub fn len(&self) -> usize {
        self.outlines.len()
    }

    /// Whether the cache contains no entries.
    pub fn is_empty(&self) -> bool {
        self.outlines.is_empty()
    }

    /// Removes all entries and resets statistics.
    pub fn clear(&mut self) {
        self.outlines.clear();
        self.access_order.clear();
        self.hits = 0;
        self.misses = 0;
    }

    /// Maximum number of entries the cache can hold.
    pub fn capacity(&self) -> usize {
        self.max_entries
    }
}

impl Default for GlyphOutlineCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_outline() -> GlyphOutline {
        GlyphOutline {
            contours: vec![Contour {
                points: vec![
                    OutlinePoint {
                        x: 10.0,
                        y: 20.0,
                        kind: OutlinePointKind::OnCurve,
                    },
                    OutlinePoint {
                        x: 50.0,
                        y: 80.0,
                        kind: OutlinePointKind::OnCurve,
                    },
                ],
                is_closed: true,
            }],
            has_bezier: false,
        }
    }

    fn bezier_outline() -> GlyphOutline {
        GlyphOutline {
            contours: vec![Contour {
                points: vec![
                    OutlinePoint {
                        x: 0.0,
                        y: 0.0,
                        kind: OutlinePointKind::OnCurve,
                    },
                    OutlinePoint {
                        x: 10.0,
                        y: 100.0,
                        kind: OutlinePointKind::OffCurveCubic,
                    },
                    OutlinePoint {
                        x: 90.0,
                        y: 100.0,
                        kind: OutlinePointKind::OffCurveCubic,
                    },
                    OutlinePoint {
                        x: 100.0,
                        y: 0.0,
                        kind: OutlinePointKind::OnCurve,
                    },
                ],
                is_closed: true,
            }],
            has_bezier: true,
        }
    }

    #[test]
    fn test_outline_cache_hit() {
        let mut cache = GlyphOutlineCache::new();
        let face_id = 1u64;
        let glyph_id = 42u32;

        let outline = sample_outline();
        let contour_count = cache
            .get_or_compute(face_id, glyph_id, |_| Some(outline.clone()))
            .contours
            .len();
        assert_eq!(contour_count, 1);

        let (hits, misses) = cache.stats();
        assert_eq!(hits, 0);
        assert_eq!(misses, 1);

        let contour_count2 = cache
            .get_or_compute(face_id, glyph_id, |_| {
                panic!("should not call compute on cache hit")
            })
            .contours
            .len();
        assert_eq!(contour_count, contour_count2);

        let (hits, misses) = cache.stats();
        assert_eq!(hits, 1);
        assert_eq!(misses, 1);
    }

    #[test]
    fn test_outline_cache_miss() {
        let mut cache = GlyphOutlineCache::new();
        let mut compute_count = 0u32;

        cache.get_or_compute(1u64, 10u32, |_| {
            compute_count += 1;
            Some(sample_outline())
        });
        assert_eq!(compute_count, 1);

        cache.get_or_compute(1u64, 20u32, |_| {
            compute_count += 1;
            Some(bezier_outline())
        });
        assert_eq!(compute_count, 2);

        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_outline_cache_eviction() {
        let max = 3;
        let mut cache = GlyphOutlineCache::with_capacity(max);

        cache.insert(1, 1, sample_outline());
        cache.insert(1, 2, sample_outline());
        cache.insert(1, 3, sample_outline());
        assert_eq!(cache.len(), 3);

        cache.insert(1, 4, sample_outline());
        assert_eq!(cache.len(), 3);

        assert!(cache.get(1, 1).is_none(), "oldest should be evicted");
        assert!(cache.get(1, 2).is_some());
        assert!(cache.get(1, 3).is_some());
        assert!(cache.get(1, 4).is_some());
    }

    #[test]
    fn test_outline_cache_lru_promotion() {
        let mut cache = GlyphOutlineCache::with_capacity(2);

        cache.insert(1, 1, sample_outline());
        cache.insert(1, 2, sample_outline());

        let _ = cache.get_or_compute(1, 1, |_| panic!("should be cached"));
        assert_eq!(cache.len(), 2);

        cache.insert(1, 3, sample_outline());
        assert_eq!(cache.len(), 2);

        assert!(
            cache.get(1, 2).is_none(),
            "LRU entry should be evicted after promotion of (1,1)"
        );
        assert!(cache.get(1, 1).is_some());
        assert!(cache.get(1, 3).is_some());
    }

    #[test]
    fn test_outline_cache_clear() {
        let mut cache = GlyphOutlineCache::new();
        cache.insert(1, 1, sample_outline());
        cache.insert(1, 2, bezier_outline());
        assert_eq!(cache.len(), 2);

        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
        assert_eq!(cache.hit_rate(), 1.0);
        let (hits, misses) = cache.stats();
        assert_eq!(hits, 0);
        assert_eq!(misses, 0);
    }

    #[test]
    fn test_outline_cache_hit_rate() {
        let mut cache = GlyphOutlineCache::new();

        let rate = cache.hit_rate();
        assert!(
            (rate - 1.0).abs() < f64::EPSILON,
            "empty cache should have 1.0 hit rate"
        );

        cache.insert(1, 1, sample_outline());

        let _ = cache.get_or_compute(1, 1, |_| panic!("should be cached"));
        assert_eq!(cache.len(), 1);
        let rate = cache.hit_rate();
        assert!((rate - 1.0).abs() < f64::EPSILON);

        let _ = cache.get_or_compute(1, 2, |_| Some(bezier_outline()));
        let rate = cache.hit_rate();
        assert!((rate - (1.0 / 2.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_outline_cache_different_faces() {
        let mut cache = GlyphOutlineCache::new();

        cache.insert(1, 10, sample_outline());
        cache.insert(2, 10, bezier_outline());

        let o1 = cache.get(1, 10).unwrap();
        let o2 = cache.get(2, 10).unwrap();

        assert_eq!(o1.contours.len(), 1);
        assert_eq!(o2.contours.len(), 1);
        assert_ne!(o1.has_bezier, o2.has_bezier);
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_outline_cache_compute_returns_none() {
        let mut cache = GlyphOutlineCache::new();

        let cached = cache.get_or_compute(1, 999, |_| None);
        assert!(cached.contours.is_empty());
        assert!(!cached.has_bezier);
    }

    #[test]
    fn test_outline_cache_capacity() {
        let cache = GlyphOutlineCache::with_capacity(100);
        assert_eq!(cache.capacity(), 100);
    }

    #[test]
    fn test_outline_cache_reset_stats() {
        let mut cache = GlyphOutlineCache::new();
        cache.insert(1, 1, sample_outline());
        let _ = cache.get_or_compute(1, 1, |_| panic!("should be cached"));

        cache.reset_stats();
        let (hits, misses) = cache.stats();
        assert_eq!(hits, 0);
        assert_eq!(misses, 0);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_outline_cache_insert_updates_existing() {
        let mut cache = GlyphOutlineCache::new();
        cache.insert(1, 1, sample_outline());
        cache.insert(1, 1, bezier_outline());

        assert_eq!(cache.len(), 1);
        let o = cache.get(1, 1).unwrap();
        assert!(o.has_bezier);
    }
}
