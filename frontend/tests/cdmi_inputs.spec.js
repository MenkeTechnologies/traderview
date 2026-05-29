// Chande DMI helpers: parser, validator, localCompute parity (stdev + SMA + Wilder RSI), badges.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, DEFAULT_TD_CONST, DEFAULT_STD_PERIOD, DEFAULT_TD_MIN, DEFAULT_TD_MAX,
    MIN_PERIOD, MAX_PERIOD,
    parseClosesBlob, closesToBlob, validateInputs, buildBody, localCompute,
    wilderRsiAt, smaOpt, zoneBadge, crossBadge, trendBadge, currentTdInfo, summarizeCloses,
    makeDemoInput,
    fmtNum, fmtPrice, fmtInt,
} from '../js/_cdmi_inputs.js';

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
        closes: new Array(50).fill(100), td_const: 14, std_period: 5, td_min: 5, td_max: 30,
    })).toBe(null);
});

test('validate rejects: bad arrays / bad periods / max < min / max < const / short / NaN', () => {
    const base = { closes: new Array(50).fill(100), td_const: 14, std_period: 5, td_min: 5, td_max: 30 };
    expect(validateInputs({ ...base, closes: 'no' })).toMatch(/closes/);
    expect(validateInputs({ ...base, td_const: 1 })).toMatch(/td_const/);
    expect(validateInputs({ ...base, std_period: 1 })).toMatch(/std_period/);
    expect(validateInputs({ ...base, td_min: 1 })).toMatch(/td_min/);
    expect(validateInputs({ ...base, td_min: 30, td_max: 5 })).toMatch(/td_max/);
    expect(validateInputs({ ...base, td_max: 10 })).toMatch(/td_max.*td_const/);
    expect(validateInputs({ ...base, closes: new Array(20).fill(100) })).toMatch(/2·std_period \+ td_max/);
    const bad = [...new Array(50)].map((_, i) => i === 5 ? NaN : 100);
    expect(validateInputs({ ...base, closes: bad })).toMatch(/finite/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody copies + passes through', () => {
    expect(buildBody({ closes: [100, 101], td_const: 14, std_period: 5, td_min: 5, td_max: 30 }))
        .toEqual({ closes: [100, 101], td_const: 14, std_period: 5, td_min: 5, td_max: 30 });
});

// ── Wilder RSI helper ─────────────────────────────────────────────

test('wilderRsiAt: monotone uptrend → 100', () => {
    const c = Array.from({ length: 30 }, (_, i) => 100 + i);
    expect(wilderRsiAt(c, 20, 10)).toBeCloseTo(100, 6);
});

test('wilderRsiAt: monotone downtrend → 0', () => {
    const c = Array.from({ length: 30 }, (_, i) => 130 - i);
    expect(wilderRsiAt(c, 20, 10)).toBeCloseTo(0, 6);
});

test('wilderRsiAt: flat → 50', () => {
    const c = new Array(30).fill(100);
    expect(wilderRsiAt(c, 20, 10)).toBe(50);
});

test('wilderRsiAt: i < td → null', () => {
    expect(wilderRsiAt([1, 2, 3], 1, 10)).toBe(null);
});

// ── smaOpt sanity ─────────────────────────────────────────────────

test('smaOpt: null in window → null in output', () => {
    const s = [1, null, 3, 4, 5];
    const r = smaOpt(s, 3);
    expect(r[0]).toBe(null);
    expect(r[1]).toBe(null);
    expect(r[2]).toBe(null);
    expect(r[4]).toBe(4);
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: invalid inputs return all null', () => {
    const c = new Array(200).fill(100);
    expect(localCompute(c, 1, 5, 5, 30).every(x => x === null)).toBe(true);
    expect(localCompute(c, 14, 5, 30, 5).every(x => x === null)).toBe(true);
});

test('local: NaN returns all null', () => {
    const c = new Array(200).fill(100);
    c[5] = NaN;
    expect(localCompute(c, 14, 5, 5, 30).every(x => x === null)).toBe(true);
});

test('local: flat market yields all null (zero stdev)', () => {
    const c = new Array(200).fill(100);
    expect(localCompute(c, 14, 5, 5, 30).every(x => x === null)).toBe(true);
});

test('local: uptrend yields DMI > 80', () => {
    const c = Array.from({ length: 200 }, (_, i) => 100 + i);
    const r = localCompute(c, 14, 5, 5, 30);
    const last = [...r].reverse().find(v => v != null);
    expect(last).toBeGreaterThan(80);
});

test('local: downtrend yields DMI < 20', () => {
    const c = Array.from({ length: 200 }, (_, i) => 300 - i);
    const r = localCompute(c, 14, 5, 5, 30);
    const last = [...r].reverse().find(v => v != null);
    expect(last).toBeLessThan(20);
});

test('local: output in [0, 100]', () => {
    let s = 42n;
    const MASK = 0xFFFFFFFFFFFFFFFFn;
    const c = Array.from({ length: 400 }, (_, i) => {
        s = (s * 6364136223846793005n + 1442695040888963407n) & MASK;
        const u = Number(s >> 32n) / 0xFFFFFFFF;
        return 100 + i * 0.1 + (u - 0.5) * 5;
    });
    const r = localCompute(c, 14, 5, 5, 30);
    for (const v of r) {
        if (v == null) continue;
        expect(v).toBeGreaterThanOrEqual(0);
        expect(v).toBeLessThanOrEqual(100);
    }
});

test('local: output length matches input', () => {
    const c = new Array(200).fill(100);
    expect(localCompute(c, 14, 5, 5, 30).length).toBe(200);
});

test('local: deterministic', () => {
    const c = Array.from({ length: 200 }, (_, i) => 100 + Math.sin(i * 0.1) * 5);
    expect(localCompute(c, 14, 5, 5, 30)).toEqual(localCompute(c, 14, 5, 5, 30));
});

// ── badges ────────────────────────────────────────────────────────

test('zoneBadge: 5 tiers', () => {
    expect(zoneBadge(90).key).toMatch(/overbought/);
    expect(zoneBadge(70).key).toMatch(/strong_buy/);
    expect(zoneBadge(50).key).toMatch(/neutral/);
    expect(zoneBadge(25).key).toMatch(/strong_sell/);
    expect(zoneBadge(10).key).toMatch(/oversold/);
    expect(zoneBadge(null).key).toMatch(/unknown/);
});

test('crossBadge: into/out_of overbought + oversold + none', () => {
    expect(crossBadge([null, 65, 75]).key).toMatch(/into_overbought/);
    expect(crossBadge([null, 75, 65]).key).toMatch(/out_of_overbought/);
    expect(crossBadge([null, 35, 25]).key).toMatch(/into_oversold/);
    expect(crossBadge([null, 25, 35]).key).toMatch(/out_of_oversold/);
    expect(crossBadge([50, 50, 50]).key).toMatch(/none/);
});

test('crossBadge: barsAgo populated', () => {
    const r = crossBadge([null, 65, 75, 80, 85]);
    expect(r.barsAgo).toBe(2);
});

test('trendBadge: tiers', () => {
    expect(trendBadge([50, 50, 50, 50, 50]).key).toMatch(/flat/);
    expect(trendBadge([10, 20, 30, 40, 90]).key).toMatch(/rising_fast/);
    expect(trendBadge([90, 80, 70, 60, 10]).key).toMatch(/falling_fast/);
    expect(trendBadge([]).key).toMatch(/unknown/);
});

// ── currentTdInfo ────────────────────────────────────────────────

test('currentTdInfo: td within bounds for a quiet uptrend', () => {
    const c = Array.from({ length: 100 }, (_, i) => 100 + i * 0.5);
    const r = currentTdInfo(c, 14, 5, 5, 30);
    expect(r.td).toBeGreaterThanOrEqual(5);
    expect(r.td).toBeLessThanOrEqual(30);
});

test('currentTdInfo: null for empty closes', () => {
    expect(currentTdInfo([], 14, 5, 5, 30).td).toBe(null);
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
    for (const k of ['uptrend','downtrend','quiet-market','volatile-market',
                     'choppy-range','reversal-up','reversal-down','short-bounds']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.closes, inp.td_const, inp.std_period, inp.td_min, inp.td_max);
        expect(r.length).toBe(inp.closes.length);
    }
});

