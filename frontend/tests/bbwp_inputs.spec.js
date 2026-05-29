// BBWP helpers: parser, validator, localCompute parity, badges.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, DEFAULT_BB_PERIOD, DEFAULT_N_STDEV, DEFAULT_LOOKBACK,
    MIN_BB_PERIOD, MAX_BB_PERIOD, MIN_LOOKBACK, MAX_LOOKBACK,
    parseClosesBlob, closesToBlob, validateInputs, buildBody, localCompute,
    regimeBadge, trendBadge, triggerBadge, summarizeCloses,
    makeDemoInput,
    fmtPrice, fmtPct, fmtNum, fmtInt,
} from '../js/_bbwp_inputs.js';

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
    expect(validateInputs({
        closes: new Array(260).fill(100), bb_period: 20, n_stdev: 2, lookback: 252,
    })).toBe(null);
});

test('validate rejects: bad array / bad bb_period / bad n_stdev / bad lookback / lookback<bb_period / too short / NaN', () => {
    const base = { closes: new Array(260).fill(100), bb_period: 20, n_stdev: 2, lookback: 252 };
    expect(validateInputs({ ...base, closes: 'no' })).toMatch(/closes/);
    expect(validateInputs({ ...base, bb_period: 1 })).toMatch(/bb_period/);
    expect(validateInputs({ ...base, bb_period: 9999 })).toMatch(/bb_period/);
    expect(validateInputs({ ...base, n_stdev: 0 })).toMatch(/n_stdev/);
    expect(validateInputs({ ...base, n_stdev: NaN })).toMatch(/n_stdev/);
    expect(validateInputs({ ...base, lookback: 1 })).toMatch(/lookback/);
    expect(validateInputs({ ...base, lookback: 99999 })).toMatch(/lookback/);
    expect(validateInputs({ ...base, lookback: 10, bb_period: 20 })).toMatch(/lookback.*bb_period/);
    expect(validateInputs({ ...base, closes: new Array(10).fill(100) })).toMatch(/lookback/);
    const bad = [...new Array(260)].map((_, i) => i === 5 ? NaN : 100);
    expect(validateInputs({ ...base, closes: bad })).toMatch(/finite/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody copies + passes through', () => {
    expect(buildBody({ closes: [100, 101], bb_period: 20, n_stdev: 2, lookback: 252 }))
        .toEqual({ closes: [100, 101], bb_period: 20, n_stdev: 2, lookback: 252 });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: invalid params return all null', () => {
    const c = new Array(300).fill(100);
    expect(localCompute(c, 1, 2, 252).every(x => x === null)).toBe(true);
    expect(localCompute(c, 20, 0, 252).every(x => x === null)).toBe(true);
    expect(localCompute(c, 20, 2, 10).every(x => x === null)).toBe(true);
});

test('local: NaN returns all null', () => {
    const c = new Array(300).fill(100);
    c[5] = NaN;
    expect(localCompute(c, 20, 2, 252).every(x => x === null)).toBe(true);
});

test('local: flat market yields rank = 100', () => {
    const c = new Array(300).fill(100);
    const r = localCompute(c, 20, 2, 252);
    // First populated is i = 270; everything from there should be 100 (all values ≤ current 0).
    for (let i = 270; i < 300; i++) {
        expect(Math.abs(r[i] - 100)).toBeLessThan(1e-9);
    }
});

test('local: volatility spike ranks high', () => {
    let stateBig = 42n;
    const MASK = 0xFFFFFFFFFFFFFFFFn;
    const c = new Array(252).fill(100);
    for (let i = 0; i < 50; i++) {
        stateBig = (stateBig * 6364136223846793005n + 1442695040888963407n) & MASK;
        const u = Number(stateBig >> 32n) / 0xFFFFFFFF;
        c.push(100 + (u - 0.5) * 50);
    }
    const r = localCompute(c, 20, 2, 252);
    const populated = r.filter(v => v != null);
    const lastFew = populated.slice(-20);
    const maxP = Math.max(...lastFew);
    expect(maxP).toBeGreaterThan(70);
});

test('local: output in [0, 100] range', () => {
    let stateBig = 42n;
    const MASK = 0xFFFFFFFFFFFFFFFFn;
    const c = Array.from({ length: 400 }, () => {
        stateBig = (stateBig * 6364136223846793005n + 1442695040888963407n) & MASK;
        const u = Number(stateBig >> 32n) / 0xFFFFFFFF;
        return 100 + (u - 0.5) * 5;
    });
    const r = localCompute(c, 20, 2, 252);
    for (const v of r) {
        if (v == null) continue;
        expect(v).toBeGreaterThanOrEqual(0);
        expect(v).toBeLessThanOrEqual(100);
    }
});

test('local: output length matches input', () => {
    const c = new Array(300).fill(100);
    expect(localCompute(c, 20, 2, 252).length).toBe(300);
});

test('local: leading nulls until lookback + bb_period − 1', () => {
    // First populated BBWP is at i = lookback - 1 + bb_period - 1 = 270
    // (window needs all `lookback` width values populated, which starts at bb_period-1).
    const c = new Array(300).fill(100);
    const r = localCompute(c, 20, 2, 252);
    for (let i = 0; i < 270; i++) expect(r[i]).toBe(null);
    expect(r[270]).not.toBe(null);
});

test('local: deterministic', () => {
    const c = Array.from({ length: 280 }, (_, i) => 100 + Math.sin(i * 0.1));
    expect(localCompute(c, 20, 2, 252)).toEqual(localCompute(c, 20, 2, 252));
});

test('local: increasing width series → BBWP near 100 at end', () => {
    // Monotonically increasing variance → width is monotonically rising,
    // so the last bar is the max, percentile rank = 100.
    const c = [];
    for (let i = 0; i < 280; i++) {
        const vol = 0.1 + i * 0.05;     // increasing volatility
        c.push(100 + Math.sin(i * 0.7) * vol);
    }
    const r = localCompute(c, 20, 2, 252);
    const last = r[r.length - 1];
    expect(last).toBeGreaterThan(90);
});

// ── badges ────────────────────────────────────────────────────────

test('regimeBadge: 7 tiers', () => {
    expect(regimeBadge(3).key).toMatch(/extreme_squeeze/);
    expect(regimeBadge(15).key).toMatch(/squeeze/);
    expect(regimeBadge(30).key).toMatch(/low/);
    expect(regimeBadge(50).key).toMatch(/neutral/);
    expect(regimeBadge(70).key).toMatch(/elevated/);
    expect(regimeBadge(85).key).toMatch(/expansion/);
    expect(regimeBadge(98).key).toMatch(/extreme_expansion/);
    expect(regimeBadge(null).key).toMatch(/unknown/);
});

test('trendBadge: tiers', () => {
    expect(trendBadge([10, 10, 10, 10, 10]).key).toMatch(/flat/);
    expect(trendBadge([10, 20, 30, 40, 90]).key).toMatch(/rising_fast/);
    expect(trendBadge([90, 80, 70, 60, 10]).key).toMatch(/falling_fast/);
    expect(trendBadge([]).key).toMatch(/unknown/);
});

test('triggerBadge: armed / firing / none / unknown', () => {
    // min ≤ 10, last > min + 5, last < 50 → firing
    expect(triggerBadge([5, 6, 7, 8, 20]).key).toMatch(/firing/);
    // min ≤ 10, but last ≈ min → armed
    expect(triggerBadge([5, 6, 7, 8, 7]).key).toMatch(/armed/);
    // min ≤ 10, but last ≥ 50 → still firing per logic? Actually if last < 50 fails → falls to armed branch
    expect(triggerBadge([5, 6, 7, 8, 60]).key).toMatch(/armed/);
    // min > 10 → none
    expect(triggerBadge([30, 40, 50, 60, 70]).key).toMatch(/none/);
    expect(triggerBadge([]).key).toMatch(/unknown/);
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
    for (const k of ['rising-vol','squeeze-end','oscillating','steady',
                     'flat','short-lookback','high-stdev','spike-and-mean-revert']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.closes, inp.bb_period, inp.n_stdev, inp.lookback);
        expect(r.length).toBe(inp.closes.length);
    }
});

