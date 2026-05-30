//! Multi-column layout reflow.
//!
//! Takes a single-column LIR page and distributes blocks across N columns
//! within the available page width. Supports column gap, balanced heights,
//! column spanning, and column break control.

use std::collections::HashMap;

use ldir_ir::fp266::Fp266;
use ldir_ir::lir::style::{ColumnBreak, FlowDirection, SpanBehavior};
use ldir_ir::lir::types::*;

/// Per-node column layout annotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeColumnAnnotation {
    /// Whether this node should span all columns.
    pub column_span: bool,
    /// Column break control for this node.
    pub column_break: Option<ColumnBreak>,
}

/// Configuration for multi-column layout.
#[derive(Debug, Clone)]
pub struct MultiColumnOptions {
    /// Number of columns (1 = no change, 2+ = reflow).
    pub columns: u8,
    /// Gap between columns in 26.6 fixed-point units.
    pub column_gap: i32,
    /// Whether to balance column heights (equal height).
    pub balanced: bool,
    /// Column spanning behavior.
    pub span_behavior: SpanBehavior,
    /// Node type names that should span full width when SpanBehavior::Designated.
    pub full_width_elements: Vec<&'static str>,
    /// Per-node annotations keyed by node ID.
    pub node_annotations: HashMap<u32, NodeColumnAnnotation>,
}

impl Default for MultiColumnOptions {
    fn default() -> Self {
        Self {
            columns: 2,
            column_gap: 36,
            balanced: false,
            span_behavior: SpanBehavior::Auto,
            full_width_elements: Vec::new(),
            node_annotations: HashMap::new(),
        }
    }
}

/// Check if a node is a known full-width element type (heading, table, figure, etc.).
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

/// Check if a node's content width exceeds the column width (width-based span detection).
fn is_wide_for_column(node: &LIRNode, col_width: Fp266) -> bool {
    let w = node.geometry().width;
    if w.is_zero() || col_width.is_zero() {
        return false;
    }
    // fp26.6: 0.9 * col_width = (col_width.raw() * 9) / 10 (shifted by 6)
    let threshold = Fp266::from_raw((col_width.raw() * 9) >> 3);
    match node {
        LIRNode::Table(_) => w > col_width,
        LIRNode::Figure(_) => w >= threshold,
        _ => false,
    }
}

/// Determine whether a node should span all columns given the options and column width.
fn should_span(node: &LIRNode, col_width: Fp266, options: &MultiColumnOptions) -> bool {
    // Explicit annotation always overrides.
    if let Some(ann) = options.node_annotations.get(&node.id())
        && ann.column_span
    {
        return true;
    }

    match options.span_behavior {
        SpanBehavior::Auto => is_full_width_node(node) || is_wide_for_column(node, col_width),
        SpanBehavior::Never => false,
        SpanBehavior::Designated => options.full_width_elements.contains(&node.type_name()),
    }
}

/// Get the column break annotation for a node, if any.
fn get_column_break(node: &LIRNode, options: &MultiColumnOptions) -> Option<ColumnBreak> {
    options
        .node_annotations
        .get(&node.id())
        .and_then(|ann| ann.column_break)
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

/// A segment of the output page: either a group of column flows or a spanning element.
enum OutputSegment {
    Columns(Vec<LIRNode>),
    Span(LIRNode),
}

/// Build a Flow node for a non-empty column of blocks.
fn build_column_flow(
    col_idx: usize,
    col_blocks: &[LIRNode],
    col_width: Fp266,
    gap: Fp266,
    margin_left: Fp266,
) -> Option<LIRNode> {
    if col_blocks.is_empty() {
        return None;
    }
    let first_y = node_y(&col_blocks[0]);
    let last = col_blocks.last()?;
    let last_bottom = node_y(last) + node_height(last);
    let flow_h = last_bottom - first_y;
    let col_start_x = margin_left + Fp266::from_int(col_idx as i32).mul(col_width + gap);

    let mut flow = LIRFlow::new(FlowDirection::TopToBottom);
    flow.geometry = LIRGeometry::new(col_start_x, first_y, col_width, flow_h);
    flow.children = col_blocks.to_vec();

    Some(LIRNode::Flow(flow))
}

/// Layout context shared across distribution functions.
#[derive(Clone)]
struct LayoutContext<'a> {
    col_width: Fp266,
    gap: Fp266,
    margin_left: Fp266,
    content_top: Fp266,
    num_cols: usize,
    page_content_height: Fp266,
    options: &'a MultiColumnOptions,
}

