use ldir_ir::fp266::Fp266;
use ldir_ir::gir::{GIRCommand, GIRDocument, GIRPage};
use ldir_ir::lir::{LIRDocument, LIRNode};

fn fp266_to_i32(v: Fp266) -> i32 {
    v.raw() as i32
}

pub fn render_lir_to_gir(lir_doc: &LIRDocument) -> GIRDocument {
    let mut gir_doc = GIRDocument::with_capacity(lir_doc.pages.len());

    for lir_page in &lir_doc.pages {
        let mut gir_page = GIRPage::with_dimensions(
            fp266_to_i32(lir_page.page_width),
            fp266_to_i32(lir_page.page_height),
        );

        for child in &lir_page.children {
            render_node(child, &mut gir_page);
        }

        gir_doc.push_page(gir_page);
    }

    gir_doc
}

fn render_node(node: &LIRNode, page: &mut GIRPage) {
    match node {
        LIRNode::Document(_) => {}
        LIRNode::Page(_) => {}
        LIRNode::Flow(flow) => {
            page.push_stack();
            for child in &flow.children {
                render_node(child, page);
            }
            page.pop_stack();
        }
        LIRNode::Paragraph(para) => {
            page.push_stack();
            for child in &para.children {
                render_node(child, page);
            }
            page.pop_stack();
        }
        LIRNode::Line(line) => {
            for child in &line.children {
                render_node(child, page);
            }
        }
        LIRNode::Glyph(glyph) => {
            let geo = &glyph.geometry;
            let x = fp266_to_i32(geo.x);
            let y = fp266_to_i32(geo.y) + fp266_to_i32(geo.baseline);
            let advance = fp266_to_i32(glyph.advance_x);

            if glyph.font_id != 0 {
                page.push(GIRCommand::new_set_font(glyph.font_id as i32));
            }

            page.push(GIRCommand::new_move_xy(x, y));
            page.push(GIRCommand::new_put_glyph(glyph.glyph_id as i32, advance));
        }
        LIRNode::Space(space) => {
            let width = fp266_to_i32(space.natural_width);
            page.push(GIRCommand::new_move_xy(
                page.get(page.len().saturating_sub(1))
                    .and_then(|c| c.arg(0))
                    .unwrap_or(0)
                    + width,
                page.get(page.len().saturating_sub(1))
                    .and_then(|c| c.arg(1))
                    .unwrap_or(0),
            ));
        }
        LIRNode::Heading(heading) => {
            page.push_stack();
            let font_id = if heading.level <= 2 { 1 } else { 0 };
            page.push(GIRCommand::new_set_font(font_id));
            for child in &heading.children {
                render_node(child, page);
            }
            page.pop_stack();
        }
        LIRNode::List(list) => {
            page.push_stack();
            for child in &list.children {
                render_node(child, page);
            }
            page.pop_stack();
        }
        LIRNode::ListItem(item) => {
            page.push_stack();
            for child in &item.children {
                render_node(child, page);
            }
            page.pop_stack();
        }
        LIRNode::Table(table) => {
            page.push_stack();
            let geo = &table.geometry;
            let x = fp266_to_i32(geo.x);
            let thickness = if table.border { 64 } else { 0 };

            if table.border {
                let y = fp266_to_i32(geo.y);
                let w = fp266_to_i32(geo.width);
                page.push(GIRCommand::new_draw_rule(x, y, w, thickness));
            }

            for child in &table.children {
                render_node(child, page);
            }

            if table.border {
                let y = fp266_to_i32(geo.y) + fp266_to_i32(geo.height);
                let w = fp266_to_i32(geo.width);
                page.push(GIRCommand::new_draw_rule(x, y, w, thickness));
            }

            page.pop_stack();
        }
        LIRNode::TableRow(row) => {
            for child in &row.children {
                render_node(child, page);
            }
        }
        LIRNode::TableCell(cell) => {
            page.push_stack();
            for child in &cell.children {
                render_node(child, page);
            }
            page.pop_stack();
        }
        LIRNode::Figure(figure) => {
            page.push_stack();
            if let Some(ref caption) = figure.caption {
                render_node(&LIRNode::Caption((**caption).clone()), page);
            }
            page.pop_stack();
        }
        LIRNode::Caption(caption) => {
            page.push_stack();
            page.push(GIRCommand::new_set_font(2));
            for child in &caption.children {
                render_node(child, page);
            }
            page.pop_stack();
        }
        LIRNode::Footnote(footnote) => {
            let geo = &footnote.geometry;
            let x = fp266_to_i32(geo.x);
            let y = fp266_to_i32(geo.y);
            page.push(GIRCommand::new_move_xy(x, y));
            page.push(GIRCommand::new_put_glyph(footnote.marker as i32, 64));
        }
        LIRNode::FootnoteBlock(block) => {
            page.push_stack();
            for child in &block.children {
                render_node(child, page);
            }
            page.pop_stack();
        }
        LIRNode::BlockQuote(blockquote) => {
            page.push_stack();
            let geo = &blockquote.geometry;
            let x = fp266_to_i32(geo.x);
            let y = fp266_to_i32(geo.y);
            let h = fp266_to_i32(geo.height);
            let thickness = fp266_to_i32(blockquote.rule_width);
            if !blockquote.rule_width.is_zero() {
                page.push(GIRCommand::new_draw_rule(x, y, thickness, h));
            }
            for child in &blockquote.children {
                render_node(child, page);
            }
            page.pop_stack();
        }
        LIRNode::CodeBlock(codeblock) => {
            page.push_stack();
            page.push(GIRCommand::new_set_font(3));
            for child in &codeblock.children {
                render_node(child, page);
            }
            page.pop_stack();
        }
        LIRNode::MathBlock(mathblock) => {
            page.push_stack();
            for child in &mathblock.children {
                render_node(child, page);
            }
            page.pop_stack();
        }
        LIRNode::ThematicBreak(break_node) => {
            let geo = &break_node.geometry;
            let x = fp266_to_i32(geo.x);
            let y = fp266_to_i32(geo.y);
            let w = fp266_to_i32(geo.width);
            let thickness = fp266_to_i32(break_node.thickness);
            page.push(GIRCommand::new_draw_rule(x, y, w, thickness));
        }
        LIRNode::TableOfContents(_toc) => {}
        LIRNode::Bibliography(bib) => {
            page.push_stack();
            for child in &bib.children {
                render_node(child, page);
            }
            page.pop_stack();
        }
        LIRNode::Citation(_) => {}
        LIRNode::PageBreak(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldir_ir::lir::{
        FlowDirection, LIRFlow, LIRGeometry, LIRGlyph, LIRLine, LIRPage, LIRParagraph, LIRTable,
        LIRTableCell, LIRTableRow,
    };

    fn make_page_with_children(children: Vec<LIRNode>) -> LIRPage {
        let mut page = LIRPage::new(1, &ldir_ir::lir::LIRDocumentMeta::us_letter());
        page.children = children;
        page
    }

    #[test]
    fn test_empty_doc() {
        let lir_doc = LIRDocument::default();
        let gir_doc = render_lir_to_gir(&lir_doc);
        assert_eq!(gir_doc.page_count(), 0);
        assert!(gir_doc.is_empty());
        assert!(gir_doc.is_well_formed());
    }

    #[test]
    fn test_empty_page() {
        let mut lir_doc = LIRDocument::default();
        lir_doc.pages.push(make_page_with_children(vec![]));
        let gir_doc = render_lir_to_gir(&lir_doc);
        assert_eq!(gir_doc.page_count(), 1);
        assert!(gir_doc.is_well_formed());
    }

    #[test]
    fn test_paragraph_rendering() {
        let mut glyph = LIRGlyph::new(72, 0, Fp266::from_int(10));
        glyph.geometry = LIRGeometry::new(
            Fp266::from_int(72),
            Fp266::from_int(72),
            Fp266::from_int(10),
            Fp266::from_int(12),
        );

        let mut line = LIRLine::new(0);
        line.children.push(LIRNode::Glyph(glyph));

        let mut para = LIRParagraph::new();
        para.children.push(LIRNode::Line(line));

        let mut flow = LIRFlow::new(FlowDirection::TopToBottom);
        flow.children.push(LIRNode::Paragraph(para));

        let page = make_page_with_children(vec![LIRNode::Flow(flow)]);

        let mut lir_doc = LIRDocument::default();
        lir_doc.pages.push(page);

        let gir_doc = render_lir_to_gir(&lir_doc);
        assert_eq!(gir_doc.page_count(), 1);
        assert!(gir_doc.is_well_formed());

        let gir_page = &gir_doc[0];
        assert!(!gir_page.is_empty());

        let opcodes: Vec<_> = gir_page.iter().map(|c| c.opcode()).collect();
        assert!(opcodes.contains(&ldir_ir::gir::GIROpcode::PutGlyph));
    }

    #[test]
    fn test_heading() {
        let mut glyph = LIRGlyph::new(72, 0, Fp266::from_int(10));
        glyph.geometry = LIRGeometry::new(
            Fp266::from_int(72),
            Fp266::from_int(72),
            Fp266::from_int(10),
            Fp266::from_int(16),
        );

        let mut line = LIRLine::new(0);
        line.children.push(LIRNode::Glyph(glyph));

        let mut heading = ldir_ir::lir::LIRHeading::new(1);
        heading.children.push(LIRNode::Line(line));

        let mut flow = LIRFlow::new(FlowDirection::TopToBottom);
        flow.children.push(LIRNode::Heading(heading));

        let page = make_page_with_children(vec![LIRNode::Flow(flow)]);

        let mut lir_doc = LIRDocument::default();
        lir_doc.pages.push(page);

        let gir_doc = render_lir_to_gir(&lir_doc);
        assert_eq!(gir_doc.page_count(), 1);
        assert!(gir_doc.is_well_formed());

        let gir_page = &gir_doc[0];
        let has_set_font_bold = gir_page
            .iter()
            .any(|c| c.opcode() == ldir_ir::gir::GIROpcode::SetFont && c.arg(0) == Some(1));
        assert!(
            has_set_font_bold,
            "heading level 1 should set bold font (font_id=1)"
        );
    }

    #[test]
    fn test_page_break() {
        let flow1 = LIRFlow::new(FlowDirection::TopToBottom);
        let flow2 = LIRFlow::new(FlowDirection::TopToBottom);

        let page = make_page_with_children(vec![
            LIRNode::Flow(flow1),
            LIRNode::PageBreak(ldir_ir::lir::LIRPageBreak::new()),
            LIRNode::Flow(flow2),
        ]);

        let mut lir_doc = LIRDocument::default();
        lir_doc.pages.push(page);

        let gir_doc = render_lir_to_gir(&lir_doc);
        assert_eq!(gir_doc.page_count(), 1);
        assert!(gir_doc.is_well_formed());
    }

    #[test]
    fn test_table() {
        let mut glyph = LIRGlyph::new(65, 0, Fp266::from_int(8));
        glyph.geometry = LIRGeometry::new(
            Fp266::from_int(100),
            Fp266::from_int(100),
            Fp266::from_int(8),
            Fp266::from_int(12),
        );

        let mut line = LIRLine::new(0);
        line.children.push(LIRNode::Glyph(glyph));

        let mut para = LIRParagraph::new();
        para.children.push(LIRNode::Line(line));

        let mut cell_flow = LIRFlow::new(FlowDirection::TopToBottom);
        cell_flow.children.push(LIRNode::Paragraph(para));

        let mut cell = LIRTableCell::new(0);
        cell.children.push(LIRNode::Flow(cell_flow));

        let mut row = LIRTableRow::new(false);
        row.children.push(LIRNode::TableCell(cell));

        let mut table = LIRTable::new(1);
        table.border = true;
        table.geometry = LIRGeometry::from_int(72, 72, 468, 100);
        table.children.push(LIRNode::TableRow(row));

        let mut flow = LIRFlow::new(FlowDirection::TopToBottom);
        flow.children.push(LIRNode::Table(table));

        let page = make_page_with_children(vec![LIRNode::Flow(flow)]);

        let mut lir_doc = LIRDocument::default();
        lir_doc.pages.push(page);

        let gir_doc = render_lir_to_gir(&lir_doc);
        assert_eq!(gir_doc.page_count(), 1);
        assert!(gir_doc.is_well_formed());

        let gir_page = &gir_doc[0];
        let has_draw_rule = gir_page
            .iter()
            .any(|c| c.opcode() == ldir_ir::gir::GIROpcode::DrawRule);
        assert!(has_draw_rule, "bordered table should emit DrawRule");
        let has_glyph = gir_page
            .iter()
            .any(|c| c.opcode() == ldir_ir::gir::GIROpcode::PutGlyph);
        assert!(has_glyph, "table cell content should have PutGlyph");
    }

    #[test]
    fn test_thematic_break() {
        let mut tb = ldir_ir::lir::LIRThematicBreak::new();
        tb.geometry = LIRGeometry::new(
            Fp266::from_int(72),
            Fp266::from_int(200),
            Fp266::from_int(468),
            Fp266::from_int(1),
        );

        let mut flow = LIRFlow::new(FlowDirection::TopToBottom);
        flow.children.push(LIRNode::ThematicBreak(tb));

        let page = make_page_with_children(vec![LIRNode::Flow(flow)]);

        let mut lir_doc = LIRDocument::default();
        lir_doc.pages.push(page);

        let gir_doc = render_lir_to_gir(&lir_doc);
        assert_eq!(gir_doc.page_count(), 1);
        assert!(gir_doc.is_well_formed());

        let has_draw_rule = gir_doc[0]
            .iter()
            .any(|c| c.opcode() == ldir_ir::gir::GIROpcode::DrawRule);
        assert!(has_draw_rule);
    }

    #[test]
    fn test_multiple_pages() {
        let mut lir_doc = LIRDocument::default();
        lir_doc.pages.push(make_page_with_children(vec![]));
        lir_doc.pages.push(make_page_with_children(vec![]));
        lir_doc.pages.push(make_page_with_children(vec![]));

        let gir_doc = render_lir_to_gir(&lir_doc);
        assert_eq!(gir_doc.page_count(), 3);
        assert!(gir_doc.is_well_formed());
    }

    #[test]
    fn test_page_dimensions() {
        let page = make_page_with_children(vec![]);
        let mut lir_doc = LIRDocument::default();
        lir_doc.pages.push(page);

        let gir_doc = render_lir_to_gir(&lir_doc);
        assert_eq!(gir_doc[0].width, 612 * 64);
        assert_eq!(gir_doc[0].height, 792 * 64);
    }
}
