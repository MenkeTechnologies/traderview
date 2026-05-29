// Heikin-Ashi Reversal helpers: bar parser, HA computation, validator,
// body shape, direction/strength badges, event-marker spread, demo
// invariants, formatters.

import { test, expect } from 'vitest';
import {
    parseBarBlob, computeHeikinAshi, validateInputs, buildBody,
    dirBadge, strengthBadge, eventMarkers, makeDemoBars,
    fmtN, fmtPct,
} from '../js/_ha_reversal_inputs.js';

// ── parseBarBlob ───────────────────────────────────────────────────

test('parseBarBlob accepts whitespace + commas + comments', () => {
    const r = parseBarBlob('# header\n100.5 101.2 100.0 100.85\n100.85, 101.5, 100.4, 101.3');
    expect(r.errors).toEqual([]);
    expect(r.bars).toEqual([
        { open: 100.5, high: 101.2, low: 100.0, close: 100.85 },
        { open: 100.85, high: 101.5, low: 100.4, close: 101.3 },
    ]);
});

test('parseBarBlob rejects wrong token count', () => {
    expect(parseBarBlob('100 101 100').errors[0].message).toMatch(/expected 4 tokens/);
});

test('parseBarBlob rejects non-positive OHLC', () => {
    expect(parseBarBlob('0 1 1 1').errors[0].message).toMatch(/OHLC/);
    expect(parseBarBlob('1 1 1 -1').errors[0].message).toMatch(/OHLC/);
});

test('parseBarBlob rejects low > high', () => {
    expect(parseBarBlob('100 99 100 99.5').errors[0].message).toMatch(/low > high/);
});

test('parseBarBlob rejects open or close outside [low, high]', () => {
    expect(parseBarBlob('100 101 99 102').errors[0].message).toMatch(/open \/ close outside/);
    expect(parseBarBlob('100 101 99 98').errors[0].message).toMatch(/open \/ close outside/);
});

test('parseBarBlob non-string returns 1 error', () => {
    expect(parseBarBlob(null).errors.length).toBe(1);
});

// ── computeHeikinAshi ─────────────────────────────────────────────

test('computeHeikinAshi single bar uses standard initialization', () => {
    const ha = computeHeikinAshi([{ open: 100, high: 102, low: 98, close: 101 }]);
    expect(ha.length).toBe(1);
    // HA_close = (O+H+L+C)/4 = (100+102+98+101)/4 = 100.25
    expect(ha[0].close).toBeCloseTo(100.25, 10);
    // HA_open initial = (O+C)/2 = 100.5
    expect(ha[0].open).toBeCloseTo(100.5, 10);
    // HA_high = max(102, 100.5, 100.25) = 102
    expect(ha[0].high).toBeCloseTo(102, 10);
    // HA_low = min(98, 100.5, 100.25) = 98
    expect(ha[0].low).toBeCloseTo(98, 10);
});

test('computeHeikinAshi second bar applies recursion correctly', () => {
    const bars = [
        { open: 100, high: 102, low: 98, close: 101 },     // HA_open=100.5, HA_close=100.25
        { open: 101, high: 103, low: 100, close: 102 },    // HA_open = (100.5+100.25)/2 = 100.375
                                                            // HA_close = (101+103+100+102)/4 = 101.5
    ];
    const ha = computeHeikinAshi(bars);
    expect(ha[1].open).toBeCloseTo(100.375, 10);
    expect(ha[1].close).toBeCloseTo(101.5, 10);
});

test('computeHeikinAshi empty/non-array returns empty', () => {
    expect(computeHeikinAshi([])).toEqual([]);
    expect(computeHeikinAshi(null)).toEqual([]);
});

// ── validateInputs ────────────────────────────────────────────────

const okCfg = { min_body_ratio: 0.6, strong_streak: 3, weak_streak: 2 };

test('validate accepts good inputs', () => {
    expect(validateInputs(Array(5).fill({}), okCfg)).toBe(null);
});

test('validate rejects < 2 bars', () => {
    expect(validateInputs([], okCfg)).toMatch(/at least 2 bars/);
    expect(validateInputs([{}], okCfg)).toMatch(/at least 2 bars/);
});

test('validate enforces min_body_ratio in [0, 1]', () => {
    expect(validateInputs([{}, {}], { ...okCfg, min_body_ratio: -0.1 })).toMatch(/min_body_ratio/);
    expect(validateInputs([{}, {}], { ...okCfg, min_body_ratio: 1.1 })).toMatch(/min_body_ratio/);
});