/// Distribute flowable blocks into columns using a given height limit.
fn distribute_blocks(
    flowable: &[LIRNode],
    ctx: &LayoutContext<'_>,
    height_limit: Fp266,
) -> (Vec<Vec<LIRNode>>, Vec<Fp266>) {
    let mut columns: Vec<Vec<LIRNode>> = vec![Vec::new(); ctx.num_cols];
    let mut column_cursor_y: Vec<Fp266> = vec![ctx.content_top; ctx.num_cols];
    let mut current_col: usize = 0;

    for mut block in flowable.iter().cloned() {
        let block_h = node_height(&block);
        let block_top_y = node_y(&block);
        let break_ctrl = get_column_break(&block, ctx.options);

        if break_ctrl == Some(ColumnBreak::Before)
            && !columns[current_col].is_empty()
            && current_col + 1 < ctx.num_cols
        {
            current_col += 1;
        }

        let overflows = column_cursor_y[current_col] + block_h > ctx.content_top + height_limit;
        if overflows
            && break_ctrl == Some(ColumnBreak::Avoid)
            && !columns[current_col].is_empty()
            && current_col + 1 < ctx.num_cols
        {
            current_col += 1;
        }

        if column_cursor_y[current_col] + block_h > ctx.content_top + height_limit
            && !columns[current_col].is_empty()
            && current_col + 1 < ctx.num_cols
        {
            current_col += 1;
        }

        let col_start_x =
            ctx.margin_left + Fp266::from_int(current_col as i32).mul(ctx.col_width + ctx.gap);
        let dx = col_start_x - block.geometry().x;
        let dy = column_cursor_y[current_col] - block_top_y;

        offset_node_x(&mut block, dx);
        shift_node_y(&mut block, dy);

        column_cursor_y[current_col] += block_h;
        columns[current_col].push(block);
    }

    (columns, column_cursor_y)
}

/// Find the optimal balanced column height via binary search.
fn find_balanced_height(
    flowable: &[LIRNode],
    ctx: &LayoutContext<'_>,
    total_height: Fp266,
) -> Fp266 {
    let mut lo = Fp266::ZERO;
    let mut hi = ctx.page_content_height;

    let ideal = if ctx.num_cols > 0 && !total_height.is_zero() {
        Fp266::from_raw((total_height.raw() as f64 / ctx.num_cols as f64).ceil() as i64)
    } else {
        return ctx.page_content_height;
    };

    for _ in 0..32 {
        let mid = Fp266::from_raw((lo.raw() + hi.raw()) / 2);
        if mid.raw() == lo.raw() || mid.raw() == hi.raw() {
            break;
        }

        let (cols, _) = distribute_blocks(flowable, ctx, mid);

        let cols_used = cols.iter().filter(|c| !c.is_empty()).count();

        if cols_used <= ctx.num_cols {
            hi = mid;
        } else {
            lo = mid;
        }
    }

    let balanced = hi;
    if balanced < ideal { ideal } else { balanced }
}

