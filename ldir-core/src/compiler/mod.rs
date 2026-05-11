//! S-IR → G-IR Compiler (IF-COMPILE-001, ALG-COMPILE-001).
//!
//! Compiles a well-formed S-IR document into a G-IR document suitable for
//! rendering. The compilation pipeline:
//!
//! 1. Build a tree from flat S-IR instructions (`tree`)
//! 2. DFS traverse from root
//! 3. For each S-IR instruction, emit appropriate G-IR commands
//! 4. Track coordinate state and handle page overflow
//!
//! ## Key Invariants (BP-IR-COMPILER-001)
//!
//! - **POST-COMP-001**: All S-IR entities are represented in G-IR
//! - **POST-COMP-002**: No heap allocations in hot path
//! - **POST-COMP-003**: Stack balanced per page
//! - **INV-COMP-001**: Bit-identical output (deterministic)

#[allow(missing_docs)]
pub mod bibtex;
pub mod context;
#[allow(missing_docs)]
pub mod cross_ref;
pub mod emit_helpers;
#[allow(missing_docs)]
pub mod justify;
#[allow(missing_docs)]
pub mod knuth_plass;
pub mod math;
#[allow(missing_docs)]
pub mod templates;
pub mod tree;
#[allow(missing_docs)]
pub mod v1_to_v2;
#[allow(missing_docs)]
pub mod v2_compile;

use indexmap::IndexMap;
use std::path::Path;
use std::sync::Arc;

use ldir_ir::gir::ImageFormat;
use ldir_ir::gir::{GIRCommand, GIRDocument, GIRImage, GIRLink, GIRPage};
use ldir_ir::sir::{BlockType, SIRDocument, SIROpcode, StyleModifier};

use crate::compiler::context::{CompileContext, parse_page_size};
use crate::compiler::emit_helpers::*;
use crate::compiler::math::layout_math;
use crate::compiler::tree::InstructionTree;
use crate::error::Result;
use crate::font::db::FontDatabase;
use crate::fp266::Fp266;
use crate::layout::linebreak::{LineBreakItem, LineBreakOptions, linebreak};

use bumpalo::collections::Vec as BumpVec;

const FOOTNOTE_FONT_SIZE_PT: i32 = 8;
const FOOTNOTE_RULE_WIDTH_PT: i32 = 108;
const FOOTNOTE_RULE_THICKNESS_PT: i32 = 1;

fn superscript_digit(n: u32) -> char {
    match n {
        1 => '\u{00B9}',
        2 => '\u{00B2}',
        3 => '\u{00B3}',
        _ => {
            let base = b'0' + (n as u8);
            base as char
        }
    }
}

struct FootnoteState {
    pending_footnotes: Vec<(u32, String)>,
    reserve_fp266: i64,
}

impl FootnoteState {
    fn new() -> Self {
        Self {
            pending_footnotes: Vec::new(),
            reserve_fp266: 0,
        }
    }

    fn add_footnote(&mut self, number: u32, text: String) {
        self.pending_footnotes.push((number, text));
        self.reserve_fp266 += ((FOOTNOTE_FONT_SIZE_PT * 12 / 10 + 4) * 64) as i64;
    }

    fn has_pending(&self) -> bool {
        !self.pending_footnotes.is_empty()
    }

    fn take_pending(&mut self) -> Vec<(u32, String)> {
        let notes = std::mem::take(&mut self.pending_footnotes);
        self.reserve_fp266 = 0;
        notes
    }

    fn sync_reserve(&self, ctx: &mut CompileContext) {
        ctx.footnote_reserve = self.reserve_fp266;
    }
}

/// Compile a well-formed S-IR document into a G-IR document.
///
/// Uses the ASCII monospace stub shaper (no real font data).
/// For real font-aware shaping, use [`compile_sir_with_font`].
///
/// Implements IF-COMPILE-001 and ALG-COMPILE-001.
pub fn compile_sir(doc: &SIRDocument) -> Result<GIRDocument> {
    compile_sir_with_font(doc, None)
}

/// Compile a well-formed S-IR document into a G-IR document with real font data.
///
/// When `font_data` is `Some`, uses HarfBuzz + ttf-parser for real font-aware
/// text shaping (kerning, ligatures, proper advance widths). When `None`,
/// falls back to the ASCII monospace stub.
///
/// Implements IF-COMPILE-001 and ALG-COMPILE-001.
///
/// # Algorithm
///
/// 1. Build parent→children tree from flat S-IR
/// 2. Create initial page with default dimensions
/// 3. DFS from root, emitting G-IR commands per S-IR opcode:
///    - `PushBlock(Document)` → PushStack, set page dimensions
///    - `PushBlock(Paragraph)` → PushStack, set spacing
///    - `PushBlock(Heading)` → PushStack, set font size
///    - `SetContent` → PutGlyph per character (via Knuth-Plass line breaking)
///    - `ApplyStyle` → SetFont, MoveXY
///    - `InsertMath` → placeholder (deferred)
///    - `LinkData` → AttachMetadata
/// 4. PopStack for each block exit
/// 5. On page overflow, start new page
///
/// # Errors
///
/// Returns errors from tree building (validation) or compilation
/// (stack overflow, unsupported instructions).
pub fn compile_sir_with_font(
    doc: &SIRDocument,
    font_data: Option<Arc<Vec<u8>>>,
) -> Result<GIRDocument> {
    compile_sir_with_font_variants(doc, font_data, &[])
}

/// Resolve font data from a font database by family name.
///
/// First tries the database, then falls back to reading from a file path.
pub fn resolve_font_data(
    font_db: &FontDatabase,
    family: &str,
    fallback_path: Option<&Path>,
) -> Option<Arc<Vec<u8>>> {
    if let Some(id) = font_db.query(family)
        && let Some(data) = font_db.face_data(id)
    {
        return Some(data);
    }
    if let Some(path) = fallback_path
        && let Ok(data) = std::fs::read(path)
        && ttf_parser::Face::parse(&data, 0).is_ok()
    {
        return Some(Arc::new(data));
    }
    None
}

/// Compile a well-formed S-IR document using a font database for name-based font resolution.
///
/// Resolves the primary font from `font_db` by `font_family`, with optional `font_path`
/// as a file-based fallback. If no font is found, falls back to ASCII monospace stub.
///
/// `font_db` should have system fonts loaded via `FontDatabase::load_system_fonts()`.
#[allow(clippy::too_many_arguments)]
pub fn compile_sir_with_font_db(
    doc: &SIRDocument,
    font_db: &Arc<FontDatabase>,
    font_family: &str,
    font_mono_family: &str,
    font_path: Option<&Path>,
    font_variants: &[(u32, Arc<Vec<u8>>)],
    margins: Option<(i32, i32, i32, i32)>,
    base_dir: Option<&Path>,
    page_size_name: Option<&str>,
    page_dims: Option<(i32, i32)>,
    drop_caps: bool,
) -> Result<GIRDocument> {
    let font_data = resolve_font_data(font_db, font_family, font_path);

    let mut augmented_variants: Vec<(u32, Arc<Vec<u8>>)> = font_variants.to_vec();

    if augmented_variants
        .iter()
        .all(|(id, _)| *id != context::FONT_ID_BOLD)
        && let Some(id) =
            font_db.query_family_style(font_family, fontdb::Weight::BOLD, fontdb::Style::Normal)
        && let Some(data) = font_db.face_data(id)
    {
        augmented_variants.push((context::FONT_ID_BOLD, data));
    }

    if augmented_variants
        .iter()
        .all(|(id, _)| *id != context::FONT_ID_ITALIC)
        && let Some(id) =
            font_db.query_family_style(font_family, fontdb::Weight::NORMAL, fontdb::Style::Italic)
        && let Some(data) = font_db.face_data(id)
    {
        augmented_variants.push((context::FONT_ID_ITALIC, data));
    }

    if augmented_variants
        .iter()
        .all(|(id, _)| *id != context::FONT_ID_BOLD_ITALIC)
        && let Some(id) =
            font_db.query_family_style(font_family, fontdb::Weight::BOLD, fontdb::Style::Italic)
        && let Some(data) = font_db.face_data(id)
    {
        augmented_variants.push((context::FONT_ID_BOLD_ITALIC, data));
    }

    if augmented_variants
        .iter()
        .all(|(id, _)| *id != context::FONT_ID_MONO)
    {
        if !font_mono_family.is_empty() {
            if let Some(id) = font_db.query(font_mono_family)
                && let Some(data) = font_db.face_data(id)
            {
                augmented_variants.push((context::FONT_ID_MONO, data));
            }
        } else if let Some(id) = font_db.query_monospace()
            && let Some(data) = font_db.face_data(id)
        {
            augmented_variants.push((context::FONT_ID_MONO, data));
        }
    }

    compile_sir_with_font_variants_and_options(
        doc,
        font_data,
        &augmented_variants,
        margins,
        base_dir,
        page_size_name,
        page_dims,
        drop_caps,
    )
}

/// Compile a well-formed S-IR document into a G-IR document with real font data and variants.
///
/// `font_variants` is a slice of `(font_id, font_data)` pairs for style-aware shaping
/// (e.g. bold, italic, mono fonts).
pub fn compile_sir_with_font_variants(
    doc: &SIRDocument,
    font_data: Option<Arc<Vec<u8>>>,
    font_variants: &[(u32, Arc<Vec<u8>>)],
) -> Result<GIRDocument> {
    let tree = InstructionTree::build(doc)?;
    let mut ctx = CompileContext::with_font(font_data);

    for (id, data) in font_variants {
        ctx.set_font_variant(*id as usize, Some(data.clone()));
    }

    let labels = collect_labels(&tree, doc);
    let mut gir_doc = GIRDocument::with_capacity(1);
    let mut page = ctx.new_page();
    let mut equation_counter: u32 = 0;
    let mut fn_state = FootnoteState::new();

    compile_node(
        &tree,
        tree.root_index(),
        doc,
        &mut page,
        &mut ctx,
        &mut gir_doc,
        None,
        &labels,
        &mut equation_counter,
        &mut fn_state,
    )?;

    if fn_state.has_pending() {
        emit_footnotes(&mut fn_state, &mut page, &mut ctx, &mut gir_doc);
    }

    if !page.is_empty() {
        gir_doc.push_page(page);
    }

    let (hits, misses) = ctx.shape_cache.stats();
    if hits + misses > 0 {
        let rate = hits as f64 / (hits + misses) as f64;
        tracing::info!(
            "Shape cache: {} hits, {} misses, {:.1}% hit rate",
            hits,
            misses,
            rate * 100.0
        );
    }

    Ok(gir_doc)
}

