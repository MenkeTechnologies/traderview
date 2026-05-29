// Centered Smoothed Momentum helpers: parser, validator, localCompute parity (SuperSmoother), badges.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, DEFAULT_MOMENTUM, DEFAULT_SMOOTH,
    MIN_MOMENTUM, MAX_MOMENTUM, MIN_SMOOTH, MAX_SMOOTH,
    parseClosesBlob, closesToBlob, validateInputs, buildBody, localCompute,
    momentumBadge, trendBadge, crossBadge, summarizeCloses,
    makeDemoInput,
    fmtNum, fmtNumSigned, fmtPrice, fmtInt,
} from '../js/_csm_inputs.js';

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
    expect(validateInputs({ closes: new Array(20).fill(100), momentum_period: 10, smooth_period: 8 })).toBe(null);
});

test('validate rejects: bad arrays / bad momentum / bad smooth / short / NaN', () => {
    const base = { closes: new Array(20).fill(100), momentum_period: 10, smooth_period: 8 };
    expect(validateInputs({ ...base, closes: 'no' })).toMatch(/closes/);
    expect(validateInputs({ ...base, momentum_period: 0 })).toMatch(/momentum_period/);
    expect(validateInputs({ ...base, momentum_period: 9999 })).toMatch(/momentum_period/);
    expect(validateInputs({ ...base, smooth_period: 3 })).toMatch(/smooth_period/);
    expect(validateInputs({ ...base, smooth_period: 9999 })).toMatch(/smooth_period/);
    expect(validateInputs({ ...base, closes: new Array(5).fill(100) })).toMatch(/momentum_period \+ 3/);
    const bad = [...new Array(20)].map((_, i) => i === 5 ? NaN : 100);
    expect(validateInputs({ ...base, closes: bad })).toMatch(/finite/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody copies + passes through', () => {
    expect(buildBody({ closes: [100, 101], momentum_period: 10, smooth_period: 8 }))
        .toEqual({ closes: [100, 101], momentum_period: 10, smooth_period: 8 });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: invalid inputs return all null', () => {
    const c = new Array(50).fill(100);
    expect(localCompute(c, 0, 8).every(x => x === null)).toBe(true);
    expect(localCompute(c, 10, 3).every(x => x === null)).toBe(true);
    expect(localCompute(c.slice(0, 5), 10, 8).every(x => x === null)).toBe(true);
});

test('local: NaN returns all null', () => {
    const c = new Array(50).fill(100);
    c[5] = NaN;
    expect(localCompute(c, 10, 8).every(x => x === null)).toBe(true);
});

test('local: flat market yields CSM ≈ 0', () => {
    const c = new Array(100).fill(100);
    const r = localCompute(c, 10, 8);
    for (let i = 30; i < 100; i++) expect(Math.abs(r[i])).toBeLessThan(1e-6);
});

test('local: uptrend CSM converges near +10 (period)', () => {
    const c = Array.from({ length: 100 }, (_, i) => 100 + i);
    const r = localCompute(c, 10, 8);
    expect(r[99]).toBeGreaterThan(8);
    expect(r[99]).toBeLessThan(12);
});

test('local: downtrend CSM converges near -10', () => {
    const c = Array.from({ length: 100 }, (_, i) => 200 - i);
    const r = localCompute(c, 10, 8);
    expect(r[99]).toBeLessThan(-8);
    expect(r[99]).toBeGreaterThan(-12);
});

test('local: output length matches input', () => {
    const c = new Array(50).fill(100);
    expect(localCompute(c, 10, 8).length).toBe(50);
});

test('local: leading nulls until momentum + 2', () => {
    const c = new Array(50).fill(100);
    const r = localCompute(c, 10, 8);
    // SuperSmoother loop starts at momentum_period + 2 = 12
    for (let i = 0; i < 12; i++) expect(r[i]).toBe(null);
    expect(r[12]).not.toBe(null);
});

test('local: deterministic', () => {
    const c = Array.from({ length: 30 }, (_, i) => 100 + Math.sin(i * 0.2));
    expect(localCompute(c, 10, 8)).toEqual(localCompute(c, 10, 8));
});

// ── badges ────────────────────────────────────────────────────────

test('momentumBadge: 5 tiers', () => {
    expect(momentumBadge(15).key).toMatch(/strong_up/);
    expect(momentumBadge(5).key).toMatch(/up/);
    expect(momentumBadge(0).key).toMatch(/neutral/);
    expect(momentumBadge(-5).key).toMatch(/down/);
    expect(momentumBadge(-15).key).toMatch(/strong_down/);
    expect(momentumBadge(null).key).toMatch(/unknown/);
});

test('trendBadge: tiers', () => {
    expect(trendBadge([0, 0, 0, 0, 0]).key).toMatch(/flat/);
    expect(trendBadge([-5, -3, 0, 3, 10]).key).toMatch(/rising_fast/);
    expect(trendBadge([10, 5, 0, -3, -10]).key).toMatch(/falling_fast/);
    expect(trendBadge([]).key).toMatch(/unknown/);
});

test('crossBadge: up / down / none', () => {
    expect(crossBadge([null, -5, -3, 2, 4]).key).toMatch(/up_recent/);
    expect(crossBadge([null, 5, 3, -2, -4]).key).toMatch(/down_recent/);
    expect(crossBadge([2, 3, 4, 5]).key).toMatch(/none/);
});

test('crossBadge: barsAgo populated', () => {
    const r = crossBadge([null, -5, -3, 2, 4, 5]);
    expect(r.barsAgo).toBe(2);
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
    for (const k of ['uptrend','downtrend','sideways','reversal-up',
                     'reversal-down','oscillating','short-smooth','long-momentum']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.closes, inp.momentum_period, inp.smooth_period);
        expect(r.length).toBe(inp.closes.length);
    }
});

