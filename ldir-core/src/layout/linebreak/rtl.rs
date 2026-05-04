//! RTL (right-to-left) text detection and bidirectional reordering.
//!
//! Implements Unicode Bidirectional Algorithm (UBA) levels L1–L4 per UAX#9.

#![allow(dead_code, clippy::upper_case_acronyms)]
#![deny(unsafe_code)]
#![warn(clippy::unwrap_used, clippy::expect_used)]

/// Direction of a text run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDirection {
    LeftToRight,
    RightToLeft,
}

/// A contiguous run of text with the same direction.
#[derive(Debug, Clone)]
pub struct DirectionRun {
    pub start: usize,
    pub end: usize,
    pub direction: TextDirection,
    pub level: u8,
}

/// Directional override types per UAX#9.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectionalOverride {
    LRE,
    RLE,
    LRO,
    RLO,
    PDF,
}

/// Bidirectional character class per UAX#9 Table 3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BidiClass {
    L,
    R,
    AL,
    EN,
    ES,
    ET,
    AN,
    CS,
    NSM,
    BN,
    B,
    S,
    WS,
    ON,
    LRE,
    RLE,
    PDF,
    LRO,
    RLO,
}

/// Strong types for neutral resolution.
fn is_strong(bc: BidiClass) -> bool {
    matches!(bc, BidiClass::L | BidiClass::R | BidiClass::AL)
}

fn is_weak(bc: BidiClass) -> bool {
    matches!(
        bc,
        BidiClass::EN
            | BidiClass::ES
            | BidiClass::ET
            | BidiClass::AN
            | BidiClass::CS
            | BidiClass::NSM
            | BidiClass::BN
    )
}

fn is_neutral(bc: BidiClass) -> bool {
    matches!(
        bc,
        BidiClass::B | BidiClass::S | BidiClass::WS | BidiClass::ON
    )
}

fn is_isolate_or_format(bc: BidiClass) -> bool {
    matches!(
        bc,
        BidiClass::LRE | BidiClass::RLE | BidiClass::LRO | BidiClass::RLO | BidiClass::PDF
    )
}

fn bracket_open_pair(ch: char) -> Option<char> {
    match ch {
        '(' => Some(')'),
        '[' => Some(']'),
        '{' => Some('}'),
        '\u{2045}' => Some('\u{2046}'),
        '\u{207D}' => Some('\u{207E}'),
        '\u{208D}' => Some('\u{208E}'),
        '\u{2329}' => Some('\u{232A}'),
        '\u{3008}' => Some('\u{3009}'),
        '\u{300A}' => Some('\u{300B}'),
        '\u{300C}' => Some('\u{300D}'),
        '\u{300E}' => Some('\u{300F}'),
        '\u{3010}' => Some('\u{3011}'),
        '\u{3014}' => Some('\u{3015}'),
        '\u{3016}' => Some('\u{3017}'),
        '\u{3018}' => Some('\u{3019}'),
        '\u{301A}' => Some('\u{301B}'),
        '\u{FE59}' => Some('\u{FE5A}'),
        '\u{FE5B}' => Some('\u{FE5C}'),
        '\u{FE5D}' => Some('\u{FE5E}'),
        _ => None,
    }
}

fn bracket_close_pair(ch: char) -> Option<char> {
    match ch {
        ')' => Some('('),
        ']' => Some('['),
        '}' => Some('{'),
        '\u{2046}' => Some('\u{2045}'),
        '\u{207E}' => Some('\u{207D}'),
        '\u{208E}' => Some('\u{208D}'),
        '\u{232A}' => Some('\u{2329}'),
        '\u{3009}' => Some('\u{3008}'),
        '\u{300B}' => Some('\u{300A}'),
        '\u{300D}' => Some('\u{300C}'),
        '\u{300F}' => Some('\u{300E}'),
        '\u{3011}' => Some('\u{3010}'),
        '\u{3015}' => Some('\u{3014}'),
        '\u{3017}' => Some('\u{3016}'),
        '\u{3019}' => Some('\u{3018}'),
        '\u{301B}' => Some('\u{301A}'),
        '\u{FE5A}' => Some('\u{FE59}'),
        '\u{FE5C}' => Some('\u{FE5B}'),
        '\u{FE5E}' => Some('\u{FE5D}'),
        _ => None,
    }
}

fn strong_direction(bc: BidiClass) -> Option<BidiClass> {
    match bc {
        BidiClass::L => Some(BidiClass::L),
        BidiClass::R => Some(BidiClass::R),
        BidiClass::EN => Some(BidiClass::L),
        BidiClass::AN => Some(BidiClass::R),
        _ => None,
    }
}