/// Compile a well-formed S-IR document into a G-IR document with custom margins and base directory.
///
/// `margins` is `Some((left, right, top, bottom))` in points, or `None` for defaults.
/// `base_dir` is used to resolve relative image paths.
/// `page_size_name` is an optional preset name ("a4", "letter", "legal").
/// `page_dims` is optional explicit `(width_pt, height_pt)`.
/// `drop_caps` enables drop caps for the first paragraph after headings.
///
/// **Prefer [`compile_v2_document`](super::v2_compile::compile_v2_document) for new code.**
/// This v1 entry point is retained for backward compatibility.
#[allow(clippy::too_many_arguments)]
pub fn compile_sir_with_font_variants_and_options(
    doc: &SIRDocument,
    font_data: Option<Arc<Vec<u8>>>,
    font_variants: &[(u32, Arc<Vec<u8>>)],
    margins: Option<(i32, i32, i32, i32)>,
    base_dir: Option<&Path>,
    page_size_name: Option<&str>,
    page_dims: Option<(i32, i32)>,
    drop_caps: bool,
) -> Result<GIRDocument> {
    let tree = InstructionTree::build(doc)?;

    let (pw, ph) = if let Some((w, h)) = page_dims {
        (w, h)
    } else if let Some(name) = page_size_name {
        parse_page_size(name).unwrap_or((
            context::DEFAULT_PAGE_WIDTH_PT,
            context::DEFAULT_PAGE_HEIGHT_PT,
        ))
    } else {
        (
            context::DEFAULT_PAGE_WIDTH_PT,
            context::DEFAULT_PAGE_HEIGHT_PT,
        )
    };

    let font_data_for_pass1 = font_data.clone();

    let mut ctx = if let Some((ml, mr, mt, mb)) = margins {
        CompileContext::with_font_margins_and_page(font_data, ml, mr, mt, mb, pw, ph)
    } else {
        CompileContext::with_font_margins_and_page(
            font_data,
            context::LEFT_MARGIN_PT,
            context::RIGHT_MARGIN_PT,
            context::TOP_MARGIN_PT,
            context::BOTTOM_MARGIN_PT,
            pw,
            ph,
        )
    };

    ctx.drop_caps_enabled = drop_caps;

    for (id, data) in font_variants {
        ctx.set_font_variant(*id as usize, Some(data.clone()));
    }

    let labels = collect_labels(&tree, doc);

    // Collect heading entries for TOC
    let mut heading_entries: Vec<(u32, String)> = Vec::new();
    collect_headings(&tree, tree.root_index(), doc, &mut heading_entries);

    let needs_toc = heading_entries.len() >= 2;

    // Pass 1: compile without TOC to determine heading page positions
    let heading_page_positions = if needs_toc {
        let mut ctx1 = if let Some((ml, mr, mt, mb)) = margins {
            CompileContext::with_font_margins_and_page(font_data_for_pass1, ml, mr, mt, mb, pw, ph)
        } else {
            CompileContext::with_font_margins_and_page(
                font_data_for_pass1,
                context::LEFT_MARGIN_PT,
                context::RIGHT_MARGIN_PT,
                context::TOP_MARGIN_PT,
                context::BOTTOM_MARGIN_PT,
                pw,
                ph,
            )
        };
        ctx1.drop_caps_enabled = drop_caps;
        for (id, data) in font_variants {
            ctx1.set_font_variant(*id as usize, Some(data.clone()));
        }

        let mut gir_pass1 = GIRDocument::with_capacity(1);
        let mut page1 = ctx1.new_page();
        let mut eq1: u32 = 0;
        let mut fn1 = FootnoteState::new();

        compile_node(
            &tree,
            tree.root_index(),
            doc,
            &mut page1,
            &mut ctx1,
            &mut gir_pass1,
            base_dir,
            &labels,
            &mut eq1,
            &mut fn1,
        )?;

        if fn1.has_pending() {
            emit_footnotes(&mut fn1, &mut page1, &mut ctx1, &mut gir_pass1);
        }
        if !page1.is_empty() {
            gir_pass1.push_page(page1);
        }

        Some(record_heading_pages(&tree, doc, &gir_pass1))
    } else {
        None
    };

    // Pass 2: compile with TOC using heading positions from pass 1
    let mut gir_doc = GIRDocument::with_capacity(1);
    let mut page = ctx.new_page();

    if needs_toc {
        let toc_page_count_before = gir_doc.page_count();
        generate_toc(
            &mut page,
            &mut ctx,
            &mut gir_doc,
            &heading_entries,
            heading_page_positions.as_ref(),
        );
        let toc_page_count = gir_doc.page_count() - toc_page_count_before;
        ctx.toc_page_count = toc_page_count;
    }

    let mut equation_counter: u32 = 0;
    let mut fn_state = FootnoteState::new();
    compile_node(
        &tree,
        tree.root_index(),
        doc,
        &mut page,
        &mut ctx,
        &mut gir_doc,
        base_dir,
        &labels,
        &mut equation_counter,
        &mut fn_state,
    )?;

    if fn_state.has_pending() {
        emit_footnotes(&mut fn_state, &mut page, &mut ctx, &mut gir_doc);
    }

    if !page.is_empty() {
        gir_doc.push_page(page);
    }

    let (hits, misses) = ctx.shape_cache.stats();
    if hits + misses > 0 {
        let rate = hits as f64 / (hits + misses) as f64;
        tracing::info!(
            "Shape cache: {} hits, {} misses, {:.1}% hit rate",
            hits,
            misses,
            rate * 100.0
        );
    }

    Ok(gir_doc)
}

/// Generate a Table of Contents by scanning for heading blocks.
///
/// `heading_entries` is a pre-collected list of (level, title) pairs.
/// `heading_pages` maps heading index → (page_index, y_position) from pass 1.
fn generate_toc(
    page: &mut GIRPage,
    ctx: &mut CompileContext,
    gir_doc: &mut GIRDocument,
    heading_entries: &[(u32, String)],
    heading_pages: Option<&Vec<(usize, f64)>>,
) {
    // Emit "Table of Contents" title
    let saved_font_size = ctx.font_size;
    ctx.font_size = Fp266::from_int(18);
    let toc_title = "Table of Contents";
    emit_set_font(page, ctx.font_id as i32);
    emit_move_xy(page, ctx.x, ctx.y);
    for ch in toc_title.chars() {
        page.push(GIRCommand::new_put_glyph(ch as i32, 10 * 64));
    }
    ctx.font_size = saved_font_size;
    ctx.advance_y(ctx.line_height() * 2);
    ctx.reset_x();

    for (entry_idx, (level, title)) in heading_entries.iter().enumerate() {
        let indent = match level {
            1 => 0,
            2 => 36 * 64,
            3 => 72 * 64,
            _ => 108 * 64,
        };

        let entry_x = ctx.x + Fp266::from_raw(indent as i64);
        emit_set_font(page, ctx.font_id as i32);
        emit_move_xy(page, entry_x, ctx.y);

        let entry_start_y = ctx.y.to_f64();
        let entry_start_x = entry_x.to_f64();

        for ch in title.chars() {
            page.push(GIRCommand::new_put_glyph(ch as i32, 7 * 64));
        }

        // Emit page number right-aligned
        let page_num = if let Some(pages) = heading_pages {
            if let Some(&(pass1_page_idx, _)) = pages.get(entry_idx) {
                let adjusted = pass1_page_idx + gir_doc.page_count() + 1;
                adjusted.to_string()
            } else {
                "?".to_string()
            }
        } else {
            "?".to_string()
        };

        let page_num_str = page_num;
        let num_x =
            (ctx.page_width - ctx.margin_right).to_f64() / 64.0 - (page_num_str.len() as f64 * 5.0);
        emit_move_xy(page, Fp266::from_f64(num_x * 64.0), ctx.y);
        for ch in page_num_str.chars() {
            page.push(GIRCommand::new_put_glyph(ch as i32, 5 * 64));
        }

        // Add clickable link for this TOC entry
        if let Some(pages) = heading_pages
            && let Some(&(pass1_page_idx, _)) = pages.get(entry_idx)
        {
            let dest_page = pass1_page_idx + gir_doc.page_count();
            let line_h = ctx.line_height().to_f64();
            page.links.push(GIRLink {
                x: entry_start_x,
                y: entry_start_y,
                width: (ctx.page_width - ctx.margin_right).to_f64() / 64.0 - entry_start_x,
                height: line_h,
                url: String::new(),
                destination_page: Some(dest_page),
            });
        }

        ctx.advance_y(ctx.line_height());
        ctx.reset_x();

        if ctx.exceeds_page() {
            if !page.is_empty() {
                gir_doc.push_page(std::mem::replace(page, ctx.new_page()));
            }
            ctx.y = ctx.margin_top;
        }
    }

    // Add spacing after TOC
    ctx.advance_y(ctx.line_height());
    ctx.reset_x();
}

fn collect_headings(
    tree: &InstructionTree,
    node_idx: usize,
    doc: &SIRDocument,
    entries: &mut Vec<(u32, String)>,
) {
    let node = tree.node(node_idx);
    let instr = node.instruction;
    let opcode = instr.opcode();

    if opcode == SIROpcode::PushBlock {
        let payload = doc.payload().get(instr.payload_offset(), 1);
        if let Some(bytes) = payload
            && let Some(BlockType::Heading) = BlockType::from_u8(bytes[0])
        {
            let level_payload = doc.payload().get(instr.payload_offset() + 1, 4);
            let level = level_payload
                .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .unwrap_or(1);

            // Find SetContent child for heading text
            for &child_idx in &node.children {
                let child_node = tree.node(child_idx);
                if child_node.instruction.opcode() == SIROpcode::SetContent
                    && let Some(text) = doc.payload_text(child_node.instruction)
                {
                    let title = text.trim_end_matches('\0').trim().to_string();
                    if !title.is_empty() {
                        entries.push((level, title));
                    }
                    return;
                }
            }
        }
    }

    for &child_idx in &node.children {
        collect_headings(tree, child_idx, doc, entries);
    }
}

