// BBW + %B helpers: parser, validator, localCompute parity (population σ²), badges.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, DEFAULT_PERIOD, DEFAULT_K, MIN_PERIOD, MAX_PERIOD,
    parseClosesBlob, closesToBlob, validateInputs, buildBody, localCompute,
    squeezeBadge, percentBBadge, widthTrendBadge, summarizeCloses,
    makeDemoInput,
    fmtPrice, fmtNum, fmtPct, fmtInt,
} from '../js/_bbw_inputs.js';

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
    expect(validateInputs({ ...base, period: 1 })).toMatch(/period/);
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

test('local: empty returns empty arrays', () => {
    const r = localCompute([], 20, 2);
    expect(r.middle).toEqual([]);
    expect(r.upper).toEqual([]);
    expect(r.lower).toEqual([]);
});

test('local: invalid params return all null', () => {
    const closes = new Array(30).fill(100);
    expect(localCompute(closes, 1, 2).middle.every(x => x === null)).toBe(true);
    expect(localCompute(closes, 20, -1).middle.every(x => x === null)).toBe(true);
    expect(localCompute(closes, 20, NaN).middle.every(x => x === null)).toBe(true);
});

test('local: shorter than period returns all null', () => {
    expect(localCompute(new Array(5).fill(100), 20, 2).middle.every(x => x === null)).toBe(true);
});

test('local: middle is SMA', () => {
    const closes = Array.from({ length: 30 }, (_, i) => i + 1);
    const r = localCompute(closes, 10, 2);
    // Last middle = mean(21..30) = 25.5.
    expect(Math.abs(r.middle[29] - 25.5)).toBeLessThan(1e-9);
});

test('local: upper above middle, lower below middle', () => {
    const closes = Array.from({ length: 30 }, (_, i) => i + 1);
    const r = localCompute(closes, 10, 2);
    for (let i = 9; i < 30; i++) {
        expect(r.upper[i]).toBeGreaterThan(r.middle[i]);
        expect(r.lower[i]).toBeLessThan(r.middle[i]);
    }
});

test('local: flat window → zero width, %B = 0.5', () => {
    const closes = new Array(30).fill(100);
    const r = localCompute(closes, 20, 2);
    expect(Math.abs(r.band_width[29])).toBeLessThan(1e-9);
    expect(Math.abs(r.percent_b[29] - 0.5)).toBeLessThan(1e-9);
});

test('local: %B < 0.5 when close is below middle', () => {
    // Last close 99 in window of 19 hundreds → middle ≈ 99.95 → %B < 0.5.
    const closes = [...new Array(19).fill(100), 99];
    const r = localCompute(closes, 20, 2);
    expect(r.percent_b[19]).toBeLessThan(0.5);
});

test('local: expansion increases BBW vs prior flat phase', () => {
    const closes = new Array(20).fill(100);
    for (let i = 0; i < 30; i++) closes.push(100 + Math.sin(i * 0.7) * 10);
    const r = localCompute(closes, 20, 2);
    expect(r.band_width[49]).toBeGreaterThan(r.band_width[19]);
});

test('local: output lengths match input', () => {
    const closes = Array.from({ length: 50 }, (_, i) => 100 + i * 0.1);
    const r = localCompute(closes, 20, 2);
    expect(r.middle.length).toBe(50);
    expect(r.upper.length).toBe(50);
    expect(r.lower.length).toBe(50);
    expect(r.band_width.length).toBe(50);
    expect(r.percent_b.length).toBe(50);
});

test('local: leading nulls until period', () => {
    const closes = Array.from({ length: 30 }, () => 100);
    const r = localCompute(closes, 20, 2);
    for (let i = 0; i < 19; i++) expect(r.middle[i]).toBe(null);
    expect(r.middle[19]).not.toBe(null);
});

test('local: deterministic', () => {
    const closes = Array.from({ length: 40 }, (_, i) => 100 + Math.sin(i * 0.2));
    const a = localCompute(closes, 20, 2);
    const b = localCompute(closes, 20, 2);
    expect(a.middle).toEqual(b.middle);
    expect(a.band_width).toEqual(b.band_width);
});

test('local: uses POPULATION variance (matches Rust impl — divides by period not period-1)', () => {
    // For closes [1..10] with period 10:
    //   mean = 5.5, population variance = sum((x-5.5)²)/10 = 8.25
    //   sd ≈ 2.872; upper = 5.5 + 2·2.872 ≈ 11.244; lower ≈ -0.244
    const closes = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    const r = localCompute(closes, 10, 2);
    expect(Math.abs(r.middle[9] - 5.5)).toBeLessThan(1e-9);
    expect(Math.abs(r.upper[9] - 11.244355)).toBeLessThan(0.001);
});

