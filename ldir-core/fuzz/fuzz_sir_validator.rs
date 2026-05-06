#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(doc) = ldir_core::parser::parse_sir(data) {
        let _ = ldir_core::validator::validate_sir(&doc);
    }
});
