//! Font database backed by `fontdb` (Phase A-1).
//!
//! Provides system font discovery and named font lookup.

use std::sync::Arc;

use fontdb::{Database, Family, ID, Query, Style, Weight};

use crate::font::loader::load_font_with_index;

/// Font database backed by `fontdb` for system font discovery and lookup.
pub struct FontDatabase {
    inner: Database,
}

impl FontDatabase {
    /// Creates a new empty font database.
    pub fn new() -> Self {
        Self {
            inner: Database::new(),
        }
    }

    /// Loads all system fonts into the database.
    ///
    /// Returns the number of new font faces added.
    pub fn load_system_fonts(&mut self) -> usize {
        let before = self.inner.len();
        self.inner.load_system_fonts();
        self.inner.len() - before
    }

    /// Loads a font from raw binary data, returning face IDs.
    pub fn load_font_data(&mut self, data: Arc<Vec<u8>>) -> Vec<ID> {
        let source = fontdb::Source::Binary(data);
        self.inner.load_font_source(source).into_iter().collect()
    }

    /// Returns face information for the given face ID.
    pub fn face_info(&self, id: ID) -> Option<&fontdb::FaceInfo> {
        self.inner.face(id)
    }

    /// Returns an iterator over all face info entries in the database.
    pub fn face_info_iter(&self) -> impl Iterator<Item = &fontdb::FaceInfo> {
        self.inner.faces()
    }

    /// Queries the database for a font matching the given family name.
    pub fn query(&self, family: &str) -> Option<ID> {
        let families = &[Family::Name(family)][..];
        let q = Query {
            families,
            ..Default::default()
        };
        self.inner.query(&q)
    }

    /// Queries the database for a font matching family and style.
    pub fn query_with_style(&self, family: &str, style: &str) -> Option<ID> {
        let parsed_style = match style.to_lowercase().as_str() {
            "italic" => Style::Italic,
            "oblique" => Style::Oblique,
            _ => Style::Normal,
        };
        let families = &[Family::Name(family)][..];
        let q = Query {
            families,
            style: parsed_style,
            weight: Weight::NORMAL,
            ..Default::default()
        };
        self.inner.query(&q)
    }

    /// Returns the font source and face index for the given face ID.
    pub fn face_source(&self, id: ID) -> Option<(fontdb::Source, u32)> {
        self.inner.face_source(id)
    }

    /// Executes a callback with the face data and index for the given ID.
    pub fn with_face_data<T>(&self, id: ID, f: impl FnOnce(&[u8], u32) -> T) -> Option<T> {
        self.inner.with_face_data(id, f)
    }

    fn extract_data(source: &fontdb::Source) -> Option<Arc<Vec<u8>>> {
        match source {
            fontdb::Source::Binary(arc) => {
                let bytes = arc.as_ref().as_ref();
                Some(Arc::new(bytes.to_vec()))
            }
            _ => None,
        }
    }

    /// Loads and returns a `LoadedFont` for the given face ID.
    pub fn load_face(&self, id: ID) -> Option<LoadedFont> {
        let (source, face_index) = self.face_source(id)?;
        let data = Self::extract_data(&source)?;
        load_font_with_index(data, face_index).ok()
    }

    /// Returns a clone of the font data for the given face ID.
    ///
    /// Works with all source types (Binary, File, SharedFile).
    pub fn face_data(&self, id: ID) -> Option<Arc<Vec<u8>>> {
        self.with_face_data(id, |data, _| Arc::new(data.to_vec()))
    }

    /// Returns the number of font faces in the database.
    pub fn face_count(&self) -> usize {
        self.inner.len()
    }

    /// Alias for [`face_count`](Self::face_count), following Rust collection conventions.
    pub fn len(&self) -> usize {
        self.face_count()
    }

    /// Returns true if the database contains no font faces.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Queries the database for a monospace font, with common fallbacks.
    pub fn query_monospace(&self) -> Option<ID> {
        self.query("DejaVu Sans Mono")
            .or_else(|| self.query("Courier New"))
            .or_else(|| self.query("Liberation Mono"))
            .or_else(|| self.query("Noto Sans Mono"))
            .or_else(|| self.query("monospace"))
    }

    /// Queries for a specific style variant of a font family.
    pub fn query_family_style(&self, family: &str, weight: Weight, style: Style) -> Option<ID> {
        let families = &[Family::Name(family)][..];
        let q = Query {
            families,
            weight,
            style,
            ..Default::default()
        };
        self.inner.query(&q)
    }

