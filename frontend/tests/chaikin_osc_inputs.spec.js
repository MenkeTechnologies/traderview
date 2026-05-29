// Chaikin Oscillator helpers: parser, validator, localCompute parity (ADL + EMA diff), badges.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, DEFAULT_FAST, DEFAULT_SLOW, MIN_PERIOD, MAX_PERIOD,
    parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute, ema,
    signBadge, crossBadge, trendBadge, divergenceBadge, summarizeBars,
    makeDemoInput,
    fmtNum, fmtSigned, fmtPrice, fmtInt,
} from '../js/_chaikin_osc_inputs.js';

const b = (h, l, c, v) => ({ high: h, low: l, close: c, volume: v });

// ── parser ────────────────────────────────────────────────────────

test('parseBarsBlob: 4 tokens per line', () => {
    const r = parseBarsBlob('101 99 100 1000\n# noise\n102 100 101 1500');
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
    expect(parseBarsBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts valid', () => {
    const bars = Array.from({ length: 20 }, () => b(101, 99, 100, 1000));
    expect(validateInputs({ bars, fast: 3, slow: 10 })).toBe(null);
});

test('validate rejects: bad array / empty / bad periods / fast >= slow / short / negative vol', () => {
    const ok = Array.from({ length: 20 }, () => b(101, 99, 100, 1000));
    expect(validateInputs({ bars: 'no', fast: 3, slow: 10 })).toMatch(/bars/);
    expect(validateInputs({ bars: [], fast: 3, slow: 10 })).toMatch(/empty/);
    expect(validateInputs({ bars: ok, fast: 0, slow: 10 })).toMatch(/fast/);
    expect(validateInputs({ bars: ok, fast: 3, slow: 0 })).toMatch(/slow/);
    expect(validateInputs({ bars: ok, fast: 10, slow: 5 })).toMatch(/fast.*slow/);
    expect(validateInputs({ bars: ok, fast: 5, slow: 5 })).toMatch(/fast.*slow/);
    expect(validateInputs({ bars: ok.slice(0, 5), fast: 3, slow: 10 })).toMatch(/slow/);
    const neg = [...ok];
    neg[5] = b(101, 99, 100, -100);
    expect(validateInputs({ bars: neg, fast: 3, slow: 10 })).toMatch(/negative/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody trims bars to HLCV', () => {
    const body = buildBody({ bars: [{ ...b(101, 99, 100, 1000), x: 'y' }], fast: 3, slow: 10 });
    expect(body).toEqual({ bars: [b(101, 99, 100, 1000)], fast: 3, slow: 10 });
});

// ── EMA helper ────────────────────────────────────────────────────

test('ema: seed = SMA of first period, then standard recursion', () => {
    const series = [1, 2, 3, 4, 5, 6, 7];
    const e = ema(series, 3);
    expect(e[0]).toBe(null);
    expect(e[1]).toBe(null);
    expect(e[2]).toBeCloseTo(2, 9);     // (1+2+3)/3 = 2
    // k = 2/(3+1) = 0.5; next = 4*0.5 + 2*0.5 = 3
    expect(e[3]).toBeCloseTo(3, 9);
});

test('ema: any null in seed window → all null', () => {
    const series = [1, null, 3, 4, 5];
    const e = ema(series, 3);
    expect(e.every(x => x === null)).toBe(true);
});

test('ema: period=0 or n<period returns all null', () => {
    expect(ema([1, 2, 3], 0).every(x => x === null)).toBe(true);
    expect(ema([1, 2], 5).every(x => x === null)).toBe(true);
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: empty returns empty', () => {
    expect(localCompute([], 3, 10)).toEqual([]);
});

test('local: invalid periods return all null', () => {
    const bars = Array.from({ length: 20 }, () => b(101, 99, 100, 1000));
    expect(localCompute(bars, 0, 10).every(x => x === null)).toBe(true);
    expect(localCompute(bars, 10, 3).every(x => x === null)).toBe(true);
    expect(localCompute(bars, 5, 5).every(x => x === null)).toBe(true);
});

test('local: shorter than slow returns all null', () => {
    const bars = Array.from({ length: 5 }, () => b(101, 99, 100, 1000));
    expect(localCompute(bars, 3, 10).every(x => x === null)).toBe(true);
});

test('local: sustained accumulation yields positive CO', () => {
    const bars = Array.from({ length: 30 }, () => b(101, 99, 101, 1000));
    const r = localCompute(bars, 3, 10);
    expect(r[29]).toBeGreaterThan(0);
});

test('local: sustained distribution yields negative CO', () => {
    const bars = Array.from({ length: 30 }, () => b(101, 99, 99, 1000));
    const r = localCompute(bars, 3, 10);
    expect(r[29]).toBeLessThan(0);
});

test('local: flat midpoint yields CO ≈ 0', () => {
    const bars = Array.from({ length: 30 }, () => b(101, 99, 100, 1000));
    const r = localCompute(bars, 3, 10);
    expect(Math.abs(r[29])).toBeLessThan(1e-9);
});

test('local: output length matches input', () => {
    const bars = Array.from({ length: 50 }, () => b(101, 99, 100, 1000));
    const r = localCompute(bars, 3, 10);
    expect(r.length).toBe(50);
    expect(r[8]).toBe(null);
    expect(r[9]).not.toBe(null);
});

test('local: deterministic', () => {
    const bars = Array.from({ length: 30 }, (_, i) => b(101 + i * 0.1, 99 + i * 0.1, 100 + i * 0.1, 1000 + i * 10));
    expect(localCompute(bars, 3, 10)).toEqual(localCompute(bars, 3, 10));
});

// ── badges ────────────────────────────────────────────────────────

test('signBadge: bullish / bearish / neutral / unknown', () => {
    expect(signBadge(50).key).toMatch(/bullish/);
    expect(signBadge(-50).key).toMatch(/bearish/);
    expect(signBadge(0).key).toMatch(/neutral/);
    expect(signBadge(null).key).toMatch(/unknown/);
});

test('crossBadge: up / down / none', () => {
    expect(crossBadge([null, -50, -30, 20, 40]).key).toMatch(/up_recent/);
    expect(crossBadge([null, 50, 30, -20, -40]).key).toMatch(/down_recent/);
    expect(crossBadge([10, 20, 30]).key).toMatch(/none/);
});

test('crossBadge: barsAgo populated', () => {
    const r = crossBadge([null, -50, -30, 20, 40, 50]);
    expect(r.barsAgo).toBe(2);
});

test('trendBadge: tiers', () => {
    expect(trendBadge([0, 0, 0, 0, 0]).key).toMatch(/flat/);
    expect(trendBadge([0, 10, 20, 30, 100]).key).toMatch(/rising_fast/);
    expect(trendBadge([100, 80, 60, 40, 0]).key).toMatch(/falling_fast/);
    expect(trendBadge([]).key).toMatch(/unknown/);
});

test('divergenceBadge: confirms / bullish / bearish / neutral / unknown', () => {
    const bars = Array.from({ length: 20 }, () => b(101, 99, 100, 1000));
    const co_up = Array.from({ length: 20 }, (_, i) => i * 10);
    const bars_up = Array.from({ length: 20 }, (_, i) => b(101 + i, 99 + i, 100 + i, 1000));
    expect(divergenceBadge(co_up, bars_up).key).toMatch(/confirms/);
    const bars_down = Array.from({ length: 20 }, (_, i) => b(101 - i, 99 - i, 100 - i, 1000));
    expect(divergenceBadge(co_up, bars_down).key).toMatch(/bullish/);
    const co_down = Array.from({ length: 20 }, (_, i) => -i * 10);
    expect(divergenceBadge(co_down, bars_up).key).toMatch(/bearish/);
    expect(divergenceBadge(new Array(20).fill(0), bars).key).toMatch(/neutral/);
    expect(divergenceBadge([], []).key).toMatch(/unknown/);
});

// ── summarize ─────────────────────────────────────────────────────

test('summarizeBars: count / last_close / total_volume / mean / extrema', () => {
    const bars = [b(102, 98, 100, 500), b(105, 100, 103, 700), b(106, 101, 105, 800)];
    const s = summarizeBars(bars);
    expect(s.count).toBe(3);
    expect(s.last_close).toBe(105);
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
    for (const k of ['accumulation','distribution','sideways-neutral','bull-divergence',
                     'bear-divergence','cross-up','wide-fast-slow','flat-zero']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.bars, inp.fast, inp.slow);
        expect(r.length).toBe(inp.bars.length);
    }
});

test('demo accumulation: last CO > 0', () => {
    const inp = makeDemoInput('accumulation');
    const r = localCompute(inp.bars, inp.fast, inp.slow);
    expect(r[r.length - 1]).toBeGreaterThan(0);
});

test('demo distribution: last CO < 0', () => {
    const inp = makeDemoInput('distribution');
    const r = localCompute(inp.bars, inp.fast, inp.slow);
    expect(r[r.length - 1]).toBeLessThan(0);
});

test('demo flat-zero: CO ≈ 0 throughout populated', () => {
    const inp = makeDemoInput('flat-zero');
    const r = localCompute(inp.bars, inp.fast, inp.slow);
    for (let i = inp.slow - 1; i < inp.bars.length; i++) {
        expect(Math.abs(r[i])).toBeLessThan(1e-9);
    }
});

test('demo wide-fast-slow uses 5/20 EMAs', () => {
    const inp = makeDemoInput('wide-fast-slow');
    expect(inp.fast).toBe(5);
    expect(inp.slow).toBe(20);
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
    expect(DEFAULT_INPUTS.fast).toBe(DEFAULT_FAST);
    expect(DEFAULT_INPUTS.slow).toBe(DEFAULT_SLOW);
    expect(DEFAULT_FAST).toBe(3);
    expect(DEFAULT_SLOW).toBe(10);
    expect(MIN_PERIOD).toBe(1);
    expect(MAX_PERIOD).toBe(500);
});
