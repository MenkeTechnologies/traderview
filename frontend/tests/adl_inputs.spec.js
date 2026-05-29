// Accumulation/Distribution Line helpers: parser, validator,
// localCompute mirror (including NaN-carry-forward + zero-range bars), badges.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS,
    parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute,
    trendBadge, divergenceBadge, phaseBadge, summarizeBars,
    makeDemoInput,
    fmtNum, fmtSigned, fmtPrice, fmtInt,
} from '../js/_adl_inputs.js';

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
    expect(parseBarsBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts valid', () => {
    expect(validateInputs({ bars: [b(101, 99, 100, 1000)] })).toBe(null);
});

test('validate rejects: bad array / empty / non-number / inverted', () => {
    expect(validateInputs({ bars: 'no' })).toMatch(/bars/);
    expect(validateInputs({ bars: [] })).toMatch(/empty/);
    expect(validateInputs({ bars: [{ high: '101', low: 99, close: 100, volume: 1000 }] })).toMatch(/numbers/);
    expect(validateInputs({ bars: [{ high: 99, low: 101, close: 100, volume: 1000 }] })).toMatch(/high < low/);
});

test('validate accepts NaN bars (Rust impl carries ADL forward)', () => {
    expect(validateInputs({ bars: [b(NaN, 99, 100, 1000)] })).toBe(null);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody trims bars to HLCV', () => {
    const body = buildBody({ bars: [{ ...b(101, 99, 100, 1000), extra: 'x' }] });
    expect(body).toEqual({ bars: [b(101, 99, 100, 1000)] });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: empty returns empty', () => {
    expect(localCompute([])).toEqual([]);
});

test('local: closes at high → ADL = nbars × volume', () => {
    const bars = Array.from({ length: 10 }, () => b(101, 99, 101, 1000));
    const out = localCompute(bars);
    expect(Math.abs(out[9] - 10000)).toBeLessThan(1e-9);
    for (let i = 1; i < out.length; i++) {
        expect(out[i]).toBeGreaterThanOrEqual(out[i - 1]);
    }
});

test('local: closes at low → ADL = -nbars × volume', () => {
    const bars = Array.from({ length: 10 }, () => b(101, 99, 99, 1000));
    const out = localCompute(bars);
    expect(Math.abs(out[9] + 10000)).toBeLessThan(1e-9);
});

test('local: midpoint close → ADL stays at 0', () => {
    const bars = Array.from({ length: 10 }, () => b(101, 99, 100, 1000));
    const out = localCompute(bars);
    expect(Math.abs(out[9])).toBeLessThan(1e-9);
});

test('local: zero-range bar contributes 0 MFV', () => {
    const bars = Array.from({ length: 5 }, () => b(101, 99, 101, 1000));
    bars.push(b(100, 100, 100, 1000));
    const out = localCompute(bars);
    expect(Math.abs(out[5] - 5000)).toBeLessThan(1e-9);
});

test('local: NaN bar carries ADL forward unchanged', () => {
    const bars = Array.from({ length: 3 }, () => b(101, 99, 101, 1000));
    bars.push(b(NaN, 99, 100, 1000));
    const out = localCompute(bars);
    expect(out[3]).toBe(out[2]);
});

test('local: output length matches input', () => {
    const bars = Array.from({ length: 50 }, () => b(101, 99, 100, 1000));
    expect(localCompute(bars).length).toBe(50);
});

test('local: deterministic for same input', () => {
    const bars = Array.from({ length: 20 }, (_, i) => b(101 + i * 0.1, 99 + i * 0.1, 100 + i * 0.1, 1000 + i * 10));
    expect(localCompute(bars)).toEqual(localCompute(bars));
});

test('local: MFM ∈ [-1, 1] enforced by formula', () => {
    // Cap volume — any MFV must be in [-vol, +vol].
    const bars = Array.from({ length: 20 }, (_, i) => b(101, 99, 100 + (i % 3 - 1), 500));
    const out = localCompute(bars);
    // ADL deltas must each be within [-500, +500].
    for (let i = 1; i < out.length; i++) {
        expect(Math.abs(out[i] - out[i - 1])).toBeLessThanOrEqual(500 + 1e-9);
    }
});

// ── badges ────────────────────────────────────────────────────────

