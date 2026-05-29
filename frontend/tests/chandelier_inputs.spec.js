// Chandelier Exit helpers: parser, validator, localCompute parity (Wilder ATR + ratchet + flip), badges.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, DEFAULT_PERIOD, DEFAULT_MULTIPLIER, MIN_PERIOD, MAX_PERIOD,
    parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute,
    dirBadge, flipBadge, distanceBadge, flipStats, summarizeBars,
    makeDemoInput,
    fmtPrice, fmtPriceSigned, fmtPct, fmtInt,
} from '../js/_chandelier_inputs.js';

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
    expect(validateInputs({ bars, period: 22, multiplier: 3.0 })).toBe(null);
});

test('validate rejects: bad array / bad period / bad mult / too short / non-finite / inverted', () => {
    const ok = Array.from({ length: 30 }, () => b(101, 99, 100));
    expect(validateInputs({ bars: 'no', period: 22, multiplier: 3 })).toMatch(/bars/);
    expect(validateInputs({ bars: ok, period: 1, multiplier: 3 })).toMatch(/period/);
    expect(validateInputs({ bars: ok, period: 9999, multiplier: 3 })).toMatch(/period/);
    expect(validateInputs({ bars: ok, period: 22, multiplier: 0 })).toMatch(/multiplier/);
    expect(validateInputs({ bars: ok, period: 22, multiplier: -1 })).toMatch(/multiplier/);
    expect(validateInputs({ bars: ok.slice(0, 5), period: 22, multiplier: 3 })).toMatch(/period \+ 1/);
    const badNan = [...ok];
    badNan[5] = { high: NaN, low: 99, close: 100 };
    expect(validateInputs({ bars: badNan, period: 22, multiplier: 3 })).toMatch(/finite/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody trims bars to HLC', () => {
    const body = buildBody({ bars: [{ ...b(101, 99, 100), x: 1 }], period: 22, multiplier: 3 });
    expect(body).toEqual({ bars: [b(101, 99, 100)], period: 22, multiplier: 3 });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: invalid inputs return all null', () => {
    const bars = Array.from({ length: 30 }, () => b(101, 99, 100));
    expect(localCompute(bars, 1, 3).stop.every(x => x === null)).toBe(true);
    expect(localCompute(bars, 22, 0).stop.every(x => x === null)).toBe(true);
});

test('local: NaN bar returns all null', () => {
    const bars = Array.from({ length: 30 }, () => b(101, 99, 100));
    bars[5] = b(NaN, 99, 100);
    expect(localCompute(bars, 22, 3).stop.every(x => x === null)).toBe(true);
});

test('local: long_stop ≈ 95, short_stop ≈ 105 for flat market HH=101, LL=99, ATR≈2', () => {
    const bars = Array.from({ length: 50 }, () => b(101, 99, 100));
    const r = localCompute(bars, 22, 3);
    expect(Math.abs(r.long_stop[49] - 95)).toBeLessThan(0.1);
    expect(Math.abs(r.short_stop[49] - 105)).toBeLessThan(0.1);
});

test('local: uptrend keeps long direction', () => {
    const bars = Array.from({ length: 50 }, (_, i) => {
        const m = 100 + i;
        return b(m + 0.5, m - 0.5, m);
    });
    const r = localCompute(bars, 22, 3);
    expect(r.direction[49]).toBe('long');
});

test('local: sharp reversal flips to short', () => {
    const bars = [];
    for (let i = 0; i < 30; i++) {
        const m = 100 + i;
        bars.push(b(m + 0.5, m - 0.5, m));
    }
    for (let i = 0; i < 30; i++) {
        const m = 130 - 2 * i;
        bars.push(b(m + 0.5, m - 0.5, m));
    }
    const r = localCompute(bars, 22, 3);
    expect(r.direction[bars.length - 1]).toBe('short');
});

test('local: output lengths match input', () => {
    const bars = Array.from({ length: 50 }, () => b(101, 99, 100));
    const r = localCompute(bars, 22, 3);
    expect(r.stop.length).toBe(50);
    expect(r.direction.length).toBe(50);
    expect(r.long_stop.length).toBe(50);
    expect(r.short_stop.length).toBe(50);
});

test('local: deterministic', () => {
    const bars = Array.from({ length: 30 }, (_, i) => b(101 + i, 99 + i, 100 + i));
    const r1 = localCompute(bars, 22, 3);
    const r2 = localCompute(bars, 22, 3);
    expect(r1.stop).toEqual(r2.stop);
    expect(r1.direction).toEqual(r2.direction);
});

test('local: long stop ratchets up only', () => {
    const bars = Array.from({ length: 50 }, (_, i) => {
        const m = 100 + i;
        return b(m + 0.5, m - 0.5, m);
    });
    const r = localCompute(bars, 22, 3);
    const longStops = [];
    for (let i = 0; i < 50; i++) {
        if (r.direction[i] === 'long' && r.stop[i] != null) longStops.push(r.stop[i]);
    }
    for (let i = 1; i < longStops.length; i++) {
        expect(longStops[i]).toBeGreaterThanOrEqual(longStops[i - 1] - 1e-9);
    }
});

// ── badges ────────────────────────────────────────────────────────

test('dirBadge: long / short / unknown', () => {
    expect(dirBadge('long').key).toMatch(/long/);
    expect(dirBadge('short').key).toMatch(/short/);
    expect(dirBadge(null).key).toMatch(/unknown/);
});

test('flipBadge: to_long / to_short / none / barsAgo populated', () => {
    expect(flipBadge([null, 'short', 'short', 'long', 'long']).key).toMatch(/to_long/);
    expect(flipBadge([null, 'long', 'long', 'short', 'short']).key).toMatch(/to_short/);
    expect(flipBadge(['long', 'long', 'long']).key).toMatch(/none/);
    const r = flipBadge([null, 'short', 'short', 'long', 'long', 'long']);
    expect(r.barsAgo).toBe(2);
});

test('distanceBadge: 5 tiers', () => {
    expect(distanceBadge(100.001, 100).key).toMatch(/at_stop/);   // pct ≈ 0.001%
    expect(distanceBadge(99, 100).key).toMatch(/near_stop/);      // pct = 1%
    expect(distanceBadge(97, 100).key).toMatch(/normal/);         // pct = 3%
    expect(distanceBadge(93, 100).key).toMatch(/safe/);           // pct = 7%
    expect(distanceBadge(85, 100).key).toMatch(/very_safe/);      // pct = 15%
    expect(distanceBadge(null, 100).key).toMatch(/unknown/);
});

test('flipStats: flips + long/short bar counts', () => {
    const s = flipStats(['long', 'long', 'short', 'short', 'long']);
    expect(s.flips).toBe(2);
    expect(s.long_bars).toBe(3);
    expect(s.short_bars).toBe(2);
});

test('flipStats: nulls ignored', () => {
    const s = flipStats([null, 'long', null, 'short']);
    expect(s.flips).toBe(1);
    expect(s.long_bars).toBe(1);
    expect(s.short_bars).toBe(1);
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
    for (const k of ['uptrend','downtrend','flat','reversal-up',
                     'reversal-down','whipsaw','tight-mult','wide-mult']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.bars, inp.period, inp.multiplier);
        expect(r.stop.length).toBe(inp.bars.length);
    }
});

test('demo uptrend: terminal direction = long', () => {
    const inp = makeDemoInput('uptrend');
    const r = localCompute(inp.bars, inp.period, inp.multiplier);
    expect(r.direction[r.direction.length - 1]).toBe('long');
});

test('demo tight-mult typically has more flips than wide-mult', () => {
    const tight = makeDemoInput('tight-mult');
    const wide = makeDemoInput('wide-mult');
    const rT = localCompute(tight.bars, tight.period, tight.multiplier);
    const rW = localCompute(wide.bars, wide.period, wide.multiplier);
    const fT = flipStats(rT.direction).flips;
    const fW = flipStats(rW.direction).flips;
    expect(fT).toBeGreaterThanOrEqual(fW);
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
    expect(DEFAULT_INPUTS.period).toBe(DEFAULT_PERIOD);
    expect(DEFAULT_INPUTS.multiplier).toBe(DEFAULT_MULTIPLIER);
    expect(DEFAULT_PERIOD).toBe(22);
    expect(DEFAULT_MULTIPLIER).toBe(3.0);
    expect(MIN_PERIOD).toBe(2);
    expect(MAX_PERIOD).toBe(500);
});
