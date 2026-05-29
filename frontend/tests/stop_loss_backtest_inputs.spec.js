// Stop-loss-backtest helpers: parser, validator, body shape, localSimulate
// Rust-mirror, stopPriceFor (long + short), methodBadge, demos.

import { test, expect } from 'vitest';
import {
    METHODS, DEFAULT_PARAMS,
    parseTradeBlob, validateInputs, buildBody, localSimulate, stopPriceFor,
    methodBadge, methodLabelKey, makeDemoTrades, makeDemoParams,
    fmtN, fmtSigned, fmtPct,
} from '../js/_stop_loss_backtest_inputs.js';

const T = (e, mae, mfe, exit) => ({ entry: e, mae, mfe, actual_exit: exit });

// ── constants ─────────────────────────────────────────────────────

test('METHODS exposes the four Rust enum values', () => {
    expect(METHODS).toEqual(['none', 'fixed_dollar', 'fixed_pct', 'atr_multiple']);
});

// ── parser ────────────────────────────────────────────────────────

test('parseTradeBlob: 4 tokens, comments stripped, finite checked', () => {
    const r = parseTradeBlob('100 3 8 106\n# note\n102 5 4 99');
    expect(r.errors).toEqual([]);
    expect(r.trades).toEqual([T(100, 3, 8, 106), T(102, 5, 4, 99)]);
});

test('parseTradeBlob: rejects bad token count / non-finite / non-positive entry / negative mae/mfe', () => {
    expect(parseTradeBlob('100 3 8').errors[0].message).toMatch(/4 tokens/);
    expect(parseTradeBlob('100 3 abc 106').errors[0].message).toMatch(/finite/);
    expect(parseTradeBlob('0 3 8 106').errors[0].message).toMatch(/entry/);
    expect(parseTradeBlob('100 -1 8 106').errors[0].message).toMatch(/mae \+ mfe/);
});

test('parseTradeBlob: accepts mae=0 (perfect entry, no adverse excursion)', () => {
    expect(parseTradeBlob('100 0 5 105').errors).toEqual([]);
});

