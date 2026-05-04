use proptest::prelude::*;

use ldir_ir::sir::{SIRDocument, SIRInstruction, SIROpcode};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_sir_roundtrip(doc in arbitrary_sir_document()) {
        let bytes = doc.to_bytes();
        let parsed = SIRDocument::from_bytes(&bytes).unwrap();
        assert_eq!(parsed, doc);
    }
}

fn arbitrary_sir_document() -> impl Strategy<Value = SIRDocument> {
    proptest::collection::vec(arbitrary_sir_instruction(), 0..20).prop_map(|instructions| {
        let mut doc = SIRDocument::new();
        for instr in instructions {
            doc.push(instr);
        }
        doc
    })
}

fn arbitrary_sir_instruction() -> impl Strategy<Value = SIRInstruction> {
    let opcode = prop_oneof![
        Just(SIROpcode::PushBlock),
        Just(SIROpcode::SetContent),
        Just(SIROpcode::ApplyStyle),
        Just(SIROpcode::InsertMath),
        Just(SIROpcode::LinkData),
    ];
    let entity_id = any::<u32>();
    let parent_id = any::<u32>();
    let payload_offset = any::<u32>();

    (opcode, entity_id, parent_id, payload_offset).prop_map(
        |(opcode, entity_id, parent_id, payload_offset)| {
            SIRInstruction::new(opcode, entity_id, parent_id, payload_offset)
        },
    )
}
