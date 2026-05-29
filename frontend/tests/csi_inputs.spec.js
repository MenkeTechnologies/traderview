// Candle Strength Index helpers: parser, validator, localCompute parity (EMA of body/range), badges.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, DEFAULT_PERIOD, MIN_PERIOD, MAX_PERIOD,
    parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute,
    strengthBadge, trendBadge, crossBadge, summarizeBars,
    makeDemoInput,
    fmtRatio, fmtPrice, fmtPct, fmtInt,
} from '../js/_csi_inputs.js';

const b = (o, h, l, c) => ({ open: o, high: h, low: l, close: c });

// ── parser ────────────────────────────────────────────────────────

test('parseBarsBlob: 4 tokens per line (OHLC)', () => {
    const r = parseBarsBlob('100 101 99 100.5\n# noise\n100.5 102 100 101.5');
    expect(r.errors).toEqual([]);
    expect(r.bars).toEqual([b(100, 101, 99, 100.5), b(100.5, 102, 100, 101.5)]);
});

test('parseBarsBlob: rejects wrong count / non-positive / low > high / close OOR / open OOR', () => {
    expect(parseBarsBlob('100 101 99').errors[0].message).toMatch(/4 tokens/);
    expect(parseBarsBlob('-1 101 99 100').errors[0].message).toMatch(/OHLC/);
    expect(parseBarsBlob('100 99 101 100').errors[0].message).toMatch(/low > high/);
    expect(parseBarsBlob('100 101 99 50').errors[0].message).toMatch(/close outside/);
    expect(parseBarsBlob('50 101 99 100').errors[0].message).toMatch(/open outside/);
});

