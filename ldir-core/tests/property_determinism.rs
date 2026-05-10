use proptest::prelude::*;

use ldir_core::compiler::compile_sir;
use ldir_core::emitter::emit_gir;
use ldir_ir::sir::{ROOT_SENTINEL, SIRDocument, SIRInstruction, SIROpcode};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn test_compile_determinism_prop(doc in arbitrary_valid_sir()) {
        let g1 = compile_sir(&doc).unwrap();
        let g2 = compile_sir(&doc).unwrap();
        assert_eq!(g1, g2);

        let b1 = emit_gir(&g1);
        let b2 = emit_gir(&g2);
        assert_eq!(b1, b2);
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

            doc.push(SIRInstruction::new(
                SIROpcode::PushBlock,
                next_id,
                ROOT_SENTINEL,
                0,
            ));
            let root_id = next_id;
            next_id += 1;

            for children in &children_groups {
                let parent_id = root_id;
                for _ in children {
                    doc.push(SIRInstruction::new(
                        SIROpcode::SetContent,
                        next_id,
                        parent_id,
                        0,
                    ));
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
                    let _ = style_id; // style node created; parent_id unchanged for this group
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
