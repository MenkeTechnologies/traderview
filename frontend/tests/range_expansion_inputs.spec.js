// Range Expansion helpers: bar parser, true-range + Wilder-ATR, validator,
// body shape, direction badge, event-marker spread, demo invariants,
// formatters.

import { test, expect } from 'vitest';
import {
    parseBarBlob, trueRange, computeAtr, validateInputs, buildBody,
    dirBadge, eventMarkers, makeDemoBars, fmtN,
} from '../js/_range_expansion_inputs.js';

// ── parseBarBlob ───────────────────────────────────────────────────

test('parseBarBlob accepts whitespace + commas + comments', () => {
    const r = parseBarBlob('# h\n100.5 99.5 100.0\n100.8, 99.8, 100.3');
    expect(r.errors).toEqual([]);
    expect(r.bars).toEqual([
        { high: 100.5, low: 99.5, close: 100.0 },
        { high: 100.8, low: 99.8, close: 100.3 },
    ]);
});

test('parseBarBlob rejects wrong token count', () => {
    expect(parseBarBlob('100 99').errors[0].message).toMatch(/expected 3 tokens/);
});

test('parseBarBlob rejects non-positive HLC + low>high + close outside', () => {
    expect(parseBarBlob('0 1 1').errors[0].message).toMatch(/HLC/);
    expect(parseBarBlob('99 100 99.5').errors[0].message).toMatch(/low > high/);
    expect(parseBarBlob('100 99 105').errors[0].message).toMatch(/close outside/);
});

test('parseBarBlob non-string returns 1 error', () => {
    expect(parseBarBlob(null).errors.length).toBe(1);
});

// ── trueRange ──────────────────────────────────────────────────────

test('trueRange first bar = H-L', () => {
    expect(trueRange([{ high: 102, low: 98, close: 100 }])[0]).toBe(4);
});

test('trueRange subsequent bar = max(H-L, |H-pc|, |L-pc|)', () => {
    const bars = [
        { high: 100, low: 95, close: 98 },
        { high: 102, low: 99, close: 101 },   // H-L=3, |102-98|=4, |99-98|=1 → 4
    ];
    expect(trueRange(bars)[1]).toBe(4);
});

test('trueRange empty/non-array returns empty', () => {
    expect(trueRange([])).toEqual([]);
    expect(trueRange(null)).toEqual([]);
});

// ── computeAtr (Wilder) ───────────────────────────────────────────

test('computeAtr first (period-1) entries are NaN, period-th is SMA', () => {
    const bars = Array.from({ length: 5 }, (_, i) => ({
        high: 100 + i + 1, low: 100 + i - 1, close: 100 + i,
    }));
    const atr = computeAtr(bars, 3);
    // TR for these bars (after first): each bar H-L = 2 (or similar).
    // First period-1 = 2 entries should be NaN.
    expect(atr[0]).toBeNaN();
    expect(atr[1]).toBeNaN();
    expect(Number.isFinite(atr[2])).toBe(true);   // first ATR = SMA of TR[0..2]
});

test('computeAtr recursion: ATR_t = (ATR_{t-1}*(p-1) + TR_t) / p', () => {
    // Use 3 simple bars with known TRs.
    const bars = [
        { high: 110, low: 100, close: 105 },  // TR0 = 10
        { high: 112, low: 103, close: 108 },  // TR1 = max(9, 7, 2) = 9
        { high: 110, low: 105, close: 109 },  // TR2 = max(5, 2, 3) = 5
    ];
    const atr = computeAtr(bars, 2);
    // ATR[1] = SMA(TR[0], TR[1]) = (10+9)/2 = 9.5
    // ATR[2] = (9.5 * (2-1) + 5) / 2 = 14.5 / 2 = 7.25
    expect(atr[1]).toBeCloseTo(9.5, 10);
    expect(atr[2]).toBeCloseTo(7.25, 10);
});

test('computeAtr short series returns all-NaN', () => {
    const bars = Array(5).fill({ high: 100, low: 99, close: 99.5 });
    const atr = computeAtr(bars, 10);
    expect(atr.every(Number.isNaN)).toBe(true);
});

test('computeAtr invalid period returns empty', () => {
    expect(computeAtr([{ high: 100, low: 99, close: 99.5 }], 0)).toEqual([]);
    expect(computeAtr([{ high: 100, low: 99, close: 99.5 }], 1.5)).toEqual([]);
});

// ── validateInputs ────────────────────────────────────────────────

