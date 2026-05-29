// A/D Oscillator helpers: parser, validator, localCompute (per-bar + EMA) parity, badges.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, DEFAULT_PERIOD, MIN_PERIOD, MAX_PERIOD,
    parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute,
    pressureBadge, crossBadge, trendBadge, summarizeBars,
    makeDemoInput,
    fmtNum, fmtSigned, fmtPrice, fmtInt,
} from '../js/_ad_osc_inputs.js';

const b = (h, l, c, v) => ({ high: h, low: l, close: c, volume: v });

// ── parser ────────────────────────────────────────────────────────

test('parseBarsBlob: 4 tokens per line', () => {
    const r = parseBarsBlob('101 99 100 1000\n# midday\n102 100 101 1500');
    expect(r.errors).toEqual([]);
    expect(r.bars).toEqual([b(101, 99, 100, 1000), b(102, 100, 101, 1500)]);
});

test('parseBarsBlob: rejects wrong count / non-positive HLC / negative vol / low > high / close OOR', () => {
    expect(parseBarsBlob('100 99 100').errors[0].message).toMatch(/4 tokens/);
    expect(parseBarsBlob('-1 99 100 100').errors[0].message).toMatch(/HLCV/);
    expect(parseBarsBlob('101 99 100 -50').errors[0].message).toMatch(/HLCV/);
    expect(parseBarsBlob('99 100 99 1000').errors[0].message).toMatch(/low > high/);
    expect(parseBarsBlob('101 99 50 1000').errors[0].message).toMatch(/close outside/);
});