test('demo uptrend: terminal DMI > 80', () => {
    const inp = makeDemoInput('uptrend');
    const r = localCompute(inp.closes, inp.td_const, inp.std_period, inp.td_min, inp.td_max);
    const last = [...r].reverse().find(v => v != null);
    expect(last).toBeGreaterThan(80);
});

test('demo downtrend: terminal DMI < 20', () => {
    const inp = makeDemoInput('downtrend');
    const r = localCompute(inp.closes, inp.td_const, inp.std_period, inp.td_min, inp.td_max);
    const last = [...r].reverse().find(v => v != null);
    expect(last).toBeLessThan(20);
});

test('demo short-bounds uses fixed td=14', () => {
    const inp = makeDemoInput('short-bounds');
    expect(inp.td_min).toBe(14);
    expect(inp.td_max).toBe(14);
});

// ── formatters ────────────────────────────────────────────────────

test('closesToBlob round-trips', () => {
    const c = [100, 100.5, 101.25];
    const back = parseClosesBlob(closesToBlob(c));
    expect(back.errors).toEqual([]);
    expect(back.closes).toEqual(c);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtNum(1.23456)).toBe('1.23');
    expect(fmtPrice(100.456)).toBe('100.46');
    expect(fmtInt(42.9)).toBe('42');
    expect(fmtNum(NaN)).toBe('—');
});

test('DEFAULTS sanity', () => {
    expect(DEFAULT_INPUTS.closes).toEqual([]);
    expect(DEFAULT_INPUTS.td_const).toBe(DEFAULT_TD_CONST);
    expect(DEFAULT_INPUTS.std_period).toBe(DEFAULT_STD_PERIOD);
    expect(DEFAULT_INPUTS.td_min).toBe(DEFAULT_TD_MIN);
    expect(DEFAULT_INPUTS.td_max).toBe(DEFAULT_TD_MAX);
    expect(DEFAULT_TD_CONST).toBe(14);
    expect(DEFAULT_STD_PERIOD).toBe(5);
    expect(DEFAULT_TD_MIN).toBe(5);
    expect(DEFAULT_TD_MAX).toBe(30);
    expect(MIN_PERIOD).toBe(2);
    expect(MAX_PERIOD).toBe(500);
});
