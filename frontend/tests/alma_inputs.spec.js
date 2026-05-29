// ALMA helpers: parser, validator, localCompute Rust mirror, badges, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, DEFAULT_PERIOD, DEFAULT_OFFSET, DEFAULT_SIGMA,
    MIN_PERIOD, MAX_PERIOD,
    parseClosesBlob, closesToBlob, validateInputs, buildBody, localCompute,
    trendBadge, positionBadge, summarizeCloses, toPlotLine,
    makeDemoInput,
    fmtPrice, fmtPriceSigned, fmtPct, fmtInt,
} from '../js/_alma_inputs.js';

// ── parser ────────────────────────────────────────────────────────

test('parseClosesBlob: comma + whitespace + comments', () => {
    const r = parseClosesBlob('100 100.5\n# midday\n101, 102');
    expect(r.errors).toEqual([]);
    expect(r.closes).toEqual([100, 100.5, 101, 102]);
});

test('parseClosesBlob: $ prefix stripped', () => {
    const r = parseClosesBlob('$100.5 $101.0');
    expect(r.errors).toEqual([]);
    expect(r.closes).toEqual([100.5, 101.0]);
});

test('parseClosesBlob: rejects non-positive', () => {
    const r = parseClosesBlob('100 -5 0 102');
    expect(r.errors.length).toBe(2);
    expect(r.closes).toEqual([100, 102]);
});