test('validate rejects non-integer / sub-1 streaks', () => {
    expect(validateInputs([{}, {}], { ...okCfg, strong_streak: 0 })).toMatch(/strong_streak/);
    expect(validateInputs([{}, {}], { ...okCfg, strong_streak: 3.5 })).toMatch(/strong_streak/);
    expect(validateInputs([{}, {}], { ...okCfg, weak_streak: 0 })).toMatch(/weak_streak/);
});

test('validate enforces weak_streak ≤ strong_streak (semantic)', () => {
    expect(validateInputs([{}, {}], { ...okCfg, weak_streak: 5 })).toMatch(/weak_streak/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody runs HA conversion before posting (backend gets HaBar[])', () => {
    const bars = [{ open: 100, high: 102, low: 98, close: 101 }];
    const body = buildBody(bars, okCfg);
    expect(body.config).toEqual(okCfg);
    expect(body.bars).toHaveLength(1);
    // Confirms HA-conversion happened by checking the canonical HA_close value.
    expect(body.bars[0].close).toBeCloseTo(100.25, 10);
});

// ── badges ────────────────────────────────────────────────────────

test('dirBadge / strengthBadge map enum + fallthrough', () => {
    expect(dirBadge('bullish_to_bearish').cls).toBe('neg');
    expect(dirBadge('bearish_to_bullish').cls).toBe('pos');
    expect(dirBadge('garbage').label).toBe('garbage');
    expect(dirBadge(null).label).toBe('—');
    expect(strengthBadge('strong').cls).toBe('pos');
    expect(strengthBadge('weak').cls).toBe('');
    expect(strengthBadge(null).label).toBe('—');
});

// ── eventMarkers ──────────────────────────────────────────────────

test('eventMarkers anchors up flips below HA low and down flips above HA high', () => {
    const haBars = [
        { open: 100, high: 102, low: 98, close: 101 },
        { open: 101, high: 103, low: 100, close: 102 },
        { open: 102, high: 104, low: 101, close: 103 },
    ];
    const events = [
        { bar_index: 1, direction: 'bearish_to_bullish' },
        { bar_index: 2, direction: 'bullish_to_bearish' },
    ];
    const { up, dn } = eventMarkers(events, haBars);
    expect(up[1]).toBeCloseTo(100 * 0.998, 10);
    expect(dn[2]).toBeCloseTo(104 * 1.002, 10);
    expect(up[0]).toBe(null);
    expect(dn[0]).toBe(null);
});

test('eventMarkers ignores out-of-bounds bar_index', () => {
    const haBars = [{ open: 100, high: 102, low: 98, close: 101 }];
    const { up, dn } = eventMarkers([{ bar_index: 5, direction: 'bearish_to_bullish' }], haBars);
    expect(up.every(v => v === null)).toBe(true);
    expect(dn.every(v => v === null)).toBe(true);
});

test('eventMarkers safe on non-array events', () => {
    const haBars = [{ open: 1, high: 1, low: 1, close: 1 }];
    expect(eventMarkers(null, haBars)).toEqual({ up: [null], dn: [null] });
});

// ── makeDemoBars ──────────────────────────────────────────────────

test('makeDemoBars deterministic for fixed seed + exactly 30 bars', () => {
    const a = makeDemoBars(42);
    const b = makeDemoBars(42);
    expect(a).toEqual(b);
    expect(a.length).toBe(30);
});

test('makeDemoBars every bar passes the parser invariants (low ≤ open/close ≤ high)', () => {
    const bars = makeDemoBars(7);
    expect(bars.every(b =>
        b.low <= b.open  + 1e-9 &&
        b.low <= b.close + 1e-9 &&
        b.high >= b.open  - 1e-9 &&
        b.high >= b.close - 1e-9 &&
        b.high >= b.low &&
        b.open > 0 && b.close > 0 && b.high > 0 && b.low > 0
    )).toBe(true);
});

test('makeDemoBars has at least 1 big BEAR body (the engineered reversal)', () => {
    const bars = makeDemoBars(1);
    const bigBear = bars.some(b => (b.open - b.close) > 4);
    expect(bigBear).toBe(true);
});

test('makeDemoBars has at least 1 big BULL body (the engineered reversal)', () => {
    const bars = makeDemoBars(1);
    const bigBull = bars.some(b => (b.close - b.open) > 4);
    expect(bigBull).toBe(true);
});

// ── formatters ────────────────────────────────────────────────────

test('formatters', () => {
    expect(fmtN(1.234)).toBe('1.23');
    expect(fmtN(NaN)).toBe('—');
    expect(fmtPct(0.78)).toBe('78.0%');
    expect(fmtPct(NaN)).toBe('—');
});