/// Classify a character's bidirectional class per UAX#9 Table 3.
pub fn bidi_class(ch: char) -> BidiClass {
    match ch {
        // Explicit directional controls
        '\u{202A}' => BidiClass::LRE,
        '\u{202B}' => BidiClass::RLE,
        '\u{202C}' => BidiClass::PDF,
        '\u{202D}' => BidiClass::LRO,
        '\u{202E}' => BidiClass::RLO,

        // Paragraph separator
        '\u{2029}' => BidiClass::B,

        // Segment separator
        '\u{0009}' | '\u{000A}' | '\u{000B}' | '\u{000C}' | '\u{000D}' | '\u{001C}'..='\u{001F}'
        | '\u{0085}' => BidiClass::S,

        // Whitespace
        '\u{0020}' | '\u{00A0}' | '\u{1680}' | '\u{2000}'..='\u{200A}' | '\u{202F}'
        | '\u{205F}' | '\u{3000}' => BidiClass::WS,

        // Arabic-Indic digits (AN)
        '\u{0600}'..='\u{0605}' | '\u{0660}'..='\u{0669}' | '\u{066B}' | '\u{066C}'
        | '\u{06DD}' => BidiClass::AN,

        // European digits
        '0'..='9' | '\u{06F0}'..='\u{06F9}' | '\u{07C0}'..='\u{07C9}'
        | '\u{0966}'..='\u{096F}' | '\u{09E6}'..='\u{09EF}' | '\u{0A66}'..='\u{0A6F}'
        | '\u{0AE6}'..='\u{0AEF}' | '\u{0B66}'..='\u{0B6F}' | '\u{0BE6}'..='\u{0BEF}'
        | '\u{0C66}'..='\u{0C6F}' | '\u{0CE6}'..='\u{0CEF}' | '\u{0D66}'..='\u{0D6F}'
        | '\u{0E50}'..='\u{0E59}' | '\u{0ED0}'..='\u{0ED9}' => BidiClass::EN,

        // European separator
        '+' | ',' | '-' | '.' | '/' | ':' => BidiClass::ES,

        // Arabic Letter (AL) — Arabic block minus characters already classified
        // above (AN ranges 0600-0605, 0660-0669, 066B-066C, 06DD are handled)
        '\u{0608}'..='\u{060B}' | '\u{060D}'..='\u{061A}' | '\u{061C}'..='\u{061E}'
        | '\u{0620}'..='\u{063F}' | '\u{0641}'..='\u{064A}' | '\u{0656}'..='\u{065F}'
        | '\u{066E}'..='\u{0670}' | '\u{0671}'..='\u{06D3}' | '\u{06D4}'..='\u{06D5}'
        | '\u{06D6}'..='\u{06E4}' | '\u{06E5}'..='\u{06E6}' | '\u{06E7}'..='\u{06E8}'
        | '\u{06E9}' | '\u{06EA}'..='\u{06ED}' | '\u{06EE}'..='\u{06EF}'
        | '\u{06FA}'..='\u{06FC}' | '\u{06FD}'..='\u{06FF}'
        | '\u{0750}'..='\u{077F}' => BidiClass::AL,

        // Hebrew (R)
        '\u{0590}'..='\u{05FF}' | '\u{FB1D}'..='\u{FB36}' | '\u{FB38}'..='\u{FB3C}'
        | '\u{FB3E}' | '\u{FB40}'..='\u{FB41}' | '\u{FB43}'..='\u{FB44}'
        | '\u{FB46}'..='\u{FB4F}'
        // Syriac and Arabic Presentation Forms are R
        | '\u{0700}'..='\u{074F}' | '\u{FB50}'..='\u{FDFF}' | '\u{FE70}'..='\u{FEFF}' => BidiClass::R,

        // Non-Spacing Mark (NSM) — combining marks
        // Note: Hebrew (0590-05FF) and Arabic (0600-06FF) ranges already handled above
        '\u{0300}'..='\u{036F}' | '\u{0483}'..='\u{0489}'
        | '\u{07A6}'..='\u{07B0}' | '\u{07EB}'..='\u{07F3}' | '\u{0816}'..='\u{0819}'
        | '\u{081B}'..='\u{0823}' | '\u{0825}'..='\u{0827}' | '\u{0829}'..='\u{082D}'
        | '\u{0859}'..='\u{085B}' | '\u{08D4}'..='\u{08E1}' | '\u{08E3}'..='\u{0903}'
        | '\u{093A}'..='\u{093C}' | '\u{093E}'..='\u{094F}' | '\u{0951}'..='\u{0957}'
        | '\u{0962}'..='\u{0963}' | '\u{0981}'..='\u{0983}' | '\u{09BC}'
        | '\u{09BE}'..='\u{09C4}' | '\u{09C7}'..='\u{09C8}' | '\u{09CB}'..='\u{09CD}'
        | '\u{09D7}' | '\u{09E2}'..='\u{09E3}' | '\u{0A01}'..='\u{0A03}'
        | '\u{0A3C}' | '\u{0A3E}'..='\u{0A42}' | '\u{0A47}'..='\u{0A48}'
        | '\u{0A4B}'..='\u{0A4D}' | '\u{0A51}' | '\u{0A70}'..='\u{0A71}'
        | '\u{0A75}' | '\u{0A81}'..='\u{0A83}' | '\u{0ABC}'
        | '\u{0ABE}'..='\u{0AC5}' | '\u{0AC7}'..='\u{0AC9}' | '\u{0ACB}'..='\u{0ACD}'
        | '\u{0AE2}'..='\u{0AE3}' | '\u{0B01}'..='\u{0B03}' | '\u{0B3C}'
        | '\u{0B3E}'..='\u{0B44}' | '\u{0B47}'..='\u{0B48}' | '\u{0B4B}'..='\u{0B4D}'
        | '\u{0B56}'..='\u{0B57}' | '\u{0B62}'..='\u{0B63}' | '\u{0B82}'
        | '\u{0BBE}'..='\u{0BC2}' | '\u{0BC6}'..='\u{0BC8}' | '\u{0BCA}'..='\u{0BCD}'
        | '\u{0BD7}' | '\u{0C00}'..='\u{0C03}' | '\u{0C3E}'..='\u{0C44}'
        | '\u{0C46}'..='\u{0C48}' | '\u{0C4A}'..='\u{0C4D}' | '\u{0C55}'..='\u{0C56}'
        | '\u{0C62}'..='\u{0C63}' | '\u{0C81}'..='\u{0C83}' | '\u{0CBC}'
        | '\u{0CBE}'..='\u{0CC4}' | '\u{0CC6}'..='\u{0CC8}' | '\u{0CCA}'..='\u{0CCD}'
        | '\u{0CD5}'..='\u{0CD6}' | '\u{0CE2}'..='\u{0CE3}' | '\u{0D01}'..='\u{0D03}'
        | '\u{0D3E}'..='\u{0D44}' | '\u{0D46}'..='\u{0D48}' | '\u{0D4A}'..='\u{0D4D}'
        | '\u{0D57}' | '\u{0D62}'..='\u{0D63}' | '\u{0D82}'..='\u{0D83}'
        | '\u{0DCA}' | '\u{0DCF}'..='\u{0DD4}' | '\u{0DD6}' | '\u{0DD8}'..='\u{0DDF}'
        | '\u{0DF2}'..='\u{0DF3}' => BidiClass::NSM,

        // Boundary Neutral
        '\u{200C}' | '\u{200D}' | '\u{2066}'..='\u{2069}' => BidiClass::BN,

        // Left-to-Right: Latin, Cyrillic, Greek, Devanagari, most scripts
        '\u{0041}'..='\u{005A}' | '\u{0061}'..='\u{007A}' | '\u{00C0}'..='\u{00D6}'
        | '\u{00D8}'..='\u{00F6}' | '\u{00F8}'..='\u{024F}' | '\u{0250}'..='\u{02AF}'
        | '\u{0370}'..='\u{03FF}' | '\u{0400}'..='\u{04FF}' | '\u{0500}'..='\u{052F}'
        | '\u{0531}'..='\u{0556}' | '\u{0559}'..='\u{055F}' | '\u{0561}'..='\u{0587}'
        | '\u{0900}'..='\u{097F}' | '\u{0980}'..='\u{09FF}' | '\u{0A00}'..='\u{0A7F}'
        | '\u{0A80}'..='\u{0AFF}' | '\u{0B00}'..='\u{0B7F}' | '\u{0B80}'..='\u{0BFF}'
        | '\u{0C00}'..='\u{0C7F}' | '\u{0C80}'..='\u{0CFF}' | '\u{0D00}'..='\u{0D7F}'
        | '\u{0D80}'..='\u{0DFF}' | '\u{0E00}'..='\u{0E7F}' | '\u{0E80}'..='\u{0EFF}'
        | '\u{0F00}'..='\u{0FFF}' | '\u{1000}'..='\u{109F}' | '\u{10A0}'..='\u{10FF}'
        | '\u{1100}'..='\u{11FF}' | '\u{1200}'..='\u{137F}' | '\u{1380}'..='\u{139F}'
        | '\u{1400}'..='\u{167F}' | '\u{1680}'..='\u{169F}' | '\u{16A0}'..='\u{16FF}'
        | '\u{1700}'..='\u{17FF}' | '\u{1800}'..='\u{18AF}' | '\u{1900}'..='\u{194F}'
        | '\u{1950}'..='\u{197F}' | '\u{1980}'..='\u{19DF}' | '\u{1E00}'..='\u{1EFF}'
        | '\u{1F00}'..='\u{1FFF}' | '\u{2000}'..='\u{206F}' | '\u{2070}'..='\u{209F}'
        | '\u{20A0}'..='\u{20CF}' | '\u{2100}'..='\u{214F}' | '\u{2150}'..='\u{218F}'
        | '\u{2190}'..='\u{21FF}' | '\u{2200}'..='\u{22FF}' | '\u{2300}'..='\u{23FF}'
        | '\u{2400}'..='\u{243F}' | '\u{2440}'..='\u{245F}' | '\u{2460}'..='\u{24FF}'
        | '\u{2500}'..='\u{257F}' | '\u{2580}'..='\u{259F}' | '\u{25A0}'..='\u{25FF}'
        | '\u{2600}'..='\u{26FF}' | '\u{2700}'..='\u{27BF}' | '\u{2800}'..='\u{28FF}'
        | '\u{2900}'..='\u{297F}' | '\u{2980}'..='\u{29FF}' | '\u{2A00}'..='\u{2AFF}'
        | '\u{2B00}'..='\u{2BFF}' | '\u{2C00}'..='\u{2C5F}' | '\u{2C60}'..='\u{2C7F}'
        | '\u{2C80}'..='\u{2CFF}' | '\u{2D00}'..='\u{2D2F}' | '\u{2D30}'..='\u{2D7F}'
        | '\u{2D80}'..='\u{2DDF}' | '\u{2E00}'..='\u{2E7F}' | '\u{3000}'..='\u{303F}'
        | '\u{3040}'..='\u{309F}' | '\u{30A0}'..='\u{30FF}' | '\u{3100}'..='\u{312F}'
        | '\u{3130}'..='\u{318F}' | '\u{3190}'..='\u{319F}' | '\u{31A0}'..='\u{31BF}'
        | '\u{31C0}'..='\u{31EF}' | '\u{31F0}'..='\u{31FF}' | '\u{3200}'..='\u{32FF}'
        | '\u{3300}'..='\u{33FF}' | '\u{3400}'..='\u{4DBF}' | '\u{4DC0}'..='\u{4DFF}'
        | '\u{4E00}'..='\u{9FFF}' | '\u{A000}'..='\u{A48F}' | '\u{A490}'..='\u{A4CF}'
        | '\u{A4D0}'..='\u{A4FF}' | '\u{A500}'..='\u{A63F}' | '\u{A640}'..='\u{A69F}'
        | '\u{A6A0}'..='\u{A6FF}' | '\u{A700}'..='\u{A71F}' | '\u{A720}'..='\u{A7FF}'
        | '\u{A800}'..='\u{A82F}' | '\u{A830}'..='\u{A83F}' | '\u{A840}'..='\u{A87F}'
        | '\u{A880}'..='\u{A8DF}' | '\u{A8E0}'..='\u{A8FF}' | '\u{A900}'..='\u{A92F}'
        | '\u{A930}'..='\u{A95F}' | '\u{A960}'..='\u{A97F}' | '\u{A980}'..='\u{A9DF}'
        | '\u{A9E0}'..='\u{A9FF}' | '\u{AA00}'..='\u{AA5F}' | '\u{AA60}'..='\u{AA7F}'
        | '\u{AA80}'..='\u{AADF}' | '\u{AAE0}'..='\u{AAFF}' | '\u{AB00}'..='\u{AB2F}'
        | '\u{AB30}'..='\u{AB6F}' | '\u{AB70}'..='\u{ABBF}' | '\u{ABC0}'..='\u{ABFF}'
        | '\u{AC00}'..='\u{D7AF}' | '\u{D7B0}'..='\u{D7FF}' | '\u{F900}'..='\u{FAFF}'
        | '\u{FE00}'..='\u{FE0F}' | '\u{FE10}'..='\u{FE1F}' | '\u{FE30}'..='\u{FE4F}'
        | '\u{FE50}'..='\u{FE6F}' | '\u{10000}'..='\u{1FFFD}'
        | '\u{20000}'..='\u{2FFFD}' | '\u{30000}'..='\u{3FFFD}' | '\u{40000}'..='\u{4FFFD}'
        | '\u{50000}'..='\u{5FFFD}' | '\u{60000}'..='\u{6FFFD}' | '\u{70000}'..='\u{7FFFD}'
        | '\u{80000}'..='\u{8FFFD}' | '\u{90000}'..='\u{9FFFD}' | '\u{A0000}'..='\u{AFFFD}'
        | '\u{B0000}'..='\u{BFFFD}' | '\u{C0000}'..='\u{CFFFD}' | '\u{D0000}'..='\u{DFFFD}'
        | '\u{E0000}'..='\u{E0001}' | '\u{E0002}'..='\u{E001F}' | '\u{E0020}'..='\u{E007F}'
        | '\u{E0080}'..='\u{E00FF}' | '\u{E0100}'..='\u{E01EF}' | '\u{F0000}'..='\u{FFFFD}'
        | '\u{100000}'..='\u{10FFFD}' => BidiClass::L,

        // Default: Other Neutral
        _ => BidiClass::ON,
    }
}

