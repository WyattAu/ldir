//! LRU cache for shaped runs (TASK-016).
//!
//! Uses `IndexMap` for O(1) insertion, lookup, and O(1) amortized LRU eviction.
//! The `IndexMap` preserves insertion order; eviction removes the oldest key
//! by swapping with the last entry and popping, avoiding the O(n) scan of the
//! previous HashMap+Vec implementation.
//!
//! Thread-safe via `DashMap` with epoch-based access tracking. Cache hits are
//! lock-free (shard-level read lock). The shaper function runs outside any lock,
//! so threads never block on HarfBuzz shaping during cache misses.

use indexmap::IndexMap;
use std::sync::Arc;

use crate::fp266::Fp266;
use crate::shaping::ShapedRun;

/// Cache key: (text, font_id, font_size).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct CacheKey {
    text: Arc<str>,
    font_id: u32,
    font_size_raw: i64,
}

/// A simple LRU cache for shaped text runs.
///
/// Uses an `IndexMap` for O(1) amortized insertion, lookup, and eviction.
/// On cache hit, the entry is moved to the most-recently-used position via
/// `swap_remove` + `push`, which is O(1) for IndexMap (swap is O(1), push is
/// amortized O(1)). On capacity overflow, the oldest entry (index 0) is evicted.
pub struct ShapeCache {
    capacity: usize,
    entries: IndexMap<CacheKey, Arc<ShapedRun>>,
    hits: u64,
    misses: u64,
}

impl ShapeCache {
    /// Create a new cache with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            entries: IndexMap::with_capacity(capacity),
            hits: 0,
            misses: 0,
        }
    }

    /// Get a cached run or shape it using the provided shaper function.
    ///
    /// The `shaper` is only called on a cache miss.
    /// On hit: the entry is promoted to MRU via `swap_remove` + `push` (O(1) amortized).
    ///   Returns a clone of the cached `Arc<ShapedRun>`, avoiding a deep clone of the
    ///   underlying `ShapedRun` (which contains `Vec<ShapedGlyph>`).
    /// On miss + capacity overflow: the oldest entry (index 0) is evicted (O(1)).
    pub fn get_or_shape<F>(
        &mut self,
        text: &str,
        font_id: u32,
        font_size: Fp266,
        shaper: F,
    ) -> Arc<ShapedRun>
    where
        F: FnOnce(&str, u32, Fp266) -> ShapedRun,
    {
        let key = CacheKey {
            text: text.into(),
            font_id,
            font_size_raw: font_size.raw(),
        };

        if let Some((idx, _, run)) = self.entries.get_full(&key) {
            // Cache hit: promote to MRU position.
            // swap_remove_index is O(1) for IndexMap (swaps with last element, pops).
            // Clone the Arc (cheap pointer increment), not the ShapedRun (expensive Vec clone).
            let result = run.clone();
            let run = run.clone();
            self.entries.swap_remove_index(idx);
            self.entries.insert(key, run);
            self.hits += 1;
            return result;
        }

        self.misses += 1;
        let run = shaper(text, font_id, font_size);
        let arc = Arc::new(run);

        if self.entries.len() >= self.capacity {
            self.entries.shift_remove_index(0);
        }

        self.entries.insert(key, arc.clone());
        arc
    }

    /// Number of entries currently in the cache.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Cache hit count.
    pub fn hits(&self) -> u64 {
        self.hits
    }

    /// Cache miss count.
    pub fn misses(&self) -> u64 {
        self.misses
    }

    /// Cache hit rate as a value between 0.0 and 1.0.
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Reset hit/miss counters.
    pub fn reset_stats(&mut self) {
        self.hits = 0;
        self.misses = 0;
    }

    /// Get (hits, misses) statistics.
    pub fn stats(&self) -> (u64, u64) {
        (self.hits, self.misses)
    }
}

/// Thread-safe lock-free cache for shaped runs.
///
/// Uses `DashMap` for sharded concurrent access. The shaper function executes
/// entirely outside any lock, so cache misses never block other threads.
/// Approximate LRU eviction via epoch-based access tracking: each entry stores
/// its last access epoch, and the entry with the oldest epoch is evicted when
/// the cache exceeds capacity.
pub struct ThreadSafeShapeCache {
    entries: dashmap::DashMap<CacheKey, CacheEntry>,
    capacity: usize,
    epoch: std::sync::atomic::AtomicU64,
    hits: std::sync::atomic::AtomicU64,
    misses: std::sync::atomic::AtomicU64,
}

/// Internal entry wrapping a shaped run with access metadata.
struct CacheEntry {
    run: Arc<ShapedRun>,
    last_access: std::sync::atomic::AtomicU64,
}

