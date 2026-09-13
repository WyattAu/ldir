#![deny(unsafe_code)]

use pdf_writer::types::StructRole;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Logical structure element types for PDF tagged (accessible) structure trees.
pub enum StructureType {
    /// The document root.
    Document,
    /// A part (grouping) element.
    Part,
    /// A chapter grouping (custom role).
    Chapter,
    /// A section grouping.
    Section,
    /// A subsection grouping (custom role).
    Subsection,
    /// A level-1 heading.
    H1,
    /// A level-2 heading.
    H2,
    /// A level-3 heading.
    H3,
    /// A level-4 heading.
    H4,
    /// A level-5 heading.
    H5,
    /// A level-6 heading.
    H6,
    /// A paragraph.
    Paragraph,
    /// A list container.
    List,
    /// A single list entry.
    ListItem,
    /// The label (bullet or number) of a list entry.
    ListLabel,
    /// The content of a list entry.
    ListBody,
    /// A table.
    Table,
    /// A table header row group.
    TableHeader,
    /// A table body row group.
    TableBody,
    /// A table row.
    TableRow,
    /// A header cell.
    TableHeaderCell,
    /// A data cell.
    TableDataCell,
    /// A figure.
    Figure,
    /// A caption.
    Caption,
    /// A code block (custom role).
    CodeBlock,
    /// A block quotation.
    BlockQuote,
    /// A display math block (custom role).
    MathBlock,
    /// A footnote.
    Footnote,
    /// An in-text footnote reference.
    FootnoteRef,
    /// The body of a footnote (custom role).
    FootnoteBody,
    /// A table of contents.
    TOC,
    /// A thematic break (custom role, non-structural).
    ThematicBreak,
    /// An inline span.
    Span,
    /// Decorative content excluded from reading order.
    Artifact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A heading level 1-6, clamped on construction.
pub struct HeadingLevel(pub u8);

impl HeadingLevel {
    /// Creates a heading level, clamping to 1-6.
    pub fn new(level: u8) -> Self {
        Self(level.clamp(1, 6))
    }

    /// The corresponding [`StructureType`] heading variant.
    pub fn to_structure_type(self) -> StructureType {
        match self.0 {
            1 => StructureType::H1,
            2 => StructureType::H2,
            3 => StructureType::H3,
            4 => StructureType::H4,
            5 => StructureType::H5,
            _ => StructureType::H6,
        }
    }

    /// The `pdf-writer` [`StructRole`] for this heading level.
    pub fn to_struct_role(self) -> StructRole {
        match self.0 {
            1 => StructRole::H1,
            2 => StructRole::H2,
            3 => StructRole::H3,
            4 => StructRole::H4,
            5 => StructRole::H5,
            _ => StructRole::H6,
        }
    }
}

impl StructureType {
    /// Maps to the standard PDF [`StructRole`], using `NonStruct` when none exists.
    pub fn to_struct_role(self) -> StructRole {
        match self {
            Self::Document => StructRole::Document,
            Self::Part => StructRole::Part,
            Self::Chapter => StructRole::Sect,
            Self::Section => StructRole::Sect,
            Self::Subsection => StructRole::Sect,
            Self::H1 => StructRole::H1,
            Self::H2 => StructRole::H2,
            Self::H3 => StructRole::H3,
            Self::H4 => StructRole::H4,
            Self::H5 => StructRole::H5,
            Self::H6 => StructRole::H6,
            Self::Paragraph => StructRole::P,
            Self::List => StructRole::L,
            Self::ListItem => StructRole::LI,
            Self::ListLabel => StructRole::Lbl,
            Self::ListBody => StructRole::LBody,
            Self::Table => StructRole::Table,
            Self::TableHeader => StructRole::THead,
            Self::TableBody => StructRole::TBody,
            Self::TableRow => StructRole::TR,
            Self::TableHeaderCell => StructRole::TH,
            Self::TableDataCell => StructRole::TD,
            Self::Figure => StructRole::Figure,
            Self::Caption => StructRole::Caption,
            Self::CodeBlock => StructRole::Code,
            Self::BlockQuote => StructRole::BlockQuote,
            Self::MathBlock => StructRole::Formula,
            Self::Footnote => StructRole::Note,
            Self::FootnoteRef => StructRole::Reference,
            Self::FootnoteBody => StructRole::Note,
            Self::TOC => StructRole::TOC,
            Self::ThematicBreak => StructRole::NonStruct,
            Self::Span => StructRole::Span,
            Self::Artifact => StructRole::NonStruct,
        }
    }

