//! LRU cache for shaped runs (TASK-016).
//!
//! Thread-safe via internal `Mutex`. Lock-free variant is deferred to Phase D.
//! Uses `Arc<str>` keys and `Arc<ShapedRun>` values for O(1) clone on cache hit.

#![allow(dead_code)]
#![allow(clippy::expect_used)]

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::fp266::Fp266;
use crate::shaping::ShapedRun;

/// Cache key: (text, font_id, font_size).
#[derive(Clone, Debug, Eq)]
struct CacheKey {
    text: Arc<str>,
    font_id: u32,
    font_size_raw: i64,
}

impl Hash for CacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.text.hash(state);
        self.font_id.hash(state);
        self.font_size_raw.hash(state);
    }
}

impl PartialEq for CacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
            && self.font_id == other.font_id
            && self.font_size_raw == other.font_size_raw
    }
}

/// A simple LRU cache for shaped text runs.
///
/// Uses a `HashMap` with an access-order list for eviction.
/// Keys are `Arc<str>` and values are `Arc<ShapedRun>` for cheap cloning.
pub struct ShapeCache {
    capacity: usize,
    entries: HashMap<CacheKey, Arc<ShapedRun>>,
    access_order: Vec<CacheKey>,
    hits: u64,
    misses: u64,
}

impl ShapeCache {
    /// Create a new cache with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            entries: HashMap::with_capacity(capacity),
            access_order: Vec::with_capacity(capacity),
            hits: 0,
            misses: 0,
        }
    }

    /// Get a cached run or shape it using the provided shaper function.
    ///
    /// The `shaper` is only called on a cache miss.
    pub fn get_or_shape<F>(
        &mut self,
        text: &str,
        font_id: u32,
        font_size: Fp266,
        shaper: F,
    ) -> ShapedRun
    where
        F: FnOnce(&str, u32, Fp266) -> ShapedRun,
    {
        let key = CacheKey {
            text: text.into(),
            font_id,
            font_size_raw: font_size.raw(),
        };

        if let Some(run) = self.entries.get(&key) {
            self.access_order.retain(|k| k != &key);
            self.access_order.push(key);
            self.hits += 1;
            return (**run).clone();
        }

        self.misses += 1;
        let run = shaper(text, font_id, font_size);

        if self.entries.len() >= self.capacity
            && let Some(evict_key) = self.access_order.first()
        {
            let evict_key = evict_key.clone();
            self.access_order.remove(0);
            self.entries.remove(&evict_key);
        }

        self.access_order.push(key.clone());
        self.entries.insert(key, Arc::new(run.clone()));
        run
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

/// Thread-safe wrapper around [`ShapeCache`].
pub struct ThreadSafeShapeCache {
    inner: std::sync::Mutex<ShapeCache>,
}

impl ThreadSafeShapeCache {
    /// Create a new thread-safe cache with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: std::sync::Mutex::new(ShapeCache::new(capacity)),
        }
    }

    /// Get a cached run or shape it using the provided shaper function.
    pub fn get_or_shape<F>(
        &self,
        text: &str,
        font_id: u32,
        font_size: Fp266,
        shaper: F,
    ) -> ShapedRun
    where
        F: FnOnce(&str, u32, Fp266) -> ShapedRun,
    {
        let mut cache = self.inner.lock().expect("shape cache lock poisoned");
        cache.get_or_shape(text, font_id, font_size, shaper)
    }

    /// Number of entries currently in the cache.
    pub fn len(&self) -> usize {
        self.inner.lock().expect("shape cache lock poisoned").len()
    }

    /// Check if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.inner
            .lock()
            .expect("shape cache lock poisoned")
            .is_empty()
    }

    /// Get (hits, misses) statistics.
    pub fn stats(&self) -> (u64, u64) {
        self.inner
            .lock()
            .expect("shape cache lock poisoned")
            .stats()
    }

    /// Reset hit/miss counters.
    pub fn reset_stats(&self) {
        self.inner
            .lock()
            .expect("shape cache lock poisoned")
            .reset_stats()
    }

    /// Cache hit rate as a value between 0.0 and 1.0.
    pub fn hit_rate(&self) -> f64 {
        self.inner
            .lock()
            .expect("shape cache lock poisoned")
            .hit_rate()
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
