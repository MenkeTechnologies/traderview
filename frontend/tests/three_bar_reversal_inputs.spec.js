// Three-Bar Reversal helpers: parser, validator, body shape, kind
// badge, event-marker spread, demo invariants, formatters.

import { test, expect } from 'vitest';
import {
    parseBarBlob, validateInputs, buildBody,
    kindBadge, eventMarkers, makeDemoBars, fmtN,
} from '../js/_three_bar_reversal_inputs.js';

// ── parseBarBlob ───────────────────────────────────────────────────

test('parseBarBlob accepts whitespace + commas + comments', () => {
    const r = parseBarBlob('# header\n100 101 99 100.5\n100.5, 102, 100, 101.5');
    expect(r.errors).toEqual([]);
    expect(r.bars).toEqual([
        { open: 100, high: 101, low: 99, close: 100.5 },
        { open: 100.5, high: 102, low: 100, close: 101.5 },
    ]);
});

test('parseBarBlob rejects wrong token count', () => {
    expect(parseBarBlob('100 101 99').errors[0].message).toMatch(/expected 4 tokens/);
});

test('parseBarBlob rejects non-positive OHLC', () => {
    expect(parseBarBlob('0 1 1 1').errors[0].message).toMatch(/OHLC/);
});

test('parseBarBlob rejects low > high', () => {
    expect(parseBarBlob('100 99 100 99.5').errors[0].message).toMatch(/low > high/);
});

test('parseBarBlob rejects open or close outside [low, high]', () => {
    expect(parseBarBlob('100 101 99 102').errors[0].message).toMatch(/open \/ close outside/);
});

test('parseBarBlob non-string returns 1 error', () => {
    expect(parseBarBlob(null).errors.length).toBe(1);
});

// ── validateInputs / buildBody ────────────────────────────────────

test('validate accepts ≥3 bars, rejects fewer', () => {
    expect(validateInputs(Array(3).fill({}))).toBe(null);
    expect(validateInputs(Array(2).fill({}))).toMatch(/at least 3 bars/);
});

test('buildBody emits backend ThreeBarReversalBody shape', () => {
    expect(buildBody([{ open: 100, high: 101, low: 99, close: 100.5 }]))
        .toEqual({ bars: [{ open: 100, high: 101, low: 99, close: 100.5 }] });
});

// ── kindBadge ─────────────────────────────────────────────────────

test('kindBadge maps both directions with hint', () => {
    expect(kindBadge('bullish').cls).toBe('pos');
    expect(kindBadge('bullish').hint).toMatch(/down → small → up/);
    expect(kindBadge('bearish').cls).toBe('neg');
    expect(kindBadge('bearish').hint).toMatch(/up → small → down/);
});

test('kindBadge handles unknown / null', () => {
    expect(kindBadge('garbage').label).toBe('garbage');
    expect(kindBadge(null).label).toBe('—');
});

// ── eventMarkers ──────────────────────────────────────────────────

test('eventMarkers anchors bullish below low and bearish above high', () => {
    const bars = [
        { open: 100, high: 102, low: 98, close: 101 },
        { open: 101, high: 103, low: 100, close: 102 },
        { open: 102, high: 104, low: 99, close: 103 },
    ];
    const { up, dn } = eventMarkers([
        { bar_index: 0, kind: 'bullish' },
        { bar_index: 2, kind: 'bearish' },
    ], bars);
    expect(up[0]).toBeCloseTo(98 * 0.998, 10);
    expect(dn[2]).toBeCloseTo(104 * 1.002, 10);
    expect(up[1]).toBe(null);
    expect(dn[1]).toBe(null);
});

test('eventMarkers ignores out-of-bounds bar_index', () => {
    const bars = [{ open: 1, high: 1, low: 1, close: 1 }];
    const { up, dn } = eventMarkers([{ bar_index: 5, kind: 'bullish' }], bars);
    expect(up.every(v => v === null)).toBe(true);
    expect(dn.every(v => v === null)).toBe(true);
});

test('eventMarkers safe on non-array events', () => {
    const bars = [{ open: 1, high: 1, low: 1, close: 1 }];
    expect(eventMarkers(null, bars)).toEqual({ up: [null], dn: [null] });
});

// ── makeDemoBars ──────────────────────────────────────────────────

test('makeDemoBars returns 14 bars', () => {
    expect(makeDemoBars().length).toBe(14);
});

test('makeDemoBars every bar passes parser invariants', () => {
    const bars = makeDemoBars();
    expect(bars.every(b =>
        b.low <= b.open  + 1e-9 &&
        b.low <= b.close + 1e-9 &&
        b.high >= b.open  - 1e-9 &&
        b.high >= b.close - 1e-9 &&
        b.high >= b.low &&
        b.open > 0 && b.close > 0
    )).toBe(true);
});

test('makeDemoBars contains the bullish 3-bar pattern at indices 2/3/4 (downbar → small → upbar > bar2.high)', () => {
    const b = makeDemoBars();
    const b1 = b[2], b2 = b[3], b3 = b[4];
    expect(b1.close).toBeLessThan(b1.open);        // bar 1 down
    expect(Math.abs(b2.close - b2.open))
        .toBeLessThanOrEqual(0.5 * Math.abs(b1.close - b1.open));  // small middle
    expect(b3.close).toBeGreaterThan(b1.high);     // close above bar1 high
});

test('makeDemoBars contains the bearish 3-bar pattern at indices 11/12/13 (upbar → small → downbar < bar1.low)', () => {
    const b = makeDemoBars();
    const b1 = b[11], b2 = b[12], b3 = b[13];
    expect(b1.close).toBeGreaterThan(b1.open);     // bar 1 up
    expect(Math.abs(b2.close - b2.open))
        .toBeLessThanOrEqual(0.5 * Math.abs(b1.close - b1.open));
    expect(b3.close).toBeLessThan(b1.low);         // close below bar1 low
});

// ── fmtN ──────────────────────────────────────────────────────────

test('fmtN handles non-finite + digit override', () => {
    expect(fmtN(1.234)).toBe('1.23');
    expect(fmtN(1.234, 3)).toBe('1.234');
    expect(fmtN(NaN)).toBe('—');
});