const okCfg = { lookback: 5, min_expansion_atrs: 1.5, prior_atr_max: 0.7 };

test('validate accepts canonical', () => {
    const bars = Array(10).fill({ high: 100, low: 99, close: 99.5 });
    const atr  = Array(10).fill(1);
    expect(validateInputs(bars, atr, okCfg)).toBe(null);
});

test('validate rejects too few bars (need lookback+1)', () => {
    const bars = Array(3).fill({});
    expect(validateInputs(bars, [], okCfg)).toMatch(/at least 6 bars/);
});

test('validate rejects atr length mismatch', () => {
    const bars = Array(10).fill({});
    expect(validateInputs(bars, [], okCfg)).toMatch(/atr series length/);
});

test('validate rejects non-integer lookback', () => {
    const bars = Array(10).fill({});
    const atr  = Array(10).fill(1);
    expect(validateInputs(bars, atr, { ...okCfg, lookback: 0 })).toMatch(/lookback/);
});

test('validate rejects non-positive thresholds', () => {
    const bars = Array(10).fill({});
    const atr  = Array(10).fill(1);
    expect(validateInputs(bars, atr, { ...okCfg, min_expansion_atrs: 0 })).toMatch(/min_expansion_atrs/);
    expect(validateInputs(bars, atr, { ...okCfg, prior_atr_max: 0 })).toMatch(/prior_atr_max/);
});

test('validate enforces semantic invariant: prior_atr_max < min_expansion_atrs', () => {
    const bars = Array(10).fill({});
    const atr  = Array(10).fill(1);
    expect(validateInputs(bars, atr, { ...okCfg, prior_atr_max: 1.5 }))
        .toMatch(/prior_atr_max must be < min_expansion_atrs/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody emits backend RangeExpansionBody shape', () => {
    const bars = [{ high: 100, low: 99, close: 99.5 }];
    const atr  = [1];
    expect(buildBody(bars, atr, okCfg)).toEqual({ bars, atr, config: okCfg });
});

// ── dirBadge ──────────────────────────────────────────────────────

test('dirBadge maps up + down + fallthrough', () => {
    expect(dirBadge('up').cls).toBe('pos');
    expect(dirBadge('down').cls).toBe('neg');
    expect(dirBadge('garbage').label).toBe('garbage');
    expect(dirBadge(null).label).toBe('—');
});

// ── eventMarkers ──────────────────────────────────────────────────

test('eventMarkers anchors UP above bar.high, DOWN below bar.low', () => {
    const bars = [
        { high: 102, low: 98, close: 100 },
        { high: 105, low: 99, close: 101 },
    ];
    const { up, dn } = eventMarkers([
        { bar_index: 0, direction: 'up' },
        { bar_index: 1, direction: 'down' },
    ], bars);
    expect(up[0]).toBeCloseTo(102 * 1.002, 10);
    expect(dn[1]).toBeCloseTo(99 * 0.998, 10);
});

test('eventMarkers ignores out-of-bounds bar_index', () => {
    const bars = [{ high: 100, low: 99, close: 99.5 }];
    const { up, dn } = eventMarkers([{ bar_index: 5, direction: 'up' }], bars);
    expect(up.every(v => v === null)).toBe(true);
    expect(dn.every(v => v === null)).toBe(true);
});

// ── makeDemoBars ──────────────────────────────────────────────────

test('makeDemoBars returns 30 bars with valid HLC', () => {
    const bars = makeDemoBars();
    expect(bars.length).toBe(30);
    expect(bars.every(b => b.low <= b.high && b.close >= b.low && b.close <= b.high && b.high > 0)).toBe(true);
});

test('makeDemoBars has narrow-range compression bars around index 18-21', () => {
    const bars = makeDemoBars();
    const ranges = bars.slice(18, 22).map(b => b.high - b.low);
    expect(ranges.every(r => r <= 0.25)).toBe(true);  // tight (≤0.25)
});

test('makeDemoBars has wide-range expansion bar at index 22 (>= 2.5)', () => {
    const bars = makeDemoBars();
    const range22 = bars[22].high - bars[22].low;
    expect(range22).toBeGreaterThanOrEqual(2.5);
});

// ── fmtN ──────────────────────────────────────────────────────────

test('fmtN handles non-finite + digit override', () => {
    expect(fmtN(1.234)).toBe('1.23');
    expect(fmtN(1.234, 3)).toBe('1.234');
    expect(fmtN(NaN)).toBe('—');
});
