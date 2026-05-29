// Portfolio Stress-Test helpers: leg parser, validator, body shape,
// default shock ladders, grid pivot, heatmap class picker, demo
// invariants, formatters.

import { test, expect } from 'vitest';
import {
    parseLegBlob, validateInputs, buildBody,
    defaultPriceShocks, defaultIvShocks,
    pivotGrid, heatStyleClass, makeDemoLegs,
    fmtUSD, fmtUSDSigned, fmtPct,
} from '../js/_stress_test_inputs.js';

// ── parseLegBlob ──────────────────────────────────────────────────

test('parseLegBlob accepts 8-token rows', () => {
    const r = parseLegBlob('SPY put 100 95 30 0.30 -1 1.20\nSPY call 100 105 30 0.30 1 2.50');
    expect(r.errors).toEqual([]);
    expect(r.legs.length).toBe(2);
    expect(r.legs[0]).toEqual({
        symbol: 'SPY', kind: 'put', spot: 100, strike: 95,
        days_to_expiry: 30, implied_vol: 0.30,
        contracts: -1, multiplier: 100, entry_price: 1.20,
    });
});

test('parseLegBlob rejects wrong token count', () => {
    expect(parseLegBlob('SPY put 100').errors[0].message).toMatch(/expected 8 tokens/);
});

test('parseLegBlob rejects bad kind enum', () => {
    expect(parseLegBlob('SPY straddle 100 95 30 0.30 -1 1.20').errors[0].message).toMatch(/kind must be/);
});

test('parseLegBlob rejects non-positive spot / strike + negative dte / iv', () => {
    expect(parseLegBlob('SPY put 0 95 30 0.30 -1 1.20').errors[0].message).toMatch(/spot/);
    expect(parseLegBlob('SPY put 100 0 30 0.30 -1 1.20').errors[0].message).toMatch(/strike/);
    expect(parseLegBlob('SPY put 100 95 -1 0.30 -1 1.20').errors[0].message).toMatch(/days_to_expiry/);
    expect(parseLegBlob('SPY put 100 95 30 -0.1 -1 1.20').errors[0].message).toMatch(/iv/);
});

test('parseLegBlob rejects non-integer / zero contracts', () => {
    expect(parseLegBlob('SPY put 100 95 30 0.30 0 1.20').errors[0].message).toMatch(/contracts/);
    expect(parseLegBlob('SPY put 100 95 30 0.30 1.5 1.20').errors[0].message).toMatch(/contracts/);
});

test('parseLegBlob accepts zero entry_price (cash-secured fallback)', () => {
    const r = parseLegBlob('SPY put 100 95 30 0.30 -1 0');
    expect(r.errors).toEqual([]);
});

test('parseLegBlob accepts zero dte (expiry-day)', () => {
    const r = parseLegBlob('SPY put 100 95 0 0.30 -1 1.20');
    expect(r.errors).toEqual([]);
});

test('parseLegBlob non-string returns 1 error', () => {
    expect(parseLegBlob(null).errors.length).toBe(1);
});

// ── validateInputs / buildBody ───────────────────────────────────

const okLegs = [makeDemoLegs()[0]];

test('validate accepts good inputs', () => {
    expect(validateInputs(okLegs, [0], [0], 0, 0.045, 0)).toBe(null);
});

test('validate rejects empty inputs', () => {
    expect(validateInputs([], [0], [0], 0, 0.045, 0)).toMatch(/at least 1 leg/);
    expect(validateInputs(okLegs, [], [0], 0, 0.045, 0)).toMatch(/price shock/);
    expect(validateInputs(okLegs, [0], [], 0, 0.045, 0)).toMatch(/iv shock/);
});

test('validate rejects negative time-decay / non-finite rate / negative div', () => {
    expect(validateInputs(okLegs, [0], [0], -1, 0.045, 0)).toMatch(/time_decay/);
    expect(validateInputs(okLegs, [0], [0], 0, NaN, 0)).toMatch(/risk_free_rate/);
    expect(validateInputs(okLegs, [0], [0], 0, 0.045, -0.01)).toMatch(/dividend/);
});

test('validate rejects non-finite shock entries', () => {
    expect(validateInputs(okLegs, [0, NaN], [0], 0, 0.045, 0)).toMatch(/finite/);
});

