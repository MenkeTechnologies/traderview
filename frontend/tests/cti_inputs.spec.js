// Chande Trend Index helpers: parser, validator, localCompute parity (Pearson r vs linear ramp), badges.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, DEFAULT_PERIOD, MIN_PERIOD, MAX_PERIOD,
    parseClosesBlob, closesToBlob, validateInputs, buildBody, localCompute,
    strengthBadge, crossBadge, changeBadge, summarizeCloses,
    makeDemoInput,
    fmtNum, fmtNumSigned, fmtPrice, fmtInt,
} from '../js/_cti_inputs.js';

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
    expect(validateInputs({ ...base, closes: new Array(5).fill(100) })).toMatch(/period/);
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

test('local: invalid period returns all null', () => {
    expect(localCompute(new Array(30).fill(100), 1).every(x => x === null)).toBe(true);
});

test('local: NaN returns all null', () => {
    expect(localCompute([100, NaN, 101, 102, 103], 3).every(x => x === null)).toBe(true);
});

test('local: perfect uptrend → CTI = +1', () => {
    const c = Array.from({ length: 30 }, (_, i) => 100 + (i + 1));
    const r = localCompute(c, 14);
    expect(Math.abs(r[29] - 1)).toBeLessThan(1e-9);
});

test('local: perfect downtrend → CTI = -1', () => {
    const c = Array.from({ length: 30 }, (_, i) => 200 - i);
    const r = localCompute(c, 14);
    expect(Math.abs(r[29] + 1)).toBeLessThan(1e-9);
});

test('local: flat → CTI = 0 (zero variance branch)', () => {
    const c = new Array(30).fill(100);
    const r = localCompute(c, 14);
    expect(r[29]).toBe(0);
});

test('local: output length matches input', () => {
    const c = Array.from({ length: 50 }, (_, i) => 100 + Math.sin(i * 0.1) * 5);
    const r = localCompute(c, 14);
    expect(r.length).toBe(50);
    expect(r[12]).toBe(null);
    expect(r[13]).not.toBe(null);
});

test('local: output in [-1, +1]', () => {
    let s = 11n;
    const MASK = 0xFFFFFFFFFFFFFFFFn;
    const c = Array.from({ length: 200 }, () => {
        s = (s * 6364136223846793005n + 1442695040888963407n) & MASK;
        return 100 + Number(s >> 32n) / 0xFFFFFFFF * 4 - 2;
    });
    const r = localCompute(c, 14);
    for (const v of r) {
        if (v == null) continue;
        expect(v).toBeGreaterThanOrEqual(-1);
        expect(v).toBeLessThanOrEqual(1);
    }
});

test('local: deterministic', () => {
    const c = Array.from({ length: 30 }, (_, i) => 100 + Math.sin(i * 0.2));
    expect(localCompute(c, 14)).toEqual(localCompute(c, 14));
});

// ── badges ────────────────────────────────────────────────────────

test('strengthBadge: 7 tiers', () => {
    expect(strengthBadge(0.95).key).toMatch(/perfect_up/);
    expect(strengthBadge(0.7).key).toMatch(/strong_up/);
    expect(strengthBadge(0.3).key).toMatch(/weak_up/);
    expect(strengthBadge(0).key).toMatch(/no_trend/);
    expect(strengthBadge(-0.3).key).toMatch(/weak_down/);
    expect(strengthBadge(-0.7).key).toMatch(/strong_down/);
    expect(strengthBadge(-0.95).key).toMatch(/perfect_down/);
    expect(strengthBadge(null).key).toMatch(/unknown/);
});

test('crossBadge: up / down / none', () => {
    expect(crossBadge([null, -0.5, -0.3, 0.2, 0.4]).key).toMatch(/up_recent/);
    expect(crossBadge([null, 0.5, 0.3, -0.2, -0.4]).key).toMatch(/down_recent/);
    expect(crossBadge([0.3, 0.4, 0.5]).key).toMatch(/none/);
});

test('crossBadge: barsAgo populated', () => {
    const r = crossBadge([null, -0.5, -0.3, 0.2, 0.4, 0.5]);
    expect(r.barsAgo).toBe(2);
});

test('changeBadge: 5 tiers', () => {
    expect(changeBadge([0, 0.2, 0.4, 0.6, 0.9]).key).toMatch(/strengthening_up/);
    expect(changeBadge([0, 0.05, 0.1, 0.15, 0.2]).key).toMatch(/firming_up/);
    expect(changeBadge([0.5, 0.5, 0.5, 0.5, 0.5]).key).toMatch(/stable/);
    expect(changeBadge([0.5, 0.4, 0.3, 0.2, 0.2]).key).toMatch(/weakening/);
    expect(changeBadge([0.9, 0.6, 0.4, 0.2, 0]).key).toMatch(/strengthening_down/);
    expect(changeBadge([]).key).toMatch(/unknown/);
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
    for (const k of ['uptrend','downtrend','flat','noisy-trend',
                     'oscillating','reversal','chop-then-trend','short-period']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.closes, inp.period);
        expect(r.length).toBe(inp.closes.length);
    }
});

test('demo uptrend: terminal CTI near +1', () => {
    const inp = makeDemoInput('uptrend');
    const r = localCompute(inp.closes, inp.period);
    expect(r[r.length - 1]).toBeGreaterThan(0.9);
});

test('demo downtrend: terminal CTI near -1', () => {
    const inp = makeDemoInput('downtrend');
    const r = localCompute(inp.closes, inp.period);
    expect(r[r.length - 1]).toBeLessThan(-0.9);
});

test('demo flat: CTI = 0 throughout populated', () => {
    const inp = makeDemoInput('flat');
    const r = localCompute(inp.closes, inp.period);
    for (let i = inp.period - 1; i < inp.closes.length; i++) {
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
    expect(fmtNum(0.7654)).toBe('0.7654');
    expect(fmtNumSigned(0.5)).toBe('+0.5000');
    expect(fmtNumSigned(-0.5)).toBe('-0.5000');
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
