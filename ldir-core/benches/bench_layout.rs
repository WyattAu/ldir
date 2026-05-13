use bumpalo::Bump;
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

use ldir_core::fp266::Fp266;
use ldir_core::layout::linebreak::cjk::insert_cjk_breaks;
use ldir_core::layout::linebreak::{LineBreakItem, LineBreakOptions, linebreak};

fn make_latin_paragraph(word_count: usize) -> Vec<LineBreakItem> {
    let words: Vec<&'static str> = vec![
        "The",
        "quick",
        "brown",
        "fox",
        "jumps",
        "over",
        "the",
        "lazy",
        "dog",
        "and",
        "through",
        "many",
        "fields",
        "of",
        "golden",
        "wheat",
        "under",
        "a",
        "bright",
        "blue",
        "sky",
        "while",
        "birds",
        "sing",
        "their",
        "morning",
        "songs",
        "in",
        "tall",
        "green",
        "trees",
        "near",
        "the",
        "river",
        "bank",
        "where",
        "fish",
        "swim",
        "gracefully",
        "below",
    ];
    let space_width = Fp266::from_int(3);
    let glue_stretch = Fp266::from_int(2);
    let glue_shrink = Fp266::from_int(1);

    let mut items = Vec::with_capacity(word_count * 2);
    for i in 0..word_count {
        let word = words[i % words.len()];
        items.push(LineBreakItem {
            width: Fp266::from_int((word.len() as i32) * 7),
            stretchability: Fp266::ZERO,
            shrinkability: Fp266::ZERO,
            penalty: 0.0,
            is_mandatory: false,
            is_hyphenation: false,
            hyphen_width: Fp266::ZERO,
            text: word,
        });
        if i < word_count - 1 {
            items.push(LineBreakItem {
                width: space_width,
                stretchability: glue_stretch,
                shrinkability: glue_shrink,
                penalty: 0.0,
                is_mandatory: false,
                is_hyphenation: false,
                hyphen_width: Fp266::ZERO,
                text: " ",
            });
        }
    }
    items.push(LineBreakItem {
        width: Fp266::ZERO,
        stretchability: Fp266::ZERO,
        shrinkability: Fp266::ZERO,
        penalty: -10000.0,
        is_mandatory: true,
        is_hyphenation: false,
        hyphen_width: Fp266::ZERO,
        text: "",
    });
    items
}

fn make_cjk_paragraph(char_count: usize) -> Vec<LineBreakItem> {
    let cjk_chars = "\u{4E16}\u{754C}\u{4E0A}\u{6700}\u{5927}\u{7684}\u{56FE}\u{4E66}\u{9986}\u{662F}\u{7F8E}\u{56FD}\u{56FD}\u{4F1A}\u{56FE}\u{4E66}\u{9986}\u{5B83}\u{5EFA}\u{4E8E}\u{4E00}\u{4E5D}\u{4E09}\u{5E74}\u{62A5}\u{544A}\u{663E}\u{793A}\u{8BE5}\u{56FE}\u{4E66}\u{9986}\u{6536}\u{85CF}\u{4E86}\u{8D85}\u{8FC7}\u{4E00}\u{4EBF}\u{4EFD}\u{4E66}\u{7C4D}\u{548C}\u{624B}\u{7A3F}";
    let char_width = Fp266::from_int(12);
    let chars: Vec<char> = cjk_chars.chars().collect();
    let mut items = Vec::with_capacity(char_count);
    for i in 0..char_count {
        let _ch = chars[i % chars.len()];
        items.push(LineBreakItem {
            width: char_width,
            stretchability: Fp266::ZERO,
            shrinkability: Fp266::ZERO,
            penalty: 0.0,
            is_mandatory: false,
            is_hyphenation: false,
            hyphen_width: Fp266::ZERO,
            text: "",
        });
    }
    items.push(LineBreakItem {
        width: Fp266::ZERO,
        stretchability: Fp266::ZERO,
        shrinkability: Fp266::ZERO,
        penalty: -10000.0,
        is_mandatory: true,
        is_hyphenation: false,
        hyphen_width: Fp266::ZERO,
        text: "",
    });
    items
}