test('parseTradeBlob: non-string returns 1 error', () => {
    expect(parseTradeBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts good inputs', () => {
    expect(validateInputs([T(100, 3, 8, 106)], DEFAULT_PARAMS, true)).toBe(null);
});

test('validate rejects bad method / non-finite value / negative atr / non-boolean side', () => {
    expect(validateInputs([], { method: 'bogus', value: 0, atr: 0 }, true)).toMatch(/method/);
    expect(validateInputs([], { method: 'none', value: NaN, atr: 0 }, true)).toMatch(/value/);
    expect(validateInputs([], { method: 'none', value: 0, atr: -1 }, true)).toMatch(/atr/);
    expect(validateInputs([], DEFAULT_PARAMS, 1)).toMatch(/side_long/);
});

test('validate rejects fixed_pct value outside [0, 1]', () => {
    expect(validateInputs([], { method: 'fixed_pct', value: 1.5, atr: 0 }, true)).toMatch(/fixed_pct/);
    expect(validateInputs([], { method: 'fixed_pct', value: -0.1, atr: 0 }, true)).toMatch(/fixed_pct/);
});

test('validate rejects negative fixed_dollar value', () => {
    expect(validateInputs([], { method: 'fixed_dollar', value: -1, atr: 0 }, true)).toMatch(/fixed_dollar/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody preserves all fields (no Decimal stringification — f64s)', () => {
    const body = buildBody([T(100, 3, 8, 106)], { method: 'fixed_pct', value: 0.02, atr: 0 }, true);
    expect(body).toEqual({
        trades: [{ entry: 100, mae: 3, mfe: 8, actual_exit: 106 }],
        params: { method: 'fixed_pct', value: 0.02, atr: 0 },
        side_long: true,
    });
});

// ── stopPriceFor (long + short) ───────────────────────────────────

test('stopPriceFor: none → ±Infinity (never hits)', () => {
    expect(stopPriceFor(T(100, 0, 0, 100), { method: 'none', value: 0, atr: 0 }, true)).toBe(-Infinity);
    expect(stopPriceFor(T(100, 0, 0, 100), { method: 'none', value: 0, atr: 0 }, false)).toBe(Infinity);
});

test('stopPriceFor: fixed_dollar long = entry - value, short = entry + value', () => {
    expect(stopPriceFor(T(100, 0, 0, 100), { method: 'fixed_dollar', value: 5, atr: 0 }, true)).toBe(95);
    expect(stopPriceFor(T(100, 0, 0, 100), { method: 'fixed_dollar', value: 5, atr: 0 }, false)).toBe(105);
});

test('stopPriceFor: fixed_pct long = entry × (1 − v), short = entry × (1 + v)', () => {
    expect(stopPriceFor(T(100, 0, 0, 100), { method: 'fixed_pct', value: 0.02, atr: 0 }, true)).toBeCloseTo(98, 9);
    expect(stopPriceFor(T(100, 0, 0, 100), { method: 'fixed_pct', value: 0.02, atr: 0 }, false)).toBeCloseTo(102, 9);
});

test('stopPriceFor: atr_multiple long = entry − N×ATR, short = entry + N×ATR', () => {
    expect(stopPriceFor(T(100, 0, 0, 100), { method: 'atr_multiple', value: 2, atr: 1.5 }, true)).toBe(97);
    expect(stopPriceFor(T(100, 0, 0, 100), { method: 'atr_multiple', value: 2, atr: 1.5 }, false)).toBe(103);
});

// ── localSimulate parity (one test per Rust property) ────────────

test('local: empty trades → zeroed result', () => {
    const r = localSimulate([], { method: 'none', value: 0, atr: 0 }, true);
    expect(r.total_realized).toBe(0);
    expect(r.avg_realized).toBe(0);
});

test('local: no stop uses actual_exit (entry 100, exit 110 → +10)', () => {
    const r = localSimulate([T(100, 5, 20, 110)], { method: 'none', value: 0, atr: 0 }, true);
    expect(r.total_realized).toBe(10);
    expect(r.winning_trades).toBe(1);
    expect(r.stopped_out_count).toBe(0);
});

test('local: tight stop hit when MAE breaches (fixed_pct 1%, MAE 3 on $100 → hit at $99, realized -1)', () => {
    const r = localSimulate([T(100, 3, 20, 110)], { method: 'fixed_pct', value: 0.01, atr: 0 }, true);
    expect(r.stopped_out_count).toBe(1);
    expect(r.total_realized).toBeCloseTo(-1, 9);
});

test('local: loose stop NOT hit (5% on $100 stops at $95, MAE only 3 → no hit)', () => {
    const r = localSimulate([T(100, 3, 20, 110)], { method: 'fixed_pct', value: 0.05, atr: 0 }, true);
    expect(r.stopped_out_count).toBe(0);
    expect(r.total_realized).toBe(10);
});

test('local: short side flips MAE comparison (entry 100, MAE 5 → adverse to 105, short stop at 102 → hit)', () => {
    const r = localSimulate([T(100, 5, 20, 90)], { method: 'fixed_pct', value: 0.02, atr: 0 }, false);
    expect(r.stopped_out_count).toBe(1);
    expect(r.total_realized).toBeCloseTo(-2, 9);
});

test('local: short side wins when actual_exit < entry and no stop hit', () => {
    const r = localSimulate([T(100, 1, 10, 90)], { method: 'none', value: 0, atr: 0 }, false);
    expect(r.total_realized).toBe(10);  // entry - exit = 100 - 90
    expect(r.winning_trades).toBe(1);
});

test('local: atr_multiple stop computed from value × atr (2 × 1.5 = 3 below entry)', () => {
    // MAE 4 on $100 → adverse to 96; stop at 97 → hit at 97 (realized -3).
    const r = localSimulate([T(100, 4, 10, 105)], { method: 'atr_multiple', value: 2, atr: 1.5 }, true);
    expect(r.stopped_out_count).toBe(1);
    expect(r.total_realized).toBeCloseTo(-3, 9);
});

test('local: avg_realized = total / n', () => {
    const trades = [T(100, 0, 5, 105), T(100, 0, 5, 110)];
    const r = localSimulate(trades, { method: 'none', value: 0, atr: 0 }, true);
    expect(r.total_realized).toBe(15);
    expect(r.avg_realized).toBe(7.5);
});

test('local: report echoes method + value', () => {
    const r = localSimulate([T(100, 0, 5, 105)], { method: 'fixed_pct', value: 0.02, atr: 0 }, true);
    expect(r.method).toBe('fixed_pct');
    expect(r.value).toBe(0.02);
});

// ── methodBadge ───────────────────────────────────────────────────

test('methodBadge: empty trades → empty badge', () => {
    expect(methodBadge({ avg_realized: 0, winning_trades: 0 }, 0).key).toMatch(/empty/);
});

test('methodBadge: avg > 0 + win% ≥ 50 → profitable', () => {
    expect(methodBadge({ avg_realized: 5, winning_trades: 6 }, 10).key).toMatch(/profitable/);
});

test('methodBadge: avg > 0 but win% < 50 → marginal', () => {
    expect(methodBadge({ avg_realized: 1, winning_trades: 3 }, 10).key).toMatch(/marginal/);
});

test('methodBadge: avg < 0 + win% < 30 → disastrous', () => {
    expect(methodBadge({ avg_realized: -3, winning_trades: 2 }, 10).key).toMatch(/disastrous/);
});

test('methodBadge: avg < 0 + reasonable win% → losing', () => {
    expect(methodBadge({ avg_realized: -1, winning_trades: 4 }, 10).key).toMatch(/losing/);
});

// ── demos / presets ──────────────────────────────────────────────

test('demos: each preset returns ≥ 4 trades with valid shape', () => {
    for (const k of ['mixed', 'high-mae', 'low-mae', 'short-only', 'all-losers', 'all-winners']) {
        const trades = makeDemoTrades(k);
        expect(trades.length).toBeGreaterThanOrEqual(4);
        for (const tr of trades) {
            expect(tr.entry).toBeGreaterThan(0);
            expect(tr.mae).toBeGreaterThanOrEqual(0);
            expect(tr.mfe).toBeGreaterThanOrEqual(0);
        }
    }
});

test('demo high-mae: tight 2% stop hits every trade (stops_hit = n)', () => {
    const trades = makeDemoTrades('high-mae');
    const r = localSimulate(trades, { method: 'fixed_pct', value: 0.02, atr: 0 }, true);
    expect(r.stopped_out_count).toBe(trades.length);
});

test('demo low-mae: tight 2% stop hits zero (MAE all < 2%)', () => {
    const trades = makeDemoTrades('low-mae');
    const r = localSimulate(trades, { method: 'fixed_pct', value: 0.02, atr: 0 }, true);
    expect(r.stopped_out_count).toBe(0);
});

test('demo all-winners + no stop: every trade wins', () => {
    const trades = makeDemoTrades('all-winners');
    const r = localSimulate(trades, { method: 'none', value: 0, atr: 0 }, true);
    expect(r.winning_trades).toBe(trades.length);
    expect(r.total_realized).toBeGreaterThan(0);
});

test('preset values match expected method', () => {
    expect(makeDemoParams('none').method).toBe('none');
    expect(makeDemoParams('tight-pct')).toEqual({ method: 'fixed_pct', value: 0.02, atr: 0 });
    expect(makeDemoParams('atr-2x')).toEqual({ method: 'atr_multiple', value: 2, atr: 1.5 });
});

// ── methodLabelKey / formatters ───────────────────────────────────

test('methodLabelKey returns view.stop_loss_backtest.method.<m>', () => {
    expect(methodLabelKey('fixed_pct')).toBe('view.stop_loss_backtest.method.fixed_pct');
    expect(methodLabelKey()).toBe('view.stop_loss_backtest.method.unknown');
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtN(123.456, 2)).toBe('123.46');
    expect(fmtSigned(2.5)).toBe('+2.50');
    expect(fmtSigned(-3)).toBe('-3.00');
    expect(fmtPct(0.42)).toBe('42.0%');
    expect(fmtN(NaN)).toBe('—');
});
