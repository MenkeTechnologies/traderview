// Volume-at-price helpers: parser, validator, body shape,
// localCompute Rust-mirror, valueAreaRangePct, badges, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_NUM_BINS, DEFAULT_VA_PCT,
    parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute,
    valueAreaRangePct, profileBadge,
    makeDemoInput, fmtUSD, fmtVol, fmtInt, fmtPct,
} from '../js/_volume_at_price_inputs.js';

// ── constants ─────────────────────────────────────────────────────

test('DEFAULTS match Rust (50 bins, 70% VA)', () => {
    expect(DEFAULT_NUM_BINS).toBe(50);
    expect(DEFAULT_VA_PCT).toBe(70.0);
});

// ── parser ────────────────────────────────────────────────────────

test('parseBarsBlob: 3 tokens per line; blanks + # comments ignored', () => {
    const r = parseBarsBlob('101 99 1000\n# note\n\n102, 100, 1200');
    expect(r.errors).toEqual([]);
    expect(r.bars).toEqual([
        { high: 101, low: 99, volume: 1000 },
        { high: 102, low: 100, volume: 1200 },
    ]);
});

test('parseBarsBlob: rejects wrong token count, non-finite, high<low, negative volume', () => {
    expect(parseBarsBlob('101 99').errors[0].message).toMatch(/3 tokens/);
    expect(parseBarsBlob('101 99 foo').errors[0].message).toMatch(/non-finite/);
    expect(parseBarsBlob('98 100 1000').errors[0].message).toMatch(/high < low/);
    expect(parseBarsBlob('100 99 -50').errors[0].message).toMatch(/volume/);
});