test('local: k=0 yields zero width', () => {
    const closes = Array.from({ length: 30 }, (_, i) => i + 1);
    const r = localCompute(closes, 20, 0);
    for (let i = 19; i < 30; i++) {
        expect(Math.abs(r.upper[i] - r.lower[i])).toBeLessThan(1e-9);
    }
});

test('local: middle = 0 case → bbw stays null', () => {
    // Construct window summing to 0 (mix of pos/neg). But closes are validated
    // positive in real use — directly call localCompute with crafted input.
    const closes = [-5, -4, -3, -2, -1, 1, 2, 3, 4, 5];
    const r = localCompute(closes, 10, 2);
    expect(r.middle[9]).toBe(0);
    expect(r.band_width[9]).toBe(null);
});

// ── badges ────────────────────────────────────────────────────────

test('squeezeBadge: tiers via percentile rank', () => {
    // Construct band_width series with last value at known percentile.
    const arr = (n, last) => {
        const w = Array.from({ length: n }, (_, i) => i + 1);
        w[w.length - 1] = last;
        return w;
    };
    expect(squeezeBadge(arr(20, 0.5)).key).toMatch(/tight/);       // last < anything → 0th pct
    expect(squeezeBadge(arr(20, 5)).key).toMatch(/narrow|tight/);  // ~20th pct
    expect(squeezeBadge(arr(20, 10)).key).toMatch(/normal/);       // ~50th pct
    expect(squeezeBadge(arr(20, 18)).key).toMatch(/expansion/);    // ~85th pct
    expect(squeezeBadge(arr(20, 1000)).key).toMatch(/extreme/);    // 100th pct
    expect(squeezeBadge([]).key).toMatch(/unknown/);
    expect(squeezeBadge([1, 2]).key).toMatch(/unknown/);            // < 5 populated
});

test('percentBBadge: 7 tiers', () => {
    expect(percentBBadge(1.1).key).toMatch(/above_upper/);
    expect(percentBBadge(0.85).key).toMatch(/near_upper/);
    expect(percentBBadge(0.65).key).toMatch(/upper_half/);
    expect(percentBBadge(0.5).key).toMatch(/middle/);
    expect(percentBBadge(0.3).key).toMatch(/lower_half/);
    expect(percentBBadge(0.1).key).toMatch(/near_lower/);
    expect(percentBBadge(-0.1).key).toMatch(/below_lower/);
    expect(percentBBadge(null).key).toMatch(/unknown/);
});

test('widthTrendBadge: tiers', () => {
    expect(widthTrendBadge([0.01, 0.01, 0.01, 0.01, 0.01]).key).toMatch(/steady/);
    expect(widthTrendBadge([0.01, 0.02, 0.03, 0.04, 0.5]).key).toMatch(/expanding/);
    expect(widthTrendBadge([0.5, 0.4, 0.3, 0.2, 0.01]).key).toMatch(/contracting/);
    expect(widthTrendBadge([]).key).toMatch(/unknown/);
});

// ── summarize ─────────────────────────────────────────────────────

test('summarizeCloses: count / last / extrema / mean', () => {
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
    for (const k of ['normal','squeeze-then-break','expansion-then-contract',
                     'trending-up','trending-down','walking-bands','wide-bands','flat-window']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.closes, inp.period, inp.k);
        expect(r.middle.length).toBe(inp.closes.length);
    }
});

test('demo squeeze-then-break: BBW expansion AFTER squeeze (last > middle)', () => {
    const inp = makeDemoInput('squeeze-then-break');
    const r = localCompute(inp.closes, inp.period, inp.k);
    // Squeeze phase ends near index 29, expansion through end.
    const populated = r.band_width.filter(v => v != null);
    expect(populated[populated.length - 1]).toBeGreaterThan(populated[0]);
});

test('demo flat-window: BBW = 0 throughout populated range', () => {
    const inp = makeDemoInput('flat-window');
    const r = localCompute(inp.closes, inp.period, inp.k);
    for (let i = inp.period - 1; i < inp.closes.length; i++) {
        expect(Math.abs(r.band_width[i])).toBeLessThan(1e-9);
        expect(Math.abs(r.percent_b[i] - 0.5)).toBeLessThan(1e-9);
    }
});

test('demo wide-bands (k=3) has wider envelope than normal (k=2) same series', () => {
    const wide = makeDemoInput('wide-bands');
    const norm = makeDemoInput('normal');
    const rW = localCompute(wide.closes, wide.period, wide.k);
    const rN = localCompute(norm.closes, norm.period, norm.k);
    const i = norm.closes.length - 1;
    expect(rW.upper[i] - rW.lower[i]).toBeGreaterThan(rN.upper[i] - rN.lower[i]);
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
    expect(fmtNum(0.12345)).toBe('0.1235');   // toFixed rounds-half-away-from-zero
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
    expect(MIN_PERIOD).toBe(2);
    expect(MAX_PERIOD).toBe(500);
});
