// Chande Momentum Oscillator helpers: parser, validator, localCompute parity, badges.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, DEFAULT_PERIOD, MIN_PERIOD, MAX_PERIOD,
    parseClosesBlob, closesToBlob, validateInputs, buildBody, localCompute,
    zoneBadge, crossBadge, trendBadge, summarizeCloses,
    makeDemoInput,
    fmtNum, fmtNumSigned, fmtPrice, fmtInt,
} from '../js/_cmo_inputs.js';

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
    expect(validateInputs({ closes: new Array(20).fill(100), period: 14 })).toBe(null);
});

test('validate rejects: bad array / bad period / short / NaN', () => {
    const base = { closes: new Array(20).fill(100), period: 14 };
    expect(validateInputs({ ...base, closes: 'no' })).toMatch(/closes/);
    expect(validateInputs({ ...base, period: 1 })).toMatch(/period/);
    expect(validateInputs({ ...base, period: 9999 })).toMatch(/period/);
    expect(validateInputs({ ...base, closes: new Array(5).fill(100) })).toMatch(/period \+ 1/);
    const bad = [...new Array(20)].map((_, i) => i === 5 ? NaN : 100);
    expect(validateInputs({ ...base, closes: bad })).toMatch(/finite/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody copies + passes through', () => {
    expect(buildBody({ closes: [100, 101], period: 14 }))
        .toEqual({ closes: [100, 101], period: 14 });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: empty returns empty', () => {
    expect(localCompute([], 14)).toEqual([]);
});

test('local: too-short returns all null', () => {
    expect(localCompute(new Array(5).fill(100), 14).every(x => x === null)).toBe(true);
});

test('local: period too small returns all null', () => {
    const c = Array.from({ length: 50 }, (_, i) => 100 + i);
    expect(localCompute(c, 1).every(x => x === null)).toBe(true);
});

test('local: strict uptrend → CMO = 100', () => {
    const c = Array.from({ length: 50 }, (_, i) => 100 + i);
    const r = localCompute(c, 14);
    expect(Math.abs(r[49] - 100)).toBeLessThan(1e-9);
});

test('local: strict downtrend → CMO = -100', () => {
    const c = Array.from({ length: 50 }, (_, i) => 200 - i);
    const r = localCompute(c, 14);
    expect(Math.abs(r[49] + 100)).toBeLessThan(1e-9);
});

test('local: flat → CMO = 0', () => {
    const c = new Array(50).fill(100);
    const r = localCompute(c, 14);
    expect(r[49]).toBe(0);
});

test('local: symmetric alternation → CMO = 0', () => {
    const c = Array.from({ length: 50 }, (_, i) => 100 + (i % 2));
    const r = localCompute(c, 14);
    expect(Math.abs(r[49])).toBeLessThan(1e-9);
});

test('local: output in [-100, +100]', () => {
    let s = 7n;
    const MASK = 0xFFFFFFFFFFFFFFFFn;
    const c = Array.from({ length: 200 }, () => {
        s = (s * 6364136223846793005n + 1442695040888963407n) & MASK;
        return 100 + Number(s >> 32n) / 0xFFFFFFFF * 10 - 5;
    });
    const r = localCompute(c, 14);
    for (const v of r) {
        if (v == null) continue;
        expect(v).toBeGreaterThanOrEqual(-100);
        expect(v).toBeLessThanOrEqual(100);
    }
});

test('local: output length matches input', () => {
    const c = Array.from({ length: 50 }, (_, i) => 100 + i * 0.1);
    const r = localCompute(c, 14);
    expect(r.length).toBe(50);
    expect(r[13]).toBe(null);
    expect(r[14]).not.toBe(null);
});

test('local: deterministic', () => {
    const c = Array.from({ length: 30 }, (_, i) => 100 + Math.sin(i * 0.2));
    expect(localCompute(c, 14)).toEqual(localCompute(c, 14));
});

test('local: NaN returns all null', () => {
    const c = new Array(30).fill(100);
    c[5] = NaN;
    expect(localCompute(c, 14).every(x => x === null)).toBe(true);
});

// ── badges ────────────────────────────────────────────────────────

test('zoneBadge: 7 tiers', () => {
    expect(zoneBadge(85).key).toMatch(/extreme_overbought/);
    expect(zoneBadge(60).key).toMatch(/overbought/);
    expect(zoneBadge(30).key).toMatch(/bullish_lean/);
    expect(zoneBadge(0).key).toMatch(/neutral/);
    expect(zoneBadge(-30).key).toMatch(/bearish_lean/);
    expect(zoneBadge(-60).key).toMatch(/oversold/);
    expect(zoneBadge(-85).key).toMatch(/extreme_oversold/);
    expect(zoneBadge(null).key).toMatch(/unknown/);
});

test('crossBadge: 6 transitions + none + unknown', () => {
    expect(crossBadge([null, 40, 60]).key).toMatch(/into_overbought/);
    expect(crossBadge([null, 60, 40]).key).toMatch(/out_of_overbought/);
    expect(crossBadge([null, -40, -60]).key).toMatch(/into_oversold/);
    expect(crossBadge([null, -60, -40]).key).toMatch(/out_of_oversold/);
    expect(crossBadge([null, -10, 10]).key).toMatch(/zero_up/);
    expect(crossBadge([null, 10, -10]).key).toMatch(/zero_down/);
    expect(crossBadge([10, 20, 30]).key).toMatch(/none/);
});

test('crossBadge: barsAgo populated', () => {
    // null, then prev=40, v=60 → into_overbought at idx 2; rest stays > 50 → idx stays at 2.
    const r = crossBadge([null, 40, 60, 65, 70]);
    expect(r.barsAgo).toBe(2);
});

test('trendBadge: tiers', () => {
    expect(trendBadge([0, 0, 0, 0, 0]).key).toMatch(/flat/);
    expect(trendBadge([-50, -30, -10, 10, 90]).key).toMatch(/rising_fast/);
    expect(trendBadge([90, 70, 50, 30, -50]).key).toMatch(/falling_fast/);
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
    for (const k of ['uptrend','downtrend','flat','alternating',
                     'oscillating','reversal-up','reversal-down','short-period']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.closes, inp.period);
        expect(r.length).toBe(inp.closes.length);
    }
});

