use indexmap::IndexMap;
use ldir_ir::sir::v2::SIRModuleV2;
use ldir_ir::sir::v2::annotations::LabelCategory;
use ldir_ir::sir::v2::nodes::NodeType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LabelKind {
    Section { level: u8 },
    Equation,
    Figure,
    Table,
    Theorem,
}

#[derive(Debug, Clone)]
pub struct ResolvedLabel {
    pub label: String,
    pub kind: LabelKind,
    pub number: String,
}

pub fn collect_labels(module: &SIRModuleV2) -> Vec<ResolvedLabel> {
    let mut labels = Vec::new();
    let mut section_counters: IndexMap<u8, u32> = IndexMap::new();
    let mut section_number: Vec<u32> = Vec::new();
    let mut eq_counter: u32 = 0;
    let mut fig_counter: u32 = 0;
    let mut tbl_counter: u32 = 0;

    for &root_id in module.body.roots() {
        collect_labels_recursive(
            root_id,
            module,
            &mut labels,
            &mut section_counters,
            &mut section_number,
            &mut eq_counter,
            &mut fig_counter,
            &mut tbl_counter,
        );
    }

    for (label, info) in &module.annotations.labels {
        if labels.iter().any(|rl| rl.label == *label) {
            continue;
        }
        let kind = match info.category {
            LabelCategory::Section => LabelKind::Section { level: 2 },
            LabelCategory::Equation => LabelKind::Equation,
            LabelCategory::Figure => LabelKind::Figure,
            LabelCategory::Table => LabelKind::Table,
            _ => LabelKind::Section { level: 2 },
        };
        labels.push(ResolvedLabel {
            label: label.clone(),
            kind,
            number: String::new(),
        });
    }

    labels
}

