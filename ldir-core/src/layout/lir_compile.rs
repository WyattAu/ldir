//! S-IR v2 -> L-IR layout compiler.
//!
//! Takes a [`SIRModuleV2`](ldir_ir::sir::v2::SIRModuleV2) and a
//! [`CompileContext`](crate::compiler::context::CompileContext) and produces
//! a positioned [`LIRDocument`](ldir_ir::lir::LIRDocument) tree with resolved
//! geometry, line-broken paragraphs, and page breaks.

#![allow(dead_code)]

use indexmap::IndexMap;
use std::collections::HashMap;

use ldir_ir::fp266::Fp266 as LirFp;
use ldir_ir::lir::style::{
    FlowDirection, LIRStyleTable, LIRTextStyle, ListType, MathType, Padding,
};
use ldir_ir::lir::types::*;
use ldir_ir::sir::v2::SIRModuleV2;
use ldir_ir::sir::v2::nodes::NodeType;

use crate::compiler::bibtex::{BibEntry, format_citation_apa, format_citation_ieee};
use crate::compiler::context::CompileContext;
use crate::compiler::context::{
    FONT_ID_BOLD, FONT_ID_BOLD_ITALIC, FONT_ID_ITALIC, FONT_ID_MONO, FONT_ID_REGULAR,
};
use crate::fp266::Fp266;
use crate::solver::{Expression, Relation, Solver, Strength};
use bumpalo::collections::Vec as BumpVec;

use crate::layout::linebreak::cjk::{insert_cjk_breaks, is_cjk_text};
use crate::layout::linebreak::{LineBreakItem, LineBreakOptions, linebreak};

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum LirError {
    NodeNotFound(u32),
    ContextError(String),
}

impl std::fmt::Display for LirError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NodeNotFound(id) => write!(f, "node {id} not found"),
            Self::ContextError(msg) => write!(f, "context error: {msg}"),
        }
    }
}

impl std::error::Error for LirError {}

type LirResult<T> = std::result::Result<T, LirError>;

// ---------------------------------------------------------------------------
// Fp266 conversion
// ---------------------------------------------------------------------------

#[inline]
fn fp_core_to_lir(v: Fp266) -> LirFp {
    LirFp::from_raw(v.raw())
}

#[inline]
fn fp_lir_to_core(v: LirFp) -> Fp266 {
    Fp266::from_raw(v.raw())
}

// ---------------------------------------------------------------------------
// Extracted node info (owned copy so we don't hold borrows on self)
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct NodeInfo {
    node_id: u32,
    node_type: NodeType,
    child_ids: Vec<u32>,
}

impl NodeInfo {
    fn from_tree(tree: &ldir_ir::sir::v2::nodes::NodeTree, id: u32) -> LirResult<Self> {
        let node = tree.get(id).ok_or(LirError::NodeNotFound(id))?;
        Ok(Self {
            node_id: id,
            node_type: node.node_type.clone(),
            child_ids: node.child_ids.clone(),
        })
    }
}

// ---------------------------------------------------------------------------
// Compiler state
// ---------------------------------------------------------------------------

struct LirCompiler<'a> {
    module: &'a SIRModuleV2,
    ctx: &'a CompileContext,
    next_id: u32,
    pages: Vec<LIRPage>,
    current_page_children: Vec<LIRNode>,
    cursor_x: Fp266,
    cursor_y: Fp266,
    section_number: Vec<u32>,
    section_counters: IndexMap<u8, u32>,
    heading_entries: Vec<(u8, String, String)>,
    pending_footnotes: Vec<(u32, String)>,
    footnote_counter: u32,
    figure_counter: u32,
    /// Figures/Tables deferred from inline flow for float placement.
    deferred_floats: Vec<LIRFigure>,
    eq_counter: u32,
    style_table: LIRStyleTable,
    bibliography: Option<&'a HashMap<String, BibEntry>>,
    cite_counter: u32,
    cite_numbers: IndexMap<String, u32>,
    bib_style: &'static str,
}

impl<'a> LirCompiler<'a> {
    fn new(module: &'a SIRModuleV2, ctx: &'a CompileContext) -> Self {
        Self::with_bib(module, ctx, None, "ieee")
    }

    fn with_bib(
        module: &'a SIRModuleV2,
        ctx: &'a CompileContext,
        bibliography: Option<&'a HashMap<String, BibEntry>>,
        bib_style: &'static str,
    ) -> Self {
        let mut style_table = LIRStyleTable::with_capacity(8);
        let body_font_size = fp_core_to_lir(ctx.font_size);
        style_table.insert(LIRTextStyle::new(0, FONT_ID_REGULAR, body_font_size));
        style_table.insert(LIRTextStyle::new(1, FONT_ID_BOLD, body_font_size));
        style_table.insert(LIRTextStyle::new(2, FONT_ID_ITALIC, body_font_size));
        style_table.insert(LIRTextStyle::new(3, FONT_ID_BOLD_ITALIC, body_font_size));
        style_table.insert(LIRTextStyle::new(4, FONT_ID_MONO, body_font_size));

        Self {
            module,
            ctx,
            next_id: 1,
            pages: Vec::new(),
            current_page_children: Vec::new(),
            cursor_x: ctx.margin_left,
            cursor_y: ctx.margin_top,
            section_number: Vec::new(),
            section_counters: IndexMap::new(),
            heading_entries: Vec::new(),
            pending_footnotes: Vec::new(),
            footnote_counter: 0,
            figure_counter: 0,
            deferred_floats: Vec::new(),
            eq_counter: 0,
            style_table,
            bibliography,
            cite_counter: 0,
            cite_numbers: IndexMap::new(),
            bib_style,
        }
    }

    fn alloc_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn content_width(&self) -> Fp266 {
        self.ctx.content_width
    }

    fn line_height(&self, font_size_pt: i32) -> Fp266 {
        let fs = Fp266::from_int(font_size_pt);
        fs + fs.mul(Fp266::from_frac(6, 10))
    }

    fn max_y(&self) -> Fp266 {
        self.ctx.page_height - self.ctx.margin_bottom
    }

    fn finish_page(&mut self) {
        let meta = LIRDocumentMeta {
            page_width: fp_core_to_lir(self.ctx.page_width),
            page_height: fp_core_to_lir(self.ctx.page_height),
            margin_top: fp_core_to_lir(self.ctx.margin_top),
            margin_bottom: fp_core_to_lir(self.ctx.margin_bottom),
            margin_left: fp_core_to_lir(self.ctx.margin_left),
            margin_right: fp_core_to_lir(self.ctx.margin_right),
            language: self.module.metadata.language.clone(),
        };
        let page_num = (self.pages.len() + 1) as u32;
        let mut page = LIRPage::new(page_num, &meta);
        page.id = self.alloc_id();
        page.children = std::mem::take(&mut self.current_page_children);
        self.pages.push(page);
    }

    fn ensure_space(&mut self, height: Fp266) -> bool {
        if self.cursor_y + height > self.max_y() && !self.current_page_children.is_empty() {
            self.finish_page();
            self.cursor_x = self.ctx.margin_left;
            self.cursor_y = self.ctx.margin_top;
            return true;
        }
        false
    }

    fn add_block(&mut self, node: LIRNode, height: Fp266) {
        self.ensure_space(height);
        self.current_page_children.push(node);
    }

    fn collect_text(&self, node_id: u32) -> String {
        self.module.body.collect_text(node_id)
    }

    fn section_number_string(&self) -> String {
        let parts: Vec<String> = self.section_number.iter().map(|n| n.to_string()).collect();
        parts.join(".")
    }

    fn increment_section_counter(&mut self, level: u8) {
        if self.section_number.len() < level as usize {
            self.section_number.resize(level as usize, 0);
        }
        self.section_number[level as usize - 1] += 1;
        for i in level as usize..self.section_number.len() {
            self.section_number[i] = 0;
        }
        *self.section_counters.entry(level).or_insert(0) += 1;
    }

    fn font_size_for_heading(&self, level: u8) -> i32 {
        match level {
            0 => 26,
            1 => 24,
            2 => 20,
            3 => 16,
            _ => 14,
        }
    }