/// Reflows a single-column LIR into multi-column layout.
pub fn reflow_multicolumn(page: &LIRPage, options: MultiColumnOptions) -> LIRPage {
    if options.columns <= 1 {
        return page.clone();
    }

    let content_width = page.page_width - page.margin_left - page.margin_right;
    let gap = Fp266::from_int(options.column_gap);
    let num_cols = options.columns as i64;
    let num_cols_usize = num_cols as usize;
    let total_gaps = Fp266::from_raw((num_cols - 1) * gap.raw());

    const FP_BITS: u32 = 6;
    let divisor = Fp266::from_int(options.columns as i32).raw();
    let col_width_raw = if content_width > total_gaps {
        ((content_width - total_gaps).raw() << FP_BITS) / divisor
    } else {
        (content_width.raw() << FP_BITS) / divisor
    };
    let col_width = Fp266::from_raw(col_width_raw);

    let content_top = page.margin_top;
    let page_content_height = page.page_height - page.margin_top - page.margin_bottom;

    let ctx = LayoutContext {
        col_width,
        gap,
        margin_left: page.margin_left,
        content_top,
        num_cols: num_cols_usize,
        page_content_height,
        options: &options,
    };

    let mut segments: Vec<OutputSegment> = Vec::new();
    let mut pending_flowable: Vec<LIRNode> = Vec::new();
    let mut next_y = content_top;

    for child in &page.children {
        if should_span(child, col_width, &options) {
            if !pending_flowable.is_empty() {
                let flowable = std::mem::take(&mut pending_flowable);
                let mut seg_ctx = ctx.clone();
                seg_ctx.content_top = next_y;
                let segment = build_column_segment(&flowable, &seg_ctx);
                if let Some(seg) = segment {
                    next_y = seg_max_bottom(&seg, next_y, page_content_height);
                    segments.push(seg);
                }
            }

            let mut span_node = child.clone();
            let dx = page.margin_left - span_node.geometry().x;
            let dy = next_y - node_y(&span_node);
            offset_node_x(&mut span_node, dx);
            shift_node_y(&mut span_node, dy);
            next_y += node_height(&span_node);
            segments.push(OutputSegment::Span(span_node));
        } else {
            pending_flowable.push(child.clone());
        }
    }

    if !pending_flowable.is_empty() {
        let mut seg_ctx = ctx.clone();
        seg_ctx.content_top = next_y;
        let segment = build_column_segment(&pending_flowable, &seg_ctx);
        if let Some(seg) = segment {
            segments.push(seg);
        }
    }

    // Assemble output children from segments.
    let mut new_children: Vec<LIRNode> = Vec::new();
    for seg in &segments {
        match seg {
            OutputSegment::Columns(flows) => new_children.extend(flows.iter().cloned()),
            OutputSegment::Span(node) => new_children.push(node.clone()),
        }
    }

    let mut new_page = page.clone();
    new_page.children = new_children;
    new_page
}

/// Build a column segment from flowable blocks.
fn build_column_segment(flowable: &[LIRNode], ctx: &LayoutContext<'_>) -> Option<OutputSegment> {
    let total_height = total_content_height(flowable);

    let height_limit = if ctx.options.balanced && !total_height.is_zero() {
        find_balanced_height(flowable, ctx, total_height)
    } else {
        ctx.page_content_height
    };

    let (columns, _cursor_y) = distribute_blocks(flowable, ctx, height_limit);

    let mut flows: Vec<LIRNode> = Vec::new();
    for (col_idx, col_blocks) in columns.iter().enumerate() {
        if let Some(flow_node) =
            build_column_flow(col_idx, col_blocks, ctx.col_width, ctx.gap, ctx.margin_left)
        {
            flows.push(flow_node);
        }
    }

    if flows.is_empty() {
        None
    } else {
        Some(OutputSegment::Columns(flows))
    }
}