    /// Custom role name for types with no exact standard counterpart (`Chapter`, `CodeBlock`, ...).
    pub fn custom_role_name(self) -> Option<&'static [u8]> {
        match self {
            Self::Chapter => Some(b"Chapter"),
            Self::Subsection => Some(b"Subsection"),
            Self::CodeBlock => Some(b"CodeBlock"),
            Self::MathBlock => Some(b"MathBlock"),
            Self::FootnoteBody => Some(b"FootnoteBody"),
            Self::ThematicBreak => Some(b"ThematicBreak"),
            Self::Artifact => Some(b"Artifact"),
            _ => None,
        }
    }

    /// Whether this is one of the `H1`-`H6` heading types.
    pub fn is_heading(self) -> bool {
        matches!(
            self,
            Self::H1 | Self::H2 | Self::H3 | Self::H4 | Self::H5 | Self::H6
        )
    }

    /// Whether this is an artifact (excluded from reading order).
    pub fn is_artifact(self) -> bool {
        self == Self::Artifact
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
/// A bounding box on the page.
pub struct BBox {
    /// Left edge.
    pub x: f32,
    /// Bottom edge.
    pub y: f32,
    /// Width.
    pub width: f32,
    /// Height.
    pub height: f32,
}

impl BBox {
    /// Creates a box from position and size.
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Debug, Clone)]
/// A run of text with a specific language.
pub struct LanguageSpan {
    /// The span text.
    pub text: String,
    /// BCP 47 language tag.
    pub lang: String,
}

#[derive(Debug, Clone)]
/// A node in the tagged-PDF structure tree.
pub struct StructureNode {
    /// Logical type of the element.
    pub element_type: StructureType,
    /// Child nodes.
    pub children: Vec<StructureNode>,
    /// Alternative description (e.g., for figures).
    pub alt_text: Option<String>,
    /// Exact text replacing descendants during extraction.
    pub actual_text: Option<String>,
    /// Expanded form of an acronym or abbreviation.
    pub expanded_text: Option<String>,
    /// Language override for the whole element (BCP 47).
    pub language: Option<String>,
    /// Sub-runs with their own language.
    pub language_spans: Vec<LanguageSpan>,
    /// Page the element is marked on.
    pub page: u32,
    /// Marked-content ID within the page content stream.
    pub mcid: u32,
    /// Position in logical reading order among leaves.
    pub reading_order: u32,
    /// Bounding box on the page, if known.
    pub bbox: Option<BBox>,
}

impl StructureNode {
    /// Creates a leaf node of the given type on a page with an MCID.
    pub fn new(element_type: StructureType, page: u32, mcid: u32) -> Self {
        Self {
            element_type,
            children: Vec::new(),
            alt_text: None,
            actual_text: None,
            expanded_text: None,
            language: None,
            language_spans: Vec::new(),
            page,
            mcid,
            reading_order: 0,
            bbox: None,
        }
    }

    /// Creates a container node with children (page and MCID unset).
    pub fn with_children(element_type: StructureType, children: Vec<StructureNode>) -> Self {
        Self {
            element_type,
            children,
            alt_text: None,
            actual_text: None,
            expanded_text: None,
            language: None,
            language_spans: Vec::new(),
            page: 0,
            mcid: 0,
            reading_order: 0,
            bbox: None,
        }
    }

    /// Builder: sets the alternative description.
    pub fn with_alt_text(mut self, text: impl Into<String>) -> Self {
        self.alt_text = Some(text.into());
        self
    }

    /// Builder: sets the exact text.
    pub fn with_actual_text(mut self, text: impl Into<String>) -> Self {
        self.actual_text = Some(text.into());
        self
    }

    /// Builder: sets the expanded form.
    pub fn with_expanded_text(mut self, text: impl Into<String>) -> Self {
        self.expanded_text = Some(text.into());
        self
    }