    fn build_heading(
        &mut self,
        level: u8,
        number: String,
        label: String,
        source_id: Option<u32>,
    ) -> LirResult<Vec<LIRNode>> {
        let font_size_pt = self.font_size_for_heading(level);
        let lh = self.line_height(font_size_pt);

        self.ensure_space(lh + Fp266::from_int(6));

        let heading_text = if number.is_empty() {
            label.clone()
        } else {
            format!("{} {}", number, label)
        };
        let para = self.build_paragraph(
            &heading_text,
            source_id,
            Some(1),
            font_size_pt,
            self.content_width(),
            self.cursor_x,
        );

        let mut heading = LIRHeading::new(level);
        heading.id = self.alloc_id();
        heading.source_node_id = source_id;
        heading.number = number;
        heading.label = label;

        if let Some(p) = para {
            heading.children = p.children;
            heading.geometry = p.geometry;
            let height = fp_lir_to_core(heading.geometry.height);
            let spacing = if level <= 1 {
                Fp266::from_int(12)
            } else {
                Fp266::from_int(8)
            };
            self.cursor_y += height + spacing;
            self.add_block(LIRNode::Heading(heading), height + spacing);
        }

        Ok(Vec::new())
    }

    fn build_paragraph(
        &mut self,
        text: &str,
        source_node_id: Option<u32>,
        style_id: Option<u32>,
        font_size_pt: i32,
        available_width: Fp266,
        x_offset: Fp266,
    ) -> Option<LIRParagraph> {
        let text = text.trim();
        if text.is_empty() {
            return None;
        }

        let font_data = self
            .ctx
            .font_data
            .as_ref()
            .map(|arc| arc.as_slice())
            .unwrap_or(&[]);

        let font_size = Fp266::from_int(font_size_pt);
        let shaped = if font_data.is_empty() {
            crate::shaping::fast_path::shape_ascii(text, font_size, 0)
        } else {
            crate::shaping::shape_text(font_data, text, font_size)
        };

        if shaped.glyphs.is_empty() {
            return None;
        }

        let lh = self.line_height(font_size_pt);
        let n = shaped.glyphs.len();

        let items: BumpVec<'_, LineBreakItem> = BumpVec::from_iter_in(
            shaped.glyphs.iter().map(|g| {
                let is_space = g.glyph_id == 0 || (g.advance == Fp266::from_int(4));
                let stretch = if is_space {
                    g.advance.div(Fp266::from_int(2))
                } else {
                    Fp266::ZERO
                };
                let shrink = if is_space {
                    g.advance.div(Fp266::from_int(3))
                } else {
                    Fp266::ZERO
                };
                LineBreakItem {
                    width: g.advance,
                    stretchability: stretch,
                    shrinkability: shrink,
                    penalty: 0.0,
                    is_mandatory: false,
                    is_hyphenation: false,
                    hyphen_width: Fp266::ZERO,
                    text: "",
                }
            }),
            &self.ctx.bump,
        );

        let options = LineBreakOptions {
            line_width: available_width,
            max_adjustment_ratio: 2.0,
            ..Default::default()
        };

        let result = if is_cjk_text(text) {
            let cjk_items = insert_cjk_breaks(text, &items, &self.ctx.bump);
            linebreak(&cjk_items, &options, &self.ctx.bump)
        } else {
            linebreak(&items, &options, &self.ctx.bump)
        };

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

        let mut para = LIRParagraph::new();
        para.id = self.alloc_id();
        para.source_node_id = source_node_id;
        para.style_id = style_id;
        para.paragraph_spacing_after = fp_core_to_lir(Fp266::from_int(6));

        let mut abs_y = self.cursor_y;

        for (line_idx, &(start, end)) in ranges.iter().enumerate() {
            let mut line = LIRLine::new(line_idx as u32);
            line.id = self.alloc_id();

            let mut line_x = x_offset;
            let max_ascender = font_size;

            for i in start..end {
                let g = &shaped.glyphs[i];
                let baseline = font_size.mul(Fp266::from_frac(8, 10));

                let mut glyph = LIRGlyph::new(g.glyph_id, 0, fp_core_to_lir(g.advance));
                glyph.id = self.alloc_id();
                glyph.geometry = LIRGeometry::with_baseline(
                    fp_core_to_lir(line_x),
                    fp_core_to_lir(abs_y),
                    fp_core_to_lir(g.advance),
                    fp_core_to_lir(font_size),
                    fp_core_to_lir(baseline),
                );
                glyph.source_node_id = source_node_id;
                line.children.push(LIRNode::Glyph(glyph));

                line_x += g.advance;
            }

            line.geometry = LIRGeometry::with_baseline(
                fp_core_to_lir(x_offset),
                fp_core_to_lir(abs_y),
                fp_core_to_lir(line_x - x_offset),
                fp_core_to_lir(lh),
                fp_core_to_lir(max_ascender),
            );

            para.children.push(LIRNode::Line(line));
            abs_y += lh;
        }

        let total_height = lh * ranges.len() as i32;
        para.geometry = LIRGeometry::new(
            fp_core_to_lir(x_offset),
            fp_core_to_lir(self.cursor_y),
            fp_core_to_lir(available_width),
            fp_core_to_lir(total_height),
        );