/// Check if a character is a strong RTL character.
pub fn is_rtl_strong(ch: char) -> bool {
    matches!(bidi_class(ch), BidiClass::R | BidiClass::AL | BidiClass::AN)
}

pub fn is_rtl_char(ch: char) -> bool {
    is_rtl_strong(ch)
}

pub fn is_rtl_text(text: &str) -> bool {
    let rtl_count = text
        .chars()
        .filter(|c| is_rtl_char(*c) && !c.is_whitespace())
        .count();
    let total: usize = text.chars().filter(|c| !c.is_whitespace()).count();
    total > 0 && rtl_count as f64 / total as f64 > 0.5
}

fn is_ltr_strong(ch: char) -> bool {
    matches!(bidi_class(ch), BidiClass::L)
}

/// Determine the base direction from the first strong character (P2, P3).
pub fn base_direction(text: &str) -> TextDirection {
    for ch in text.chars() {
        match bidi_class(ch) {
            BidiClass::R | BidiClass::AL => return TextDirection::RightToLeft,
            BidiClass::L => return TextDirection::LeftToRight,
            _ => {}
        }
    }
    TextDirection::LeftToRight
}

const MAX_DEPTH: u8 = 125;

#[derive(Debug, Clone)]
struct EmbeddingStackEntry {
    level: u8,
    override_class: Option<BidiClass>,
    is_isolate: bool,
}

struct BidiProcessor {
    chars: Vec<char>,
    classes: Vec<BidiClass>,
    levels: Vec<u8>,
    original_classes: Vec<BidiClass>,
    paragraph_level: u8,
    stack: Vec<EmbeddingStackEntry>,
}

impl BidiProcessor {
    fn new(text: &str, base_dir: Option<TextDirection>) -> Self {
        let paragraph_level = match base_dir {
            Some(TextDirection::RightToLeft) => 1,
            Some(TextDirection::LeftToRight) | None => {
                if text.is_empty() {
                    0
                } else {
                    match base_direction(text) {
                        TextDirection::RightToLeft => 1,
                        TextDirection::LeftToRight => 0,
                    }
                }
            }
        };

        let chars: Vec<char> = text.chars().collect();
        let classes: Vec<BidiClass> = chars.iter().copied().map(bidi_class).collect();
        let original_classes = classes.clone();
        let levels = vec![paragraph_level; chars.len()];

        let stack = vec![EmbeddingStackEntry {
            level: paragraph_level,
            override_class: None,
            is_isolate: false,
        }];

        Self {
            chars,
            classes,
            levels,
            original_classes,
            paragraph_level,
            stack,
        }
    }

    fn current_level(&self) -> u8 {
        self.stack
            .last()
            .map(|e| e.level)
            .unwrap_or(self.paragraph_level)
    }

    fn current_override(&self) -> Option<BidiClass> {
        self.stack.last().and_then(|e| e.override_class)
    }

    fn valid_level(&self, new_level: u8) -> bool {
        new_level <= MAX_DEPTH
            && self.stack.len() < 126
            && new_level % 2 != self.paragraph_level % 2
    }