#[allow(clippy::too_many_arguments)]
fn collect_labels_recursive(
    node_id: u32,
    module: &SIRModuleV2,
    labels: &mut Vec<ResolvedLabel>,
    section_counters: &mut IndexMap<u8, u32>,
    section_number: &mut Vec<u32>,
    eq_counter: &mut u32,
    fig_counter: &mut u32,
    tbl_counter: &mut u32,
) {
    let node = match module.body.get(node_id) {
        Some(n) => n,
        None => return,
    };

    match &node.node_type {
        NodeType::Chapter => {
            increment_section_counter(1, section_counters, section_number);
            let num = section_number_string(section_number);
            if let Some(label) = &node.label {
                labels.push(ResolvedLabel {
                    label: label.clone(),
                    kind: LabelKind::Section { level: 1 },
                    number: num,
                });
            }
        }
        NodeType::Section => {
            increment_section_counter(2, section_counters, section_number);
            let num = section_number_string(section_number);
            if let Some(label) = &node.label {
                labels.push(ResolvedLabel {
                    label: label.clone(),
                    kind: LabelKind::Section { level: 2 },
                    number: num,
                });
            }
        }
        NodeType::Subsection => {
            increment_section_counter(3, section_counters, section_number);
            let num = section_number_string(section_number);
            if let Some(label) = &node.label {
                labels.push(ResolvedLabel {
                    label: label.clone(),
                    kind: LabelKind::Section { level: 3 },
                    number: num,
                });
            }
        }
        NodeType::Subsubsection => {
            increment_section_counter(4, section_counters, section_number);
            let num = section_number_string(section_number);
            if let Some(label) = &node.label {
                labels.push(ResolvedLabel {
                    label: label.clone(),
                    kind: LabelKind::Section { level: 4 },
                    number: num,
                });
            }
        }
        NodeType::MathBlock { numbered: true, .. } => {
            *eq_counter += 1;
            if let Some(label) = &node.label {
                labels.push(ResolvedLabel {
                    label: label.clone(),
                    kind: LabelKind::Equation,
                    number: eq_counter.to_string(),
                });
            }
        }
        NodeType::Figure { .. } => {
            *fig_counter += 1;
            if let Some(label) = &node.label {
                labels.push(ResolvedLabel {
                    label: label.clone(),
                    kind: LabelKind::Figure,
                    number: fig_counter.to_string(),
                });
            }
        }
        NodeType::Table { .. } => {
            *tbl_counter += 1;
            if let Some(label) = &node.label {
                labels.push(ResolvedLabel {
                    label: label.clone(),
                    kind: LabelKind::Table,
                    number: tbl_counter.to_string(),
                });
            }
        }
        _ => {}
    }

    for &child_id in &node.child_ids {
        collect_labels_recursive(
            child_id,
            module,
            labels,
            section_counters,
            section_number,
            eq_counter,
            fig_counter,
            tbl_counter,
        );
    }
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

pub fn resolve_references(labels: &[ResolvedLabel]) -> IndexMap<String, String> {
    labels
        .iter()
        .filter(|rl| !rl.number.is_empty())
        .map(|rl| (rl.label.clone(), rl.number.clone()))
        .collect()
}

pub fn resolve_kind_map(labels: &[ResolvedLabel]) -> IndexMap<String, LabelKind> {
    labels
        .iter()
        .map(|rl| (rl.label.clone(), rl.kind.clone()))
        .collect()
}

pub fn resolve_text_references(
    text: &str,
    numbers: &IndexMap<String, String>,
    kinds: &IndexMap<String, LabelKind>,
) -> String {
    let mut result = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'\\' {
            let rest = match text.get(i..) {
                Some(r) => r,
                None => {
                    result.push(bytes[i] as char);
                    i += 1;
                    continue;
                }
            };
            if let Some((label, consumed)) = try_parse_cmd(rest, "\\ref{", 5) {
                let resolved = numbers
                    .get(&label)
                    .filter(|n| !n.is_empty())
                    .cloned()
                    .unwrap_or_else(|| "??".to_string());
                result.push_str(&resolved);
                i += consumed;
                continue;
            }
            if let Some((label, consumed)) = try_parse_cmd(rest, "\\eqref{", 7) {
                let resolved = numbers
                    .get(&label)
                    .filter(|n| !n.is_empty())
                    .map(|n| format!("({})", n))
                    .unwrap_or_else(|| "??".to_string());
                result.push_str(&resolved);
                i += consumed;
                continue;
            }
            if let Some((label, consumed)) = try_parse_cmd(rest, "\\autoref{", 9) {
                if let Some(number) = numbers.get(&label).filter(|n| !n.is_empty()) {
                    let prefix = kinds
                        .get(&label)
                        .map(|k| kind_prefix(k))
                        .unwrap_or_else(|| infer_autoref_prefix(&label));
                    if prefix.is_empty() {
                        result.push_str(number);
                    } else {
                        result.push_str(&format!("{} {}", prefix, number));
                    }
                } else {
                    result.push_str("??");
                }
                i += consumed;
                continue;
            }
        }

        if bytes[i] == b'@' && i + 1 < bytes.len() {
            let rest = &text[i + 1..];
            let label_end = rest
                .find(|c: char| {
                    !c.is_alphanumeric() && c != ':' && c != '_' && c != '-' && c != '.'
                })
                .unwrap_or(rest.len());
            if label_end > 0 {
                let label = &rest[..label_end];
                if let Some(number) = numbers.get(label).filter(|n| !n.is_empty()) {
                    result.push_str(number);
                    i += 1 + label_end;
                    continue;
                }
            }
        }

        result.push(bytes[i] as char);
        i += 1;
    }

    result
}

fn try_parse_cmd(text: &str, prefix: &str, prefix_len: usize) -> Option<(String, usize)> {
    if !text.starts_with(prefix) {
        return None;
    }
    let after_prefix = &text[prefix_len..];
    let end = after_prefix.find('}')?;
    let label = after_prefix[..end].to_string();
    Some((label, prefix_len + end + 1))
}

pub fn kind_prefix(kind: &LabelKind) -> &'static str {
    match kind {
        LabelKind::Section { .. } => "Section",
        LabelKind::Equation => "Equation",
        LabelKind::Figure => "Figure",
        LabelKind::Table => "Table",
        LabelKind::Theorem => "Theorem",
    }
}

