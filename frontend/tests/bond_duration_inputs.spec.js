// Bond-duration helpers: parser, validator, body shape, localCompute
// Rust-mirror, priceChangePct, buildCouponBond, badge tiers, demos.

import { test, expect } from 'vitest';
import {
    parseCashFlowBlob, validateInputs, buildBody, localCompute,
    priceChangePct, buildCouponBond, durationBadge, SENSITIVITY_BPS,
    makeDemoConfig, fmtUSD, fmtPctSigned, fmtPct, fmtYears, fmtBpsSigned,
} from '../js/_bond_duration_inputs.js';

const cf = (t, a) => ({ time_years: t, amount: a });

// ── parser ────────────────────────────────────────────────────────

test('parseCashFlowBlob: 2 tokens + comments', () => {
    const r = parseCashFlowBlob('1 5\n2 5\n# coupon\n5 105');
    expect(r.errors).toEqual([]);
    expect(r.cash_flows).toEqual([cf(1, 5), cf(2, 5), cf(5, 105)]);
});

test('parseCashFlowBlob: rejects bad token count / non-finite / non-positive time', () => {
    expect(parseCashFlowBlob('1').errors[0].message).toMatch(/2 tokens/);
    expect(parseCashFlowBlob('1 abc').errors[0].message).toMatch(/finite/);
    expect(parseCashFlowBlob('0 100').errors[0].message).toMatch(/time_years/);
});

test('parseCashFlowBlob: accepts negative amount (early redemption fee, etc.)', () => {
    expect(parseCashFlowBlob('1 -5').errors).toEqual([]);
});

test('parseCashFlowBlob: non-string returns 1 error', () => {
    expect(parseCashFlowBlob(null).errors.length).toBe(1);
});

// ── validator / buildBody ─────────────────────────────────────────

test('validate accepts good inputs', () => {
    expect(validateInputs([cf(5, 100)], 0.05, 2)).toBe(null);
});

test('validate rejects empty / non-finite / bad compounding', () => {
    expect(validateInputs([], 0.05, 2)).toMatch(/≥ 1/);
    expect(validateInputs([cf(1, 1)], NaN, 2)).toMatch(/ytm/);
    expect(validateInputs([cf(1, 1)], 0.05, 0)).toMatch(/compounding/);
    expect(validateInputs([cf(1, 1)], 0.05, 1.5)).toMatch(/compounding/);
});

test('buildBody passes through cash_flows + ytm + compounding', () => {
    const body = buildBody([cf(1, 5)], 0.05, 2);
    expect(body).toEqual({
        cash_flows: [{ time_years: 1, amount: 5 }],
        ytm: 0.05, compounding_per_year: 2,
    });
});

// ── localCompute parity (one test per Rust property) ──────────────

test('local: empty CFs returns zeroed default', () => {
    const r = localCompute([], 0.05, 2);
    expect(r.macaulay_duration).toBe(0);
    expect(r.yield_to_maturity).toBe(0.05);
});

test('local: zero-coupon bond Macaulay = maturity', () => {
    const r = localCompute([cf(5, 100)], 0.05, 2);
    expect(r.macaulay_duration).toBeCloseTo(5, 9);
});

test('local: Modified strictly less than Macaulay (for positive ytm)', () => {
    const r = localCompute([cf(5, 100)], 0.05, 2);
    expect(r.modified_duration).toBeLessThan(r.macaulay_duration);
});

test('local: coupon bond duration < maturity (but close to it)', () => {
    const cfs = [cf(1, 5), cf(2, 5), cf(3, 5), cf(4, 5), cf(5, 105)];
    const r = localCompute(cfs, 0.05, 1);
    expect(r.macaulay_duration).toBeLessThan(5);
    expect(r.macaulay_duration).toBeGreaterThan(4);
});

test('local: price at par when YTM = coupon rate', () => {
    const cfs = [cf(1, 5), cf(2, 5), cf(3, 5), cf(4, 5), cf(5, 105)];
    expect(localCompute(cfs, 0.05, 1).price).toBeCloseTo(100, 9);
});

test('local: higher YTM → shorter duration (CFs weighted more toward near term)', () => {
    const cfs = [cf(1, 5), cf(2, 5), cf(3, 5), cf(4, 5), cf(5, 105)];
    const lo = localCompute(cfs, 0.03, 1);
    const hi = localCompute(cfs, 0.08, 1);
    expect(hi.macaulay_duration).toBeLessThan(lo.macaulay_duration);
});

test('local: semi-annual vs annual changes PV', () => {
    const cfs = [cf(1, 100)];
    const annual = localCompute(cfs, 0.06, 1);
    const semi   = localCompute(cfs, 0.06, 2);
    expect(semi.price).toBeLessThan(annual.price);
});

test('local: degenerate ytm = -1 (1 + ytm/m = 0) → returns zeroed default', () => {
    const r = localCompute([cf(5, 100)], -1.0, 1);
    expect(r.price).toBe(0);
    expect(r.macaulay_duration).toBe(0);
});

test('local: yield_to_maturity field echoed in output', () => {
    expect(localCompute([cf(1, 1)], 0.0345, 2).yield_to_maturity).toBe(0.0345);
});

