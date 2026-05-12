#![deny(unsafe_code)]
#![warn(clippy::unwrap_used, clippy::expect_used)]
#![allow(dead_code)]

use indexmap::IndexMap;
use std::collections::HashMap;

use bumpalo::collections::Vec as BumpVec;
use ldir_ir::gir::{GIRCommand, GIRDocument, GIRPage};
use ldir_ir::sir::StyleModifier;
use ldir_ir::sir::v2::annotations::LabelCategory;
use ldir_ir::sir::v2::metadata::Dimension;
use ldir_ir::sir::v2::nodes::{ColSpec, ColumnAlign, NodeType};
use ldir_ir::sir::v2::resources::ResourceDecls;
use ldir_ir::sir::v2::{DocumentMetadata, SIRModuleV2};

use crate::compiler::bibtex::{BibEntry, format_citation_ieee};
use crate::compiler::context::{
    CompileContext, FONT_ID_BOLD, FONT_ID_BOLD_ITALIC, FONT_ID_ITALIC, FONT_ID_MONO,
    FONT_ID_REGULAR,
};
use crate::compiler::cross_ref;
use crate::compiler::emit_helpers;
use crate::compiler::justify;
use crate::error::Result;
use crate::fp266::Fp266;
use crate::layout::linebreak::cjk::{insert_cjk_breaks, is_cjk_text};
use crate::layout::linebreak::{LineBreakItem, LineBreakOptions, linebreak};

fn to_superscript(n: usize) -> String {
    const SUPERSCRIPTS: &[char] = &[
        '\u{2070}', '\u{00B9}', '\u{00B2}', '\u{00B3}', '\u{2074}', '\u{2075}', '\u{2076}',
        '\u{2077}', '\u{2078}', '\u{2079}',
    ];
    n.to_string()
        .as_bytes()
        .iter()
        .map(|&d| SUPERSCRIPTS[(d - b'0') as usize])
        .collect()
}

/// Compile a v2 S-IR module into a G-IR document.
pub fn compile_v2_document(module: &SIRModuleV2, ctx: &mut CompileContext) -> Result<GIRDocument> {
    apply_page_geometry(ctx, &module.metadata);
    resolve_v2_fonts(ctx, &module.resources);

    let resolved_labels = cross_ref::collect_labels(module);
    let mut labels = cross_ref::resolve_references(&resolved_labels);
    let refs_map = labels.clone();

    let mut gir_doc = GIRDocument::with_capacity(1);
    let mut page = ctx.new_page();
    let mut eq_counter: u32 = 0;
    let mut section_counters: IndexMap<u8, u32> = IndexMap::new();
    let mut section_number: Vec<u32> = Vec::new();
    let mut footnotes: Vec<(usize, String)> = Vec::new();
    let mut figure_counter: usize = 0;

    for &root_id in module.body.roots() {
        compile_v2_node(
            root_id,
            module,
            &mut page,
            ctx,
            &mut gir_doc,
            &mut labels,
            &refs_map,
            &mut eq_counter,
            &mut section_counters,
            &mut section_number,
            &mut footnotes,
            &mut figure_counter,
        )?;
    }

    if !page.is_empty() {
        gir_doc.push_page(page);
    }

    Ok(gir_doc)
}

/// Compile a v2 S-IR module with bibliography into a G-IR document.
pub fn compile_v2_document_with_bib(
    module: &SIRModuleV2,
    ctx: &mut CompileContext,
    bibliography: &HashMap<String, BibEntry>,
) -> Result<GIRDocument> {
    apply_page_geometry(ctx, &module.metadata);
    resolve_v2_fonts(ctx, &module.resources);

    let resolved_labels = cross_ref::collect_labels(module);
    let mut labels = cross_ref::resolve_references(&resolved_labels);
    let refs_map = labels.clone();

    let mut gir_doc = GIRDocument::with_capacity(1);
    let mut page = ctx.new_page();
    let mut eq_counter: u32 = 0;
    let mut section_counters: IndexMap<u8, u32> = IndexMap::new();
    let mut section_number: Vec<u32> = Vec::new();
    let mut cite_counter: u32 = 0;
    let mut cite_numbers: IndexMap<String, u32> = IndexMap::new();
    let mut footnotes: Vec<(usize, String)> = Vec::new();
    let mut figure_counter: usize = 0;

    for &root_id in module.body.roots() {
        compile_v2_node_with_bib(
            root_id,
            module,
            &mut page,
            ctx,
            &mut gir_doc,
            &mut labels,
            &refs_map,
            &mut eq_counter,
            &mut section_counters,
            &mut section_number,
            bibliography,
            &mut cite_counter,
            &mut cite_numbers,
            &mut footnotes,
            &mut figure_counter,
        )?;
    }

    emit_bibliography_page(
        module,
        bibliography,
        &cite_numbers,
        ctx,
        &mut page,
        &mut gir_doc,
    );

    if !page.is_empty() {
        gir_doc.push_page(page);
    }

    Ok(gir_doc)
}

fn apply_page_geometry(ctx: &mut CompileContext, metadata: &DocumentMetadata) {
    if let Some(ps) = &metadata.page_style {
        let pw_fp = Fp266::from_f64(ps.page_width);
        let ph_fp = Fp266::from_f64(ps.page_height);
        let ml_fp = Fp266::from_f64(ps.margin_left);
        let mr_fp = Fp266::from_f64(ps.margin_right);
        let mt_fp = Fp266::from_f64(ps.margin_top);
        let mb_fp = Fp266::from_f64(ps.margin_bottom);

        ctx.page_width = pw_fp;
        ctx.page_height = ph_fp;
        ctx.margin_left = ml_fp;
        ctx.margin_right = mr_fp;
        ctx.margin_top = mt_fp;
        ctx.margin_bottom = mb_fp;
        ctx.content_width = pw_fp - ml_fp - mr_fp;
        ctx.x = ml_fp;
        ctx.y = mt_fp;
        return;
    }

    let pg = match &metadata.page_geometry {
        Some(pg) => pg,
        None => return,
    };

    let w = pg.width.to_points() as i32;
    let h = pg.height.to_points() as i32;

    let ml = pg.margin_left.to_points() as i32;
    let mr = pg.margin_right.to_points() as i32;
    let mt = pg.margin_top.to_points() as i32;
    let mb = pg.margin_bottom.to_points() as i32;

    let ml_fp = Fp266::from_int(ml);
    let mr_fp = Fp266::from_int(mr);
    let mt_fp = Fp266::from_int(mt);
    let mb_fp = Fp266::from_int(mb);
    let pw_fp = Fp266::from_int(w);
    let ph_fp = Fp266::from_int(h);

    ctx.page_width = pw_fp;
    ctx.page_height = ph_fp;
    ctx.margin_left = ml_fp;
    ctx.margin_right = mr_fp;
    ctx.margin_top = mt_fp;
    ctx.margin_bottom = mb_fp;
    ctx.content_width = pw_fp - ml_fp - mr_fp;
    ctx.x = ml_fp;
    ctx.y = mt_fp;
}

