//! Multi-column layout reflow.
//!
//! Takes a single-column LIR page and distributes blocks across N columns
//! within the available page width. Supports column gap, balanced heights,
//! and column break control.

use ldir_ir::fp266::Fp266;
use ldir_ir::lir::style::FlowDirection;
use ldir_ir::lir::types::*;

/// Configuration for multi-column layout.
#[derive(Debug, Clone, Copy)]
pub struct MultiColumnOptions {
    /// Number of columns (1 = no change, 2+ = reflow).
    pub columns: u8,
    /// Gap between columns in 26.6 fixed-point units.
    pub column_gap: i32,
    /// Whether to balance column heights (equal height).
    pub balanced: bool,
}

impl Default for MultiColumnOptions {
    fn default() -> Self {
        Self {
            columns: 2,
            column_gap: 36,
            balanced: false,
        }
    }
}

/// Check if a node is a full-width element (heading, table, figure, code block, etc.)
/// that should span all columns.
fn is_full_width_node(node: &LIRNode) -> bool {
    matches!(
        node,
        LIRNode::Heading(_)
            | LIRNode::Table(_)
            | LIRNode::Figure(_)
            | LIRNode::CodeBlock(_)
            | LIRNode::MathBlock(_)
            | LIRNode::ThematicBreak(_)
            | LIRNode::TableOfContents(_)
            | LIRNode::Bibliography(_)
            | LIRNode::FootnoteBlock(_)
    )
}

/// Get the height of a node.
fn node_height(node: &LIRNode) -> Fp266 {
    node.geometry().height
}

/// Get the y position of a node relative to content area top.
fn node_y(node: &LIRNode) -> Fp266 {
    node.geometry().y
}

/// Calculate total content height from all children.
fn total_content_height(children: &[LIRNode]) -> Fp266 {
    let mut max_bottom = Fp266::ZERO;
    for child in children {
        let g = child.geometry();
        let bottom = g.y + g.height;
        if bottom > max_bottom {
            max_bottom = bottom;
        }
    }
    let min_top = children
        .iter()
        .map(|c| c.geometry().y)
        .min()
        .unwrap_or(Fp266::ZERO);
    if max_bottom > min_top {
        max_bottom - min_top
    } else {
        Fp266::ZERO
    }
}

/// Offset the x position of a node and all its descendants by `dx`.
fn offset_node_x(node: &mut LIRNode, dx: Fp266) {
    match node {
        LIRNode::Document(n) => n.geometry.x += dx,
        LIRNode::Page(n) => n.geometry.x += dx,
        LIRNode::Flow(n) => {
            n.geometry.x += dx;
            for child in &mut n.children {
                offset_node_x(child, dx);
            }
        }
        LIRNode::Paragraph(n) => {
            n.geometry.x += dx;
            for child in &mut n.children {
                offset_node_x(child, dx);
            }
        }
        LIRNode::Line(n) => {
            n.geometry.x += dx;
            for child in &mut n.children {
                offset_node_x(child, dx);
            }
        }
        LIRNode::Glyph(n) => n.geometry.x += dx,
        LIRNode::Space(n) => n.geometry.x += dx,
        LIRNode::Heading(n) => {
            n.geometry.x += dx;
            for child in &mut n.children {
                offset_node_x(child, dx);
            }
        }
        LIRNode::List(n) => {
            n.geometry.x += dx;
            for child in &mut n.children {
                offset_node_x(child, dx);
            }
        }
        LIRNode::ListItem(n) => {
            n.geometry.x += dx;
            for child in &mut n.children {
                offset_node_x(child, dx);
            }
        }
        LIRNode::Table(n) => {
            n.geometry.x += dx;
            for child in &mut n.children {
                offset_node_x(child, dx);
            }
        }
        LIRNode::TableRow(n) => {
            n.geometry.x += dx;
            for child in &mut n.children {
                offset_node_x(child, dx);
            }
        }
        LIRNode::TableCell(n) => {
            n.geometry.x += dx;
            for child in &mut n.children {
                offset_node_x(child, dx);
            }
        }
        LIRNode::Figure(n) => n.geometry.x += dx,
        LIRNode::Caption(n) => {
            n.geometry.x += dx;
            for child in &mut n.children {
                offset_node_x(child, dx);
            }
        }
        LIRNode::Footnote(n) => n.geometry.x += dx,
        LIRNode::FootnoteBlock(n) => {
            n.geometry.x += dx;
            for child in &mut n.children {
                offset_node_x(child, dx);
            }
        }
        LIRNode::BlockQuote(n) => {
            n.geometry.x += dx;
            for child in &mut n.children {
                offset_node_x(child, dx);
            }
        }
        LIRNode::CodeBlock(n) => {
            n.geometry.x += dx;
            for child in &mut n.children {
                offset_node_x(child, dx);
            }
        }
        LIRNode::MathBlock(n) => {
            n.geometry.x += dx;
            for child in &mut n.children {
                offset_node_x(child, dx);
            }
        }
        LIRNode::ThematicBreak(n) => n.geometry.x += dx,
        LIRNode::TableOfContents(n) => n.geometry.x += dx,
        LIRNode::Bibliography(n) => {
            n.geometry.x += dx;
            for child in &mut n.children {
                offset_node_x(child, dx);
            }
        }
        LIRNode::Citation(n) => n.geometry.x += dx,
        LIRNode::PageBreak(n) => n.geometry.x += dx,
    }
}

