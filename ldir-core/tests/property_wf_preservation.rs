use proptest::prelude::*;

use ldir_core::compiler::compile_sir;
use ldir_core::validator::validate_sir;
use ldir_core::verifier::check_gir;
use ldir_ir::sir::{BlockType, ROOT_SENTINEL, SIRDocument, SIRInstruction, SIROpcode};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn test_wf_preservation(doc in arbitrary_valid_sir()) {
        assert!(validate_sir(&doc).is_ok(), "generated S-IR must be valid");
        let gir = compile_sir(&doc).unwrap();
        assert!(gir.is_well_formed(), "compiled G-IR must be well-formed (stack balanced)");
        assert!(check_gir(&gir).is_ok(), "compiled G-IR must pass full verifier");
    }
}

fn arbitrary_valid_sir() -> impl Strategy<Value = SIRDocument> {
    let max_children = 1usize..5usize;
    max_children.prop_flat_map(|max_ch| {
        proptest::collection::vec(
            proptest::collection::vec(valid_child_instruction(), 0..max_ch),
            1..5usize,
        )
        .prop_map(move |children_groups| {
            let mut doc = SIRDocument::new();
            let mut next_id: u32 = 0;

            doc.push_with_payload(
                SIRInstruction::new(SIROpcode::PushBlock, next_id, ROOT_SENTINEL, 0),
                &[BlockType::Document as u8],
            );
            let root_id = next_id;
            next_id += 1;

            for children in &children_groups {
                let mut parent_id = root_id;
                for _ in children {
                    let text = format!("text {}\x00", next_id);
                    doc.push_with_payload(
                        SIRInstruction::new(SIROpcode::SetContent, next_id, parent_id, 0),
                        text.as_bytes(),
                    );
                    next_id += 1;
                }
                if !children.is_empty() {
                    let style_id = next_id;
                    doc.push(SIRInstruction::new(
                        SIROpcode::ApplyStyle,
                        style_id,
                        parent_id,
                        0,
                    ));
                    parent_id = style_id;
                    next_id += 1;
                }
            }

            doc
        })
    })
}

fn valid_child_instruction() -> impl Strategy<Value = SIROpcode> {
    prop_oneof![
        Just(SIROpcode::SetContent),
        Just(SIROpcode::ApplyStyle),
        Just(SIROpcode::InsertMath),
        Just(SIROpcode::LinkData),
    ]
}
