// Chande-Kroll Stop helpers: parser, validator, localCompute parity (Wilder ATR + 2-pass extremes), badges.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, DEFAULT_P, DEFAULT_X, DEFAULT_Q, MIN_PERIOD, MAX_PERIOD,
    parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute,
    regimeBadge, widthBadge, longTrendBadge, shortTrendBadge, summarizeBars,
    makeDemoInput,
    fmtPrice, fmtPriceSigned, fmtPct, fmtInt,
} from '../js/_cks_inputs.js';

const b = (h, l, c) => ({ high: h, low: l, close: c });

// ── parser ────────────────────────────────────────────────────────

test('parseBarsBlob: 3 tokens per line', () => {
    const r = parseBarsBlob('101 99 100\n# noise\n102 100 101');
    expect(r.errors).toEqual([]);
    expect(r.bars).toEqual([b(101, 99, 100), b(102, 100, 101)]);
});

test('parseBarsBlob: rejects wrong count / non-positive / low > high / close OOR', () => {
    expect(parseBarsBlob('100 99').errors[0].message).toMatch(/3 tokens/);
    expect(parseBarsBlob('-1 99 100').errors[0].message).toMatch(/HLC/);
    expect(parseBarsBlob('99 100 99').errors[0].message).toMatch(/low > high/);
    expect(parseBarsBlob('101 99 50').errors[0].message).toMatch(/close outside/);
});