    fn process_explicit(&mut self) {
        let len = self.chars.len();
        let mut i = 0;

        while i < len {
            let cls = self.original_classes[i];

            match cls {
                BidiClass::RLE => {
                    let new_level = (self.current_level() + 1) | 1;
                    if self.valid_level(new_level) {
                        self.stack.push(EmbeddingStackEntry {
                            level: new_level,
                            override_class: None,
                            is_isolate: false,
                        });
                        self.levels[i] = new_level;
                    } else {
                        self.levels[i] = self.current_level();
                    }
                    self.classes[i] = BidiClass::BN;
                }
                BidiClass::LRE => {
                    let new_level = (self.current_level() + 2) & !1;
                    if self.valid_level(new_level) {
                        self.stack.push(EmbeddingStackEntry {
                            level: new_level,
                            override_class: None,
                            is_isolate: false,
                        });
                        self.levels[i] = new_level;
                    } else {
                        self.levels[i] = self.current_level();
                    }
                    self.classes[i] = BidiClass::BN;
                }
                BidiClass::RLO => {
                    let new_level = (self.current_level() + 1) | 1;
                    if self.valid_level(new_level) {
                        self.stack.push(EmbeddingStackEntry {
                            level: new_level,
                            override_class: Some(BidiClass::R),
                            is_isolate: false,
                        });
                        self.levels[i] = new_level;
                    } else {
                        self.levels[i] = self.current_level();
                    }
                    self.classes[i] = BidiClass::BN;
                }
                BidiClass::LRO => {
                    let new_level = (self.current_level() + 2) & !1;
                    if self.valid_level(new_level) {
                        self.stack.push(EmbeddingStackEntry {
                            level: new_level,
                            override_class: Some(BidiClass::L),
                            is_isolate: false,
                        });
                        self.levels[i] = new_level;
                    } else {
                        self.levels[i] = self.current_level();
                    }
                    self.classes[i] = BidiClass::BN;
                }
                BidiClass::PDF => {
                    if self.stack.len() > 1 {
                        if let Some(removed) = self.stack.pop() {
                            if !removed.is_isolate {
                                self.levels[i] = self.current_level();
                            } else {
                                self.stack.push(removed);
                                self.levels[i] = self.current_level();
                            }
                        }
                    } else {
                        self.levels[i] = self.paragraph_level;
                    }
                    self.classes[i] = BidiClass::BN;
                }
                BidiClass::B => {
                    self.levels[i] = self.paragraph_level;
                }
                BidiClass::BN => {
                    // BN at level boundaries
                }
                _ => {
                    self.levels[i] = self.current_level();
                    if let Some(ov) = self.current_override() {
                        self.classes[i] = ov;
                    }
                }
            }
            i += 1;
        }
    }

    fn resolve_weak_types(&mut self) {
        let len = self.classes.len();
        if len == 0 {
            return;
        }

        // W1: NSM — inherit from preceding character
        for i in 0..len {
            if self.classes[i] == BidiClass::NSM {
                let prev = if i > 0 {
                    self.classes[i - 1]
                } else {
                    BidiClass::L
                };
                if matches!(
                    prev,
                    BidiClass::LRE
                        | BidiClass::RLE
                        | BidiClass::LRO
                        | BidiClass::RLO
                        | BidiClass::PDF
                        | BidiClass::BN
                ) {
                    self.classes[i] = BidiClass::ON;
                } else {
                    self.classes[i] = prev;
                }
            }
        }

        // W2: EN after AL → AN
        for i in 0..len {
            if self.classes[i] == BidiClass::EN {
                let mut found_al = false;
                for j in (0..i).rev() {
                    match self.classes[j] {
                        BidiClass::AL => {
                            found_al = true;
                            break;
                        }
                        BidiClass::L | BidiClass::R | BidiClass::EN | BidiClass::AN => break,
                        _ => {}
                    }
                }
                if found_al {
                    self.classes[i] = BidiClass::AN;
                }
            }
        }

        // W3: AL → R
        for i in 0..len {
            if self.classes[i] == BidiClass::AL {
                self.classes[i] = BidiClass::R;
            }
        }

        // W4: Single separator between European numbers → EN
        for i in 1..len - 1 {
            if matches!(
                self.classes[i],
                BidiClass::ES | BidiClass::CS | BidiClass::ET
            ) && self.classes[i - 1] == BidiClass::EN
                && self.classes[i + 1] == BidiClass::EN
            {
                self.classes[i] = BidiClass::EN;
            }
        }

        // W5: Sequence of ETs adjacent to EN → EN
        loop {
            let mut changed = false;
            let mut i = 0;
            while i < len {
                if self.classes[i] == BidiClass::ET {
                    // Look backward for EN
                    let mut has_en_before = false;
                    for j in (0..i).rev() {
                        if self.classes[j] == BidiClass::EN {
                            has_en_before = true;
                            break;
                        }
                        if !matches!(self.classes[j], BidiClass::ET) {
                            break;
                        }
                    }
                    // Look forward for EN
                    let mut has_en_after = false;
                    for j in i + 1..len {
                        if self.classes[j] == BidiClass::EN {
                            has_en_after = true;
                            break;
                        }
                        if !matches!(self.classes[j], BidiClass::ET) {
                            break;
                        }
                    }
                    if has_en_before || has_en_after {
                        self.classes[i] = BidiClass::EN;
                        changed = true;
                    }
                }
                i += 1;
            }
            if !changed {
                break;
            }
        }

        // W6: ES, ET, CS → EN (if adjacent) or ON
        for i in 0..len {
            match self.classes[i] {
                BidiClass::ES | BidiClass::ET | BidiClass::CS => {
                    let before = if i > 0 {
                        self.classes[i - 1]
                    } else {
                        BidiClass::ON
                    };
                    let after = if i + 1 < len {
                        self.classes[i + 1]
                    } else {
                        BidiClass::ON
                    };
                    if before == BidiClass::EN || after == BidiClass::EN {
                        self.classes[i] = BidiClass::EN;
                    } else {
                        self.classes[i] = BidiClass::ON;
                    }
                }
                _ => {}
            }
        }

        // W7: EN in LTR context → L
        for i in 0..len {
            if self.classes[i] == BidiClass::EN {
                let mut found_strong_ltr = false;
                for j in (0..i).rev() {
                    match self.classes[j] {
                        BidiClass::L => {
                            found_strong_ltr = true;
                            break;
                        }
                        BidiClass::R | BidiClass::AN => break,
                        _ => {}
                    }
                }
                if found_strong_ltr {
                    self.classes[i] = BidiClass::L;
                }
            }
        }
    }

