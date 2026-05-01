# WCET Analysis: Interactive Frame Budget

**Version:** 1.0.0
**Status:** DRAFT
**Date:** 2026-04-23

---

## 1. Scope

Worst-Case Execution Time (WCET) analysis for LDIR interactive preview mode. Target: 16 ms frame budget at 60 FPS (PRF-010), secondary: 6.944 ms at 144 Hz (TC-005, REQ-6.1.3).

---

## 2. Frame Pipeline Model

### 2.1 Pipeline Stages

```
User Input → Parse Delta → Compile Delta → Layout → Paginate → Render → Display
    |             |              |           |          |          |         |
  <0.1ms       <2ms           <4ms       <6ms      <2ms      <4ms     <0.5ms
```

### 2.2 Budget Allocation

| Stage | Budget (60 FPS) | Budget (144 FPS) | Derived From |
|-------|----------------|------------------|--------------|
| Input handling | 0.1 ms | 0.05 ms | Negligible |
| Parse delta | 2.0 ms | 0.5 ms | PRF-001 |
| Compile delta | 4.0 ms | 1.0 ms | PRF-002, PRF-011 |
| Layout (line break) | 6.0 ms | 1.5 ms | PRF-004 |
| Pagination | 2.0 ms | 0.5 ms | PRF-005 |
| Render (GPU upload) | 1.5 ms | 0.4 ms | REQ-6.1.2 |
| Display (vsync) | 0.4 ms | 0.1 ms | GPU compositing |
| **Total** | **16.0 ms** | **4.05 ms** | |
| **Margin** | 0 ms | 2.89 ms | |

### 2.3 Assumptions

- Typical edit: single paragraph changes (REQ-11.1.2)
- Typical scroll: 1-2 pages enter/leave viewport
- No full recompilation: incremental delta only
- Font cache warm from previous frames

---

## 3. Stage-by-Stage WCET Analysis

### 3.1 STAGE-PARSE: Delta S-IR Parsing

**Function:** `parse_sir(delta_bytes)`

| Scenario | Input Size | WCET | Budget | Status |
|----------|-----------|------|--------|--------|
| Typical paste | 10 KB | 0.02 ms | 2.0 ms | 99% headroom |
| Large paste | 1 MB | 2.0 ms | 2.0 ms | 0% headroom |
| Extreme paste | > 1 MB | > 2.0 ms | 2.0 ms | **OVER** |

**Mitigation:** Debounce pastes > 100 KB; show placeholder during paste, compile after idle.

### 3.2 STAGE-COMPILE: Delta Compilation

**Function:** `compile_sir(&delta_doc)` — incremental recompilation.

| Scenario | Entities | WCET | Budget | Status |
|----------|----------|------|--------|--------|
| Single paragraph (200 words) | ~250 | 0.25 ms | 4.0 ms | 94% headroom |
| Cascade (10 paragraphs) | ~2,500 | 2.5 ms | 4.0 ms | 37% headroom |
| Large cascade (20 paragraphs) | ~5,000 | 5.0 ms | 4.0 ms | **OVER** |

**Determinism:** Incremental G-IR must match full compilation for affected pages (REQ-9.2).

### 3.3 STAGE-LAYOUT: Line Breaking

**Function:** `break_lines(glyphs, width, style)` — Knuth-Plass O(n).

| Scenario | Break Points | WCET | Budget | Status |
|----------|-------------|------|--------|--------|
| Typical (80 words) | ~70 | 3.5 us | 6.0 ms | 99.9% headroom |
| Long (200 words) | ~180 | 9 us | 6.0 ms | 99.9% headroom |
| CJK (500 chars) | ~490 | 25 us | 6.0 ms | 99.6% headroom |
| 10 paragraphs | ~700 | 35 us | 6.0 ms | 99.4% headroom |

**Note:** Layout is the most comfortable stage. Branchless KP at ~50ns/state is well within budget.

### 3.4 STAGE-PAGINATE: Page Breaking

**Function:** `paginate(sections)` — DAG branch-and-bound.

| Scenario | Pages | WCET | Budget | Status |
|----------|-------|------|--------|--------|
| Single page overflow | 1 | 0.5 ms | 2.0 ms | 75% headroom |
| 5-page ripple | 5 | 2.5 ms | 2.0 ms | **OVER** |
| 5 pages + floats (3/page) | 5 | 4.0 ms | 2.0 ms | **OVER** |

