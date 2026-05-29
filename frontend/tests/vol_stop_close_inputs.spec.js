// Vol-Stop (close-based) helpers: local Rust-mirror, compare, demos.

import { test, expect } from 'vitest';
import {
    parseBarBlob, computeAtr, validateInputs, buildBody,
    localVolStopClose, localChandelier, compareStops, makeDemoBars,
} from '../js/_vol_stop_close_inputs.js';

const b = (h, l, c) => ({ high: h, low: l, close: c });

// ── Re-exported helpers still work via the proxy module ───────────

test('re-exports: parseBarBlob accepts HLC triples', () => {
    const r = parseBarBlob('100 99 99.5\n101 99 100');
    expect(r.errors).toEqual([]);
    expect(r.bars.length).toBe(2);
});

test('re-exports: validateInputs accepts compatible inputs', () => {
    const bars = [b(100, 99, 99.5), b(101, 100, 100.5)];
    const atr = [0.5, 0.5];
    expect(validateInputs(bars, atr, 'long', { lookback: 2, atr_multiplier: 2 })).toBe(null);
});

test('re-exports: buildBody emits backend ChandelierBody shape', () => {
    const bars = [b(100, 99, 99.5)];
    const body = buildBody(bars, [0.5], 'short', { lookback: 1, atr_multiplier: 3 });
    expect(body).toEqual({ bars, atr: [0.5], side: 'short', config: { lookback: 1, atr_multiplier: 3 }});
});

// ── localVolStopClose mirror ──────────────────────────────────────

test('localVolStopClose: long uses highest CLOSE in window', () => {
    // 5 bars, lookback 5, ATR=1, mult=2. Highest CLOSE = 104. Stop = 104 - 2 = 102.
    const bars = [b(101, 99, 100), b(102, 100, 101), b(105, 102, 104), b(103, 100, 101), b(104, 101, 102)];
    const atr = [1, 1, 1, 1, 1];
    const out = localVolStopClose(bars, atr, 'long', { lookback: 5, atr_multiplier: 2 });
    expect(out[4].stop_price).toBeCloseTo(102, 9);
});

test('localVolStopClose: short uses lowest CLOSE in window', () => {
    const bars = [b(100, 99, 99.5), b(99, 98, 98.5), b(98, 95, 96), b(99, 96, 97), b(100, 97, 98)];
    const atr = [1, 1, 1, 1, 1];
    const out = localVolStopClose(bars, atr, 'short', { lookback: 5, atr_multiplier: 2 });
    // Lowest close = 96, stop = 96 + 2 = 98.
    expect(out[4].stop_price).toBeCloseTo(98, 9);
});

test('localVolStopClose: warmup bars (i < lookback-1) are zeroed', () => {
    const bars = Array.from({ length: 5 }, () => b(100, 99, 99.5));
    const atr  = [1, 1, 1, 1, 1];
    const out  = localVolStopClose(bars, atr, 'long', { lookback: 5, atr_multiplier: 2 });
    for (let i = 0; i < 4; i++) {
        expect(out[i].stop_price).toBe(0);
        expect(out[i].triggered).toBe(false);
    }
});

test('localVolStopClose: long triggered when close ≤ stop', () => {
    // Highest close = 100 over 3 bars, ATR=1, mult=2 → stop=98. Bar 4 close = 97 → triggered.
    const bars = [b(101, 99, 100), b(101, 99, 100), b(101, 99, 100), b(101, 96, 97)];
    const atr = [1, 1, 1, 1];
    const out = localVolStopClose(bars, atr, 'long', { lookback: 3, atr_multiplier: 2 });
    expect(out[3].triggered).toBe(true);
});

test('localVolStopClose: short triggered when close ≥ stop', () => {
    const bars = [b(100, 98, 99), b(100, 98, 99), b(100, 98, 99), b(105, 100, 104)];
    const atr = [1, 1, 1, 1];
    // Lowest close in last 3 (idx 1..3 inclusive): min(99, 99, 104) = 99, stop = 99+2 = 101. Close=104 ≥ 101 → triggered.
    const out = localVolStopClose(bars, atr, 'short', { lookback: 3, atr_multiplier: 2 });
    expect(out[3].triggered).toBe(true);
});

test('localVolStopClose: ATR length mismatch returns all-warmup', () => {
    const bars = Array.from({ length: 10 }, () => b(100, 99, 99.5));
    const atr  = [1, 1, 1];
    const out  = localVolStopClose(bars, atr, 'long', { lookback: 5, atr_multiplier: 2 });
    expect(out.every(s => s.stop_price === 0 && !s.triggered)).toBe(true);
});

test('localVolStopClose: lookback=0 returns all-warmup (degenerate)', () => {
    const bars = Array.from({ length: 5 }, () => b(100, 99, 99.5));
    const atr  = [1, 1, 1, 1, 1];
    const out  = localVolStopClose(bars, atr, 'long', { lookback: 0, atr_multiplier: 2 });
    expect(out.every(s => s.stop_price === 0 && !s.triggered)).toBe(true);
});