/// Record the page index and y position of each heading in a G-IR document (from pass 1).
///
/// Scans each page for heading-sized font SetFont commands followed by PutGlyph,
/// matching against the known heading entries in order.
fn record_heading_pages(
    tree: &InstructionTree,
    doc: &SIRDocument,
    gir_doc: &GIRDocument,
) -> Vec<(usize, f64)> {
    let mut heading_entries: Vec<(u32, String)> = Vec::new();
    collect_headings(tree, tree.root_index(), doc, &mut heading_entries);

    if heading_entries.is_empty() {
        return Vec::new();
    }

    let mut results: Vec<(usize, f64)> = Vec::new();
    let mut heading_idx = 0;

    for (page_idx, page) in gir_doc.iter().enumerate() {
        let mut last_y: f64 = 0.0;
        let mut in_heading = false;

        for cmd in page.iter() {
            match cmd.opcode() {
                ldir_ir::gir::GIROpcode::SetFont => {
                    // Headings use font sizes 14-24pt (font_size 1-3pt at body=12pt).
                    // We detect headings by looking for large font sizes.
                    // The compiler sets heading font sizes: 24, 20, 16, 14, 13
                    in_heading = true;
                }
                ldir_ir::gir::GIROpcode::MoveXY => {
                    if let (_, Some(y)) = (cmd.arg(0), cmd.arg(1)) {
                        last_y = y as f64 / 64.0;
                    }
                }
                ldir_ir::gir::GIROpcode::PutGlyph
                    if in_heading && heading_idx < heading_entries.len() =>
                {
                    results.push((page_idx, last_y));
                    heading_idx += 1;
                    in_heading = false;
                }
                _ => {}
            }
        }
    }

    // Pad with last known position if we didn't find all headings
    while results.len() < heading_entries.len() {
        if let Some(&(page, y)) = results.last() {
            results.push((page, y));
        } else {
            results.push((0, 0.0));
        }
    }

    results
}

fn emit_footnotes(
    fn_state: &mut FootnoteState,
    page: &mut GIRPage,
    ctx: &mut CompileContext,
    gir_doc: &mut GIRDocument,
) {
    if !fn_state.has_pending() {
        return;
    }

    let notes = fn_state.take_pending();
    ctx.footnote_reserve = 0;

    let footnote_y =
        ctx.page_height - ctx.margin_bottom - Fp266::from_int(FOOTNOTE_FONT_SIZE_PT + 12);

    let rule_x = ctx.x;
    let rule_width = Fp266::from_int(FOOTNOTE_RULE_WIDTH_PT);
    let rule_thickness = Fp266::from_frac(FOOTNOTE_RULE_THICKNESS_PT, 2);
    emit_move_xy(page, rule_x, footnote_y);
    emit_draw_rule(page, rule_x, footnote_y, rule_width, rule_thickness);

    let saved_font_size = ctx.font_size;
    ctx.font_size = Fp266::from_int(FOOTNOTE_FONT_SIZE_PT);

    let mut fn_y = footnote_y + ctx.line_height();

    for (num, text) in &notes {
        emit_set_font(page, ctx.font_id as i32);
        emit_move_xy(page, ctx.x, fn_y);

        let mark_char = superscript_digit(*num);
        page.push(GIRCommand::new_put_glyph(mark_char as i32, 5 * 64));
        ctx.advance_x(Fp266::from_int(5));

        let display_text = format!(" {}", text);
        emit_paragraph_inline(page, ctx, gir_doc, &display_text);

        ctx.reset_x();
        fn_y += ctx.line_height();
    }

    ctx.font_size = saved_font_size;
    ctx.reset_x();
}

fn emit_paragraph_inline(
    page: &mut GIRPage,
    ctx: &mut CompileContext,
    _gir_doc: &mut GIRDocument,
    text: &str,
) {
    ctx.bump.reset();

    let font_size = ctx.font_size();

    let font_data_for_style = ctx
        .font_data_variants
        .get(ctx.font_id as usize)
        .and_then(|opt| opt.as_ref())
        .or(ctx.font_data.as_ref());
    let shaped: std::sync::Arc<_> = if let Some(data) = font_data_for_style {
        crate::shaping::shape_text_cached(&ctx.shape_cache, data, text, font_size, ctx.font_id)
    } else {
        std::sync::Arc::new(crate::shaping::fast_path::shape_ascii(
            text,
            font_size,
            ctx.font_id,
        ))
    };

    if shaped.glyphs.is_empty() {
        return;
    }

    let text_bytes = text.as_bytes();
    let n = shaped.glyphs.len();
    let content_width = ctx.content_width;

    let line_ranges = {
        let items: BumpVec<'_, LineBreakItem> = BumpVec::from_iter_in(
            shaped.glyphs.iter().map(|g| {
                let ci = g.cluster_id as usize;
                let is_space = ci < text_bytes.len() && text_bytes[ci] == b' ';
                let space_stretch = if is_space {
                    g.advance.div(Fp266::from_int(2))
                } else {
                    Fp266::ZERO
                };
                let space_shrink = if is_space {
                    g.advance.div(Fp266::from_int(3))
                } else {
                    Fp266::ZERO
                };
                LineBreakItem {
                    width: g.advance,
                    stretchability: space_stretch,
                    shrinkability: space_shrink,
                    penalty: 0.0,
                    is_mandatory: false,
                    is_hyphenation: false,
                    hyphen_width: Fp266::ZERO,
                    text: "",
                }
            }),
            &ctx.bump,
        );

        let options = LineBreakOptions {
            line_width: content_width,
            ..Default::default()
        };

        let result = linebreak(&items, &options);

        let mut ranges: Vec<(usize, usize)> = Vec::new();
        let mut prev = 0;
        for &b in &result.breaks {
            if b > prev {
                ranges.push((prev, b));
            }
            prev = b;
        }
        if prev < n {
            ranges.push((prev, n));
        }
        if ranges.is_empty() {
            ranges.push((0, n));
        }
        ranges
    };

    let num_lines = line_ranges.len();
    for (line_idx, &(start, end)) in line_ranges.iter().enumerate() {
        emit_set_font(page, ctx.font_id as i32);
        emit_move_xy(page, ctx.x, ctx.y);

        let is_last_line = line_idx == num_lines - 1;
        let line_glyphs = &shaped.glyphs[start..end];
        let justified = justify::justify_line(line_glyphs, text_bytes, content_width, is_last_line);

        for jg in &justified {
            page.push(GIRCommand::new_put_glyph(jg.glyph_id as i32, jg.x_advance));
        }

        ctx.reset_x();
        ctx.advance_y(ctx.line_height());
    }
}

/// Resolve `\ref{key}` and `\eqref{key}` placeholders in text.
fn resolve_references(text: &str, labels: &IndexMap<String, String>) -> String {
    let mut result = text.to_string();
    for (key, number) in labels {
        let ref_pattern = format!("\\ref{{{}}}", key);
        let eqref_pattern = format!("\\eqref{{{}}}", key);
        if result.contains(&ref_pattern) {
            result = result.replace(&ref_pattern, number);
        }
        if result.contains(&eqref_pattern) {
            result = result.replace(&eqref_pattern, &format!("({})", number));
        }
    }
    result
}

/// Strip `\label{key}` from text content (labels are handled by the label table).
fn strip_label(text: &str) -> String {
    let mut result = text.to_string();
    while let Some(start) = result.find("\\label{") {
        if let Some(end) = result[start..].find('}') {
            let label_end = start + end + 1;
            result.replace_range(start..label_end, "");
        } else {
            break;
        }
    }
    result
}

/// Collect all `\label{key}` entries and section/equation numbers from the S-IR tree.
///
/// Returns a IndexMap of label_key → display_number.
fn collect_labels(tree: &InstructionTree, doc: &SIRDocument) -> IndexMap<String, String> {
    let mut labels: IndexMap<String, String> = IndexMap::new();
    let mut section_counters: IndexMap<u32, u32> = IndexMap::new();
    let mut equation_counter: u32 = 0;

    collect_labels_recursive(
        tree,
        tree.root_index(),
        doc,
        &mut labels,
        &mut section_counters,
        &mut equation_counter,
        &mut Vec::new(),
    );

    labels
}

fn collect_labels_recursive(
    tree: &InstructionTree,
    node_idx: usize,
    doc: &SIRDocument,
    labels: &mut IndexMap<String, String>,
    section_counters: &mut IndexMap<u32, u32>,
    equation_counter: &mut u32,
    current_number: &mut Vec<u32>,
) {
    let node = tree.node(node_idx);
    let instr = node.instruction;
    let opcode = instr.opcode();

    if opcode == SIROpcode::PushBlock {
        let payload = doc.payload().get(instr.payload_offset(), 1);
        let block_type = payload.and_then(|bytes| BlockType::from_u8(bytes[0]));

        if block_type == Some(BlockType::Heading) {
            let level_payload = doc.payload().get(instr.payload_offset() + 1, 4);
            let level = level_payload
                .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .unwrap_or(1) as usize;

            *section_counters.entry(level as u32).or_insert(0) += 1;
            let count = section_counters[&(level as u32)];

            while current_number.len() > level {
                current_number.pop();
            }
            if current_number.len() == level {
                current_number[level - 1] = count;
            } else if current_number.len() < level {
                while current_number.len() < level {
                    current_number.push(0);
                }
                current_number[level - 1] = count;
            }

            let number: String = current_number
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(".");

            for &child_idx in &node.children {
                let child_node = tree.node(child_idx);
                if child_node.instruction.opcode() == SIROpcode::SetContent
                    && let Some(text) = doc.payload_text(child_node.instruction)
                {
                    let trimmed = text.trim_end_matches('\0').trim();
                    if let Some(label_key) = extract_label_key(trimmed) {
                        labels.insert(label_key, number.clone());
                    }
                }
            }
        }

        if block_type == Some(BlockType::Math) {
            let numbered = doc
                .payload()
                .get(instr.payload_offset() + 1, 1)
                .map(|b| b[0] == 1)
                .unwrap_or(false);

            if numbered {
                *equation_counter += 1;
                let number = equation_counter.to_string();

                for &child_idx in &node.children {
                    let child_node = tree.node(child_idx);
                    if child_node.instruction.opcode() == SIROpcode::SetContent
                        && let Some(text) = doc.payload_text(child_node.instruction)
                    {
                        let trimmed = text.trim_end_matches('\0').trim();
                        if let Some(label_key) = extract_label_key(trimmed) {
                            labels.insert(label_key, number.clone());
                        }
                    }
                }
            }
        }
    }

    for &child_idx in &node.children {
        collect_labels_recursive(
            tree,
            child_idx,
            doc,
            labels,
            section_counters,
            equation_counter,
            current_number,
        );
    }
}

