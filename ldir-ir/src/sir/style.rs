//! S-IR style definitions.
//!
//! Style modifiers for inline text formatting (bold, italic, monospace, etc.)
//! are encoded as bitflags in the `ApplyStyle` instruction's payload.
//!
//! ## Encoding
//!
//! The `ApplyStyle` instruction's `payload_offset` field stores a packed u32:
//! - Bits 0-7: Style modifier flags (bold, italic, mono, etc.)
//! - Bits 8-15: Reserved for future use (font size, color index)
//!
//! ## Push/Pop Convention
//!
//! Styles are applied using `ApplyStyle` instructions with a direction flag:
//! - Bit 7 set → push style onto stack (enter span)
//! - Bit 7 clear → pop style from stack (exit span)

/// Style modifier bitflags for inline text formatting.
///
/// Encoded in the lower byte of the `ApplyStyle` payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct StyleModifier(pub u8);

impl StyleModifier {
    /// Push style onto the stack (enter styled span).
    /// When clear, this is a pop operation (exit styled span).
    pub const PUSH: u8 = 1 << 7;
    /// Bold text (**bold**).
    pub const BOLD: u8 = 1 << 0;
    /// Italic text (*italic*).
    pub const ITALIC: u8 = 1 << 1;
    /// Monospace text (`code`).
    pub const MONO: u8 = 1 << 2;
    /// Underlined text.
    pub const UNDERLINE: u8 = 1 << 3;
    /// Strikethrough text (~~strike~~).
    pub const STRIKE: u8 = 1 << 4;
    /// Small caps.
    pub const SMALL_CAPS: u8 = 1 << 5;

    /// No modifiers.
    pub const EMPTY: StyleModifier = StyleModifier(0);

    /// Bold style.
    pub const BOLD_STYLE: StyleModifier = StyleModifier(Self::BOLD);
    /// Italic style.
    pub const ITALIC_STYLE: StyleModifier = StyleModifier(Self::ITALIC);
    /// Monospace (inline code) style.
    pub const MONO_STYLE: StyleModifier = StyleModifier(Self::MONO);
    /// Bold + italic combined.
    pub const BOLD_ITALIC: StyleModifier = StyleModifier(Self::BOLD | Self::ITALIC);

    /// Check if a specific flag is set.
    pub const fn contains(&self, flag: u8) -> bool {
        self.0 & flag != 0
    }

    /// Encode a style push operation.
    pub fn push(modifiers: StyleModifier) -> u32 {
        (Self::PUSH | modifiers.0) as u32
    }

    /// Encode a style pop operation.
    pub fn pop() -> u32 {
        0u32
    }

    /// Decode style modifiers from a packed u32.
    pub fn from_packed(packed: u32) -> (StyleModifier, bool) {
        let byte = packed as u8;
        let is_push = byte & Self::PUSH != 0;
        let modifiers = StyleModifier(byte & !Self::PUSH);
        (modifiers, is_push)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_push_bold() {
        let packed = StyleModifier::push(StyleModifier::BOLD_STYLE);
        let (mods, is_push) = StyleModifier::from_packed(packed);
        assert!(is_push);
        assert!(mods.contains(StyleModifier::BOLD));
    }

    #[test]
    fn test_style_push_bold_italic() {
        let packed = StyleModifier::push(StyleModifier::BOLD_ITALIC);
        let (mods, is_push) = StyleModifier::from_packed(packed);
        assert!(is_push);
        assert!(mods.contains(StyleModifier::BOLD));
        assert!(mods.contains(StyleModifier::ITALIC));
    }

    #[test]
    fn test_style_pop() {
        let packed = StyleModifier::pop();
        let (_mods, is_push) = StyleModifier::from_packed(packed);
        assert!(!is_push);
    }

    #[test]
    fn test_style_mixed() {
        let packed = StyleModifier::push(StyleModifier(
            StyleModifier::MONO | StyleModifier::UNDERLINE,
        ));
        let (mods, is_push) = StyleModifier::from_packed(packed);
        assert!(is_push);
        assert!(mods.contains(StyleModifier::MONO));
        assert!(mods.contains(StyleModifier::UNDERLINE));
        assert!(!mods.contains(StyleModifier::BOLD));
    }

    #[test]
    fn test_empty_style() {
        assert!(!StyleModifier::EMPTY.contains(StyleModifier::BOLD));
    }
}