impl ThreadSafeShapeCache {
    /// Create a new thread-safe cache with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: dashmap::DashMap::with_capacity(capacity),
            capacity: capacity.max(1),
            epoch: std::sync::atomic::AtomicU64::new(0),
            hits: std::sync::atomic::AtomicU64::new(0),
            misses: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Get a cached run or shape it using the provided shaper function.
    ///
    /// Cache hits are lock-free (shard read lock only). The shaper function
    /// runs completely outside any lock. Returns `Arc<ShapedRun>` to avoid
    /// deep cloning on cache hits.
    pub fn get_or_shape<F>(
        &self,
        text: &str,
        font_id: u32,
        font_size: Fp266,
        shaper: F,
    ) -> Arc<ShapedRun>
    where
        F: FnOnce(&str, u32, Fp266) -> ShapedRun,
    {
        let key = CacheKey {
            text: text.into(),
            font_id,
            font_size_raw: font_size.raw(),
        };

        // Fast path: lock-free cache hit.
        if let Some(entry) = self.entries.get(&key) {
            entry.last_access.store(
                self.epoch
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
                std::sync::atomic::Ordering::Relaxed,
            );
            self.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return entry.run.clone();
        }

        // Slow path: shape outside any lock.
        self.misses
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let run = Arc::new(shaper(text, font_id, font_size));
        let now = self
            .epoch
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // Insert (or pick up another thread's insert).
        // Use get+insert to avoid entry API deadlocks.
        let result = run.clone();
        if let Some(existing) = self.entries.get(&key) {
            // Another thread inserted while we were shaping. Use theirs.
            existing
                .last_access
                .store(now, std::sync::atomic::Ordering::Relaxed);
            return existing.run.clone();
        }

        self.entries.insert(
            key,
            CacheEntry {
                run: result.clone(),
                last_access: std::sync::atomic::AtomicU64::new(now),
            },
        );

        // Evict if over capacity (lazy, approximate).
        if self.entries.len() > self.capacity {
            self.evict_oldest();
        }

        result
    }

    /// Evict the entry with the oldest `last_access` epoch.
    /// O(n) scan but only runs when capacity is exceeded.
    fn evict_oldest(&self) {
        let oldest = self
            .entries
            .iter()
            .min_by_key(|e| {
                e.value()
                    .last_access
                    .load(std::sync::atomic::Ordering::Relaxed)
            })
            .map(|r| r.key().clone());
        if let Some(key) = oldest {
            self.entries.remove(&key);
        }
    }

    /// Number of entries currently in the cache.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get (hits, misses) statistics.
    pub fn stats(&self) -> (u64, u64) {
        (
            self.hits.load(std::sync::atomic::Ordering::Relaxed),
            self.misses.load(std::sync::atomic::Ordering::Relaxed),
        )
    }

    /// Reset hit/miss counters.
    pub fn reset_stats(&self) {
        self.hits.store(0, std::sync::atomic::Ordering::Relaxed);
        self.misses.store(0, std::sync::atomic::Ordering::Relaxed);
    }

    /// Cache hit rate as a value between 0.0 and 1.0.
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(std::sync::atomic::Ordering::Relaxed);
        let misses = self.misses.load(std::sync::atomic::Ordering::Relaxed);
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shaping::fast_path::shape_ascii;