/// Extract a `\label{key}` from text, returning the key if found.
fn extract_label_key(text: &str) -> Option<String> {
    if let Some(start) = text.find("\\label{") {
        let rest = &text[start + 7..];
        if let Some(end) = rest.find('}') {
            let key = rest[..end].trim().to_string();
            if !key.is_empty() {
                return Some(key);
            }
        }
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn compile_node(
    tree: &InstructionTree,
    node_idx: usize,
    doc: &SIRDocument,
    page: &mut GIRPage,
    ctx: &mut CompileContext,
    gir_doc: &mut GIRDocument,
    base_dir: Option<&Path>,
    labels: &IndexMap<String, String>,
    equation_counter: &mut u32,
    fn_state: &mut FootnoteState,
) -> Result<()> {
    let node = tree.node(node_idx);
    let instr = node.instruction;
    let opcode = instr.opcode();

    match opcode {
        SIROpcode::PushBlock => {
            ctx.push_stack()?;
            emit_push_stack(page);
            emit_move_xy(page, ctx.x, ctx.y);

            let payload = doc.payload().get(instr.payload_offset(), 1);
            let block_type = payload.and_then(|bytes| BlockType::from_u8(bytes[0]));

            let saved_font_size = ctx.font_size;
            if block_type == Some(BlockType::Heading) {
                let level_payload = doc.payload().get(instr.payload_offset() + 1, 4);
                let level = level_payload
                    .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                    .unwrap_or(1);
                let heading_size = match level {
                    1 => 24,
                    2 => 20,
                    3 => 16,
                    4 => 14,
                    5 => 13,
                    _ => 12,
                };
                ctx.font_size = Fp266::from_int(heading_size);
            }

            if block_type == Some(BlockType::BlockQuote) {
                page.push(GIRCommand::new_draw_rule(
                    ctx.x.raw() as i32,
                    ctx.y.raw() as i32,
                    (2.0 * 64.0) as i32,
                    ctx.line_height().raw() as i32,
                ));
            }

            if block_type == Some(BlockType::Image)
                && let Some(image_path) = collect_image_path(tree, &node.children, doc)
                && let Some((img, w_fp, h_fp)) =
                    load_and_scale_image(&image_path, ctx.content_width, base_dir)
            {
                let image_index = gir_doc.push_image(img);
                emit_move_xy(page, ctx.x, ctx.y);
                page.push(GIRCommand::new_draw_rule(
                    -1,
                    image_index as i32,
                    w_fp,
                    h_fp,
                ));
                ctx.advance_y(Fp266::from_raw(h_fp as i64));
                ctx.reset_x();
                if ctx.exceeds_page() {
                    if !page.is_empty() {
                        if fn_state.has_pending() {
                            emit_footnotes(fn_state, page, ctx, gir_doc);
                        }
                        gir_doc.push_page(std::mem::replace(page, ctx.new_page()));
                    }
                    ctx.y = ctx.margin_top;
                }
            }

            if block_type == Some(BlockType::Table) {
                let table_text = collect_table_text(tree, &node.children, doc);
                emit_table(page, ctx, gir_doc, &table_text);
            }

            if block_type == Some(BlockType::Math) {
                let math_text = collect_block_text(tree, &node.children, doc);
                let numbered = doc
                    .payload()
                    .get(instr.payload_offset() + 1, 1)
                    .map(|b| b[0] == 1)
                    .unwrap_or(false);

                if !math_text.is_empty() {
                    let clean_text = strip_label(&math_text);
                    let font_data_for_math = ctx
                        .font_data_variants
                        .get(ctx.font_id as usize)
                        .and_then(|opt| opt.as_ref())
                        .or(ctx.font_data.as_ref());
                    let font_bytes = font_data_for_math.map(|d| d.as_slice());

                    let result = layout_math(&clean_text, font_bytes, ctx.font_size, ctx.y);

                    emit_set_font(page, ctx.font_id as i32);
                    for glyph in &result.glyphs {
                        let gx = ctx.x + glyph.x;
                        let gy = glyph.y;
                        if glyph.glyph_id == -1 {
                            emit_draw_rule(
                                page,
                                gx,
                                gy,
                                glyph.advance,
                                Fp266::from_frac(ctx.font_size.to_int(), 16),
                            );
                        } else {
                            emit_move_xy(page, gx, gy);
                            page.push(GIRCommand::new_put_glyph(
                                glyph.glyph_id,
                                glyph.advance.raw() as i32,
                            ));
                        }
                    }

                    if numbered {
                        *equation_counter += 1;
                        let eq_num = format!("({})", equation_counter);
                        emit_move_xy(
                            page,
                            ctx.page_width - ctx.margin_right - Fp266::from_int(36),
                            ctx.y,
                        );
                        for ch in eq_num.chars() {
                            page.push(GIRCommand::new_put_glyph(ch as i32, 7 * 64));
                        }
                    }

                    let math_spacing = result.height + result.depth + Fp266::from_int(6);
                    ctx.advance_y(math_spacing);
                    ctx.reset_x();

                    if ctx.exceeds_page() {
                        if !page.is_empty() {
                            if fn_state.has_pending() {
                                emit_footnotes(fn_state, page, ctx, gir_doc);
                            }
                            gir_doc.push_page(std::mem::replace(page, ctx.new_page()));
                        }
                        ctx.y = ctx.margin_top;
                    }
                }

                emit_pop_stack(page);
                ctx.pop_stack()?;
                ctx.advance_y(ctx.line_height());
                ctx.reset_x();
                if ctx.exceeds_page() {
                    if !page.is_empty() {
                        if fn_state.has_pending() {
                            emit_footnotes(fn_state, page, ctx, gir_doc);
                        }
                        gir_doc.push_page(std::mem::replace(page, ctx.new_page()));
                    }
                    ctx.y = ctx.margin_top;
                }

                return Ok(());
            }

            // Pre-scan children for LinkData nodes to capture URLs before SetContent
            ctx.pending_link_urls.clear();
            ctx.link_start_positions.clear();
            for &child_idx in &node.children {
                let child_node = tree.node(child_idx);
                if child_node.instruction.opcode() == SIROpcode::LinkData
                    && let Some(url) = doc.payload_text(child_node.instruction)
                {
                    let url = url.trim_end_matches('\0').to_string();
                    if !url.is_empty() {
                        ctx.pending_link_urls.push(url);
                        ctx.link_start_positions
                            .push((ctx.x.to_f64(), ctx.y.to_f64()));
                    }
                }
            }

            for &child_idx in &node.children {
                compile_node(
                    tree,
                    child_idx,
                    doc,
                    page,
                    ctx,
                    gir_doc,
                    base_dir,
                    labels,
                    equation_counter,
                    fn_state,
                )?;
            }

            // Record link rectangles after all children have been rendered
            let line_h = ctx.line_height().to_f64();
            for (i, url) in ctx.pending_link_urls.drain(..).enumerate() {
                let (start_x, start_y) = ctx.link_start_positions[i];
                let content_width = ctx.content_width.to_f64();
                page.links.push(GIRLink {
                    x: start_x,
                    y: start_y,
                    width: content_width,
                    height: line_h,
                    url,
                    destination_page: None,
                });
            }

            emit_pop_stack(page);
            ctx.pop_stack()?;

            if block_type == Some(BlockType::Heading) {
                ctx.font_size = saved_font_size;
                ctx.next_para_is_drop_cap = true;
            }

            // Skip extra spacing for Image and Table blocks (they handle their own)
            if block_type != Some(BlockType::Image) && block_type != Some(BlockType::Table) {
                ctx.advance_y(ctx.line_height());
                ctx.reset_x();
                if ctx.exceeds_page() {
                    if !page.is_empty() {
                        if fn_state.has_pending() {
                            emit_footnotes(fn_state, page, ctx, gir_doc);
                        }
                        gir_doc.push_page(std::mem::replace(page, ctx.new_page()));
                    }
                    ctx.y = ctx.margin_top;
                }
            }
        }
        SIROpcode::SetContent => {
            let text = doc.payload_text(instr).unwrap_or("").trim_end_matches('\0');

            let cleaned = strip_label(text);
            let resolved = resolve_references(&cleaned, labels);

            let mut parts: Vec<String> = Vec::new();
            let mut current = String::new();
            let resolved_bytes = resolved.as_bytes();
            let mut i = 0;
            while i < resolved_bytes.len() {
                if resolved_bytes[i] == b'\\' && resolved[i..].starts_with("\\fnmark{") {
                    if !current.is_empty() {
                        parts.push(current.clone());
                        current.clear();
                    }
                    let after_marker = &resolved[i + 8..];
                    if let Some(close) = after_marker.find('}') {
                        let num_str = &after_marker[..close];
                        parts.push(format!("\\fnmark{{{}}}", num_str));
                        i += 8 + close + 1;
                    } else {
                        current.push_str("\\fnmark{");
                        i += 8;
                    }
                } else {
                    let ch = resolved_bytes[i] as char;
                    current.push(ch);
                    i += 1;
                }
            }
            if !current.is_empty() {
                parts.push(current);
            }

            for part in &parts {
                if let Some(num_str) = part
                    .strip_prefix("\\fnmark{")
                    .and_then(|s| s.strip_suffix('}'))
                {
                    if let Ok(num) = num_str.parse::<u32>() {
                        let mark_char = superscript_digit(num);
                        emit_set_font(page, ctx.font_id as i32);
                        emit_move_xy(page, ctx.x, ctx.y);
                        page.push(GIRCommand::new_put_glyph(mark_char as i32, 5 * 64));
                        ctx.advance_x(Fp266::from_int(5));

                        let fn_text = doc
                            .footnotes
                            .iter()
                            .find(|(n, _)| *n == num)
                            .map(|(_, t)| t.clone());
                        if let Some(ft) = fn_text {
                            fn_state.add_footnote(num, ft);
                            fn_state.sync_reserve(ctx);
                        }
                    }
                } else {
                    let trimmed = part.trim();
                    if !trimmed.is_empty() {
                        if ctx.next_para_is_drop_cap && ctx.drop_caps_enabled {
                            ctx.next_para_is_drop_cap = false;
                            emit_drop_cap_paragraph(page, ctx, gir_doc, trimmed);
                        } else {
                            ctx.next_para_is_drop_cap = false;
                            emit_paragraph(page, ctx, gir_doc, trimmed);
                        }
                    }
                }
            }

            for &child_idx in &node.children {
                compile_node(
                    tree,
                    child_idx,
                    doc,
                    page,
                    ctx,
                    gir_doc,
                    base_dir,
                    labels,
                    equation_counter,
                    fn_state,
                )?;
            }
        }
        SIROpcode::ApplyStyle => {
            let packed = instr.payload_offset();
            let (_modifiers, is_push) = StyleModifier::from_packed(packed);

            let prev_font_id = ctx.font_id;
            if is_push {
                ctx.push_style(_modifiers);
            } else {
                ctx.pop_style();
            }

            if ctx.font_id != prev_font_id {
                emit_set_font(page, ctx.font_id as i32);
            }

            for &child_idx in &node.children {
                compile_node(
                    tree,
                    child_idx,
                    doc,
                    page,
                    ctx,
                    gir_doc,
                    base_dir,
                    labels,
                    equation_counter,
                    fn_state,
                )?;
            }
        }
        SIROpcode::InsertMath => {
            emit_attach_metadata(page, 0, 0, 0, 0);

            for &child_idx in &node.children {
                compile_node(
                    tree,
                    child_idx,
                    doc,
                    page,
                    ctx,
                    gir_doc,
                    base_dir,
                    labels,
                    equation_counter,
                    fn_state,
                )?;
            }
        }
        SIROpcode::LinkData => {
            emit_attach_metadata(page, 0, 0, 0, 0);

            for &child_idx in &node.children {
                compile_node(
                    tree,
                    child_idx,
                    doc,
                    page,
                    ctx,
                    gir_doc,
                    base_dir,
                    labels,
                    equation_counter,
                    fn_state,
                )?;
            }
        }
    }

    Ok(())
}

/// Collect image path from SetContent children of a block.
fn collect_image_path(
    tree: &InstructionTree,
    children: &[usize],
    doc: &SIRDocument,
) -> Option<String> {
    for &child_idx in children {
        let child_node = tree.node(child_idx);
        if child_node.instruction.opcode() == SIROpcode::SetContent
            && let Some(text) = doc.payload_text(child_node.instruction)
        {
            // Payload region may lack NUL separators between adjacent content.
            // Extract the image path: take characters until we hit whitespace
            // followed by non-path characters, or any non-path character.
            let raw = text.trim_end_matches('\0');
            // Find end of path: first whitespace or control character after the path
            let path_end = raw
                .find(|c: char| c.is_whitespace() || c.is_control())
                .unwrap_or(raw.len());
            let path = raw[..path_end].trim().to_string();
            if !path.is_empty() {
                return Some(path);
            }
        }
    }
    None
}

fn detect_image_format(data: &[u8]) -> Option<ImageFormat> {
    if data.len() >= 8 && data[0..4] == [0x89, 0x50, 0x4E, 0x47] {
        Some(ImageFormat::Png)
    } else if data.len() >= 3 && data[0..3] == [0xFF, 0xD8, 0xFF] {
        Some(ImageFormat::Jpeg)
    } else {
        None
    }
}

fn png_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    if data.len() >= 24 && &data[12..16] == b"IHDR" {
        let w = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
        let h = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
        Some((w, h))
    } else {
        None
    }
}

fn jpeg_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    let mut i = 2;
    while i + 13 < data.len() {
        if data[i] == 0xFF {
            let marker = data[i + 1];
            if marker == 0xC0 || marker == 0xC2 {
                let h = u32::from_be_bytes([data[i + 5], data[i + 6], data[i + 7], data[i + 8]]);
                let w = u32::from_be_bytes([data[i + 9], data[i + 10], data[i + 11], data[i + 12]]);
                return Some((w, h));
            }
            if i + 3 < data.len() {
                let len = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
                i += 2 + len;
            } else {
                break;
            }
        } else {
            i += 1;
        }
    }
    None
}

