// Chandelier Stop helpers: bar parser, ATR, validator, body shape,
// stop-series splitter, trigger markers, summarize, demo, formatters.

import { test, expect } from 'vitest';
import {
    parseBarBlob, trueRange, computeAtr, validateInputs, buildBody,
    splitStops, triggerMarkers, summarize,
    makeDemoBars, fmtN, fmtPct,
} from '../js/_chandelier_stop_inputs.js';

// ── parseBarBlob ──────────────────────────────────────────────────

test('parseBarBlob accepts whitespace + commas + comments', () => {
    const r = parseBarBlob('# h\n100.5 99.5 100.0\n101, 100, 100.5');
    expect(r.errors).toEqual([]);
    expect(r.bars).toEqual([
        { high: 100.5, low: 99.5, close: 100.0 },
        { high: 101, low: 100, close: 100.5 },
    ]);
});

test('parseBarBlob rejects wrong token count + non-positive HLC + low>high + close-outside', () => {
    expect(parseBarBlob('100').errors[0].message).toMatch(/expected 3 tokens/);
    expect(parseBarBlob('0 1 1').errors[0].message).toMatch(/HLC/);
    expect(parseBarBlob('99 100 99').errors[0].message).toMatch(/low > high/);
    expect(parseBarBlob('100 99 105').errors[0].message).toMatch(/close outside/);
});

test('parseBarBlob non-string returns 1 error', () => {
    expect(parseBarBlob(null).errors.length).toBe(1);
});

// ── trueRange + ATR ──────────────────────────────────────────────

test('trueRange first bar = H-L; subsequent uses max-of-three', () => {
    const bars = [
        { high: 102, low: 98, close: 100 },
        { high: 105, low: 99, close: 104 },   // H-L=6, |105-100|=5, |99-100|=1 → 6
    ];
    expect(trueRange(bars)[0]).toBe(4);
    expect(trueRange(bars)[1]).toBe(6);
});

test('computeAtr Wilder smoothing with known values', () => {
    const bars = [
        { high: 110, low: 100, close: 105 },  // TR0 = 10
        { high: 112, low: 103, close: 108 },  // TR1 = max(9,7,2) = 9
        { high: 110, low: 105, close: 109 },  // TR2 = max(5,2,3) = 5
    ];
    const atr = computeAtr(bars, 2);
    expect(atr[1]).toBeCloseTo(9.5, 10);    // SMA(10,9) = 9.5
    expect(atr[2]).toBeCloseTo(7.25, 10);   // (9.5×1 + 5)/2 = 7.25
});

test('computeAtr invalid period returns empty', () => {
    expect(computeAtr([{ high: 1, low: 1, close: 1 }], 0)).toEqual([]);
});

// ── validateInputs ────────────────────────────────────────────────

const okBars = Array(30).fill({ high: 101, low: 99, close: 100 });
const okAtr  = Array(30).fill(2);
const okCfg  = { lookback: 22, atr_multiplier: 3.0 };

test('validate accepts good inputs', () => {
    expect(validateInputs(okBars, okAtr, 'long', okCfg)).toBe(null);
});

test('validate rejects empty + atr-length mismatch', () => {
    expect(validateInputs([], okAtr, 'long', okCfg)).toMatch(/at least 1 bar/);
    expect(validateInputs(okBars, [1], 'long', okCfg)).toMatch(/atr length/);
});

test('validate rejects bad side + bad config', () => {
    expect(validateInputs(okBars, okAtr, 'flat', okCfg)).toMatch(/side/);
    expect(validateInputs(okBars, okAtr, 'long', { ...okCfg, lookback: 0 })).toMatch(/lookback/);
    expect(validateInputs(okBars, okAtr, 'long', { ...okCfg, atr_multiplier: 0 })).toMatch(/atr_multiplier/);
});

