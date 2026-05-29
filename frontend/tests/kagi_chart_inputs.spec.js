// Kagi-chart helpers: parser, validator, body shape,
// localCompute Rust-mirror, classifyYangYin, summarize, badges, demos.

import { test, expect } from 'vitest';
import {
    KINDS, DEFAULT_INPUTS,
    parseCloses, validateInputs, buildBody, localCompute,
    classifyYangYin, summarize, trendBadge, linesToPolyline,
    makeDemoInput,
    fmtUSD, fmtMove, fmtInt, fmtPct,
    dirLabelKey, yangYinLabelKey,
} from '../js/_kagi_chart_inputs.js';

// ── constants ─────────────────────────────────────────────────────

test('KINDS exposes the two Rust enum strings (snake_case)', () => {
    expect(KINDS).toEqual(['absolute', 'pct']);
});

// ── parser ────────────────────────────────────────────────────────

test('parseCloses: comma + whitespace-separated; # comments ignored', () => {
    const r = parseCloses('100, 101 # mid-day\n102.5  103');
    expect(r.errors).toEqual([]);
    expect(r.closes).toEqual([100, 101, 102.5, 103]);
});

test('parseCloses: rejects non-finite + non-positive tokens', () => {
    expect(parseCloses('100, foo').errors[0].message).toMatch(/foo/);
    expect(parseCloses('100, -5').errors[0].message).toMatch(/-5/);
    expect(parseCloses('100, 0').errors[0].message).toMatch(/0/);
});