test('demo rising-vol ends with high BBWP', () => {
    const inp = makeDemoInput('rising-vol');
    const r = localCompute(inp.closes, inp.bb_period, inp.n_stdev, inp.lookback);
    const last = r[r.length - 1];
    expect(last).toBeGreaterThan(60);
});

test('demo flat: BBWP = 100 at end (every value ≤ current 0)', () => {
    const inp = makeDemoInput('flat');
    const r = localCompute(inp.closes, inp.bb_period, inp.n_stdev, inp.lookback);
    const last = r[r.length - 1];
    expect(Math.abs(last - 100)).toBeLessThan(1e-9);
});

test('demo short-lookback uses lookback=60', () => {
    const inp = makeDemoInput('short-lookback');
    expect(inp.lookback).toBe(60);
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
    expect(fmtPct(75.5)).toBe('75.50%');
    expect(fmtNum(1.23456)).toBe('1.2346');
    expect(fmtInt(42.9)).toBe('42');
    expect(fmtPrice(NaN)).toBe('—');
});

test('DEFAULTS sanity', () => {
    expect(DEFAULT_INPUTS.closes).toEqual([]);
    expect(DEFAULT_INPUTS.bb_period).toBe(DEFAULT_BB_PERIOD);
    expect(DEFAULT_INPUTS.n_stdev).toBe(DEFAULT_N_STDEV);
    expect(DEFAULT_INPUTS.lookback).toBe(DEFAULT_LOOKBACK);
    expect(DEFAULT_BB_PERIOD).toBe(20);
    expect(DEFAULT_N_STDEV).toBe(2);
    expect(DEFAULT_LOOKBACK).toBe(252);
    expect(MIN_BB_PERIOD).toBe(2);
    expect(MAX_BB_PERIOD).toBe(500);
    expect(MIN_LOOKBACK).toBe(2);
    expect(MAX_LOOKBACK).toBe(2000);
});