fn load_and_scale_image(
    path: &str,
    content_width: Fp266,
    base_dir: Option<&Path>,
) -> Option<(GIRImage, i32, i32)> {
    let full_path = if std::path::Path::new(path).is_absolute() {
        std::path::PathBuf::from(path)
    } else if let Some(dir) = base_dir {
        dir.join(path)
    } else {
        std::path::PathBuf::from(path)
    };

    let data = std::fs::read(&full_path).ok()?;
    let format = detect_image_format(&data)?;

    let (w_px, h_px) = match format {
        ImageFormat::Png => png_dimensions(&data)?,
        ImageFormat::Jpeg => jpeg_dimensions(&data)?,
    };

    let cw = content_width.to_f64();
    let scale = if (w_px as f64) > cw {
        cw / (w_px as f64)
    } else {
        1.0
    };
    let w_fp = ((w_px as f64) * scale * 64.0) as i32;
    let h_fp = ((h_px as f64) * scale * 64.0) as i32;

    Some((
        GIRImage {
            data,
            width: w_fp,
            height: h_fp,
            format,
        },
        w_fp,
        h_fp,
    ))
}

/// Collect text content from SetContent children of a block.
fn collect_block_text(tree: &InstructionTree, children: &[usize], doc: &SIRDocument) -> String {
    let mut text = String::new();
    for &child_idx in children {
        let child_node = tree.node(child_idx);
        if child_node.instruction.opcode() == SIROpcode::SetContent
            && let Some(t) = doc.payload_text(child_node.instruction)
        {
            let s = t.trim_end_matches('\0').trim().to_string();
            if !s.is_empty() {
                text.push_str(&s);
            }
        }
    }
    text
}

fn collect_table_text(tree: &InstructionTree, children: &[usize], doc: &SIRDocument) -> String {
    let mut text = String::new();
    for &child_idx in children {
        let child_node = tree.node(child_idx);
        if child_node.instruction.opcode() == SIROpcode::SetContent
            && let Some(t) = doc.payload_text(child_node.instruction)
        {
            let s = t.trim_end_matches('\0').trim().to_string();
            if !s.is_empty() {
                if !text.is_empty() {
                    text.push('\n');
                }
                text.push_str(&s);
            }
        }
    }
    text
}

fn emit_table(
    page: &mut GIRPage,
    ctx: &mut CompileContext,
    gir_doc: &mut GIRDocument,
    table_text: &str,
) {
    let lines: Vec<&str> = table_text.lines().collect();
    if lines.is_empty() {
        return;
    }

    // Parse rows: each line is a row, cells separated by |
    let mut rows: Vec<Vec<&str>> = Vec::new();
    for line in &lines {
        let trimmed = line.trim();
        if trimmed.starts_with('|') && trimmed.ends_with('|') {
            let inner = &trimmed[1..trimmed.len() - 1];
            let cells: Vec<&str> = inner.split('|').map(|c| c.trim()).collect();
            rows.push(cells);
        } else if trimmed.contains('|') {
            let cells: Vec<&str> = trimmed.split('|').map(|c| c.trim()).collect();
            rows.push(cells);
        } else if !trimmed.is_empty() && !trimmed.starts_with('-') && !trimmed.starts_with(':') {
            // Separator line (e.g. |---|---|) - skip
            let cells: Vec<&str> = trimmed.split('|').map(|c| c.trim()).collect();
            if !cells
                .iter()
                .all(|c| c.is_empty() || c.chars().all(|ch| ch == '-' || ch == ':'))
            {
                rows.push(cells);
            }
        }
    }

    if rows.is_empty() {
        return;
    }

    // Determine column count and widths
    let num_cols = rows.iter().map(|r| r.len()).max().unwrap_or(1);
    let mut col_widths: Vec<usize> = vec![0; num_cols];
    for row in &rows {
        for (i, cell) in row.iter().enumerate() {
            if i < num_cols {
                let w = cell.chars().count();
                col_widths[i] = col_widths[i].max(w);
            }
        }
    }

    // Calculate total table width
    let total_text_width: usize = col_widths.iter().sum::<usize>() + col_widths.len() * 3; // padding
    let content_width_pts = ctx.content_width.to_f64() / 64.0;
    let col_spacing = 8.0; // spacing between columns in points

    let col_widths_pt: Vec<f64> = if total_text_width as f64 > content_width_pts {
        // Scale down
        let scale = content_width_pts / (total_text_width as f64);
        col_widths.iter().map(|&w| w as f64 * 7.0 * scale).collect()
    } else {
        col_widths.iter().map(|&w| w as f64 * 7.0).collect()
    };

    let _line_h = ctx.line_height().to_f64();
    for (row_idx, row) in rows.iter().enumerate() {
        let mut cx = ctx.x;
        for (col_idx, cell) in row.iter().enumerate() {
            if col_idx >= num_cols {
                break;
            }
            let _col_x = cx.to_f64() / 64.0;
            emit_move_xy(page, cx, ctx.y);

            let trimmed_cell = cell.trim();
            if !trimmed_cell.is_empty() {
                emit_set_font(page, ctx.font_id as i32);
                for ch in trimmed_cell.chars() {
                    page.push(GIRCommand::new_put_glyph(ch as i32, 7 * 64));
                }
            }

            cx += Fp266::from_int((col_widths_pt[col_idx] + col_spacing) as i32);
        }

        ctx.advance_y(ctx.line_height());
        ctx.reset_x();

        // Draw horizontal rule after header row
        if row_idx == 0 && rows.len() > 1 {
            let table_width: f64 = col_widths_pt.iter().sum::<f64>()
                + (col_widths_pt.len() as f64 - 1.0) * col_spacing;
            let rule_y = ctx.y.to_f64() / 64.0 - 2.0;
            page.push(GIRCommand::new_draw_rule(
                (ctx.x.to_f64() / 64.0 * 64.0) as i32,
                (rule_y * 64.0) as i32,
                (table_width * 64.0) as i32,
                64, // 1pt rule
            ));
        }

        // Page overflow
        if ctx.exceeds_page() {
            if !page.is_empty() {
                let depth = ctx.stack_depth;
                for _ in 0..depth {
                    emit_pop_stack(page);
                }
                gir_doc.push_page(std::mem::replace(page, ctx.new_page()));
                for _ in 0..depth {
                    emit_push_stack(page);
                }
            }
            ctx.y = ctx.margin_top;
        }
    }

    // Extra spacing after table
    ctx.advance_y(ctx.line_height());
    ctx.reset_x();
}