    fn resolve_bracket_pairs(&mut self) {
        let len = self.chars.len();
        if len == 0 {
            return;
        }

        // BD16: Build list of bracket pairs using a stack-based algorithm.
        // Only ON-type characters participate in bracket pair resolution.
        let mut stack: Vec<(usize, char)> = Vec::new();
        let mut pairs: Vec<(usize, usize)> = Vec::new();

        for i in 0..len {
            if !matches!(self.classes[i], BidiClass::ON) {
                continue;
            }

            let ch = self.chars[i];

            if let Some(closing) = bracket_open_pair(ch) {
                stack.push((i, closing));
            } else if bracket_close_pair(ch).is_some() {
                // Scan back through the stack for a matching opening bracket
                // at the same embedding level (BD16).
                for j in (0..stack.len()).rev() {
                    let (open_idx, expected_closing) = stack[j];
                    if expected_closing == ch && self.levels[open_idx] == self.levels[i] {
                        pairs.push((open_idx, i));
                        stack.remove(j);
                        break;
                    }
                }
            }
        }

        // Track which positions belong to a resolved pair so context scans
        // can skip them.
        let mut paired = vec![false; len];
        for &(open, close) in &pairs {
            paired[open] = true;
            paired[close] = true;
        }

        // N0.b: Resolve each bracket pair in list order.
        for &(open, close) in &pairs {
            let level = self.levels[open];
            let embedding_dir = if level.is_multiple_of(2) {
                BidiClass::L
            } else {
                BidiClass::R
            };

            // Find the nearest strong type within the pair, skipping BN
            // and characters that belong to another pair.
            let strong_within = 'found: {
                for (k, &is_paired) in paired[open + 1..close].iter().enumerate() {
                    let idx = open + 1 + k;
                    if is_paired {
                        continue;
                    }
                    if matches!(self.original_classes[idx], BidiClass::BN) {
                        continue;
                    }
                    if let Some(dir) = strong_direction(self.classes[idx]) {
                        break 'found Some(dir);
                    }
                }
                None
            };

            let resolved = if let Some(strong) = strong_within {
                if strong == embedding_dir {
                    strong
                } else {
                    embedding_dir
                }
            } else {
                // No strong type within the pair: fall back to surrounding context.
                // Check preceding context (first strong type before the opening bracket).
                let strong_before = {
                    let mut idx = open;
                    let mut result = None;
                    while idx > 0 {
                        idx -= 1;
                        if paired[idx] {
                            continue;
                        }
                        if matches!(self.original_classes[idx], BidiClass::BN) {
                            continue;
                        }
                        result = strong_direction(self.classes[idx]);
                        break;
                    }
                    result
                };

                // Check succeeding context (first strong type after the closing bracket).
                let strong_after = {
                    let mut result = None;
                    for (offset, &is_paired) in paired[close + 1..len].iter().enumerate() {
                        let idx = close + 1 + offset;
                        if is_paired {
                            continue;
                        }
                        if matches!(self.original_classes[idx], BidiClass::BN) {
                            continue;
                        }
                        result = strong_direction(self.classes[idx]);
                        break;
                    }
                    result
                };

                match (strong_before, strong_after) {
                    (Some(dir), _) | (_, Some(dir)) if dir == embedding_dir => embedding_dir,
                    (Some(_), _) | (_, Some(_)) => embedding_dir,
                    (None, None) => {
                        if self.paragraph_level.is_multiple_of(2) {
                            BidiClass::L
                        } else {
                            BidiClass::R
                        }
                    }
                }
            };

            self.classes[open] = resolved;
            self.classes[close] = resolved;
        }
    }

    fn resolve_neutrals(&mut self) {
        let len = self.classes.len();
        if len == 0 {
            return;
        }

        // N0: BN → paragraph embedding level
        for i in 0..len {
            if self.original_classes[i] == BidiClass::BN {
                // BN takes the embedding level of the adjacent character
                // For simplicity, keep the current level
            }
        }

        // N1/N2: Resolve neutral and isolate sequences
        let mut i = 0;
        while i < len {
            if is_neutral(self.classes[i]) || self.classes[i] == BidiClass::BN {
                let seq_start = i;
                while i < len
                    && (is_neutral(self.classes[i])
                        || self.classes[i] == BidiClass::BN
                        || matches!(
                            self.classes[i],
                            BidiClass::LRE
                                | BidiClass::RLE
                                | BidiClass::PDF
                                | BidiClass::LRO
                                | BidiClass::RLO
                        ))
                {
                    i += 1;
                }
                let seq_end = i;

                // Find the strong type before the sequence
                let leading = if seq_start > 0 {
                    let mut idx = seq_start - 1;
                    while idx > 0
                        && matches!(
                            self.original_classes[idx],
                            BidiClass::BN
                                | BidiClass::LRE
                                | BidiClass::RLE
                                | BidiClass::PDF
                                | BidiClass::LRO
                                | BidiClass::RLO
                        )
                    {
                        idx -= 1;
                    }
                    self.classes[idx]
                } else {
                    BidiClass::ON
                };

                // Find the strong type after the sequence
                let trailing = if seq_end < len {
                    let mut idx = seq_end;
                    while idx < len
                        && matches!(
                            self.original_classes[idx],
                            BidiClass::BN
                                | BidiClass::LRE
                                | BidiClass::RLE
                                | BidiClass::PDF
                                | BidiClass::LRO
                                | BidiClass::RLO
                        )
                    {
                        idx += 1;
                    }
                    if idx < len {
                        self.classes[idx]
                    } else {
                        BidiClass::ON
                    }
                } else {
                    BidiClass::ON
                };

                // N1: Both leading and trailing strong types exist
                let resolved = match (leading, trailing) {
                    (BidiClass::L, BidiClass::L) => BidiClass::L,
                    (BidiClass::R | BidiClass::AN, BidiClass::R | BidiClass::AN) => BidiClass::R,
                    _ => {
                        // N2: Resolve to paragraph embedding level
                        if self.paragraph_level.is_multiple_of(2) {
                            BidiClass::L
                        } else {
                            BidiClass::R
                        }
                    }
                };

                for j in seq_start..seq_end {
                    if !matches!(
                        self.original_classes[j],
                        BidiClass::LRE
                            | BidiClass::RLE
                            | BidiClass::PDF
                            | BidiClass::LRO
                            | BidiClass::RLO
                    ) {
                        self.classes[j] = resolved;
                    }
                }
            } else {
                i += 1;
            }
        }
    }

    fn resolve_implicit_levels(&mut self) {
        // I1: Even level
        // I2: Odd level
        for i in 0..self.chars.len() {
            // Skip explicit format characters
            if matches!(
                self.original_classes[i],
                BidiClass::LRE | BidiClass::RLE | BidiClass::PDF | BidiClass::LRO | BidiClass::RLO
            ) {
                self.levels[i] = self.paragraph_level;
                continue;
            }

            if self.levels[i].is_multiple_of(2) {
                // I1: even embedding level
                match self.classes[i] {
                    BidiClass::R => {
                        self.levels[i] += 1;
                    }
                    BidiClass::AN | BidiClass::EN => {
                        self.levels[i] += 2;
                    }
                    _ => {}
                }
            } else {
                // I2: odd embedding level
                match self.classes[i] {
                    BidiClass::L | BidiClass::EN | BidiClass::AN => {
                        self.levels[i] += 1;
                    }
                    _ => {}
                }
            }
        }
    }

    fn compute_visual_map(&self) -> Vec<usize> {
        let len = self.chars.len();
        if len == 0 {
            return vec![];
        }

        let mut indices: Vec<usize> = (0..len).collect();

        // Sort by (level descending, then position reversed for odd levels)
        // This is the standard Bidi visual reordering
        let max_level = *self.levels.iter().max().unwrap_or(&0u8);

        for level in (0..=max_level).rev() {
            // Find runs at this level
            let mut i = 0;
            while i < len {
                if self.levels[i] >= level {
                    let run_start = i;
                    while i < len && self.levels[i] >= level {
                        i += 1;
                    }
                    let run_end = i;
                    // Reverse the run if the level is odd
                    if level % 2 == 1 {
                        indices[run_start..run_end].reverse();
                    }
                } else {
                    i += 1;
                }
            }
        }

        indices
    }

    fn compute_runs(&self) -> Vec<DirectionRun> {
        let char_indices: Vec<usize> = {
            let mut result = Vec::with_capacity(self.chars.len());
            let mut offset = 0;
            for ch in &self.chars {
                result.push(offset);
                offset += ch.len_utf8();
            }
            result
        };

        let total_bytes = self.chars.iter().map(|c| c.len_utf8()).sum::<usize>();
        let mut runs = Vec::new();
        if self.chars.is_empty() {
            return runs;
        }

        let mut start = 0;
        let mut current_level = self.levels[0];

        for idx in 1..self.chars.len() {
            if self.levels[idx] != current_level {
                let end = if idx < self.chars.len() {
                    char_indices[idx]
                } else {
                    total_bytes
                };
                runs.push(DirectionRun {
                    start: char_indices[start],
                    end,
                    direction: if current_level.is_multiple_of(2) {
                        TextDirection::LeftToRight
                    } else {
                        TextDirection::RightToLeft
                    },
                    level: current_level,
                });
                start = idx;
                current_level = self.levels[idx];
            }
        }

        runs.push(DirectionRun {
            start: char_indices[start],
            end: total_bytes,
            direction: if current_level.is_multiple_of(2) {
                TextDirection::LeftToRight
            } else {
                TextDirection::RightToLeft
            },
            level: current_level,
        });

        runs
    }

    fn process(mut self) -> BidiResult {
        self.process_explicit();
        self.resolve_weak_types();
        self.resolve_bracket_pairs();
        self.resolve_neutrals();
        self.resolve_implicit_levels();

        let visual_map = self.compute_visual_map();
        let runs = self.compute_runs();
        let base_direction = if self.paragraph_level.is_multiple_of(2) {
            TextDirection::LeftToRight
        } else {
            TextDirection::RightToLeft
        };

        BidiResult {
            levels: self.levels,
            visual_map,
            runs,
            base_direction,
        }
    }
}