fn make_mixed_script_paragraph() -> Vec<LineBreakItem> {
    let latin_words: Vec<&'static str> = vec![
        "The", "quick", "brown", "fox", "jumps", "over", "the", "lazy", "dog", "and", "through",
        "many", "fields",
    ];
    let cjk_chars = "\u{4E16}\u{754C}\u{4E0A}\u{6700}\u{5927}\u{7684}\u{56FE}\u{4E66}\u{9986}\u{662F}\u{7F8E}\u{56FD}\u{56FD}\u{4F1A}";
    let arabic_words: Vec<&'static str> = vec![
        "\u{0645}\u{0631}\u{062D}\u{0628}\u{0627}",
        "\u{0628}\u{0627}\u{0644}\u{0639}\u{0627}\u{0644}\u{0645}",
    ];

    let space_width = Fp266::from_int(3);
    let glue_stretch = Fp266::from_int(2);
    let glue_shrink = Fp266::from_int(1);
    let latin_char_width: i32 = 7;
    let cjk_char_width: i32 = 12;

    let mut items = Vec::new();

    for word in &latin_words {
        items.push(LineBreakItem {
            width: Fp266::from_int(word.len() as i32 * latin_char_width),
            stretchability: Fp266::ZERO,
            shrinkability: Fp266::ZERO,
            penalty: 0.0,
            is_mandatory: false,
            is_hyphenation: false,
            hyphen_width: Fp266::ZERO,
            text: word,
        });
        items.push(LineBreakItem {
            width: space_width,
            stretchability: glue_stretch,
            shrinkability: glue_shrink,
            penalty: 0.0,
            is_mandatory: false,
            is_hyphenation: false,
            hyphen_width: Fp266::ZERO,
            text: " ",
        });
    }

    for ch in cjk_chars.chars() {
        let _ = ch; // CJK chars used as potential break points
        items.push(LineBreakItem {
            width: Fp266::from_int(cjk_char_width),
            stretchability: Fp266::ZERO,
            shrinkability: Fp266::ZERO,
            penalty: 0.0,
            is_mandatory: false,
            is_hyphenation: false,
            hyphen_width: Fp266::ZERO,
            text: "",
        });
    }

    for word in &arabic_words {
        items.push(LineBreakItem {
            width: Fp266::from_int(word.chars().count() as i32 * latin_char_width),
            stretchability: Fp266::ZERO,
            shrinkability: Fp266::ZERO,
            penalty: 0.0,
            is_mandatory: false,
            is_hyphenation: false,
            hyphen_width: Fp266::ZERO,
            text: word,
        });
        items.push(LineBreakItem {
            width: space_width,
            stretchability: glue_stretch,
            shrinkability: glue_shrink,
            penalty: 0.0,
            is_mandatory: false,
            is_hyphenation: false,
            hyphen_width: Fp266::ZERO,
            text: " ",
        });
    }

    items.push(LineBreakItem {
        width: Fp266::ZERO,
        stretchability: Fp266::ZERO,
        shrinkability: Fp266::ZERO,
        penalty: -10000.0,
        is_mandatory: true,
        is_hyphenation: false,
        hyphen_width: Fp266::ZERO,
        text: "",
    });
    items
}

fn bench_line_break_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("BM-LAYOUT-001");
    let options = LineBreakOptions::default();

    let short = make_latin_paragraph(20);
    group.bench_function(BenchmarkId::new("line_break_short", "20_words"), |b| {
        b.iter(|| {
            let bump: Bump = Bump::new();
            black_box(linebreak(&short, &options, &bump))
        })
    });

    let typical = make_latin_paragraph(80);
    group.bench_function(BenchmarkId::new("line_break_typical", "80_words"), |b| {
        b.iter(|| {
            let bump: Bump = Bump::new();
            black_box(linebreak(&typical, &options, &bump))
        })
    });

    let long = make_latin_paragraph(200);
    group.bench_function(BenchmarkId::new("line_break_long", "200_words"), |b| {
        b.iter(|| {
            let bump: Bump = Bump::new();
            black_box(linebreak(&long, &options, &bump))
        })
    });

    let cjk_items = make_cjk_paragraph(80);
    group.bench_function(BenchmarkId::new("line_break_cjk", "80_chars"), |b| {
        b.iter(|| {
            let bump: Bump = Bump::new();
            black_box(linebreak(&cjk_items, &options, &bump))
        })
    });

    let mixed_items = make_mixed_script_paragraph();
    group.bench_function(
        BenchmarkId::new("line_break_mixed_script", "latin+cjk+arabic"),
        |b| {
            b.iter(|| {
                let bump: Bump = Bump::new();
                black_box(linebreak(&mixed_items, &options, &bump))
            })
        },
    );

    group.finish();
}

fn bench_line_break_throughput(c: &mut Criterion) {
    let typical = make_latin_paragraph(80);
    let options = LineBreakOptions::default();

    c.bench_function("BM-LAYOUT-001f/line_break_1000_paragraphs", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let bump: Bump = Bump::new();
                black_box(linebreak(&typical, &options, &bump));
            }
        })
    });
}

fn bench_cjk_insert_breaks(c: &mut Criterion) {
    let cjk_items = make_cjk_paragraph(80);
    let cjk_text: String = "\u{4E16}\u{754C}\u{4E0A}\u{6700}\u{5927}\u{7684}\u{56FE}\u{4E66}\u{9986}\u{662F}\u{7F8E}\u{56FD}\u{56FD}\u{4F1A}\u{56FE}\u{4E66}\u{9986}\u{5B83}\u{5EFA}\u{4E8E}\u{4E00}\u{4E5D}\u{4E09}\u{5E74}\u{62A5}\u{544A}\u{663E}\u{793A}\u{8BE5}\u{56FE}\u{4E66}\u{9986}\u{6536}\u{85CF}\u{4E86}\u{8D85}\u{8FC7}\u{4E00}\u{4EBF}\u{4EFD}\u{4E66}\u{7C4D}\u{548C}\u{624B}\u{7A3F}"
        .chars()
        .cycle()
        .take(80)
        .collect();

    c.bench_function("BM-LAYOUT-001/cjk_insert_breaks_80chars", |b| {
        b.iter(|| {
            let bump = Bump::new();
            black_box(insert_cjk_breaks(&cjk_text, &cjk_items, &bump))
        })
    });
}

criterion_group!(
    benches,
    bench_line_break_sizes,
    bench_line_break_throughput,
    bench_cjk_insert_breaks,
);
criterion_main!(benches);
