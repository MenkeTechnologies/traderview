// Volume-bar helpers: parser, validator, body shape,
// localCompute Rust-mirror, badges, summarize, demos.

import { test, expect } from 'vitest';
import {
    parsePrintsBlob, printsToBlob, validateInputs, buildBody, localCompute,
    trendBadge, coverageBadge, summarize,
    makeDemoInput,
    fmtUSD, fmtMove, fmtNum, fmtInt, fmtVol,
} from '../js/_volume_bar_inputs.js';

const p = (price, size) => ({ price, size });

// ── parser ────────────────────────────────────────────────────────

test('parsePrintsBlob: 2 tokens per line, blanks + comments ignored', () => {
    const r = parsePrintsBlob('100.05 200\n# midday\n100.06, 250');
    expect(r.errors).toEqual([]);
    expect(r.prints).toEqual([p(100.05, 200), p(100.06, 250)]);
});

test('parsePrintsBlob: rejects wrong count / bad price / bad size', () => {
    expect(parsePrintsBlob('100').errors[0].message).toMatch(/2 tokens/);
    expect(parsePrintsBlob('-1 10').errors[0].message).toMatch(/price/);
    expect(parsePrintsBlob('100 -1').errors[0].message).toMatch(/size/);
});

test('parsePrintsBlob: non-string returns 1 error', () => {
    expect(parsePrintsBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts default', () => {
    expect(validateInputs({ prints: [p(100, 10)], volume_per_bar: 1000 })).toBe(null);
});

test('validate rejects: bad array / volume_per_bar ≤ 0 / NaN', () => {
    expect(validateInputs({ prints: 'no', volume_per_bar: 1000 })).toMatch(/prints/);
    expect(validateInputs({ prints: [p(100, 10)], volume_per_bar: 0 })).toMatch(/volume_per_bar/);
    expect(validateInputs({ prints: [p(100, 10)], volume_per_bar: NaN })).toMatch(/volume_per_bar/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: strips extras', () => {
    const body = buildBody({ prints: [{ ...p(100, 10), extra: 'x' }], volume_per_bar: 500 });
    expect(body).toEqual({ prints: [p(100, 10)], volume_per_bar: 500 });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: empty / volume_per_bar=0 → empty', () => {
    expect(localCompute([], 1000)).toEqual([]);
    expect(localCompute(Array(50).fill(p(100, 10)), 0)).toEqual([]);
});

test('local: NaN price → empty', () => {
    expect(localCompute([p(NaN, 10)], 100)).toEqual([]);
});

test('local: negative size → empty', () => {
    expect(localCompute([p(100, -1)], 100)).toEqual([]);
});

test('local: 5 prints × 200 size, target 1000 → 1 bar with volume = 1000', () => {
    const prints = Array.from({ length: 5 }, (_, i) => p(100 + i, 200));
    const bars = localCompute(prints, 1000);
    expect(bars.length).toBe(1);
    expect(bars[0].volume).toBeCloseTo(1000, 9);
});

test('local: emits ≥ 5 bars from 50 prints × 100 size, target 500', () => {
    const prints = Array.from({ length: 50 }, (_, i) => p(100 + i * 0.1, 100));
    const bars = localCompute(prints, 500);
    expect(bars.length).toBeGreaterThanOrEqual(5);
});

test('local: high/low tracked over volume window', () => {
    const prints = [p(100, 200), p(110, 200), p(95, 200), p(102, 200), p(98, 200)];
    const bars = localCompute(prints, 1000);
    expect(bars.length).toBe(1);
    expect(bars[0].high).toBeCloseTo(110, 9);
    expect(bars[0].low).toBeCloseTo(95, 9);
});

test('local: trailing partial bar dropped', () => {
    // 7 prints × 200 size = 1400 → 1 full 1000-vol bar + 400 partial (dropped).
    const prints = Array.from({ length: 7 }, (_, i) => p(100 + i, 200));
    expect(localCompute(prints, 1000).length).toBe(1);
});

test('local: open=first print of bar, close=triggering print', () => {
    const prints = [p(100, 600), p(105, 500)];
    const bars = localCompute(prints, 1000);
    expect(bars.length).toBe(1);
    expect(bars[0].open).toBeCloseTo(100, 9);
    expect(bars[0].close).toBeCloseTo(105, 9);
});

test('local: new bar opens at the print AFTER the close', () => {
    // 10 prints × 200 → 2 bars of 1000 each. Bar 2 opens at print 5 = 105.
    const prints = Array.from({ length: 10 }, (_, i) => p(100 + i, 200));
    const bars = localCompute(prints, 1000);
    expect(bars.length).toBe(2);
    expect(bars[1].open).toBeCloseTo(105, 9);
});

test('local: one giant print fills the bar instantly', () => {
    const prints = [p(100, 5000), p(101, 100)];
    const bars = localCompute(prints, 1000);
    expect(bars.length).toBe(1);
    expect(bars[0].volume).toBe(5000);
    expect(bars[0].tick_count).toBe(1);
});

test('local: bar volume can exceed target (≥ check, not == ); next bar resets', () => {
    const prints = [p(100, 1200), p(105, 600), p(110, 400)];
    const bars = localCompute(prints, 1000);
    expect(bars.length).toBe(2);
    expect(bars[0].volume).toBe(1200);
    expect(bars[1].volume).toBe(1000);
});

// ── trendBadge / coverageBadge ────────────────────────────────────

test('trendBadge: last bar direction', () => {
    expect(trendBadge([]).key).toMatch(/flat/);
    expect(trendBadge([{ open: 100, close: 105 }]).key).toMatch(/uptrend/);
    expect(trendBadge([{ open: 100, close: 95 }]).key).toMatch(/downtrend/);
    expect(trendBadge([{ open: 100, close: 100 }]).key).toMatch(/flat/);
});

test('coverageBadge: low / normal / high / full', () => {
    const mk = (vols) => vols.map(v => ({ volume: v }));
    expect(coverageBadge(mk([300]),  1000, 500).key).toMatch(/low/);     // 0.3
    expect(coverageBadge(mk([700]),  1000, 500).key).toMatch(/normal/);  // 0.7
    expect(coverageBadge(mk([900]),  1000, 500).key).toMatch(/high/);    // 0.9
    expect(coverageBadge(mk([1000]), 1000, 500).key).toMatch(/full/);    // 1.0
    expect(coverageBadge([], 0, 500).key).toMatch(/unknown/);
});

// ── summarize ─────────────────────────────────────────────────────

test('summarize: counts/volume/range/last_close', () => {
    const bars = [
        { open: 100, high: 105, low: 99, close: 105, volume: 1000, tick_count: 5 },
        { open: 105, high: 106, low: 100, close: 102, volume: 1000, tick_count: 6 },
        { open: 102, high: 103, low: 101, close: 102, volume: 1000, tick_count: 4 },
    ];
    const s = summarize(bars);
    expect(s.count).toBe(3);
    expect(s.total_volume).toBe(3000);
    expect(s.total_ticks).toBe(15);
    expect(s.avg_ticks).toBeCloseTo(5, 9);
    expect(s.avg_range).toBeCloseTo((6 + 6 + 2) / 3, 9);
    expect(s.ups).toBe(1);
    expect(s.downs).toBe(1);
    expect(s.doji).toBe(1);
    expect(s.last_close).toBe(102);
});

test('summarize: empty → count 0, NaN aggregates', () => {
    const s = summarize([]);
    expect(s.count).toBe(0);
    expect(Number.isNaN(s.avg_ticks)).toBe(true);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + computes without error', () => {
    for (const k of ['uptrend-large','downtrend-large','flat-volume','spiky-volume',
                     'tiny-target','huge-target','partial-trail','noisy-walk']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const bars = localCompute(inp.prints, inp.volume_per_bar);
        expect(Array.isArray(bars)).toBe(true);
    }
});

test('demo partial-trail: emits exactly 1 bar (200 volume dropped)', () => {
    const inp = makeDemoInput('partial-trail');
    const bars = localCompute(inp.prints, inp.volume_per_bar);
    expect(bars.length).toBe(1);
});

test('demo tiny-target: many bars (every ~50 vol)', () => {
    const inp = makeDemoInput('tiny-target');
    const bars = localCompute(inp.prints, inp.volume_per_bar);
    expect(bars.length).toBeGreaterThanOrEqual(20);
});

test('demo huge-target: few bars (5000 each)', () => {
    const inp = makeDemoInput('huge-target');
    const bars = localCompute(inp.prints, inp.volume_per_bar);
    // 50 prints × ~100 = 5000 → at most 1 full bar.
    expect(bars.length).toBeLessThanOrEqual(2);
});

test('demo flat-volume: every bar has identical open/high/low/close', () => {
    const inp = makeDemoInput('flat-volume');
    const bars = localCompute(inp.prints, inp.volume_per_bar);
    expect(bars.length).toBeGreaterThan(0);
    for (const b of bars) {
        expect(b.open).toBe(b.close);
        expect(b.high).toBe(b.low);
    }
});

// ── round-trip + formatters ──────────────────────────────────────

test('printsToBlob round-trips through parsePrintsBlob', () => {
    const prints = [p(100.05, 200), p(100.06, 250)];
    const back = parsePrintsBlob(printsToBlob(prints));
    expect(back.errors).toEqual([]);
    expect(back.prints).toEqual(prints);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtUSD(100.5)).toBe('$100.50');
    expect(fmtMove(2.5)).toBe('+$2.50');
    expect(fmtMove(-2.5)).toBe('-$2.50');
    expect(fmtNum(1.234, 1)).toBe('1.2');
    expect(fmtInt(42.9)).toBe('42');
    expect(fmtVol(1_500_000)).toBe('1.50M');
    expect(fmtVol(15_500)).toBe('15.50k');
    expect(fmtVol(42)).toBe('42');
    expect(fmtUSD(NaN)).toBe('—');
});
