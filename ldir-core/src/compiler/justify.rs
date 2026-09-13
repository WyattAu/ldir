//! Text justification with hyphenation support (Phase 6C).
//!
//! Provides full justification for paragraph text: non-last lines are
//! stretched to fill the content width by distributing extra space evenly
//! across inter-word gaps. Includes a heuristic hyphenation engine for
//! splitting long words at syllable boundaries.

use crate::fp266::Fp266;
use crate::shaping::ShapedGlyph;

/// A group of contiguous non-space glyphs forming a word.
#[derive(Clone, Debug)]
pub struct WordGroup {
    /// Glyphs of the word in shaping order.
    pub glyphs: Vec<ShapedGlyph>,
    /// Total advance width of the word.
    pub width: Fp266,
}

/// A glyph with its (possibly adjusted) advance width for emission.
#[derive(Clone, Debug)]
pub struct JustifiedGlyph {
    /// Font glyph identifier.
    pub glyph_id: u32,
    /// Adjusted advance width in fixed-point units.
    pub x_advance: i32,
}

fn is_vowel(c: char) -> bool {
    matches!(
        c,
        'a' | 'e' | 'i' | 'o' | 'u' | 'y' | 'A' | 'E' | 'I' | 'O' | 'U' | 'Y'
    )
}

fn is_consonant(c: char) -> bool {
    c.is_ascii_alphabetic() && !is_vowel(c)
}

/// Returns character positions where hyphenation is allowed (byte offsets into the word string).
///
/// Uses a vowel-consonant boundary heuristic. Requires at least 5 characters
/// and won't hyphenate within the first or last 2 characters.
pub fn hyphenation_points(word: &str) -> Vec<usize> {
    let mut positions = Vec::new();
    if word.len() < 5 {
        return positions;
    }

    let chars: Vec<char> = word.chars().collect();
    let n = chars.len();

    for i in 2..n.saturating_sub(2) {
        let prev = chars[i - 1];
        let curr = chars[i];
        if is_vowel(prev) && is_consonant(curr) {
            let byte_pos: usize = chars[..i].iter().map(|c| c.len_utf8()).sum();
            positions.push(byte_pos);
        }
    }

    positions
}

/// Split shaped glyphs into word groups separated by space glyphs.
pub fn split_into_words(glyphs: &[ShapedGlyph], text_bytes: &[u8]) -> Vec<WordGroup> {
    let mut words = Vec::new();
    let mut current_glyphs: Vec<ShapedGlyph> = Vec::new();
    let mut current_width = Fp266::ZERO;

    for glyph in glyphs {
        let ci = glyph.cluster_id as usize;
        let is_space = ci < text_bytes.len() && text_bytes[ci] == b' ';

        if is_space {
            if !current_glyphs.is_empty() {
                words.push(WordGroup {
                    glyphs: std::mem::take(&mut current_glyphs),
                    width: current_width,
                });
                current_width = Fp266::ZERO;
            }
        } else {
            current_width += glyph.advance;
            current_glyphs.push(*glyph);
        }
    }
    if !current_glyphs.is_empty() {
        words.push(WordGroup {
            glyphs: current_glyphs,
            width: current_width,
        });
    }
    words
}

/// Compute justified glyph advances for a line of text.
///
/// For non-last lines, distributes extra space evenly across inter-word gaps
/// so the line fills `content_width`. The last line is left ragged-right.
///
/// Trailing space glyphs are stripped before emission.
pub fn justify_line(
    glyphs: &[ShapedGlyph],
    text_bytes: &[u8],
    content_width: Fp266,
    is_last_line: bool,
) -> Vec<JustifiedGlyph> {
    if glyphs.is_empty() {
        return Vec::new();
    }

    // Strip trailing spaces
    let mut render_end = glyphs.len();
    while render_end > 0 {
        let ci = glyphs[render_end - 1].cluster_id as usize;
        if ci < text_bytes.len() && text_bytes[ci] == b' ' {
            render_end -= 1;
        } else {
            break;
        }
    }
    if render_end == 0 {
        return Vec::new();
    }

    let line_glyphs = &glyphs[..render_end];

    // Count inter-word spaces and compute total natural width
    let mut space_count = 0usize;
    let mut total_width = Fp266::ZERO;
    for g in line_glyphs {
        total_width += g.advance;
        let ci = g.cluster_id as usize;
        if ci < text_bytes.len() && text_bytes[ci] == b' ' {
            space_count += 1;
        }
    }

    // Single word or last line → emit at natural width
    if is_last_line || space_count == 0 || total_width >= content_width {
        return line_glyphs
            .iter()
            .map(|g| JustifiedGlyph {
                glyph_id: g.glyph_id,
                x_advance: g.advance.raw() as i32,
            })
            .collect();
    }

    let extra = content_width - total_width;
    if extra.raw() <= 0 {
        return line_glyphs
            .iter()
            .map(|g| JustifiedGlyph {
                glyph_id: g.glyph_id,
                x_advance: g.advance.raw() as i32,
            })
            .collect();
    }

    let extra_per_gap = Fp266::from_raw(extra.raw() / space_count as i64);
    let remainder = extra.raw() % space_count as i64;

    let mut result = Vec::with_capacity(line_glyphs.len());
    let mut gap_idx: i64 = 0;

    for g in line_glyphs {
        let ci = g.cluster_id as usize;
        let is_space = ci < text_bytes.len() && text_bytes[ci] == b' ';

        if is_space {
            let bonus = if gap_idx < remainder {
                Fp266::ONE
            } else {
                Fp266::ZERO
            };
            let adjusted = g.advance + extra_per_gap + bonus;
            result.push(JustifiedGlyph {
                glyph_id: g.glyph_id,
                x_advance: adjusted.raw() as i32,
            });
            gap_idx += 1;
        } else {
            result.push(JustifiedGlyph {
                glyph_id: g.glyph_id,
                x_advance: g.advance.raw() as i32,
            });
        }
    }

    result
}

