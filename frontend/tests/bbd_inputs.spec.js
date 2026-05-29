// Bollinger Band Distance helpers: parser, validator, localCompute parity, badges.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, DEFAULT_PERIOD, DEFAULT_N_STDEV, MIN_PERIOD, MAX_PERIOD,
    parseClosesBlob, closesToBlob, validateInputs, buildBody, localCompute,
    positionBadge, trendBadge, kissBadge, summarizeCloses,
    makeDemoInput,
    fmtPrice, fmtNum, fmtPct, fmtInt,
} from '../js/_bbd_inputs.js';

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
    expect(validateInputs({ closes: new Array(30).fill(100), period: 20, n_stdev: 2 })).toBe(null);
});

test('validate rejects: bad array / bad period / bad n_stdev / short / NaN', () => {
    const base = { closes: new Array(30).fill(100), period: 20, n_stdev: 2 };
    expect(validateInputs({ ...base, closes: 'no' })).toMatch(/closes/);
    expect(validateInputs({ ...base, period: 1 })).toMatch(/period/);
    expect(validateInputs({ ...base, period: 9999 })).toMatch(/period/);
    expect(validateInputs({ ...base, n_stdev: 0 })).toMatch(/n_stdev/);
    expect(validateInputs({ ...base, n_stdev: NaN })).toMatch(/n_stdev/);
    expect(validateInputs({ ...base, closes: new Array(5).fill(100) })).toMatch(/period/);
    const bad = [...new Array(30)].map((_, i) => i === 5 ? NaN : 100);
    expect(validateInputs({ ...base, closes: bad })).toMatch(/finite/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody copies + passes through', () => {
    expect(buildBody({ closes: [100, 101], period: 20, n_stdev: 2 }))
        .toEqual({ closes: [100, 101], period: 20, n_stdev: 2 });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: invalid params return all null', () => {
    const c = new Array(50).fill(100);
    expect(localCompute(c, 1, 2).every(x => x === null)).toBe(true);
    expect(localCompute(c, 20, 0).every(x => x === null)).toBe(true);
});

test('local: NaN returns all null', () => {
    const c = new Array(50).fill(100);
    c[5] = NaN;
    expect(localCompute(c, 20, 2).every(x => x === null)).toBe(true);
});

test('local: flat market yields BBD = 0', () => {
    const c = new Array(50).fill(100);
    const r = localCompute(c, 20, 2);
    for (let i = 19; i < 50; i++) expect(Math.abs(r[i])).toBeLessThan(1e-9);
});

test('local: close at midline of non-degenerate window yields BBD ≈ 0.5', () => {
    const c = [];
    for (let i = 0; i < 19; i++) c.push(i % 2 === 0 ? 99 : 101);
    c.push(100);
    const r = localCompute(c, 20, 2);
    expect(Math.abs(r[19] - 0.5)).toBeLessThan(0.05);
});

test('local: close far beyond upper band yields BBD finite and > 0.5', () => {
    const c = [];
    for (let i = 0; i < 19; i++) c.push(i % 2 === 0 ? 99 : 101);
    c.push(500);   // extreme spike, far above upper band
    const r = localCompute(c, 20, 2);
    expect(Number.isFinite(r[19])).toBe(true);
    expect(r[19]).toBeGreaterThan(0.5);
});

test('local: output length matches input', () => {
    const c = new Array(50).fill(100);
    expect(localCompute(c, 20, 2).length).toBe(50);
});

test('local: leading nulls until period', () => {
    const c = new Array(50).fill(100);
    const r = localCompute(c, 20, 2);
    for (let i = 0; i < 19; i++) expect(r[i]).toBe(null);
    expect(r[19]).not.toBe(null);
});

test('local: deterministic', () => {
    const c = Array.from({ length: 40 }, (_, i) => 100 + Math.sin(i * 0.2));
    expect(localCompute(c, 20, 2)).toEqual(localCompute(c, 20, 2));
});

test('local: BBD never negative on populated bars', () => {
    const c = Array.from({ length: 60 }, (_, i) => 100 + Math.sin(i * 0.3) * 5);
    const r = localCompute(c, 20, 2);
    for (const v of r) if (v != null) expect(v).toBeGreaterThanOrEqual(0);
});

// ── badges ────────────────────────────────────────────────────────

test('positionBadge: tiers', () => {
    expect(positionBadge(0.7).key).toMatch(/outside_band/);
    expect(positionBadge(0.48).key).toMatch(/midline/);
    expect(positionBadge(0.35).key).toMatch(/mid_zone/);
    expect(positionBadge(0.20).key).toMatch(/toward_band/);
    expect(positionBadge(0.10).key).toMatch(/near_band/);
    expect(positionBadge(0.02).key).toMatch(/at_band/);
    expect(positionBadge(null).key).toMatch(/unknown/);
});

test('trendBadge: tiers', () => {
    expect(trendBadge([0.5, 0.5, 0.5, 0.5, 0.5]).key).toMatch(/flat/);
    expect(trendBadge([0.05, 0.1, 0.2, 0.3, 0.5]).key).toMatch(/toward_midline_fast/);
    expect(trendBadge([0.5, 0.4, 0.3, 0.2, 0.05]).key).toMatch(/toward_band_fast/);
    expect(trendBadge([]).key).toMatch(/unknown/);
});

test('kissBadge: tiers', () => {
    expect(kissBadge([0.5, 0.5, 0.5, 0.5, 0]).key).toMatch(/touched/);
    expect(kissBadge([0.5, 0.5, 0.5, 0.5, 0.03]).key).toMatch(/kissed_band/);
    expect(kissBadge([0.5, 0.5, 0.5, 0.5, 0.1]).key).toMatch(/approached/);
    expect(kissBadge([0.5, 0.5, 0.5, 0.5, 0.4]).key).toMatch(/no_visit/);
    expect(kissBadge([]).key).toMatch(/unknown/);
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
    for (const k of ['oscillating','midline-walk','band-walking','breakout-up',
                     'breakdown','wide-bands','tight-bands','flat']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.closes, inp.period, inp.n_stdev);
        expect(r.length).toBe(inp.closes.length);
    }
});

