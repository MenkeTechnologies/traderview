// Three-Line Break helpers: parser, validator, body shape,
// localCompute Rust-mirror, trend / flip / run-length / summarize, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_NUM_LINES,
    parseCloses, closesToBlob, validateInputs, buildBody, localCompute,
    trendBadge, flipCount, finalRunLength, summarize, linesToPolyline,
    makeDemoInput,
    fmtUSD, fmtMove, fmtInt, dirLabelKey,
} from '../js/_three_line_break_inputs.js';

// ── constants ─────────────────────────────────────────────────────

test('DEFAULT_NUM_LINES = 3 (matches Rust default)', () => {
    expect(DEFAULT_NUM_LINES).toBe(3);
});

// ── parser ────────────────────────────────────────────────────────

test('parseCloses: comma + whitespace + # comments', () => {
    const r = parseCloses('100, 102 # midday\n104, 106');
    expect(r.errors).toEqual([]);
    expect(r.closes).toEqual([100, 102, 104, 106]);
});

test('parseCloses: rejects non-finite token', () => {
    expect(parseCloses('100, foo').errors[0].message).toMatch(/foo/);
});

test('parseCloses: non-string returns 1 error', () => {
    expect(parseCloses(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts default', () => {
    expect(validateInputs({ closes: [100, 102], num_lines: 3 })).toBe(null);
});

test('validate rejects: bad array / NaN / non-integer N / N<1', () => {
    expect(validateInputs({ closes: 'nope', num_lines: 3 })).toMatch(/closes/);
    expect(validateInputs({ closes: [100, NaN], num_lines: 3 })).toMatch(/finite/);
    expect(validateInputs({ closes: [100], num_lines: 1.5 })).toMatch(/integer/);
    expect(validateInputs({ closes: [100], num_lines: 0 })).toMatch(/≥ 1/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: passes through', () => {
    expect(buildBody({ closes: [100, 102], num_lines: 3 }))
        .toEqual({ closes: [100, 102], num_lines: 3 });
});

// ── localCompute parity (mirrors every Rust #[test] + boundaries) ─

test('local: empty / num_lines<1 → empty', () => {
    expect(localCompute([], 3)).toEqual([]);
    expect(localCompute(Array(5).fill(100), 0)).toEqual([]);
});

test('local: NaN in series → empty', () => {
    expect(localCompute([100, NaN], 3)).toEqual([]);
});

test('local: flat market produces no lines', () => {
    expect(localCompute(Array(20).fill(100), 3)).toEqual([]);
});

test('local: pure uptrend (10 ascending closes) → 9 Up lines', () => {
    const closes = Array.from({ length: 10 }, (_, i) => 100 + i);
    const r = localCompute(closes, 3);
    expect(r.length).toBe(9);
    for (const l of r) expect(l.direction).toBe('Up');
});

test('local: small pullback after 3 up-lines does NOT flip (no break of bar-1 open)', () => {
    // Bars 100,102,104,106,105.5 → 3 Up lines; 105.5 > 100 (open of bar-1) → no flip.
    const r = localCompute([100, 102, 104, 106, 105.5], 3);
    expect(r.length).toBe(3);
    for (const l of r) expect(l.direction).toBe('Up');
});

test('local: deep pullback (break of N-line open) → flip to Down', () => {
    // Bar 4 (99) < bar-1 open (100) → Down line emitted.
    const r = localCompute([100, 102, 104, 106, 99], 3);
    expect(r.some(l => l.direction === 'Down')).toBe(true);
});

test('local: source_index increases for each new line', () => {
    const closes = Array.from({ length: 10 }, (_, i) => 100 + i);
    const r = localCompute(closes, 3);
    for (let i = 1; i < r.length; i++) {
        expect(r[i].source_index).toBeGreaterThan(r[i - 1].source_index);
    }
});

test('local: continuing Up line opens at prior Up line\'s close', () => {
    const closes = [100, 102, 104, 106];
    const r = localCompute(closes, 3);
    for (let i = 1; i < r.length; i++) {
        if (r[i].direction === r[i - 1].direction) {
            expect(r[i].open).toBeCloseTo(r[i - 1].close, 9);
        }
    }
});

test('local: 2-line break is more sensitive than 3-line (sees flip with only 2 up-lines)', () => {
    // Bars 100,102,104 → 2 Up lines. Then 99 < 100 (lowest open of last 2) → flip with N=2.
    // With N=3, only 2 up-lines exist (< 3 required) → no flip.
    const closes = [100, 102, 104, 99];
    const r2 = localCompute(closes, 2);
    const r3 = localCompute(closes, 3);
    expect(r2.some(l => l.direction === 'Down')).toBe(true);
    expect(r3.some(l => l.direction === 'Down')).toBe(false);
});

test('local: 5-line break requires 5 up-lines before considering flip', () => {
    // 5 up-lines then pullback NOT below open of any of them.
    const closes = [100, 102, 104, 106, 108, 110, 109];
    const r5 = localCompute(closes, 5);
    expect(r5.every(l => l.direction === 'Up')).toBe(true);
});

test('local: pure downtrend yields all Down lines', () => {
    const closes = Array.from({ length: 10 }, (_, i) => 110 - i);
    const r = localCompute(closes, 3);
    expect(r.length).toBe(9);
    for (const l of r) expect(l.direction).toBe('Down');
});

test('local: first move sets the initial direction (no flip-block on bar 0)', () => {
    expect(localCompute([100, 99], 3)[0].direction).toBe('Down');
    expect(localCompute([100, 101], 3)[0].direction).toBe('Up');
});

// ── trendBadge / flipCount / finalRunLength ──────────────────────

test('trendBadge: last direction wins', () => {
    expect(trendBadge([]).key).toMatch(/flat/);
    expect(trendBadge([{ direction: 'Up',   open: 100, close: 102, source_index: 1 }]).key).toMatch(/uptrend/);
    expect(trendBadge([
        { direction: 'Up',   open: 100, close: 102, source_index: 1 },
        { direction: 'Down', open: 102, close: 95,  source_index: 5 },
    ]).key).toMatch(/downtrend/);
});

test('flipCount: number of direction changes', () => {
    expect(flipCount([])).toBe(0);
    expect(flipCount([{ direction: 'Up', open: 100, close: 102, source_index: 1 }])).toBe(0);
    expect(flipCount([
        { direction: 'Up', open: 100, close: 102, source_index: 1 },
        { direction: 'Up', open: 102, close: 104, source_index: 2 },
        { direction: 'Down', open: 104, close: 99, source_index: 5 },
        { direction: 'Up', open: 99, close: 110, source_index: 7 },
    ])).toBe(2);
});

test('finalRunLength: contiguous count of final direction', () => {
    expect(finalRunLength([])).toBe(0);
    expect(finalRunLength([
        { direction: 'Up',   open: 100, close: 102, source_index: 1 },
        { direction: 'Down', open: 102, close: 99,  source_index: 5 },
        { direction: 'Down', open: 99,  close: 95,  source_index: 7 },
        { direction: 'Down', open: 95,  close: 92,  source_index: 9 },
    ])).toBe(3);
});

// ── summarize ─────────────────────────────────────────────────────

test('summarize: counts, averages, last_dir, last_close', () => {
    const s = summarize([
        { direction: 'Up',   open: 100, close: 102, source_index: 1 },
        { direction: 'Down', open: 102, close: 95,  source_index: 5 },
    ]);
    expect(s.count).toBe(2);
    expect(s.ups).toBe(1);
    expect(s.downs).toBe(1);
    expect(s.avg_up).toBeCloseTo(2, 9);
    expect(s.avg_down).toBeCloseTo(7, 9);
    expect(s.last_dir).toBe('Down');
    expect(s.last_close).toBe(95);
});

test('summarize: empty → count 0, NaN aggregates, null last_dir', () => {
    const s = summarize([]);
    expect(s.count).toBe(0);
    expect(Number.isNaN(s.avg_up)).toBe(true);
    expect(s.last_dir).toBeNull();
});

// ── linesToPolyline ──────────────────────────────────────────────

test('linesToPolyline: 2 points per line (open + close at same x)', () => {
    const lines = [
        { direction: 'Up',   open: 100, close: 102, source_index: 1 },
        { direction: 'Down', open: 102, close: 95,  source_index: 5 },
    ];
    const { xs, ys } = linesToPolyline(lines);
    expect(xs).toEqual([0, 0, 1, 1]);
    expect(ys).toEqual([100, 102, 102, 95]);
});

test('linesToPolyline: empty → empty arrays', () => {
    const r = linesToPolyline([]);
    expect(r.xs).toEqual([]);
    expect(r.ys).toEqual([]);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + computes without errors', () => {
    for (const k of ['uptrend','downtrend','small-pullback','deep-pullback',
                     'choppy','flat','two-line','five-line']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.closes, inp.num_lines);
        expect(Array.isArray(r)).toBe(true);
    }
});

test('demo uptrend: all Up', () => {
    const inp = makeDemoInput('uptrend');
    const r = localCompute(inp.closes, inp.num_lines);
    expect(r.length).toBeGreaterThan(0);
    for (const l of r) expect(l.direction).toBe('Up');
});

test('demo small-pullback: NO Down lines (insufficient break)', () => {
    const inp = makeDemoInput('small-pullback');
    const r = localCompute(inp.closes, inp.num_lines);
    expect(r.every(l => l.direction === 'Up')).toBe(true);
});

test('demo deep-pullback: at least one Down line', () => {
    const inp = makeDemoInput('deep-pullback');
    const r = localCompute(inp.closes, inp.num_lines);
    expect(r.some(l => l.direction === 'Down')).toBe(true);
});

test('demo flat: no lines emitted', () => {
    const inp = makeDemoInput('flat');
    expect(localCompute(inp.closes, inp.num_lines).length).toBe(0);
});

test('demo five-line: only 4 Up lines exist (< 5 N-line minimum) → pullback bar 95 cannot trigger flip', () => {
    const inp = makeDemoInput('five-line');
    const r = localCompute(inp.closes, inp.num_lines);
    // [100, 102, 104, 106, 108, 95] — produces 4 Up lines (open=100→close=102 etc).
    // Bar 95 sees only 4 < 5 required → no flip → all Up.
    expect(r.length).toBe(4);
    for (const l of r) expect(l.direction).toBe('Up');
});

// ── round-trip + formatters ───────────────────────────────────────

test('closesToBlob round-trips through parseCloses', () => {
    const closes = [100, 102, 104, 106, 99];
    const back = parseCloses(closesToBlob(closes));
    expect(back.errors).toEqual([]);
    expect(back.closes).toEqual(closes);
});

test('dirLabelKey: i18n keys', () => {
    expect(dirLabelKey('Up')).toBe('view.tlb.dir.up');
    expect(dirLabelKey('Down')).toBe('view.tlb.dir.down');
    expect(dirLabelKey()).toBe('view.tlb.dir.unknown');
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtUSD(100)).toBe('$100.00');
    expect(fmtMove(2.5)).toBe('+$2.50');
    expect(fmtMove(-2.5)).toBe('-$2.50');
    expect(fmtInt(42.7)).toBe('42');
    expect(fmtUSD(NaN)).toBe('—');
});