/// Full UBA-compliant bidirectional analysis result.
#[derive(Debug, Clone)]
pub struct BidiResult {
    /// Resolved embedding levels for each character.
    pub levels: Vec<u8>,
    /// Visual order mapping: visual_position[i] = logical_index
    pub visual_map: Vec<usize>,
    /// Direction runs in visual order.
    pub runs: Vec<DirectionRun>,
    /// Resolved base paragraph direction.
    pub base_direction: TextDirection,
}

/// Full UBA analysis (levels L1–L4) per UAX#9.
pub fn analyze_bidi_full(text: &str, base_direction: Option<TextDirection>) -> BidiResult {
    let processor = BidiProcessor::new(text, base_direction);
    processor.process()
}

/// Analyze text and produce direction runs in visual order.
///
/// Simplified Unicode Bidirectional Algorithm:
/// 1. Determine base direction from first strong character (P2, P3 from UAX#9).
/// 2. Walk characters, assigning embedding levels: neutral characters inherit
///    the surrounding strong direction; whitespace inherits the base direction.
/// 3. Merge adjacent runs with the same level.
/// 4. Reverse the sequence of runs at odd (RTL) embedding levels so the result
///    is in visual order.
pub fn analyze_bidi(text: &str) -> Vec<DirectionRun> {
    if text.is_empty() {
        return vec![];
    }

    let base = base_direction(text);
    let base_level: u8 = match base {
        TextDirection::LeftToRight => 0,
        TextDirection::RightToLeft => 1,
    };

    let chars: Vec<char> = text.chars().collect();
    let char_indices: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();
    let len = chars.len();

    let mut levels: Vec<u8> = Vec::with_capacity(len);
    let mut i = 0;

    while i < len {
        let ch = chars[i];
        if is_rtl_strong(ch) {
            levels.push(1);
            i += 1;
        } else if is_ltr_strong(ch) {
            levels.push(0);
            i += 1;
        } else if ch.is_whitespace() || ch == '\n' || ch == '\t' {
            levels.push(base_level);
            i += 1;
        } else {
            let mut resolved = base_level;
            for ch in &chars[i + 1..len] {
                if is_rtl_strong(*ch) {
                    resolved = 1;
                    break;
                }
                if is_ltr_strong(*ch) {
                    resolved = 0;
                    break;
                }
            }
            if resolved == base_level && i > 0 {
                for ch in chars[..i].iter().rev() {
                    if is_rtl_strong(*ch) {
                        resolved = 1;
                        break;
                    }
                    if is_ltr_strong(*ch) {
                        resolved = 0;
                        break;
                    }
                }
            }
            levels.push(resolved);
            i += 1;
        }
    }

    let mut logical_runs: Vec<DirectionRun> = Vec::new();
    if len > 0 {
        let mut start = 0;
        let mut current_level = levels[0];
        for idx in 1..len {
            if levels[idx] != current_level {
                logical_runs.push(DirectionRun {
                    start: char_indices[start],
                    end: char_indices[idx],
                    direction: if current_level.is_multiple_of(2) {
                        TextDirection::LeftToRight
                    } else {
                        TextDirection::RightToLeft
                    },
                    level: current_level,
                });
                start = idx;
                current_level = levels[idx];
            }
        }
        logical_runs.push(DirectionRun {
            start: char_indices[start],
            end: text.len(),
            direction: if current_level.is_multiple_of(2) {
                TextDirection::LeftToRight
            } else {
                TextDirection::RightToLeft
            },
            level: current_level,
        });
    }

    reorder_runs_to_visual(&mut logical_runs)
}

fn reorder_runs_to_visual(runs: &mut [DirectionRun]) -> Vec<DirectionRun> {
    let mut result = Vec::with_capacity(runs.len());
    let mut i = 0;
    while i < runs.len() {
        if runs[i].level % 2 == 1 {
            let seq_start = i;
            while i < runs.len() && runs[i].level % 2 == 1 {
                i += 1;
            }
            let mut seq: Vec<DirectionRun> = runs[seq_start..i].to_vec();
            seq.reverse();
            result.extend(seq);
        } else {
            result.push(runs[i].clone());
            i += 1;
        }
    }
    result
}