test('parseClosesBlob: non-string returns 1 error', () => {
    expect(parseClosesBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts valid', () => {
    expect(validateInputs({ closes: new Array(20).fill(100), period: 9, offset: 0.85, sigma: 6 })).toBe(null);
});

test('validate rejects: bad array / bad period / bad offset / bad sigma / too short / NaN', () => {
    const base = { closes: new Array(20).fill(100), period: 9, offset: 0.85, sigma: 6 };
    expect(validateInputs({ ...base, closes: 'no' })).toMatch(/closes/);
    expect(validateInputs({ ...base, period: 1 })).toMatch(/period/);
    expect(validateInputs({ ...base, period: 1000 })).toMatch(/period/);
    expect(validateInputs({ ...base, offset: -0.1 })).toMatch(/offset/);
    expect(validateInputs({ ...base, offset: 1.1 })).toMatch(/offset/);
    expect(validateInputs({ ...base, sigma: 0 })).toMatch(/sigma/);
    expect(validateInputs({ ...base, sigma: -1 })).toMatch(/sigma/);
    expect(validateInputs({ ...base, closes: new Array(5).fill(100) })).toMatch(/period/);
    expect(validateInputs({ ...base, closes: [100, NaN, 102, 103, 104, 105, 106, 107, 108, 109] })).toMatch(/finite/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody passes through', () => {
    expect(buildBody({ closes: [100, 101], period: 9, offset: 0.85, sigma: 6 }))
        .toEqual({ closes: [100, 101], period: 9, offset: 0.85, sigma: 6 });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: empty returns empty', () => {
    expect(localCompute([], 9, 0.85, 6)).toEqual([]);
});

test('local: invalid params return all null', () => {
    const closes = new Array(30).fill(100);
    expect(localCompute(closes, 1, 0.85, 6).every(x => x === null)).toBe(true);
    expect(localCompute(closes, 9, -0.1, 6).every(x => x === null)).toBe(true);
    expect(localCompute(closes, 9, 1.1, 6).every(x => x === null)).toBe(true);
    expect(localCompute(closes, 9, 0.85, 0).every(x => x === null)).toBe(true);
    expect(localCompute(closes, 9, 0.85, -1).every(x => x === null)).toBe(true);
    expect(localCompute(closes, 9, NaN, 6).every(x => x === null)).toBe(true);
});

test('local: shorter than period returns all null', () => {
    expect(localCompute(new Array(5).fill(100), 9, 0.85, 6).every(x => x === null)).toBe(true);
});

test('local: flat series yields flat ALMA at the same level', () => {
    const closes = new Array(30).fill(100);
    const out = localCompute(closes, 9, 0.85, 6);
    for (let i = 8; i < 30; i++) {
        expect(Math.abs(out[i] - 100)).toBeLessThan(1e-12);
    }
});

test('local: uptrend → ALMA below current price (lag)', () => {
    const closes = Array.from({ length: 30 }, (_, i) => 100 + i);
    const out = localCompute(closes, 9, 0.85, 6);
    for (let i = 8; i < 30; i++) {
        expect(out[i]).toBeLessThan(closes[i]);
    }
});

test('local: output length matches input', () => {
    const closes = Array.from({ length: 50 }, (_, i) => 100 + Math.sin(i * 0.3) * 5);
    const out = localCompute(closes, 9, 0.85, 6);
    expect(out.length).toBe(50);
    expect(out[7]).toBe(null);
    expect(out[8]).not.toBe(null);
});

test('local: higher offset responds faster (step function)', () => {
    const closes = [...new Array(20).fill(100), ...new Array(20).fill(110)];
    const low_off  = localCompute(closes, 9, 0.10, 6);
    const high_off = localCompute(closes, 9, 0.95, 6);
    expect(high_off[25]).toBeGreaterThan(low_off[25]);
});

test('local: NaN close → all null', () => {
    const closes = [100, NaN, 101, 102, 103, 104, 105, 106, 107, 108];
    expect(localCompute(closes, 9, 0.85, 6).every(x => x === null)).toBe(true);
});

test('local: deterministic', () => {
    const closes = Array.from({ length: 40 }, (_, i) => 100 + Math.sin(i * 0.2));
    const a = localCompute(closes, 9, 0.85, 6);
    const b = localCompute(closes, 9, 0.85, 6);
    expect(a).toEqual(b);
});

test('local: offset=1.0 puts kernel peak at most recent bar', () => {
    // Heuristic: offset=1.0 → kernel weighting heavily favors recent bar.
    // For a step from 100 → 110 at index 20, offset=1.0 should be closer to 110
    // than offset=0.0 a few bars later.
    const closes = [...new Array(20).fill(100), ...new Array(20).fill(110)];
    const off0 = localCompute(closes, 9, 0.0, 6);
    const off1 = localCompute(closes, 9, 1.0, 6);
    expect(off1[25]).toBeGreaterThan(off0[25]);
});

// ── toPlotLine ────────────────────────────────────────────────────

test('toPlotLine: passes finite, maps NaN/undefined → null', () => {
    expect(toPlotLine([1, null, NaN, 2, undefined])).toEqual([1, null, null, 2, null]);
});

test('toPlotLine: non-array → empty', () => {
    expect(toPlotLine(null)).toEqual([]);
});

// ── badges ────────────────────────────────────────────────────────

test('trendBadge: up_strong / up / down_strong / down / flat / unknown', () => {
    expect(trendBadge([100, 100, 100, 100, 100]).key).toMatch(/flat/);
    expect(trendBadge([100, 101, 102, 103, 110]).key).toMatch(/up_strong/);
    expect(trendBadge([100, 100.5, 100.7, 100.8, 100.9]).key).toMatch(/up/);
    expect(trendBadge([110, 109, 108, 107, 100]).key).toMatch(/down_strong/);
    expect(trendBadge([110, 109.5, 109.3, 109.2, 109.1]).key).toMatch(/down/);
    expect(trendBadge([1, 2]).key).toMatch(/unknown/);   // 2 < lookback=5
    expect(trendBadge([]).key).toMatch(/unknown/);
});

test('positionBadge: well_above / above / well_below / below / at', () => {
    expect(positionBadge(110, 100).key).toMatch(/well_above/);
    expect(positionBadge(101, 100).key).toMatch(/above/);
    expect(positionBadge(100, 100).key).toMatch(/at/);
    expect(positionBadge(99, 100).key).toMatch(/below/);
    expect(positionBadge(85, 100).key).toMatch(/well_below/);
    expect(positionBadge(null, 100).key).toMatch(/unknown/);
    expect(positionBadge(100, 0).key).toMatch(/unknown/);
});

// ── summarize ─────────────────────────────────────────────────────

test('summarizeCloses: counts / extrema / mean / last', () => {
    const s = summarizeCloses([100, 102, 98, 105]);
    expect(s.count).toBe(4);
    expect(s.last).toBe(105);
    expect(s.min).toBe(98);
    expect(s.max).toBe(105);
    expect(s.mean).toBeCloseTo(101.25, 6);
});

test('summarizeCloses: empty → NaN', () => {
    const s = summarizeCloses([]);
    expect(s.count).toBe(0);
    expect(Number.isNaN(s.last)).toBe(true);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + computes', () => {
    for (const k of ['uptrend','downtrend','sideways','step-up',
                     'high-offset','low-offset','sharp-kernel','soft-kernel']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const out = localCompute(inp.closes, inp.period, inp.offset, inp.sigma);
        expect(out.length).toBe(inp.closes.length);
        expect(out[out.length - 1]).not.toBe(null);
    }
});

test('demo step-up: ALMA at end > 100 (responds to step)', () => {
    const inp = makeDemoInput('step-up');
    const out = localCompute(inp.closes, inp.period, inp.offset, inp.sigma);
    expect(out[out.length - 1]).toBeGreaterThan(100);
});

test('demo high-offset: very responsive in late uptrend', () => {
    const inp = makeDemoInput('high-offset');
    const out = localCompute(inp.closes, inp.period, inp.offset, inp.sigma);
    const last = out[out.length - 1];
    // last close should be close to last ALMA when offset ≈ 1 on a rising series
    const lastClose = inp.closes[inp.closes.length - 1];
    expect(Math.abs(last - lastClose) / lastClose).toBeLessThan(0.05);
});

// ── formatters ────────────────────────────────────────────────────

test('closesToBlob round-trips', () => {
    const c = [100, 100.5, 101.25];
    const back = parseClosesBlob(closesToBlob(c));
    expect(back.errors).toEqual([]);
    expect(back.closes).toEqual(c);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtPrice(100.456)).toBe('100.46');
    expect(fmtPriceSigned(1.5)).toBe('+1.50');
    expect(fmtPriceSigned(-1.5)).toBe('-1.50');
    expect(fmtPct(0.0125)).toBe('1.25%');
    expect(fmtInt(42.9)).toBe('42');
    expect(fmtPrice(NaN)).toBe('—');
    expect(fmtInt(NaN)).toBe('—');
});

test('DEFAULTS sanity', () => {
    expect(DEFAULT_INPUTS.closes).toEqual([]);
    expect(DEFAULT_INPUTS.period).toBe(DEFAULT_PERIOD);
    expect(DEFAULT_INPUTS.offset).toBe(DEFAULT_OFFSET);
    expect(DEFAULT_INPUTS.sigma).toBe(DEFAULT_SIGMA);
    expect(DEFAULT_PERIOD).toBe(9);
    expect(DEFAULT_OFFSET).toBe(0.85);
    expect(DEFAULT_SIGMA).toBe(6);
    expect(MIN_PERIOD).toBe(2);
    expect(MAX_PERIOD).toBe(500);
});