test('parseBarsBlob: non-string returns 1 error', () => {
    expect(parseBarsBlob(undefined).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts valid', () => {
    const bars = Array.from({ length: 20 }, () => b(100, 101, 99, 100.5));
    expect(validateInputs({ bars, period: 14 })).toBe(null);
});

test('validate rejects: bad array / bad period / too short / non-finite / inverted / OHL violations', () => {
    const ok = Array.from({ length: 20 }, () => b(100, 101, 99, 100.5));
    expect(validateInputs({ bars: 'no', period: 14 })).toMatch(/bars/);
    expect(validateInputs({ bars: ok, period: 1 })).toMatch(/period/);
    expect(validateInputs({ bars: ok, period: 9999 })).toMatch(/period/);
    expect(validateInputs({ bars: ok.slice(0, 5), period: 14 })).toMatch(/period/);
    const badNan = [...ok];
    badNan[5] = { open: NaN, high: 101, low: 99, close: 100 };
    expect(validateInputs({ bars: badNan, period: 14 })).toMatch(/finite/);
    const inv = [...ok];
    inv[5] = b(100, 99, 101, 100);
    expect(validateInputs({ bars: inv, period: 14 })).toMatch(/high < low/);
    const closeOOR = [...ok];
    closeOOR[5] = b(100, 101, 99, 50);
    expect(validateInputs({ bars: closeOOR, period: 14 })).toMatch(/close outside/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody trims bars to OHLC', () => {
    const body = buildBody({ bars: [{ ...b(100, 101, 99, 100.5), x: 1 }], period: 14 });
    expect(body).toEqual({ bars: [b(100, 101, 99, 100.5)], period: 14 });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: invalid inputs return all null', () => {
    const bars = Array.from({ length: 30 }, () => b(100, 101, 99, 100.5));
    expect(localCompute(bars, 1).every(x => x === null)).toBe(true);
    expect(localCompute(bars.slice(0, 5), 14).every(x => x === null)).toBe(true);
});

test('local: NaN returns all null', () => {
    const bars = Array.from({ length: 30 }, () => b(100, 101, 99, 100.5));
    bars[5] = b(NaN, 101, 99, 100.5);
    expect(localCompute(bars, 14).every(x => x === null)).toBe(true);
});

test('local: all green marubozu yields +1', () => {
    const bars = Array.from({ length: 30 }, () => b(100, 110, 100, 110));
    const r = localCompute(bars, 14);
    for (let i = 13; i < 30; i++) expect(Math.abs(r[i] - 1)).toBeLessThan(1e-9);
});

test('local: all red marubozu yields -1', () => {
    const bars = Array.from({ length: 30 }, () => b(110, 110, 100, 100));
    const r = localCompute(bars, 14);
    for (let i = 13; i < 30; i++) expect(Math.abs(r[i] + 1)).toBeLessThan(1e-9);
});

test('local: doji bars yield 0', () => {
    const bars = Array.from({ length: 30 }, () => b(100, 101, 99, 100));
    const r = localCompute(bars, 14);
    for (let i = 13; i < 30; i++) expect(Math.abs(r[i])).toBeLessThan(1e-9);
});

test('local: alternating bars average near 0', () => {
    const bars = Array.from({ length: 30 }, (_, i) =>
        i % 2 === 0 ? b(100, 110, 100, 110) : b(110, 110, 100, 100));
    const r = localCompute(bars, 14);
    expect(Math.abs(r[29])).toBeLessThan(0.2);
});

test('local: output in [-1, +1] for noise input', () => {
    let stateBig = 42n;
    const MASK = 0xFFFFFFFFFFFFFFFFn;
    const bars = Array.from({ length: 200 }, () => {
        stateBig = (stateBig * 6364136223846793005n + 1442695040888963407n) & MASK;
        const u = Number(stateBig >> 32n) / 0xFFFFFFFF;
        const m = 100 + (u - 0.5) * 4;
        return b(m, m + 1, m - 1, m + (u - 0.5) * 0.5);
    });
    const r = localCompute(bars, 14);
    for (const v of r) {
        if (v == null) continue;
        expect(v).toBeGreaterThanOrEqual(-1);
        expect(v).toBeLessThanOrEqual(1);
    }
});

test('local: output length matches input', () => {
    const bars = Array.from({ length: 30 }, () => b(100, 101, 99, 100.5));
    expect(localCompute(bars, 14).length).toBe(30);
});

test('local: leading nulls until period', () => {
    const bars = Array.from({ length: 30 }, () => b(100, 101, 99, 100.5));
    const r = localCompute(bars, 14);
    for (let i = 0; i < 13; i++) expect(r[i]).toBe(null);
    expect(r[13]).not.toBe(null);
});

test('local: deterministic', () => {
    const bars = Array.from({ length: 30 }, (_, i) => b(100 + i * 0.1, 101 + i * 0.1, 99 + i * 0.1, 100.5 + i * 0.1));
    expect(localCompute(bars, 14)).toEqual(localCompute(bars, 14));
});

test('local: zero-range bar contributes 0 to raw', () => {
    const bars = Array.from({ length: 20 }, () => b(100, 100, 100, 100));
    const r = localCompute(bars, 14);
    for (let i = 13; i < 20; i++) expect(r[i]).toBe(0);
});

// ── badges ────────────────────────────────────────────────────────

test('strengthBadge: 7 tiers', () => {
    expect(strengthBadge(0.9).key).toMatch(/marubozu_green/);
    expect(strengthBadge(0.5).key).toMatch(/strong_buy/);
    expect(strengthBadge(0.25).key).toMatch(/buy_lean/);
    expect(strengthBadge(0).key).toMatch(/indecision/);
    expect(strengthBadge(-0.25).key).toMatch(/sell_lean/);
    expect(strengthBadge(-0.5).key).toMatch(/strong_sell/);
    expect(strengthBadge(-0.9).key).toMatch(/marubozu_red/);
    expect(strengthBadge(null).key).toMatch(/unknown/);
});

test('trendBadge: tiers', () => {
    expect(trendBadge([0.5, 0.5, 0.5, 0.5, 0.5]).key).toMatch(/flat/);
    expect(trendBadge([-0.5, -0.3, -0.1, 0.1, 0.9]).key).toMatch(/rising_fast/);
    expect(trendBadge([0.9, 0.7, 0.5, 0.3, -0.5]).key).toMatch(/falling_fast/);
    expect(trendBadge([]).key).toMatch(/unknown/);
});

test('crossBadge: up_recent / down_recent / none / unknown', () => {
    expect(crossBadge([null, -0.5, -0.3, 0.2, 0.4]).key).toMatch(/up_recent/);
    expect(crossBadge([null, 0.5, 0.3, -0.2, -0.4]).key).toMatch(/down_recent/);
    expect(crossBadge([0.3, 0.4, 0.5]).key).toMatch(/none/);
});

test('crossBadge: barsAgo populated', () => {
    const r = crossBadge([null, -0.5, -0.3, 0.2, 0.4, 0.5]);
    expect(r.barsAgo).toBe(2);
});

// ── summarize ─────────────────────────────────────────────────────

test('summarizeBars: count / last_close / extrema / up/down/doji bars', () => {
    const bars = [b(100, 102, 99, 101), b(101, 103, 100, 100), b(100, 105, 99, 100)];
    const s = summarizeBars(bars);
    expect(s.count).toBe(3);
    expect(s.last_close).toBe(100);
    expect(s.min_low).toBe(99);
    expect(s.max_high).toBe(105);
    expect(s.up_bars).toBe(1);
    expect(s.down_bars).toBe(1);
    expect(s.doji_bars).toBe(1);
});

test('summarizeBars: empty → NaN + 0', () => {
    const s = summarizeBars([]);
    expect(s.count).toBe(0);
    expect(Number.isNaN(s.last_close)).toBe(true);
    expect(s.up_bars).toBe(0);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + computes', () => {
    for (const k of ['mixed','green-marubozu','red-marubozu','doji-cluster',
                     'alternating','shifting-bullish','long-period','breakout-up']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.bars, inp.period);
        expect(r.length).toBe(inp.bars.length);
    }
});

test('demo green-marubozu: last CSI ≈ 1', () => {
    const inp = makeDemoInput('green-marubozu');
    const r = localCompute(inp.bars, inp.period);
    expect(Math.abs(r[r.length - 1] - 1)).toBeLessThan(1e-9);
});

test('demo red-marubozu: last CSI ≈ -1', () => {
    const inp = makeDemoInput('red-marubozu');
    const r = localCompute(inp.bars, inp.period);
    expect(Math.abs(r[r.length - 1] + 1)).toBeLessThan(1e-9);
});

test('demo doji-cluster: CSI = 0 throughout populated', () => {
    const inp = makeDemoInput('doji-cluster');
    const r = localCompute(inp.bars, inp.period);
    for (let i = inp.period - 1; i < inp.bars.length; i++) {
        expect(Math.abs(r[i])).toBeLessThan(1e-9);
    }
});

test('demo long-period uses period=30', () => {
    const inp = makeDemoInput('long-period');
    expect(inp.period).toBe(30);
});

// ── formatters ────────────────────────────────────────────────────

test('barsToBlob round-trips', () => {
    const bars = [b(100, 101, 99, 100.5), b(100.5, 102, 100, 101.5)];
    const back = parseBarsBlob(barsToBlob(bars));
    expect(back.errors).toEqual([]);
    expect(back.bars).toEqual(bars);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtRatio(0.7654)).toBe('0.7654');
    expect(fmtPrice(100.456)).toBe('100.46');
    expect(fmtPct(0.0125)).toBe('1.25%');
    expect(fmtInt(42.9)).toBe('42');
    expect(fmtRatio(NaN)).toBe('—');
});

test('DEFAULTS sanity', () => {
    expect(DEFAULT_INPUTS.bars).toEqual([]);
    expect(DEFAULT_INPUTS.period).toBe(DEFAULT_PERIOD);
    expect(DEFAULT_PERIOD).toBe(14);
    expect(MIN_PERIOD).toBe(2);
    expect(MAX_PERIOD).toBe(500);
});
