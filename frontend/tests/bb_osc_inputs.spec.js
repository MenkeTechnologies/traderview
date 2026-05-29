// Bollinger Oscillators (combined %B + Bandwidth) helpers — parser, validator, parity, badges.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, DEFAULT_PERIOD, DEFAULT_K, MIN_PERIOD, MAX_PERIOD,
    parseClosesBlob, closesToBlob, validateInputs, buildBody, localCompute,
    pbBadge, bwBadge, pbTrendBadge, summarizeCloses,
    makeDemoInput,
    fmtPrice, fmtNum, fmtPct, fmtInt,
} from '../js/_bb_osc_inputs.js';

// ── parser ────────────────────────────────────────────────────────

test('parseClosesBlob: comma + whitespace', () => {
    const r = parseClosesBlob('100 100.5\n# noise\n101, 102');
    expect(r.errors).toEqual([]);
    expect(r.closes).toEqual([100, 100.5, 101, 102]);
});

test('parseClosesBlob: rejects non-positive', () => {
    expect(parseClosesBlob('100 -5 0 102').errors.length).toBe(2);
});

test('parseClosesBlob: non-string returns 1 error', () => {
    expect(parseClosesBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts valid', () => {
    expect(validateInputs({ closes: new Array(30).fill(100), period: 20, k: 2 })).toBe(null);
});

test('validate rejects: bad array / bad period / bad k / short / NaN', () => {
    const base = { closes: new Array(30).fill(100), period: 20, k: 2 };
    expect(validateInputs({ ...base, closes: 'no' })).toMatch(/closes/);
    expect(validateInputs({ ...base, period: 0 })).toMatch(/period/);
    expect(validateInputs({ ...base, period: 9999 })).toMatch(/period/);
    expect(validateInputs({ ...base, k: -1 })).toMatch(/k/);
    expect(validateInputs({ ...base, k: NaN })).toMatch(/k/);
    expect(validateInputs({ ...base, closes: new Array(5).fill(100) })).toMatch(/period/);
    const bad = [...new Array(30)].map((_, i) => i === 5 ? NaN : 100);
    expect(validateInputs({ ...base, closes: bad })).toMatch(/finite/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody copies + passes through', () => {
    expect(buildBody({ closes: [100, 101], period: 20, k: 2 }))
        .toEqual({ closes: [100, 101], period: 20, k: 2 });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: empty returns default (empty arrays)', () => {
    const r = localCompute([], 20, 2);
    expect(r.percent_b).toEqual([]);
    expect(r.bandwidth).toEqual([]);
    expect(r.middle).toEqual([]);
    expect(r.upper).toEqual([]);
    expect(r.lower).toEqual([]);
});

test('local: period=0 returns all null', () => {
    const r = localCompute(new Array(30).fill(100), 0, 2);
    expect(r.percent_b.every(x => x === null)).toBe(true);
});

test('local: negative k returns all null', () => {
    const r = localCompute(new Array(30).fill(100), 20, -1);
    expect(r.percent_b.every(x => x === null)).toBe(true);
});

test('local: flat series yields bandwidth = 0, %B = null (division by zero)', () => {
    const r = localCompute(new Array(30).fill(100), 20, 2);
    for (const v of r.bandwidth) {
        if (v == null) continue;
        expect(Math.abs(v)).toBeLessThan(1e-12);
    }
    for (let i = 0; i < 30; i++) {
        expect(r.percent_b[i]).toBe(null);
    }
});

test('local: middle ≤ upper, middle ≥ lower for all populated', () => {
    const c = Array.from({ length: 50 }, (_, i) => 100 + Math.cos(i * 0.07) * 5);
    const r = localCompute(c, 20, 2);
    for (let i = 0; i < 50; i++) {
        if (r.middle[i] != null && r.upper[i] != null && r.lower[i] != null) {
            expect(r.lower[i]).toBeLessThanOrEqual(r.middle[i]);
            expect(r.middle[i]).toBeLessThanOrEqual(r.upper[i]);
        }
    }
});

test('local: bandwidth positive after real movement', () => {
    const c = Array.from({ length: 50 }, (_, i) => 100 + Math.sin(i * 0.1) * 5);
    const r = localCompute(c, 20, 2);
    expect(r.bandwidth[49]).toBeGreaterThan(0);
});

test('local: %B in plausible range for oscillating input', () => {
    const c = Array.from({ length: 200 }, (_, i) => 100 + Math.sin(i * 0.07) * 5);
    const r = localCompute(c, 20, 2);
    for (const v of r.percent_b) {
        if (v == null) continue;
        expect(v).toBeGreaterThanOrEqual(-2);
        expect(v).toBeLessThanOrEqual(3);
    }
});

test('local: NaN input does not throw; output length matches', () => {
    const c = new Array(50).fill(100);
    c[25] = NaN;
    const r = localCompute(c, 20, 2);
    expect(r.percent_b.length).toBe(50);
});

test('local: higher k yields wider bands', () => {
    const c = Array.from({ length: 50 }, (_, i) => 100 + Math.sin(i * 0.07) * 5);
    const r1 = localCompute(c, 20, 1);
    const r2 = localCompute(c, 20, 2);
    expect(r2.bandwidth[49]).toBeGreaterThan(r1.bandwidth[49]);
});

test('local: deterministic', () => {
    const c = Array.from({ length: 30 }, (_, i) => 100 + Math.sin(i * 0.2));
    const a = localCompute(c, 20, 2);
    const b = localCompute(c, 20, 2);
    expect(a.percent_b).toEqual(b.percent_b);
    expect(a.bandwidth).toEqual(b.bandwidth);
});

test('local: leading nulls until period', () => {
    const c = new Array(30).fill(100);
    const r = localCompute(c, 20, 2);
    for (let i = 0; i < 19; i++) {
        expect(r.middle[i]).toBe(null);
    }
});

// ── badges ────────────────────────────────────────────────────────

test('pbBadge: 7 tiers', () => {
    expect(pbBadge(1.1).key).toMatch(/breakout/);
    expect(pbBadge(0.85).key).toMatch(/near_upper/);
    expect(pbBadge(0.65).key).toMatch(/upper_half/);
    expect(pbBadge(0.5).key).toMatch(/middle/);
    expect(pbBadge(0.3).key).toMatch(/lower_half/);
    expect(pbBadge(0.1).key).toMatch(/near_lower/);
    expect(pbBadge(-0.1).key).toMatch(/breakdown/);
    expect(pbBadge(null).key).toMatch(/unknown/);
});

test('bwBadge: tiers via percentile rank', () => {
    const arr = (n, last) => {
        const w = Array.from({ length: n }, (_, i) => i + 1);
        w[w.length - 1] = last;
        return w;
    };
    expect(bwBadge(arr(20, 0.5)).key).toMatch(/tight_squeeze/);
    expect(bwBadge(arr(20, 5)).key).toMatch(/compression|tight_squeeze/);
    expect(bwBadge(arr(20, 10)).key).toMatch(/normal/);
    expect(bwBadge(arr(20, 18)).key).toMatch(/expansion|extreme_expansion/);
    expect(bwBadge(arr(20, 1000)).key).toMatch(/extreme_expansion/);
    expect(bwBadge([]).key).toMatch(/unknown/);
    expect(bwBadge([1, 2]).key).toMatch(/unknown/);
});

test('pbTrendBadge: tiers', () => {
    expect(pbTrendBadge([0.5, 0.5, 0.5, 0.5, 0.5]).key).toMatch(/flat/);
    expect(pbTrendBadge([0.1, 0.2, 0.3, 0.4, 0.9]).key).toMatch(/rising_fast/);
    expect(pbTrendBadge([0.9, 0.8, 0.7, 0.6, 0.1]).key).toMatch(/falling_fast/);
    expect(pbTrendBadge([]).key).toMatch(/unknown/);
});

// ── summarize ─────────────────────────────────────────────────────

test('summarizeCloses: count / last / extrema / mean', () => {
    const s = summarizeCloses([100, 102, 98, 105]);
    expect(s.count).toBe(4);
    expect(s.last).toBe(105);
    expect(s.min).toBe(98);
    expect(s.max).toBe(105);
});

test('summarizeCloses: empty → NaN', () => {
    const s = summarizeCloses([]);
    expect(s.count).toBe(0);
    expect(Number.isNaN(s.last)).toBe(true);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + computes', () => {
    for (const k of ['normal-trend','ttm-squeeze','walking-upper','walking-lower',
                     'oscillating','flat','wide-bands','tight-bands']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.closes, inp.period, inp.k);
        expect(r.middle.length).toBe(inp.closes.length);
    }
});

test('demo flat: bandwidth = 0, %B = null throughout populated', () => {
    const inp = makeDemoInput('flat');
    const r = localCompute(inp.closes, inp.period, inp.k);
    for (let i = inp.period - 1; i < inp.closes.length; i++) {
        expect(Math.abs(r.bandwidth[i])).toBeLessThan(1e-9);
        expect(r.percent_b[i]).toBe(null);
    }
});

test('demo wide-bands has larger bandwidth than tight-bands', () => {
    const wide = makeDemoInput('wide-bands');
    const tight = makeDemoInput('tight-bands');
    const rW = localCompute(wide.closes, wide.period, wide.k);
    const rT = localCompute(tight.closes, tight.period, tight.k);
    expect(rW.bandwidth[rW.bandwidth.length - 1])
        .toBeGreaterThan(rT.bandwidth[rT.bandwidth.length - 1]);
});

test('demo walking-upper: last %B > 0.5', () => {
    const inp = makeDemoInput('walking-upper');
    const r = localCompute(inp.closes, inp.period, inp.k);
    expect(r.percent_b[r.percent_b.length - 1]).toBeGreaterThan(0.5);
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
    expect(fmtNum(0.7654)).toBe('0.7654');
    expect(fmtPct(0.0125)).toBe('1.25%');
    expect(fmtInt(42.9)).toBe('42');
    expect(fmtPrice(NaN)).toBe('—');
});

test('DEFAULTS sanity', () => {
    expect(DEFAULT_INPUTS.closes).toEqual([]);
    expect(DEFAULT_INPUTS.period).toBe(DEFAULT_PERIOD);
    expect(DEFAULT_INPUTS.k).toBe(DEFAULT_K);
    expect(DEFAULT_PERIOD).toBe(20);
    expect(DEFAULT_K).toBe(2);
    expect(MIN_PERIOD).toBe(1);
    expect(MAX_PERIOD).toBe(500);
});
