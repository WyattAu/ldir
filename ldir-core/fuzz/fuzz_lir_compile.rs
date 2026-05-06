#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut module = ldir_ir::sir::v2::SIRModuleV2::new();
    let text_node = ldir_ir::sir::v2::Node::new(
        1,
        ldir_ir::sir::v2::NodeType::Text {
            content: String::from_utf8_lossy(data).into_owned(),
        },
    )
    .with_parent(0);
    module.body.push(text_node);

    let ctx = ldir_core::compiler::context::CompileContext::default();
    let _ = ldir_core::compile_sir_to_lir(&module, &ctx);
});