/// Shift the y position of a node and all its descendants by `dy`.
fn shift_node_y(node: &mut LIRNode, dy: Fp266) {
    match node {
        LIRNode::Document(n) => n.geometry.y += dy,
        LIRNode::Page(n) => n.geometry.y += dy,
        LIRNode::Flow(n) => {
            n.geometry.y += dy;
            for child in &mut n.children {
                shift_node_y(child, dy);
            }
        }
        LIRNode::Paragraph(n) => {
            n.geometry.y += dy;
            for child in &mut n.children {
                shift_node_y(child, dy);
            }
        }
        LIRNode::Line(n) => {
            n.geometry.y += dy;
            for child in &mut n.children {
                shift_node_y(child, dy);
            }
        }
        LIRNode::Glyph(n) => n.geometry.y += dy,
        LIRNode::Space(n) => n.geometry.y += dy,
        LIRNode::Heading(n) => {
            n.geometry.y += dy;
            for child in &mut n.children {
                shift_node_y(child, dy);
            }
        }
        LIRNode::List(n) => {
            n.geometry.y += dy;
            for child in &mut n.children {
                shift_node_y(child, dy);
            }
        }
        LIRNode::ListItem(n) => {
            n.geometry.y += dy;
            for child in &mut n.children {
                shift_node_y(child, dy);
            }
        }
        LIRNode::Table(n) => {
            n.geometry.y += dy;
            for child in &mut n.children {
                shift_node_y(child, dy);
            }
        }
        LIRNode::TableRow(n) => {
            n.geometry.y += dy;
            for child in &mut n.children {
                shift_node_y(child, dy);
            }
        }
        LIRNode::TableCell(n) => {
            n.geometry.y += dy;
            for child in &mut n.children {
                shift_node_y(child, dy);
            }
        }
        LIRNode::Figure(n) => n.geometry.y += dy,
        LIRNode::Caption(n) => {
            n.geometry.y += dy;
            for child in &mut n.children {
                shift_node_y(child, dy);
            }
        }
        LIRNode::Footnote(n) => n.geometry.y += dy,
        LIRNode::FootnoteBlock(n) => {
            n.geometry.y += dy;
            for child in &mut n.children {
                shift_node_y(child, dy);
            }
        }
        LIRNode::BlockQuote(n) => {
            n.geometry.y += dy;
            for child in &mut n.children {
                shift_node_y(child, dy);
            }
        }
        LIRNode::CodeBlock(n) => {
            n.geometry.y += dy;
            for child in &mut n.children {
                shift_node_y(child, dy);
            }
        }
        LIRNode::MathBlock(n) => {
            n.geometry.y += dy;
            for child in &mut n.children {
                shift_node_y(child, dy);
            }
        }
        LIRNode::ThematicBreak(n) => n.geometry.y += dy,
        LIRNode::TableOfContents(n) => n.geometry.y += dy,
        LIRNode::Bibliography(n) => {
            n.geometry.y += dy;
            for child in &mut n.children {
                shift_node_y(child, dy);
            }
        }
        LIRNode::Citation(n) => n.geometry.y += dy,
        LIRNode::PageBreak(n) => n.geometry.y += dy,
    }
}