pub fn infer_autoref_prefix(label: &str) -> &'static str {
    let prefix = label.split(':').next().unwrap_or("");
    match prefix {
        "sec" | "ch" | "chapter" | "subsec" | "subsection" | "subsubsec" => "Section",
        "eq" | "equation" => "Equation",
        "fig" | "figure" => "Figure",
        "tab" | "table" => "Table",
        "thm" | "theorem" | "lem" | "lemma" | "cor" | "corollary" | "prop" | "proposition"
        | "def" | "definition" => "Theorem",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldir_ir::sir::v2::nodes::{Node, NodeType};

    fn make_section_module(label: &str) -> SIRModuleV2 {
        let mut m = SIRModuleV2::new();
        let doc_id = m.body.push(Node::new(0, NodeType::Document));
        let sec_id = m.body.push(
            Node::new(1, NodeType::Section)
                .with_label(label)
                .with_parent(doc_id),
        );
        let text_id = m.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "Intro".into(),
                },
            )
            .with_parent(sec_id),
        );
        if let Some(d) = m.body.get_mut(doc_id) {
            d.add_child(sec_id);
        }
        if let Some(s) = m.body.get_mut(sec_id) {
            s.add_child(text_id);
        }
        m
    }

    #[test]
    fn test_collect_labels_section() {
        let module = make_section_module("sec:intro");
        let labels = collect_labels(&module);
        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0].label, "sec:intro");
        assert_eq!(labels[0].kind, LabelKind::Section { level: 2 });
        assert_eq!(labels[0].number, "0.1");
    }

    #[test]
    fn test_collect_labels_equation() {
        let mut m = SIRModuleV2::new();
        m.body.push(
            Node::new(
                0,
                NodeType::MathBlock {
                    math_type: ldir_ir::sir::v2::nodes::MathType::Equation,
                    numbered: true,
                },
            )
            .with_label("eq:einstein"),
        );
        let labels = collect_labels(&m);
        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0].label, "eq:einstein");
        assert_eq!(labels[0].kind, LabelKind::Equation);
        assert_eq!(labels[0].number, "1");
    }

    #[test]
    fn test_collect_labels_figure() {
        let mut m = SIRModuleV2::new();
        m.body.push(
            Node::new(
                0,
                NodeType::Figure {
                    placement: ldir_ir::sir::v2::nodes::FloatPlacement::Here,
                },
            )
            .with_label("fig:diagram"),
        );
        let labels = collect_labels(&m);
        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0].label, "fig:diagram");
        assert_eq!(labels[0].kind, LabelKind::Figure);
        assert_eq!(labels[0].number, "1");
    }

    #[test]
    fn test_collect_labels_table() {
        let mut m = SIRModuleV2::new();
        m.body.push(
            Node::new(
                0,
                NodeType::Table {
                    col_specs: vec![],
                    num_cols: 1,
                },
            )
            .with_label("tab:data"),
        );
        let labels = collect_labels(&m);
        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0].label, "tab:data");
        assert_eq!(labels[0].kind, LabelKind::Table);
        assert_eq!(labels[0].number, "1");
    }

    #[test]
    fn test_collect_labels_multi_kind() {
        let mut m = SIRModuleV2::new();
        let doc_id = m.body.push(Node::new(0, NodeType::Document));

        let sec_id = m.body.push(
            Node::new(1, NodeType::Section)
                .with_label("sec:methods")
                .with_parent(doc_id),
        );
        if let Some(d) = m.body.get_mut(doc_id) {
            d.add_child(sec_id);
        }

        m.body.push(
            Node::new(
                2,
                NodeType::MathBlock {
                    math_type: ldir_ir::sir::v2::nodes::MathType::Equation,
                    numbered: true,
                },
            )
            .with_label("eq:pythagoras"),
        );
        m.body.push(
            Node::new(
                3,
                NodeType::Figure {
                    placement: ldir_ir::sir::v2::nodes::FloatPlacement::Here,
                },
            )
            .with_label("fig:results"),
        );

        let labels = collect_labels(&m);
        assert_eq!(labels.len(), 3);

        let sec = labels.iter().find(|l| l.label == "sec:methods").unwrap();
        assert_eq!(sec.kind, LabelKind::Section { level: 2 });
        assert_eq!(sec.number, "0.1");

        let eq = labels.iter().find(|l| l.label == "eq:pythagoras").unwrap();
        assert_eq!(eq.kind, LabelKind::Equation);
        assert_eq!(eq.number, "1");

        let fig = labels.iter().find(|l| l.label == "fig:results").unwrap();
        assert_eq!(fig.kind, LabelKind::Figure);
        assert_eq!(fig.number, "1");
    }

    #[test]
    fn test_collect_labels_section_numbering() {
        let mut m = SIRModuleV2::new();
        let doc_id = m.body.push(Node::new(0, NodeType::Document));

        let sec1 = m.body.push(
            Node::new(1, NodeType::Section)
                .with_label("sec:a")
                .with_parent(doc_id),
        );
        let sec2 = m.body.push(
            Node::new(2, NodeType::Section)
                .with_label("sec:b")
                .with_parent(doc_id),
        );
        let sub = m.body.push(
            Node::new(3, NodeType::Subsection)
                .with_label("subsec:c")
                .with_parent(doc_id),
        );
        if let Some(d) = m.body.get_mut(doc_id) {
            d.add_child(sec1);
            d.add_child(sec2);
            d.add_child(sub);
        }

        let labels = collect_labels(&m);
        let a = labels.iter().find(|l| l.label == "sec:a").unwrap();
        assert_eq!(a.number, "0.1");
        let b = labels.iter().find(|l| l.label == "sec:b").unwrap();
        assert_eq!(b.number, "0.2");
        let c = labels.iter().find(|l| l.label == "subsec:c").unwrap();
        assert_eq!(c.number, "0.2.1");
    }

    #[test]
    fn test_resolve_references_basic() {
        let labels = vec![ResolvedLabel {
            label: "sec:intro".to_string(),
            kind: LabelKind::Section { level: 2 },
            number: "1.2".to_string(),
        }];
        let map = resolve_references(&labels);
        assert_eq!(map.get("sec:intro").unwrap(), "1.2");
    }

    #[test]
    fn test_resolve_text_references_ref() {
        let mut numbers = IndexMap::new();
        numbers.insert("sec:intro".to_string(), "1".to_string());
        let kinds = IndexMap::new();

        let text = r"See \ref{sec:intro} for details.";
        let resolved = resolve_text_references(text, &numbers, &kinds);
        assert_eq!(resolved, "See 1 for details.");
    }

    #[test]
    fn test_resolve_text_references_eqref() {
        let mut numbers = IndexMap::new();
        numbers.insert("eq:euler".to_string(), "3".to_string());
        let kinds = IndexMap::new();

        let text = r"By \eqref{eq:euler}, we know...";
        let resolved = resolve_text_references(text, &numbers, &kinds);
        assert_eq!(resolved, "By (3), we know...");
    }

    #[test]
    fn test_resolve_text_references_autoref_with_kind() {
        let mut numbers = IndexMap::new();
        numbers.insert("fig:diagram".to_string(), "2".to_string());
        let mut kinds = IndexMap::new();
        kinds.insert("fig:diagram".to_string(), LabelKind::Figure);

        let text = r"See \autoref{fig:diagram}.";
        let resolved = resolve_text_references(text, &numbers, &kinds);
        assert_eq!(resolved, "See Figure 2.");
    }

    #[test]
    fn test_resolve_text_references_autoref_prefix_inference() {
        let mut numbers = IndexMap::new();
        numbers.insert("tab:data".to_string(), "1".to_string());
        let kinds = IndexMap::new();

        let text = r"See \autoref{tab:data}.";
        let resolved = resolve_text_references(text, &numbers, &kinds);
        assert_eq!(resolved, "See Table 1.");
    }

    #[test]
    fn test_resolve_text_references_unknown_label() {
        let numbers = IndexMap::new();
        let kinds = IndexMap::new();

        let text = r"See \ref{missing} for details.";
        let resolved = resolve_text_references(text, &numbers, &kinds);
        assert_eq!(resolved, "See ?? for details.");
    }

    #[test]
    fn test_resolve_text_references_typst_style() {
        let mut numbers = IndexMap::new();
        numbers.insert("sec:results".to_string(), "2.3".to_string());
        let kinds = IndexMap::new();

        let text = "As shown in @sec:results, the results are clear.";
        let resolved = resolve_text_references(text, &numbers, &kinds);
        assert_eq!(resolved, "As shown in 2.3, the results are clear.");
    }

    #[test]
    fn test_resolve_text_references_mixed() {
        let mut numbers = IndexMap::new();
        numbers.insert("sec:intro".to_string(), "1".to_string());
        numbers.insert("eq:einstein".to_string(), "1".to_string());
        numbers.insert("fig:diagram".to_string(), "2".to_string());
        let mut kinds = IndexMap::new();
        kinds.insert("sec:intro".to_string(), LabelKind::Section { level: 2 });
        kinds.insert("eq:einstein".to_string(), LabelKind::Equation);
        kinds.insert("fig:diagram".to_string(), LabelKind::Figure);

        let text = r"See \ref{sec:intro}, \eqref{eq:einstein}, and \autoref{fig:diagram}.";
        let resolved = resolve_text_references(text, &numbers, &kinds);
        assert_eq!(resolved, "See 1, (1), and Figure 2.");
    }

    #[test]
    fn test_infer_autoref_prefix() {
        assert_eq!(infer_autoref_prefix("sec:intro"), "Section");
        assert_eq!(infer_autoref_prefix("ch:one"), "Section");
        assert_eq!(infer_autoref_prefix("eq:euler"), "Equation");
        assert_eq!(infer_autoref_prefix("fig:diagram"), "Figure");
        assert_eq!(infer_autoref_prefix("tab:data"), "Table");
        assert_eq!(infer_autoref_prefix("thm:pythagoras"), "Theorem");
        assert_eq!(infer_autoref_prefix("lem:bound"), "Theorem");
        assert_eq!(infer_autoref_prefix("custom:thing"), "");
    }

    #[test]
    fn test_collect_labels_from_annotations_fallback() {
        let mut m = SIRModuleV2::new();
        m.annotations
            .add_label("fig:missing_node".to_string(), 99, LabelCategory::Figure);

        let labels = collect_labels(&m);
        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0].label, "fig:missing_node");
        assert_eq!(labels[0].kind, LabelKind::Figure);
        assert!(labels[0].number.is_empty());
    }

    #[test]
    fn test_collect_labels_does_not_duplicate_annotations() {
        let mut m = SIRModuleV2::new();
        m.body.push(
            Node::new(
                0,
                NodeType::MathBlock {
                    math_type: ldir_ir::sir::v2::nodes::MathType::Equation,
                    numbered: true,
                },
            )
            .with_label("eq:dup"),
        );
        m.annotations
            .add_label("eq:dup".to_string(), 0, LabelCategory::Equation);

        let labels = collect_labels(&m);
        assert_eq!(labels.len(), 1);
    }

    #[test]
    fn test_collect_labels_chapter_section_numbering() {
        let mut m = SIRModuleV2::new();
        let doc_id = m.body.push(Node::new(0, NodeType::Document));

        let ch = m.body.push(
            Node::new(1, NodeType::Chapter)
                .with_label("ch:intro")
                .with_parent(doc_id),
        );
        if let Some(d) = m.body.get_mut(doc_id) {
            d.add_child(ch);
        }

        let sec = m.body.push(
            Node::new(2, NodeType::Section)
                .with_label("sec:background")
                .with_parent(doc_id),
        );
        if let Some(d) = m.body.get_mut(doc_id) {
            d.add_child(sec);
        }

        let labels = collect_labels(&m);
        let ch_label = labels.iter().find(|l| l.label == "ch:intro").unwrap();
        assert_eq!(ch_label.number, "1");
        assert_eq!(ch_label.kind, LabelKind::Section { level: 1 });

        let sec_label = labels.iter().find(|l| l.label == "sec:background").unwrap();
        assert_eq!(sec_label.number, "1.1");
        assert_eq!(sec_label.kind, LabelKind::Section { level: 2 });
    }

    #[test]
    fn test_collect_labels_multiple_equations() {
        let mut m = SIRModuleV2::new();
        m.body.push(
            Node::new(
                0,
                NodeType::MathBlock {
                    math_type: ldir_ir::sir::v2::nodes::MathType::Equation,
                    numbered: true,
                },
            )
            .with_label("eq:first"),
        );
        m.body.push(
            Node::new(
                1,
                NodeType::MathBlock {
                    math_type: ldir_ir::sir::v2::nodes::MathType::Equation,
                    numbered: true,
                },
            )
            .with_label("eq:second"),
        );
        m.body.push(Node::new(
            2,
            NodeType::MathBlock {
                math_type: ldir_ir::sir::v2::nodes::MathType::Equation,
                numbered: false,
            },
        ));

        let labels = collect_labels(&m);
        assert_eq!(labels.len(), 2);
        assert_eq!(labels[0].number, "1");
        assert_eq!(labels[1].number, "2");
    }
}