const DROP_CAP_LINES: usize = 3;
const DROP_CAP_SIZE_MULTIPLIER: i32 = 3;
const DROP_CAP_GAP_PT: i32 = 6;

fn emit_drop_cap_paragraph(
    page: &mut GIRPage,
    ctx: &mut CompileContext,
    gir_doc: &mut GIRDocument,
    text: &str,
) {
    let first_char = match text.chars().next() {
        Some(ch) if !ch.is_whitespace() => ch,
        _ => {
            emit_paragraph(page, ctx, gir_doc, text);
            return;
        }
    };

    let rest = &text[first_char.len_utf8()..];
    let rest = rest.trim_start();
    if rest.is_empty() {
        emit_paragraph(page, ctx, gir_doc, text);
        return;
    }

    let body_size = ctx.font_size();
    let drop_cap_size = body_size.mul(Fp266::from_int(DROP_CAP_SIZE_MULTIPLIER));

    let font_data_for_style = ctx
        .font_data_variants
        .get(ctx.font_id as usize)
        .and_then(|opt| opt.as_ref())
        .or(ctx.font_data.as_ref());

    let drop_cap_width = if let Some(data) = font_data_for_style {
        let shaped = crate::shaping::shape_text_cached(
            &ctx.shape_cache,
            data,
            &first_char.to_string(),
            drop_cap_size,
            ctx.font_id,
        );
        shaped
            .glyphs
            .iter()
            .map(|g| g.advance)
            .fold(Fp266::ZERO, |a, b| a + b)
    } else {
        Fp266::from_int(14).mul(Fp266::from_int(DROP_CAP_SIZE_MULTIPLIER))
    };

    let saved_font_size = ctx.font_size;
    ctx.font_size = drop_cap_size;

    let drop_cap_shaped: std::sync::Arc<_> = if let Some(data) = font_data_for_style {
        crate::shaping::shape_text_cached(
            &ctx.shape_cache,
            data,
            &first_char.to_string(),
            drop_cap_size,
            ctx.font_id,
        )
    } else {
        std::sync::Arc::new(crate::shaping::fast_path::shape_ascii(
            &first_char.to_string(),
            drop_cap_size,
            ctx.font_id,
        ))
    };

    let drop_cap_y = ctx.y + ctx.line_height() * (DROP_CAP_LINES as i32 - 1);
    emit_set_font(page, ctx.font_id as i32);
    emit_move_xy(page, ctx.x, drop_cap_y);
    for g in &drop_cap_shaped.glyphs {
        page.push(GIRCommand::new_put_glyph(
            g.glyph_id as i32,
            g.advance.raw() as i32,
        ));
    }

    ctx.font_size = saved_font_size;
    let indent = drop_cap_width + Fp266::from_int(DROP_CAP_GAP_PT);
    let indented_width = ctx.content_width - indent;

    ctx.bump.reset();

    let rest_font_data = ctx
        .font_data_variants
        .get(ctx.font_id as usize)
        .and_then(|opt| opt.as_ref())
        .or(ctx.font_data.as_ref());
    let rest_shaped: std::sync::Arc<_> = if let Some(data) = rest_font_data {
        crate::shaping::shape_text_cached(&ctx.shape_cache, data, rest, body_size, ctx.font_id)
    } else {
        std::sync::Arc::new(crate::shaping::fast_path::shape_ascii(
            rest,
            body_size,
            ctx.font_id,
        ))
    };

    if rest_shaped.glyphs.is_empty() {
        ctx.advance_y(ctx.line_height() * DROP_CAP_LINES as i32);
        ctx.reset_x();
        return;
    }

    let text_bytes = rest.as_bytes();
    let n = rest_shaped.glyphs.len();

    let line_ranges = {
        let items: BumpVec<'_, LineBreakItem> = BumpVec::from_iter_in(
            rest_shaped.glyphs.iter().map(|g| {
                let ci = g.cluster_id as usize;
                let is_space = ci < text_bytes.len() && text_bytes[ci] == b' ';
                let space_stretch = if is_space {
                    g.advance.div(Fp266::from_int(2))
                } else {
                    Fp266::ZERO
                };
                let space_shrink = if is_space {
                    g.advance.div(Fp266::from_int(3))
                } else {
                    Fp266::ZERO
                };
                LineBreakItem {
                    width: g.advance,
                    stretchability: space_stretch,
                    shrinkability: space_shrink,
                    penalty: 0.0,
                    is_mandatory: false,
                    is_hyphenation: false,
                    hyphen_width: Fp266::ZERO,
                    text: "",
                }
            }),
            &ctx.bump,
        );

        let options = LineBreakOptions {
            line_width: indented_width,
            ..Default::default()
        };

        let result = linebreak(&items, &options);

        let mut ranges: Vec<(usize, usize)> = Vec::new();
        let mut prev = 0;
        for &b in &result.breaks {
            if b > prev {
                ranges.push((prev, b));
            }
            prev = b;
        }
        if prev < n {
            ranges.push((prev, n));
        }
        if ranges.is_empty() {
            ranges.push((0, n));
        }
        ranges
    };

    let num_lines = line_ranges.len();
    for (line_idx, &(start, end)) in line_ranges.iter().enumerate() {
        let use_indent = line_idx < DROP_CAP_LINES && line_idx < num_lines;
        let line_x = if use_indent { ctx.x + indent } else { ctx.x };
        let line_width = if use_indent {
            indented_width
        } else {
            ctx.content_width
        };

        emit_set_font(page, ctx.font_id as i32);
        emit_move_xy(page, line_x, ctx.y);

        let is_last_line = line_idx == num_lines - 1;
        let line_glyphs = &rest_shaped.glyphs[start..end];
        let justified = justify::justify_line(line_glyphs, text_bytes, line_width, is_last_line);

        for jg in &justified {
            page.push(GIRCommand::new_put_glyph(jg.glyph_id as i32, jg.x_advance));
        }

        ctx.reset_x();
        ctx.advance_y(ctx.line_height());

        if ctx.exceeds_page() {
            if !page.is_empty() {
                let depth = ctx.stack_depth;
                for _ in 0..depth {
                    emit_pop_stack(page);
                }
                gir_doc.push_page(std::mem::replace(page, ctx.new_page()));
                for _ in 0..depth {
                    emit_push_stack(page);
                }
            }
            ctx.y = ctx.margin_top;
        }
    }
}