/// Reflows a single-column LIR into multi-column layout.
pub fn reflow_multicolumn(page: &LIRPage, options: MultiColumnOptions) -> LIRPage {
    if options.columns <= 1 {
        return page.clone();
    }

    let content_width = page.page_width - page.margin_left - page.margin_right;
    let gap = Fp266::from_int(options.column_gap);
    let num_cols = options.columns as i64;
    let total_gaps = Fp266::from_raw((num_cols - 1) * gap.raw());

    // fp26.6 division: (a/b) in fp26.6 = (a.raw << 6) / b.raw
    const FP_BITS: u32 = 6;
    let divisor = Fp266::from_int(options.columns as i32).raw();
    let col_width_raw = if content_width > total_gaps {
        ((content_width - total_gaps).raw() << FP_BITS) / divisor
    } else {
        (content_width.raw() << FP_BITS) / divisor
    };
    let col_width = Fp266::from_raw(col_width_raw);

    let content_top = page.margin_top;

    // Separate full-width and flowable blocks
    let mut full_width_blocks: Vec<LIRNode> = Vec::new();
    let mut flowable_blocks: Vec<LIRNode> = Vec::new();

    for child in &page.children {
        if is_full_width_node(child) {
            full_width_blocks.push(child.clone());
        } else {
            flowable_blocks.push(child.clone());
        }
    }

    // Calculate column height limit
    let flow_height = total_content_height(&flowable_blocks);
    let column_limit = if options.balanced && !flow_height.is_zero() {
        let per_column = (flow_height.raw() as f64 / num_cols as f64).ceil() as i64;
        Fp266::from_raw(per_column)
    } else {
        page.page_height - page.margin_top - page.margin_bottom
    };

    // Distribute flowable blocks into columns
    let mut columns: Vec<Vec<LIRNode>> = vec![Vec::new(); num_cols as usize];
    let mut column_cursor_y: Vec<Fp266> = vec![content_top; num_cols as usize];
    let mut current_col: usize = 0;

    for mut block in flowable_blocks {
        let block_h = node_height(&block);
        let block_top_y = node_y(&block);

        // Move to next column if this block would exceed the limit
        // (but don't move if the current column is empty -- always place at least one block)
        if column_cursor_y[current_col] + block_h > content_top + column_limit
            && !columns[current_col].is_empty()
            && current_col + 1 < num_cols as usize
        {
            current_col += 1;
        }

        let col_start_x =
            page.margin_left + Fp266::from_int(current_col as i32).mul(col_width + gap);
        let dx = col_start_x - block.geometry().x;
        let dy = column_cursor_y[current_col] - block_top_y;

        offset_node_x(&mut block, dx);
        shift_node_y(&mut block, dy);

        column_cursor_y[current_col] += block_h;
        columns[current_col].push(block);
    }

    // Build the output page
    let mut new_children: Vec<LIRNode> = Vec::new();

    // Build column flows
    for (col_idx, col_blocks) in columns.iter().enumerate() {
        if col_blocks.is_empty() {
            continue;
        }

        let Some(first) = col_blocks.first() else {
            continue;
        };
        let first_y = node_y(first);
        let Some(last) = col_blocks.last() else {
            continue;
        };
        let last_bottom = node_y(last) + node_height(last);
        let flow_h = last_bottom - first_y;

        let col_start_x = page.margin_left + Fp266::from_int(col_idx as i32).mul(col_width + gap);

        let mut flow = LIRFlow::new(FlowDirection::TopToBottom);
        flow.geometry = LIRGeometry::new(col_start_x, first_y, col_width, flow_h);
        flow.children = col_blocks.clone();

        new_children.push(LIRNode::Flow(flow));
    }

    // Append full-width blocks below columns
    let max_col_bottom = if options.balanced {
        content_top + column_limit
    } else {
        column_cursor_y.iter().copied().max().unwrap_or(content_top)
    };

    let mut global_y = max_col_bottom;
    for mut block in full_width_blocks {
        let old_y = node_y(&block);
        let dx = page.margin_left - block.geometry().x;
        let dy = global_y - old_y;
        offset_node_x(&mut block, dx);
        shift_node_y(&mut block, dy);
        global_y += node_height(&block);
        new_children.push(block);
    }

    let mut new_page = page.clone();
    new_page.children = new_children;
    new_page
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldir_ir::lir::style::TextAlign;

    fn make_page_with_content_height(content_height: i32, children: Vec<LIRNode>) -> LIRPage {
        let mut meta = LIRDocumentMeta::default();
        // Set page_height so content_height = page_height - margins (72+72=144)
        meta.page_height = Fp266::from_int(content_height + 144);
        let mut page = LIRPage::new(1, &meta);
        page.page_height = Fp266::from_int(content_height + 144);
        page.children = children;
        page
    }

    fn make_paragraph(id: u32, x: i32, y: i32, w: i32, h: i32) -> LIRNode {
        let mut para = LIRParagraph::new();
        para.id = id;
        para.geometry = LIRGeometry::new(
            Fp266::from_int(x),
            Fp266::from_int(y),
            Fp266::from_int(w),
            Fp266::from_int(h),
        );
        para.text_align = TextAlign::Left;
        LIRNode::Paragraph(para)
    }

    fn make_heading(id: u32, x: i32, y: i32, w: i32, h: i32, level: u8) -> LIRNode {
        let mut heading = LIRHeading::new(level);
        heading.id = id;
        heading.geometry = LIRGeometry::new(
            Fp266::from_int(x),
            Fp266::from_int(y),
            Fp266::from_int(w),
            Fp266::from_int(h),
        );
        LIRNode::Heading(heading)
    }

    fn make_table(id: u32, x: i32, y: i32, w: i32, h: i32) -> LIRNode {
        let mut table = LIRTable::new(1);
        table.id = id;
        table.geometry = LIRGeometry::new(
            Fp266::from_int(x),
            Fp266::from_int(y),
            Fp266::from_int(w),
            Fp266::from_int(h),
        );
        LIRNode::Table(table)
    }

    #[test]
    fn test_single_column_passthrough() {
        let children = vec![
            make_paragraph(1, 72, 72, 468, 24),
            make_paragraph(2, 72, 96, 468, 24),
        ];
        let page = make_page_with_content_height(200, children);

        let result = reflow_multicolumn(
            &page,
            MultiColumnOptions {
                columns: 1,
                ..Default::default()
            },
        );

        assert_eq!(result.children.len(), 2);
    }

    #[test]
    fn test_two_column_basic() {
        // Content height = 250sp. Two columns => 125sp each (approx).
        // 4 paragraphs of 100sp each = 400sp total.
        // First column fills ~125sp (1 para), rest goes to column 2.
        let children = vec![
            make_paragraph(1, 72, 72, 468, 100),
            make_paragraph(2, 72, 172, 468, 100),
            make_paragraph(3, 72, 272, 468, 100),
            make_paragraph(4, 72, 372, 468, 100),
        ];
        let page = make_page_with_content_height(250, children);

        let result = reflow_multicolumn(
            &page,
            MultiColumnOptions {
                columns: 2,
                column_gap: 36,
                balanced: false,
            },
        );

        // Should have 2 column flows
        let flow_count = result
            .children
            .iter()
            .filter(|n| matches!(n, LIRNode::Flow(_)))
            .count();
        assert_eq!(flow_count, 2, "expected 2 column flows");

        // First column should start at margin_left (72)
        if let Some(LIRNode::Flow(f)) = result
            .children
            .iter()
            .find(|n| matches!(n, LIRNode::Flow(_)))
        {
            assert_eq!(f.geometry.x, Fp266::from_int(72));
        }
    }

    #[test]
    fn test_column_gap() {
        let children = vec![
            make_paragraph(1, 72, 72, 468, 200),
            make_paragraph(2, 72, 272, 468, 200),
        ];
        let page = make_page_with_content_height(250, children);

        let gap_width = 48;
        let result = reflow_multicolumn(
            &page,
            MultiColumnOptions {
                columns: 2,
                column_gap: gap_width,
                balanced: false,
            },
        );

        let flows: Vec<_> = result
            .children
            .iter()
            .filter_map(|n| match n {
                LIRNode::Flow(f) => Some(f.clone()),
                _ => None,
            })
            .collect();

        assert_eq!(flows.len(), 2);

        // Second column x should be: margin_left + col_width + gap
        let col_width_raw = ((Fp266::from_int(468) - Fp266::from_int(gap_width)).raw() << 6)
            / Fp266::from_int(2).raw();
        let col_width = Fp266::from_raw(col_width_raw);
        let expected_col2_x = Fp266::from_int(72) + col_width + Fp266::from_int(gap_width);
        assert_eq!(flows[1].geometry.x, expected_col2_x);
    }

    #[test]
    fn test_full_width_elements() {
        let children = vec![
            make_heading(1, 72, 72, 468, 30, 1),
            make_paragraph(2, 72, 102, 468, 100),
            make_paragraph(3, 72, 202, 468, 100),
            make_table(4, 72, 302, 468, 60),
        ];
        let page = make_page_with_content_height(250, children);

        let result = reflow_multicolumn(
            &page,
            MultiColumnOptions {
                columns: 2,
                column_gap: 36,
                balanced: false,
            },
        );

        // Heading and table should NOT be inside column flows
        let has_heading_direct = result
            .children
            .iter()
            .any(|n| matches!(n, LIRNode::Heading(_)));
        let has_table_direct = result
            .children
            .iter()
            .any(|n| matches!(n, LIRNode::Table(_)));

        assert!(
            has_heading_direct,
            "heading should be a direct child (full-width)"
        );
        assert!(
            has_table_direct,
            "table should be a direct child (full-width)"
        );

        // Heading x should be at margin_left
        if let Some(LIRNode::Heading(h)) = result
            .children
            .iter()
            .find(|n| matches!(n, LIRNode::Heading(_)))
        {
            assert_eq!(h.geometry.x, Fp266::from_int(72));
            assert_eq!(h.geometry.width, Fp266::from_int(468));
        }
    }

    #[test]
    fn test_balanced_columns() {
        // Create 6 paragraphs of equal height (total 600sp)
        // With 2 balanced columns, each should hold ~300sp
        let children: Vec<LIRNode> = (0..6u32)
            .map(|i| make_paragraph(i + 1, 72, 72 + (i * 100) as i32, 468, 100))
            .collect();
        let page = make_page_with_content_height(600, children);

        let result = reflow_multicolumn(
            &page,
            MultiColumnOptions {
                columns: 2,
                column_gap: 36,
                balanced: true,
            },
        );

        let flows: Vec<_> = result
            .children
            .iter()
            .filter_map(|n| match n {
                LIRNode::Flow(f) => Some(f.clone()),
                _ => None,
            })
            .collect();

        assert_eq!(flows.len(), 2);

        // Both columns should have similar heights (within ~100sp tolerance)
        let h1 = flows[0].geometry.height;
        let h2 = flows[1].geometry.height;
        let diff = if h1 > h2 { h1 - h2 } else { h2 - h1 };
        // Allow up to 1 paragraph height difference
        assert!(
            diff < Fp266::from_int(110),
            "balanced columns differ too much: {} vs {}",
            h1.to_f64(),
            h2.to_f64()
        );
    }
}