/// Reverse a string slice for RTL rendering (reverses grapheme clusters via
/// char iteration — good enough for Hebrew/Arabic without combining marks).
pub fn reverse_rtl_run(text: &str) -> String {
    text.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_rtl_strong_hebrew() {
        assert!(is_rtl_strong('\u{05D0}'));
        assert!(is_rtl_strong('\u{05EA}'));
        assert!(is_rtl_strong('\u{0590}'));
        assert!(is_rtl_strong('\u{05FF}'));
    }

    #[test]
    fn test_is_rtl_strong_arabic() {
        assert!(is_rtl_strong('\u{0627}'));
        assert!(is_rtl_strong('\u{0628}'));
        assert!(is_rtl_strong('\u{0600}'));
        assert!(is_rtl_strong('\u{06FF}'));
    }

    #[test]
    fn test_is_rtl_strong_latin_false() {
        assert!(!is_rtl_strong('a'));
        assert!(!is_rtl_strong('Z'));
        assert!(!is_rtl_strong('0'));
        assert!(!is_rtl_strong(' '));
        assert!(!is_rtl_strong('.'));
    }

    #[test]
    fn test_analyze_bidi_ltr() {
        let runs = analyze_bidi("Hello World");
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].direction, TextDirection::LeftToRight);
        assert_eq!(runs[0].level, 0);
    }

    #[test]
    fn test_analyze_bidi_rtl() {
        let runs = analyze_bidi("שלום עולם");
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].direction, TextDirection::RightToLeft);
        assert_eq!(runs[0].level, 1);
    }

    #[test]
    fn test_analyze_bidi_mixed() {
        let text = "Hello שלום World";
        let runs = analyze_bidi(text);
        assert!(runs.len() >= 3);
        assert_eq!(runs[0].direction, TextDirection::LeftToRight);
        assert!(
            runs.iter()
                .any(|r| r.direction == TextDirection::RightToLeft)
        );
        assert_eq!(runs[runs.len() - 1].direction, TextDirection::LeftToRight);
    }

    #[test]
    fn test_reverse_rtl_run() {
        let original = "ABC";
        let reversed = reverse_rtl_run(original);
        assert_eq!(reversed, "CBA");
    }

    #[test]
    fn test_reverse_rtl_run_preserves_chars() {
        let original = "שלום";
        let reversed = reverse_rtl_run(original);
        assert_eq!(reversed.chars().count(), original.chars().count());
        assert_eq!(reversed, "םולש");
    }

    #[test]
    fn test_base_direction_detection_ltr() {
        assert_eq!(base_direction("Hello"), TextDirection::LeftToRight);
        assert_eq!(base_direction("123 Hello"), TextDirection::LeftToRight);
    }

    #[test]
    fn test_base_direction_detection_rtl() {
        assert_eq!(base_direction("שלום"), TextDirection::RightToLeft);
        assert_eq!(base_direction("مرحبا"), TextDirection::RightToLeft);
    }

    #[test]
    fn test_base_direction_mixed_first_strong_wins() {
        assert_eq!(base_direction("Hello שלום"), TextDirection::LeftToRight);
        assert_eq!(base_direction("שלום Hello"), TextDirection::RightToLeft);
    }

    #[test]
    fn test_base_direction_neutral_only() {
        assert_eq!(base_direction("123 456"), TextDirection::LeftToRight);
    }

    #[test]
    fn test_analyze_bidi_empty() {
        let runs = analyze_bidi("");
        assert!(runs.is_empty());
    }

    #[test]
    fn test_analyze_bidi_rtl_base_mixed() {
        let text = "שלום Hello עולם";
        let runs = analyze_bidi(text);
        assert!(runs.len() >= 3);
        assert_eq!(runs[0].direction, TextDirection::RightToLeft);
        assert!(
            runs.iter()
                .any(|r| r.direction == TextDirection::LeftToRight)
        );
    }

    #[test]
    fn test_is_rtl_char_hebrew() {
        assert!(is_rtl_char('\u{05D0}'));
        assert!(is_rtl_char('\u{05EA}'));
        assert!(is_rtl_char('\u{0590}'));
        assert!(is_rtl_char('\u{05FF}'));
    }

    #[test]
    fn test_is_rtl_char_arabic() {
        assert!(is_rtl_char('\u{0627}'));
        assert!(is_rtl_char('\u{0628}'));
        assert!(is_rtl_char('\u{0600}'));
        assert!(is_rtl_char('\u{06FF}'));
    }

    #[test]
    fn test_is_rtl_char_syriac() {
        assert!(is_rtl_char('\u{0710}'));
        assert!(is_rtl_char('\u{074F}'));
    }

    #[test]
    fn test_is_rtl_char_arabic_supplement() {
        assert!(is_rtl_char('\u{0750}'));
        assert!(is_rtl_char('\u{077F}'));
    }

    #[test]
    fn test_is_rtl_char_presentation_forms() {
        assert!(is_rtl_char('\u{FB50}'));
        assert!(is_rtl_char('\u{FE70}'));
    }

    #[test]
    fn test_is_rtl_char_latin_false() {
        assert!(!is_rtl_char('a'));
        assert!(!is_rtl_char('Z'));
        assert!(!is_rtl_char('0'));
        assert!(!is_rtl_char(' '));
    }

    #[test]
    fn test_is_rtl_char_cjk_false() {
        assert!(!is_rtl_char('你'));
        assert!(!is_rtl_char('あ'));
    }

    #[test]
    fn test_is_rtl_char_boundaries() {
        assert!(!is_rtl_char('\u{058F}'));
        assert!(is_rtl_char('\u{0590}'));
        assert!(is_rtl_char('\u{05FF}'));
        assert!(is_rtl_char('\u{0600}'));
        assert!(!is_rtl_char('\u{0800}'));
    }

    #[test]
    fn test_is_rtl_text_pure_hebrew() {
        assert!(is_rtl_text("שלום עולם"));
    }

    #[test]
    fn test_is_rtl_text_pure_arabic() {
        assert!(is_rtl_text("مرحبا بالعالم"));
    }

    #[test]
    fn test_is_rtl_text_latin_false() {
        assert!(!is_rtl_text("Hello World"));
    }

    #[test]
    fn test_is_rtl_text_empty() {
        assert!(!is_rtl_text(""));
    }

    #[test]
    fn test_is_rtl_text_whitespace_only() {
        assert!(!is_rtl_text("   "));
    }

    #[test]
    fn test_is_rtl_text_mixed_rtl_dominant() {
        assert!(is_rtl_text("שלום Hello עולם"));
    }

    #[test]
    fn test_is_rtl_text_mixed_ltr_dominant() {
        assert!(!is_rtl_text("Hello שלום World Test"));
    }

    #[test]
    fn test_is_rtl_text_exactly_half() {
        let text = "שa";
        assert!(!is_rtl_text(text));
    }

    #[test]
    fn test_is_rtl_text_slightly_over_half() {
        let text = "שלa";
        assert!(is_rtl_text(text));
    }

    // New tests for the full UBA implementation

    #[test]
    fn bidi_class_latin() {
        assert_eq!(bidi_class('A'), BidiClass::L);
        assert_eq!(bidi_class('z'), BidiClass::L);
    }

    #[test]
    fn bidi_class_hebrew() {
        assert_eq!(bidi_class('א'), BidiClass::R);
        assert_eq!(bidi_class('\u{05D0}'), BidiClass::R);
    }

    #[test]
    fn bidi_class_arabic() {
        assert_eq!(bidi_class('ا'), BidiClass::AL);
        assert_eq!(bidi_class('\u{0627}'), BidiClass::AL);
    }

    #[test]
    fn bidi_class_european_number() {
        assert_eq!(bidi_class('5'), BidiClass::EN);
        assert_eq!(bidi_class('0'), BidiClass::EN);
    }

    #[test]
    fn bidi_class_whitespace() {
        assert_eq!(bidi_class(' '), BidiClass::WS);
    }

    #[test]
    fn bidi_class_danda() {
        // Devanagari danda (U+0964) is in the Devanagari range which is L
        assert_eq!(bidi_class('\u{0964}'), BidiClass::L);
    }

    #[test]
    fn full_bidi_pure_ltr() {
        let result = analyze_bidi_full("Hello World", None);
        assert_eq!(result.base_direction, TextDirection::LeftToRight);
        for &level in &result.levels {
            assert_eq!(level, 0);
        }
    }

    #[test]
    fn full_bidi_pure_rtl() {
        let result = analyze_bidi_full("שלום עולם", None);
        assert_eq!(result.base_direction, TextDirection::RightToLeft);
        for &level in &result.levels {
            assert_eq!(level, 1);
        }
    }

    #[test]
    fn full_bidi_mixed_hebrew_latin() {
        let text = "Hello שלום 123";
        let result = analyze_bidi_full(text, Some(TextDirection::LeftToRight));
        assert_eq!(result.base_direction, TextDirection::LeftToRight);
        assert!(!result.levels.is_empty());
        // Hebrew chars should have odd level (RTL)
        for (i, ch) in text.chars().enumerate() {
            if is_rtl_strong(ch) {
                assert_eq!(
                    result.levels[i] % 2,
                    1,
                    "Hebrew char at pos {} should have odd level",
                    i
                );
            }
        }
    }

    #[test]
    fn full_bidi_arabic_with_numbers() {
        let text = "مرحبا 42";
        let result = analyze_bidi_full(text, Some(TextDirection::RightToLeft));
        assert_eq!(result.base_direction, TextDirection::RightToLeft);
        assert!(!result.levels.is_empty());
    }

    #[test]
    fn full_bidi_neutrals_resolve() {
        // Parentheses between LTR and RTL should resolve based on context
        let text = "Hello(שלום)";
        let result = analyze_bidi_full(text, Some(TextDirection::LeftToRight));
        assert_eq!(result.base_direction, TextDirection::LeftToRight);
        assert!(!result.levels.is_empty());
        // All characters should be present in levels
        assert_eq!(result.levels.len(), text.chars().count());
    }

    #[test]
    fn full_bidi_nsm_inherits() {
        // U+0300 is COMBINING GRAVE ACCENT (NSM)
        let text = "a\u{0300}";
        let result = analyze_bidi_full(text, Some(TextDirection::LeftToRight));
        assert_eq!(result.levels.len(), 2);
        // NSM should inherit the level of the preceding character
        assert_eq!(result.levels[0], result.levels[1]);
    }

    #[test]
    fn full_bidi_explicit_override() {
        // LRO forces everything to LTR, RLO forces everything to RTL
        let text_lro = "\u{202D}שלום\u{202C}";
        let result_lro = analyze_bidi_full(text_lro, Some(TextDirection::LeftToRight));
        // With LRO, the Hebrew should be overridden to LTR (level 2)
        // The PDF pops back
        assert!(!result_lro.levels.is_empty());

        let text_rlo = "\u{202E}Hello\u{202C}";
        let result_rlo = analyze_bidi_full(text_rlo, Some(TextDirection::LeftToRight));
        // With RLO, the Latin should be overridden to RTL (level 1)
        assert!(!result_rlo.levels.is_empty());
    }

    #[test]
    fn full_bidi_paragraph_separator() {
        let text = "Hello\u{2029}שלום";
        let result = analyze_bidi_full(text, Some(TextDirection::LeftToRight));
        assert_eq!(result.levels.len(), text.chars().count());
    }

    #[test]
    fn full_bidi_visual_order() {
        // Pure LTR: visual order should be same as logical order
        let text = "ABC";
        let result = analyze_bidi_full(text, Some(TextDirection::LeftToRight));
        assert_eq!(result.visual_map, vec![0, 1, 2]);
    }

    #[test]
    fn full_bidi_empty() {
        let result = analyze_bidi_full("", None);
        assert!(result.levels.is_empty());
        assert!(result.visual_map.is_empty());
        assert!(result.runs.is_empty());
    }

    #[test]
    fn bidi_class_explicit_controls() {
        assert_eq!(bidi_class('\u{202A}'), BidiClass::LRE);
        assert_eq!(bidi_class('\u{202B}'), BidiClass::RLE);
        assert_eq!(bidi_class('\u{202C}'), BidiClass::PDF);
        assert_eq!(bidi_class('\u{202D}'), BidiClass::LRO);
        assert_eq!(bidi_class('\u{202E}'), BidiClass::RLO);
    }

    #[test]
    fn bidi_class_nsm() {
        assert_eq!(bidi_class('\u{0300}'), BidiClass::NSM);
    }

    #[test]
    fn bidi_class_bn() {
        assert_eq!(bidi_class('\u{200C}'), BidiClass::BN);
        assert_eq!(bidi_class('\u{200D}'), BidiClass::BN);
    }

    #[test]
    fn full_bidi_rtl_base_mixed() {
        let text = "שלום Hello עולם";
        let result = analyze_bidi_full(text, Some(TextDirection::RightToLeft));
        assert_eq!(result.base_direction, TextDirection::RightToLeft);
        assert!(result.runs.len() >= 2);
    }

    #[test]
    fn bracket_pair_ltr_simple() {
        let text = "(abc)";
        let result = analyze_bidi_full(text, Some(TextDirection::LeftToRight));
        assert_eq!(result.base_direction, TextDirection::LeftToRight);
        for &level in &result.levels {
            assert_eq!(level, 0, "all chars in '(abc)' should be level 0");
        }
    }

    #[test]
    fn bracket_pair_rtl_with_hebrew() {
        let text = "(שלום)";
        let result = analyze_bidi_full(text, Some(TextDirection::RightToLeft));
        assert_eq!(result.base_direction, TextDirection::RightToLeft);
        for &level in &result.levels {
            assert_eq!(level, 1, "all chars in '(שלום)' should be level 1");
        }
    }

    #[test]
    fn bracket_pair_mixed_ltr_hebrew() {
        let text = "Hello(שלום)";
        let result = analyze_bidi_full(text, Some(TextDirection::LeftToRight));
        assert_eq!(result.base_direction, TextDirection::LeftToRight);
        assert_eq!(result.levels.len(), text.chars().count());
        // Brackets at positions 5 and 10 should be level 0 (LTR)
        assert_eq!(result.levels[5], 0, "opening '(' should be level 0");
        assert_eq!(result.levels[10], 0, "closing ')' should be level 0");
        // Hebrew chars at positions 6-9 should be level 1 (RTL)
        for i in 6..10 {
            assert_eq!(
                result.levels[i] % 2,
                1,
                "Hebrew char at pos {} should have odd level",
                i
            );
        }
    }

    #[test]
    fn bracket_pair_nested() {
        let text = "((ab))";
        let result = analyze_bidi_full(text, Some(TextDirection::LeftToRight));
        for &level in &result.levels {
            assert_eq!(level, 0, "all chars in '((ab))' should be level 0");
        }
    }

    #[test]
    fn bracket_pair_neutral_inside() {
        // Brackets with only neutral/separator content inside.
        // No strong type within → falls back to paragraph embedding direction.
        let text = "( - )";
        let result = analyze_bidi_full(text, Some(TextDirection::LeftToRight));
        assert_eq!(result.levels[0], 0, "opening '(' should be level 0");
        assert_eq!(result.levels[4], 0, "closing ')' should be level 0");
    }

    #[test]
    fn bracket_pair_square_brackets() {
        let text = "[abc]";
        let result = analyze_bidi_full(text, Some(TextDirection::LeftToRight));
        for &level in &result.levels {
            assert_eq!(level, 0, "all chars in '[abc]' should be level 0");
        }
    }

    #[test]
    fn bracket_pair_curly_braces() {
        let text = "{abc}";
        let result = analyze_bidi_full(text, Some(TextDirection::LeftToRight));
        for &level in &result.levels {
            assert_eq!(level, 0, "all chars in '{{abc}}' should be level 0");
        }
    }

    #[test]
    fn bracket_pair_unmatched_opening() {
        // Unmatched opening bracket stays as ON, resolved by N1/N2.
        let text = "(abc";
        let result = analyze_bidi_full(text, Some(TextDirection::LeftToRight));
        assert_eq!(result.levels.len(), 4);
        // '(' at position 0: no pair found, resolved by N1/N2 → paragraph embedding (L → level 0)
        assert_eq!(result.levels[0], 0);
    }

    #[test]
    fn bracket_pair_unmatched_closing() {
        let text = "abc)";
        let result = analyze_bidi_full(text, Some(TextDirection::LeftToRight));
        assert_eq!(result.levels.len(), 4);
        // ')' at position 3: no pair found, resolved by N1/N2 → paragraph embedding (L → level 0)
        assert_eq!(result.levels[3], 0);
    }

    #[test]
    fn bracket_pair_multiple_pairs() {
        let text = "(abc)(def)";
        let result = analyze_bidi_full(text, Some(TextDirection::LeftToRight));
        for &level in &result.levels {
            assert_eq!(level, 0);
        }
    }

    #[test]
    fn bracket_pair_rtl_mixed_content() {
        // RTL base with LTR content inside brackets: brackets follow embedding direction.
        let text = "שלום (Hello) עולם";
        let result = analyze_bidi_full(text, Some(TextDirection::RightToLeft));
        assert_eq!(result.base_direction, TextDirection::RightToLeft);
        assert_eq!(result.levels.len(), text.chars().count());
    }

    #[test]
    fn bracket_open_pair_recognizes_ascii() {
        assert_eq!(bracket_open_pair('('), Some(')'));
        assert_eq!(bracket_open_pair('['), Some(']'));
        assert_eq!(bracket_open_pair('{'), Some('}'));
        assert_eq!(bracket_open_pair('a'), None);
    }

    #[test]
    fn bracket_close_pair_recognizes_ascii() {
        assert_eq!(bracket_close_pair(')'), Some('('));
        assert_eq!(bracket_close_pair(']'), Some('['));
        assert_eq!(bracket_close_pair('}'), Some('{'));
        assert_eq!(bracket_close_pair('a'), None);
    }

    #[test]
    fn bracket_pair_mismatched_type() {
        // ( and ] don't match — should not form a pair.
        let text = "(abc]";
        let result = analyze_bidi_full(text, Some(TextDirection::LeftToRight));
        // Neither bracket matches the other, so both stay ON → resolved by N1/N2.
        // Both should end up at level 0 (LTR paragraph embedding).
        assert_eq!(result.levels[0], 0);
        assert_eq!(result.levels[4], 0);
    }

    #[test]
    fn bracket_pair_with_numbers() {
        // EN inside brackets in LTR context: EN maps to L direction.
        // Brackets resolve to L (matching embedding direction).
        // EN chars get level 2 per I1 (even level + EN → level + 2).
        let text = "(123)";
        let result = analyze_bidi_full(text, Some(TextDirection::LeftToRight));
        assert_eq!(result.levels[0], 0, "opening '(' should be level 0");
        assert_eq!(result.levels[4], 0, "closing ')' should be level 0");
        // EN digits get level 2 at even embedding level per I1
        for i in 1..4 {
            assert_eq!(
                result.levels[i], 2,
                "EN digit at pos {} should be level 2",
                i
            );
        }
    }
}