test('parseBarsBlob: non-string returns 1 error', () => {
    expect(parseBarsBlob(undefined).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts valid', () => {
    const bars = Array.from({ length: 30 }, () => b(101, 99, 100));
    expect(validateInputs({ bars, p: 10, x: 1, q: 9 })).toBe(null);
});

test('validate rejects: bad array / bad p / bad q / bad x / too short / non-finite / inverted', () => {
    const ok = Array.from({ length: 30 }, () => b(101, 99, 100));
    expect(validateInputs({ bars: 'no', p: 10, x: 1, q: 9 })).toMatch(/bars/);
    expect(validateInputs({ bars: ok, p: 1, x: 1, q: 9 })).toMatch(/p must be/);
    expect(validateInputs({ bars: ok, p: 10, x: 1, q: 1 })).toMatch(/q must be/);
    expect(validateInputs({ bars: ok, p: 10, x: 0, q: 9 })).toMatch(/x must be/);
    expect(validateInputs({ bars: ok.slice(0, 5), p: 10, x: 1, q: 9 })).toMatch(/p \+ q/);
    const badNan = [...ok];
    badNan[5] = { high: NaN, low: 99, close: 100 };
    expect(validateInputs({ bars: badNan, p: 10, x: 1, q: 9 })).toMatch(/finite/);
    const inv = [...ok];
    inv[5] = b(99, 101, 100);
    expect(validateInputs({ bars: inv, p: 10, x: 1, q: 9 })).toMatch(/high < low/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody trims bars to HLC', () => {
    const body = buildBody({ bars: [{ ...b(101, 99, 100), x: 1 }], p: 10, x: 1, q: 9 });
    expect(body).toEqual({ bars: [b(101, 99, 100)], p: 10, x: 1, q: 9 });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: invalid inputs return all null', () => {
    const bars = Array.from({ length: 30 }, () => b(101, 99, 100));
    expect(localCompute(bars, 1, 1, 9).long_stop.every(x => x === null)).toBe(true);
    expect(localCompute(bars, 10, 0, 9).long_stop.every(x => x === null)).toBe(true);
});

test('local: NaN bar returns all null', () => {
    const bars = Array.from({ length: 30 }, () => b(101, 99, 100));
    bars[5] = b(NaN, 99, 100);
    expect(localCompute(bars, 10, 1, 9).long_stop.every(x => x === null)).toBe(true);
});

test('local: flat market long_stop < short_stop', () => {
    const bars = Array.from({ length: 50 }, () => b(101, 99, 100));
    const r = localCompute(bars, 10, 1, 9);
    expect(r.long_stop[49]).toBeLessThan(r.short_stop[49]);
});

test('local: long_stop rises in uptrend', () => {
    const bars = Array.from({ length: 50 }, (_, i) => {
        const m = 100 + i;
        return b(m + 0.5, m - 0.5, m);
    });
    const r = localCompute(bars, 10, 1, 9);
    const vals = r.long_stop.filter(v => v != null);
    for (let i = 1; i < vals.length; i++) {
        expect(vals[i]).toBeGreaterThanOrEqual(vals[i - 1] - 1e-9);
    }
});

test('local: short_stop falls in downtrend', () => {
    const bars = Array.from({ length: 50 }, (_, i) => {
        const m = 200 - i;
        return b(m + 0.5, m - 0.5, m);
    });
    const r = localCompute(bars, 10, 1, 9);
    const vals = r.short_stop.filter(v => v != null);
    for (let i = 1; i < vals.length; i++) {
        expect(vals[i]).toBeLessThanOrEqual(vals[i - 1] + 1e-9);
    }
});

test('local: higher x widens stop distance', () => {
    const bars = Array.from({ length: 50 }, (_, i) => {
        const m = 100 + Math.sin(i * 0.3) * 5;
        return b(m + 0.5, m - 0.5, m);
    });
    const r1 = localCompute(bars, 10, 1, 9);
    const r2 = localCompute(bars, 10, 3, 9);
    expect(r2.long_stop[49]).toBeLessThan(r1.long_stop[49]);
    expect(r2.short_stop[49]).toBeGreaterThan(r1.short_stop[49]);
});

test('local: output lengths match input', () => {
    const bars = Array.from({ length: 50 }, () => b(101, 99, 100));
    const r = localCompute(bars, 10, 1, 9);
    expect(r.long_stop.length).toBe(50);
    expect(r.short_stop.length).toBe(50);
});

test('local: leading nulls until p + q − 1', () => {
    const bars = Array.from({ length: 30 }, () => b(101, 99, 100));
    const r = localCompute(bars, 10, 1, 9);
    // p + q − 1 = 18; first populated at index 18.
    for (let i = 0; i < 18; i++) expect(r.long_stop[i]).toBe(null);
    expect(r.long_stop[18]).not.toBe(null);
});

test('local: deterministic', () => {
    const bars = Array.from({ length: 30 }, (_, i) => b(101 + i, 99 + i, 100 + i));
    const r1 = localCompute(bars, 10, 1, 9);
    const r2 = localCompute(bars, 10, 1, 9);
    expect(r1.long_stop).toEqual(r2.long_stop);
    expect(r1.short_stop).toEqual(r2.short_stop);
});

// ── badges ────────────────────────────────────────────────────────

test('regimeBadge: long_bias / short_bias / long_active / short_active / unknown', () => {
    // Without close → bias from stops alone.
    expect(regimeBadge(95, 105, NaN).key).toMatch(/long_bias/);
    expect(regimeBadge(105, 95, NaN).key).toMatch(/short_bias/);
    // With close → active above/below both stops.
    expect(regimeBadge(95, 105, 110).key).toMatch(/long_active/);
    expect(regimeBadge(95, 105, 90).key).toMatch(/short_active/);
    expect(regimeBadge(null, 105, 100).key).toMatch(/unknown/);
});

test('widthBadge: tiers', () => {
    expect(widthBadge(95, 105).key).toMatch(/very_wide|wide/);   // pct ≈ 10%
    expect(widthBadge(99.5, 100.5).key).toMatch(/normal/);         // pct = 0.01 → normal (≥ 0.005, < 0.02)
    expect(widthBadge(99.9, 100.1).key).toMatch(/tight/);          // pct = 0.002 → tight (< 0.005)
    expect(widthBadge(105, 95).key).toMatch(/inverted/);
    expect(widthBadge(null, 100).key).toMatch(/unknown/);
});

test('longTrendBadge / shortTrendBadge: tiers', () => {
    expect(longTrendBadge([100, 100, 100, 100, 100]).key).toMatch(/flat/);
    expect(longTrendBadge([100, 102, 104, 106, 108]).key).toMatch(/rising/);
    expect(longTrendBadge([108, 106, 104, 102, 100]).key).toMatch(/falling/);
    expect(shortTrendBadge([108, 106, 104, 102, 100]).key).toMatch(/falling/);
    expect(longTrendBadge([]).key).toMatch(/unknown/);
});

// ── summarize ─────────────────────────────────────────────────────

test('summarizeBars: count / last_close / extrema / mean', () => {
    const bars = [b(102, 98, 100), b(105, 100, 103), b(106, 101, 105)];
    const s = summarizeBars(bars);
    expect(s.count).toBe(3);
    expect(s.last_close).toBe(105);
    expect(s.min_low).toBe(98);
    expect(s.max_high).toBe(106);
});

test('summarizeBars: empty → NaN', () => {
    const s = summarizeBars([]);
    expect(s.count).toBe(0);
    expect(Number.isNaN(s.last_close)).toBe(true);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + computes', () => {
    for (const k of ['flat','uptrend','downtrend','reversal-up',
                     'reversal-down','high-x','short-bars','volatile']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.bars, inp.p, inp.x, inp.q);
        expect(r.long_stop.length).toBe(inp.bars.length);
    }
});

test('demo uptrend: terminal long_stop > demo[0] price', () => {
    const inp = makeDemoInput('uptrend');
    const r = localCompute(inp.bars, inp.p, inp.x, inp.q);
    const last = r.long_stop[r.long_stop.length - 1];
    expect(last).toBeGreaterThan(100);
});

test('demo high-x has wider stop band than short-bars', () => {
    const hi = makeDemoInput('high-x');
    const sm = makeDemoInput('short-bars');
    const rH = localCompute(hi.bars, hi.p, hi.x, hi.q);
    const rS = localCompute(sm.bars, sm.p, sm.x, sm.q);
    const widthH = (rH.short_stop[rH.short_stop.length - 1] - rH.long_stop[rH.long_stop.length - 1]);
    const widthS = (rS.short_stop[rS.short_stop.length - 1] - rS.long_stop[rS.long_stop.length - 1]);
    expect(widthH).toBeGreaterThan(widthS);
});

// ── formatters ────────────────────────────────────────────────────

test('barsToBlob round-trips', () => {
    const bars = [b(101, 99, 100), b(102, 100, 101)];
    const back = parseBarsBlob(barsToBlob(bars));
    expect(back.errors).toEqual([]);
    expect(back.bars).toEqual(bars);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtPrice(100.456)).toBe('100.46');
    expect(fmtPriceSigned(1.5)).toBe('+1.50');
    expect(fmtPriceSigned(-1.5)).toBe('-1.50');
    expect(fmtPct(0.0125)).toBe('1.25%');
    expect(fmtInt(42.9)).toBe('42');
    expect(fmtPrice(NaN)).toBe('—');
});

test('DEFAULTS sanity', () => {
    expect(DEFAULT_INPUTS.bars).toEqual([]);
    expect(DEFAULT_INPUTS.p).toBe(DEFAULT_P);
    expect(DEFAULT_INPUTS.x).toBe(DEFAULT_X);
    expect(DEFAULT_INPUTS.q).toBe(DEFAULT_Q);
    expect(DEFAULT_P).toBe(10);
    expect(DEFAULT_X).toBe(1.0);
    expect(DEFAULT_Q).toBe(9);
    expect(MIN_PERIOD).toBe(2);
    expect(MAX_PERIOD).toBe(500);
});
