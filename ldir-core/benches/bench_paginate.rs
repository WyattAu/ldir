use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ldir_core::fp266::Fp266;
use ldir_core::layout::pagination::{LineBlock, PaginationOptions, ParagraphBlock, paginate};

fn make_text_pages(page_count: usize, lines_per_para: usize) -> Vec<ParagraphBlock> {
    let line_height = Fp266::from_int(12);
    let baseline = Fp266::from_int(10);
    let usable = 648;
    let lines_per_page = usable / 12;
    let paras_per_page = lines_per_page / lines_per_para;
    let total_paras = page_count * paras_per_page;

    (0..total_paras)
        .map(|_| {
            let lines = (0..lines_per_para)
                .map(|_| LineBlock::new(line_height, baseline))
                .collect();
            ParagraphBlock::new(lines)
        })
        .collect()
}

fn make_mixed_pages(page_count: usize) -> Vec<ParagraphBlock> {
    let line_height = Fp266::from_int(12);
    let baseline = Fp266::from_int(10);
    let usable = 648;
    let lines_per_page = usable / 12;

    let mut items = Vec::new();
    let mut accumulated_lines = 0usize;
    let mut para_index = 0usize;

    while accumulated_lines < page_count * lines_per_page {
        let lines_per_para = 5 + (para_index % 11);
        let lines = (0..lines_per_para)
            .map(|_| LineBlock::new(line_height, baseline))
            .collect();
        items.push(ParagraphBlock::new(lines));
        accumulated_lines += lines_per_para;
        para_index += 1;
    }

    items
}

fn make_orphan_trigger_pages(page_count: usize) -> Vec<ParagraphBlock> {
    let line_height = Fp266::from_int(12);
    let baseline = Fp266::from_int(10);
    let usable = 648;
    let lines_per_page = usable / 12;

    let mut items = Vec::new();
    let mut accumulated_lines = 0usize;
    let mut para_index = 0usize;

    while accumulated_lines < page_count * lines_per_page {
        let lines_per_para = match para_index % 3 {
            0 => 13,
            1 => 1,
            _ => 11,
        };
        let lines = (0..lines_per_para)
            .map(|_| LineBlock::new(line_height, baseline))
            .collect();
        items.push(ParagraphBlock::new(lines));
        accumulated_lines += lines_per_para;
        para_index += 1;
    }

    items
}

fn default_options() -> PaginationOptions {
    PaginationOptions::new(
        Fp266::from_int(792),
        Fp266::from_int(612),
        Fp266::from_int(72),
        Fp266::from_int(72),
    )
}

fn bm_paginate_001a(c: &mut Criterion) {
    let items = make_text_pages(100, 10);
    let options = default_options();

    c.bench_function("BM-PAGINATE-001a/page_break_text_only", |b| {
        b.iter(|| black_box(paginate(&items, &options)))
    });
}

fn bm_paginate_001b(c: &mut Criterion) {
    let items = make_mixed_pages(100);
    let options = default_options();

    c.bench_function("BM-PAGINATE-001b/page_break_with_floats", |b| {
        b.iter(|| black_box(paginate(&items, &options)))
    });
}

fn bm_paginate_001c(c: &mut Criterion) {
    let items = make_orphan_trigger_pages(100);
    let mut options = default_options();
    options.widow_penalty = 100.0;
    options.orphan_penalty = 100.0;

    c.bench_function("BM-PAGINATE-001c/page_break_orphan_avoidance", |b| {
        b.iter(|| black_box(paginate(&items, &options)))
    });
}

fn bm_paginate_001d(c: &mut Criterion) {
    let items = make_text_pages(500, 10);
    let options = default_options();

    c.bench_function("BM-PAGINATE-001d/page_break_500_pages", |b| {
        b.iter(|| black_box(paginate(&items, &options)))
    });
}

criterion_group!(
    benches,
    bm_paginate_001a,
    bm_paginate_001b,
    bm_paginate_001c,
    bm_paginate_001d,
);
criterion_main!(benches);