        Some(para)
    }

    fn compile_children(&mut self, node_id: u32) -> LirResult<Vec<LIRNode>> {
        let info = NodeInfo::from_tree(&self.module.body, node_id)?;
        let mut children = Vec::new();
        for child_id in info.child_ids {
            let child_nodes = self.compile_node(child_id)?;
            children.extend(child_nodes);
        }
        Ok(children)
    }

    fn compile_node(&mut self, node_id: u32) -> LirResult<Vec<LIRNode>> {
        let info = NodeInfo::from_tree(&self.module.body, node_id)?;
        let source_id = Some(info.node_id);
        let child_ids = info.child_ids.clone();
        let nt = info.node_type;

        match nt {
            NodeType::Document => {
                let mut children = Vec::new();
                for child_id in child_ids {
                    children.extend(self.compile_node(child_id)?);
                }
                Ok(children)
            }

            NodeType::Chapter => {
                self.increment_section_counter(1);
                let num = self.section_number_string();
                let label = self.collect_text(node_id);
                self.heading_entries.push((1, num.clone(), label.clone()));
                self.build_heading(1, num, label, source_id)
            }

            NodeType::Section => {
                self.increment_section_counter(2);
                let num = self.section_number_string();
                let label = self.collect_text(node_id);
                self.heading_entries.push((2, num.clone(), label.clone()));
                self.build_heading(2, num, label, source_id)
            }

            NodeType::Subsection => {
                self.increment_section_counter(3);
                let num = self.section_number_string();
                let label = self.collect_text(node_id);
                self.heading_entries.push((3, num.clone(), label.clone()));
                self.build_heading(3, num, label, source_id)
            }

            NodeType::Subsubsection => {
                self.increment_section_counter(4);
                let num = self.section_number_string();
                let label = self.collect_text(node_id);
                self.heading_entries.push((4, num.clone(), label.clone()));
                self.build_heading(4, num, label, source_id)
            }

            NodeType::Part => {
                let label = self.collect_text(node_id);
                self.build_heading(0, String::new(), label, source_id)
            }

            NodeType::Paragraph => {
                let text = self.collect_text(node_id);
                if text.trim().is_empty() {
                    return Ok(Vec::new());
                }
                let lh = self.line_height(12);
                self.ensure_space(lh);

                if let Some(p) = self.build_paragraph(
                    &text,
                    source_id,
                    Some(0),
                    12,
                    self.content_width(),
                    self.cursor_x,
                ) {
                    let height = fp_lir_to_core(p.geometry.height);
                    self.cursor_y += height + Fp266::from_int(6);
                    self.add_block(LIRNode::Paragraph(p), height + Fp266::from_int(6));
                }
                Ok(Vec::new())
            }

            NodeType::Text { ref content } => {
                let text = content.trim();
                if text.is_empty() {
                    return Ok(Vec::new());
                }
                let lh = self.line_height(12);
                self.ensure_space(lh);

                if let Some(p) = self.build_paragraph(
                    text,
                    source_id,
                    Some(0),
                    12,
                    self.content_width(),
                    self.cursor_x,
                ) {
                    let height = fp_lir_to_core(p.geometry.height);
                    self.cursor_y += height + Fp266::from_int(6);
                    self.add_block(LIRNode::Paragraph(p), height + Fp266::from_int(6));
                }
                Ok(Vec::new())
            }

            NodeType::Bold
            | NodeType::Italic
            | NodeType::Mono
            | NodeType::Underline
            | NodeType::Strikethrough
            | NodeType::SmallCaps
            | NodeType::Link { .. }
            | NodeType::Group
            | NodeType::Styled { .. } => self.compile_children(node_id),

            NodeType::List { ordered, start, .. } => {
                let lir_list_type = if ordered {
                    ListType::Ordered
                } else {
                    ListType::Unordered
                };

                let mut list = LIRList::new(lir_list_type);
                list.id = self.alloc_id();
                list.source_node_id = source_id;
                if let Some(s) = start {
                    list.start = s;
                }

                let saved_x = self.cursor_x;
                self.cursor_x += Fp266::from_int(36);

                for child_id in &child_ids {
                    let child_info = NodeInfo::from_tree(&self.module.body, *child_id)?;
                    if matches!(child_info.node_type, NodeType::ListItem) {
                        let item_idx = list.children.len() as u32 + list.start;
                        let marker = if ordered {
                            format!("{}.", item_idx)
                        } else {
                            "\u{2022}".to_string()
                        };

                        let mut item = LIRListItem::new();
                        item.id = self.alloc_id();
                        item.source_node_id = Some(*child_id);
                        item.marker = Some(marker);

                        let text = self.collect_text(*child_id);
                        let lh = self.line_height(12);

                        if !text.trim().is_empty() {
                            self.ensure_space(lh);
                            if let Some(p) = self.build_paragraph(
                                &text,
                                Some(*child_id),
                                Some(0),
                                12,
                                self.content_width() - Fp266::from_int(36),
                                self.cursor_x,
                            ) {
                                let height = fp_lir_to_core(p.geometry.height);
                                self.cursor_y += height + Fp266::from_int(4);
                                item.children.push(LIRNode::Paragraph(p));
                            }
                        }

                        list.children.push(LIRNode::ListItem(item));
                    }
                }

                self.cursor_x = saved_x;

                let list_height = Fp266::from_int(12);
                list.geometry = LIRGeometry::new(
                    fp_core_to_lir(self.cursor_x),
                    fp_core_to_lir(self.cursor_y),
                    fp_core_to_lir(self.content_width()),
                    fp_core_to_lir(list_height),
                );

                self.add_block(LIRNode::List(list), list_height);
                Ok(Vec::new())
            }

            NodeType::ListItem => self.compile_children(node_id),

            NodeType::BlockQuote => {
                let mut bq = LIRBlockQuote::new();
                bq.id = self.alloc_id();
                bq.source_node_id = source_id;

                let saved_x = self.cursor_x;
                self.cursor_x += Fp266::from_int(36);

                let child_nodes = self.compile_children(node_id)?;
                bq.children = child_nodes;

                self.cursor_x = saved_x;

                let bq_height = Fp266::from_int(12);
                bq.geometry = LIRGeometry::new(
                    fp_core_to_lir(saved_x),
                    fp_core_to_lir(self.cursor_y),
                    fp_core_to_lir(self.content_width() - Fp266::from_int(36)),
                    fp_core_to_lir(bq_height),
                );

                self.add_block(LIRNode::BlockQuote(bq), bq_height);
                Ok(Vec::new())
            }

            NodeType::CodeBlock { ref language, .. } => {
                let text = self.collect_text(node_id);
                let lang = language.clone().unwrap_or_default();

                let mut cb = LIRCodeBlock::new(&lang);
                cb.id = self.alloc_id();
                cb.source_node_id = source_id;

                let font_size = 10;
                let lh = self.line_height(font_size);
                let lines: Vec<&str> = text.lines().collect();

                for (line_idx, line_text) in lines.iter().enumerate() {
                    self.ensure_space(lh);

                    let mut line = LIRLine::new(line_idx as u32);
                    line.id = self.alloc_id();

                    let font_data = self
                        .ctx
                        .font_data_variants
                        .get(FONT_ID_MONO as usize)
                        .and_then(|opt| opt.as_ref())
                        .map(|arc| arc.as_slice())
                        .unwrap_or(&[]);

                    let fs = Fp266::from_int(font_size);
                    let shaped = if font_data.is_empty() {
                        crate::shaping::fast_path::shape_ascii(line_text, fs, FONT_ID_MONO)
                    } else {
                        crate::shaping::shape_text(font_data, line_text, fs)
                    };

                    let mut lx = self.cursor_x;
                    for g in &shaped.glyphs {
                        let mut glyph =
                            LIRGlyph::new(g.glyph_id, FONT_ID_MONO, fp_core_to_lir(g.advance));
                        glyph.id = self.alloc_id();
                        let baseline = fs.mul(Fp266::from_frac(8, 10));
                        glyph.geometry = LIRGeometry::with_baseline(
                            fp_core_to_lir(lx),
                            fp_core_to_lir(self.cursor_y),
                            fp_core_to_lir(g.advance),
                            fp_core_to_lir(fs),
                            fp_core_to_lir(baseline),
                        );
                        line.children.push(LIRNode::Glyph(glyph));
                        lx += g.advance;
                    }

                    line.geometry = LIRGeometry::with_baseline(
                        fp_core_to_lir(self.cursor_x),
                        fp_core_to_lir(self.cursor_y),
                        fp_core_to_lir(lx - self.cursor_x),
                        fp_core_to_lir(lh),
                        fp_core_to_lir(fs.mul(Fp266::from_frac(8, 10))),
                    );

                    cb.children.push(LIRNode::Line(line));
                    self.cursor_y += lh;
                }

                let total_height = if lines.is_empty() {
                    lh
                } else {
                    lh * lines.len() as i32
                };
                cb.geometry = LIRGeometry::new(
                    fp_core_to_lir(self.cursor_x),
                    fp_core_to_lir(self.cursor_y - total_height),
                    fp_core_to_lir(self.content_width()),
                    fp_core_to_lir(total_height),
                );

                self.add_block(LIRNode::CodeBlock(cb), total_height + Fp266::from_int(6));
                Ok(Vec::new())
            }

            NodeType::MathBlock { numbered, .. } => {
                self.eq_counter += 1;
                let eq_num = if numbered {
                    Some(self.eq_counter)
                } else {
                    None
                };

                let text = self.collect_text(node_id);
                let lh = self.line_height(12);
                self.ensure_space(lh);

                let mut mb = LIRMathBlock::new(MathType::Display);
                mb.id = self.alloc_id();
                mb.source_node_id = source_id;
                mb.number = eq_num;

                let font_size = Fp266::from_int(12);
                let font_data = self
                    .ctx
                    .font_data
                    .as_ref()
                    .map(|arc| arc.as_slice())
                    .unwrap_or(&[]);

                let shaped = if font_data.is_empty() {
                    crate::shaping::fast_path::shape_ascii(&text, font_size, 0)
                } else {
                    crate::shaping::shape_text(font_data, &text, font_size)
                };

                let mut lx = self.cursor_x;
                for g in &shaped.glyphs {
                    let mut glyph = LIRGlyph::new(g.glyph_id, 0, fp_core_to_lir(g.advance));
                    glyph.id = self.alloc_id();
                    glyph.geometry = LIRGeometry::with_baseline(
                        fp_core_to_lir(lx),
                        fp_core_to_lir(self.cursor_y),
                        fp_core_to_lir(g.advance),
                        fp_core_to_lir(font_size),
                        fp_core_to_lir(font_size.mul(Fp266::from_frac(8, 10))),
                    );
                    mb.children.push(LIRNode::Glyph(glyph));
                    lx += g.advance;
                }

                mb.geometry = LIRGeometry::with_baseline(
                    fp_core_to_lir(self.cursor_x),
                    fp_core_to_lir(self.cursor_y),
                    fp_core_to_lir(lx - self.cursor_x),
                    fp_core_to_lir(lh),
                    fp_core_to_lir(font_size.mul(Fp266::from_frac(8, 10))),
                );

                self.cursor_y += lh;
                self.add_block(LIRNode::MathBlock(mb), lh + Fp266::from_int(6));
                Ok(Vec::new())
            }

            NodeType::Table { num_cols, .. } => {
                let nc = num_cols as u16;
                let mut table = LIRTable::new(nc);
                table.id = self.alloc_id();
                table.source_node_id = source_id;
                table.border = true;

                let col_w = self.content_width().div(Fp266::from_int(num_cols as i32));
                table.col_widths = vec![fp_core_to_lir(col_w); nc as usize];

                let mut row_idx = 0u16;

                for child_id in &child_ids {
                    let child_info = NodeInfo::from_tree(&self.module.body, *child_id)?;
                    if let NodeType::TableRow { is_header } = &child_info.node_type {
                        let lh = self.line_height(12);
                        self.ensure_space(lh + Fp266::from_int(4));

                        let mut row = LIRTableRow::new(*is_header);
                        row.id = self.alloc_id();
                        row.source_node_id = Some(*child_id);

                        let mut col_idx = 0u16;
                        for cell_id in &child_info.child_ids {
                            let cell_info = NodeInfo::from_tree(&self.module.body, *cell_id)?;
                            if let NodeType::TableCell { colspan, rowspan } = &cell_info.node_type {
                                let cell_w = col_w * (*colspan as i32);
                                let cell_text = self.collect_text(*cell_id);

                                let mut cell = LIRTableCell::new(col_idx);
                                cell.id = self.alloc_id();
                                cell.source_node_id = Some(*cell_id);
                                cell.colspan = *colspan as u16;
                                cell.rowspan = *rowspan as u16;
                                cell.padding = Padding::uniform(fp_core_to_lir(Fp266::from_int(4)));

                                if !cell_text.trim().is_empty()
                                    && let Some(p) = self.build_paragraph(
                                        &cell_text,
                                        Some(*cell_id),
                                        Some(0),
                                        11,
                                        cell_w - Fp266::from_int(8),
                                        self.cursor_x + Fp266::from_int(4),
                                    )
                                {
                                    cell.children.push(LIRNode::Paragraph(p));
                                }

                                row.children.push(LIRNode::TableCell(cell));
                                col_idx += *colspan as u16;
                            }
                        }

                        let row_height = self.line_height(12) + Fp266::from_int(4);
                        row.geometry = LIRGeometry::new(
                            fp_core_to_lir(self.cursor_x),
                            fp_core_to_lir(self.cursor_y),
                            fp_core_to_lir(self.content_width()),
                            fp_core_to_lir(row_height),
                        );

                        self.cursor_y += row_height;
                        table.children.push(LIRNode::TableRow(row));
                        row_idx += 1;
                    }
                }

                let table_height = Fp266::from_int(row_idx as i32).mul(self.line_height(12));
                let start_y = self.cursor_y - table_height;
                table.geometry = LIRGeometry::new(
                    fp_core_to_lir(self.cursor_x),
                    fp_core_to_lir(start_y),
                    fp_core_to_lir(self.content_width()),
                    fp_core_to_lir(table_height),
                );

                self.add_block(LIRNode::Table(table), table_height + Fp266::from_int(6));
                Ok(Vec::new())
            }

            NodeType::TableRow { .. } | NodeType::TableCell { .. } => Ok(Vec::new()),

            NodeType::Figure { placement } => {
                self.figure_counter += 1;
                let mut fig = LIRFigure::new();
                fig.id = self.alloc_id();
                fig.source_node_id = source_id;
                // Map SIR FloatPlacement to LIR Placement
                fig.placement = match placement {
                    ldir_ir::sir::v2::nodes::FloatPlacement::Here => {
                        ldir_ir::lir::style::Placement::Here
                    }
                    ldir_ir::sir::v2::nodes::FloatPlacement::Top => {
                        ldir_ir::lir::style::Placement::Top
                    }
                    ldir_ir::sir::v2::nodes::FloatPlacement::Bottom => {
                        ldir_ir::lir::style::Placement::Bottom
                    }
                    ldir_ir::sir::v2::nodes::FloatPlacement::Page => {
                        ldir_ir::lir::style::Placement::Float
                    }
                    ldir_ir::sir::v2::nodes::FloatPlacement::ForceHere => {
                        ldir_ir::lir::style::Placement::Here
                    }
                };

                for child_id in &child_ids {
                    let child_info = NodeInfo::from_tree(&self.module.body, *child_id)?;
                    match child_info.node_type {
                        NodeType::Image { .. } => {
                            fig.image_index = Some(self.figure_counter - 1);
                        }
                        NodeType::Caption => {
                            let caption_text = self.collect_text(*child_id);
                            let mut caption = LIRCaption::new("Figure", self.figure_counter);
                            caption.id = self.alloc_id();
                            caption.source_node_id = Some(*child_id);

                            if !caption_text.trim().is_empty()
                                && let Some(p) = self.build_paragraph(
                                    &caption_text,
                                    Some(*child_id),
                                    Some(0),
                                    10,
                                    self.content_width(),
                                    self.cursor_x,
                                )
                            {
                                caption.children = p.children;
                            }
                            fig.caption = Some(Box::new(caption));
                        }
                        _ => {}
                    }
                }

                // Defer non-here floats for post-pagination placement pass.
                if fig.placement == ldir_ir::lir::style::Placement::Here {
                    let fig_height = Fp266::from_int(200);
                    self.ensure_space(fig_height);

                    fig.geometry = LIRGeometry::new(
                        fp_core_to_lir(self.cursor_x),
                        fp_core_to_lir(self.cursor_y),
                        fp_core_to_lir(self.content_width()),
                        fp_core_to_lir(fig_height),
                    );

                    self.cursor_y += fig_height + Fp266::from_int(6);
                    self.add_block(LIRNode::Figure(fig), fig_height + Fp266::from_int(6));
                } else {
                    // Defer: position will be set by float placement pass.
                    fig.geometry = LIRGeometry::new(
                        fp_core_to_lir(Fp266::ZERO),
                        fp_core_to_lir(Fp266::ZERO),
                        fp_core_to_lir(self.content_width()),
                        fp_core_to_lir(Fp266::from_int(200)),
                    );
                    self.deferred_floats.push(fig);
                }
                Ok(Vec::new())
            }

            NodeType::Caption => {
                let caption_text = self.collect_text(node_id);
                let mut caption = LIRCaption::new("Figure", 0);
                caption.id = self.alloc_id();
                caption.source_node_id = source_id;

                if !caption_text.trim().is_empty()
                    && let Some(p) = self.build_paragraph(
                        &caption_text,
                        source_id,
                        Some(0),
                        10,
                        self.content_width(),
                        self.cursor_x,
                    )
                {
                    caption.children = p.children;
                    let height = fp_lir_to_core(p.geometry.height);
                    self.cursor_y += height;
                }

                Ok(vec![LIRNode::Caption(caption)])
            }

            NodeType::Footnote { ref content } => {
                self.footnote_counter += 1;
                let fn_id = self.footnote_counter;
                let marker = b'*';

                self.pending_footnotes.push((fn_id, content.clone()));

                let mut footnote = LIRFootnote::new(fn_id, marker);
                footnote.id = self.alloc_id();
                footnote.source_node_id = source_id;

                let fs = Fp266::from_int(8);
                footnote.geometry = LIRGeometry::with_baseline(
                    fp_core_to_lir(self.cursor_x),
                    fp_core_to_lir(self.cursor_y),
                    fp_core_to_lir(fs),
                    fp_core_to_lir(fs),
                    fp_core_to_lir(fs.mul(Fp266::from_frac(8, 10))),
                );

                Ok(vec![LIRNode::Footnote(footnote)])
            }

            NodeType::FootnoteBlock => {
                let mut block = LIRFootnoteBlock::new();
                block.id = self.alloc_id();
                block.source_node_id = source_id;

                let footnotes: Vec<(u32, String)> = self.pending_footnotes.drain(..).collect();
                for (fn_id, content) in &footnotes {
                    block.footnote_ids.push(*fn_id);
                    let fn_text = format!("{} {}", fn_id, content);
                    if let Some(p) = self.build_paragraph(
                        &fn_text,
                        source_id,
                        Some(0),
                        9,
                        self.content_width(),
                        self.cursor_x,
                    ) {
                        let height = fp_lir_to_core(p.geometry.height);
                        self.cursor_y += height;
                        block.children.push(LIRNode::Paragraph(p));
                    }
                }

                let block_height = Fp266::from_int(12);
                block.geometry = LIRGeometry::new(
                    fp_core_to_lir(self.cursor_x),
                    fp_core_to_lir(self.cursor_y),
                    fp_core_to_lir(self.content_width()),
                    fp_core_to_lir(block_height),
                );

                self.add_block(LIRNode::FootnoteBlock(block), block_height);
                Ok(Vec::new())
            }

            NodeType::TableOfContents { max_depth } => {
                let mut toc = LIRTableOfContents::new(max_depth);
                toc.id = self.alloc_id();
                toc.source_node_id = source_id;

                let toc_height = Fp266::from_int(48);
                self.ensure_space(toc_height);

                toc.geometry = LIRGeometry::new(
                    fp_core_to_lir(self.cursor_x),
                    fp_core_to_lir(self.cursor_y),
                    fp_core_to_lir(self.content_width()),
                    fp_core_to_lir(toc_height),
                );

                self.cursor_y += toc_height + Fp266::from_int(12);
                self.add_block(
                    LIRNode::TableOfContents(toc),
                    toc_height + Fp266::from_int(12),
                );
                Ok(Vec::new())
            }

            NodeType::ThematicBreak => {
                let hr_height = Fp266::from_int(12);
                self.ensure_space(hr_height);

                let mut hr = LIRThematicBreak::new();
                hr.id = self.alloc_id();
                hr.source_node_id = source_id;
                hr.geometry = LIRGeometry::new(
                    fp_core_to_lir(self.cursor_x),
                    fp_core_to_lir(self.cursor_y),
                    fp_core_to_lir(self.content_width()),
                    LirFp::ZERO,
                );

                self.cursor_y += hr_height;
                self.add_block(LIRNode::ThematicBreak(hr), hr_height);
                Ok(Vec::new())
            }

            NodeType::PageBreak => {
                if !self.current_page_children.is_empty() {
                    self.finish_page();
                    self.cursor_x = self.ctx.margin_left;
                    self.cursor_y = self.ctx.margin_top;
                }
                Ok(Vec::new())
            }

            NodeType::Image { .. } => {
                let img_height = Fp266::from_int(200);
                self.ensure_space(img_height);

                let mut p = LIRFlow::new(FlowDirection::TopToBottom);
                p.id = self.alloc_id();
                p.source_node_id = source_id;
                p.geometry = LIRGeometry::new(
                    fp_core_to_lir(self.cursor_x),
                    fp_core_to_lir(self.cursor_y),
                    fp_core_to_lir(self.content_width()),
                    fp_core_to_lir(img_height),
                );

                self.cursor_y += img_height + Fp266::from_int(6);
                self.add_block(LIRNode::Flow(p), img_height + Fp266::from_int(6));
                Ok(Vec::new())
            }

            NodeType::MathInline { ref content } => {
                let content_len = content.len() as i32 * 7;
                let mut glyph = LIRGlyph::new(0, 0, fp_core_to_lir(Fp266::from_int(content_len)));
                glyph.id = self.alloc_id();
                glyph.source_node_id = source_id;
                glyph.geometry = LIRGeometry::new(
                    fp_core_to_lir(self.cursor_x),
                    fp_core_to_lir(self.cursor_y),
                    fp_core_to_lir(Fp266::from_int(content_len)),
                    fp_core_to_lir(Fp266::from_int(12)),
                );
                self.cursor_x += Fp266::from_int(content_len);
                Ok(vec![LIRNode::Glyph(glyph)])
            }

            NodeType::Citation { ref keys, .. } => {
                let mut resolved_nums = Vec::new();
                let mut parts: Vec<String> = Vec::new();

                for key in keys {
                    if let Some(_bib) = self.bibliography {
                        if !self.cite_numbers.contains_key(key) {
                            self.cite_counter += 1;
                            self.cite_numbers.insert(key.clone(), self.cite_counter);
                        }
                        if let Some(&num) = self.cite_numbers.get(key) {
                            resolved_nums.push(num);
                            parts.push(format!("[{}]", num));
                        } else {
                            parts.push(format!("[{}]", key));
                        }
                    } else {
                        parts.push(format!("[{}]", key));
                    }
                }

                let text = parts.join(", ");
                let lh = self.line_height(12);
                self.ensure_space(lh);

                if let Some(p) = self.build_paragraph(
                    &text,
                    source_id,
                    Some(0),
                    12,
                    self.content_width(),
                    self.cursor_x,
                ) {
                    let height = fp_lir_to_core(p.geometry.height);
                    self.cursor_y += height;
                    self.add_block(LIRNode::Paragraph(p), height);
                }

                if self.bibliography.is_some() && !resolved_nums.is_empty() {
                    let mut citation = LIRCitation::new(keys.clone());
                    citation.id = self.alloc_id();
                    citation.source_node_id = source_id;
                    citation.numbers = resolved_nums;
                    let font_size = Fp266::from_int(12);
                    citation.geometry = LIRGeometry::new(
                        fp_core_to_lir(self.cursor_x),
                        fp_core_to_lir(self.cursor_y - self.line_height(12)),
                        fp_core_to_lir(Fp266::from_int(20)),
                        fp_core_to_lir(font_size),
                    );
                    return Ok(vec![LIRNode::Citation(citation)]);
                }

                Ok(Vec::new())
            }

            NodeType::LineBreak => {
                self.cursor_y += self.line_height(12);
                Ok(Vec::new())
            }

            NodeType::Reference { .. } | NodeType::Label { .. } => Ok(Vec::new()),
        }
    }

    fn emit_bibliography_section(&mut self) {
        let bib = match self.bibliography {
            Some(b) => b,
            None => return,
        };
        if self.cite_numbers.is_empty() || bib.is_empty() {
            return;
        }

        if !self.current_page_children.is_empty() {
            self.finish_page();
            self.cursor_x = self.ctx.margin_left;
            self.cursor_y = self.ctx.margin_top;
        }

        let mut bibliography = LIRBibliography::new("References");
        bibliography.id = self.alloc_id();
        bibliography.style = self.bib_style.to_string();

        let heading_font_size = 16;
        let heading_lh = self.line_height(heading_font_size);
        self.ensure_space(heading_lh + Fp266::from_int(8));

        if let Some(p) = self.build_paragraph(
            "References",
            None,
            Some(1),
            heading_font_size,
            self.content_width(),
            self.cursor_x,
        ) {
            let height = fp_lir_to_core(p.geometry.height);
            self.cursor_y += height + Fp266::from_int(8);
            bibliography.children.push(LIRNode::Paragraph(p));
        }

        let entry_font_size = 10;
        let mut sorted: Vec<(String, u32)> = self
            .cite_numbers
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        sorted.sort_by_key(|&(_, num)| num);

        let bib_entries: Vec<(u32, String, String)> = sorted
            .iter()
            .filter_map(|(key, num)| {
                bib.get(key.as_str()).map(|entry| {
                    let formatted = match self.bib_style {
                        "apa" => format_citation_apa(entry),
                        _ => format_citation_ieee(entry),
                    };
                    (*num, key.clone(), formatted)
                })
            })
            .collect();

        for (num, key, formatted) in &bib_entries {
            let ref_text = format!("[{}] {}", num, formatted);

            bibliography.entries.push(LIRBibEntry {
                number: *num,
                key: key.clone(),
                formatted: formatted.clone(),
            });

            let entry_lh = self.line_height(entry_font_size);
            self.ensure_space(entry_lh);

            if let Some(p) = self.build_paragraph(
                &ref_text,
                None,
                Some(0),
                entry_font_size,
                self.content_width(),
                self.cursor_x,
            ) {
                let height = fp_lir_to_core(p.geometry.height);
                self.cursor_y += height + Fp266::from_int(6);
                bibliography.children.push(LIRNode::Paragraph(p));
            }
        }

        let bib_height = Fp266::from_int(100);
        bibliography.geometry = LIRGeometry::new(
            fp_core_to_lir(self.cursor_x),
            fp_core_to_lir(self.ctx.margin_top),
            fp_core_to_lir(self.content_width()),
            fp_core_to_lir(bib_height),
        );

        self.add_block(LIRNode::Bibliography(bibliography), Fp266::ZERO);
    }

    /// Place deferred floats using the Cassowary constraint solver.
    ///
    /// For each float, creates solver variables for (x, y) position.
    /// REQUIRED constraints: within page margins. STRONG constraints:
    /// placement hint (Top→y≈margin_top, Bottom→y≈page_bottom-height).
    /// Float placement: tries current page first, defers if infeasible.
    fn place_floats(&mut self) {
        if self.deferred_floats.is_empty() {
            return;
        }

        let page_height = self.ctx.page_height;
        let margin_top = self.ctx.margin_top;
        let margin_bottom = self.ctx.margin_bottom;
        let margin_left = self.ctx.margin_left;
        let content_width = self.content_width();

        // First pass: deterministic placement for explicit hints
        let mut float_pool: Vec<LIRFigure> = Vec::new();
        for mut fig in self.deferred_floats.drain(..) {
            match fig.placement {
                ldir_ir::lir::style::Placement::Top | ldir_ir::lir::style::Placement::Here => {
                    fig.geometry.y = fp_core_to_lir(margin_top);
                    fig.geometry.x = fp_core_to_lir(margin_left);
                    self.current_page_children.push(LIRNode::Figure(fig));
                }
                ldir_ir::lir::style::Placement::Bottom => {
                    let fig_h_core = fp_lir_to_core(fig.geometry.height);
                    fig.geometry.y = fp_core_to_lir(page_height - margin_bottom - fig_h_core);
                    fig.geometry.x = fp_core_to_lir(margin_left);
                    self.current_page_children.push(LIRNode::Figure(fig));
                }
                ldir_ir::lir::style::Placement::Float => {
                    float_pool.push(fig);
                }
            }
        }

        // Second pass: Cassowary solver for Float placement.
        // Try to fit each float on the current page; defer if infeasible.
        if float_pool.is_empty() {
            return;
        }

        let mut deferred: Vec<LIRFigure> = Vec::new();

        for mut fig in float_pool {
            let fig_h = fp_lir_to_core(fig.geometry.height).to_f64();
            let fig_w = fp_lir_to_core(fig.geometry.width).to_f64();
            let page_bottom = (page_height - margin_bottom).to_f64();
            let page_top = margin_top.to_f64();
            let page_left = margin_left.to_f64();
            let page_right = (margin_left + content_width).to_f64();

            let mut solver = Solver::new();
            let var_x = solver.add_variable();
            let var_y = solver.add_variable();

            // REQUIRED: within page bounds
            solver.add_constraint(
                Expression::from_var(var_x, 1.0).add_const(-page_left),
                Strength::REQUIRED,
                Relation::GEQ, // x >= margin_left
            );
            solver.add_constraint(
                Expression::from_var(var_x, 1.0).add_const(-(page_right - fig_w)),
                Strength::REQUIRED,
                Relation::LEQ, // x + w <= page_right => x <= page_right - w
            );
            solver.add_constraint(
                Expression::from_var(var_y, 1.0).add_const(-page_top),
                Strength::REQUIRED,
                Relation::GEQ, // y >= margin_top
            );
            solver.add_constraint(
                Expression::from_var(var_y, 1.0).add_const(-(page_bottom - fig_h)),
                Strength::REQUIRED,
                Relation::LEQ, // y + h <= page_bottom
            );

            // STRONG: prefer left-aligned
            solver.suggest_value(var_x, page_left);
            // STRONG: prefer placed near bottom (typical float style)
            solver.suggest_value(var_y, (page_bottom - fig_h).max(page_top));

            match solver.resolve() {
                Ok(values) => {
                    let solved_x = values.get(&var_x).copied().unwrap_or(page_left);
                    let solved_y = values
                        .get(&var_y)
                        .copied()
                        .unwrap_or((page_bottom - fig_h).max(page_top));
                    fig.geometry.x = LirFp::from_f64(solved_x);
                    fig.geometry.y = LirFp::from_f64(solved_y);
                    self.current_page_children.push(LIRNode::Figure(fig));
                }
                Err(_) => {
                    // Infeasible: defer to next page
                    deferred.push(fig);
                }
            }
        }

        // Place deferred floats on new pages (top-aligned)
        for mut fig in deferred {
            if !self.current_page_children.is_empty() {
                self.finish_page();
            }
            fig.geometry.y = fp_core_to_lir(margin_top);
            fig.geometry.x = fp_core_to_lir(margin_left);
            self.current_page_children.push(LIRNode::Figure(fig));
        }
    }

    fn build_document(mut self) -> LIRDocument {
        self.place_floats();
        self.emit_bibliography_section();

        if !self.current_page_children.is_empty() {
            self.finish_page();
        }

        let style_table = std::mem::take(&mut self.style_table);
        let pages = std::mem::take(&mut self.pages);

        let mut doc = LIRDocument {
            id: 1,
            metadata: LIRDocumentMeta {
                page_width: fp_core_to_lir(self.ctx.page_width),
                page_height: fp_core_to_lir(self.ctx.page_height),
                margin_top: fp_core_to_lir(self.ctx.margin_top),
                margin_bottom: fp_core_to_lir(self.ctx.margin_bottom),
                margin_left: fp_core_to_lir(self.ctx.margin_left),
                margin_right: fp_core_to_lir(self.ctx.margin_right),
                language: self.module.metadata.language.clone(),
            },
            pages,
            style_table,
            ..Default::default()
        };

        let mut toc = LIRTableOfContents::new(4);
        toc.id = self.alloc_id();
        for (level, number, label) in &self.heading_entries {
            let page = doc
                .pages
                .iter()
                .find(|p| {
                    p.children.iter().any(|c| {
                        if let LIRNode::Heading(h) = c {
                            h.number == *number && h.label == *label
                        } else {
                            false
                        }
                    })
                })
                .map(|p| p.page_number)
                .unwrap_or(1);
            toc.entries.push(TOCEntry {
                level: *level,
                number: number.clone(),
                label: label.clone(),
                page_number: page,
            });
        }
        if !toc.entries.is_empty() {
            doc.toc = Some(toc);
        }

        if let Some(LIRNode::Bibliography(bib)) = doc
            .pages
            .iter()
            .flat_map(|p| p.children.iter())
            .find(|n| matches!(n, LIRNode::Bibliography(_)))
        {
            doc.bibliography = Some(bib.clone());
        }

        doc
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Compile an S-IR v2 module into a positioned LIR document tree.
///
/// This is the main entry point for the S-IR -> L-IR compilation pass.
/// It produces a fully positioned document with:
/// - Page breaks when content exceeds page height
/// - Line-broken paragraphs using Knuth-Plass
/// - Section numbering for headings
/// - Collected footnotes and table of contents
pub fn compile_sir_to_lir(module: &SIRModuleV2, ctx: &CompileContext) -> LirResult<LIRDocument> {
    compile_sir_to_lir_inner(module, ctx, None, "ieee")
}

/// Compile an S-IR v2 module with bibliography into a positioned LIR document tree.
///
/// Same as [`compile_sir_to_lir`] but also resolves citations against the provided
/// bibliography and emits a formatted references section.
///
/// `bib_style` controls formatting: `"ieee"` (default) or `"apa"`.
pub fn compile_sir_to_lir_with_bib(
    module: &SIRModuleV2,
    ctx: &CompileContext,
    bibliography: &HashMap<String, BibEntry>,
    bib_style: &str,
) -> LirResult<LIRDocument> {
    let style: &'static str = match bib_style {
        "apa" => "apa",
        _ => "ieee",
    };
    compile_sir_to_lir_inner(module, ctx, Some(bibliography), style)
}

