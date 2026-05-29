// Bollinger %B helpers: parser, validator, localCompute parity, badges.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, DEFAULT_PERIOD, DEFAULT_N_STDEV, MIN_PERIOD, MAX_PERIOD,
    parseClosesBlob, closesToBlob, validateInputs, buildBody, localCompute,
    zoneBadge, crossBadge, trendBadge, summarizeCloses,
    makeDemoInput,
    fmtPrice, fmtPb, fmtPct, fmtInt,
} from '../js/_bb_pb_inputs.js';

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

test('local: flat market yields %B = 0.5', () => {
    const c = new Array(50).fill(100);
    const r = localCompute(c, 20, 2);
    for (let i = 19; i < 50; i++) expect(Math.abs(r[i] - 0.5)).toBeLessThan(1e-9);
});

test('local: close above upper band → %B > 1', () => {
    const c = [...new Array(19).fill(100), 200];
    const r = localCompute(c, 20, 2);
    expect(r[19]).toBeGreaterThan(1);
});

test('local: close below lower band → %B < 0', () => {
    const c = [...new Array(19).fill(100), 20];
    const r = localCompute(c, 20, 2);
    expect(r[19]).toBeLessThan(0);
});

test('local: close > middle → %B > 0.5', () => {
    const c = [...new Array(19).fill(100), 110];
    const r = localCompute(c, 20, 2);
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

test('local: tighter k → %B saturates faster (more extreme values)', () => {
    const c = Array.from({ length: 40 }, (_, i) => 100 + i * 0.5);
    const tight  = localCompute(c, 20, 1);
    const normal = localCompute(c, 20, 2);
    expect(Math.abs(tight[39] - 0.5)).toBeGreaterThan(Math.abs(normal[39] - 0.5));
});

// ── badges ────────────────────────────────────────────────────────

test('zoneBadge: 7 tiers', () => {
    expect(zoneBadge(1.1).key).toMatch(/above_upper/);
    expect(zoneBadge(0.85).key).toMatch(/near_upper/);
    expect(zoneBadge(0.65).key).toMatch(/upper_half/);
    expect(zoneBadge(0.5).key).toMatch(/middle/);
    expect(zoneBadge(0.3).key).toMatch(/lower_half/);
    expect(zoneBadge(0.1).key).toMatch(/near_lower/);
    expect(zoneBadge(-0.1).key).toMatch(/below_lower/);
    expect(zoneBadge(null).key).toMatch(/unknown/);
});

test('crossBadge: breakout / breakdown / returned_below_upper / returned_above_lower / none', () => {
    // Crosses 1.0 going up
    expect(crossBadge([null, 0.9, 1.2]).key).toMatch(/breakout/);
    // Crosses 1.0 going down
    expect(crossBadge([null, 1.2, 0.7]).key).toMatch(/returned_below_upper/);
    // Crosses 0 going down
    expect(crossBadge([null, 0.1, -0.2]).key).toMatch(/breakdown/);
    // Crosses 0 going up
    expect(crossBadge([null, -0.2, 0.3]).key).toMatch(/returned_above_lower/);
    // No cross
    expect(crossBadge([0.3, 0.4, 0.5, 0.6]).key).toMatch(/none/);
});

test('crossBadge: barsAgo populated on cross', () => {
    const r = crossBadge([null, 0.5, 0.9, 1.2, 1.3, 1.4]);
    expect(r.barsAgo).toBe(2);   // breakout happened at index 3, last=5
});

test('trendBadge: tiers', () => {
    expect(trendBadge([0.5, 0.5, 0.5, 0.5, 0.5]).key).toMatch(/flat/);
    expect(trendBadge([0.1, 0.2, 0.3, 0.4, 0.9]).key).toMatch(/rising_fast/);
    expect(trendBadge([0.9, 0.8, 0.7, 0.6, 0.1]).key).toMatch(/falling_fast/);
    expect(trendBadge([]).key).toMatch(/unknown/);
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
    for (const k of ['walking-up','walking-down','oscillating','breakout-up',
                     'breakdown','mean-revert','flat','tight-bands']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.closes, inp.period, inp.n_stdev);
        expect(r.length).toBe(inp.closes.length);
    }
});

test('demo walking-up: last %B near or above 1', () => {
    const inp = makeDemoInput('walking-up');
    const r = localCompute(inp.closes, inp.period, inp.n_stdev);
    expect(r[r.length - 1]).toBeGreaterThan(0.6);
});

test('demo walking-down: last %B near or below 0', () => {
    const inp = makeDemoInput('walking-down');
    const r = localCompute(inp.closes, inp.period, inp.n_stdev);
    expect(r[r.length - 1]).toBeLessThan(0.4);
});

test('demo flat: %B = 0.5 throughout populated', () => {
    const inp = makeDemoInput('flat');
    const r = localCompute(inp.closes, inp.period, inp.n_stdev);
    for (let i = inp.period - 1; i < inp.closes.length; i++) {
        expect(Math.abs(r[i] - 0.5)).toBeLessThan(1e-9);
    }
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
    expect(fmtPb(0.7654)).toBe('0.7654');
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