    /// Builds a fallback chain for a given primary font.
    ///
    /// Returns primary ID followed by fallback IDs for CJK, symbols, etc.
    pub fn fallback_chain(&self, primary_id: ID) -> Vec<ID> {
        let mut chain = vec![primary_id];
        for fallback in &[
            "Noto Sans",
            "DejaVu Sans",
            "Symbola",
            "Arial Unicode MS",
            "Noto Sans CJK SC",
        ] {
            if let Some(id) = self.query(fallback)
                && !chain.contains(&id)
            {
                chain.push(id);
            }
        }
        chain
    }
}

impl Default for FontDatabase {
    fn default() -> Self {
        Self::new()
    }
}

use crate::font::loader::LoadedFont;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_database_is_empty() {
        let db = FontDatabase::new();
        assert!(db.inner.is_empty());
    }

    #[test]
    fn load_font_data_returns_ids() {
        let mut db = FontDatabase::new();
        let path = "/usr/share/fonts/TTF/DejaVuSans.ttf";
        let data = Arc::new(std::fs::read(path).expect("test font should exist"));
        let ids = db.load_font_data(data);
        assert!(!ids.is_empty());
    }

    #[test]
    fn query_loaded_font() {
        let mut db = FontDatabase::new();
        let path = "/usr/share/fonts/TTF/DejaVuSans.ttf";
        let data = Arc::new(std::fs::read(path).expect("test font should exist"));
        db.load_font_data(data);
        let id = db.query("DejaVu Sans");
        assert!(id.is_some());
    }

    #[test]
    fn face_info_for_loaded_font() {
        let mut db = FontDatabase::new();
        let path = "/usr/share/fonts/TTF/DejaVuSans.ttf";
        let data = Arc::new(std::fs::read(path).expect("test font should exist"));
        let ids = db.load_font_data(data);
        let id = ids.into_iter().next().unwrap();
        let info = db.face_info(id);
        assert!(info.is_some());
        let info = info.unwrap();
        assert!(!info.families.is_empty());
    }

    #[test]
    fn load_face_from_database() {
        let mut db = FontDatabase::new();
        let path = "/usr/share/fonts/TTF/DejaVuSans.ttf";
        let data = Arc::new(std::fs::read(path).expect("test font should exist"));
        let ids = db.load_font_data(data);
        let id = ids.into_iter().next().unwrap();
        let loaded = db.load_face(id);
        assert!(loaded.is_some());
    }

    #[test]
    fn query_nonexistent_font_returns_none() {
        let mut db = FontDatabase::new();
        let path = "/usr/share/fonts/TTF/DejaVuSans.ttf";
        let data = Arc::new(std::fs::read(path).expect("test font should exist"));
        db.load_font_data(data);
        let id = db.query("NonExistentFont");
        assert!(id.is_none());
    }

    #[test]
    fn system_fonts_load() {
        let mut db = FontDatabase::new();
        let count = db.load_system_fonts();
        assert!(
            count > 0 || !db.inner.is_empty(),
            "should load system fonts"
        );
        let id = db.query("DejaVu Sans");
        assert!(id.is_some());
    }

    #[test]
    fn face_count_empty() {
        let db = FontDatabase::new();
        assert_eq!(db.face_count(), 0);
    }

    #[test]
    fn face_count_after_load() {
        let mut db = FontDatabase::new();
        db.load_system_fonts();
        assert!(db.face_count() > 0);
    }

    #[test]
    fn query_monospace_finds_font() {
        let mut db = FontDatabase::new();
        db.load_system_fonts();
        let id = db.query_monospace();
        if db.face_count() > 0 {
            assert!(id.is_some(), "should find a monospace font");
        }
    }

    #[test]
    fn query_family_style() {
        let mut db = FontDatabase::new();
        db.load_system_fonts();
        let result = db.query_family_style("DejaVu Sans", Weight::BOLD, Style::Normal);
        if db.face_count() > 0 {
            assert!(result.is_some(), "should find bold DejaVu Sans");
        }
    }

    #[test]
    fn fallback_chain_includes_primary() {
        let mut db = FontDatabase::new();
        let path = "/usr/share/fonts/TTF/DejaVuSans.ttf";
        let data = Arc::new(std::fs::read(path).expect("test font should exist"));
        let ids = db.load_font_data(data);
        let primary = ids
            .into_iter()
            .next()
            .expect("should have at least one face");
        let chain = db.fallback_chain(primary);
        assert!(!chain.is_empty());
        assert_eq!(chain[0], primary);
    }

    #[test]
    fn face_data_from_file_source() {
        let mut db = FontDatabase::new();
        db.load_system_fonts();
        let id = db.query("DejaVu Sans");
        if let Some(id) = id {
            let data = db.face_data(id);
            assert!(
                data.is_some(),
                "face_data should work for file-backed fonts"
            );
            assert!(data.as_ref().unwrap().len() > 0);
        }
    }
}
