use super::MacroRegistry;

pub fn register(registry: &mut MacroRegistry) {
    let symbols: &[(&str, &str)] = &[
        ("implies", "\u{21D2}"),
        ("iff", "\u{21D4}"),
        ("lim", "lim"),
        ("colon", ":"),
    ];
    for &(cmd, sym) in symbols {
        registry.math_symbols.insert(cmd, sym);
    }
}
