// IV Solver pure helpers: validator (with no-arb bound pre-flight),
// payload shape, BS no-arb bounds, σ sweep, formatters.

import { test, expect } from 'vitest';
import {
    buildBody, validateParams, arbBounds,
    priceVsSigmaSweep, fmtVolPct, fmtPrice,
} from '../js/_iv_solver_inputs.js';

const baseParams = {
    kind: 'call',
    spot: 100, strike: 105, time_to_expiry: 0.25,
    risk_free: 0.05, dividend_yield: 0,
    market_price: 2.50,
};

// ── validateParams ─────────────────────────────────────────────────

test('validate accepts good defaults', () => {
    expect(validateParams(baseParams)).toBe(null);
});

test('validate rejects bad kind / non-positive spot/strike/T', () => {
    expect(validateParams({ ...baseParams, kind: 'straddle' })).toMatch(/kind/);
    expect(validateParams({ ...baseParams, spot: 0 })).toMatch(/spot/);
    expect(validateParams({ ...baseParams, strike: -1 })).toMatch(/strike/);
    expect(validateParams({ ...baseParams, time_to_expiry: 0 })).toMatch(/time_to_expiry/);
});

test('validate accepts negative risk_free (real-world legal)', () => {
    expect(validateParams({ ...baseParams, risk_free: -0.005 })).toBe(null);
});

test('validate rejects negative dividend', () => {
    expect(validateParams({ ...baseParams, dividend_yield: -0.01 })).toMatch(/dividend_yield/);
});

test('validate rejects negative market_price', () => {
    expect(validateParams({ ...baseParams, market_price: -1 })).toMatch(/market_price/);
});

test('validate pre-flights no-arb bounds', () => {
    // For a call: upper bound = S·e^(-qT) = 100. A 1000 price is way over.
    expect(validateParams({ ...baseParams, market_price: 1000 })).toMatch(/no-arb/);
    // Below lower bound is also rejected. Call lower = max(S − K·e^(-rT), 0)
    // = max(100 − 103.69, 0) = 0; so any non-negative price is at-or-above the
    // lower bound and the test there reduces to upper-bound enforcement.
});

test('validate accepts price exactly at no-arb upper bound (tolerance)', () => {
    const upper = baseParams.spot;  // q=0, so upper = spot = 100.
    expect(validateParams({ ...baseParams, market_price: upper })).toBe(null);
});

// ── arbBounds ─────────────────────────────────────────────────────

test('arbBounds for call: lower = max(F - K·e^(-rT), 0), upper = F', () => {
    // F = 100·e^(-0·0.25) = 100. K·e^(-rT) = 105·e^(-0.05·0.25) ≈ 103.696.
    // Lower = max(100 - 103.696, 0) = 0. Upper = 100.
    const b = arbBounds(baseParams);
    expect(b.lower).toBe(0);
    expect(b.upper).toBeCloseTo(100, 6);
});

test('arbBounds for put: lower = max(K·e^(-rT) - F, 0), upper = K·e^(-rT)', () => {
    const b = arbBounds({ ...baseParams, kind: 'put' });
    const kDisc = 105 * Math.exp(-0.05 * 0.25);
    expect(b.upper).toBeCloseTo(kDisc, 6);
    expect(b.lower).toBeCloseTo(kDisc - 100, 6);  // F = 100
});

test('arbBounds: dividend yield bumps the forward downward', () => {
    const b = arbBounds({ ...baseParams, dividend_yield: 0.02 });
    // F = 100·e^(-0.02·0.25) ≈ 99.501. Upper for call = F.
    expect(b.upper).toBeCloseTo(100 * Math.exp(-0.005), 6);
});

// ── buildBody ──────────────────────────────────────────────────────

test('buildBody passes all 7 fields through', () => {
    const b = buildBody(baseParams);
    expect(b).toEqual({
        market_price: 2.50, spot: 100, strike: 105, time_to_expiry: 0.25,
        risk_free: 0.05, dividend_yield: 0, kind: 'call',
    });
});

// ── priceVsSigmaSweep ─────────────────────────────────────────────

test('priceVsSigmaSweep produces parallel xs and ys', () => {
    const { xs, ys } = priceVsSigmaSweep(baseParams, 1.0, 11);
    expect(xs.length).toBe(11);
    expect(ys.length).toBe(11);
});

test('priceVsSigmaSweep is monotone increasing in σ for a call', () => {
    // BS call is monotone in σ holding other params fixed.
    const { ys } = priceVsSigmaSweep(baseParams, 2.0, 51);
    for (let i = 1; i < ys.length; i++) {
        expect(ys[i]).toBeGreaterThanOrEqual(ys[i - 1] - 1e-9);
    }
});

test('priceVsSigmaSweep at very high σ approaches the no-arb upper bound', () => {
    // For a call as σ → ∞, BS price → S·e^(-qT).
    const { ys } = priceVsSigmaSweep(baseParams, 10.0, 51);
    const last = ys[ys.length - 1];
    const upper = baseParams.spot * Math.exp(-baseParams.dividend_yield * baseParams.time_to_expiry);
    expect(last).toBeLessThanOrEqual(upper + 1e-6);
});

test('priceVsSigmaSweep returns empty when maxSigma ≤ 0', () => {
    expect(priceVsSigmaSweep(baseParams, 0).xs).toEqual([]);
    expect(priceVsSigmaSweep(baseParams, -1).xs).toEqual([]);
});

// ── formatters ─────────────────────────────────────────────────────

test('fmtVolPct emits 2-decimal percent', () => {
    expect(fmtVolPct(0.2534)).toBe('25.34%');
});

test('fmtVolPct returns "—" on non-finite', () => {
    expect(fmtVolPct(NaN)).toBe('—');
});

test('fmtPrice emits 4 decimals by default', () => {
    expect(fmtPrice(1.23456789)).toBe('1.2346');
});

test('fmtPrice returns "—" on non-finite', () => {
    expect(fmtPrice(NaN)).toBe('—');
});