test('demo uptrend: terminal CMO close to +100', () => {
    const inp = makeDemoInput('uptrend');
    const r = localCompute(inp.closes, inp.period);
    expect(r[r.length - 1]).toBeGreaterThan(80);
});

test('demo downtrend: terminal CMO close to -100', () => {
    const inp = makeDemoInput('downtrend');
    const r = localCompute(inp.closes, inp.period);
    expect(r[r.length - 1]).toBeLessThan(-80);
});

test('demo flat: CMO = 0 throughout populated', () => {
    const inp = makeDemoInput('flat');
    const r = localCompute(inp.closes, inp.period);
    for (let i = inp.period; i < inp.closes.length; i++) {
        expect(r[i]).toBe(0);
    }
});

test('demo short-period uses period=5', () => {
    const inp = makeDemoInput('short-period');
    expect(inp.period).toBe(5);
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
    expect(fmtNumSigned(1.5)).toBe('+1.50');
    expect(fmtNumSigned(-1.5)).toBe('-1.50');
    expect(fmtPrice(100.456)).toBe('100.46');
    expect(fmtInt(42.9)).toBe('42');
    expect(fmtNum(NaN)).toBe('—');
});

test('DEFAULTS sanity', () => {
    expect(DEFAULT_INPUTS.closes).toEqual([]);
    expect(DEFAULT_INPUTS.period).toBe(DEFAULT_PERIOD);
    expect(DEFAULT_PERIOD).toBe(14);
    expect(MIN_PERIOD).toBe(2);
    expect(MAX_PERIOD).toBe(500);
});