    /// Builder: sets the element language.
    pub fn with_language(mut self, lang: impl Into<String>) -> Self {
        self.language = Some(lang.into());
        self
    }

    /// Builder: appends a language-specific sub-run.
    pub fn with_language_span(mut self, text: impl Into<String>, lang: impl Into<String>) -> Self {
        self.language_spans.push(LanguageSpan {
            text: text.into(),
            lang: lang.into(),
        });
        self
    }

    /// Builder: sets the bounding box.
    pub fn with_bbox(mut self, bbox: BBox) -> Self {
        self.bbox = Some(bbox);
        self
    }

    /// Builder: sets the reading order explicitly.
    pub fn with_reading_order(mut self, order: u32) -> Self {
        self.reading_order = order;
        self
    }

    /// Whether the node has no children.
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    /// Numbers leaf nodes in document order, skipping artifacts; returns the number of leaves.
    pub fn assign_reading_order(&mut self) -> u32 {
        let mut counter = 0u32;
        self.assign_reading_order_inner(&mut counter);
        counter
    }

    fn assign_reading_order_inner(&mut self, counter: &mut u32) {
        if self.element_type.is_artifact() {
            return;
        }
        if self.is_leaf() {
            self.reading_order = *counter;
            *counter += 1;
        }
        for child in &mut self.children {
            child.assign_reading_order_inner(counter);
        }
    }

    /// Concatenates the subtree text, preferring `actual_text` and language spans.
    pub fn collect_text(&self) -> String {
        let mut parts = Vec::new();
        self.collect_text_inner(&mut parts);
        parts.join("")
    }