test('parseCloses: non-string returns 1 error', () => {
    expect(parseCloses(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts default', () => {
    expect(validateInputs({ ...DEFAULT_INPUTS, closes: [100, 101] })).toBe(null);
});

test('validate rejects: bad array, non-finite, non-positive, bad reversal, bad kind', () => {
    expect(validateInputs({ ...DEFAULT_INPUTS, closes: 'nope' })).toMatch(/closes/);
    expect(validateInputs({ ...DEFAULT_INPUTS, closes: [100, NaN] })).toMatch(/finite/);
    expect(validateInputs({ ...DEFAULT_INPUTS, closes: [100, 0] })).toMatch(/> 0/);
    expect(validateInputs({ ...DEFAULT_INPUTS, closes: [100], reversal: 0 })).toMatch(/reversal/);
    expect(validateInputs({ ...DEFAULT_INPUTS, closes: [100], kind: 'huge' })).toMatch(/kind/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: passes through (no Decimal in this route)', () => {
    const body = buildBody({ closes: [100, 101], reversal: 1, kind: 'absolute' });
    expect(body).toEqual({ closes: [100, 101], reversal: 1, kind: 'absolute' });
});

// ── localCompute parity (mirrors every Rust #[test] + boundaries) ─

test('local: empty / invalid reversal / non-positive close → empty', () => {
    expect(localCompute([], 1, 'absolute')).toEqual([]);
    expect(localCompute([100, 100, 100], 0, 'absolute')).toEqual([]);
    expect(localCompute([100, -1], 1, 'absolute')).toEqual([]);
});

test('local: NaN in series → empty', () => {
    expect(localCompute([100, NaN], 1, 'absolute')).toEqual([]);
});

test('local: flat market produces no lines (no reversal trigger)', () => {
    expect(localCompute(Array(20).fill(100), 1, 'absolute')).toEqual([]);
});

test('local: pure uptrend yields a single Up line ending at the peak', () => {
    const closes = Array.from({ length: 20 }, (_, i) => 100 + i);
    const r = localCompute(closes, 1, 'absolute');
    expect(r.length).toBe(1);
    expect(r[0].direction).toBe('Up');
    expect(r[0].end_price).toBeCloseTo(119, 9);
});

test('local: up-then-down produces an Up line followed by a Down line', () => {
    const ups = Array.from({ length: 20 }, (_, i) => 100 + i);
    const downs = Array.from({ length: 20 }, (_, i) => 119 - i);
    const r = localCompute([...ups, ...downs], 2, 'absolute');
    expect(r.length).toBeGreaterThanOrEqual(2);
    expect(r[0].direction).toBe('Up');
    expect(r[1].direction).toBe('Down');
});

test('local: pct threshold scales with price (gentle 0.5%/step uptrend)', () => {
    const closes = Array.from({ length: 20 }, (_, i) => 100 * (1 + i * 0.005));
    const r = localCompute(closes, 0.5, 'pct');
    expect(r.length).toBeGreaterThan(0);
    expect(r[0].direction).toBe('Up');
});

test('local: reversal exactly at threshold triggers (≥ not strict >)', () => {
    // 100 → 102 (Up, +2). Then 102 → 100 — reverse by exactly 2 = threshold.
    const r = localCompute([100, 102, 100], 2, 'absolute');
    expect(r.length).toBeGreaterThan(0);
});

test('local: source_index tracks the bar at which each line started', () => {
    const closes = [100, 102, 104, 106, 100];
    const r = localCompute(closes, 2, 'absolute');
    expect(r[0].source_index).toBe(0);
    if (r.length > 1) expect(r[1].source_index).toBeGreaterThan(r[0].source_index);
});

test('local: anchor_price = end_price of the previous line (continuity)', () => {
    const closes = [100, 110, 100, 110];
    const r = localCompute(closes, 5, 'absolute');
    for (let i = 1; i < r.length; i++) {
        expect(r[i].anchor_price).toBeCloseTo(r[i - 1].end_price, 9);
    }
});

test('local: final line is the in-progress trend (always emitted if direction set)', () => {
    const closes = [100, 105, 110]; // pure uptrend, never reverses
    const r = localCompute(closes, 1, 'absolute');
    expect(r.length).toBe(1);
    expect(r[0].direction).toBe('Up');
});

// ── classifyYangYin ──────────────────────────────────────────────

test('classifyYangYin: Up crossing prior peak = yang; Down crossing prior trough = yin', () => {
    const lines = [
        { direction: 'Up', anchor_price: 100, end_price: 110, source_index: 0 },
        { direction: 'Down', anchor_price: 110, end_price: 95, source_index: 5 },
        { direction: 'Up', anchor_price: 95, end_price: 120, source_index: 10 },  // crosses 110 → yang
        { direction: 'Down', anchor_price: 120, end_price: 80, source_index: 15 }, // crosses 95 → yin
    ];
    expect(classifyYangYin(lines)).toEqual(['yang', 'yin', 'yang', 'yin']);
});

test('classifyYangYin: empty array → empty', () => {
    expect(classifyYangYin([])).toEqual([]);
});

// ── summarize ────────────────────────────────────────────────────

test('summarize: count/ups/downs/avg + last_dir', () => {
    const lines = [
        { direction: 'Up',   anchor_price: 100, end_price: 110, source_index: 0 },
        { direction: 'Down', anchor_price: 110, end_price: 95,  source_index: 5 },
    ];
    const s = summarize(lines);
    expect(s.count).toBe(2);
    expect(s.ups).toBe(1);
    expect(s.downs).toBe(1);
    expect(s.avg_up).toBeCloseTo(10, 9);
    expect(s.avg_down).toBeCloseTo(15, 9);
    expect(s.last_dir).toBe('Down');
});

test('summarize: empty → count 0, NaN aggregates', () => {
    const s = summarize([]);
    expect(s.count).toBe(0);
    expect(Number.isNaN(s.avg_up)).toBe(true);
    expect(s.last_dir).toBeNull();
});

// ── trendBadge ────────────────────────────────────────────────────

test('trendBadge: last direction wins', () => {
    expect(trendBadge([]).key).toMatch(/flat/);
    expect(trendBadge([{ direction: 'Up',   anchor_price: 100, end_price: 110, source_index: 0 }]).key).toMatch(/uptrend/);
    expect(trendBadge([
        { direction: 'Up',   anchor_price: 100, end_price: 110, source_index: 0 },
        { direction: 'Down', anchor_price: 110, end_price: 95,  source_index: 5 },
    ]).key).toMatch(/downtrend/);
});

// ── linesToPolyline ──────────────────────────────────────────────

test('linesToPolyline: 2 points per line (anchor + end at same x)', () => {
    const lines = [
        { direction: 'Up',   anchor_price: 100, end_price: 110, source_index: 0 },
        { direction: 'Down', anchor_price: 110, end_price: 95,  source_index: 5 },
    ];
    const { xs, ys } = linesToPolyline(lines);
    expect(xs.length).toBe(4);
    expect(ys).toEqual([100, 110, 110, 95]);
});

test('linesToPolyline: empty → empty arrays', () => {
    const r = linesToPolyline([]);
    expect(r.xs).toEqual([]);
    expect(r.ys).toEqual([]);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + produces non-error compute', () => {
    for (const k of ['uptrend','downtrend','choppy','breakout','flat','pct-reversal','reversal-storm','gentle-bull']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.closes, inp.reversal, inp.kind);
        expect(Array.isArray(r)).toBe(true);
    }
});

test('demo uptrend: produces exactly one Up line', () => {
    const inp = makeDemoInput('uptrend');
    const r = localCompute(inp.closes, inp.reversal, inp.kind);
    expect(r.length).toBe(1);
    expect(r[0].direction).toBe('Up');
});

test('demo downtrend: last line is Down', () => {
    const inp = makeDemoInput('downtrend');
    const r = localCompute(inp.closes, inp.reversal, inp.kind);
    expect(r[r.length - 1].direction).toBe('Down');
});

test('demo flat: produces no lines', () => {
    const inp = makeDemoInput('flat');
    const r = localCompute(inp.closes, inp.reversal, inp.kind);
    expect(r.length).toBe(0);
});

test('demo reversal-storm: produces ≥ 5 alternating lines', () => {
    const inp = makeDemoInput('reversal-storm');
    const r = localCompute(inp.closes, inp.reversal, inp.kind);
    expect(r.length).toBeGreaterThanOrEqual(5);
    for (let i = 1; i < r.length; i++) {
        expect(r[i].direction).not.toBe(r[i - 1].direction);
    }
});

test('demo breakout: first line begins inside flat region, then trends', () => {
    const inp = makeDemoInput('breakout');
    const r = localCompute(inp.closes, inp.reversal, inp.kind);
    expect(r.length).toBeGreaterThan(0);
    expect(r[r.length - 1].direction).toBe('Up');
});

// ── label keys + formatters ──────────────────────────────────────

test('dirLabelKey + yangYinLabelKey: return i18n keys', () => {
    expect(dirLabelKey('Up')).toBe('view.kagi.dir.up');
    expect(dirLabelKey('Down')).toBe('view.kagi.dir.down');
    expect(dirLabelKey()).toBe('view.kagi.dir.unknown');
    expect(yangYinLabelKey('yang')).toBe('view.kagi.yy.yang');
    expect(yangYinLabelKey('yin')).toBe('view.kagi.yy.yin');
    expect(yangYinLabelKey()).toBe('view.kagi.yy.neutral');
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtUSD(100)).toBe('$100.00');
    expect(fmtMove(2.5)).toBe('+$2.50');
    expect(fmtMove(-2.5)).toBe('-$2.50');
    expect(fmtInt(42.7)).toBe('42');
    expect(fmtPct(0.5)).toBe('0.50%');
    expect(fmtUSD(NaN)).toBe('—');
});
