// Chande Volatility Index helpers: parser, validator, localCompute parity, badges.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, DEFAULT_EMA, DEFAULT_ROC, MIN_EMA, MIN_ROC, MAX_PERIOD,
    parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute,
    regimeBadge, crossBadge, trendBadge, summarizeBars,
    makeDemoInput,
    fmtPct, fmtPctSigned, fmtPrice, fmtInt,
} from '../js/_cvi_inputs.js';

const b = (h, l) => ({ high: h, low: l });

// ── parser ────────────────────────────────────────────────────────

test('parseBarsBlob: 2 tokens per line', () => {
    const r = parseBarsBlob('101 99\n# noise\n102 100');
    expect(r.errors).toEqual([]);
    expect(r.bars).toEqual([b(101, 99), b(102, 100)]);
});

test('parseBarsBlob: rejects wrong count / non-positive / low > high', () => {
    expect(parseBarsBlob('100').errors[0].message).toMatch(/2 tokens/);
    expect(parseBarsBlob('-1 99').errors[0].message).toMatch(/HL/);
    expect(parseBarsBlob('99 100').errors[0].message).toMatch(/low > high/);
});

test('parseBarsBlob: non-string returns 1 error', () => {
    expect(parseBarsBlob(undefined).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts valid', () => {
    const bars = Array.from({ length: 30 }, () => b(101, 99));
    expect(validateInputs({ bars, ema_period: 10, roc_period: 10 })).toBe(null);
});

test('validate rejects: bad array / bad ema / bad roc / too short / non-finite / inverted', () => {
    const ok = Array.from({ length: 30 }, () => b(101, 99));
    expect(validateInputs({ bars: 'no', ema_period: 10, roc_period: 10 })).toMatch(/bars/);
    expect(validateInputs({ bars: ok, ema_period: 1, roc_period: 10 })).toMatch(/ema_period/);
    expect(validateInputs({ bars: ok, ema_period: 10, roc_period: 0 })).toMatch(/roc_period/);
    expect(validateInputs({ bars: ok.slice(0, 5), ema_period: 10, roc_period: 10 })).toMatch(/ema_period \+ roc_period/);
    const badNan = [...ok];
    badNan[5] = { high: NaN, low: 99 };
    expect(validateInputs({ bars: badNan, ema_period: 10, roc_period: 10 })).toMatch(/finite/);
    const inv = [...ok];
    inv[5] = b(99, 101);
    expect(validateInputs({ bars: inv, ema_period: 10, roc_period: 10 })).toMatch(/high < low/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody trims bars to HL', () => {
    const body = buildBody({ bars: [{ ...b(101, 99), x: 1 }], ema_period: 10, roc_period: 10 });
    expect(body).toEqual({ bars: [b(101, 99)], ema_period: 10, roc_period: 10 });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: invalid inputs return all null', () => {
    const bars = Array.from({ length: 50 }, () => b(101, 99));
    expect(localCompute(bars, 1, 10).every(x => x === null)).toBe(true);
    expect(localCompute(bars.slice(0, 5), 10, 10).every(x => x === null)).toBe(true);
});

test('local: NaN returns all null', () => {
    const bars = Array.from({ length: 50 }, () => b(101, 99));
    bars[5] = b(NaN, 99);
    expect(localCompute(bars, 10, 10).every(x => x === null)).toBe(true);
});

test('local: constant range yields CVI = 0', () => {
    const bars = Array.from({ length: 50 }, () => b(101, 99));
    const r = localCompute(bars, 10, 10);
    for (const v of r) if (v != null) expect(Math.abs(v)).toBeLessThan(1e-9);
});

test('local: expanding range yields positive CVI', () => {
    const bars = Array.from({ length: 30 }, () => b(101, 99));
    for (let i = 0; i < 20; i++) {
        const half = 1 + i * 0.5;
        bars.push(b(100 + half, 100 - half));
    }
    const r = localCompute(bars, 10, 10);
    expect(r[bars.length - 1]).toBeGreaterThan(0);
});

test('local: contracting range yields negative CVI', () => {
    const bars = Array.from({ length: 30 }, (_, i) => {
        const half = 10 - i * 0.2;
        return b(100 + half, 100 - half);
    });
    for (let i = 0; i < 20; i++) bars.push(b(100.5, 99.5));
    const r = localCompute(bars, 10, 10);
    expect(r[bars.length - 1]).toBeLessThan(0);
});

test('local: output length matches input', () => {
    const bars = Array.from({ length: 50 }, () => b(101, 99));
    expect(localCompute(bars, 10, 10).length).toBe(50);
});

test('local: leading nulls until ema + roc − 1', () => {
    const bars = Array.from({ length: 50 }, () => b(101, 99));
    const r = localCompute(bars, 10, 10);
    for (let i = 0; i < 18; i++) expect(r[i]).toBe(null);
    expect(r[19]).not.toBe(null);
});

test('local: deterministic', () => {
    const bars = Array.from({ length: 30 }, (_, i) => b(101 + i * 0.1, 99 + i * 0.1));
    expect(localCompute(bars, 10, 10)).toEqual(localCompute(bars, 10, 10));
});

// ── badges ────────────────────────────────────────────────────────

test('regimeBadge: 5 tiers', () => {
    expect(regimeBadge(50).key).toMatch(/expansion_strong/);
    expect(regimeBadge(20).key).toMatch(/expansion/);
    expect(regimeBadge(0).key).toMatch(/steady/);
    expect(regimeBadge(-20).key).toMatch(/contraction/);
    expect(regimeBadge(-50).key).toMatch(/contraction_strong/);
    expect(regimeBadge(null).key).toMatch(/unknown/);
});

test('crossBadge: up / down / none', () => {
    expect(crossBadge([null, -10, -5, 5, 10]).key).toMatch(/up_recent/);
    expect(crossBadge([null, 10, 5, -5, -10]).key).toMatch(/down_recent/);
    expect(crossBadge([1, 2, 3]).key).toMatch(/none/);
});

test('crossBadge: barsAgo populated', () => {
    const r = crossBadge([null, -10, -5, 5, 10, 15]);
    expect(r.barsAgo).toBe(2);
});

test('trendBadge: tiers', () => {
    expect(trendBadge([0, 0, 0, 0, 0]).key).toMatch(/flat/);
    expect(trendBadge([-50, -25, 0, 25, 100]).key).toMatch(/rising_fast/);
    expect(trendBadge([100, 75, 50, 25, -50]).key).toMatch(/falling_fast/);
    expect(trendBadge([]).key).toMatch(/unknown/);
});

// ── summarize ─────────────────────────────────────────────────────

test('summarizeBars: count / mean_range / extrema', () => {
    const bars = [b(102, 98), b(105, 100), b(106, 101)];
    const s = summarizeBars(bars);
    expect(s.count).toBe(3);
    expect(s.mean_range).toBeCloseTo((4 + 5 + 5) / 3, 6);
    expect(s.min_low).toBe(98);
    expect(s.max_high).toBe(106);
});

test('summarizeBars: empty → NaN', () => {
    const s = summarizeBars([]);
    expect(s.count).toBe(0);
    expect(Number.isNaN(s.mean_range)).toBe(true);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + computes', () => {
    for (const k of ['expanding','contracting','steady','spike',
                     'oscillating','long-ema','short-roc','climax-volatility']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.bars, inp.ema_period, inp.roc_period);
        expect(r.length).toBe(inp.bars.length);
    }
});

test('demo expanding: terminal CVI > 0', () => {
    const inp = makeDemoInput('expanding');
    const r = localCompute(inp.bars, inp.ema_period, inp.roc_period);
    expect(r[r.length - 1]).toBeGreaterThan(0);
});

test('demo contracting: terminal CVI < 0', () => {
    const inp = makeDemoInput('contracting');
    const r = localCompute(inp.bars, inp.ema_period, inp.roc_period);
    expect(r[r.length - 1]).toBeLessThan(0);
});

test('demo steady: terminal |CVI| small', () => {
    const inp = makeDemoInput('steady');
    const r = localCompute(inp.bars, inp.ema_period, inp.roc_period);
    expect(Math.abs(r[r.length - 1])).toBeLessThan(10);
});

test('demo long-ema uses 25/25', () => {
    const inp = makeDemoInput('long-ema');
    expect(inp.ema_period).toBe(25);
    expect(inp.roc_period).toBe(25);
});

// ── formatters ────────────────────────────────────────────────────

test('barsToBlob round-trips', () => {
    const bars = [b(101, 99), b(102, 100)];
    const back = parseBarsBlob(barsToBlob(bars));
    expect(back.errors).toEqual([]);
    expect(back.bars).toEqual(bars);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtPct(2.5)).toBe('2.50%');
    expect(fmtPctSigned(2.5)).toBe('+2.50%');
    expect(fmtPctSigned(-2.5)).toBe('-2.50%');
    expect(fmtPrice(100.456)).toBe('100.46');
    expect(fmtInt(42.9)).toBe('42');
    expect(fmtPct(NaN)).toBe('—');
});

test('DEFAULTS sanity', () => {
    expect(DEFAULT_INPUTS.bars).toEqual([]);
    expect(DEFAULT_INPUTS.ema_period).toBe(DEFAULT_EMA);
    expect(DEFAULT_INPUTS.roc_period).toBe(DEFAULT_ROC);
    expect(DEFAULT_EMA).toBe(10);
    expect(DEFAULT_ROC).toBe(10);
    expect(MIN_EMA).toBe(2);
    expect(MIN_ROC).toBe(1);
    expect(MAX_PERIOD).toBe(500);
});