**Risk:** Global pagination (REQ-4.3.3.3, 100+ pages) cannot run within frame budget. Mitigation: limit to visible pages + 1 buffer; defer global optimization to idle.

### 3.5 STAGE-RENDER: GPU Upload

**Function:** Upload G-IR buffer to GPU via WGPU/Vello.

| Scenario | Commands | WCET | Budget | Status |
|----------|----------|------|--------|--------|
| Typical viewport (2 pages) | ~20,000 | 1.5 ms | 1.5 ms | 0% headroom |
| Zoom change (full redraw) | ~20,000 | 1.5 ms | 1.5 ms | 0% headroom |

**Note:** CPU bounding boxes (REQ-6.1.2) + GPU rasterization fits but has no margin.

---

## 4. End-to-End WCET Summary

### 4.1 Typical Edit (single paragraph, no cascade)

| Stage | WCET | Budget | Headroom |
|-------|------|--------|----------|
| Parse delta | 0.05 ms | 2.0 ms | 97.5% |
| Compile delta | 0.5 ms | 4.0 ms | 87.5% |
| Layout | 0.02 ms | 6.0 ms | 99.7% |
| Paginate | 0.5 ms | 2.0 ms | 75.0% |
| Render | 1.5 ms | 1.5 ms | 0% |
| **Total** | **2.57 ms** | **16.0 ms** | **84% headroom** |

### 4.2 Worst-Case Edit (paste + cascade + floats)

| Stage | WCET | Budget | Headroom |
|-------|------|--------|----------|
| Parse delta | 2.0 ms | 2.0 ms | 0% |
| Compile delta | 2.5 ms | 4.0 ms | 37.5% |
| Layout | 0.04 ms | 6.0 ms | 99.3% |
| Paginate | 4.0 ms | 2.0 ms | **-100% OVER** |
| Render | 1.5 ms | 1.5 ms | 0% |
| **Total** | **10.04 ms** | **16.0 ms** | **37% headroom** |

### 4.3 144 Hz Budget

Typical edit (2.57 ms) fits 6.944 ms budget with 63% headroom. Worst-case (10.04 ms) requires graceful degradation.

---

## 5. Graceful Degradation Strategy

### 5.1 Degradation Levels

| Level | Trigger | Action | Visual Impact |
|-------|---------|--------|---------------|
| **L0** | < 12 ms | Normal rendering | None |
| **L1** | 12-16 ms | Skip global pagination; use cached breaks | Minor |
| **L2** | 16-24 ms | Layout visible paragraphs only | None visible |
| **L3** | > 24 ms | Show cached frame; render in background | 1-frame lag |
| **L4** | > 50 ms | Gray placeholders for uncomputed regions | Visible |

### 5.2 Debounce and Coalescing

| Input | Debounce | Strategy |
|-------|----------|----------|
| Keystroke | 16 ms | Accumulate, compile once/frame |
| Paste | 50 ms | Placeholder during paste, compile after idle |
| Scroll | None | Visible pages only, prefetch adjacent |
| Zoom | None | Re-render at new scale |

### 5.3 Progressive Rendering

```
Frame N:   [Input] → [Parse] → [Compile] → [Layout] → [Render]
Frame N+1:                                         [Paginate (deferred)]
Frame N+2:                                [Global optimization (idle)]
```

### 5.4 Cancellation

- Knuth-Plass: fuel counter per paragraph (analogous to REQ-7.3)
- Cassowary solver: iteration limit with best-effort result
- WASM plugins: fuel limit 100K instructions (NC-010)

---

## 6. Verification

```rust
#[test]
fn test_wcet_typical_edit() {
    let engine = LdirEngine::new();
    let doc = load_sir("fixtures/large.sir");
    let delta = generate_single_word_delta(&doc);
    let start = Instant::now();
    engine.compile_incremental(&doc, &delta);
    assert!(start.elapsed() < Duration::from_millis(5));
}
```

Runtime monitoring via `tracing-chrome` exports per-stage durations with nanosecond resolution (REQ-8.1).

---

## 7. Risk Summary

| Risk | Impact | Mitigation |
|------|--------|------------|
| Large paste (> 1 MB) | Parse exceeds budget | Debounce + placeholder |
| Multi-page cascade | Pagination exceeds budget | L1 degradation |
| Complex constraints | Cassowary slow | Iteration limit + L2 |
| WASM plugin slow | Compile exceeds budget | Fuel limit (NC-010) + L3 |
| 144 Hz mode | All budgets halved | L1 default; L2 on paste |

---

*End of wcet_analysis.md v1.0.0*