test('buildBody emits backend StressInput shape (flat)', () => {
    expect(buildBody(okLegs, [0.01], [0.10], 1, 0.045, 0.02)).toEqual({
        legs: okLegs, price_shocks_pct: [0.01], iv_shocks_pct: [0.10],
        time_decay_days: 1, risk_free_rate: 0.045, dividend_yield: 0.02,
    });
});

// ── default shocks ───────────────────────────────────────────────

test('defaultPriceShocks symmetric around 0', () => {
    const ps = defaultPriceShocks();
    expect(ps.includes(0)).toBe(true);
    expect(ps[0]).toBeLessThan(0);
    expect(ps[ps.length - 1]).toBeGreaterThan(0);
});

test('defaultIvShocks symmetric around 0', () => {
    const ivs = defaultIvShocks();
    expect(ivs.includes(0)).toBe(true);
});

// ── pivotGrid ────────────────────────────────────────────────────

test('pivotGrid lays cells into [priceIdx][ivIdx] matrix', () => {
    const grid = [
        { price_shock_pct: -0.05, iv_shock_pct: -0.10, pnl_dollars: -100 },
        { price_shock_pct: -0.05, iv_shock_pct: 0,     pnl_dollars: -50 },
        { price_shock_pct: 0,     iv_shock_pct: -0.10, pnl_dollars: 200 },
        { price_shock_pct: 0,     iv_shock_pct: 0,     pnl_dollars: 300 },
    ];
    const matrix = pivotGrid(grid, [-0.05, 0], [-0.10, 0]);
    expect(matrix[0][0].pnl_dollars).toBe(-100);
    expect(matrix[0][1].pnl_dollars).toBe(-50);
    expect(matrix[1][0].pnl_dollars).toBe(200);
    expect(matrix[1][1].pnl_dollars).toBe(300);
});

test('pivotGrid handles short grid by leaving nulls', () => {
    const matrix = pivotGrid([], [-0.05, 0], [-0.10, 0]);
    expect(matrix.every(row => row.every(c => c === null))).toBe(true);
});

test('pivotGrid non-array safe', () => {
    const matrix = pivotGrid(null, [0], [0]);
    expect(matrix).toEqual([[null]]);
});

// ── heatStyleClass ───────────────────────────────────────────────

test('heatStyleClass: 4-tier intensity per side', () => {
    expect(heatStyleClass(20, 100)).toBe('heat-pos-1');     // 20% < 25
    expect(heatStyleClass(40, 100)).toBe('heat-pos-2');     // 40% < 50
    expect(heatStyleClass(60, 100)).toBe('heat-pos-3');     // 60% < 75
    expect(heatStyleClass(90, 100)).toBe('heat-pos-4');     // 90% ≥ 75
    expect(heatStyleClass(-40, 100)).toBe('heat-neg-2');
});

test('heatStyleClass: zero / NaN / zero-max → empty', () => {
    expect(heatStyleClass(0, 100)).toBe('heat-empty');
    expect(heatStyleClass(NaN, 100)).toBe('heat-empty');
    expect(heatStyleClass(50, 0)).toBe('heat-empty');
});

// ── makeDemoLegs ─────────────────────────────────────────────────

test('makeDemoLegs returns valid 4-leg iron condor', () => {
    const legs = makeDemoLegs();
    expect(legs.length).toBe(4);
    expect(legs.every(l => l.multiplier === 100)).toBe(true);
    expect(legs.every(l => l.spot === 100)).toBe(true);
    // Short put (95) + long put (90) + short call (105) + long call (110)
    const strikes = legs.map(l => l.strike).sort((a, b) => a - b);
    expect(strikes).toEqual([90, 95, 105, 110]);
    // Two shorts and two longs.
    expect(legs.filter(l => l.contracts < 0).length).toBe(2);
    expect(legs.filter(l => l.contracts > 0).length).toBe(2);
});

// ── Formatters ───────────────────────────────────────────────────

test('fmtUSD / fmtUSDSigned / fmtPct', () => {
    expect(fmtUSD(1234)).toBe('$1234');
    expect(fmtUSD(-100)).toBe('-$100');
    expect(fmtUSDSigned(1234)).toBe('+$1234');
    expect(fmtUSDSigned(-100)).toBe('-$100');
    expect(fmtPct(0.05)).toBe('+5.0%');
    expect(fmtPct(-0.10)).toBe('-10.0%');
    expect(fmtUSD(NaN)).toBe('—');
    expect(fmtPct(NaN)).toBe('—');
});