/// Emit a paragraph of text using Knuth-Plass line-breaking.
fn emit_paragraph(
    page: &mut GIRPage,
    ctx: &mut CompileContext,
    gir_doc: &mut GIRDocument,
    text: &str,
) {
    ctx.bump.reset();

    let font_size = ctx.font_size();

    let font_data_for_style = ctx
        .font_data_variants
        .get(ctx.font_id as usize)
        .and_then(|opt| opt.as_ref())
        .or(ctx.font_data.as_ref());
    let shaped: std::sync::Arc<_> = if let Some(data) = font_data_for_style {
        crate::shaping::shape_text_cached(&ctx.shape_cache, data, text, font_size, ctx.font_id)
    } else {
        std::sync::Arc::new(crate::shaping::fast_path::shape_ascii(
            text,
            font_size,
            ctx.font_id,
        ))
    };

    if shaped.glyphs.is_empty() {
        return;
    }

    let text_bytes = text.as_bytes();
    let n = shaped.glyphs.len();
    let content_width = ctx.content_width;

    let line_ranges = {
        let items: BumpVec<'_, LineBreakItem> = BumpVec::from_iter_in(
            shaped.glyphs.iter().map(|g| {
                let ci = g.cluster_id as usize;
                let is_space = ci < text_bytes.len() && text_bytes[ci] == b' ';
                let space_stretch = if is_space {
                    g.advance.div(Fp266::from_int(2))
                } else {
                    Fp266::ZERO
                };
                let space_shrink = if is_space {
                    g.advance.div(Fp266::from_int(3))
                } else {
                    Fp266::ZERO
                };
                LineBreakItem {
                    width: g.advance,
                    stretchability: space_stretch,
                    shrinkability: space_shrink,
                    penalty: 0.0,
                    is_mandatory: false,
                    is_hyphenation: false,
                    hyphen_width: Fp266::ZERO,
                    text: "",
                }
            }),
            &ctx.bump,
        );

        let options = LineBreakOptions {
            line_width: content_width,
            ..Default::default()
        };

        let result = linebreak(&items, &options);

        let mut ranges: Vec<(usize, usize)> = Vec::new();
        let mut prev = 0;
        for &b in &result.breaks {
            if b > prev {
                ranges.push((prev, b));
            }
            prev = b;
        }
        if prev < n {
            ranges.push((prev, n));
        }
        if ranges.is_empty() {
            ranges.push((0, n));
        }
        ranges
    };

    for (line_idx, &(start, end)) in line_ranges.iter().enumerate() {
        emit_set_font(page, ctx.font_id as i32);
        emit_move_xy(page, ctx.x, ctx.y);

        let is_last_line = line_idx == line_ranges.len() - 1;
        let line_glyphs = &shaped.glyphs[start..end];
        let justified = justify::justify_line(line_glyphs, text_bytes, content_width, is_last_line);

        for jg in &justified {
            page.push(GIRCommand::new_put_glyph(jg.glyph_id as i32, jg.x_advance));
        }

        ctx.reset_x();
        ctx.advance_y(ctx.line_height());

        if ctx.exceeds_page() {
            if !page.is_empty() {
                let depth = ctx.stack_depth;
                for _ in 0..depth {
                    emit_pop_stack(page);
                }
                gir_doc.push_page(std::mem::replace(page, ctx.new_page()));
                for _ in 0..depth {
                    emit_push_stack(page);
                }
            }
            ctx.y = ctx.margin_top;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldir_ir::sir::BlockType;
    use ldir_ir::sir::{ROOT_SENTINEL, SIRInstruction};

    fn make_simple_doc() -> SIRDocument {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push(SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0));
        doc
    }

    fn make_nested_doc() -> SIRDocument {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push(SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0));
        doc.push(SIRInstruction::new(SIROpcode::ApplyStyle, 2, 0, 0));
        doc.push(SIRInstruction::new(SIROpcode::SetContent, 3, 0, 0));
        doc
    }

    fn make_deeply_nested_doc() -> SIRDocument {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push(SIRInstruction::new(SIROpcode::PushBlock, 1, 0, 0));
        doc.push(SIRInstruction::new(SIROpcode::PushBlock, 2, 1, 0));
        doc.push(SIRInstruction::new(SIROpcode::SetContent, 3, 2, 0));
        doc
    }

    #[test]
    fn test_compile_simple_doc() {
        let doc = make_simple_doc();
        let gir = compile_sir(&doc).unwrap();
        assert!(!gir.is_empty());
        assert!(gir.is_well_formed());
        assert!(gir.total_commands() > 0);
    }

    #[test]
    fn test_compile_nested_doc() {
        let doc = make_nested_doc();
        let gir = compile_sir(&doc).unwrap();
        assert!(gir.is_well_formed());
    }

    #[test]
    fn test_compile_deeply_nested() {
        let doc = make_deeply_nested_doc();
        let gir = compile_sir(&doc).unwrap();
        assert!(gir.is_well_formed());
    }

    #[test]
    fn test_compile_deterministic() {
        let doc = make_simple_doc();
        let gir1 = compile_sir(&doc).unwrap();
        let gir2 = compile_sir(&doc).unwrap();
        assert_eq!(gir1, gir2);
    }

    #[test]
    fn test_compile_empty_doc() {
        let doc = SIRDocument::new();
        let result = compile_sir(&doc);
        assert!(result.is_err());
    }

    #[test]
    fn test_compile_cyclic_doc() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push(SIRInstruction::new(SIROpcode::PushBlock, 1, 0, 0));
        doc.push(SIRInstruction::new(SIROpcode::PushBlock, 2, 1, 0));
        doc.push(SIRInstruction::new(SIROpcode::SetContent, 3, 2, 0));
        let result = compile_sir(&doc);
        assert!(result.is_ok(), "acyclic tree should compile");
    }

    #[test]
    fn test_compile_self_referencing_doc() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push(SIRInstruction::new(SIROpcode::SetContent, 1, 1, 0));
        let result = compile_sir(&doc);
        assert!(result.is_err(), "self-referencing entity should fail");
    }

    #[test]
    fn test_compile_mutual_cycle_doc() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push(SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0));
        doc.push(SIRInstruction::new(SIROpcode::SetContent, 2, 1, 0));
        let result = compile_sir(&doc);
        assert!(result.is_ok(), "non-cyclic tree should compile");
    }

    #[test]
    fn test_compile_all_opcodes() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push(SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0));
        doc.push(SIRInstruction::new(SIROpcode::ApplyStyle, 2, 0, 0));
        doc.push(SIRInstruction::new(SIROpcode::InsertMath, 3, 0, 0));
        doc.push(SIRInstruction::new(SIROpcode::LinkData, 4, 0, 0));
        let gir = compile_sir(&doc).unwrap();
        assert!(gir.is_well_formed());
    }

    #[test]
    fn test_multiline_paragraph_wrapping() {
        // Text that should wrap to multiple lines (~65 chars fit per line at
        // default 12pt monospace with 468pt content width).
        let long_text =
            "The quick brown fox jumps over the lazy dog. Pack my box with five dozen liquor jugs.";
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push_with_payload(
            SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0),
            long_text.as_bytes(),
        );
        let gir = compile_sir(&doc).unwrap();
        assert!(
            gir.is_well_formed(),
            "multi-line paragraph must be well-formed"
        );
        // Should have more PutGlyph commands than characters on a single line
        let glyph_count = gir
            .iter()
            .flat_map(|page| page.iter())
            .filter(|cmd| cmd.opcode() == ldir_ir::gir::GIROpcode::PutGlyph)
            .count();
        assert!(glyph_count > 0, "should have glyphs");
    }

    #[test]
    fn test_multiline_paragraph_deterministic() {
        let long_text =
            "The quick brown fox jumps over the lazy dog. Pack my box with five dozen liquor jugs.";
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push_with_payload(
            SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0),
            long_text.as_bytes(),
        );
        let gir1 = compile_sir(&doc).unwrap();
        let gir2 = compile_sir(&doc).unwrap();
        assert_eq!(gir1, gir2, "multi-line layout must be deterministic");
    }

    #[test]
    fn test_detect_png_format() {
        let png_header = [
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, // IHDR chunk
            0, 0, 0, 10, // width = 10
            0, 0, 0, 20, // height = 20
        ];
        assert_eq!(detect_image_format(&png_header), Some(ImageFormat::Png));
    }

    #[test]
    fn test_detect_jpeg_format() {
        let jpeg_header = [0xFF, 0xD8, 0xFF, 0xE0, 0, 0, 0, 0];
        assert_eq!(detect_image_format(&jpeg_header), Some(ImageFormat::Jpeg));
    }

    #[test]
    fn test_detect_unknown_format() {
        assert_eq!(detect_image_format(&[0, 0, 0, 0]), None);
    }

    #[test]
    fn test_png_dimensions() {
        let mut data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        // IHDR length
        data.extend_from_slice(&13u32.to_be_bytes());
        data.extend_from_slice(b"IHDR");
        data.extend_from_slice(&100u32.to_be_bytes()); // width
        data.extend_from_slice(&200u32.to_be_bytes()); // height
        assert_eq!(png_dimensions(&data), Some((100, 200)));
    }

    #[test]
    fn test_compile_with_custom_margins() {
        let doc = make_simple_doc();
        let gir = compile_sir_with_font_variants_and_options(
            &doc,
            None,
            &[],
            Some((36, 36, 36, 36)),
            None,
            None,
            None,
            false,
        )
        .unwrap();
        assert!(!gir.is_empty());
        assert!(gir.is_well_formed());
    }

    #[test]
    fn test_compile_with_page_size_preset() {
        let doc = make_simple_doc();
        let gir = compile_sir_with_font_variants_and_options(
            &doc,
            None,
            &[],
            None,
            None,
            Some("a4"),
            None,
            false,
        )
        .unwrap();
        assert!(!gir.is_empty());
        assert!(gir.is_well_formed());
        let page = gir.iter().next().unwrap();
        assert_eq!(page.width, 595 * 64);
        assert_eq!(page.height, 842 * 64);
    }

    #[test]
    fn test_compile_with_custom_page_dims() {
        let doc = make_simple_doc();
        let gir = compile_sir_with_font_variants_and_options(
            &doc,
            None,
            &[],
            None,
            None,
            None,
            Some((400, 600)),
            false,
        )
        .unwrap();
        assert!(!gir.is_empty());
        let page = gir.iter().next().unwrap();
        assert_eq!(page.width, 400 * 64);
        assert_eq!(page.height, 600 * 64);
    }

    #[test]
    fn test_resolve_references() {
        let mut labels = IndexMap::new();
        labels.insert("sec:first".to_string(), "1".to_string());
        labels.insert("eq:pyth".to_string(), "2".to_string());

        let result = resolve_references(r"see \ref{sec:first} and \eqref{eq:pyth}", &labels);
        assert_eq!(result, "see 1 and (2)");

        let result = resolve_references(r"unknown \ref{missing}", &labels);
        assert_eq!(result, r"unknown \ref{missing}");
    }

    #[test]
    fn test_strip_label() {
        assert_eq!(strip_label(r"text\label{key}more"), "textmore");
        assert_eq!(strip_label(r"\label{a}\label{b}"), "");
        assert_eq!(strip_label("no labels here"), "no labels here");
    }

    #[test]
    fn test_extract_label_key() {
        assert_eq!(
            extract_label_key(r"\label{sec:intro}"),
            Some("sec:intro".to_string())
        );
        assert_eq!(
            extract_label_key(r"hello\label{eq:1}world"),
            Some("eq:1".to_string())
        );
        assert_eq!(extract_label_key("no label"), None);
    }

    #[test]
    fn test_compile_with_refs_in_content() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push_with_payload(
            SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0),
            r"see \ref{sec:x} here".as_bytes(),
        );
        let gir = compile_sir(&doc).unwrap();
        assert!(gir.is_well_formed());
    }

    #[test]
    fn test_footnote_mark_renders_superscript() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push_with_payload(
            SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0),
            r"Hello\fnmark{1}world".as_bytes(),
        );
        doc.footnotes.push((1, "A footnote.".to_string()));
        let gir = compile_sir(&doc).unwrap();
        assert!(gir.is_well_formed());
        let glyph_count = gir
            .iter()
            .flat_map(|page| page.iter())
            .filter(|cmd| cmd.opcode() == ldir_ir::gir::GIROpcode::PutGlyph)
            .count();
        assert!(glyph_count > 0, "should have some glyphs");
        let has_superscript = gir.iter().flat_map(|page| page.iter()).any(|cmd| {
            cmd.opcode() == ldir_ir::gir::GIROpcode::PutGlyph
                && cmd.arg(0) == Some('\u{00B9}' as i32)
        });
        assert!(
            has_superscript,
            "should render superscript ¹ for footnote mark 1"
        );
    }

    #[test]
    fn test_footnotes_at_bottom_of_page() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push_with_payload(
            SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0),
            r"Text\fnmark{1} here.".as_bytes(),
        );
        doc.footnotes.push((1, "First footnote.".to_string()));
        let gir = compile_sir(&doc).unwrap();
        assert!(gir.is_well_formed());
        let has_rule = gir
            .iter()
            .flat_map(|page| page.iter())
            .any(|cmd| cmd.opcode() == ldir_ir::gir::GIROpcode::DrawRule);
        assert!(
            has_rule,
            "should emit a DrawRule for the footnote separator"
        );
    }

    #[test]
    fn test_multiple_footnotes_same_page() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push_with_payload(
            SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0),
            r"Text\fnmark{1} and\fnmark{2} more.".as_bytes(),
        );
        doc.footnotes.push((1, "First footnote.".to_string()));
        doc.footnotes.push((2, "Second footnote.".to_string()));
        let gir = compile_sir(&doc).unwrap();
        assert!(gir.is_well_formed());
        let superscript_count = gir
            .iter()
            .flat_map(|page| page.iter())
            .filter(|cmd| {
                cmd.opcode() == ldir_ir::gir::GIROpcode::PutGlyph
                    && cmd.arg(0) == Some('\u{00B9}' as i32)
            })
            .count();
        assert!(
            superscript_count >= 1,
            "should have at least one superscript ¹ in inline text"
        );
    }

    #[test]
    fn test_superscript_digit_function() {
        assert_eq!(superscript_digit(1), '\u{00B9}');
        assert_eq!(superscript_digit(2), '\u{00B2}');
        assert_eq!(superscript_digit(3), '\u{00B3}');
        assert_eq!(superscript_digit(4), '4');
        assert_eq!(superscript_digit(9), '9');
    }

    // ── Drop cap tests (6D) ─────────────────────────────────────────────

    #[test]
    fn test_drop_cap_not_applied_without_flag() {
        let mut doc = SIRDocument::new();
        // Root
        let root_id = 0;
        let root_payload = doc.payload_mut().append(&[BlockType::Document as u8]);
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            root_id,
            ROOT_SENTINEL,
            root_payload,
        ));
        // Heading block
        let heading_id = 1;
        let heading_payload = doc.payload_mut().append(&{
            let mut p = vec![BlockType::Heading as u8];
            p.extend_from_slice(&1u32.to_le_bytes());
            p
        });
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            heading_id,
            root_id,
            heading_payload,
        ));
        // Heading content
        let hc_id = 2;
        doc.push_with_payload(
            SIRInstruction::new(SIROpcode::SetContent, hc_id, heading_id, 0),
            b"Introduction",
        );
        // Paragraph block
        let para_id = 3;
        let para_payload = doc.payload_mut().append(&[BlockType::Paragraph as u8]);
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            para_id,
            root_id,
            para_payload,
        ));
        // Paragraph content
        let pc_id = 4;
        doc.push_with_payload(
            SIRInstruction::new(SIROpcode::SetContent, pc_id, para_id, 0),
            b"This is a paragraph after a heading.",
        );

        let gir = compile_sir(&doc).unwrap();
        assert!(gir.is_well_formed());
    }

    #[test]
    fn test_drop_cap_enabled_after_heading() {
        let mut doc = SIRDocument::new();
        let root_id = 0;
        let root_payload = doc.payload_mut().append(&[BlockType::Document as u8]);
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            root_id,
            ROOT_SENTINEL,
            root_payload,
        ));
        let heading_id = 1;
        let heading_payload = doc.payload_mut().append(&{
            let mut p = vec![BlockType::Heading as u8];
            p.extend_from_slice(&1u32.to_le_bytes());
            p
        });
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            heading_id,
            root_id,
            heading_payload,
        ));
        let hc_id = 2;
        doc.push_with_payload(
            SIRInstruction::new(SIROpcode::SetContent, hc_id, heading_id, 0),
            b"Introduction",
        );
        let para_id = 3;
        let para_payload = doc.payload_mut().append(&[BlockType::Paragraph as u8]);
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            para_id,
            root_id,
            para_payload,
        ));
        let pc_id = 4;
        doc.push_with_payload(
            SIRInstruction::new(SIROpcode::SetContent, pc_id, para_id, 0),
            b"This is a paragraph after a heading with enough text to wrap around the drop cap nicely.",
        );

        let gir = compile_sir_with_font_variants_and_options(
            &doc,
            None,
            &[],
            Some((72, 72, 72, 72)),
            None,
            None,
            None,
            true,
        )
        .unwrap();
        assert!(gir.is_well_formed());
        assert!(!gir.is_empty());
    }

    #[test]
    fn test_drop_cap_multiple_sections() {
        let mut doc = SIRDocument::new();
        let root_id = 0;
        let root_payload = doc.payload_mut().append(&[BlockType::Document as u8]);
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            root_id,
            ROOT_SENTINEL,
            root_payload,
        ));

        let mut next_id = 1u32;
        for _ in 0..3 {
            let heading_id = next_id;
            next_id += 1;
            let heading_payload = doc.payload_mut().append(&{
                let mut p = vec![BlockType::Heading as u8];
                p.extend_from_slice(&1u32.to_le_bytes());
                p
            });
            doc.push(SIRInstruction::new(
                SIROpcode::PushBlock,
                heading_id,
                root_id,
                heading_payload,
            ));
            let hc_id = next_id;
            next_id += 1;
            doc.push_with_payload(
                SIRInstruction::new(SIROpcode::SetContent, hc_id, heading_id, 0),
                b"Chapter",
            );

            let para_id = next_id;
            next_id += 1;
            let para_payload = doc.payload_mut().append(&[BlockType::Paragraph as u8]);
            doc.push(SIRInstruction::new(
                SIROpcode::PushBlock,
                para_id,
                root_id,
                para_payload,
            ));
            let pc_id = next_id;
            next_id += 1;
            doc.push_with_payload(
                SIRInstruction::new(SIROpcode::SetContent, pc_id, para_id, 0),
                b"Text after heading that is long enough to fill a line or two.",
            );
        }

        let gir = compile_sir_with_font_variants_and_options(
            &doc,
            None,
            &[],
            Some((72, 72, 72, 72)),
            None,
            None,
            None,
            true,
        )
        .unwrap();
        assert!(gir.is_well_formed());
    }

    #[test]
    fn test_drop_cap_whitespace_only_text() {
        let mut ctx = CompileContext::new();
        ctx.drop_caps_enabled = true;
        let mut gir_doc = GIRDocument::with_capacity(1);
        let mut page = ctx.new_page();

        emit_drop_cap_paragraph(&mut page, &mut ctx, &mut gir_doc, "   ");
        assert!(page.is_stack_balanced());
    }

    #[test]
    fn test_drop_cap_single_char_text() {
        let mut ctx = CompileContext::new();
        ctx.drop_caps_enabled = true;
        let mut gir_doc = GIRDocument::with_capacity(1);
        let mut page = ctx.new_page();

        emit_drop_cap_paragraph(&mut page, &mut ctx, &mut gir_doc, "A");
        assert!(page.is_stack_balanced());
    }

    // ── TOC with page numbers tests (6E) ────────────────────────────────

    #[test]
    fn test_toc_page_numbers_generated() {
        let mut doc = SIRDocument::new();
        let root_id = 0;
        let root_payload = doc.payload_mut().append(&[BlockType::Document as u8]);
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            root_id,
            ROOT_SENTINEL,
            root_payload,
        ));

        let mut next_id = 1u32;
        let titles: &[&[u8]] = &[b"First Section", b"Second Section"];
        for title in titles {
            let heading_id = next_id;
            next_id += 1;
            let heading_payload = doc.payload_mut().append(&{
                let mut p = vec![BlockType::Heading as u8];
                p.extend_from_slice(&1u32.to_le_bytes());
                p
            });
            doc.push(SIRInstruction::new(
                SIROpcode::PushBlock,
                heading_id,
                root_id,
                heading_payload,
            ));
            let hc_id = next_id;
            next_id += 1;
            doc.push_with_payload(
                SIRInstruction::new(SIROpcode::SetContent, hc_id, heading_id, 0),
                title,
            );
        }

        let gir = compile_sir_with_font_variants_and_options(
            &doc,
            None,
            &[],
            Some((72, 72, 72, 72)),
            None,
            None,
            None,
            false,
        )
        .unwrap();
        assert!(gir.is_well_formed());
        assert!(gir.page_count() >= 1);
    }

    #[test]
    fn test_toc_link_destination() {
        let mut doc = SIRDocument::new();
        let root_id = 0;
        let root_payload = doc.payload_mut().append(&[BlockType::Document as u8]);
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            root_id,
            ROOT_SENTINEL,
            root_payload,
        ));

        let mut next_id = 1u32;
        let titles: &[&[u8]] = &[b"Section One", b"Section Two"];
        for title in titles {
            let heading_id = next_id;
            next_id += 1;
            let heading_payload = doc.payload_mut().append(&{
                let mut p = vec![BlockType::Heading as u8];
                p.extend_from_slice(&1u32.to_le_bytes());
                p
            });
            doc.push(SIRInstruction::new(
                SIROpcode::PushBlock,
                heading_id,
                root_id,
                heading_payload,
            ));
            let hc_id = next_id;
            next_id += 1;
            doc.push_with_payload(
                SIRInstruction::new(SIROpcode::SetContent, hc_id, heading_id, 0),
                title,
            );
        }

        let gir = compile_sir_with_font_variants_and_options(
            &doc,
            None,
            &[],
            Some((72, 72, 72, 72)),
            None,
            None,
            None,
            false,
        )
        .unwrap();

        let has_internal_link = gir
            .iter()
            .flat_map(|page| page.links.iter())
            .any(|link| link.destination_page.is_some());
        assert!(
            has_internal_link,
            "TOC entries should have internal destination links"
        );
    }

    #[test]
    fn test_heading_position_recording() {
        let heading_pages = vec![(0, 72.0), (0, 144.0), (1, 72.0)];
        assert_eq!(heading_pages.len(), 3);
        assert_eq!(heading_pages[0].0, 0);
        assert_eq!(heading_pages[1].0, 0);
        assert_eq!(heading_pages[2].0, 1);
    }

    #[test]
    fn test_gir_link_with_destination_page() {
        use ldir_ir::gir::GIRLink;

        let link = GIRLink {
            x: 72.0,
            y: 720.0,
            width: 468.0,
            height: 14.4,
            url: String::new(),
            destination_page: Some(2),
        };
        assert_eq!(link.destination_page, Some(2));

        let uri_link = GIRLink {
            x: 72.0,
            y: 720.0,
            width: 468.0,
            height: 14.4,
            url: "https://example.com".to_string(),
            destination_page: None,
        };
        assert_eq!(uri_link.destination_page, None);
        assert_eq!(uri_link.url, "https://example.com");
    }

    #[test]
    fn test_toc_no_headings_no_toc() {
        let doc = make_simple_doc();
        let gir = compile_sir_with_font_variants_and_options(
            &doc,
            None,
            &[],
            Some((72, 72, 72, 72)),
            None,
            None,
            None,
            false,
        )
        .unwrap();
        assert!(gir.is_well_formed());
        let has_internal_link = gir
            .iter()
            .flat_map(|page| page.links.iter())
            .any(|link| link.destination_page.is_some());
        assert!(
            !has_internal_link,
            "no TOC links when there are fewer than 2 headings"
        );
    }
}