fn compile_sir_to_lir_inner(
    module: &SIRModuleV2,
    ctx: &CompileContext,
    bibliography: Option<&HashMap<String, BibEntry>>,
    bib_style: &'static str,
) -> LirResult<LIRDocument> {
    let mut compiler = LirCompiler::with_bib(module, ctx, bibliography, bib_style);

    for &root_id in module.body.roots() {
        compiler.compile_node(root_id)?;
    }

    Ok(compiler.build_document())
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use ldir_ir::sir::v2::nodes::ListType as SIRListType;
    use ldir_ir::sir::v2::nodes::Node;

    fn make_module() -> SIRModuleV2 {
        SIRModuleV2::new()
    }

    fn make_ctx() -> CompileContext {
        CompileContext::new()
    }

    fn add_child(tree: &mut ldir_ir::sir::v2::nodes::NodeTree, parent_id: u32, child_id: u32) {
        if let Some(node) = tree.get_mut(parent_id) {
            node.add_child(child_id);
        }
    }

    #[test]
    fn test_empty_document() {
        let module = make_module();
        let ctx = make_ctx();
        let doc = compile_sir_to_lir(&module, &ctx).unwrap();
        assert!(doc.pages.is_empty());
    }

    #[test]
    fn test_single_paragraph() {
        let mut module = make_module();
        let para_id = module.body.push(Node::new(1, NodeType::Paragraph));
        let text_id = module.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "Hello world".into(),
                },
            )
            .with_parent(1),
        );
        add_child(&mut module.body, para_id, text_id);

        let ctx = make_ctx();
        let doc = compile_sir_to_lir(&module, &ctx).unwrap();
        assert_eq!(doc.pages.len(), 1);
        let found = doc.pages[0]
            .children
            .iter()
            .any(|n| matches!(n, LIRNode::Paragraph(_)));
        assert!(found, "expected a paragraph node");
    }

    #[test]
    fn test_heading_and_paragraph() {
        let mut module = make_module();
        let doc_id = module.body.push(Node::new(1, NodeType::Document));
        let sec_id = module.body.push(
            Node::new(2, NodeType::Section)
                .with_parent(1)
                .with_label("intro"),
        );
        let sec_text_id = module.body.push(
            Node::new(
                3,
                NodeType::Text {
                    content: "Introduction".into(),
                },
            )
            .with_parent(2),
        );
        let para_id = module
            .body
            .push(Node::new(4, NodeType::Paragraph).with_parent(1));
        let para_text_id = module.body.push(
            Node::new(
                5,
                NodeType::Text {
                    content: "Some body text here.".into(),
                },
            )
            .with_parent(4),
        );

        add_child(&mut module.body, doc_id, sec_id);
        add_child(&mut module.body, doc_id, para_id);
        add_child(&mut module.body, sec_id, sec_text_id);
        add_child(&mut module.body, para_id, para_text_id);

        let ctx = make_ctx();
        let doc = compile_sir_to_lir(&module, &ctx).unwrap();
        assert_eq!(doc.pages.len(), 1);

        let has_heading = doc.pages[0]
            .children
            .iter()
            .any(|n| matches!(n, LIRNode::Heading(_)));
        assert!(
            has_heading,
            "expected a heading node, got: {:?}",
            doc.pages[0]
                .children
                .iter()
                .map(|n| n.type_name())
                .collect::<Vec<_>>()
        );

        if let Some(LIRNode::Heading(h)) = doc.pages[0]
            .children
            .iter()
            .find(|n| matches!(n, LIRNode::Heading(_)))
        {
            assert_eq!(h.level, 2);
            assert_eq!(h.label, "Introduction");
        }
    }

    #[test]
    fn test_page_breaking() {
        let mut module = make_module();
        let doc_id = module.body.push(Node::new(1, NodeType::Document));

        let mut next_id = 2u32;
        for i in 0..100 {
            let para_id = module
                .body
                .push(Node::new(next_id, NodeType::Paragraph).with_parent(1));
            let text_id = module.body.push(
                Node::new(
                    next_id + 1,
                    NodeType::Text {
                        content: format!("Paragraph {} with enough text to fill a line.", i),
                    },
                )
                .with_parent(next_id),
            );
            add_child(&mut module.body, doc_id, para_id);
            add_child(&mut module.body, para_id, text_id);
            next_id += 2;
        }

        let ctx = make_ctx();
        let doc = compile_sir_to_lir(&module, &ctx).unwrap();
        assert!(
            doc.pages.len() > 1,
            "expected multiple pages, got {}",
            doc.pages.len()
        );
    }

    #[test]
    fn test_explicit_page_break() {
        let mut module = make_module();
        let doc_id = module.body.push(Node::new(1, NodeType::Document));
        let para1_id = module
            .body
            .push(Node::new(2, NodeType::Paragraph).with_parent(1));
        let text1_id = module.body.push(
            Node::new(
                3,
                NodeType::Text {
                    content: "Page 1 content".into(),
                },
            )
            .with_parent(2),
        );
        let pb_id = module
            .body
            .push(Node::new(4, NodeType::PageBreak).with_parent(1));
        let para2_id = module
            .body
            .push(Node::new(5, NodeType::Paragraph).with_parent(1));
        let text2_id = module.body.push(
            Node::new(
                6,
                NodeType::Text {
                    content: "Page 2 content".into(),
                },
            )
            .with_parent(5),
        );

        add_child(&mut module.body, doc_id, para1_id);
        add_child(&mut module.body, doc_id, pb_id);
        add_child(&mut module.body, doc_id, para2_id);
        add_child(&mut module.body, para1_id, text1_id);
        add_child(&mut module.body, para2_id, text2_id);

        let ctx = make_ctx();
        let doc = compile_sir_to_lir(&module, &ctx).unwrap();
        assert_eq!(doc.pages.len(), 2);
    }

    #[test]
    fn test_unordered_list() {
        let mut module = make_module();
        let doc_id = module.body.push(Node::new(1, NodeType::Document));
        let list_id = module.body.push(
            Node::new(
                2,
                NodeType::List {
                    list_type: SIRListType::Unordered,
                    ordered: false,
                    start: None,
                },
            )
            .with_parent(1),
        );
        let item1_id = module
            .body
            .push(Node::new(3, NodeType::ListItem).with_parent(2));
        let text1_id = module.body.push(
            Node::new(
                4,
                NodeType::Text {
                    content: "First item".into(),
                },
            )
            .with_parent(3),
        );
        let item2_id = module
            .body
            .push(Node::new(5, NodeType::ListItem).with_parent(2));
        let text2_id = module.body.push(
            Node::new(
                6,
                NodeType::Text {
                    content: "Second item".into(),
                },
            )
            .with_parent(5),
        );

        add_child(&mut module.body, doc_id, list_id);
        add_child(&mut module.body, list_id, item1_id);
        add_child(&mut module.body, list_id, item2_id);
        add_child(&mut module.body, item1_id, text1_id);
        add_child(&mut module.body, item2_id, text2_id);

        let ctx = make_ctx();
        let doc = compile_sir_to_lir(&module, &ctx).unwrap();
        assert_eq!(doc.pages.len(), 1);

        if let Some(LIRNode::List(l)) = doc.pages[0]
            .children
            .iter()
            .find(|n| matches!(n, LIRNode::List(_)))
        {
            assert_eq!(l.list_type, ListType::Unordered);
            assert_eq!(l.children.len(), 2);
        }
    }

    #[test]
    fn test_table() {
        let mut module = make_module();
        let doc_id = module.body.push(Node::new(1, NodeType::Document));
        let table_id = module.body.push(
            Node::new(
                2,
                NodeType::Table {
                    col_specs: vec![
                        ldir_ir::sir::v2::nodes::ColSpec {
                            align: ldir_ir::sir::v2::nodes::ColumnAlign::Left,
                            width: None,
                        },
                        ldir_ir::sir::v2::nodes::ColSpec {
                            align: ldir_ir::sir::v2::nodes::ColumnAlign::Left,
                            width: None,
                        },
                    ],
                    num_cols: 2,
                    caption: None,
                    column_widths: vec![],
                    header_row: false,
                },
            )
            .with_parent(1),
        );
        let row_id = module
            .body
            .push(Node::new(3, NodeType::TableRow { is_header: true }).with_parent(2));
        let cell1_id = module.body.push(
            Node::new(
                4,
                NodeType::TableCell {
                    colspan: 1,
                    rowspan: 1,
                },
            )
            .with_parent(3),
        );
        let cell1_text = module.body.push(
            Node::new(
                5,
                NodeType::Text {
                    content: "Header 1".into(),
                },
            )
            .with_parent(4),
        );
        let cell2_id = module.body.push(
            Node::new(
                6,
                NodeType::TableCell {
                    colspan: 1,
                    rowspan: 1,
                },
            )
            .with_parent(3),
        );
        let cell2_text = module.body.push(
            Node::new(
                7,
                NodeType::Text {
                    content: "Header 2".into(),
                },
            )
            .with_parent(6),
        );

        add_child(&mut module.body, doc_id, table_id);
        add_child(&mut module.body, table_id, row_id);
        add_child(&mut module.body, row_id, cell1_id);
        add_child(&mut module.body, row_id, cell2_id);
        add_child(&mut module.body, cell1_id, cell1_text);
        add_child(&mut module.body, cell2_id, cell2_text);

        let ctx = make_ctx();
        let doc = compile_sir_to_lir(&module, &ctx).unwrap();

        if let Some(LIRNode::Table(t)) = doc.pages[0]
            .children
            .iter()
            .find(|n| matches!(n, LIRNode::Table(_)))
        {
            assert_eq!(t.num_cols, 2);
            assert_eq!(t.children.len(), 1);
            assert!(t.border);
        }
    }

    #[test]
    fn test_footnote() {
        let mut module = make_module();
        let doc_id = module.body.push(Node::new(1, NodeType::Document));
        let para_id = module
            .body
            .push(Node::new(2, NodeType::Paragraph).with_parent(1));
        let fn_id = module.body.push(
            Node::new(
                3,
                NodeType::Footnote {
                    content: "A footnote".into(),
                },
            )
            .with_parent(2),
        );
        let text_id = module.body.push(
            Node::new(
                4,
                NodeType::Text {
                    content: "Text with".into(),
                },
            )
            .with_parent(2),
        );
        let fn_block_id = module
            .body
            .push(Node::new(5, NodeType::FootnoteBlock).with_parent(1));

        add_child(&mut module.body, doc_id, para_id);
        add_child(&mut module.body, doc_id, fn_block_id);
        add_child(&mut module.body, para_id, text_id);
        add_child(&mut module.body, para_id, fn_id);

        let ctx = make_ctx();
        let doc = compile_sir_to_lir(&module, &ctx).unwrap();

        let has_fn_block = doc.pages[0]
            .children
            .iter()
            .any(|n| matches!(n, LIRNode::FootnoteBlock(_)));
        assert!(has_fn_block, "expected a footnote block");
    }

    #[test]
    fn test_toc() {
        let mut module = make_module();
        let doc_id = module.body.push(Node::new(1, NodeType::Document));
        let toc_id = module
            .body
            .push(Node::new(2, NodeType::TableOfContents { max_depth: 3 }).with_parent(1));
        add_child(&mut module.body, doc_id, toc_id);

        let ctx = make_ctx();
        let doc = compile_sir_to_lir(&module, &ctx).unwrap();

        let has_toc = doc.pages[0]
            .children
            .iter()
            .any(|n| matches!(n, LIRNode::TableOfContents(_)));
        assert!(has_toc, "expected a TOC node");
    }

    #[test]
    fn test_thematic_break() {
        let mut module = make_module();
        let doc_id = module.body.push(Node::new(1, NodeType::Document));
        let hr_id = module
            .body
            .push(Node::new(2, NodeType::ThematicBreak).with_parent(1));
        add_child(&mut module.body, doc_id, hr_id);

        let ctx = make_ctx();
        let doc = compile_sir_to_lir(&module, &ctx).unwrap();

        let has_hr = doc.pages[0]
            .children
            .iter()
            .any(|n| matches!(n, LIRNode::ThematicBreak(_)));
        assert!(has_hr, "expected a thematic break");
    }

    #[test]
    fn test_code_block() {
        let mut module = make_module();
        let doc_id = module.body.push(Node::new(1, NodeType::Document));
        let cb_id = module.body.push(
            Node::new(
                2,
                NodeType::CodeBlock {
                    language: Some("rust".into()),
                    content: String::new(),
                },
            )
            .with_parent(1),
        );
        let line1 = module.body.push(
            Node::new(
                3,
                NodeType::Text {
                    content: "fn main() {}".into(),
                },
            )
            .with_parent(2),
        );
        add_child(&mut module.body, doc_id, cb_id);
        add_child(&mut module.body, cb_id, line1);

        let ctx = make_ctx();
        let doc = compile_sir_to_lir(&module, &ctx).unwrap();

        if let Some(LIRNode::CodeBlock(cb)) = doc.pages[0]
            .children
            .iter()
            .find(|n| matches!(n, LIRNode::CodeBlock(_)))
        {
            assert_eq!(cb.language, "rust");
            assert!(!cb.children.is_empty());
        }
    }

    #[test]
    fn test_node_not_found_error() {
        let mut module = make_module();
        module.body.push(Node::new(1, NodeType::Document));
        if let Some(node) = module.body.get_mut(1) {
            node.child_ids.push(999);
        }

        let ctx = make_ctx();
        let result = compile_sir_to_lir(&module, &ctx);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LirError::NodeNotFound(999)));
    }

    #[test]
    fn test_document_metadata() {
        let module = make_module();
        let ctx = CompileContext::with_font_margins_and_page(None, 36, 36, 50, 50, 595, 842);
        let doc = compile_sir_to_lir(&module, &ctx).unwrap();

        assert_eq!(doc.metadata.page_width, LirFp::from_int(595));
        assert_eq!(doc.metadata.page_height, LirFp::from_int(842));
        assert_eq!(doc.metadata.margin_left, LirFp::from_int(36));
        assert_eq!(doc.metadata.content_width(), LirFp::from_int(523));
    }

    #[test]
    fn test_style_table_populated() {
        let mut module = make_module();
        let para_id = module.body.push(Node::new(1, NodeType::Paragraph));
        let text_id = module.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "Test".into(),
                },
            )
            .with_parent(1),
        );
        add_child(&mut module.body, para_id, text_id);

        let ctx = make_ctx();
        let doc = compile_sir_to_lir(&module, &ctx).unwrap();
        assert!(doc.style_table.len() >= 5);
    }

    #[test]
    fn test_block_quote() {
        let mut module = make_module();
        let doc_id = module.body.push(Node::new(1, NodeType::Document));
        let bq_id = module
            .body
            .push(Node::new(2, NodeType::BlockQuote).with_parent(1));
        let para_id = module
            .body
            .push(Node::new(3, NodeType::Paragraph).with_parent(2));
        let text_id = module.body.push(
            Node::new(
                4,
                NodeType::Text {
                    content: "A quote".into(),
                },
            )
            .with_parent(3),
        );

        add_child(&mut module.body, doc_id, bq_id);
        add_child(&mut module.body, bq_id, para_id);
        add_child(&mut module.body, para_id, text_id);

        let ctx = make_ctx();
        let doc = compile_sir_to_lir(&module, &ctx).unwrap();

        let has_bq = doc.pages[0]
            .children
            .iter()
            .any(|n| matches!(n, LIRNode::BlockQuote(_)));
        assert!(has_bq, "expected a block quote");
    }

    #[test]
    fn test_math_block() {
        let mut module = make_module();
        let doc_id = module.body.push(Node::new(1, NodeType::Document));
        let math_id = module.body.push(
            Node::new(
                2,
                NodeType::MathBlock {
                    math_type: ldir_ir::sir::v2::nodes::MathType::Equation,
                    numbered: true,
                },
            )
            .with_parent(1),
        );
        let math_text = module.body.push(
            Node::new(
                3,
                NodeType::Text {
                    content: "E = mc^2".into(),
                },
            )
            .with_parent(2),
        );
        add_child(&mut module.body, doc_id, math_id);
        add_child(&mut module.body, math_id, math_text);

        let ctx = make_ctx();
        let doc = compile_sir_to_lir(&module, &ctx).unwrap();

        if let Some(LIRNode::MathBlock(mb)) = doc.pages[0]
            .children
            .iter()
            .find(|n| matches!(n, LIRNode::MathBlock(_)))
        {
            assert_eq!(mb.math_type, MathType::Display);
            assert_eq!(mb.number, Some(1));
        }
    }

    #[test]
    fn test_citation_without_bib() {
        let mut module = make_module();
        let doc_id = module.body.push(Node::new(1, NodeType::Document));
        let para_id = module
            .body
            .push(Node::new(2, NodeType::Paragraph).with_parent(1));
        let cite_id = module.body.push(
            Node::new(
                3,
                NodeType::Citation {
                    keys: vec!["knuth1984".into()],
                    style: None,
                },
            )
            .with_parent(2),
        );
        let text_id = module.body.push(
            Node::new(
                4,
                NodeType::Text {
                    content: "See ".into(),
                },
            )
            .with_parent(2),
        );

        add_child(&mut module.body, doc_id, para_id);
        add_child(&mut module.body, para_id, text_id);
        add_child(&mut module.body, para_id, cite_id);

        let ctx = make_ctx();
        let doc = compile_sir_to_lir(&module, &ctx).unwrap();
        assert_eq!(doc.pages.len(), 1);
        assert!(
            doc.bibliography.is_none(),
            "no bib data means no bibliography"
        );
    }

    #[test]
    fn test_bibliography_with_citations() {
        use crate::compiler::bibtex::parse_bib;

        let mut module = make_module();
        let doc_id = module.body.push(Node::new(1, NodeType::Document));
        let para_id = module
            .body
            .push(Node::new(2, NodeType::Paragraph).with_parent(1));
        let text_id = module.body.push(
            Node::new(
                3,
                NodeType::Text {
                    content: "As shown by [knuth1984].".into(),
                },
            )
            .with_parent(2),
        );
        let cite_id = module.body.push(
            Node::new(
                4,
                NodeType::Citation {
                    keys: vec!["knuth1984".into()],
                    style: None,
                },
            )
            .with_parent(1),
        );

        add_child(&mut module.body, doc_id, para_id);
        add_child(&mut module.body, para_id, text_id);
        add_child(&mut module.body, doc_id, cite_id);

        let bib_content = r#"@article{knuth1984,
            author = {Donald E. Knuth},
            title = {Literate Programming},
            journal = {The Computer Journal},
            volume = {27},
            pages = {97--111},
            year = {1984},
        }"#;

        let bibliography = parse_bib(bib_content).expect("parse bib");
        let ctx = make_ctx();
        let doc = compile_sir_to_lir_with_bib(&module, &ctx, &bibliography, "ieee").unwrap();

        assert!(
            doc.bibliography.is_some(),
            "bibliography should be generated"
        );
        let bib = doc.bibliography.as_ref().unwrap();
        assert_eq!(bib.entries.len(), 1);
        assert_eq!(bib.entries[0].key, "knuth1984");
        assert_eq!(bib.entries[0].number, 1);
        assert!(bib.entries[0].formatted.contains("Knuth"));
        assert!(bib.entries[0].formatted.contains("1984"));
    }

    #[test]
    fn test_bibliography_apa_style() {
        use crate::compiler::bibtex::parse_bib;

        let mut module = make_module();
        let doc_id = module.body.push(Node::new(1, NodeType::Document));
        let para_id = module
            .body
            .push(Node::new(2, NodeType::Paragraph).with_parent(1));
        let cite_id = module.body.push(
            Node::new(
                3,
                NodeType::Citation {
                    keys: vec!["knuth1984".into()],
                    style: None,
                },
            )
            .with_parent(1),
        );
        add_child(&mut module.body, doc_id, para_id);
        add_child(&mut module.body, doc_id, cite_id);

        let bib_content = r#"@article{knuth1984,
            author = {Donald E. Knuth},
            title = {Literate Programming},
            journal = {The Computer Journal},
            year = {1984},
        }"#;

        let bibliography = parse_bib(bib_content).expect("parse bib");
        let ctx = make_ctx();
        let doc = compile_sir_to_lir_with_bib(&module, &ctx, &bibliography, "apa").unwrap();

        let bib = doc.bibliography.as_ref().unwrap();
        assert_eq!(bib.style, "apa");
        assert!(bib.entries[0].formatted.contains("Knuth (1984)"));
    }
}