fn resolve_v2_fonts(ctx: &mut CompileContext, resources: &ResourceDecls) {
    let font_db = match &ctx.font_db {
        Some(db) => db.clone(),
        None => return,
    };

    for decl in &resources.fonts {
        let weight = match decl.weight {
            ldir_ir::sir::v2::resources::FontWeight::Bold
            | ldir_ir::sir::v2::resources::FontWeight::ExtraBold
            | ldir_ir::sir::v2::resources::FontWeight::SemiBold
            | ldir_ir::sir::v2::resources::FontWeight::Black => fontdb::Weight::BOLD,
            ldir_ir::sir::v2::resources::FontWeight::Thin
            | ldir_ir::sir::v2::resources::FontWeight::ExtraLight
            | ldir_ir::sir::v2::resources::FontWeight::Light
            | ldir_ir::sir::v2::resources::FontWeight::Medium => fontdb::Weight::NORMAL,
            _ => fontdb::Weight::NORMAL,
        };
        let style = match decl.style {
            ldir_ir::sir::v2::resources::FontStyle::Italic
            | ldir_ir::sir::v2::resources::FontStyle::Oblique => fontdb::Style::Italic,
            _ => fontdb::Style::Normal,
        };

        if let Some(id) = font_db.query_family_style(&decl.family, weight, style)
            && let Some(data) = font_db.face_data(id)
        {
            let font_id = match (weight, style) {
                (fontdb::Weight::BOLD, fontdb::Style::Italic) => FONT_ID_BOLD_ITALIC,
                (fontdb::Weight::BOLD, _) => FONT_ID_BOLD,
                (_, fontdb::Style::Italic) => FONT_ID_ITALIC,
                _ => FONT_ID_REGULAR,
            };
            ctx.set_font_variant(font_id as usize, Some(data));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn compile_v2_node(
    node_id: u32,
    module: &SIRModuleV2,
    page: &mut GIRPage,
    ctx: &mut CompileContext,
    gir_doc: &mut GIRDocument,
    labels: &mut IndexMap<String, String>,
    refs_map: &IndexMap<String, String>,
    eq_counter: &mut u32,
    section_counters: &mut IndexMap<u8, u32>,
    section_number: &mut Vec<u32>,
    footnotes: &mut Vec<(usize, String)>,
    figure_counter: &mut usize,
) -> Result<()> {
    let node = module.body.get(node_id).ok_or(crate::error::LdirError {
        kind: crate::error::ErrorKind::Compile(
            crate::error::CompileErrorKind::UnsupportedInstruction { entity_id: node_id },
        ),
        entity_id: Some(node_id),
        byte_offset: None,
    })?;

    match &node.node_type {
        NodeType::Document => {
            compile_children_with_refs(
                node_id,
                module,
                page,
                ctx,
                gir_doc,
                labels,
                refs_map,
                eq_counter,
                section_counters,
                section_number,
                footnotes,
                figure_counter,
            )?;
        }
        NodeType::Paragraph => {
            emit_v2_paragraph(node_id, module, page, ctx, gir_doc, labels, refs_map)?;
        }
        NodeType::Chapter => {
            increment_section_counter(1, section_counters, section_number);
            let num = section_number_string(section_number);
            if let Some(label) = &node.label {
                labels.insert(label.clone(), num.clone());
            }
            ctx.chapter_title = module.body.collect_text(node_id);
            emit_v2_heading_with_number(node_id, module, page, ctx, gir_doc, 24, &num)?;
        }
        NodeType::Section => {
            increment_section_counter(2, section_counters, section_number);
            let num = section_number_string(section_number);
            if let Some(label) = &node.label {
                labels.insert(label.clone(), num.clone());
            }
            ctx.section_title = module.body.collect_text(node_id);
            emit_v2_heading_with_number(node_id, module, page, ctx, gir_doc, 20, &num)?;
        }
        NodeType::Subsection => {
            increment_section_counter(3, section_counters, section_number);
            let num = section_number_string(section_number);
            if let Some(label) = &node.label {
                labels.insert(label.clone(), num.clone());
            }
            emit_v2_heading_with_number(node_id, module, page, ctx, gir_doc, 16, &num)?;
        }
        NodeType::Subsubsection => {
            increment_section_counter(4, section_counters, section_number);
            let num = section_number_string(section_number);
            if let Some(label) = &node.label {
                labels.insert(label.clone(), num.clone());
            }
            emit_v2_heading_with_number(node_id, module, page, ctx, gir_doc, 14, &num)?;
        }
        NodeType::Part => emit_v2_heading(node_id, module, page, ctx, gir_doc, 26)?,
        NodeType::Text { .. } => {
            let text = module.body.collect_text(node_id);
            let resolved = resolve_v2_references(&text, labels, refs_map);
            if !resolved.trim().is_empty() {
                emit_v2_text_inline(&resolved, page, ctx, gir_doc);
            }
        }
        NodeType::Bold => {
            ctx.push_style(StyleModifier::BOLD_STYLE);
            compile_children_with_refs(
                node_id,
                module,
                page,
                ctx,
                gir_doc,
                labels,
                refs_map,
                eq_counter,
                section_counters,
                section_number,
                footnotes,
                figure_counter,
            )?;
            ctx.pop_style();
        }
        NodeType::Italic => {
            ctx.push_style(StyleModifier::ITALIC_STYLE);
            compile_children_with_refs(
                node_id,
                module,
                page,
                ctx,
                gir_doc,
                labels,
                refs_map,
                eq_counter,
                section_counters,
                section_number,
                footnotes,
                figure_counter,
            )?;
            ctx.pop_style();
        }
        NodeType::Mono => {
            ctx.push_style(StyleModifier::MONO_STYLE);
            compile_children_with_refs(
                node_id,
                module,
                page,
                ctx,
                gir_doc,
                labels,
                refs_map,
                eq_counter,
                section_counters,
                section_number,
                footnotes,
                figure_counter,
            )?;
            ctx.pop_style();
        }
        NodeType::Underline => {
            ctx.push_style(StyleModifier(StyleModifier::UNDERLINE));
            compile_children_with_refs(
                node_id,
                module,
                page,
                ctx,
                gir_doc,
                labels,
                refs_map,
                eq_counter,
                section_counters,
                section_number,
                footnotes,
                figure_counter,
            )?;
            ctx.pop_style();
        }
        NodeType::Strikethrough => {
            ctx.push_style(StyleModifier(StyleModifier::STRIKE));
            compile_children_with_refs(
                node_id,
                module,
                page,
                ctx,
                gir_doc,
                labels,
                refs_map,
                eq_counter,
                section_counters,
                section_number,
                footnotes,
                figure_counter,
            )?;
            ctx.pop_style();
        }
        NodeType::SmallCaps => {
            ctx.push_style(StyleModifier(StyleModifier::SMALL_CAPS));
            compile_children_with_refs(
                node_id,
                module,
                page,
                ctx,
                gir_doc,
                labels,
                refs_map,
                eq_counter,
                section_counters,
                section_number,
                footnotes,
                figure_counter,
            )?;
            ctx.pop_style();
        }
        NodeType::Styled { style_name } => {
            let needs_italic = module.styles.find(style_name).is_some_and(|style_decl| {
                style_decl.properties.font_style.as_deref() == Some("italic")
                    || style_decl.properties.font_style.as_deref() == Some("oblique")
            });
            if needs_italic {
                ctx.push_style(StyleModifier::ITALIC_STYLE);
            }
            compile_children_with_refs(
                node_id,
                module,
                page,
                ctx,
                gir_doc,
                labels,
                refs_map,
                eq_counter,
                section_counters,
                section_number,
                footnotes,
                figure_counter,
            )?;
            if needs_italic {
                ctx.pop_style();
            }
        }
        NodeType::BlockQuote => {
            let saved_margin = ctx.margin_left;
            ctx.margin_left += Fp266::from_int(18);
            ctx.content_width = ctx.page_width - ctx.margin_left - ctx.margin_right;
            ctx.reset_x();

            let rule_x = saved_margin + Fp266::from_int(12);
            emit_helpers::emit_draw_rule(
                page,
                rule_x,
                ctx.y,
                Fp266::from_int(2),
                ctx.line_height() * 2,
            );

            compile_children_with_refs(
                node_id,
                module,
                page,
                ctx,
                gir_doc,
                labels,
                refs_map,
                eq_counter,
                section_counters,
                section_number,
                footnotes,
                figure_counter,
            )?;

            ctx.margin_left = saved_margin;
            ctx.content_width = ctx.page_width - ctx.margin_left - ctx.margin_right;
            ctx.reset_x();
            ctx.advance_y(ctx.line_height());
            maybe_new_page(page, ctx, gir_doc);
        }
        NodeType::ThematicBreak => {
            let rule_y = ctx.y - Fp266::from_int(6);
            let rule_width = ctx.page_width - ctx.margin_left - ctx.margin_right;
            emit_helpers::emit_draw_rule(
                page,
                ctx.margin_left,
                rule_y,
                rule_width,
                Fp266::from_frac(1, 2),
            );
            ctx.y = rule_y - Fp266::from_int(6);
            ctx.reset_x();
            maybe_new_page(page, ctx, gir_doc);
        }
        NodeType::PageBreak => {
            if !page.is_empty() {
                gir_doc.push_page(std::mem::replace(page, ctx.new_page()));
            }
            ctx.y = ctx.margin_top;
            ctx.reset_x();
        }
        NodeType::LineBreak => {
            ctx.advance_y(ctx.font_size);
            ctx.reset_x();
            if ctx.exceeds_page() {
                if !page.is_empty() {
                    gir_doc.push_page(std::mem::replace(page, ctx.new_page()));
                }
                ctx.y = ctx.margin_top;
            }
        }
        NodeType::Link { url, .. } => {
            let link_text = module.body.collect_text(node_id);
            let resolved = resolve_v2_references(&link_text, labels, refs_map);
            if !resolved.trim().is_empty() {
                let link_start_x = ctx.x.to_f64();
                let link_start_y = ctx.y.to_f64();
                emit_v2_paragraph_text(&resolved, page, ctx, gir_doc);
                let line_h = ctx.line_height().to_f64();
                page.links.push(ldir_ir::gir::GIRLink {
                    x: link_start_x,
                    y: link_start_y,
                    width: ctx.content_width.to_f64(),
                    height: line_h,
                    url: url.clone(),
                    destination_page: None,
                });
            }
        }
        NodeType::MathInline { content } => {
            emit_v2_math_inline(content, page, ctx)?;
        }
        NodeType::MathBlock { numbered, .. } => {
            let math_text = module.body.collect_text(node_id);
            if !math_text.trim().is_empty() {
                if *numbered {
                    *eq_counter += 1;
                    let eq_num = eq_counter.to_string();
                    if let Some(label) = &node.label {
                        labels.insert(label.clone(), eq_num.clone());
                    }
                    emit_v2_math_block(&math_text, *numbered, Some(&eq_num), page, ctx, gir_doc);
                } else {
                    emit_v2_math_block(&math_text, *numbered, None, page, ctx, gir_doc);
                }
            }
        }
        NodeType::Image { source, .. } => {
            let base_dir = module
                .header
                .source_path
                .as_ref()
                .and_then(|p| std::path::Path::new(p).parent());
            if let Some((img, w_fp, h_fp)) =
                load_and_scale_image(source, ctx.content_width, base_dir)
            {
                let image_index = gir_doc.push_image(img);
                emit_helpers::emit_move_xy(page, ctx.x, ctx.y);
                page.push(GIRCommand::new_draw_rule(
                    -1,
                    image_index as i32,
                    w_fp,
                    h_fp,
                ));
                ctx.advance_y(Fp266::from_raw(h_fp as i64));
                ctx.reset_x();
                maybe_new_page(page, ctx, gir_doc);
            }
        }
        NodeType::CodeBlock { .. } => {
            ctx.push_style(StyleModifier::MONO_STYLE);
            let code_text = module.body.collect_text(node_id);
            if !code_text.trim().is_empty() {
                emit_v2_paragraph_text(&code_text, page, ctx, gir_doc);
            }
            ctx.pop_style();
        }
        NodeType::List { .. } => {
            compile_v2_list_fallback(node_id, module, page, ctx, gir_doc)?;
        }
        NodeType::Table {
            col_specs,
            num_cols,
            ..
        } => {
            compile_v2_table_improved(node_id, module, page, ctx, gir_doc, col_specs, *num_cols)?;
        }
        NodeType::Figure { .. } => {
            emit_v2_figure(node_id, module, page, ctx, gir_doc, figure_counter)?;
        }
        NodeType::Caption => {}
        NodeType::Footnote { content } => {
            let fn_num = footnotes.len() + 1;
            footnotes.push((fn_num, content.clone()));
            let marker = to_superscript(fn_num);
            let marker_start_x = ctx.x.to_f64();
            let marker_start_y = ctx.y.to_f64();
            emit_v2_text_inline(&marker, page, ctx, gir_doc);
            page.links.push(ldir_ir::gir::GIRLink {
                x: marker_start_x,
                y: marker_start_y,
                width: ctx.content_width.to_f64(),
                height: ctx.font_size.to_f64(),
                url: format!("#footnote-{}", fn_num),
                destination_page: None,
            });
        }
        NodeType::FootnoteBlock if !footnotes.is_empty() => {
            ctx.advance_y(Fp266::from_int(12));
            if ctx.exceeds_page() {
                if !page.is_empty() {
                    gir_doc.push_page(std::mem::replace(page, ctx.new_page()));
                }
                ctx.y = ctx.margin_top;
            }
            let rule_width = ctx.content_width.div(Fp266::from_int(4));
            let rule_x = ctx.margin_left + (ctx.content_width - rule_width).div(Fp266::from_int(2));
            emit_helpers::emit_draw_rule(page, rule_x, ctx.y, rule_width, Fp266::from_int(1));
            ctx.advance_y(Fp266::from_int(6));
            let saved_font_size = ctx.font_size;
            ctx.font_size = Fp266::from_int(8);
            for (num, content) in footnotes.iter() {
                let marker = to_superscript(*num);
                let entry = format!("{} {}", marker, content);
                emit_v2_paragraph_text(&entry, page, ctx, gir_doc);
                ctx.advance_y(Fp266::from_int(2));
            }
            ctx.font_size = saved_font_size;
            footnotes.clear();
        }
        NodeType::FootnoteBlock => {}
        NodeType::TableOfContents { max_depth } => {
            ctx.advance_y(Fp266::from_int(12));
            emit_helpers::emit_move_xy(page, ctx.x, ctx.y);

            let saved_font_size = ctx.font_size;
            ctx.font_size = Fp266::from_int(16);
            emit_v2_paragraph_text("Contents", page, ctx, gir_doc);
            ctx.advance_y(ctx.line_height());
            ctx.reset_x();
            maybe_new_page(page, ctx, gir_doc);

            ctx.font_size = Fp266::from_int(10);

            for node in module.body.iter() {
                let level = match &node.node_type {
                    NodeType::Chapter => 1,
                    NodeType::Section => 2,
                    NodeType::Subsection => 3,
                    NodeType::Subsubsection => 4,
                    _ => continue,
                };

                if level > *max_depth as usize {
                    continue;
                }

                if node.label.is_none() {
                    continue;
                }

                let text = module.body.collect_text(node.id);
                if text.trim().is_empty() {
                    continue;
                }

                let number = node.counter.as_deref().unwrap_or("");
                let indent = Fp266::from_int((level as i32 - 1) * 20);
                ctx.advance_x(indent);
                emit_helpers::emit_move_xy(page, ctx.x, ctx.y);

                let entry_text = if number.is_empty() {
                    text.trim().to_string()
                } else {
                    format!("{}  {}", number, text.trim())
                };

                emit_v2_paragraph_text(&entry_text, page, ctx, gir_doc);

                ctx.advance_y(Fp266::from_int(2));
                if ctx.exceeds_page() {
                    if !page.is_empty() {
                        gir_doc.push_page(std::mem::replace(page, ctx.new_page()));
                    }
                    ctx.y = ctx.margin_top;
                }
                ctx.reset_x();
            }

            ctx.font_size = saved_font_size;
        }
        NodeType::Group => {
            compile_children_with_refs(
                node_id,
                module,
                page,
                ctx,
                gir_doc,
                labels,
                refs_map,
                eq_counter,
                section_counters,
                section_number,
                footnotes,
                figure_counter,
            )?;
        }
        NodeType::ListItem => {
            compile_children_with_refs(
                node_id,
                module,
                page,
                ctx,
                gir_doc,
                labels,
                refs_map,
                eq_counter,
                section_counters,
                section_number,
                footnotes,
                figure_counter,
            )?;
        }
        NodeType::TableRow { .. } | NodeType::TableCell { .. } => {
            compile_children_with_refs(
                node_id,
                module,
                page,
                ctx,
                gir_doc,
                labels,
                refs_map,
                eq_counter,
                section_counters,
                section_number,
                footnotes,
                figure_counter,
            )?;
        }
        NodeType::Citation { keys, .. } => {
            let mut parts: Vec<String> = Vec::new();
            for key in keys {
                parts.push(format!("[{}]", key));
            }
            let cite_text = parts.join(", ");
            emit_v2_text_inline(&cite_text, page, ctx, gir_doc);
        }
        NodeType::Reference { label } => {
            let resolved = labels
                .get(label)
                .cloned()
                .unwrap_or_else(|| format!("[{}]", label));
            emit_v2_text_inline(&resolved, page, ctx, gir_doc);
        }
        NodeType::Label { .. } => {}
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn compile_v2_node_with_bib(
    node_id: u32,
    module: &SIRModuleV2,
    page: &mut GIRPage,
    ctx: &mut CompileContext,
    gir_doc: &mut GIRDocument,
    labels: &mut IndexMap<String, String>,
    refs_map: &IndexMap<String, String>,
    eq_counter: &mut u32,
    section_counters: &mut IndexMap<u8, u32>,
    section_number: &mut Vec<u32>,
    bibliography: &HashMap<String, BibEntry>,
    cite_counter: &mut u32,
    cite_numbers: &mut IndexMap<String, u32>,
    footnotes: &mut Vec<(usize, String)>,
    figure_counter: &mut usize,
) -> Result<()> {
    let node = module.body.get(node_id).ok_or(crate::error::LdirError {
        kind: crate::error::ErrorKind::Compile(
            crate::error::CompileErrorKind::UnsupportedInstruction { entity_id: node_id },
        ),
        entity_id: Some(node_id),
        byte_offset: None,
    })?;

    if let NodeType::Citation { keys, .. } = &node.node_type {
        let mut parts: Vec<String> = Vec::new();
        for key in keys {
            if !cite_numbers.contains_key(key) {
                *cite_counter += 1;
                cite_numbers.insert(key.clone(), *cite_counter);
            }
            if let Some(&num) = cite_numbers.get(key) {
                parts.push(format!("[{}]", num));
            } else {
                parts.push(format!("[{}]", key));
            }
        }
        let cite_text = parts.join(", ");
        emit_v2_text_inline(&cite_text, page, ctx, gir_doc);
        return Ok(());
    }

    match &node.node_type {
        NodeType::Document => {
            compile_children_with_bib(
                node_id,
                module,
                page,
                ctx,
                gir_doc,
                labels,
                refs_map,
                eq_counter,
                section_counters,
                section_number,
                bibliography,
                cite_counter,
                cite_numbers,
                footnotes,
                figure_counter,
            )?;
        }
        NodeType::Paragraph => {
            emit_v2_paragraph_with_bib(
                node_id,
                module,
                page,
                ctx,
                gir_doc,
                labels,
                refs_map,
                bibliography,
                cite_counter,
                cite_numbers,
            )?;
        }
        NodeType::Chapter => {
            increment_section_counter(1, section_counters, section_number);
            let num = section_number_string(section_number);
            if let Some(label) = &node.label {
                labels.insert(label.clone(), num.clone());
            }
            ctx.chapter_title = module.body.collect_text(node_id);
            emit_v2_heading_with_number(node_id, module, page, ctx, gir_doc, 24, &num)?;
        }
        NodeType::Section => {
            increment_section_counter(2, section_counters, section_number);
            let num = section_number_string(section_number);
            if let Some(label) = &node.label {
                labels.insert(label.clone(), num.clone());
            }
            ctx.section_title = module.body.collect_text(node_id);
            emit_v2_heading_with_number(node_id, module, page, ctx, gir_doc, 20, &num)?;
        }
        NodeType::Subsection => {
            increment_section_counter(3, section_counters, section_number);
            let num = section_number_string(section_number);
            if let Some(label) = &node.label {
                labels.insert(label.clone(), num.clone());
            }
            emit_v2_heading_with_number(node_id, module, page, ctx, gir_doc, 16, &num)?;
        }
        NodeType::Subsubsection => {
            increment_section_counter(4, section_counters, section_number);
            let num = section_number_string(section_number);
            if let Some(label) = &node.label {
                labels.insert(label.clone(), num.clone());
            }
            emit_v2_heading_with_number(node_id, module, page, ctx, gir_doc, 14, &num)?;
        }
        NodeType::Part => emit_v2_heading(node_id, module, page, ctx, gir_doc, 26)?,
        NodeType::Text { .. } => {
            let text = module.body.collect_text(node_id);
            let resolved = resolve_v2_references(&text, labels, refs_map);
            if !resolved.trim().is_empty() {
                emit_v2_text_inline(&resolved, page, ctx, gir_doc);
            }
        }
        NodeType::Bold => {
            ctx.push_style(StyleModifier::BOLD_STYLE);
            compile_children_with_bib(
                node_id,
                module,
                page,
                ctx,
                gir_doc,
                labels,
                refs_map,
                eq_counter,
                section_counters,
                section_number,
                bibliography,
                cite_counter,
                cite_numbers,
                footnotes,
                figure_counter,
            )?;
            ctx.pop_style();
        }
        NodeType::Italic => {
            ctx.push_style(StyleModifier::ITALIC_STYLE);
            compile_children_with_bib(
                node_id,
                module,
                page,
                ctx,
                gir_doc,
                labels,
                refs_map,
                eq_counter,
                section_counters,
                section_number,
                bibliography,
                cite_counter,
                cite_numbers,
                footnotes,
                figure_counter,
            )?;
            ctx.pop_style();
        }
        NodeType::Mono => {
            ctx.push_style(StyleModifier::MONO_STYLE);
            compile_children_with_bib(
                node_id,
                module,
                page,
                ctx,
                gir_doc,
                labels,
                refs_map,
                eq_counter,
                section_counters,
                section_number,
                bibliography,
                cite_counter,
                cite_numbers,
                footnotes,
                figure_counter,
            )?;
            ctx.pop_style();
        }
        NodeType::Underline => {
            ctx.push_style(StyleModifier(StyleModifier::UNDERLINE));
            compile_children_with_bib(
                node_id,
                module,
                page,
                ctx,
                gir_doc,
                labels,
                refs_map,
                eq_counter,
                section_counters,
                section_number,
                bibliography,
                cite_counter,
                cite_numbers,
                footnotes,
                figure_counter,
            )?;
            ctx.pop_style();
        }
        NodeType::Strikethrough => {
            ctx.push_style(StyleModifier(StyleModifier::STRIKE));
            compile_children_with_bib(
                node_id,
                module,
                page,
                ctx,
                gir_doc,
                labels,
                refs_map,
                eq_counter,
                section_counters,
                section_number,
                bibliography,
                cite_counter,
                cite_numbers,
                footnotes,
                figure_counter,
            )?;
            ctx.pop_style();
        }
        NodeType::SmallCaps => {
            ctx.push_style(StyleModifier(StyleModifier::SMALL_CAPS));
            compile_children_with_bib(
                node_id,
                module,
                page,
                ctx,
                gir_doc,
                labels,
                refs_map,
                eq_counter,
                section_counters,
                section_number,
                bibliography,
                cite_counter,
                cite_numbers,
                footnotes,
                figure_counter,
            )?;
            ctx.pop_style();
        }
        NodeType::Styled { style_name } => {
            let needs_italic = module.styles.find(style_name).is_some_and(|style_decl| {
                style_decl.properties.font_style.as_deref() == Some("italic")
                    || style_decl.properties.font_style.as_deref() == Some("oblique")
            });
            if needs_italic {
                ctx.push_style(StyleModifier::ITALIC_STYLE);
            }
            compile_children_with_bib(
                node_id,
                module,
                page,
                ctx,
                gir_doc,
                labels,
                refs_map,
                eq_counter,
                section_counters,
                section_number,
                bibliography,
                cite_counter,
                cite_numbers,
                footnotes,
                figure_counter,
            )?;
            if needs_italic {
                ctx.pop_style();
            }
        }
        NodeType::BlockQuote => {
            let saved_margin = ctx.margin_left;
            ctx.margin_left += Fp266::from_int(18);
            ctx.content_width = ctx.page_width - ctx.margin_left - ctx.margin_right;
            ctx.reset_x();
            let rule_x = saved_margin + Fp266::from_int(12);
            emit_helpers::emit_draw_rule(
                page,
                rule_x,
                ctx.y,
                Fp266::from_int(2),
                ctx.line_height() * 2,
            );
            compile_children_with_bib(
                node_id,
                module,
                page,
                ctx,
                gir_doc,
                labels,
                refs_map,
                eq_counter,
                section_counters,
                section_number,
                bibliography,
                cite_counter,
                cite_numbers,
                footnotes,
                figure_counter,
            )?;
            ctx.margin_left = saved_margin;
            ctx.content_width = ctx.page_width - ctx.margin_left - ctx.margin_right;
            ctx.reset_x();
            ctx.advance_y(ctx.line_height());
            maybe_new_page(page, ctx, gir_doc);
        }
        NodeType::ThematicBreak => {
            let rule_y = ctx.y - Fp266::from_int(6);
            let rule_width = ctx.page_width - ctx.margin_left - ctx.margin_right;
            emit_helpers::emit_draw_rule(
                page,
                ctx.margin_left,
                rule_y,
                rule_width,
                Fp266::from_frac(1, 2),
            );
            ctx.y = rule_y - Fp266::from_int(6);
            ctx.reset_x();
            maybe_new_page(page, ctx, gir_doc);
        }
        NodeType::PageBreak => {
            if !page.is_empty() {
                gir_doc.push_page(std::mem::replace(page, ctx.new_page()));
            }
            ctx.y = ctx.margin_top;
            ctx.reset_x();
        }
        NodeType::LineBreak => {
            ctx.advance_y(ctx.font_size);
            ctx.reset_x();
            if ctx.exceeds_page() {
                if !page.is_empty() {
                    gir_doc.push_page(std::mem::replace(page, ctx.new_page()));
                }
                ctx.y = ctx.margin_top;
            }
        }
        NodeType::Link { url, .. } => {
            let link_text = module.body.collect_text(node_id);
            let resolved = resolve_v2_references(&link_text, labels, refs_map);
            if !resolved.trim().is_empty() {
                let link_start_x = ctx.x.to_f64();
                let link_start_y = ctx.y.to_f64();
                emit_v2_paragraph_text(&resolved, page, ctx, gir_doc);
                let line_h = ctx.line_height().to_f64();
                page.links.push(ldir_ir::gir::GIRLink {
                    x: link_start_x,
                    y: link_start_y,
                    width: ctx.content_width.to_f64(),
                    height: line_h,
                    url: url.clone(),
                    destination_page: None,
                });
            }
        }
        NodeType::MathInline { content } => {
            emit_v2_math_inline(content, page, ctx)?;
        }
        NodeType::MathBlock { numbered, .. } => {
            let math_text = module.body.collect_text(node_id);
            if !math_text.trim().is_empty() {
                if *numbered {
                    *eq_counter += 1;
                    let eq_num = eq_counter.to_string();
                    if let Some(label) = &node.label {
                        labels.insert(label.clone(), eq_num.clone());
                    }
                    emit_v2_math_block(&math_text, *numbered, Some(&eq_num), page, ctx, gir_doc);
                } else {
                    emit_v2_math_block(&math_text, *numbered, None, page, ctx, gir_doc);
                }
            }
        }
        NodeType::Image { source, .. } => {
            let base_dir = module
                .header
                .source_path
                .as_ref()
                .and_then(|p| std::path::Path::new(p).parent());
            if let Some((img, w_fp, h_fp)) =
                load_and_scale_image(source, ctx.content_width, base_dir)
            {
                let image_index = gir_doc.push_image(img);
                emit_helpers::emit_move_xy(page, ctx.x, ctx.y);
                page.push(GIRCommand::new_draw_rule(
                    -1,
                    image_index as i32,
                    w_fp,
                    h_fp,
                ));
                ctx.advance_y(Fp266::from_raw(h_fp as i64));
                ctx.reset_x();
                maybe_new_page(page, ctx, gir_doc);
            }
        }
        NodeType::CodeBlock { .. } => {
            ctx.push_style(StyleModifier::MONO_STYLE);
            let code_text = module.body.collect_text(node_id);
            if !code_text.trim().is_empty() {
                emit_v2_paragraph_text(&code_text, page, ctx, gir_doc);
            }
            ctx.pop_style();
        }
        NodeType::List { .. } => {
            compile_v2_list_fallback(node_id, module, page, ctx, gir_doc)?;
        }
        NodeType::Table {
            col_specs,
            num_cols,
            ..
        } => {
            compile_v2_table_improved(node_id, module, page, ctx, gir_doc, col_specs, *num_cols)?;
        }
        NodeType::Figure { .. } => {
            emit_v2_figure(node_id, module, page, ctx, gir_doc, figure_counter)?;
        }
        NodeType::Caption => {}
        NodeType::Footnote { content } => {
            let fn_num = footnotes.len() + 1;
            footnotes.push((fn_num, content.clone()));
            let marker = to_superscript(fn_num);
            let marker_start_x = ctx.x.to_f64();
            let marker_start_y = ctx.y.to_f64();
            emit_v2_text_inline(&marker, page, ctx, gir_doc);
            page.links.push(ldir_ir::gir::GIRLink {
                x: marker_start_x,
                y: marker_start_y,
                width: ctx.content_width.to_f64(),
                height: ctx.font_size.to_f64(),
                url: format!("#footnote-{}", fn_num),
                destination_page: None,
            });
        }
        NodeType::FootnoteBlock if !footnotes.is_empty() => {
            ctx.advance_y(Fp266::from_int(12));
            if ctx.exceeds_page() {
                if !page.is_empty() {
                    gir_doc.push_page(std::mem::replace(page, ctx.new_page()));
                }
                ctx.y = ctx.margin_top;
            }
            let rule_width = ctx.content_width.div(Fp266::from_int(4));
            let rule_x = ctx.margin_left + (ctx.content_width - rule_width).div(Fp266::from_int(2));
            emit_helpers::emit_draw_rule(page, rule_x, ctx.y, rule_width, Fp266::from_int(1));
            ctx.advance_y(Fp266::from_int(6));
            let saved_font_size = ctx.font_size;
            ctx.font_size = Fp266::from_int(8);
            for (num, content) in footnotes.iter() {
                let marker = to_superscript(*num);
                let entry = format!("{} {}", marker, content);
                emit_v2_paragraph_text(&entry, page, ctx, gir_doc);
                ctx.advance_y(Fp266::from_int(2));
            }
            ctx.font_size = saved_font_size;
            footnotes.clear();
        }
        NodeType::FootnoteBlock => {}
        NodeType::TableOfContents { max_depth } => {
            ctx.advance_y(Fp266::from_int(12));
            emit_helpers::emit_move_xy(page, ctx.x, ctx.y);

            let saved_font_size = ctx.font_size;
            ctx.font_size = Fp266::from_int(16);
            emit_v2_paragraph_text("Contents", page, ctx, gir_doc);
            ctx.advance_y(ctx.line_height());
            ctx.reset_x();
            maybe_new_page(page, ctx, gir_doc);

            ctx.font_size = Fp266::from_int(10);

            for node in module.body.iter() {
                let level = match &node.node_type {
                    NodeType::Chapter => 1,
                    NodeType::Section => 2,
                    NodeType::Subsection => 3,
                    NodeType::Subsubsection => 4,
                    _ => continue,
                };

                if level > *max_depth as usize {
                    continue;
                }

                if node.label.is_none() {
                    continue;
                }

                let text = module.body.collect_text(node.id);
                if text.trim().is_empty() {
                    continue;
                }

                let number = node.counter.as_deref().unwrap_or("");
                let indent = Fp266::from_int((level as i32 - 1) * 20);
                ctx.advance_x(indent);
                emit_helpers::emit_move_xy(page, ctx.x, ctx.y);

                let entry_text = if number.is_empty() {
                    text.trim().to_string()
                } else {
                    format!("{}  {}", number, text.trim())
                };

                emit_v2_paragraph_text(&entry_text, page, ctx, gir_doc);

                ctx.advance_y(Fp266::from_int(2));
                if ctx.exceeds_page() {
                    if !page.is_empty() {
                        gir_doc.push_page(std::mem::replace(page, ctx.new_page()));
                    }
                    ctx.y = ctx.margin_top;
                }
                ctx.reset_x();
            }

            ctx.font_size = saved_font_size;
        }
        NodeType::Group => {
            compile_children_with_bib(
                node_id,
                module,
                page,
                ctx,
                gir_doc,
                labels,
                refs_map,
                eq_counter,
                section_counters,
                section_number,
                bibliography,
                cite_counter,
                cite_numbers,
                footnotes,
                figure_counter,
            )?;
        }
        NodeType::ListItem => {
            compile_children_with_bib(
                node_id,
                module,
                page,
                ctx,
                gir_doc,
                labels,
                refs_map,
                eq_counter,
                section_counters,
                section_number,
                bibliography,
                cite_counter,
                cite_numbers,
                footnotes,
                figure_counter,
            )?;
        }
        NodeType::TableRow { .. } | NodeType::TableCell { .. } => {
            compile_children_with_bib(
                node_id,
                module,
                page,
                ctx,
                gir_doc,
                labels,
                refs_map,
                eq_counter,
                section_counters,
                section_number,
                bibliography,
                cite_counter,
                cite_numbers,
                footnotes,
                figure_counter,
            )?;
        }
        _ => {}
    }

    Ok(())
}

fn compile_children(
    node_id: u32,
    module: &SIRModuleV2,
    page: &mut GIRPage,
    ctx: &mut CompileContext,
    gir_doc: &mut GIRDocument,
) -> Result<()> {
    let node = match module.body.get(node_id) {
        Some(n) => n,
        None => return Ok(()),
    };
    let mut labels = IndexMap::new();
    let refs_map = IndexMap::new();
    let mut eq_counter: u32 = 0;
    let mut section_counters: IndexMap<u8, u32> = IndexMap::new();
    let mut section_number: Vec<u32> = Vec::new();
    let mut footnotes: Vec<(usize, String)> = Vec::new();
    let mut figure_counter: usize = 0;
    for &child_id in &node.child_ids {
        compile_v2_node(
            child_id,
            module,
            page,
            ctx,
            gir_doc,
            &mut labels,
            &refs_map,
            &mut eq_counter,
            &mut section_counters,
            &mut section_number,
            &mut footnotes,
            &mut figure_counter,
        )?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn compile_children_with_refs(
    node_id: u32,
    module: &SIRModuleV2,
    page: &mut GIRPage,
    ctx: &mut CompileContext,
    gir_doc: &mut GIRDocument,
    labels: &mut IndexMap<String, String>,
    refs_map: &IndexMap<String, String>,
    eq_counter: &mut u32,
    section_counters: &mut IndexMap<u8, u32>,
    section_number: &mut Vec<u32>,
    footnotes: &mut Vec<(usize, String)>,
    figure_counter: &mut usize,
) -> Result<()> {
    let node = match module.body.get(node_id) {
        Some(n) => n,
        None => return Ok(()),
    };
    for &child_id in &node.child_ids {
        compile_v2_node(
            child_id,
            module,
            page,
            ctx,
            gir_doc,
            labels,
            refs_map,
            eq_counter,
            section_counters,
            section_number,
            footnotes,
            figure_counter,
        )?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn compile_children_with_bib(
    node_id: u32,
    module: &SIRModuleV2,
    page: &mut GIRPage,
    ctx: &mut CompileContext,
    gir_doc: &mut GIRDocument,
    labels: &mut IndexMap<String, String>,
    refs_map: &IndexMap<String, String>,
    eq_counter: &mut u32,
    section_counters: &mut IndexMap<u8, u32>,
    section_number: &mut Vec<u32>,
    bibliography: &HashMap<String, BibEntry>,
    cite_counter: &mut u32,
    cite_numbers: &mut IndexMap<String, u32>,
    footnotes: &mut Vec<(usize, String)>,
    figure_counter: &mut usize,
) -> Result<()> {
    let node = match module.body.get(node_id) {
        Some(n) => n,
        None => return Ok(()),
    };
    for &child_id in &node.child_ids {
        compile_v2_node_with_bib(
            child_id,
            module,
            page,
            ctx,
            gir_doc,
            labels,
            refs_map,
            eq_counter,
            section_counters,
            section_number,
            bibliography,
            cite_counter,
            cite_numbers,
            footnotes,
            figure_counter,
        )?;
    }
    Ok(())
}

fn collect_v2_labels(module: &SIRModuleV2) -> IndexMap<String, String> {
    let mut labels = IndexMap::new();

    for node in module.body.iter() {
        if let Some(label) = &node.label {
            match &node.node_type {
                NodeType::Section => {
                    labels.insert(label.clone(), node.counter.clone().unwrap_or_default());
                }
                NodeType::Subsection => {
                    labels.insert(label.clone(), node.counter.clone().unwrap_or_default());
                }
                NodeType::Subsubsection => {
                    labels.insert(label.clone(), node.counter.clone().unwrap_or_default());
                }
                NodeType::Chapter => {
                    labels.insert(label.clone(), node.counter.clone().unwrap_or_default());
                }
                NodeType::MathBlock { numbered: true, .. } => {
                    labels.insert(label.clone(), node.counter.clone().unwrap_or_default());
                }
                _ => {
                    labels.insert(label.clone(), String::new());
                }
            }
        }
    }

    for (label, info) in &module.annotations.labels {
        if !labels.contains_key(label) {
            labels.insert(label.clone(), String::new());
            match info.category {
                LabelCategory::Section => {
                    labels.insert(label.clone(), String::new());
                }
                LabelCategory::Equation => {
                    labels.insert(label.clone(), String::new());
                }
                LabelCategory::Figure => {
                    labels.insert(label.clone(), String::new());
                }
                _ => {}
            }
        }
    }

    labels
}

fn collect_v2_refs(
    module: &SIRModuleV2,
    labels: &IndexMap<String, String>,
) -> IndexMap<String, String> {
    let mut refs_map = IndexMap::new();

    for xref in &module.annotations.refs {
        if let Some(number) = labels.get(&xref.label) {
            refs_map.insert(xref.label.clone(), number.clone());
        }
    }

    refs_map
}

fn resolve_v2_references(
    text: &str,
    labels: &IndexMap<String, String>,
    refs_map: &IndexMap<String, String>,
) -> String {
    let mut numbers = IndexMap::new();
    for (k, v) in labels.iter().chain(refs_map.iter()) {
        numbers.entry(k.clone()).or_insert_with(|| v.clone());
    }
    cross_ref::resolve_text_references(text, &numbers, &IndexMap::new())
}

fn increment_section_counter(
    level: u8,
    section_counters: &mut IndexMap<u8, u32>,
    section_number: &mut Vec<u32>,
) {
    *section_counters.entry(level).or_insert(0) += 1;
    let count = section_counters[&level];

    while section_number.len() > level as usize {
        section_number.pop();
    }
    while section_number.len() < level as usize {
        section_number.push(0);
    }
    section_number[level as usize - 1] = count;
}

fn section_number_string(section_number: &[u32]) -> String {
    section_number
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(".")
}

fn emit_v2_heading_with_number(
    node_id: u32,
    module: &SIRModuleV2,
    page: &mut GIRPage,
    ctx: &mut CompileContext,
    gir_doc: &mut GIRDocument,
    font_size_pt: i32,
    number: &str,
) -> Result<()> {
    let saved_font_size = ctx.font_size;
    ctx.font_size = Fp266::from_int(font_size_pt);

    let empty_labels = IndexMap::new();
    let empty_refs = IndexMap::new();
    let mut runs = collect_styled_runs(
        node_id,
        module,
        StyleModifier::EMPTY,
        &empty_labels,
        &empty_refs,
    );
    if !number.is_empty() {
        runs.insert(0, (format!("{} ", number), StyleModifier::EMPTY));
    }
    if !runs.iter().all(|(t, _)| t.trim().is_empty()) {
        emit_v2_styled_paragraph(&runs, page, ctx, gir_doc)?;
    }

    ctx.font_size = saved_font_size;
    ctx.next_para_is_drop_cap = true;

    ctx.advance_y(ctx.line_height());
    ctx.reset_x();
    maybe_new_page(page, ctx, gir_doc);

    Ok(())
}

fn emit_bibliography_page(
    _module: &SIRModuleV2,
    bibliography: &HashMap<String, BibEntry>,
    cite_numbers: &IndexMap<String, u32>,
    ctx: &mut CompileContext,
    page: &mut GIRPage,
    gir_doc: &mut GIRDocument,
) {
    if cite_numbers.is_empty() || bibliography.is_empty() {
        return;
    }

    if !page.is_empty() {
        gir_doc.push_page(std::mem::replace(page, ctx.new_page()));
    }

    let saved_font_size = ctx.font_size;
    ctx.font_size = Fp266::from_int(16);
    emit_v2_paragraph_text("References", page, ctx, gir_doc);
    ctx.font_size = saved_font_size;
    ctx.advance_y(ctx.line_height());
    ctx.reset_x();

    ctx.font_size = Fp266::from_int(10);

    let mut sorted: Vec<(&String, &u32)> = cite_numbers.iter().collect();
    sorted.sort_by_key(|&(_, &num)| num);

    for (key, num) in &sorted {
        if let Some(entry) = bibliography.get(*key) {
            let formatted = format_citation_ieee(entry);
            let ref_text = format!("[{}] {}", *num, formatted);
            emit_v2_paragraph_text(&ref_text, page, ctx, gir_doc);
            ctx.advance_y(Fp266::from_int(6));
            ctx.reset_x();
        }
    }

    ctx.font_size = saved_font_size;
}

fn emit_v2_heading(
    node_id: u32,
    module: &SIRModuleV2,
    page: &mut GIRPage,
    ctx: &mut CompileContext,
    gir_doc: &mut GIRDocument,
    font_size_pt: i32,
) -> Result<()> {
    let saved_font_size = ctx.font_size;
    ctx.font_size = Fp266::from_int(font_size_pt);

    let empty_labels = IndexMap::new();
    let empty_refs = IndexMap::new();
    let runs = collect_styled_runs(
        node_id,
        module,
        StyleModifier::EMPTY,
        &empty_labels,
        &empty_refs,
    );
    if !runs.iter().all(|(t, _)| t.trim().is_empty()) {
        emit_v2_styled_paragraph(&runs, page, ctx, gir_doc)?;
    }

    ctx.font_size = saved_font_size;
    ctx.next_para_is_drop_cap = true;

    ctx.advance_y(ctx.line_height());
    ctx.reset_x();
    maybe_new_page(page, ctx, gir_doc);

    Ok(())
}

fn collect_styled_runs(
    node_id: u32,
    module: &SIRModuleV2,
    style_modifier: StyleModifier,
    labels: &IndexMap<String, String>,
    refs_map: &IndexMap<String, String>,
) -> Vec<(String, StyleModifier)> {
    let node = match module.body.get(node_id) {
        Some(n) => n,
        None => return Vec::new(),
    };

    let mut runs: Vec<(String, StyleModifier)> = Vec::with_capacity(8);
    let mut current_text = String::with_capacity(256);
    let current_style = style_modifier;

    for &child_id in &node.child_ids {
        let child = match module.body.get(child_id) {
            Some(n) => n,
            None => continue,
        };

        match &child.node_type {
            NodeType::Text { content } => {
                let resolved = resolve_v2_references(content, labels, refs_map);
                current_text.push_str(&resolved);
            }
            NodeType::Bold => {
                if !current_text.is_empty() {
                    runs.push((std::mem::take(&mut current_text), current_style));
                }
                let bold_runs = collect_styled_runs(
                    child_id,
                    module,
                    StyleModifier(current_style.0 | StyleModifier::BOLD),
                    labels,
                    refs_map,
                );
                runs.extend(bold_runs);
            }
            NodeType::Italic => {
                if !current_text.is_empty() {
                    runs.push((std::mem::take(&mut current_text), current_style));
                }
                let italic_runs = collect_styled_runs(
                    child_id,
                    module,
                    StyleModifier(current_style.0 | StyleModifier::ITALIC),
                    labels,
                    refs_map,
                );
                runs.extend(italic_runs);
            }
            NodeType::Mono => {
                if !current_text.is_empty() {
                    runs.push((std::mem::take(&mut current_text), current_style));
                }
                let mono_runs = collect_styled_runs(
                    child_id,
                    module,
                    StyleModifier(current_style.0 | StyleModifier::MONO),
                    labels,
                    refs_map,
                );
                runs.extend(mono_runs);
            }
            NodeType::Underline => {
                if !current_text.is_empty() {
                    runs.push((std::mem::take(&mut current_text), current_style));
                }
                let ul_runs = collect_styled_runs(
                    child_id,
                    module,
                    StyleModifier(current_style.0 | StyleModifier::UNDERLINE),
                    labels,
                    refs_map,
                );
                runs.extend(ul_runs);
            }
            NodeType::Strikethrough => {
                if !current_text.is_empty() {
                    runs.push((std::mem::take(&mut current_text), current_style));
                }
                let strike_runs = collect_styled_runs(
                    child_id,
                    module,
                    StyleModifier(current_style.0 | StyleModifier::STRIKE),
                    labels,
                    refs_map,
                );
                runs.extend(strike_runs);
            }
            NodeType::SmallCaps => {
                if !current_text.is_empty() {
                    runs.push((std::mem::take(&mut current_text), current_style));
                }
                let sc_runs = collect_styled_runs(
                    child_id,
                    module,
                    StyleModifier(current_style.0 | StyleModifier::SMALL_CAPS),
                    labels,
                    refs_map,
                );
                runs.extend(sc_runs);
            }
            NodeType::Styled { style_name } => {
                if !current_text.is_empty() {
                    runs.push((std::mem::take(&mut current_text), current_style));
                }
                let mut child_style = current_style;
                if let Some(style_decl) = module.styles.find(style_name)
                    && (style_decl.properties.font_style.as_deref() == Some("italic")
                        || style_decl.properties.font_style.as_deref() == Some("oblique"))
                {
                    child_style = StyleModifier(child_style.0 | StyleModifier::ITALIC);
                }
                let styled_runs =
                    collect_styled_runs(child_id, module, child_style, labels, refs_map);
                runs.extend(styled_runs);
            }
            NodeType::MathInline { content } => {
                current_text.push_str(content);
            }
            NodeType::Link { url: _, title: _ } => {
                let link_runs =
                    collect_styled_runs(child_id, module, current_style, labels, refs_map);
                for (text, _) in &link_runs {
                    current_text.push_str(text);
                }
            }
            NodeType::Citation { keys, .. } => {
                let mut parts: Vec<String> = Vec::new();
                for key in keys {
                    parts.push(format!("[{}]", key));
                }
                current_text.push_str(&parts.join(", "));
            }
            _ => {
                let sub_runs =
                    collect_styled_runs(child_id, module, current_style, labels, refs_map);
                for (text, _) in &sub_runs {
                    current_text.push_str(text);
                }
            }
        }
    }

    if !current_text.is_empty() {
        runs.push((current_text, current_style));
    }

    runs
}

fn style_to_font_id(style: StyleModifier) -> u32 {
    if style.contains(StyleModifier::MONO) {
        return FONT_ID_MONO;
    }
    let bold = style.contains(StyleModifier::BOLD);
    let italic = style.contains(StyleModifier::ITALIC);
    match (bold, italic) {
        (false, false) => FONT_ID_REGULAR,
        (true, false) => FONT_ID_BOLD,
        (false, true) => FONT_ID_ITALIC,
        (true, true) => FONT_ID_BOLD_ITALIC,
    }
}

fn emit_v2_styled_paragraph(
    runs: &[(String, StyleModifier)],
    page: &mut GIRPage,
    ctx: &mut CompileContext,
    gir_doc: &mut GIRDocument,
) -> Result<()> {
    if runs.is_empty() {
        return Ok(());
    }

    ctx.bump.reset();

    let estimated_text_len: usize = runs.iter().map(|(t, _)| t.len()).sum();
    let mut all_text = String::with_capacity(estimated_text_len);
    let mut all_glyphs: Vec<crate::shaping::ShapedGlyph> = Vec::with_capacity(estimated_text_len);
    let mut all_font_ids: Vec<u32> = Vec::with_capacity(estimated_text_len);

    for (text, style) in runs {
        if text.is_empty() {
            continue;
        }

        let font_id = style_to_font_id(*style);

        let font_data_for_style = ctx
            .font_data_variants
            .get(font_id as usize)
            .and_then(|opt| opt.as_ref())
            .or(ctx.font_data.as_ref());

        let font_size = ctx.font_size();
        let byte_offset = all_text.len();
        all_text.push_str(text);

        let shaped: std::sync::Arc<_> = if let Some(data) = font_data_for_style {
            crate::shaping::shape_text_cached(&ctx.shape_cache, data, text, font_size, font_id)
        } else {
            std::sync::Arc::new(crate::shaping::fast_path::shape_ascii(
                text, font_size, font_id,
            ))
        };

        for g in &shaped.glyphs {
            let mut g = *g;
            g.cluster_id = g.cluster_id.saturating_add(byte_offset as u32);
            all_glyphs.push(g);
            all_font_ids.push(font_id);
        }
    }

    if all_glyphs.is_empty() {
        return Ok(());
    }

    let text_bytes = all_text.as_bytes();
    let n = all_glyphs.len();
    let content_width = ctx.content_width;

    let line_ranges = {
        let items: BumpVec<'_, LineBreakItem> = BumpVec::from_iter_in(
            all_glyphs.iter().map(|g| {
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

        let result = if is_cjk_text(&all_text) {
            let cjk_items = insert_cjk_breaks(&all_text, &items, &ctx.bump);
            linebreak(&cjk_items, &options)
        } else {
            linebreak(&items, &options)
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
        ranges
    };

    for (line_idx, &(start, end)) in line_ranges.iter().enumerate() {
        emit_helpers::emit_move_xy(page, ctx.x, ctx.y);

        let is_last_line = line_idx == line_ranges.len() - 1;
        let line_glyphs = &all_glyphs[start..end];

        let justified = justify::justify_line(line_glyphs, text_bytes, content_width, is_last_line);
        let line_font_ids = &all_font_ids[start..start + justified.len()];

        let mut current_font_id: i32 = -1;
        for (jg, &fid) in justified.iter().zip(line_font_ids.iter()) {
            if fid as i32 != current_font_id {
                emit_helpers::emit_set_font(page, fid as i32);
                current_font_id = fid as i32;
            }
            page.push(GIRCommand::new_put_glyph(jg.glyph_id as i32, jg.x_advance));
        }

        ctx.reset_x();
        ctx.advance_y(ctx.line_height());

        if ctx.exceeds_page() {
            if !page.is_empty() {
                gir_doc.push_page(std::mem::replace(page, ctx.new_page()));
            }
            ctx.y = ctx.margin_top;
        }
    }

    Ok(())
}

fn emit_v2_paragraph(
    node_id: u32,
    module: &SIRModuleV2,
    page: &mut GIRPage,
    ctx: &mut CompileContext,
    gir_doc: &mut GIRDocument,
    labels: &IndexMap<String, String>,
    refs_map: &IndexMap<String, String>,
) -> Result<()> {
    let runs = collect_styled_runs(node_id, module, StyleModifier::EMPTY, labels, refs_map);
    if runs.iter().all(|(t, _)| t.trim().is_empty()) {
        return Ok(());
    }

    let style_name = module.body.get(node_id).and_then(|n| n.style.as_deref());
    let saved_font_size = ctx.font_size;

    if let Some(name) = style_name
        && let Some(decl) = module.styles.find(name)
        && let Some(Dimension::Pt(size)) = &decl.properties.font_size
    {
        ctx.font_size = Fp266::from_f64(*size);
    }

    let link_urls = collect_link_urls(node_id, module);
    let para_start_x = ctx.x.to_f64();
    let para_start_y = ctx.y.to_f64();

    emit_v2_styled_paragraph(&runs, page, ctx, gir_doc)?;

    let line_h = ctx.line_height().to_f64();
    for url in &link_urls {
        page.links.push(ldir_ir::gir::GIRLink {
            x: para_start_x,
            y: para_start_y,
            width: ctx.content_width.to_f64(),
            height: line_h,
            url: url.clone(),
            destination_page: None,
        });
    }

    ctx.font_size = saved_font_size;
    ctx.advance_y(ctx.line_height());
    ctx.reset_x();
    maybe_new_page(page, ctx, gir_doc);

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn emit_v2_paragraph_with_bib(
    node_id: u32,
    module: &SIRModuleV2,
    page: &mut GIRPage,
    ctx: &mut CompileContext,
    gir_doc: &mut GIRDocument,
    labels: &IndexMap<String, String>,
    refs_map: &IndexMap<String, String>,
    bibliography: &HashMap<String, BibEntry>,
    cite_counter: &mut u32,
    cite_numbers: &mut IndexMap<String, u32>,
) -> Result<()> {
    let runs = collect_styled_runs(node_id, module, StyleModifier::EMPTY, labels, refs_map);
    if runs.iter().all(|(t, _)| t.trim().is_empty()) {
        return Ok(());
    }

    let style_name = module.body.get(node_id).and_then(|n| n.style.as_deref());
    let saved_font_size = ctx.font_size;

    if let Some(name) = style_name
        && let Some(decl) = module.styles.find(name)
        && let Some(Dimension::Pt(size)) = &decl.properties.font_size
    {
        ctx.font_size = Fp266::from_f64(*size);
    }

    let para_start_x = ctx.x.to_f64();
    let para_start_y = ctx.y.to_f64();

    let link_urls = collect_link_urls(node_id, module);
    emit_v2_styled_paragraph(&runs, page, ctx, gir_doc)?;

    let line_h = ctx.line_height().to_f64();
    for url in &link_urls {
        page.links.push(ldir_ir::gir::GIRLink {
            x: para_start_x,
            y: para_start_y,
            width: ctx.content_width.to_f64(),
            height: line_h,
            url: url.clone(),
            destination_page: None,
        });
    }

    ctx.font_size = saved_font_size;
    ctx.advance_y(ctx.line_height());
    ctx.reset_x();
    maybe_new_page(page, ctx, gir_doc);

    let _ = (bibliography, cite_counter, cite_numbers);

    Ok(())
}

fn collect_link_urls(node_id: u32, module: &SIRModuleV2) -> Vec<String> {
    if module.body.get(node_id).is_none() {
        return Vec::new();
    }
    let mut urls = Vec::with_capacity(4);
    collect_link_urls_recursive(node_id, module, &mut urls);
    urls
}

fn collect_link_urls_recursive(node_id: u32, module: &SIRModuleV2, urls: &mut Vec<String>) {
    let node = match module.body.get(node_id) {
        Some(n) => n,
        None => return,
    };
    if let NodeType::Link { url, .. } = &node.node_type {
        urls.push(url.clone());
        return;
    }
    for &child_id in &node.child_ids {
        collect_link_urls_recursive(child_id, module, urls);
    }
}

fn emit_v2_paragraph_text(
    text: &str,
    page: &mut GIRPage,
    ctx: &mut CompileContext,
    gir_doc: &mut GIRDocument,
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

        let result = if is_cjk_text(text) {
            let cjk_items = insert_cjk_breaks(text, &items, &ctx.bump);
            linebreak(&cjk_items, &options)
        } else {
            linebreak(&items, &options)
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
        ranges
    };

    for (line_idx, &(start, end)) in line_ranges.iter().enumerate() {
        emit_helpers::emit_set_font(page, ctx.font_id as i32);
        emit_helpers::emit_move_xy(page, ctx.x, ctx.y);

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
                gir_doc.push_page(std::mem::replace(page, ctx.new_page()));
            }
            ctx.y = ctx.margin_top;
        }
    }
}

fn emit_v2_text_inline(
    text: &str,
    page: &mut GIRPage,
    ctx: &mut CompileContext,
    gir_doc: &mut GIRDocument,
) {
    if text.trim().is_empty() {
        return;
    }
    emit_v2_paragraph_text(text, page, ctx, gir_doc);
    maybe_new_page(page, ctx, gir_doc);
}

fn emit_v2_math_inline(_content: &str, page: &mut GIRPage, ctx: &mut CompileContext) -> Result<()> {
    emit_helpers::emit_set_font(page, ctx.font_id as i32);
    emit_helpers::emit_move_xy(page, ctx.x, ctx.y);
    page.push(GIRCommand::new_put_glyph('[' as i32, 5 * 64));
    page.push(GIRCommand::new_put_glyph('m' as i32, 7 * 64));
    page.push(GIRCommand::new_put_glyph(']' as i32, 5 * 64));
    ctx.advance_x(Fp266::from_int(24));
    Ok(())
}

fn emit_v2_math_block(
    math_text: &str,
    numbered: bool,
    eq_number: Option<&str>,
    page: &mut GIRPage,
    ctx: &mut CompileContext,
    gir_doc: &mut GIRDocument,
) {
    let font_data_for_math = ctx
        .font_data_variants
        .get(ctx.font_id as usize)
        .and_then(|opt| opt.as_ref())
        .or(ctx.font_data.as_ref());
    let font_bytes = font_data_for_math.map(|d| d.as_slice());

    let result = crate::compiler::math::layout_math(math_text, font_bytes, ctx.font_size, ctx.y);

    emit_helpers::emit_set_font(page, ctx.font_id as i32);
    for glyph in &result.glyphs {
        let gx = ctx.x + glyph.x;
        let gy = glyph.y;
        if glyph.glyph_id == -1 {
            emit_helpers::emit_draw_rule(
                page,
                gx,
                gy,
                glyph.advance,
                Fp266::from_frac(ctx.font_size.to_int(), 16),
            );
        } else {
            emit_helpers::emit_move_xy(page, gx, gy);
            page.push(GIRCommand::new_put_glyph(
                glyph.glyph_id,
                glyph.advance.raw() as i32,
            ));
        }
    }

    if numbered {
        let num_x = ctx.page_width - ctx.margin_right - Fp266::from_int(36);
        emit_helpers::emit_move_xy(page, num_x, ctx.y);
        let display_num = eq_number.unwrap_or("1");
        let paren_num = format!("({})", display_num);
        for &ch in paren_num.as_bytes() {
            page.push(GIRCommand::new_put_glyph(ch as i32, 7 * 64));
        }
    }

    let math_spacing = result.height + result.depth + Fp266::from_int(6);
    ctx.advance_y(math_spacing);
    ctx.reset_x();
    maybe_new_page(page, ctx, gir_doc);
}

fn compile_v2_list_fallback(
    node_id: u32,
    module: &SIRModuleV2,
    page: &mut GIRPage,
    ctx: &mut CompileContext,
    gir_doc: &mut GIRDocument,
) -> Result<()> {
    let node = match module.body.get(node_id) {
        Some(n) => n,
        None => return Ok(()),
    };

    let is_ordered = match &node.node_type {
        NodeType::List { ordered, .. } => *ordered,
        _ => false,
    };

    let saved_margin = ctx.margin_left;
    ctx.margin_left += Fp266::from_int(24);
    ctx.content_width = ctx.page_width - ctx.margin_left - ctx.margin_right;
    ctx.reset_x();

    let mut item_idx: u32 = 0;
    for &child_id in &node.child_ids {
        let child = match module.body.get(child_id) {
            Some(c) => c,
            None => continue,
        };
        if !matches!(child.node_type, NodeType::ListItem) {
            continue;
        }

        item_idx += 1;
        let bullet = if is_ordered {
            format!("{}. ", item_idx)
        } else {
            "\u{2022} ".to_string()
        };

        emit_helpers::emit_set_font(page, ctx.font_id as i32);
        emit_helpers::emit_move_xy(page, saved_margin + Fp266::from_int(12), ctx.y);
        for ch in bullet.chars() {
            page.push(GIRCommand::new_put_glyph(ch as i32, 7 * 64));
        }

        let item_text = module.body.collect_text(child_id);
        if !item_text.trim().is_empty() {
            emit_v2_paragraph_text(&item_text, page, ctx, gir_doc);
        }

        ctx.advance_y(ctx.line_height());
        ctx.reset_x();
        maybe_new_page(page, ctx, gir_doc);
    }

    ctx.margin_left = saved_margin;
    ctx.content_width = ctx.page_width - ctx.margin_left - ctx.margin_right;
    ctx.reset_x();
    ctx.advance_y(ctx.line_height());
    maybe_new_page(page, ctx, gir_doc);

    Ok(())
}

fn compile_v2_table_improved(
    node_id: u32,
    module: &SIRModuleV2,
    page: &mut GIRPage,
    ctx: &mut CompileContext,
    gir_doc: &mut GIRDocument,
    col_specs: &[ColSpec],
    num_cols: usize,
) -> Result<()> {
    let node = match module.body.get(node_id) {
        Some(n) => n,
        None => return Ok(()),
    };

    struct RowData {
        cells: Vec<String>,
        is_header: bool,
    }

    let mut rows: Vec<RowData> = Vec::new();
    for &child_id in &node.child_ids {
        let child = match module.body.get(child_id) {
            Some(c) => c,
            None => continue,
        };
        let is_header = match &child.node_type {
            NodeType::TableRow { is_header } => *is_header,
            _ => continue,
        };
        let mut cells: Vec<String> = Vec::new();
        for &cell_id in &child.child_ids {
            let cell = match module.body.get(cell_id) {
                Some(c) => c,
                None => continue,
            };
            if matches!(cell.node_type, NodeType::TableCell { .. }) {
                cells.push(module.body.collect_text(cell_id));
            }
        }
        if !cells.is_empty() {
            rows.push(RowData { cells, is_header });
        }
    }

    if rows.is_empty() {
        return Ok(());
    }

    let effective_num_cols = if num_cols > 0 { num_cols } else { 1 };
    let mut col_widths: Vec<usize> = vec![0; effective_num_cols];
    for row in &rows {
        for (i, cell) in row.cells.iter().enumerate() {
            if i < effective_num_cols {
                col_widths[i] = col_widths[i].max(cell.chars().count());
            }
        }
    }

    let content_width_pts = ctx.content_width.to_f64() / 64.0;
    let cell_padding: f64 = 3.0;
    let rule_thickness: f64 = 0.5;
    let _header_rule_thickness: f64 = 1.0;
    let col_spacing_pts: f64 = col_specs.len() as f64 * rule_thickness;

    let col_widths_pt: Vec<f64> = {
        let total_text_width: f64 = col_widths.iter().map(|&w| w as f64 * 7.0).sum::<f64>()
            + effective_num_cols as f64 * 2.0 * cell_padding;
        let available = content_width_pts - col_spacing_pts;
        if total_text_width > available {
            let scale = available / total_text_width;
            col_widths
                .iter()
                .map(|&w| w as f64 * 7.0 * scale + 2.0 * cell_padding)
                .collect()
        } else {
            col_widths
                .iter()
                .map(|&w| w as f64 * 7.0 + 2.0 * cell_padding)
                .collect()
        }
    };

    let col_aligns: Vec<ColumnAlign> = col_specs
        .iter()
        .map(|spec| spec.align)
        .chain(std::iter::repeat(ColumnAlign::Left))
        .take(effective_num_cols)
        .collect();

    let table_start_x = ctx.x;
    let table_width_fp = ctx.content_width;

    let mut col_positions: Vec<Fp266> = Vec::with_capacity(effective_num_cols);
    let mut cx_fp = table_start_x;
    for (i, &col_w_pt) in col_widths_pt.iter().enumerate().take(effective_num_cols) {
        col_positions.push(cx_fp);
        let col_w_fp = Fp266::from_int((col_w_pt * 64.0) as i32);
        cx_fp += col_w_fp;
        if i < effective_num_cols - 1 {
            cx_fp += Fp266::from_frac(1, 2);
        }
    }

    ctx.advance_y(Fp266::from_int(4));

    for (row_idx, row) in rows.iter().enumerate() {
        let row_y = ctx.y;
        let saved_font_id = ctx.font_id;
        if row.is_header {
            ctx.font_id = FONT_ID_BOLD;
        }

        for (col_idx, cell) in row.cells.iter().enumerate() {
            if col_idx >= effective_num_cols {
                break;
            }

            let trimmed = cell.trim();
            if trimmed.is_empty() {
                continue;
            }

            let col_start_x = col_positions[col_idx];
            let col_width_fp = Fp266::from_int((col_widths_pt[col_idx] * 64.0) as i32);
            let padding_fp = Fp266::from_frac(3, 1);

            let text_width_bytes = trimmed.len();
            let text_width_fp = text_width_bytes as f64 * 7.0 * 64.0;

            let text_x = match col_aligns[col_idx] {
                ColumnAlign::Right => {
                    col_start_x + col_width_fp - padding_fp - Fp266::from_int(text_width_fp as i32)
                }
                ColumnAlign::Center => {
                    let mid = col_start_x + col_width_fp.div(Fp266::from_int(2));
                    mid - Fp266::from_int((text_width_fp / 2.0) as i32)
                }
                _ => col_start_x + padding_fp,
            };

            emit_helpers::emit_move_xy(page, text_x, row_y);
            emit_helpers::emit_set_font(page, ctx.font_id as i32);
            for ch in trimmed.chars() {
                page.push(GIRCommand::new_put_glyph(ch as i32, 7 * 64));
            }
        }

        ctx.font_id = saved_font_id;
        ctx.advance_y(ctx.line_height());

        if row_idx == 0 && row.is_header {
            let rule_y = ctx.y;
            emit_helpers::emit_draw_rule(
                page,
                table_start_x,
                rule_y,
                table_width_fp,
                Fp266::from_frac(1, 1),
            );
            ctx.advance_y(Fp266::from_int(2));
        }

        if row_idx > 0 {
            let rule_y = ctx.y - ctx.line_height();
            for rx in col_positions.iter().take(effective_num_cols).skip(1) {
                let rx = *rx - Fp266::from_frac(1, 2);
                emit_helpers::emit_draw_rule(
                    page,
                    rx,
                    rule_y,
                    Fp266::from_frac(1, 2),
                    ctx.line_height(),
                );
            }
        }

        ctx.reset_x();
        maybe_new_page(page, ctx, gir_doc);
    }

    for rx in col_positions.iter().take(effective_num_cols).skip(1) {
        let rx = *rx - Fp266::from_frac(1, 2);
        emit_helpers::emit_draw_rule(
            page,
            rx,
            ctx.y - ctx.line_height(),
            Fp266::from_frac(1, 2),
            ctx.line_height(),
        );
    }

    ctx.advance_y(ctx.line_height());
    ctx.reset_x();
    Ok(())
}

fn emit_v2_figure(
    node_id: u32,
    module: &SIRModuleV2,
    page: &mut GIRPage,
    ctx: &mut CompileContext,
    gir_doc: &mut GIRDocument,
    figure_counter: &mut usize,
) -> Result<()> {
    let node = match module.body.get(node_id) {
        Some(n) => n,
        None => return Ok(()),
    };

    *figure_counter += 1;
    let fig_num = *figure_counter;

    ctx.advance_y(Fp266::from_int(6));

    let mut caption_text: Option<String> = None;
    let mut caption_child_id: Option<u32> = None;

    for &child_id in &node.child_ids {
        let child = match module.body.get(child_id) {
            Some(c) => c,
            None => continue,
        };
        if matches!(child.node_type, NodeType::Caption) {
            caption_text = Some(module.body.collect_text(child_id));
            caption_child_id = Some(child_id);
        }
    }

    for &child_id in &node.child_ids {
        let child = match module.body.get(child_id) {
            Some(c) => c,
            None => continue,
        };
        if matches!(child.node_type, NodeType::Caption) {
            continue;
        }
        if let NodeType::Image { source, .. } = &child.node_type {
            let base_dir = module
                .header
                .source_path
                .as_ref()
                .and_then(|p| std::path::Path::new(p).parent());
            if let Some((img, w_fp, h_fp)) =
                load_and_scale_image(source, ctx.content_width, base_dir)
            {
                let image_index = gir_doc.push_image(img);
                emit_helpers::emit_move_xy(page, ctx.x, ctx.y);
                page.push(GIRCommand::new_draw_rule(
                    -1,
                    image_index as i32,
                    w_fp,
                    h_fp,
                ));
                ctx.advance_y(Fp266::from_raw(h_fp as i64));
                ctx.reset_x();
                maybe_new_page(page, ctx, gir_doc);
            }
        }
    }

    if let Some(cap_text) = caption_text {
        let trimmed = cap_text.trim();
        if !trimmed.is_empty() {
            ctx.advance_y(Fp266::from_int(4));

            let saved_font_size = ctx.font_size;
            let saved_font_id = ctx.font_id;
            ctx.font_size = Fp266::from_int(10);
            ctx.font_id = FONT_ID_ITALIC;

            let caption_label = format!("Figure {}: {}", fig_num, trimmed);
            let label_len = caption_label.len();
            let approx_width = label_len as f64 * 5.5 * 64.0;
            let center_x = ctx.margin_left
                + (ctx.content_width - Fp266::from_int(approx_width as i32))
                    .div(Fp266::from_int(2));

            emit_helpers::emit_move_xy(page, center_x, ctx.y);
            emit_helpers::emit_set_font(page, FONT_ID_ITALIC as i32);
            for ch in caption_label.chars() {
                page.push(GIRCommand::new_put_glyph(ch as i32, 6 * 64));
            }

            ctx.font_size = saved_font_size;
            ctx.font_id = saved_font_id;
            ctx.advance_y(ctx.line_height());
            ctx.reset_x();
            maybe_new_page(page, ctx, gir_doc);
        }
    }

    ctx.advance_y(ctx.line_height());
    ctx.reset_x();
    maybe_new_page(page, ctx, gir_doc);

    let _ = caption_child_id;

    Ok(())
}

fn maybe_new_page(page: &mut GIRPage, ctx: &mut CompileContext, gir_doc: &mut GIRDocument) {
    if ctx.exceeds_page() {
        if !page.is_empty() {
            gir_doc.push_page(std::mem::replace(page, ctx.new_page()));
        }
        ctx.y = ctx.margin_top;
    }
}

fn load_and_scale_image(
    path: &str,
    content_width: Fp266,
    base_dir: Option<&std::path::Path>,
) -> Option<(ldir_ir::gir::GIRImage, i32, i32)> {
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
        ldir_ir::gir::ImageFormat::Png => png_dimensions(&data)?,
        ldir_ir::gir::ImageFormat::Jpeg => jpeg_dimensions(&data)?,
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
        ldir_ir::gir::GIRImage {
            data,
            width: w_fp,
            height: h_fp,
            format,
        },
        w_fp,
        h_fp,
    ))
}

fn detect_image_format(data: &[u8]) -> Option<ldir_ir::gir::ImageFormat> {
    if data.len() >= 8 && data[0..4] == [0x89, 0x50, 0x4E, 0x47] {
        Some(ldir_ir::gir::ImageFormat::Png)
    } else if data.len() >= 3 && data[0..3] == [0xFF, 0xD8, 0xFF] {
        Some(ldir_ir::gir::ImageFormat::Jpeg)
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

#[cfg(test)]
mod tests {
    use super::*;
    use ldir_ir::sir::v2::nodes::{Node, NodeType};

    fn make_simple_v2_module() -> SIRModuleV2 {
        let mut module = SIRModuleV2::new();
        let doc_id = module.body.push(Node::new(0, NodeType::Document));
        let para_id = module
            .body
            .push(Node::new(1, NodeType::Paragraph).with_parent(doc_id));
        let text_id = module.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "Hello world".into(),
                },
            )
            .with_parent(para_id),
        );
        if let Some(p) = module.body.get_mut(doc_id) {
            p.add_child(para_id);
        }
        if let Some(p) = module.body.get_mut(para_id) {
            p.add_child(text_id);
        }
        module
    }

    #[test]
    fn test_v2_compile_paragraph() {
        let module = make_simple_v2_module();
        let mut ctx = CompileContext::default();
        let gir = compile_v2_document(&module, &mut ctx).unwrap();

        assert!(!gir.is_empty());
        assert!(gir.is_well_formed());
        assert!(gir.page_count() >= 1);

        let glyph_count = gir
            .iter()
            .flat_map(|page| page.iter())
            .filter(|cmd| cmd.opcode() == ldir_ir::gir::GIROpcode::PutGlyph)
            .count();
        assert!(glyph_count > 0, "should have glyphs");
    }

    #[test]
    fn test_v2_compile_heading() {
        let mut module = SIRModuleV2::new();
        let doc_id = module.body.push(Node::new(0, NodeType::Document));
        let ch_id = module
            .body
            .push(Node::new(1, NodeType::Chapter).with_parent(doc_id));
        let text_id = module.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "Introduction".into(),
                },
            )
            .with_parent(ch_id),
        );
        if let Some(d) = module.body.get_mut(doc_id) {
            d.add_child(ch_id);
        }
        if let Some(c) = module.body.get_mut(ch_id) {
            c.add_child(text_id);
        }

        let mut ctx = CompileContext::default();
        let gir = compile_v2_document(&module, &mut ctx).unwrap();

        assert!(gir.is_well_formed());
        let glyph_count = gir
            .iter()
            .flat_map(|page| page.iter())
            .filter(|cmd| cmd.opcode() == ldir_ir::gir::GIROpcode::PutGlyph)
            .count();
        assert!(glyph_count > 0);
    }

    #[test]
    fn test_v2_compile_bold_italic() {
        let mut module = SIRModuleV2::new();
        let doc_id = module.body.push(Node::new(0, NodeType::Document));
        let para_id = module
            .body
            .push(Node::new(1, NodeType::Paragraph).with_parent(doc_id));
        let bold_id = module
            .body
            .push(Node::new(2, NodeType::Bold).with_parent(para_id));
        let text_id = module.body.push(
            Node::new(
                3,
                NodeType::Text {
                    content: "bold text".into(),
                },
            )
            .with_parent(bold_id),
        );
        let italic_id = module
            .body
            .push(Node::new(4, NodeType::Italic).with_parent(para_id));
        let text2_id = module.body.push(
            Node::new(
                5,
                NodeType::Text {
                    content: " italic text".into(),
                },
            )
            .with_parent(italic_id),
        );
        if let Some(d) = module.body.get_mut(doc_id) {
            d.add_child(para_id);
        }
        if let Some(p) = module.body.get_mut(para_id) {
            p.add_child(bold_id);
            p.add_child(italic_id);
        }
        if let Some(b) = module.body.get_mut(bold_id) {
            b.add_child(text_id);
        }
        if let Some(i) = module.body.get_mut(italic_id) {
            i.add_child(text2_id);
        }

        let mut ctx = CompileContext::default();
        let gir = compile_v2_document(&module, &mut ctx).unwrap();

        assert!(gir.is_well_formed());
    }

    #[test]
    fn test_v2_compile_thematic_break() {
        let mut module = SIRModuleV2::new();
        let doc_id = module.body.push(Node::new(0, NodeType::Document));
        let para_id = module
            .body
            .push(Node::new(1, NodeType::Paragraph).with_parent(doc_id));
        let text_id = module.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "Before".into(),
                },
            )
            .with_parent(para_id),
        );
        let break_id = module
            .body
            .push(Node::new(3, NodeType::ThematicBreak).with_parent(doc_id));
        let para2_id = module
            .body
            .push(Node::new(4, NodeType::Paragraph).with_parent(doc_id));
        let text2_id = module.body.push(
            Node::new(
                5,
                NodeType::Text {
                    content: "After".into(),
                },
            )
            .with_parent(para2_id),
        );
        if let Some(d) = module.body.get_mut(doc_id) {
            d.add_child(para_id);
            d.add_child(break_id);
            d.add_child(para2_id);
        }
        if let Some(p) = module.body.get_mut(para_id) {
            p.add_child(text_id);
        }
        if let Some(p) = module.body.get_mut(para2_id) {
            p.add_child(text2_id);
        }

        let mut ctx = CompileContext::default();
        let gir = compile_v2_document(&module, &mut ctx).unwrap();

        assert!(gir.is_well_formed());
        let has_rule = gir
            .iter()
            .flat_map(|page| page.iter())
            .any(|cmd| cmd.opcode() == ldir_ir::gir::GIROpcode::DrawRule);
        assert!(has_rule);
    }

    #[test]
    fn test_v2_compile_blockquote() {
        let mut module = SIRModuleV2::new();
        let doc_id = module.body.push(Node::new(0, NodeType::Document));
        let bq_id = module
            .body
            .push(Node::new(1, NodeType::BlockQuote).with_parent(doc_id));
        let para_id = module
            .body
            .push(Node::new(2, NodeType::Paragraph).with_parent(bq_id));
        let text_id = module.body.push(
            Node::new(
                3,
                NodeType::Text {
                    content: "A quoted passage.".into(),
                },
            )
            .with_parent(para_id),
        );
        if let Some(d) = module.body.get_mut(doc_id) {
            d.add_child(bq_id);
        }
        if let Some(b) = module.body.get_mut(bq_id) {
            b.add_child(para_id);
        }
        if let Some(p) = module.body.get_mut(para_id) {
            p.add_child(text_id);
        }

        let mut ctx = CompileContext::default();
        let gir = compile_v2_document(&module, &mut ctx).unwrap();

        assert!(gir.is_well_formed());
        let has_rule = gir
            .iter()
            .flat_map(|page| page.iter())
            .any(|cmd| cmd.opcode() == ldir_ir::gir::GIROpcode::DrawRule);
        assert!(has_rule, "blockquote should have a left rule");
    }

    #[test]
    fn test_v2_compile_page_break() {
        let mut module = SIRModuleV2::new();
        let doc_id = module.body.push(Node::new(0, NodeType::Document));
        let para1_id = module
            .body
            .push(Node::new(1, NodeType::Paragraph).with_parent(doc_id));
        let text1_id = module.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "Page one content.".into(),
                },
            )
            .with_parent(para1_id),
        );
        let pb_id = module
            .body
            .push(Node::new(3, NodeType::PageBreak).with_parent(doc_id));
        let para2_id = module
            .body
            .push(Node::new(4, NodeType::Paragraph).with_parent(doc_id));
        let text2_id = module.body.push(
            Node::new(
                5,
                NodeType::Text {
                    content: "Page two content.".into(),
                },
            )
            .with_parent(para2_id),
        );
        if let Some(d) = module.body.get_mut(doc_id) {
            d.add_child(para1_id);
            d.add_child(pb_id);
            d.add_child(para2_id);
        }
        if let Some(p) = module.body.get_mut(para1_id) {
            p.add_child(text1_id);
        }
        if let Some(p) = module.body.get_mut(para2_id) {
            p.add_child(text2_id);
        }

        let mut ctx = CompileContext::default();
        let gir = compile_v2_document(&module, &mut ctx).unwrap();

        assert!(gir.is_well_formed());
        assert!(
            gir.page_count() >= 2,
            "page break should create at least 2 pages"
        );
    }

    #[test]
    fn test_v2_compile_multiline_paragraph() {
        let long_text = "The quick brown fox jumps over the lazy dog. Pack my box with five dozen liquor jugs. This should wrap to multiple lines.";
        let mut module = SIRModuleV2::new();
        let doc_id = module.body.push(Node::new(0, NodeType::Document));
        let para_id = module
            .body
            .push(Node::new(1, NodeType::Paragraph).with_parent(doc_id));
        let text_id = module.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: long_text.into(),
                },
            )
            .with_parent(para_id),
        );
        if let Some(d) = module.body.get_mut(doc_id) {
            d.add_child(para_id);
        }
        if let Some(p) = module.body.get_mut(para_id) {
            p.add_child(text_id);
        }

        let mut ctx = CompileContext::default();
        let gir = compile_v2_document(&module, &mut ctx).unwrap();

        assert!(gir.is_well_formed());
        let glyph_count = gir
            .iter()
            .flat_map(|page| page.iter())
            .filter(|cmd| cmd.opcode() == ldir_ir::gir::GIROpcode::PutGlyph)
            .count();
        assert!(glyph_count > 0);
    }

    #[test]
    fn test_v2_compile_deterministic() {
        let module = make_simple_v2_module();
        let mut ctx1 = CompileContext::default();
        let mut ctx2 = CompileContext::default();
        let gir1 = compile_v2_document(&module, &mut ctx1).unwrap();
        let gir2 = compile_v2_document(&module, &mut ctx2).unwrap();
        assert_eq!(gir1, gir2);
    }

    #[test]
    fn test_v2_compile_empty_module() {
        let module = SIRModuleV2::new();
        let mut ctx = CompileContext::default();
        let gir = compile_v2_document(&module, &mut ctx).unwrap();
        assert!(gir.is_well_formed());
    }

    #[test]
    fn test_v2_compile_multiple_headings() {
        let mut module = SIRModuleV2::new();
        let doc_id = module.body.push(Node::new(0, NodeType::Document));

        let ch_id = module
            .body
            .push(Node::new(1, NodeType::Chapter).with_parent(doc_id));
        let ch_text = module.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "Chapter 1".into(),
                },
            )
            .with_parent(ch_id),
        );
        if let Some(c) = module.body.get_mut(ch_id) {
            c.add_child(ch_text);
        }

        let sec_id = module
            .body
            .push(Node::new(3, NodeType::Section).with_parent(doc_id));
        let sec_text = module.body.push(
            Node::new(
                4,
                NodeType::Text {
                    content: "Section 1.1".into(),
                },
            )
            .with_parent(sec_id),
        );
        if let Some(s) = module.body.get_mut(sec_id) {
            s.add_child(sec_text);
        }

        let para_id = module
            .body
            .push(Node::new(5, NodeType::Paragraph).with_parent(doc_id));
        let para_text = module.body.push(
            Node::new(
                6,
                NodeType::Text {
                    content: "Some content here.".into(),
                },
            )
            .with_parent(para_id),
        );
        if let Some(p) = module.body.get_mut(para_id) {
            p.add_child(para_text);
        }

        if let Some(d) = module.body.get_mut(doc_id) {
            d.add_child(ch_id);
            d.add_child(sec_id);
            d.add_child(para_id);
        }

        let mut ctx = CompileContext::default();
        let gir = compile_v2_document(&module, &mut ctx).unwrap();

        assert!(gir.is_well_formed());
        assert!(gir.page_count() >= 1);
    }

    #[test]
    fn test_v2_compile_code_block() {
        let mut module = SIRModuleV2::new();
        let doc_id = module.body.push(Node::new(0, NodeType::Document));
        let code_id = module.body.push(
            Node::new(
                1,
                NodeType::CodeBlock {
                    language: Some("rust".into()),
                    content: String::new(),
                },
            )
            .with_parent(doc_id),
        );
        let text_id = module.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "fn main() { println!(\"Hello\"); }".into(),
                },
            )
            .with_parent(code_id),
        );
        if let Some(d) = module.body.get_mut(doc_id) {
            d.add_child(code_id);
        }
        if let Some(c) = module.body.get_mut(code_id) {
            c.add_child(text_id);
        }

        let mut ctx = CompileContext::default();
        let gir = compile_v2_document(&module, &mut ctx).unwrap();

        assert!(gir.is_well_formed());
    }

    #[test]
    fn test_v2_compile_list_fallback() {
        let mut module = SIRModuleV2::new();
        let doc_id = module.body.push(Node::new(0, NodeType::Document));
        let list_id = module.body.push(
            Node::new(
                1,
                NodeType::List {
                    list_type: ldir_ir::sir::v2::nodes::ListType::Unordered,
                    ordered: false,
                    start: None,
                },
            )
            .with_parent(doc_id),
        );

        let item1_id = module
            .body
            .push(Node::new(2, NodeType::ListItem).with_parent(list_id));
        let item1_text = module.body.push(
            Node::new(
                3,
                NodeType::Text {
                    content: "First item".into(),
                },
            )
            .with_parent(item1_id),
        );
        if let Some(i) = module.body.get_mut(item1_id) {
            i.add_child(item1_text);
        }

        let item2_id = module
            .body
            .push(Node::new(4, NodeType::ListItem).with_parent(list_id));
        let item2_text = module.body.push(
            Node::new(
                5,
                NodeType::Text {
                    content: "Second item".into(),
                },
            )
            .with_parent(item2_id),
        );
        if let Some(i) = module.body.get_mut(item2_id) {
            i.add_child(item2_text);
        }

        if let Some(d) = module.body.get_mut(doc_id) {
            d.add_child(list_id);
        }
        if let Some(l) = module.body.get_mut(list_id) {
            l.add_child(item1_id);
            l.add_child(item2_id);
        }

        let mut ctx = CompileContext::default();
        let gir = compile_v2_document(&module, &mut ctx).unwrap();

        assert!(gir.is_well_formed());
        let glyph_count = gir
            .iter()
            .flat_map(|page| page.iter())
            .filter(|cmd| cmd.opcode() == ldir_ir::gir::GIROpcode::PutGlyph)
            .count();
        assert!(glyph_count > 0);
    }

    #[test]
    fn test_v2_compile_ordered_list() {
        let mut module = SIRModuleV2::new();
        let doc_id = module.body.push(Node::new(0, NodeType::Document));
        let list_id = module.body.push(
            Node::new(
                1,
                NodeType::List {
                    list_type: ldir_ir::sir::v2::nodes::ListType::Ordered,
                    ordered: true,
                    start: None,
                },
            )
            .with_parent(doc_id),
        );

        let item1_id = module
            .body
            .push(Node::new(2, NodeType::ListItem).with_parent(list_id));
        let item1_text = module.body.push(
            Node::new(
                3,
                NodeType::Text {
                    content: "First".into(),
                },
            )
            .with_parent(item1_id),
        );
        if let Some(i) = module.body.get_mut(item1_id) {
            i.add_child(item1_text);
        }

        if let Some(d) = module.body.get_mut(doc_id) {
            d.add_child(list_id);
        }
        if let Some(l) = module.body.get_mut(list_id) {
            l.add_child(item1_id);
        }

        let mut ctx = CompileContext::default();
        let gir = compile_v2_document(&module, &mut ctx).unwrap();

        assert!(gir.is_well_formed());
    }

    #[test]
    fn test_v2_compile_link() {
        let mut module = SIRModuleV2::new();
        let doc_id = module.body.push(Node::new(0, NodeType::Document));
        let para_id = module
            .body
            .push(Node::new(1, NodeType::Paragraph).with_parent(doc_id));
        let link_id = module.body.push(
            Node::new(
                2,
                NodeType::Link {
                    url: "https://example.com".into(),
                    title: None,
                },
            )
            .with_parent(para_id),
        );
        let text_id = module.body.push(
            Node::new(
                3,
                NodeType::Text {
                    content: "click here".into(),
                },
            )
            .with_parent(link_id),
        );
        if let Some(d) = module.body.get_mut(doc_id) {
            d.add_child(para_id);
        }
        if let Some(p) = module.body.get_mut(para_id) {
            p.add_child(link_id);
        }
        if let Some(l) = module.body.get_mut(link_id) {
            l.add_child(text_id);
        }

        let mut ctx = CompileContext::default();
        let gir = compile_v2_document(&module, &mut ctx).unwrap();

        assert!(gir.is_well_formed());
        let has_link = gir
            .iter()
            .flat_map(|page| page.links.iter())
            .any(|link| link.url == "https://example.com");
        assert!(has_link);
    }

    #[test]
    fn test_v2_compile_mixed_formatting() {
        let mut module = SIRModuleV2::new();
        let doc_id = module.body.push(Node::new(0, NodeType::Document));
        let para_id = module
            .body
            .push(Node::new(1, NodeType::Paragraph).with_parent(doc_id));
        let text1 = module.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "Normal ".into(),
                },
            )
            .with_parent(para_id),
        );
        let bold = module
            .body
            .push(Node::new(3, NodeType::Bold).with_parent(para_id));
        let text2 = module.body.push(
            Node::new(
                4,
                NodeType::Text {
                    content: "bold".into(),
                },
            )
            .with_parent(bold),
        );
        let italic = module
            .body
            .push(Node::new(5, NodeType::Italic).with_parent(para_id));
        let text3 = module.body.push(
            Node::new(
                6,
                NodeType::Text {
                    content: " italic".into(),
                },
            )
            .with_parent(italic),
        );
        let mono = module
            .body
            .push(Node::new(7, NodeType::Mono).with_parent(para_id));
        let text4 = module.body.push(
            Node::new(
                8,
                NodeType::Text {
                    content: " code".into(),
                },
            )
            .with_parent(mono),
        );
        if let Some(d) = module.body.get_mut(doc_id) {
            d.add_child(para_id);
        }
        if let Some(p) = module.body.get_mut(para_id) {
            p.add_child(text1);
            p.add_child(bold);
            p.add_child(italic);
            p.add_child(mono);
        }
        if let Some(b) = module.body.get_mut(bold) {
            b.add_child(text2);
        }
        if let Some(i) = module.body.get_mut(italic) {
            i.add_child(text3);
        }
        if let Some(m) = module.body.get_mut(mono) {
            m.add_child(text4);
        }

        let mut ctx = CompileContext::default();
        let gir = compile_v2_document(&module, &mut ctx).unwrap();

        assert!(gir.is_well_formed());
    }

    #[test]
    fn test_collect_v2_labels_from_module() {
        let mut module = SIRModuleV2::new();
        let doc_id = module.body.push(Node::new(0, NodeType::Document));
        let sec_id = module.body.push(
            Node::new(1, NodeType::Section)
                .with_label("sec:intro")
                .with_parent(doc_id),
        );
        let sec_text = module.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "Introduction".into(),
                },
            )
            .with_parent(sec_id),
        );
        if let Some(d) = module.body.get_mut(doc_id) {
            d.add_child(sec_id);
        }
        if let Some(s) = module.body.get_mut(sec_id) {
            s.add_child(sec_text);
        }

        let labels = collect_v2_labels(&module);
        assert!(labels.contains_key("sec:intro"));
    }

    #[test]
    fn test_collect_v2_labels_from_annotations() {
        let mut module = SIRModuleV2::new();
        module
            .annotations
            .add_label("fig:diagram".to_string(), 42, LabelCategory::Figure);

        let labels = collect_v2_labels(&module);
        assert!(labels.contains_key("fig:diagram"));
    }

    #[test]
    fn test_resolve_v2_references_ref() {
        let mut labels = IndexMap::new();
        labels.insert("sec:intro".to_string(), "1".to_string());
        let refs_map = IndexMap::new();

        let text = r"See \ref{sec:intro} for details.";
        let resolved = resolve_v2_references(text, &labels, &refs_map);
        assert_eq!(resolved, r"See 1 for details.");
    }

    #[test]
    fn test_resolve_v2_references_eqref() {
        let mut labels = IndexMap::new();
        labels.insert("eq:euler".to_string(), "3".to_string());
        let refs_map = IndexMap::new();

        let text = r"By \eqref{eq:euler}, we know...";
        let resolved = resolve_v2_references(text, &labels, &refs_map);
        assert_eq!(resolved, "By (3), we know...");
    }

    #[test]
    fn test_resolve_v2_references_unknown() {
        let labels = IndexMap::new();
        let refs_map = IndexMap::new();

        let text = r"See \ref{missing} for details.";
        let resolved = resolve_v2_references(text, &labels, &refs_map);
        assert_eq!(resolved, "See ?? for details.");
    }

    #[test]
    fn test_resolve_v2_references_typst_style() {
        let mut labels = IndexMap::new();
        labels.insert("sec:results".to_string(), "2.3".to_string());
        let refs_map = IndexMap::new();

        let text = "As shown in @sec:results, the results are clear.";
        let resolved = resolve_v2_references(text, &labels, &refs_map);
        assert_eq!(resolved, "As shown in 2.3, the results are clear.");
    }

    #[test]
    fn test_resolve_v2_references_autoref() {
        let mut labels = IndexMap::new();
        labels.insert("sec:methods".to_string(), "2".to_string());
        let refs_map = IndexMap::new();

        let text = r"See \autoref{sec:methods}.";
        let resolved = resolve_v2_references(text, &labels, &refs_map);
        assert_eq!(resolved, "See Section 2.");
    }

    #[test]
    fn test_section_numbering() {
        let mut module = SIRModuleV2::new();
        let doc_id = module.body.push(Node::new(0, NodeType::Document));

        let sec1 = module
            .body
            .push(Node::new(1, NodeType::Section).with_parent(doc_id));
        let sec1_text = module.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "First Section".into(),
                },
            )
            .with_parent(sec1),
        );
        if let Some(s) = module.body.get_mut(sec1) {
            s.add_child(sec1_text);
        }

        let sec2 = module
            .body
            .push(Node::new(3, NodeType::Section).with_parent(doc_id));
        let sec2_text = module.body.push(
            Node::new(
                4,
                NodeType::Text {
                    content: "Second Section".into(),
                },
            )
            .with_parent(sec2),
        );
        if let Some(s) = module.body.get_mut(sec2) {
            s.add_child(sec2_text);
        }

        if let Some(d) = module.body.get_mut(doc_id) {
            d.add_child(sec1);
            d.add_child(sec2);
        }

        let mut ctx = CompileContext::default();
        let gir = compile_v2_document(&module, &mut ctx).unwrap();
        assert!(gir.is_well_formed());
        assert!(gir.page_count() >= 1);
    }

    #[test]
    fn test_v2_compile_citation_node() {
        let mut module = SIRModuleV2::new();
        let doc_id = module.body.push(Node::new(0, NodeType::Document));
        let para_id = module
            .body
            .push(Node::new(1, NodeType::Paragraph).with_parent(doc_id));
        let text1 = module.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "As shown in ".into(),
                },
            )
            .with_parent(para_id),
        );
        let cite = module.body.push(
            Node::new(
                3,
                NodeType::Citation {
                    keys: vec!["knuth1984".to_string()],
                    style: None,
                },
            )
            .with_parent(para_id),
        );
        let text2 = module.body.push(
            Node::new(
                4,
                NodeType::Text {
                    content: ", we see...".into(),
                },
            )
            .with_parent(para_id),
        );
        if let Some(d) = module.body.get_mut(doc_id) {
            d.add_child(para_id);
        }
        if let Some(p) = module.body.get_mut(para_id) {
            p.add_child(text1);
            p.add_child(cite);
            p.add_child(text2);
        }

        let mut ctx = CompileContext::default();
        let gir = compile_v2_document(&module, &mut ctx).unwrap();
        assert!(gir.is_well_formed());
    }

    #[test]
    fn test_v2_compile_citation_with_bib() {
        let mut module = SIRModuleV2::new();
        let doc_id = module.body.push(Node::new(0, NodeType::Document));
        let para_id = module
            .body
            .push(Node::new(1, NodeType::Paragraph).with_parent(doc_id));
        let text1 = module.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "See ".into(),
                },
            )
            .with_parent(para_id),
        );
        let cite = module.body.push(
            Node::new(
                3,
                NodeType::Citation {
                    keys: vec!["knuth1984".to_string()],
                    style: None,
                },
            )
            .with_parent(para_id),
        );
        if let Some(d) = module.body.get_mut(doc_id) {
            d.add_child(para_id);
        }
        if let Some(p) = module.body.get_mut(para_id) {
            p.add_child(text1);
            p.add_child(cite);
        }

        let bib_content = r#"@article{knuth1984, author = {Donald E. Knuth}, title = {Literate Programming}, journal = {The Computer Journal}, volume = {27}, year = {1984}}"#;
        let bibliography =
            crate::compiler::bibtex::parse_bib(bib_content).expect("parse should succeed");

        let mut ctx = CompileContext::default();
        let gir = compile_v2_document_with_bib(&module, &mut ctx, &bibliography).unwrap();
        assert!(gir.is_well_formed());
    }

    #[test]
    fn test_v2_compile_with_refs_and_labels() {
        let mut module = SIRModuleV2::new();
        let doc_id = module.body.push(Node::new(0, NodeType::Document));

        let sec_id = module.body.push(
            Node::new(1, NodeType::Section)
                .with_label("sec:test")
                .with_parent(doc_id),
        );
        let sec_text = module.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "Test Section".into(),
                },
            )
            .with_parent(sec_id),
        );
        if let Some(s) = module.body.get_mut(sec_id) {
            s.add_child(sec_text);
        }

        let para_id = module
            .body
            .push(Node::new(3, NodeType::Paragraph).with_parent(doc_id));
        let para_text = module.body.push(
            Node::new(
                4,
                NodeType::Text {
                    content: r"Refer to \ref{sec:test} here.".into(),
                },
            )
            .with_parent(para_id),
        );
        if let Some(p) = module.body.get_mut(para_id) {
            p.add_child(para_text);
        }

        if let Some(d) = module.body.get_mut(doc_id) {
            d.add_child(sec_id);
            d.add_child(para_id);
        }

        let mut ctx = CompileContext::default();
        let gir = compile_v2_document(&module, &mut ctx).unwrap();
        assert!(gir.is_well_formed());
    }
}