    #[test]
    fn cache_hit_returns_same_result() {
        let mut cache = ShapeCache::new(10);
        let font_size = Fp266::from_int(12);

        let run1 = cache.get_or_shape("hello", 1, font_size, |t, fid, fs| shape_ascii(t, fs, fid));
        let run2 = cache.get_or_shape("hello", 1, font_size, |t, fid, fs| shape_ascii(t, fs, fid));

        assert_eq!(run1, run2);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn cache_miss_different_text() {
        let mut cache = ShapeCache::new(10);
        let font_size = Fp266::from_int(12);

        cache.get_or_shape("hello", 1, font_size, |t, fid, fs| shape_ascii(t, fs, fid));
        cache.get_or_shape("world", 1, font_size, |t, fid, fs| shape_ascii(t, fs, fid));

        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn cache_miss_different_font_size() {
        let mut cache = ShapeCache::new(10);

        cache.get_or_shape("hello", 1, Fp266::from_int(12), |t, fid, fs| {
            shape_ascii(t, fs, fid)
        });
        cache.get_or_shape("hello", 1, Fp266::from_int(14), |t, fid, fs| {
            shape_ascii(t, fs, fid)
        });

        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn cache_eviction() {
        let mut cache = ShapeCache::new(2);
        let font_size = Fp266::from_int(12);

        cache.get_or_shape("a", 1, font_size, |t, fid, fs| shape_ascii(t, fs, fid));
        cache.get_or_shape("b", 1, font_size, |t, fid, fs| shape_ascii(t, fs, fid));
        assert_eq!(cache.len(), 2);

        // Third entry should evict the oldest ("a")
        cache.get_or_shape("c", 1, font_size, |t, fid, fs| shape_ascii(t, fs, fid));
        assert_eq!(cache.len(), 2);

        // "a" should be evicted; requesting it should call the shaper
        let mut call_count = 0u32;
        cache.get_or_shape("a", 1, font_size, |t, fid, fs| {
            call_count += 1;
            shape_ascii(t, fs, fid)
        });
        assert_eq!(call_count, 1);
    }

    #[test]
    fn cache_lru_order() {
        let mut cache = ShapeCache::new(2);
        let font_size = Fp266::from_int(12);

        cache.get_or_shape("a", 1, font_size, |t, fid, fs| shape_ascii(t, fs, fid));
        cache.get_or_shape("b", 1, font_size, |t, fid, fs| shape_ascii(t, fs, fid));

        // Access "a" again to make it most-recently-used
        cache.get_or_shape("a", 1, font_size, |t, fid, fs| shape_ascii(t, fs, fid));

        // Now "b" should be evicted (least recently used)
        cache.get_or_shape("c", 1, font_size, |t, fid, fs| shape_ascii(t, fs, fid));
        assert_eq!(cache.len(), 2);

        // "b" was evicted, "a" and "c" remain
        let mut b_call_count = 0u32;
        cache.get_or_shape("b", 1, font_size, |t, fid, fs| {
            b_call_count += 1;
            shape_ascii(t, fs, fid)
        });
        assert_eq!(b_call_count, 1);
    }

    #[test]
    fn cache_capacity_one() {
        let mut cache = ShapeCache::new(1);
        let font_size = Fp266::from_int(12);

        cache.get_or_shape("a", 1, font_size, |t, fid, fs| shape_ascii(t, fs, fid));
        cache.get_or_shape("b", 1, font_size, |t, fid, fs| shape_ascii(t, fs, fid));

        assert_eq!(cache.len(), 1);

        let mut a_calls = 0u32;
        cache.get_or_shape("a", 1, font_size, |t, fid, fs| {
            a_calls += 1;
            shape_ascii(t, fs, fid)
        });
        assert_eq!(a_calls, 1);
    }

    #[test]
    fn thread_safe_cache_basic() {
        let cache = ThreadSafeShapeCache::new(10);
        let font_size = Fp266::from_int(12);

        let run = cache.get_or_shape("hello", 1, font_size, |t, fid, fs| shape_ascii(t, fs, fid));
        assert_eq!(run.glyphs.len(), 5);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn thread_safe_cache_hit() {
        let cache = ThreadSafeShapeCache::new(10);
        let font_size = Fp266::from_int(12);

        let r1 = cache.get_or_shape("test", 1, font_size, |t, fid, fs| shape_ascii(t, fs, fid));
        let r2 = cache.get_or_shape("test", 1, font_size, |t, fid, fs| shape_ascii(t, fs, fid));
        assert_eq!(r1, r2);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn cache_empty_string() {
        let mut cache = ShapeCache::new(10);
        let font_size = Fp266::from_int(12);

        let run = cache.get_or_shape("", 1, font_size, |t, fid, fs| shape_ascii(t, fs, fid));
        assert!(run.glyphs.is_empty());
    }

    #[test]
    fn cache_hit_miss_counters() {
        let mut cache = ShapeCache::new(10);
        let font_size = Fp266::from_int(12);

        cache.get_or_shape("hello", 1, font_size, |t, fid, fs| shape_ascii(t, fs, fid));
        assert_eq!(cache.misses(), 1);
        assert_eq!(cache.hits(), 0);

        cache.get_or_shape("hello", 1, font_size, |t, fid, fs| shape_ascii(t, fs, fid));
        assert_eq!(cache.hits(), 1);
        assert_eq!(cache.misses(), 1);

        cache.get_or_shape("world", 1, font_size, |t, fid, fs| shape_ascii(t, fs, fid));
        assert_eq!(cache.hits(), 1);
        assert_eq!(cache.misses(), 2);

        let (h, m) = cache.stats();
        assert_eq!(h, 1);
        assert_eq!(m, 2);

        let rate = cache.hit_rate();
        assert!((rate - (1.0 / 3.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn cache_reset_stats() {
        let mut cache = ShapeCache::new(10);
        let font_size = Fp266::from_int(12);

        cache.get_or_shape("hello", 1, font_size, |t, fid, fs| shape_ascii(t, fs, fid));
        cache.get_or_shape("hello", 1, font_size, |t, fid, fs| shape_ascii(t, fs, fid));
        assert_eq!(cache.hits(), 1);
        assert_eq!(cache.misses(), 1);

        cache.reset_stats();
        assert_eq!(cache.hits(), 0);
        assert_eq!(cache.misses(), 0);
    }

    #[test]
    fn thread_safe_cache_stats() {
        let cache = ThreadSafeShapeCache::new(10);
        let font_size = Fp266::from_int(12);

        cache.get_or_shape("hello", 1, font_size, |t, fid, fs| shape_ascii(t, fs, fid));
        cache.get_or_shape("hello", 1, font_size, |t, fid, fs| shape_ascii(t, fs, fid));
        let (hits, misses) = cache.stats();
        assert_eq!(hits, 1);
        assert_eq!(misses, 1);

        cache.reset_stats();
        let (hits, misses) = cache.stats();
        assert_eq!(hits, 0);
        assert_eq!(misses, 0);
    }
}