test('validate rejects fewer bars than lookback', () => {
    expect(validateInputs(Array(10).fill({ high: 101, low: 99, close: 100 }),
        Array(10).fill(2), 'long', okCfg)).toMatch(/at least 22 bars/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody emits backend ChandelierBody shape', () => {
    expect(buildBody(okBars.slice(0, 2), [1, 2], 'long', okCfg)).toEqual({
        bars: okBars.slice(0, 2), atr: [1, 2], side: 'long', config: okCfg,
    });
});

// ── splitStops ───────────────────────────────────────────────────

test('splitStops treats stop_price=0 + triggered=false as warmup → null', () => {
    const { stopPrice, triggers } = splitStops([
        { stop_price: 0,    triggered: false },  // warmup
        { stop_price: 95,   triggered: false },
        { stop_price: 96,   triggered: true },
        { stop_price: 0,    triggered: true },   // stopped right at 0 — kept (triggered)
    ]);
    expect(stopPrice).toEqual([null, 95, 96, 0]);
    expect(triggers).toEqual([false, false, true, true]);
});

test('splitStops non-array safe', () => {
    expect(splitStops(null)).toEqual({ stopPrice: [], triggers: [] });
});

// ── triggerMarkers ───────────────────────────────────────────────

test('triggerMarkers anchors at bar close where trigger fired', () => {
    const bars = [
        { high: 101, low: 99, close: 100 },
        { high: 102, low: 100, close: 101 },
        { high: 100, low: 95, close: 96 },
    ];
    const stops = [
        { stop_price: 95, triggered: false },
        { stop_price: 96, triggered: false },
        { stop_price: 96, triggered: true },
    ];
    const m = triggerMarkers(stops, bars);
    expect(m).toEqual([null, null, 96]);
});

test('triggerMarkers handles non-array safely', () => {
    expect(triggerMarkers(null, [])).toEqual([]);
});

// ── summarize ────────────────────────────────────────────────────

test('summarize: latest stop, distance %, trigger count, first trigger', () => {
    const bars = [
        { high: 101, low: 99, close: 100 },
        { high: 102, low: 100, close: 101 },
        { high: 103, low: 101, close: 102 },
    ];
    const stops = [
        { stop_price: 0,    triggered: false },
        { stop_price: 95,   triggered: false },
        { stop_price: 98,   triggered: false },
    ];
    const s = summarize(stops, bars, 'long');
    expect(s.latestClose).toBe(102);
    expect(s.latestStop).toBe(98);
    expect(s.distancePct).toBeCloseTo(4 / 102, 8);
    expect(s.triggerCount).toBe(0);
    expect(s.firstTriggerIdx).toBe(-1);
});

test('summarize: counts triggers + finds first', () => {
    const bars = [
        { high: 1, low: 1, close: 100 },
        { high: 1, low: 1, close: 99 },
        { high: 1, low: 1, close: 98 },
    ];
    const stops = [
        { stop_price: 99, triggered: false },
        { stop_price: 99, triggered: true },
        { stop_price: 99, triggered: true },
    ];
    const s = summarize(stops, bars, 'long');
    expect(s.triggerCount).toBe(2);
    expect(s.firstTriggerIdx).toBe(1);
});

test('summarize: short-side distance flips sign convention', () => {
    const bars = [{ high: 1, low: 1, close: 100 }];
    const stops = [{ stop_price: 105, triggered: false }];
    const s = summarize(stops, bars, 'short');
    // For short: distance = (stop − close) / close = 5/100 = 0.05
    expect(s.distancePct).toBeCloseTo(0.05, 10);
});

test('summarize empty-input safe', () => {
    const s = summarize([], [], 'long');
    expect(Number.isNaN(s.latestClose)).toBe(true);
    expect(s.triggerCount).toBe(0);
});

// ── makeDemoBars ─────────────────────────────────────────────────

test('makeDemoBars: 60 bars, 40 up + 20 down', () => {
    const b = makeDemoBars();
    expect(b.length).toBe(60);
    // First 40 should have monotonically rising closes.
    for (let i = 1; i < 40; i++) expect(b[i].close).toBeGreaterThan(b[i - 1].close);
    // Last 20 should be monotonically falling.
    for (let i = 41; i < 60; i++) expect(b[i].close).toBeLessThan(b[i - 1].close);
});

test('makeDemoBars: per-bar HLC validity invariant', () => {
    const b = makeDemoBars();
    expect(b.every(x =>
        x.low <= x.high && x.close >= x.low && x.close <= x.high && x.high > 0
    )).toBe(true);
});

// ── Formatters ───────────────────────────────────────────────────

test('formatters', () => {
    expect(fmtN(123.456)).toBe('123.46');
    expect(fmtN(NaN)).toBe('—');
    expect(fmtPct(0.0234)).toBe('2.34%');
    expect(fmtPct(NaN)).toBe('—');
});
