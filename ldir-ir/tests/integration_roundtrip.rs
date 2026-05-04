use ldir_ir::sir::v2::*;

#[test]
fn test_full_roundtrip() {
    let mut module = SIRModuleV2::from_source("test", "roundtrip.ldir");
    module.metadata.title = Some("Roundtrip Test".to_string());
    module.metadata.author = Some("Test Author".to_string());
    module.metadata.document_class = Some("article".to_string());

    module
        .body
        .push(nodes::Node::new(1, nodes::NodeType::Section).with_label("sec:intro"));
    module
        .body
        .push(nodes::Node::new(2, nodes::NodeType::Paragraph).with_parent(1));
    module.body.push(
        nodes::Node::new(
            3,
            nodes::NodeType::Text {
                content: "Hello, world!".to_string(),
            },
        )
        .with_parent(2),
    );
    module
        .body
        .push(nodes::Node::new(4, nodes::NodeType::Bold).with_parent(2));
    module.body.push(
        nodes::Node::new(
            5,
            nodes::NodeType::Text {
                content: "bold text".to_string(),
            },
        )
        .with_parent(4),
    );

    let binary = serialize::serialize_module(&module);
    assert!(binary.len() > 17);
    assert_eq!(&binary[0..4], b"LDIR");

    let restored = serialize::deserialize_module(&binary).unwrap();
    assert_eq!(restored.metadata.title.as_deref(), Some("Roundtrip Test"));
    assert_eq!(restored.metadata.author.as_deref(), Some("Test Author"));
    assert_eq!(restored.body.len(), 5);
    assert!(restored.body.find_by_label("sec:intro").is_some());

    let text = text::module_to_text(&module);
    assert!(text.contains(";; ldir-ir v2.0.0"));
    assert!(text.contains("title = \"Roundtrip Test\""));
    assert!(text.contains("@section [id=1, label=\"sec:intro\"]"));
    assert!(text.contains("@paragraph [id=2, parent=1]"));
    assert!(text.contains("@text [id=3, parent=2] { \"Hello, world!\" }"));
    assert!(text.contains("@bold [id=4, parent=2]"));
    assert!(text.contains("@text [id=5, parent=4] { \"bold text\" }"));
}

#[test]
fn test_binary_roundtrip_preserves_all_sections() {
    let mut module = SIRModuleV2::new();
    module.metadata.title = Some("Complex Doc".to_string());
    module.resources.fonts.push(resources::FontDecl {
        name: "body".into(),
        family: "Inter".into(),
        weight: resources::FontWeight::Regular,
        style: resources::FontStyle::Normal,
        source: resources::FontSource::System,
        features: vec!["liga".into()],
    });
    module.resources.counters.push(resources::CounterDecl {
        name: "section".into(),
        format: resources::CounterFormat::Arabic,
        reset_scope: resources::CounterReset::PerChapter,
    });
    module.styles.styles.push(styles::StyleDecl {
        name: "body-text".into(),
        parent: None,
        properties: styles::StyleProperties::default(),
    });
    module
        .body
        .push(nodes::Node::new(1, nodes::NodeType::Document));
    module
        .body
        .push(nodes::Node::new(2, nodes::NodeType::Chapter).with_parent(1));
    module
        .body
        .push(nodes::Node::new(3, nodes::NodeType::Section).with_parent(2));
    module.body.push(
        nodes::Node::new(
            4,
            nodes::NodeType::MathBlock {
                math_type: nodes::MathType::Equation,
                numbered: true,
            },
        )
        .with_parent(3),
    );
    module.body.push(
        nodes::Node::new(
            5,
            nodes::NodeType::Table {
                col_specs: vec![],
                num_cols: 0,
                caption: None,
                column_widths: vec![],
                header_row: false,
            },
        )
        .with_parent(3),
    );

    let bytes = serialize::serialize_module(&module);
    let restored = serialize::deserialize_module(&bytes).unwrap();

    assert_eq!(restored.resources.fonts.len(), 1);
    assert_eq!(restored.resources.counters.len(), 1);
    assert_eq!(restored.styles.styles.len(), 1);
    assert_eq!(restored.body.len(), 5);
    assert_eq!(restored.metadata.title.as_deref(), Some("Complex Doc"));
}