test('demo flat: BBD = 0 throughout populated', () => {
    const inp = makeDemoInput('flat');
    const r = localCompute(inp.closes, inp.period, inp.n_stdev);
    for (let i = inp.period - 1; i < inp.closes.length; i++) {
        expect(Math.abs(r[i])).toBeLessThan(1e-9);
    }
});

test('demo band-walking: last BBD small (close near upper band)', () => {
    const inp = makeDemoInput('band-walking');
    const r = localCompute(inp.closes, inp.period, inp.n_stdev);
    expect(r[r.length - 1]).toBeLessThan(0.3);
});

test('demo breakout-up: last BBD > 0 and possibly > 0.5', () => {
    const inp = makeDemoInput('breakout-up');
    const r = localCompute(inp.closes, inp.period, inp.n_stdev);
    expect(r[r.length - 1]).toBeGreaterThan(0);
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
    expect(fmtNum(0.4321)).toBe('0.4321');
    expect(fmtPct(0.0125)).toBe('1.25%');
    expect(fmtInt(42.9)).toBe('42');
    expect(fmtPrice(NaN)).toBe('—');
});

test('DEFAULTS sanity', () => {
    expect(DEFAULT_INPUTS.closes).toEqual([]);
    expect(DEFAULT_INPUTS.period).toBe(DEFAULT_PERIOD);
    expect(DEFAULT_INPUTS.n_stdev).toBe(DEFAULT_N_STDEV);
    expect(DEFAULT_PERIOD).toBe(20);
    expect(DEFAULT_N_STDEV).toBe(2);
    expect(MIN_PERIOD).toBe(2);
    expect(MAX_PERIOD).toBe(500);
});