test('parseBarsBlob: non-string returns 1 error', () => {
    expect(parseBarsBlob(undefined).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts valid', () => {
    const bars = Array.from({ length: 20 }, () => b(101, 99, 100, 1000));
    expect(validateInputs({ bars, period: 14 })).toBe(null);
});

test('validate rejects: bad array / bad period / too short / non-finite / negative vol / inverted', () => {
    const ok = Array.from({ length: 20 }, () => b(101, 99, 100, 1000));
    expect(validateInputs({ bars: 'no', period: 14 })).toMatch(/bars/);
    expect(validateInputs({ bars: ok, period: 1 })).toMatch(/period/);
    expect(validateInputs({ bars: ok, period: 9999 })).toMatch(/period/);
    expect(validateInputs({ bars: ok.slice(0, 5), period: 14 })).toMatch(/period/);
    const badNan = [...ok];
    badNan[5] = { high: NaN, low: 99, close: 100, volume: 1000 };
    expect(validateInputs({ bars: badNan, period: 14 })).toMatch(/finite/);
    const negVol = [...ok];
    negVol[5] = b(101, 99, 100, -100);
    expect(validateInputs({ bars: negVol, period: 14 })).toMatch(/negative/);
    const inv = [...ok];
    inv[5] = b(99, 101, 100, 1000);
    expect(validateInputs({ bars: inv, period: 14 })).toMatch(/high < low/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody trims bars to HLCV', () => {
    const body = buildBody({ bars: [{ ...b(101, 99, 100, 1000), x: 'y' }], period: 14 });
    expect(body).toEqual({ bars: [b(101, 99, 100, 1000)], period: 14 });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: invalid inputs return all null', () => {
    const bars = Array.from({ length: 30 }, () => b(101, 99, 100, 1000));
    expect(localCompute(bars, 1).per_bar.every(x => x === null)).toBe(true);
    expect(localCompute(bars.slice(0, 5), 14).per_bar.every(x => x === null)).toBe(true);
});

test('local: NaN or negative volume returns all null', () => {
    const bars = Array.from({ length: 30 }, () => b(101, 99, 100, 1000));
    bars[5] = b(NaN, 99, 100, 1000);
    expect(localCompute(bars, 14).per_bar.every(x => x === null)).toBe(true);
    const bars2 = Array.from({ length: 30 }, () => b(101, 99, 100, 1000));
    bars2[5] = b(101, 99, 100, -100);
    expect(localCompute(bars2, 14).per_bar.every(x => x === null)).toBe(true);
});

test('local: close-at-high → per_bar = +volume', () => {
    const bars = Array.from({ length: 30 }, () => b(110, 100, 110, 1000));
    const r = localCompute(bars, 14);
    for (const v of r.per_bar) expect(Math.abs(v - 1000)).toBeLessThan(1e-9);
});

test('local: close-at-low → per_bar = -volume', () => {
    const bars = Array.from({ length: 30 }, () => b(110, 100, 100, 1000));
    const r = localCompute(bars, 14);
    for (const v of r.per_bar) expect(Math.abs(v + 1000)).toBeLessThan(1e-9);
});

test('local: midpoint close → per_bar = 0', () => {
    const bars = Array.from({ length: 30 }, () => b(110, 100, 105, 1000));
    const r = localCompute(bars, 14);
    for (const v of r.per_bar) expect(Math.abs(v)).toBeLessThan(1e-9);
});

test('local: zero-range bar → per_bar = 0', () => {
    const bars = Array.from({ length: 30 }, () => b(100, 100, 100, 1000));
    const r = localCompute(bars, 14);
    for (const v of r.per_bar) expect(Math.abs(v)).toBeLessThan(1e-9);
});

test('local: EMA matches steady-state per_bar', () => {
    const bars = Array.from({ length: 30 }, () => b(110, 100, 110, 1000));
    const r = localCompute(bars, 14);
    expect(Math.abs(r.ema[29] - 1000)).toBeLessThan(1e-9);
});

test('local: output lengths match input', () => {
    const bars = Array.from({ length: 30 }, () => b(101, 99, 100, 1000));
    const r = localCompute(bars, 14);
    expect(r.per_bar.length).toBe(30);
    expect(r.ema.length).toBe(30);
});

test('local: leading EMA values null until period', () => {
    const bars = Array.from({ length: 30 }, () => b(110, 100, 110, 1000));
    const r = localCompute(bars, 14);
    for (let i = 0; i < 13; i++) expect(r.ema[i]).toBe(null);
    expect(r.ema[13]).not.toBe(null);
});

test('local: deterministic', () => {
    const bars = Array.from({ length: 30 }, (_, i) => b(101 + i * 0.1, 99 + i * 0.1, 100 + i * 0.1, 1000 + i * 10));
    const r1 = localCompute(bars, 14);
    const r2 = localCompute(bars, 14);
    expect(r1.per_bar).toEqual(r2.per_bar);
    expect(r1.ema).toEqual(r2.ema);
});

// ── badges ────────────────────────────────────────────────────────

test('pressureBadge: tiers (scaled by volume_scale)', () => {
    expect(pressureBadge(600, 1000).key).toMatch(/strong_buy/);
    expect(pressureBadge(100, 1000).key).toMatch(/buying/);
    expect(pressureBadge(20,  1000).key).toMatch(/neutral/);
    expect(pressureBadge(-100, 1000).key).toMatch(/selling/);
    expect(pressureBadge(-600, 1000).key).toMatch(/strong_sell/);
    expect(pressureBadge(0,    1000).key).toMatch(/neutral/);
    expect(pressureBadge(null, 1000).key).toMatch(/unknown/);
});

test('pressureBadge: NaN volume_scale → coarse direction only', () => {
    expect(pressureBadge(100, NaN).key).toMatch(/buying/);
    expect(pressureBadge(-100, NaN).key).toMatch(/selling/);
    expect(pressureBadge(0, NaN).key).toMatch(/neutral/);
});

test('crossBadge: up_recent / down_recent / none / unknown', () => {
    // EMA goes -100, -50, +50 → up cross at idx 2
    expect(crossBadge([null, null, -100, -50, 50, 60]).key).toMatch(/up_recent/);
    // EMA goes 100, 50, -50 → down cross
    expect(crossBadge([null, null, 100, 50, -50, -60]).key).toMatch(/down_recent/);
    // No cross
    expect(crossBadge([null, null, 100, 100, 100]).key).toMatch(/none/);
    // Empty
    expect(crossBadge([]).key).toMatch(/none|unknown/);
});

test('crossBadge: barsAgo populated on cross', () => {
    const r = crossBadge([null, null, -100, -50, 50, 60, 70]);
    expect(r.barsAgo).toBe(2);
});

test('trendBadge: tiers', () => {
    expect(trendBadge([0, 0, 0, 0, 0, 0, 0, 0, 0, 0]).key).toMatch(/flat/);
    expect(trendBadge([0, 1, 2, 3, 4, 5, 6, 7, 8, 100]).key).toMatch(/strong_up/);
    expect(trendBadge([10, 10.5, 10.7, 10.8, 10.9, 11, 11.2, 11.3, 11.5, 11.7]).key).toMatch(/up/);
    expect(trendBadge([100, 90, 80, 70, 60, 50, 40, 30, 20, 0]).key).toMatch(/strong_down/);
    expect(trendBadge([10, 9.9, 9.8, 9.7, 9.6, 9.5, 9.4, 9.3, 9.2, 9.1]).key).toMatch(/down/);
    expect(trendBadge([]).key).toMatch(/unknown/);
});

// ── summarize ─────────────────────────────────────────────────────

test('summarizeBars: count / last_close / mean_volume / extrema', () => {
    const bars = [b(102, 98, 100, 500), b(105, 100, 103, 700), b(106, 101, 105, 800)];
    const s = summarizeBars(bars);
    expect(s.count).toBe(3);
    expect(s.last_close).toBe(105);
    expect(s.mean_volume).toBeCloseTo(666.67, 1);
    expect(s.total_volume).toBe(2000);
    expect(s.min_low).toBe(98);
    expect(s.max_high).toBe(106);
});

test('summarizeBars: empty → NaN', () => {
    const s = summarizeBars([]);
    expect(s.count).toBe(0);
    expect(Number.isNaN(s.last_close)).toBe(true);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + computes', () => {
    for (const k of ['buying','selling','neutral','cross-up','cross-down',
                     'climax-buy','zero-range','short-period']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.bars, inp.period);
        expect(r.per_bar.length).toBe(inp.bars.length);
    }
});

test('demo buying: last EMA > 0', () => {
    const inp = makeDemoInput('buying');
    const r = localCompute(inp.bars, inp.period);
    expect(r.ema[r.ema.length - 1]).toBeGreaterThan(0);
});

test('demo selling: last EMA < 0', () => {
    const inp = makeDemoInput('selling');
    const r = localCompute(inp.bars, inp.period);
    expect(r.ema[r.ema.length - 1]).toBeLessThan(0);
});

test('demo zero-range: all per_bar = 0', () => {
    const inp = makeDemoInput('zero-range');
    const r = localCompute(inp.bars, inp.period);
    for (const v of r.per_bar) expect(v).toBe(0);
});

test('demo short-period uses period 5', () => {
    const inp = makeDemoInput('short-period');
    expect(inp.period).toBe(5);
});

// ── formatters ────────────────────────────────────────────────────

test('barsToBlob round-trips', () => {
    const bars = [b(101, 99, 100, 1500), b(102, 100, 101.5, 1800)];
    const back = parseBarsBlob(barsToBlob(bars));
    expect(back.errors).toEqual([]);
    expect(back.bars).toEqual(bars);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtNum(1500)).toBe('1.50k');
    expect(fmtNum(1_500_000)).toBe('1.50M');
    expect(fmtSigned(1500)).toBe('+1.50k');
    expect(fmtSigned(-1500)).toBe('-1.50k');
    expect(fmtPrice(100.456)).toBe('100.46');
    expect(fmtInt(42.9)).toBe('42');
    expect(fmtNum(NaN)).toBe('—');
});

test('DEFAULTS sanity', () => {
    expect(DEFAULT_INPUTS.bars).toEqual([]);
    expect(DEFAULT_INPUTS.period).toBe(DEFAULT_PERIOD);
    expect(DEFAULT_PERIOD).toBe(14);
    expect(MIN_PERIOD).toBe(2);
    expect(MAX_PERIOD).toBe(500);
});