    fn collect_text_inner(&self, parts: &mut Vec<String>) {
        if let Some(ref text) = self.actual_text
            && !text.is_empty()
        {
            parts.push(text.clone());
            return;
        }
        if !self.language_spans.is_empty() {
            for span in &self.language_spans {
                parts.push(span.text.clone());
            }
            return;
        }
        if !self.children.is_empty() {
            for child in &self.children {
                child.collect_text_inner(parts);
            }
        }
    }
}

/// Builds a heading leaf with actual text.
pub fn heading(level: u8, text: &str, page: u32, mcid: u32) -> StructureNode {
    StructureNode::new(HeadingLevel::new(level).to_structure_type(), page, mcid)
        .with_actual_text(text.to_string())
}

/// Builds a paragraph leaf with actual text.
pub fn paragraph(text: &str, page: u32, mcid: u32) -> StructureNode {
    StructureNode::new(StructureType::Paragraph, page, mcid).with_actual_text(text.to_string())
}

/// Builds a list entry with a labeled marker and body.
pub fn list_item(
    label: &str,
    body: &str,
    page: u32,
    label_mcid: u32,
    body_mcid: u32,
) -> StructureNode {
    StructureNode::with_children(
        StructureType::ListItem,
        vec![
            StructureNode::new(StructureType::ListLabel, page, label_mcid)
                .with_actual_text(label.to_string()),
            StructureNode::new(StructureType::ListBody, page, body_mcid)
                .with_actual_text(body.to_string()),
        ],
    )
}

/// Builds a table row from `(cell type, text)` pairs with sequential MCIDs.
pub fn table_row(cells: Vec<(StructureType, &str)>, page: u32, mcid_start: u32) -> StructureNode {
    let children: Vec<StructureNode> = cells
        .into_iter()
        .enumerate()
        .map(|(i, (cell_type, text))| {
            StructureNode::new(cell_type, page, mcid_start + i as u32)
                .with_actual_text(text.to_string())
        })
        .collect();
    StructureNode::with_children(StructureType::TableRow, children)
}

/// Builds a full table with a header row group and body rows.
pub fn table_with_header(
    headers: Vec<&str>,
    rows: Vec<Vec<&str>>,
    page: u32,
    mcid_start: u32,
) -> StructureNode {
    let mut mcid = mcid_start;
    let mut children = Vec::new();

    let header_row: Vec<StructureNode> = headers
        .iter()
        .enumerate()
        .map(|(i, text)| {
            StructureNode::new(StructureType::TableHeaderCell, page, mcid + i as u32)
                .with_actual_text((*text).to_string())
        })
        .collect();
    mcid += headers.len() as u32;

    let header_group = StructureNode::with_children(
        StructureType::TableHeader,
        vec![StructureNode::with_children(
            StructureType::TableRow,
            header_row,
        )],
    );
    children.push(header_group);

    let mut body_rows = Vec::new();
    for row_data in &rows {
        let row_cells: Vec<StructureNode> = row_data
            .iter()
            .enumerate()
            .map(|(i, text)| {
                StructureNode::new(StructureType::TableDataCell, page, mcid + i as u32)
                    .with_actual_text((*text).to_string())
            })
            .collect();
        mcid += row_data.len() as u32;
        body_rows.push(StructureNode::with_children(
            StructureType::TableRow,
            row_cells,
        ));
    }
    children.push(StructureNode::with_children(
        StructureType::TableBody,
        body_rows,
    ));

    StructureNode::with_children(StructureType::Table, children)
}

/// Builds a figure with alternative text and a caption.
pub fn figure_with_caption(
    alt: &str,
    caption: &str,
    page: u32,
    fig_mcid: u32,
    cap_mcid: u32,
) -> StructureNode {
    StructureNode::with_children(
        StructureType::Figure,
        vec![
            StructureNode::new(StructureType::Paragraph, page, fig_mcid).with_alt_text(alt),
            StructureNode::new(StructureType::Caption, page, cap_mcid)
                .with_actual_text(caption.to_string()),
        ],
    )
}

/// Builds a footnote reference and its body.
pub fn footnote_pair(
    ref_text: &str,
    body_text: &str,
    page: u32,
    ref_mcid: u32,
    body_mcid: u32,
) -> (StructureNode, StructureNode) {
    let footnote_ref = StructureNode::new(StructureType::FootnoteRef, page, ref_mcid)
        .with_actual_text(ref_text.to_string());
    let footnote_body = StructureNode::new(StructureType::FootnoteBody, page, body_mcid)
        .with_actual_text(body_text.to_string());
    (footnote_ref, footnote_body)
}

/// Builds an inline span tagged with a language.
pub fn language_span_node(text: &str, lang: &str, page: u32, mcid: u32) -> StructureNode {
    StructureNode::new(StructureType::Span, page, mcid)
        .with_language(lang)
        .with_actual_text(text)
        .with_language_span(text, lang)
}

/// Builds an artifact node (decorative, excluded from reading order).
pub fn artifact_node() -> StructureNode {
    StructureNode::new(StructureType::Artifact, 0, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn structure_tree_creation() {
        let doc = StructureNode::with_children(
            StructureType::Document,
            vec![
                StructureNode::with_children(
                    StructureType::H1,
                    vec![StructureNode::new(StructureType::Paragraph, 1, 0)],
                ),
                StructureNode::new(StructureType::Paragraph, 1, 1),
            ],
        );
        assert_eq!(doc.element_type, StructureType::Document);
        assert_eq!(doc.children.len(), 2);
        assert_eq!(doc.children[1].element_type, StructureType::Paragraph);
    }

    #[test]
    fn structure_type_from_node_type() {
        assert_eq!(
            StructureType::Document.to_struct_role(),
            StructRole::Document
        );
        assert_eq!(StructureType::Paragraph.to_struct_role(), StructRole::P);
        assert_eq!(StructureType::Chapter.to_struct_role(), StructRole::Sect);
        assert_eq!(StructureType::List.to_struct_role(), StructRole::L);
        assert_eq!(StructureType::ListItem.to_struct_role(), StructRole::LI);
        assert_eq!(StructureType::Table.to_struct_role(), StructRole::Table);
        assert_eq!(StructureType::Figure.to_struct_role(), StructRole::Figure);
        assert_eq!(StructureType::Caption.to_struct_role(), StructRole::Caption);
        assert_eq!(StructureType::CodeBlock.to_struct_role(), StructRole::Code);
        assert_eq!(
            StructureType::BlockQuote.to_struct_role(),
            StructRole::BlockQuote
        );
        assert_eq!(
            StructureType::MathBlock.to_struct_role(),
            StructRole::Formula
        );
        assert_eq!(StructureType::TOC.to_struct_role(), StructRole::TOC);
    }

    #[test]
    fn custom_role_names() {
        assert_eq!(
            StructureType::Chapter.custom_role_name(),
            Some(b"Chapter".as_slice())
        );
        assert_eq!(
            StructureType::Subsection.custom_role_name(),
            Some(b"Subsection".as_slice())
        );
        assert_eq!(StructureType::Document.custom_role_name(), None);
        assert_eq!(StructureType::Paragraph.custom_role_name(), None);
    }

    #[test]
    fn alt_text_on_figure() {
        let fig = StructureNode::new(StructureType::Figure, 1, 0).with_alt_text("A diagram");
        assert_eq!(fig.alt_text.as_deref(), Some("A diagram"));
        assert!(fig.is_leaf());
    }

    #[test]
    fn nested_structure() {
        let doc = StructureNode::with_children(
            StructureType::Document,
            vec![StructureNode::with_children(
                StructureType::Section,
                vec![
                    StructureNode::new(StructureType::Paragraph, 1, 0),
                    StructureNode::new(StructureType::Paragraph, 1, 1),
                ],
            )],
        );
        let section = &doc.children[0];
        assert_eq!(section.element_type, StructureType::Section);
        assert_eq!(section.children.len(), 2);
        assert_eq!(section.children[0].mcid, 0);
        assert_eq!(section.children[1].mcid, 1);
    }

    #[test]
    fn empty_structure_tree() {
        let doc = StructureNode::new(StructureType::Document, 0, 0);
        assert!(doc.children.is_empty());
        assert!(doc.is_leaf());
    }

    #[test]
    fn heading_levels_produce_correct_roles() {
        assert_eq!(StructureType::H1.to_struct_role(), StructRole::H1);
        assert_eq!(StructureType::H2.to_struct_role(), StructRole::H2);
        assert_eq!(StructureType::H3.to_struct_role(), StructRole::H3);
        assert_eq!(StructureType::H4.to_struct_role(), StructRole::H4);
        assert_eq!(StructureType::H5.to_struct_role(), StructRole::H5);
        assert_eq!(StructureType::H6.to_struct_role(), StructRole::H6);
    }

    #[test]
    fn heading_level_from_number() {
        assert_eq!(HeadingLevel::new(1).to_structure_type(), StructureType::H1);
        assert_eq!(HeadingLevel::new(3).to_structure_type(), StructureType::H3);
        assert_eq!(HeadingLevel::new(6).to_structure_type(), StructureType::H6);
        assert_eq!(HeadingLevel::new(0).to_structure_type(), StructureType::H1);
        assert_eq!(HeadingLevel::new(99).to_structure_type(), StructureType::H6);
    }

    #[test]
    fn document_with_headings_produces_h1_h6_structure() {
        let doc = StructureNode::with_children(
            StructureType::Document,
            vec![
                heading(1, "Title", 1, 0),
                heading(2, "Chapter 1", 1, 1),
                heading(3, "Section 1.1", 1, 2),
                heading(4, "Subsection 1.1.1", 1, 3),
                heading(5, "Detail 1.1.1.1", 1, 4),
                heading(6, "Note 1.1.1.1.1", 1, 5),
            ],
        );
        assert_eq!(doc.children[0].element_type, StructureType::H1);
        assert_eq!(doc.children[1].element_type, StructureType::H2);
        assert_eq!(doc.children[2].element_type, StructureType::H3);
        assert_eq!(doc.children[3].element_type, StructureType::H4);
        assert_eq!(doc.children[4].element_type, StructureType::H5);
        assert_eq!(doc.children[5].element_type, StructureType::H6);

        for child in &doc.children {
            assert!(child.element_type.is_heading());
        }
        assert!(!StructureType::Paragraph.is_heading());
    }

    #[test]
    fn table_produces_proper_tr_th_td_nesting() {
        let table = table_with_header(
            vec!["Name", "Value"],
            vec![vec!["foo", "1"], vec!["bar", "2"]],
            1,
            0,
        );
        assert_eq!(table.element_type, StructureType::Table);
        assert_eq!(table.children.len(), 2);

        let thead = &table.children[0];
        assert_eq!(thead.element_type, StructureType::TableHeader);
        assert_eq!(thead.children.len(), 1);
        let header_row = &thead.children[0];
        assert_eq!(header_row.element_type, StructureType::TableRow);
        assert_eq!(header_row.children.len(), 2);
        assert_eq!(
            header_row.children[0].element_type,
            StructureType::TableHeaderCell
        );
        assert_eq!(
            header_row.children[1].element_type,
            StructureType::TableHeaderCell
        );
        assert_eq!(header_row.children[0].actual_text.as_deref(), Some("Name"));

        let tbody = &table.children[1];
        assert_eq!(tbody.element_type, StructureType::TableBody);
        assert_eq!(tbody.children.len(), 2);
        assert_eq!(tbody.children[0].element_type, StructureType::TableRow);
        assert_eq!(
            tbody.children[0].children[0].element_type,
            StructureType::TableDataCell
        );
        assert_eq!(
            tbody.children[0].children[0].actual_text.as_deref(),
            Some("foo")
        );
    }

    #[test]
    fn list_item_has_label_and_body() {
        let li = list_item("1.", "First item", 1, 0, 1);
        assert_eq!(li.element_type, StructureType::ListItem);
        assert_eq!(li.children.len(), 2);
        assert_eq!(li.children[0].element_type, StructureType::ListLabel);
        assert_eq!(li.children[0].actual_text.as_deref(), Some("1."));
        assert_eq!(li.children[1].element_type, StructureType::ListBody);
        assert_eq!(li.children[1].actual_text.as_deref(), Some("First item"));
    }

    #[test]
    fn figure_with_caption_linked() {
        let fig = figure_with_caption("Sunset photo", "Figure 1: A beautiful sunset", 1, 0, 1);
        assert_eq!(fig.element_type, StructureType::Figure);
        assert_eq!(fig.children.len(), 2);
        assert_eq!(fig.children[0].alt_text.as_deref(), Some("Sunset photo"));
        assert_eq!(fig.children[1].element_type, StructureType::Caption);
        assert_eq!(
            fig.children[1].actual_text.as_deref(),
            Some("Figure 1: A beautiful sunset")
        );
    }

    #[test]
    fn footnote_pair_linked() {
        let (ref_node, body_node) = footnote_pair("[1]", "Footnote 1 text", 1, 0, 1);
        assert_eq!(ref_node.element_type, StructureType::FootnoteRef);
        assert_eq!(ref_node.actual_text.as_deref(), Some("[1]"));
        assert_eq!(body_node.element_type, StructureType::FootnoteBody);
        assert_eq!(body_node.actual_text.as_deref(), Some("Footnote 1 text"));
    }

    #[test]
    fn language_span_marks_language() {
        let span = language_span_node("Bonjour", "fr", 1, 0);
        assert_eq!(span.element_type, StructureType::Span);
        assert_eq!(span.language.as_deref(), Some("fr"));
        assert_eq!(span.language_spans.len(), 1);
        assert_eq!(span.language_spans[0].text, "Bonjour");
        assert_eq!(span.language_spans[0].lang, "fr");
    }

    #[test]
    fn language_spans_within_paragraph() {
        let para = StructureNode::with_children(
            StructureType::Paragraph,
            vec![
                StructureNode::new(StructureType::Span, 1, 0).with_actual_text("The French say "),
                language_span_node("Bonjour", "fr", 1, 1),
            ],
        );
        let text = para.collect_text();
        assert_eq!(text, "The French say Bonjour");
    }

    #[test]
    fn reading_order_is_sequential() {
        let mut doc = StructureNode::with_children(
            StructureType::Document,
            vec![
                heading(1, "Title", 1, 0),
                paragraph("Body text", 1, 1),
                paragraph("More text", 1, 2),
            ],
        );
        let count = doc.assign_reading_order();
        assert_eq!(count, 3);
        assert_eq!(doc.children[0].reading_order, 0);
        assert_eq!(doc.children[1].reading_order, 1);
        assert_eq!(doc.children[2].reading_order, 2);
    }

    #[test]
    fn reading_order_skips_artifacts() {
        let mut doc = StructureNode::with_children(
            StructureType::Document,
            vec![
                paragraph("First", 1, 0),
                artifact_node(),
                paragraph("Second", 1, 2),
            ],
        );
        let count = doc.assign_reading_order();
        assert_eq!(count, 2);
        assert_eq!(doc.children[0].reading_order, 0);
        assert_eq!(doc.children[1].reading_order, 0);
        assert_eq!(doc.children[2].reading_order, 1);
    }

    #[test]
    fn reading_order_nested_structures() {
        let mut doc = StructureNode::with_children(
            StructureType::Document,
            vec![
                heading(1, "Title", 1, 0),
                list_item("1.", "Item", 1, 1, 2),
                paragraph("After list", 1, 3),
            ],
        );
        doc.assign_reading_order();
        assert_eq!(doc.children[0].reading_order, 0);
        let li = &doc.children[1];
        assert_eq!(li.children[0].reading_order, 1);
        assert_eq!(li.children[1].reading_order, 2);
        assert_eq!(doc.children[2].reading_order, 3);
    }

    #[test]
    fn collect_text_from_actual_text() {
        let node = paragraph("Hello world", 1, 0);
        assert_eq!(node.collect_text(), "Hello world");
    }

    #[test]
    fn collect_text_from_children() {
        let node = StructureNode::with_children(
            StructureType::Paragraph,
            vec![
                StructureNode::new(StructureType::Span, 1, 0).with_actual_text("Hello "),
                StructureNode::new(StructureType::Span, 1, 1).with_actual_text("world"),
            ],
        );
        assert_eq!(node.collect_text(), "Hello world");
    }

    #[test]
    fn is_heading() {
        assert!(StructureType::H1.is_heading());
        assert!(StructureType::H6.is_heading());
        assert!(!StructureType::Paragraph.is_heading());
        assert!(!StructureType::Table.is_heading());
    }

    #[test]
    fn is_artifact() {
        assert!(StructureType::Artifact.is_artifact());
        assert!(!StructureType::Paragraph.is_artifact());
        assert!(!StructureType::ThematicBreak.is_artifact());
    }

    #[test]
    fn bbox_on_structure_node() {
        let node = StructureNode::new(StructureType::Figure, 1, 0)
            .with_bbox(BBox::new(72.0, 700.0, 100.0, 50.0));
        let bbox = node.bbox.as_ref().unwrap();
        assert!((bbox.x - 72.0).abs() < f32::EPSILON);
        assert!((bbox.width - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn actual_text_and_expanded_text() {
        let node = StructureNode::new(StructureType::Span, 1, 0)
            .with_actual_text("HTML")
            .with_expanded_text("Hypertext Markup Language");
        assert_eq!(node.actual_text.as_deref(), Some("HTML"));
        assert_eq!(
            node.expanded_text.as_deref(),
            Some("Hypertext Markup Language")
        );
    }

    #[test]
    fn table_row_helper() {
        let row = table_row(
            vec![
                (StructureType::TableHeaderCell, "A"),
                (StructureType::TableHeaderCell, "B"),
            ],
            1,
            0,
        );
        assert_eq!(row.element_type, StructureType::TableRow);
        assert_eq!(row.children.len(), 2);
        assert_eq!(row.children[0].mcid, 0);
        assert_eq!(row.children[1].mcid, 1);
        assert_eq!(row.children[0].actual_text.as_deref(), Some("A"));
    }

    #[test]
    fn list_label_and_body_roles() {
        assert_eq!(StructureType::ListLabel.to_struct_role(), StructRole::Lbl);
        assert_eq!(StructureType::ListBody.to_struct_role(), StructRole::LBody);
        assert_eq!(
            StructureType::TableHeaderCell.to_struct_role(),
            StructRole::TH
        );
        assert_eq!(
            StructureType::TableDataCell.to_struct_role(),
            StructRole::TD
        );
        assert_eq!(
            StructureType::TableHeader.to_struct_role(),
            StructRole::THead
        );
        assert_eq!(StructureType::TableBody.to_struct_role(), StructRole::TBody);
        assert_eq!(
            StructureType::FootnoteRef.to_struct_role(),
            StructRole::Reference
        );
        assert_eq!(StructureType::Span.to_struct_role(), StructRole::Span);
    }
}