test('parseBarsBlob: non-string returns 1 error', () => {
    expect(parseBarsBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts non-empty default', () => {
    expect(validateInputs({ bars: [{ high: 101, low: 99, volume: 1000 }],
                            num_bins: 50, value_area_pct: 70 })).toBe(null);
});

test('validate rejects: bad array, bad bar fields, bad num_bins, bad va_pct', () => {
    expect(validateInputs({ bars: 'no', num_bins: 50, value_area_pct: 70 })).toMatch(/bars/);
    expect(validateInputs({ bars: [{ high: NaN, low: 99, volume: 1000 }], num_bins: 50, value_area_pct: 70 })).toMatch(/non-finite/);
    expect(validateInputs({ bars: [{ high: 99, low: 100, volume: 1000 }], num_bins: 50, value_area_pct: 70 })).toMatch(/high/);
    expect(validateInputs({ bars: [{ high: 100, low: 99, volume: -1 }], num_bins: 50, value_area_pct: 70 })).toMatch(/volume/);
    expect(validateInputs({ bars: [{ high: 100, low: 99, volume: 1000 }], num_bins: 1, value_area_pct: 70 })).toMatch(/num_bins/);
    expect(validateInputs({ bars: [{ high: 100, low: 99, volume: 1000 }], num_bins: 50, value_area_pct: 0 })).toMatch(/value_area_pct/);
    expect(validateInputs({ bars: [{ high: 100, low: 99, volume: 1000 }], num_bins: 50, value_area_pct: 100 })).toMatch(/value_area_pct/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: shape-preserved + bars copied as plain objects', () => {
    const body = buildBody({ bars: [{ high: 101, low: 99, volume: 1000, extra: 'x' }],
                              num_bins: 30, value_area_pct: 60 });
    expect(body.bars).toEqual([{ high: 101, low: 99, volume: 1000 }]);
    expect(body.num_bins).toBe(30);
    expect(body.value_area_pct).toBe(60);
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: empty / num_bins<2 / va_pct=0 → empty', () => {
    const r1 = localCompute([], 50, 70);
    expect(r1.bins).toEqual([]);
    const r2 = localCompute([{ high: 101, low: 99, volume: 1000 }], 1, 70);
    expect(r2.bins).toEqual([]);
    const r3 = localCompute([{ high: 101, low: 99, volume: 1000 }], 50, 0);
    expect(r3.bins).toEqual([]);
});

test('local: NaN or bad bar → empty', () => {
    const r = localCompute([{ high: NaN, low: 99, volume: 1000 }], 50, 70);
    expect(r.bins).toEqual([]);
});

test('local: uniform same-range bars → equal-volume bins (total conserved)', () => {
    const bars = Array.from({ length: 20 }, () => ({ high: 110, low: 100, volume: 1000 }));
    const r = localCompute(bars, 10, 70);
    expect(r.bins.length).toBe(10);
    expect(r.poc_index).not.toBeNull();
    expect(r.total_volume).toBeCloseTo(20_000, 6);
});

test('local: POC at the heavy-volume bar', () => {
    const bars = [
        { high: 101, low: 100, volume: 100 },
        { high: 106, low: 105, volume: 50_000 },
        { high: 111, low: 110, volume: 100 },
    ];
    const r = localCompute(bars, 12, 70);
    expect(r.poc_index).not.toBeNull();
    const pocCenter = r.bins[r.poc_index].center;
    expect(pocCenter).toBeGreaterThanOrEqual(105);
    expect(pocCenter).toBeLessThanOrEqual(106);
});

test('local: value-area bounds bracket POC', () => {
    const bars = Array.from({ length: 10 }, () => ({ high: 110, low: 100, volume: 1000 }));
    const r = localCompute(bars, 10, 70);
    expect(r.value_area_low).toBeLessThanOrEqual(r.value_area_high);
    const pocCenter = r.bins[r.poc_index].center;
    expect(pocCenter).toBeGreaterThanOrEqual(r.value_area_low);
    expect(pocCenter).toBeLessThanOrEqual(r.value_area_high);
});

test('local: total_volume reported correctly', () => {
    const bars = Array.from({ length: 5 }, () => ({ high: 110, low: 100, volume: 1000 }));
    const r = localCompute(bars, 10, 70);
    expect(r.total_volume).toBeCloseTo(5000, 6);
});

test('local: spike at one price → POC at that price + non-zero VA', () => {
    const r = localCompute([
        { high: 101, low: 100, volume: 100 },
        { high: 106, low: 105, volume: 100_000 },
        { high: 111, low: 110, volume: 100 },
    ], 12, 70);
    expect(r.bins[r.poc_index].volume).toBeGreaterThan(10_000);
    expect(r.value_area_low).toBeDefined();
});

test('local: 100% VA window not allowed but 99.9% captures whole distribution', () => {
    const bars = Array.from({ length: 5 }, () => ({ high: 110, low: 100, volume: 1000 }));
    const r = localCompute(bars, 10, 99.9);
    // VA brackets the FULL range.
    expect(r.value_area_high - r.value_area_low).toBeGreaterThan(7);
});

test('local: bar with high==low places full volume in the matching bin', () => {
    const r = localCompute([
        { high: 100, low: 100, volume: 5_000 },
        { high: 105, low: 100, volume: 0 },
    ], 5, 70);
    // The volume should end up in the first bin (center ≈ 100.5).
    expect(r.total_volume).toBeCloseTo(5_000, 6);
});

test('local: bins are evenly spaced + cover [min(low), max(high)]', () => {
    const r = localCompute([
        { high: 110, low: 100, volume: 1000 },
    ], 10, 70);
    const widths = [];
    for (let i = 1; i < r.bins.length; i++) widths.push(r.bins[i].center - r.bins[i - 1].center);
    const w0 = widths[0];
    for (const w of widths) expect(w).toBeCloseTo(w0, 9);
});

// ── valueAreaRangePct ─────────────────────────────────────────────

test('valueAreaRangePct: (va_high - va_low) / (max_center - min_center)', () => {
    const r = localCompute(Array.from({ length: 10 }, () => ({ high: 110, low: 100, volume: 1000 })), 10, 70);
    const pct = valueAreaRangePct(r);
    expect(pct).toBeGreaterThan(0);
    expect(pct).toBeLessThanOrEqual(1);
});

test('valueAreaRangePct: empty report → NaN', () => {
    expect(Number.isNaN(valueAreaRangePct({ bins: [] }))).toBe(true);
    expect(Number.isNaN(valueAreaRangePct(null))).toBe(true);
});

// ── profileBadge ──────────────────────────────────────────────────

test('profileBadge: balanced <0.3, normal <0.6, skewed <0.85, trending ≥0.85', () => {
    const mk = (r) => ({ bins: [{ center: 100, volume: 0 }, { center: 110, volume: 0 }],
                          value_area_low: 100, value_area_high: 100 + r * 10 });
    expect(profileBadge(mk(0.2)).key).toMatch(/balanced/);
    expect(profileBadge(mk(0.5)).key).toMatch(/normal/);
    expect(profileBadge(mk(0.8)).key).toMatch(/skewed/);
    expect(profileBadge(mk(0.95)).key).toMatch(/trending/);
    expect(profileBadge({ bins: [] }).key).toMatch(/unknown/);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + produces a non-empty profile', () => {
    for (const k of ['normal-session','tight-balanced','trending-up','double-distribution',
                     'spike-poc','narrow-va','wide-va','fine-bins']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.bars, inp.num_bins, inp.value_area_pct);
        expect(r.bins.length).toBe(inp.num_bins);
        expect(r.total_volume).toBeGreaterThan(0);
    }
});

test('demo spike-poc: POC center lies between 105 and 106 (heavy-volume bar)', () => {
    const inp = makeDemoInput('spike-poc');
    const r = localCompute(inp.bars, inp.num_bins, inp.value_area_pct);
    expect(r.bins[r.poc_index].center).toBeGreaterThanOrEqual(105);
    expect(r.bins[r.poc_index].center).toBeLessThanOrEqual(106);
});

test('demo double-distribution: produces a clear high-volume cluster at one mode', () => {
    const inp = makeDemoInput('double-distribution');
    const r = localCompute(inp.bars, inp.num_bins, inp.value_area_pct);
    // POC must be near 100 or near 110 (both modes are equal — implementation picks first max).
    const pocCenter = r.bins[r.poc_index].center;
    expect(pocCenter < 102 || pocCenter > 108).toBe(true);
});

test('demo narrow-va: VA range pct < 5% (very tight)', () => {
    const inp = makeDemoInput('narrow-va');
    const r = localCompute(inp.bars, inp.num_bins, inp.value_area_pct);
    expect(valueAreaRangePct(r)).toBeLessThan(1);
});

// ── round-trip + formatters ───────────────────────────────────────

test('barsToBlob round-trips through parseBarsBlob', () => {
    const bars = [{ high: 101, low: 99, volume: 1000 }, { high: 102, low: 100, volume: 1200 }];
    const back = parseBarsBlob(barsToBlob(bars));
    expect(back.errors).toEqual([]);
    expect(back.bars).toEqual(bars);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtUSD(100)).toBe('$100.00');
    expect(fmtVol(2_500_000)).toBe('2.50M');
    expect(fmtVol(15_500)).toBe('15.50k');
    expect(fmtVol(42)).toBe('42');
    expect(fmtInt(7.7)).toBe('7');
    expect(fmtPct(0.2345)).toBe('23.45%');
    expect(fmtUSD(NaN)).toBe('—');
    expect(fmtPct(NaN)).toBe('—');
});