// ── localChandelier mirror (the comparison line) ──────────────────

test('localChandelier: long uses highest HIGH (not close)', () => {
    const bars = [b(110, 99, 100), b(105, 99, 100), b(105, 99, 100), b(105, 99, 100), b(105, 99, 100)];
    const atr  = [1, 1, 1, 1, 1];
    const out  = localChandelier(bars, atr, 'long', { lookback: 5, atr_multiplier: 2 });
    // Highest high = 110, stop = 110-2 = 108.
    expect(out[4].stop_price).toBeCloseTo(108, 9);
});

test('localChandelier: long triggered uses LOW (wick down), not close', () => {
    const bars = [b(105, 104, 105), b(105, 104, 105), b(105, 104, 105), b(105, 100, 102), b(105, 100, 102)];
    const atr  = [1, 1, 1, 1, 1];
    const out  = localChandelier(bars, atr, 'long', { lookback: 5, atr_multiplier: 2 });
    // Highest high = 105, stop = 103. Bar 4 low = 100 ≤ 103 → triggered.
    expect(out[4].triggered).toBe(true);
});

// ── Wick-spike: the cause for vol_stop_close existing ─────────────

test('wick-spike demo: chandelier stop > close-based stop on the spike bar', () => {
    const bars = makeDemoBars('wicks');
    const atr  = computeAtr(bars, 14).map(v => Number.isFinite(v) ? v : 0);
    const cfg  = { lookback: 22, atr_multiplier: 3 };
    const chand = localChandelier(bars, atr, 'long', cfg);
    const close = localVolStopClose(bars, atr, 'long', cfg);
    // Spike is at bar idx 35. By bar 35 (where window covers bars 14..35),
    // chandelier sees the wick high; close-based does not.
    const idx = 35;
    expect(chand[idx].stop_price).toBeGreaterThan(close[idx].stop_price);
});

test('wick-spike demo: close-based fires fewer triggers than chandelier on wicks', () => {
    // Bars where the wicks pierce a hypothetical long stop but closes hold.
    const bars = [];
    for (let i = 0; i < 30; i++) bars.push(b(105, 104, 104.5));   // setup
    for (let i = 0; i < 10; i++) bars.push(b(106, 95, 104));      // wicks down, closes stay
    const atr = bars.map(() => 1);
    const cfg = { lookback: 22, atr_multiplier: 2 };
    const chand = localChandelier(bars, atr, 'long', cfg);
    const close = localVolStopClose(bars, atr, 'long', cfg);
    const chandTrigs = chand.filter(s => s.triggered).length;
    const closeTrigs = close.filter(s => s.triggered).length;
    expect(chandTrigs).toBeGreaterThan(closeTrigs);
});

// ── compareStops aggregation ──────────────────────────────────────

test('compareStops: returns NaNs on empty / mismatched arrays', () => {
    expect(compareStops([], []).diff).toBeNaN();
    expect(compareStops(null, null).chandLatest).toBeNaN();
});

test('compareStops: ignores warmup bars in agreement counting', () => {
    const c = [
        { stop_price: 0, triggered: false },
        { stop_price: 100, triggered: false },
        { stop_price: 99, triggered: true },
    ];
    const k = [
        { stop_price: 0, triggered: false },
        { stop_price: 98, triggered: false },
        { stop_price: 97, triggered: true },
    ];
    const r = compareStops(c, k);
    expect(r.agreement).toBe(2);
    expect(r.disagreement).toBe(0);
    expect(r.chandTriggers).toBe(1);
    expect(r.closeTriggers).toBe(1);
});

test('compareStops: latest non-warmup stops + spread + spread pct', () => {
    const c = [{ stop_price: 0, triggered: false }, { stop_price: 100, triggered: false }];
    const k = [{ stop_price: 0, triggered: false }, { stop_price: 98, triggered: false }];
    const r = compareStops(c, k);
    expect(r.chandLatest).toBe(100);
    expect(r.closeLatest).toBe(98);
    expect(r.diff).toBeCloseTo(2, 9);
    expect(r.diffPct).toBeCloseTo(2 / 98, 9);
});

// ── makeDemoBars invariants ───────────────────────────────────────

test('makeDemoBars: each demo emits exactly 60 valid bars', () => {
    for (const k of ['wicks', 'uptrend-reverse', 'downtrend', 'chop']) {
        const bars = makeDemoBars(k);
        expect(bars.length).toBe(60);
        for (const bar of bars) {
            expect(bar.high).toBeGreaterThanOrEqual(bar.low);
            expect(bar.close).toBeGreaterThanOrEqual(bar.low - 1e-9);
            expect(bar.close).toBeLessThanOrEqual(bar.high + 1e-9);
        }
    }
});

test('makeDemoBars: wicks preset puts a much-larger upper wick on bar 35', () => {
    const bars = makeDemoBars('wicks');
    const wickSize = bars[35].high - bars[35].close;
    const neighbourWick = bars[34].high - bars[34].close;
    expect(wickSize).toBeGreaterThan(neighbourWick * 10);
});