// ── priceChangePct ────────────────────────────────────────────────

test('priceChangePct: 1yr duration + 100bps → -1.00%', () => {
    expect(priceChangePct(1, 100)).toBeCloseTo(-0.01, 12);
});

test('priceChangePct: 5yr duration + -50bps → +2.50%', () => {
    expect(priceChangePct(5, -50)).toBeCloseTo(0.025, 12);
});

test('priceChangePct: non-finite inputs → 0', () => {
    expect(priceChangePct(NaN, 100)).toBe(0);
    expect(priceChangePct(5, NaN)).toBe(0);
});

// ── buildCouponBond ───────────────────────────────────────────────

test('buildCouponBond: 5% annual 5yr → 5 CFs ending at year 5 with par+coupon', () => {
    const cfs = buildCouponBond(100, 0.05, 5, 1);
    expect(cfs.length).toBe(5);
    expect(cfs[0]).toEqual({ time_years: 1, amount: 5 });
    expect(cfs[4]).toEqual({ time_years: 5, amount: 105 });
});

test('buildCouponBond: 4% semi 10yr → 20 CFs, periodic coupon of 2', () => {
    const cfs = buildCouponBond(100, 0.04, 10, 2);
    expect(cfs.length).toBe(20);
    expect(cfs[0]).toEqual({ time_years: 0.5, amount: 2 });
    expect(cfs[19]).toEqual({ time_years: 10, amount: 102 });
});

test('buildCouponBond: par bond at YTM=coupon prices at par', () => {
    const cfs = buildCouponBond(100, 0.05, 5, 1);
    expect(localCompute(cfs, 0.05, 1).price).toBeCloseTo(100, 9);
});

test('buildCouponBond: zero coupon (rate=0) emits only par-at-maturity', () => {
    const cfs = buildCouponBond(100, 0, 5, 1);
    expect(cfs.length).toBe(5);
    // First 4 CFs are 0 (coupon=0), last is 100 (par).
    expect(cfs[0].amount).toBe(0);
    expect(cfs[4].amount).toBe(100);
});

test('buildCouponBond: par ≤ 0 / maturity ≤ 0 / bad compounding → empty', () => {
    expect(buildCouponBond(0, 0.05, 5, 1)).toEqual([]);
    expect(buildCouponBond(100, 0.05, 0, 1)).toEqual([]);
    expect(buildCouponBond(100, 0.05, 5, 0)).toEqual([]);
});

// ── durationBadge tiers ───────────────────────────────────────────

test('durationBadge: < 1 = cash, < 3 = short, < 7 = intermediate, < 12 = long, ≥ 12 = ultra', () => {
    expect(durationBadge(0.5).key).toMatch(/cash/);
    expect(durationBadge(2.5).key).toMatch(/short/);
    expect(durationBadge(5).key).toMatch(/intermediate/);
    expect(durationBadge(10).key).toMatch(/long/);
    expect(durationBadge(20).key).toMatch(/ultra/);
    expect(durationBadge(NaN).key).toMatch(/unknown/);
});

// ── SENSITIVITY_BPS ───────────────────────────────────────────────

test('SENSITIVITY_BPS includes ±200 / ±100 / ±50 / ±25 / ±10', () => {
    expect(SENSITIVITY_BPS).toEqual([-200, -100, -50, -25, -10, 10, 25, 50, 100, 200]);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset produces a valid duration result', () => {
    for (const k of ['zero-5yr', 'treasury-5yr-coupon', 'treasury-10yr-semi',
                     'treasury-30yr-semi', 'corporate-7yr-high-coupon', 'tips-zero-2yr']) {
        const cfg = makeDemoConfig(k);
        const r = localCompute(cfg.cash_flows, cfg.ytm, cfg.compounding_per_year);
        expect(r.price).toBeGreaterThan(0);
        expect(r.macaulay_duration).toBeGreaterThan(0);
        expect(r.modified_duration).toBeGreaterThan(0);
    }
});

test('demo treasury-30yr is ultra-long duration', () => {
    const cfg = makeDemoConfig('treasury-30yr-semi');
    const r = localCompute(cfg.cash_flows, cfg.ytm, cfg.compounding_per_year);
    expect(durationBadge(r.macaulay_duration).key).toMatch(/ultra|long/);
});

test('demo tips-zero-2yr is short-duration', () => {
    const cfg = makeDemoConfig('tips-zero-2yr');
    const r = localCompute(cfg.cash_flows, cfg.ytm, cfg.compounding_per_year);
    expect(r.macaulay_duration).toBeCloseTo(2, 9);
});

// ── formatters ────────────────────────────────────────────────────

test('fmt helpers + non-finite guards', () => {
    expect(fmtUSD(1234.5)).toBe('$1234.50');
    expect(fmtPct(0.0345)).toBe('3.45%');
    expect(fmtPctSigned(0.025)).toBe('+2.500%');
    expect(fmtPctSigned(-0.01, 2)).toBe('-1.00%');
    expect(fmtYears(4.567)).toBe('4.567 yr');
    expect(fmtBpsSigned(100)).toBe('+100 bps');
    expect(fmtBpsSigned(-50)).toBe('-50 bps');
    expect(fmtUSD(NaN)).toBe('—');
});