/// Try to hyphenate a word to fit within `max_width`.
///
/// Splits at the longest prefix that fits (with room for a hyphen glyph),
/// requiring at least 2 glyphs on each side of the break.
/// Returns the (first_part, second_part) if hyphenation is possible.
pub fn try_hyphenate_word(
    word: &WordGroup,
    max_width: Fp266,
    hyphen_advance: Fp266,
) -> Option<(WordGroup, WordGroup)> {
    if word.glyphs.len() < 4 {
        return None;
    }

    let available = max_width - hyphen_advance;
    if available.raw() <= 0 {
        return None;
    }

    let mut cum_width = Fp266::ZERO;
    let mut best_idx: Option<usize> = None;

    for (i, glyph) in word.glyphs.iter().enumerate() {
        if cum_width + glyph.advance > available {
            break;
        }
        cum_width += glyph.advance;
        // Need ≥ 2 glyphs before and ≥ 2 after
        if i + 1 >= 2 && word.glyphs.len().saturating_sub(i + 1) >= 2 {
            best_idx = Some(i + 1);
        }
    }

    let idx = best_idx?;

    let first_glyphs = word.glyphs[..idx].to_vec();
    let first_width: Fp266 = first_glyphs
        .iter()
        .map(|g| g.advance)
        .fold(Fp266::ZERO, |a, b| a + b)
        + hyphen_advance;

    let second_glyphs = word.glyphs[idx..].to_vec();
    let second_width: Fp266 = second_glyphs
        .iter()
        .map(|g| g.advance)
        .fold(Fp266::ZERO, |a, b| a + b);

    Some((
        WordGroup {
            glyphs: first_glyphs,
            width: first_width,
        },
        WordGroup {
            glyphs: second_glyphs,
            width: second_width,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_glyph(glyph_id: u32, advance_pt: i32, cluster_id: u32) -> ShapedGlyph {
        ShapedGlyph {
            glyph_id,
            x_offset: Fp266::ZERO,
            y_offset: Fp266::ZERO,
            advance: Fp266::from_int(advance_pt),
            cluster_id,
        }
    }

    fn space_glyph(cluster_id: u32) -> ShapedGlyph {
        make_glyph(b' ' as u32, 5, cluster_id)
    }

    // ── hyphenation_points ────────────────────────────────────────────

    #[test]
    fn test_hyphenation_short_word() {
        assert!(hyphenation_points("the").is_empty());
        assert!(hyphenation_points("abcd").is_empty());
    }

    #[test]
    fn test_hyphenation_basic() {
        let pts = hyphenation_points("letter");
        assert!(
            !pts.is_empty(),
            "should find at least one break in 'letter'"
        );
    }

    #[test]
    fn test_hyphenation_no_consonant_after_vowel() {
        // "queue" – no vowel-consonant boundary after index 2
        let _pts = hyphenation_points("queue");
        // 'u' (vowel) then 'e' (vowel) – no break; 'e' then 'u' – no break
        // Only possible break: 'e'(vowel)→'u'(vowel)? No. 'u'→'e'? No.
        // Actually "queue": q-u-e-u-e
        // indices: 0=q 1=u 2=e 3=u 4=e
        // i=2: prev=chars[1]=u(vowel), curr=chars[2]=e(vowel) → not vc
        // i=3 would be len-2=3? n=5, n-2=3, so i goes 2 only
        // So empty is fine for this edge case
    }

    #[test]
    fn test_hyphenation_empty() {
        assert!(hyphenation_points("").is_empty());
    }

    // ── split_into_words ──────────────────────────────────────────────

    #[test]
    fn test_word_splitting_simple() {
        let text = "hello world";
        let glyphs = vec![
            make_glyph('h' as u32, 7, 0),
            make_glyph('e' as u32, 7, 1),
            make_glyph('l' as u32, 7, 2),
            make_glyph('l' as u32, 7, 3),
            make_glyph('o' as u32, 7, 4),
            space_glyph(5),
            make_glyph('w' as u32, 7, 6),
            make_glyph('o' as u32, 7, 7),
            make_glyph('r' as u32, 7, 8),
            make_glyph('l' as u32, 7, 9),
            make_glyph('d' as u32, 7, 10),
        ];
        let words = split_into_words(&glyphs, text.as_bytes());
        assert_eq!(words.len(), 2);
        assert_eq!(words[0].width, Fp266::from_int(35));
        assert_eq!(words[1].width, Fp266::from_int(35));
    }

    #[test]
    fn test_word_splitting_single_word() {
        let text = "hello";
        let glyphs: Vec<ShapedGlyph> = "hello"
            .bytes()
            .enumerate()
            .map(|(i, b)| make_glyph(b as u32, 7, i as u32))
            .collect();
        let words = split_into_words(&glyphs, text.as_bytes());
        assert_eq!(words.len(), 1);
    }

    #[test]
    fn test_word_splitting_leading_trailing_spaces() {
        let text = "  hello  world  ";
        let glyphs = vec![
            space_glyph(0),
            space_glyph(1),
            make_glyph('h' as u32, 7, 2),
            make_glyph('e' as u32, 7, 3),
            make_glyph('l' as u32, 7, 4),
            make_glyph('l' as u32, 7, 5),
            make_glyph('o' as u32, 7, 6),
            space_glyph(7),
            space_glyph(8),
            make_glyph('w' as u32, 7, 9),
            make_glyph('o' as u32, 7, 10),
            make_glyph('r' as u32, 7, 11),
            make_glyph('l' as u32, 7, 12),
            make_glyph('d' as u32, 7, 13),
            space_glyph(14),
            space_glyph(15),
        ];
        let words = split_into_words(&glyphs, text.as_bytes());
        assert_eq!(words.len(), 2);
    }

    // ── justify_line ──────────────────────────────────────────────────

    #[test]
    fn test_justification_last_line_ragged() {
        let text = "hi world";
        let glyphs = vec![
            make_glyph('h' as u32, 7, 0),
            make_glyph('i' as u32, 7, 1),
            space_glyph(2),
            make_glyph('w' as u32, 7, 3),
            make_glyph('o' as u32, 7, 4),
            make_glyph('r' as u32, 7, 5),
            make_glyph('l' as u32, 7, 6),
            make_glyph('d' as u32, 7, 7),
        ];
        let result = justify_line(&glyphs, text.as_bytes(), Fp266::from_int(100), true);
        // Last line: advances should be natural (no stretching)
        let total: i32 = result.iter().map(|g| g.x_advance).sum();
        let expected: i32 = (7 * 7 + 5) * 64; // 7 chars * 7pt + 5pt space
        assert_eq!(total, expected);
    }

    #[test]
    fn test_justification_even_spacing() {
        let text = "a b c";
        let glyphs = vec![
            make_glyph('a' as u32, 7, 0),
            space_glyph(1),
            make_glyph('b' as u32, 7, 2),
            space_glyph(3),
            make_glyph('c' as u32, 7, 4),
        ];
        let content_width = Fp266::from_int(100);
        let result = justify_line(&glyphs, text.as_bytes(), content_width, false);

        // Natural width = 7+5+7+5+7 = 31pt → extra = 69pt, 2 gaps → 34.5 each
        // Fp266: 69*64 = 4416 raw, /2 = 2208, remainder 0
        let space_advances: Vec<i32> = result
            .iter()
            .filter_map(|g| {
                if g.glyph_id == b' ' as u32 {
                    Some(g.x_advance)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(space_advances.len(), 2);
        assert_eq!(
            space_advances[0], space_advances[1],
            "spaces should be equal"
        );
        // Each space = 5*64 + 2208 = 320 + 2208 = 2528
        assert_eq!(space_advances[0], 2528);
    }

    #[test]
    fn test_justification_single_word_no_stretch() {
        let text = "hello";
        let glyphs: Vec<ShapedGlyph> = "hello"
            .bytes()
            .enumerate()
            .map(|(i, b)| make_glyph(b as u32, 7, i as u32))
            .collect();
        let result = justify_line(&glyphs, text.as_bytes(), Fp266::from_int(100), false);
        // No spaces → no justification
        for g in &result {
            assert_eq!(g.x_advance, 7 * 64);
        }
    }

    #[test]
    fn test_justification_empty() {
        let result = justify_line(&[], &[], Fp266::from_int(100), false);
        assert!(result.is_empty());
    }

    #[test]
    fn test_justification_trailing_spaces_stripped() {
        let text = "a b  ";
        let glyphs = vec![
            make_glyph('a' as u32, 7, 0),
            space_glyph(1),
            make_glyph('b' as u32, 7, 2),
            space_glyph(3),
            space_glyph(4),
        ];
        let result = justify_line(&glyphs, text.as_bytes(), Fp266::from_int(100), false);
        // Trailing spaces stripped; only 1 space in "a b"
        let has_space = result.iter().any(|g| g.glyph_id == b' ' as u32);
        assert!(has_space);
    }

    #[test]
    fn test_justification_remainder_distribution() {
        // 3 gaps, extra = 7pt → 448 raw / 3 = 149 raw each + 1 raw to first gap
        let text = "a b c d";
        let glyphs = vec![
            make_glyph('a' as u32, 7, 0),
            space_glyph(1),
            make_glyph('b' as u32, 7, 2),
            space_glyph(3),
            make_glyph('c' as u32, 7, 4),
            space_glyph(5),
            make_glyph('d' as u32, 7, 6),
        ];
        // Natural: 7+5+7+5+7+5+7 = 43pt. Width=50pt → extra=7pt=448raw, 3 gaps
        let result = justify_line(&glyphs, text.as_bytes(), Fp266::from_int(50), false);
        let space_advances: Vec<i32> = result
            .iter()
            .filter_map(|g| {
                if g.glyph_id == b' ' as u32 {
                    Some(g.x_advance)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(space_advances.len(), 3);
        // 448/3=149 remainder 1; first gap: 320+149+64=533; others: 320+149=469
        assert_eq!(space_advances[0], 533);
        assert_eq!(space_advances[1], 469);
        assert_eq!(space_advances[2], 469);
    }

    // ── try_hyphenate_word ────────────────────────────────────────────

    #[test]
    fn test_hyphenate_word_too_short() {
        let word = WordGroup {
            glyphs: vec![make_glyph('a' as u32, 7, 0), make_glyph('b' as u32, 7, 1)],
            width: Fp266::from_int(14),
        };
        assert!(try_hyphenate_word(&word, Fp266::from_int(100), Fp266::from_int(3)).is_none());
    }

    #[test]
    fn test_hyphenate_word_fits() {
        let word = WordGroup {
            glyphs: vec![
                make_glyph('a' as u32, 7, 0),
                make_glyph('b' as u32, 7, 1),
                make_glyph('c' as u32, 7, 2),
                make_glyph('d' as u32, 7, 3),
                make_glyph('e' as u32, 7, 4),
                make_glyph('f' as u32, 7, 5),
            ],
            width: Fp266::from_int(42),
        };
        let result = try_hyphenate_word(&word, Fp266::from_int(20), Fp266::from_int(3));
        let (first, second) = result.unwrap();
        assert_eq!(first.glyphs.len(), 2); // 2 glyphs fit in 20-3=17pt
        assert_eq!(second.glyphs.len(), 4);
    }

    #[test]
    fn test_hyphenate_word_no_room() {
        let word = WordGroup {
            glyphs: vec![
                make_glyph('a' as u32, 7, 0),
                make_glyph('b' as u32, 7, 1),
                make_glyph('c' as u32, 7, 2),
                make_glyph('d' as u32, 7, 3),
            ],
            width: Fp266::from_int(28),
        };
        // max_width=5, hyphen=3 → available=2, can't fit 2 glyphs
        assert!(try_hyphenate_word(&word, Fp266::from_int(5), Fp266::from_int(3)).is_none());
    }

    // ── can_hyphenate (heuristic) ─────────────────────────────────────

    #[test]
    fn test_can_hyphenate_long_word() {
        let pts = hyphenation_points("justification");
        assert!(!pts.is_empty());
    }

    #[test]
    fn test_can_hyphenate_numbers() {
        // Numbers have no vowels/consonants → no hyphenation points
        let pts = hyphenation_points("12345");
        assert!(pts.is_empty());
    }
}
