// Cup & Handle pure helpers: OHLC bar parser, validator, body shape,
// demo data invariants, depth-quality classifier, formatters.

import { test, expect } from 'vitest';
import {
    parseBarBlob, validateInputs, buildBody,
    makeDemoBars, fmtN, fmtPct, depthQuality,
} from '../js/_cup_and_handle_inputs.js';

// ── parseBarBlob ───────────────────────────────────────────────────

test('parseBarBlob parses whitespace and comma-separated bars', () => {
    const r = parseBarBlob('100.5 99.2 100.1\n101.3, 100.0, 100.85');
    expect(r.errors).toEqual([]);
    expect(r.bars).toEqual([
        { high: 100.5, low: 99.2, close: 100.1 },
        { high: 101.3, low: 100.0, close: 100.85 },
    ]);
});

test('parseBarBlob skips # comments and blanks', () => {
    const r = parseBarBlob('# header\n\n100 99 99.5\n# end\n');
    expect(r.errors).toEqual([]);
    expect(r.bars.length).toBe(1);
});

test('parseBarBlob rejects wrong token count', () => {
    const r = parseBarBlob('100 99\n100 99 99.5 extra');
    expect(r.bars).toEqual([]);
    expect(r.errors.length).toBe(2);
});

test('parseBarBlob rejects non-positive / non-finite OHLC', () => {
    const r = parseBarBlob('100 0 99.5\n-1 99 99.5\nabc def ghi');
    expect(r.bars).toEqual([]);
    expect(r.errors.length).toBe(3);
});

test('parseBarBlob rejects low > high', () => {
    const r = parseBarBlob('99 100 99.5');
    expect(r.bars).toEqual([]);
    expect(r.errors[0].message).toMatch(/low > high/);
});

test('parseBarBlob rejects close outside [low, high]', () => {
    const r = parseBarBlob('100 99 105');
    expect(r.bars).toEqual([]);
    expect(r.errors[0].message).toMatch(/close outside/);
});

test('parseBarBlob returns error on non-string input', () => {
    const r = parseBarBlob(42);
    expect(r.bars).toEqual([]);
    expect(r.errors.length).toBe(1);
});

// ── validateInputs ─────────────────────────────────────────────────

const okCfg = {
    cup_min_bars: 30, cup_max_bars: 250,
    min_depth_pct: 0.10, max_depth_pct: 0.33, rim_tolerance_pct: 0.05,
    handle_min_bars: 5, handle_max_bars: 25, max_handle_depth_pct: 0.15,
};

test('validate accepts canonical IBD config with enough bars', () => {
    const bars = Array(50).fill({ high: 100, low: 99, close: 99.5 });
    expect(validateInputs(bars, okCfg)).toBe(null);
});

test('validate rejects cup_min_bars < 4', () => {
    expect(validateInputs([], { ...okCfg, cup_min_bars: 3 })).toMatch(/cup_min_bars/);
});

test('validate rejects cup_max ≤ cup_min', () => {
    expect(validateInputs([], { ...okCfg, cup_max_bars: 30 })).toMatch(/cup_max_bars/);
});

test('validate rejects handle bounds', () => {
    expect(validateInputs([], { ...okCfg, handle_min_bars: 0 })).toMatch(/handle_min_bars/);
    expect(validateInputs([], { ...okCfg, handle_max_bars: 4 })).toMatch(/handle_max_bars/);
});

test('validate rejects min ≥ max depth', () => {
    expect(validateInputs([], { ...okCfg, min_depth_pct: 0 })).toMatch(/min_depth_pct/);
    expect(validateInputs([], { ...okCfg, max_depth_pct: 0.10 })).toMatch(/max_depth_pct/);
});

test('validate rejects negative rim tolerance and bad handle depth', () => {
    expect(validateInputs([], { ...okCfg, rim_tolerance_pct: -0.01 })).toMatch(/rim_tolerance/);
    expect(validateInputs([], { ...okCfg, max_handle_depth_pct: 0 })).toMatch(/max_handle_depth/);
});

test('validate requires at least cup_min + handle_min bars', () => {
    const bars = Array(20).fill({ high: 100, low: 99, close: 99.5 });
    expect(validateInputs(bars, okCfg)).toMatch(/at least 35 bars/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody emits backend CupHandleBody shape', () => {
    const bars = [{ high: 100, low: 99, close: 99.5 }];
    expect(buildBody(bars, okCfg)).toEqual({ bars, config: okCfg });
});

// ── makeDemoBars ──────────────────────────────────────────────────

test('makeDemoBars is deterministic for fixed seed', () => {
    const a = makeDemoBars(7);
    const b = makeDemoBars(7);
    expect(a).toEqual(b);
});

test('makeDemoBars length matches design (30 pre + 80 cup + 12 handle = 122)', () => {
    expect(makeDemoBars(1).length).toBe(122);
});

test('makeDemoBars output respects low ≤ close ≤ high per bar', () => {
    const bars = makeDemoBars(42);
    expect(bars.every(b => b.low <= b.close && b.close <= b.high)).toBe(true);
});

test('makeDemoBars cup trough is substantially below rim (≥ 15%)', () => {
    const bars = makeDemoBars(1);
    const rim = bars[30].close;
    const cupSlice = bars.slice(30, 110);
    const trough = Math.min(...cupSlice.map(b => b.low));
    expect((rim - trough) / rim).toBeGreaterThanOrEqual(0.15);
});

// ── depthQuality ──────────────────────────────────────────────────

test('depthQuality buckets', () => {
    expect(depthQuality(0.05).label).toMatch(/shallow/);
    expect(depthQuality(0.05).cls).toBe('neg');
    expect(depthQuality(0.20).label).toMatch(/textbook/);
    expect(depthQuality(0.20).cls).toBe('pos');
    expect(depthQuality(0.45).label).toMatch(/deep/);
    expect(depthQuality(0.45).cls).toBe('neg');
    expect(depthQuality(NaN).label).toBe('—');
});

// ── formatters ─────────────────────────────────────────────────────

test('fmtN / fmtPct handle non-finite', () => {
    expect(fmtN(NaN)).toBe('—');
    expect(fmtN(12.345)).toBe('12.35');
    expect(fmtPct(NaN)).toBe('—');
    expect(fmtPct(0.123)).toBe('12.30%');
});
