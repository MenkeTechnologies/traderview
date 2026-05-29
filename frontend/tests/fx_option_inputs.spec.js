// FX Option Calculator pure helpers: payload shape, validator,
// Garman-Kohlhagen closed-form, formatters.

import { test, expect } from 'vitest';
import {
    buildGkBody, validateGkParams,
    garmanKohlhagenPrice, fmtRate, fmtGreek,
} from '../js/_fx_option_inputs.js';
import { blackScholesEuropean } from '../js/_american_option_inputs.js';

const baseParams = {
    kind: 'call',
    spot: 1.10, strike: 1.10, t_years: 0.25,
    rate_dom: 0.05, rate_for: 0.03, sigma: 0.10,
};

// ── validateGkParams ────────────────────────────────────────────────

test('validate accepts good defaults', () => {
    expect(validateGkParams(baseParams)).toBe(null);
});

test('validate rejects bad kind', () => {
    expect(validateGkParams({ ...baseParams, kind: 'forward' })).toMatch(/kind/);
});

test('validate rejects non-positive spot/strike/t', () => {
    expect(validateGkParams({ ...baseParams, spot: 0 })).toMatch(/spot/);
    expect(validateGkParams({ ...baseParams, strike: -1 })).toMatch(/strike/);
    expect(validateGkParams({ ...baseParams, t_years: 0 })).toMatch(/t_years/);
});

test('validate accepts negative rates (rate-domestic could be negative)', () => {
    // Real markets see negative rates (e.g. CHF 2015-2022, JPY 2016-2024).
    // Validator must allow this even though it looks "wrong".
    expect(validateGkParams({ ...baseParams, rate_dom: -0.005 })).toBe(null);
    expect(validateGkParams({ ...baseParams, rate_for: -0.005 })).toBe(null);
});

test('validate rejects non-finite rates', () => {
    expect(validateGkParams({ ...baseParams, rate_dom: NaN })).toMatch(/rate_dom/);
    expect(validateGkParams({ ...baseParams, rate_for: Infinity })).toMatch(/rate_for/);
});

test('validate rejects negative sigma', () => {
    expect(validateGkParams({ ...baseParams, sigma: -0.01 })).toMatch(/sigma/);
});

// ── buildGkBody ─────────────────────────────────────────────────────

test('buildGkBody passes through all 7 fields', () => {
    const b = buildGkBody(baseParams);
    expect(b).toEqual({
        kind: 'call', spot: 1.10, strike: 1.10, t_years: 0.25,
        rate_dom: 0.05, rate_for: 0.03, sigma: 0.10,
    });
});

// ── garmanKohlhagenPrice ────────────────────────────────────────────

test('GK reduces to BS when foreign rate = dividend yield', () => {
    // The mathematical identity: GK(rd, rf=q) ≡ BS(r=rd, q).
    const S = 100, K = 100, T = 1, r = 0.05, q = 0.02, sigma = 0.20;
    const gk = garmanKohlhagenPrice('call', S, K, T, r, q, sigma);
    const bs = blackScholesEuropean('call', S, K, T, r, q, sigma);
    expect(gk).toBeCloseTo(bs, 10);
});

test('GK put-call parity: C − P = S·e^(−r_f·T) − K·e^(−r_d·T)', () => {
    const S = 1.20, K = 1.10, T = 0.5, rd = 0.04, rf = 0.01, sigma = 0.12;
    const c = garmanKohlhagenPrice('call', S, K, T, rd, rf, sigma);
    const p = garmanKohlhagenPrice('put',  S, K, T, rd, rf, sigma);
    const parity = S * Math.exp(-rf * T) - K * Math.exp(-rd * T);
    expect(c - p).toBeCloseTo(parity, 8);
});

test('GK at expiry collapses to intrinsic', () => {
    expect(garmanKohlhagenPrice('call', 1.20, 1.10, 0, 0.05, 0.03, 0.10)).toBeCloseTo(0.10, 12);
    expect(garmanKohlhagenPrice('put',  1.00, 1.10, 0, 0.05, 0.03, 0.10)).toBeCloseTo(0.10, 12);
    expect(garmanKohlhagenPrice('call', 1.00, 1.10, 0, 0.05, 0.03, 0.10)).toBe(0);
});

test('GK at zero vol collapses to (un-discounted) intrinsic', () => {
    expect(garmanKohlhagenPrice('call', 1.20, 1.10, 1, 0.05, 0.03, 0)).toBeCloseTo(0.10, 12);
});

test('GK call is monotone increasing in spot', () => {
    let prev = -Infinity;
    for (const s of [0.90, 0.95, 1.00, 1.05, 1.10, 1.15]) {
        const c = garmanKohlhagenPrice('call', s, 1.05, 0.5, 0.04, 0.02, 0.10);
        expect(c).toBeGreaterThanOrEqual(prev);
        prev = c;
    }
});

test('GK put is monotone decreasing in spot', () => {
    let prev = Infinity;
    for (const s of [0.90, 0.95, 1.00, 1.05, 1.10, 1.15]) {
        const p = garmanKohlhagenPrice('put', s, 1.05, 0.5, 0.04, 0.02, 0.10);
        expect(p).toBeLessThanOrEqual(prev);
        prev = p;
    }
});

test('GK handles negative rates without error', () => {
    // CHF-style: negative domestic rate. Pricer should still produce a
    // positive option value, no NaN/Infinity.
    const c = garmanKohlhagenPrice('call', 1.10, 1.10, 0.5, -0.005, 0.04, 0.08);
    expect(Number.isFinite(c)).toBe(true);
    expect(c).toBeGreaterThan(0);
});

// ── fmtRate / fmtGreek ─────────────────────────────────────────────

test('fmtRate defaults to 4 decimals', () => {
    expect(fmtRate(1.234567)).toBe('1.2346');
});

test('fmtRate honors custom digits', () => {
    expect(fmtRate(1.234567, 2)).toBe('1.23');
});

test('fmtGreek defaults to 6 decimals', () => {
    expect(fmtGreek(0.0123456789)).toBe('0.012346');
});

test('fmtRate / fmtGreek return "—" on non-finite', () => {
    expect(fmtRate(NaN)).toBe('—');
    expect(fmtRate(Infinity)).toBe('—');
    expect(fmtGreek(NaN)).toBe('—');
    expect(fmtGreek(Infinity)).toBe('—');
});