/// Compute the y position after a column segment ends.
fn seg_max_bottom(seg: &OutputSegment, content_top: Fp266, page_content_height: Fp266) -> Fp266 {
    match seg {
        OutputSegment::Span(node) => node_y(node) + node_height(node),
        OutputSegment::Columns(flows) => {
            let mut max_bottom = content_top;
            for f in flows {
                if let LIRNode::Flow(flow) = f {
                    let bottom = flow.geometry.y + flow.geometry.height;
                    if bottom > max_bottom {
                        max_bottom = bottom;
                    }
                }
            }
            // Ensure we don't exceed page content area.
            let page_bottom = content_top + page_content_height;
            if max_bottom > page_bottom {
                page_bottom
            } else {
                max_bottom
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldir_ir::lir::style::TextAlign;

    fn make_page_with_content_height(content_height: i32, children: Vec<LIRNode>) -> LIRPage {
        let mut meta = LIRDocumentMeta::default();
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

    fn make_blockquote(id: u32, x: i32, y: i32, w: i32, h: i32) -> LIRNode {
        let mut bq = LIRBlockQuote::new();
        bq.id = id;
        bq.geometry = LIRGeometry::new(
            Fp266::from_int(x),
            Fp266::from_int(y),
            Fp266::from_int(w),
            Fp266::from_int(h),
        );
        LIRNode::BlockQuote(bq)
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
                ..Default::default()
            },
        );

        let flow_count = result
            .children
            .iter()
            .filter(|n| matches!(n, LIRNode::Flow(_)))
            .count();
        assert_eq!(flow_count, 2, "expected 2 column flows");

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
                ..Default::default()
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
                ..Default::default()
            },
        );

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

        if let Some(LIRNode::Heading(h)) = result
            .children
            .iter()
            .find(|n| matches!(n, LIRNode::Heading(_)))
        {
            assert_eq!(h.geometry.x, Fp266::from_int(72));
        }
    }

    #[test]
    fn test_balanced_columns() {
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
                ..Default::default()
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

        let h1 = flows[0].geometry.height;
        let h2 = flows[1].geometry.height;
        let diff = if h1 > h2 { h1 - h2 } else { h2 - h1 };
        assert!(
            diff < Fp266::from_int(110),
            "balanced columns differ too much: {} vs {}",
            h1.to_f64(),
            h2.to_f64()
        );
    }

    #[test]
    fn test_column_span_auto_table() {
        // A wide table (wider than single column) should span in Auto mode.
        // Content area is 468sp. With 2 cols and 36 gap, col_width ~ 216sp.
        // Table is 468sp wide, clearly wider than one column.
        let children = vec![
            make_paragraph(1, 72, 72, 468, 100),
            make_paragraph(2, 72, 172, 468, 100),
            make_table(3, 72, 272, 468, 60),
            make_paragraph(4, 72, 332, 468, 100),
        ];
        let page = make_page_with_content_height(500, children);

        let result = reflow_multicolumn(
            &page,
            MultiColumnOptions {
                columns: 2,
                column_gap: 36,
                balanced: false,
                ..Default::default()
            },
        );

        // Table should be a direct page child (not inside a Flow column).
        let has_table_direct = result
            .children
            .iter()
            .any(|n| matches!(n, LIRNode::Table(_)));
        assert!(has_table_direct, "wide table should span in Auto mode");
    }

    #[test]
    fn test_column_span_designated_blockquote() {
        // In Designated mode, only types in full_width_elements should span.
        // BlockQuote is NOT in is_full_width_node, so it only spans when designated.
        let children = vec![
            make_paragraph(1, 72, 72, 468, 100),
            make_blockquote(2, 72, 172, 468, 80),
            make_paragraph(3, 72, 252, 468, 100),
        ];
        let page = make_page_with_content_height(500, children);

        // Without BlockQuote designated -> BlockQuote should be inside a column flow.
        let result_no_designate = reflow_multicolumn(
            &page,
            MultiColumnOptions {
                columns: 2,
                column_gap: 36,
                balanced: false,
                span_behavior: SpanBehavior::Designated,
                full_width_elements: vec![],
                ..Default::default()
            },
        );
        let bq_in_flow = result_no_designate
            .children
            .iter()
            .any(|n| matches!(n, LIRNode::Flow(f) if f.children.iter().any(|c| matches!(c, LIRNode::BlockQuote(_)))));
        assert!(
            bq_in_flow,
            "BlockQuote should be inside a column flow when not designated"
        );

        // With BlockQuote designated -> BlockQuote should be a direct page child.
        let result_designated = reflow_multicolumn(
            &page,
            MultiColumnOptions {
                columns: 2,
                column_gap: 36,
                balanced: false,
                span_behavior: SpanBehavior::Designated,
                full_width_elements: vec!["BlockQuote"],
                ..Default::default()
            },
        );
        let has_bq_direct = result_designated
            .children
            .iter()
            .any(|n| matches!(n, LIRNode::BlockQuote(_)));
        assert!(
            has_bq_direct,
            "BlockQuote should span when designated in Designated mode"
        );
    }

    #[test]
    fn test_column_break_before() {
        // Element with column_break: Before should start a new column.
        // 3 paragraphs: p1(50), p2(50), p3(50). p3 has break-before.
        // With column_limit=100, p1+p2 fill col 1. p3 with break-before goes to col 2.
        let mut annotations = HashMap::new();
        annotations.insert(
            3,
            NodeColumnAnnotation {
                column_span: false,
                column_break: Some(ColumnBreak::Before),
            },
        );

        let children = vec![
            make_paragraph(1, 72, 72, 468, 50),
            make_paragraph(2, 72, 122, 468, 50),
            make_paragraph(3, 72, 172, 468, 50),
        ];
        let page = make_page_with_content_height(500, children);

        let result = reflow_multicolumn(
            &page,
            MultiColumnOptions {
                columns: 2,
                column_gap: 36,
                balanced: false,
                node_annotations: annotations,
                ..Default::default()
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

        // Should have 2 column flows.
        assert_eq!(flows.len(), 2);

        // First column should have p1 and p2.
        assert_eq!(flows[0].children.len(), 2);

        // Second column should have only p3.
        assert_eq!(flows[1].children.len(), 1);
        if let LIRNode::Paragraph(p) = &flows[1].children[0] {
            assert_eq!(p.id, 3);
        }
    }

    #[test]
    fn test_column_break_avoid() {
        // Element with column_break: Avoid should not be split across columns.
        // p1(40), p2(tall=200, avoid break). Column limit=100.
        // p1 fits in col 1 (40 <= 100). p2 is 200 > remaining 60 in col 1,
        // but avoid-break means move entirely to next column.
        let mut annotations = HashMap::new();
        annotations.insert(
            2,
            NodeColumnAnnotation {
                column_span: false,
                column_break: Some(ColumnBreak::Avoid),
            },
        );

        let children = vec![
            make_paragraph(1, 72, 72, 468, 40),
            make_paragraph(2, 72, 112, 468, 200),
        ];
        // Page content height = 100sp, so column_limit = 100sp.
        let page = make_page_with_content_height(100, children);

        let result = reflow_multicolumn(
            &page,
            MultiColumnOptions {
                columns: 2,
                column_gap: 36,
                balanced: false,
                node_annotations: annotations,
                ..Default::default()
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

        // First column has only p1.
        assert_eq!(flows[0].children.len(), 1);

        // Second column has p2 (the avoid-break element).
        assert_eq!(flows[1].children.len(), 1);
    }

    #[test]
    fn test_span_never_mode() {
        // In Never mode, no elements should span (even headings and tables).
        let children = vec![
            make_heading(1, 72, 72, 468, 30, 1),
            make_paragraph(2, 72, 102, 468, 100),
            make_table(3, 72, 202, 468, 60),
        ];
        let page = make_page_with_content_height(500, children);

        let result = reflow_multicolumn(
            &page,
            MultiColumnOptions {
                columns: 2,
                column_gap: 36,
                balanced: false,
                span_behavior: SpanBehavior::Never,
                ..Default::default()
            },
        );

        // All elements should be inside column flows -- no direct page children
        // that are headings or tables.
        let has_direct_heading = result
            .children
            .iter()
            .any(|n| matches!(n, LIRNode::Heading(_)));
        let has_direct_table = result
            .children
            .iter()
            .any(|n| matches!(n, LIRNode::Table(_)));

        assert!(!has_direct_heading, "heading should not span in Never mode");
        assert!(!has_direct_table, "table should not span in Never mode");
    }

    #[test]
    fn test_annotation_column_span_override() {
        // Explicit column_span:true annotation should force spanning
        // even for nodes that are not typically full-width (e.g. a paragraph).
        let mut annotations = HashMap::new();
        annotations.insert(
            2,
            NodeColumnAnnotation {
                column_span: true,
                column_break: None,
            },
        );

        let children = vec![
            make_paragraph(1, 72, 72, 468, 50),
            make_paragraph(2, 72, 122, 468, 30),
            make_paragraph(3, 72, 152, 468, 50),
        ];
        let page = make_page_with_content_height(300, children);

        let result = reflow_multicolumn(
            &page,
            MultiColumnOptions {
                columns: 2,
                column_gap: 36,
                balanced: false,
                span_behavior: SpanBehavior::Never,
                node_annotations: annotations,
                ..Default::default()
            },
        );

        // Paragraph 2 should be a direct child even though Never mode is active.
        let has_direct_para2 = result.children.iter().any(|n| {
            if let LIRNode::Paragraph(p) = n {
                p.id == 2
            } else {
                false
            }
        });
        assert!(
            has_direct_para2,
            "paragraph with column_span:true should span even in Never mode"
        );
    }
}