#[test]
fn test_text_output_contains_expected_content() {
    let mut module = SIRModuleV2::new();
    module.metadata.title = Some("Content Test".to_string());
    module
        .body
        .push(nodes::Node::new(1, nodes::NodeType::Chapter));
    module
        .body
        .push(nodes::Node::new(2, nodes::NodeType::Section).with_parent(1));
    module
        .body
        .push(nodes::Node::new(3, nodes::NodeType::Paragraph).with_parent(2));
    module.body.push(
        nodes::Node::new(
            4,
            nodes::NodeType::Text {
                content: "Section content here".to_string(),
            },
        )
        .with_parent(3),
    );
    module.body.push(
        nodes::Node::new(
            5,
            nodes::NodeType::Link {
                url: "https://example.com".into(),
                title: None,
            },
        )
        .with_parent(3),
    );
    module.body.push(
        nodes::Node::new(
            6,
            nodes::NodeType::CodeBlock {
                language: Some("rust".into()),
                content: String::new(),
            },
        )
        .with_parent(2),
    );
    module.body.push(nodes::Node::new(
        7,
        nodes::NodeType::TableOfContents { max_depth: 4 },
    ));
    module
        .body
        .push(nodes::Node::new(8, nodes::NodeType::ThematicBreak));
    module.body.push(nodes::Node::new(
        9,
        nodes::NodeType::Footnote {
            content: "A footnote".into(),
        },
    ));

    let text = text::module_to_text(&module);

    assert!(text.contains(";; ldir-ir v2.0.0"));
    assert!(text.contains("@meta {"));
    assert!(text.contains("title = \"Content Test\""));
    assert!(text.contains("@chapter [id=1]"));
    assert!(text.contains("@section [id=2, parent=1]"));
    assert!(text.contains("@paragraph [id=3, parent=2]"));
    assert!(text.contains("\"Section content here\""));
    assert!(text.contains("https://example.com"));
    assert!(text.contains("lang=\"rust\""));
    assert!(text.contains("depth=4"));
    assert!(text.contains("@hr [id=8]"));
    assert!(text.contains("\"A footnote\""));
}

#[test]
fn test_json_serialization() {
    let mut module = SIRModuleV2::new();
    module.metadata.title = Some("JSON Test".to_string());
    module
        .body
        .push(nodes::Node::new(1, nodes::NodeType::Section));

    let json = serde_json::to_string(&module).unwrap();
    assert!(json.contains("JSON Test"));
    assert!(json.contains("Section"));

    let restored: SIRModuleV2 = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.metadata.title.as_deref(), Some("JSON Test"));
    assert_eq!(restored.body.len(), 1);
}

#[test]
fn test_serialize_then_disassemble_text() {
    let mut module = SIRModuleV2::from_source("markdown", "example.md");
    module.metadata.title = Some("Disassemble Test".to_string());
    module
        .body
        .push(nodes::Node::new(1, nodes::NodeType::Part).with_label("part:one"));
    module
        .body
        .push(nodes::Node::new(2, nodes::NodeType::Chapter).with_parent(1));
    module.body.push(
        nodes::Node::new(3, nodes::NodeType::Section)
            .with_parent(2)
            .with_label("sec:methods"),
    );
    module
        .body
        .push(nodes::Node::new(4, nodes::NodeType::Paragraph).with_parent(3));
    module.body.push(
        nodes::Node::new(
            5,
            nodes::NodeType::Text {
                content: "Methods and analysis".to_string(),
            },
        )
        .with_parent(4),
    );

    let binary = serialize::serialize_module(&module);
    let disassembled = serialize::deserialize_module(&binary).unwrap();
    let text = text::module_to_text(&disassembled);

    assert!(text.contains(";; source: markdown"));
    assert!(text.contains("title = \"Disassemble Test\""));
    assert!(text.contains("@part [id=1, label=\"part:one\"]"));
    assert!(text.contains("@chapter [id=2, parent=1]"));
    assert!(text.contains("@section [id=3, parent=2, label=\"sec:methods\"]"));
    assert!(text.contains("@paragraph [id=4, parent=3]"));
    assert!(text.contains("\"Methods and analysis\""));
}