test('trendBadge: strong_accum / accum / flat / dist / strong_dist / unknown', () => {
    expect(trendBadge([0, 0, 0, 0, 0, 0, 0, 0, 0, 0]).key).toMatch(/flat/);
    expect(trendBadge([0, 1, 2, 3, 4, 5, 6, 7, 8, 100]).key).toMatch(/strong_accum/);
    expect(trendBadge([10, 10.5, 10.7, 10.8, 10.9, 11, 11.2, 11.3, 11.5, 11.7]).key).toMatch(/accum/);
    expect(trendBadge([100, 90, 80, 70, 60, 50, 40, 30, 20, 0]).key).toMatch(/strong_dist/);
    expect(trendBadge([10, 9.9, 9.8, 9.7, 9.6, 9.5, 9.4, 9.3, 9.2, 9.1]).key).toMatch(/dist/);
    expect(trendBadge([]).key).toMatch(/unknown/);
});

test('divergenceBadge: confirms / bullish / bearish / neutral / unknown', () => {
    const bars = Array.from({ length: 20 }, () => b(101, 99, 100, 1000));
    // ADL rising + close rising → confirms
    const adlUp = Array.from({ length: 20 }, (_, i) => i * 100);
    const barsUp = Array.from({ length: 20 }, (_, i) => b(101 + i, 99 + i, 100 + i, 1000));
    expect(divergenceBadge(adlUp, barsUp).key).toMatch(/confirms/);
    // ADL up + close down → bullish divergence
    const barsDown = Array.from({ length: 20 }, (_, i) => b(101 - i, 99 - i, 100 - i, 1000));
    expect(divergenceBadge(adlUp, barsDown).key).toMatch(/bullish/);
    // ADL down + close up → bearish divergence
    const adlDown = Array.from({ length: 20 }, (_, i) => -i * 100);
    expect(divergenceBadge(adlDown, barsUp).key).toMatch(/bearish/);
    // Both flat → neutral (deltas == 0)
    expect(divergenceBadge(new Array(20).fill(0), bars).key).toMatch(/neutral/);
    expect(divergenceBadge([], []).key).toMatch(/unknown/);
});

test('phaseBadge: accumulation / distribution / neutral / unknown', () => {
    expect(phaseBadge(100).key).toMatch(/accumulation/);
    expect(phaseBadge(-100).key).toMatch(/distribution/);
    expect(phaseBadge(0).key).toMatch(/neutral/);
    expect(phaseBadge(null).key).toMatch(/unknown/);
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
    for (const k of ['accumulation','distribution','bull-divergence','bear-divergence',
                     'sideways','climax-volume','doji-cluster','small-volume']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const out = localCompute(inp.bars);
        expect(out.length).toBe(inp.bars.length);
    }
});

test('demo accumulation: last ADL > 0', () => {
    const inp = makeDemoInput('accumulation');
    const out = localCompute(inp.bars);
    expect(out[out.length - 1]).toBeGreaterThan(0);
});

test('demo distribution: last ADL < 0', () => {
    const inp = makeDemoInput('distribution');
    const out = localCompute(inp.bars);
    expect(out[out.length - 1]).toBeLessThan(0);
});

test('demo doji-cluster: ADL stays at 0 (all range=0)', () => {
    const inp = makeDemoInput('doji-cluster');
    const out = localCompute(inp.bars);
    for (const v of out) expect(v).toBe(0);
});

test('demo bull-divergence: ADL rising, price falling', () => {
    const inp = makeDemoInput('bull-divergence');
    const out = localCompute(inp.bars);
    expect(out[out.length - 1]).toBeGreaterThan(out[0]);
    expect(inp.bars[inp.bars.length - 1].close).toBeLessThan(inp.bars[0].close);
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
    expect(fmtNum(1_500_000_000)).toBe('1.50B');
    expect(fmtNum(42)).toBe('42');
    expect(fmtSigned(1500)).toBe('+1.50k');
    expect(fmtSigned(-1500)).toBe('-1.50k');
    expect(fmtPrice(100.456)).toBe('100.46');
    expect(fmtInt(42.9)).toBe('42');
    expect(fmtNum(NaN)).toBe('—');
    expect(fmtPrice(NaN)).toBe('—');
});

test('DEFAULT_INPUTS sanity', () => {
    expect(DEFAULT_INPUTS.bars).toEqual([]);
});