test('demo uptrend: last CSM > 0', () => {
    const inp = makeDemoInput('uptrend');
    const r = localCompute(inp.closes, inp.momentum_period, inp.smooth_period);
    expect(r[r.length - 1]).toBeGreaterThan(0);
});

test('demo downtrend: last CSM < 0', () => {
    const inp = makeDemoInput('downtrend');
    const r = localCompute(inp.closes, inp.momentum_period, inp.smooth_period);
    expect(r[r.length - 1]).toBeLessThan(0);
});

test('demo long-momentum uses momentum_period=25', () => {
    const inp = makeDemoInput('long-momentum');
    expect(inp.momentum_period).toBe(25);
});

// ── formatters ────────────────────────────────────────────────────

test('closesToBlob round-trips', () => {
    const c = [100, 100.5, 101.25];
    const back = parseClosesBlob(closesToBlob(c));
    expect(back.errors).toEqual([]);
    expect(back.closes).toEqual(c);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtNum(1.23456)).toBe('1.2346');
    expect(fmtNumSigned(1.5)).toBe('+1.5000');
    expect(fmtNumSigned(-1.5)).toBe('-1.5000');
    expect(fmtPrice(100.456)).toBe('100.46');
    expect(fmtInt(42.9)).toBe('42');
    expect(fmtNum(NaN)).toBe('—');
});

test('DEFAULTS sanity', () => {
    expect(DEFAULT_INPUTS.closes).toEqual([]);
    expect(DEFAULT_INPUTS.momentum_period).toBe(DEFAULT_MOMENTUM);
    expect(DEFAULT_INPUTS.smooth_period).toBe(DEFAULT_SMOOTH);
    expect(DEFAULT_MOMENTUM).toBe(10);
    expect(DEFAULT_SMOOTH).toBe(8);
    expect(MIN_MOMENTUM).toBe(1);
    expect(MAX_MOMENTUM).toBe(500);
    expect(MIN_SMOOTH).toBe(4);
    expect(MAX_SMOOTH).toBe(500);
});
