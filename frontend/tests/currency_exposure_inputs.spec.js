// Currency-exposure helpers: parsers, validator, body shape, local
// analyze parity with Rust, badges, demos.

import { test, expect } from 'vitest';
import {
    parsePositionBlob, parseFxBlob, validateInputs, buildBody, localAnalyze,
    concentrationBadge, makeDemoPositions, makeDemoFx, defaultFxRates,
    ccyColor, fmtUSD, fmtUSDSigned, fmtPct, fmtNum, fmtRate,
} from '../js/_currency_exposure_inputs.js';

const p = (sym, ccy, n) => ({ symbol: sym, currency: ccy, notional_native: n });
const RATES = { EUR: 1.10, GBP: 1.27, JPY: 0.0064 };

// ── parsers ───────────────────────────────────────────────────────

test('parsePositionBlob: 3 tokens + comments + upcased ccy', () => {
    const r = parsePositionBlob('aapl USD 30000\n# tech\nsap eur 20000');
    expect(r.errors).toEqual([]);
    expect(r.positions).toEqual([p('AAPL', 'USD', 30000), p('SAP', 'EUR', 20000)]);
});

test('parsePositionBlob: accepts negative notional (short)', () => {
    expect(parsePositionBlob('SHORT EUR -5000').errors).toEqual([]);
});

test('parsePositionBlob: rejects bad ccy / non-finite notional / bad token count', () => {
    // Bad ccy: digit-prefixed string fails the regex /^[A-Z]{2,5}$/.
    expect(parsePositionBlob('AAPL US1 1000').errors[0].message).toMatch(/currency/);
    expect(parsePositionBlob('AAPL USD abc').errors[0].message).toMatch(/finite/);
    expect(parsePositionBlob('AAPL USD').errors[0].message).toMatch(/3 tokens/);
});

test('parseFxBlob: 2 tokens; rejects rate ≤ 0', () => {
    const r = parseFxBlob('EUR 1.10\nGBP 1.27');
    expect(r.errors).toEqual([]);
    expect(r.fx).toEqual({ EUR: 1.10, GBP: 1.27 });
    expect(parseFxBlob('EUR 0').errors[0].message).toMatch(/rate/);
});

test('parsers: non-string returns 1 error', () => {
    expect(parsePositionBlob(null).errors.length).toBe(1);
    expect(parseFxBlob(null).errors.length).toBe(1);
});

// ── validator / buildBody ─────────────────────────────────────────

test('validate accepts well-formed inputs', () => {
    expect(validateInputs([p('AAPL', 'USD', 1)], 'USD', RATES)).toBe(null);
});

test('validate rejects bad home / bad fx values', () => {
    expect(validateInputs([], 'usd', RATES)).toMatch(/home_currency/);
    expect(validateInputs([], 'USD', null)).toMatch(/fx_to_home/);
    expect(validateInputs([], 'USD', { EUR: 0 })).toMatch(/EUR/);
});

test('buildBody passes through positions + spreads fx object', () => {
    const body = buildBody([p('AAPL', 'USD', 100)], 'USD', RATES);
    expect(body.positions[0]).toEqual({ symbol: 'AAPL', currency: 'USD', notional_native: 100 });
    expect(body.home_currency).toBe('USD');
    expect(body.fx_to_home).toEqual(RATES);
});

// ── localAnalyze parity (one test per Rust property) ─────────────

test('local: empty positions → empty buckets + default report', () => {
    const r = localAnalyze([], 'USD', RATES);
    expect(r.buckets).toEqual([]);
    expect(r.home_currency).toBe('USD');
});

test('local: home currency uses rate 1.0', () => {
    const r = localAnalyze([p('AAPL', 'USD', 10000)], 'USD', RATES);
    const usd = r.buckets.find(b => b.currency === 'USD');
    expect(usd.gross_home).toBe(10000);
});

test('local: foreign currency uses supplied fx rate', () => {
    const r = localAnalyze([p('SAP', 'EUR', 10000)], 'USD', RATES);
    const eur = r.buckets.find(b => b.currency === 'EUR');
    expect(eur.gross_home).toBeCloseTo(11000, 9);
});

test('local: missing fx rate falls back to 0', () => {
    const r = localAnalyze([p('RY', 'CAD', 10000)], 'USD', RATES);
    const cad = r.buckets.find(b => b.currency === 'CAD');
    expect(cad.gross_home).toBe(0);
});

test('local: overweight flagged above 25% of home gross (non-home only)', () => {
    const r = localAnalyze([p('AAPL', 'USD', 1000), p('SAP', 'EUR', 10000)], 'USD', RATES);
    expect(r.overweight_currencies).toContain('EUR');
});

test('local: home currency never marked overweight (even at 100%)', () => {
    const r = localAnalyze([p('AAPL', 'USD', 100000)], 'USD', RATES);
    expect(r.overweight_currencies).toEqual([]);
});

test('local: short position reduces NET, keeps GROSS', () => {
    const r = localAnalyze([p('LONG', 'USD', 10000), p('SHORT', 'USD', -5000)], 'USD', RATES);
    const usd = r.buckets.find(b => b.currency === 'USD');
    expect(usd.gross_home).toBe(15000);
    expect(usd.net_home).toBe(5000);
});

test('local: buckets sorted by gross_home DESC', () => {
    const r = localAnalyze([
        p('SMALL', 'GBP', 1000),    // 1270 USD
        p('BIG',   'USD', 100000),  // 100000 USD
        p('MID',   'EUR', 10000),   // 11000 USD
    ], 'USD', RATES);
    expect(r.buckets.map(b => b.currency)).toEqual(['USD', 'EUR', 'GBP']);
});

test('local: pct_of_total sums to ~1.0', () => {
    const r = localAnalyze([
        p('A', 'USD', 50000),
        p('B', 'EUR', 30000),
        p('C', 'GBP', 5000),
    ], 'USD', RATES);
    const sum = r.buckets.reduce((s, b) => s + b.pct_of_total, 0);
    expect(sum).toBeCloseTo(1.0, 9);
});

test('local: position_count counts per-bucket', () => {
    const r = localAnalyze([
        p('A', 'USD', 1), p('B', 'USD', 2), p('C', 'EUR', 3),
    ], 'USD', RATES);
    expect(r.buckets.find(b => b.currency === 'USD').position_count).toBe(2);
    expect(r.buckets.find(b => b.currency === 'EUR').position_count).toBe(1);
});

// ── boundary: exactly 25% NOT overweight (strict >) ───────────────

test('local: exactly 25% non-home gross → NOT overweight (strict >)', () => {
    // 25% threshold: USD 30k home + EUR ~10k (gets converted to 11k home).
    // To hit exactly 25%: total = 4 × overweight. 11k EUR = 25% of total 44k.
    // Need USD = 33k home. → exactly 25% EUR.
    const r = localAnalyze([p('A', 'USD', 33000), p('B', 'EUR', 10000)], 'USD', RATES);
    const eur = r.buckets.find(b => b.currency === 'EUR');
    expect(eur.pct_of_total).toBeCloseTo(0.25, 9);
    expect(r.overweight_currencies).not.toContain('EUR');
});

// ── concentrationBadge ────────────────────────────────────────────

test('concentrationBadge: home-only → no_fx', () => {
    const r = localAnalyze([p('A', 'USD', 100)], 'USD', RATES);
    expect(concentrationBadge(r, 'USD').key).toMatch(/no_fx/);
});

test('concentrationBadge: ≥ 50% non-home → concentrated', () => {
    const r = localAnalyze([p('A', 'EUR', 10000)], 'USD', RATES);  // 100% EUR
    expect(concentrationBadge(r, 'USD').key).toMatch(/concentrated/);
});

test('concentrationBadge: tilted at 25-50%', () => {
    const r = localAnalyze([p('A', 'USD', 20000), p('B', 'EUR', 10000)], 'USD', RATES);
    expect(concentrationBadge(r, 'USD').key).toMatch(/tilted/);
});

test('concentrationBadge: diversified at 10-25%, minimal < 10%', () => {
    // EUR 20k × 1.10 = 22k vs USD 100k → 22/122 ≈ 18% → diversified.
    const r1 = localAnalyze([p('A', 'USD', 100000), p('B', 'EUR', 20000)], 'USD', RATES);
    expect(concentrationBadge(r1, 'USD').key).toMatch(/diversified/);
    // EUR 2k × 1.10 = 2.2k vs USD 100k → 2.2/102.2 ≈ 2% → minimal.
    const r2 = localAnalyze([p('A', 'USD', 100000), p('B', 'EUR', 2000)], 'USD', RATES);
    expect(concentrationBadge(r2, 'USD').key).toMatch(/minimal/);
});

// ── default fx rates / demos ──────────────────────────────────────

test('defaultFxRates includes EUR/GBP/JPY/CAD/CHF/AUD', () => {
    const r = defaultFxRates();
    for (const c of ['EUR', 'GBP', 'JPY', 'CAD', 'CHF', 'AUD']) {
        expect(r[c]).toBeGreaterThan(0);
    }
});

test('demo multi-region: 4 currencies, gross > 0 each', () => {
    const positions = makeDemoPositions('multi-region');
    const fx = makeDemoFx('multi-region');
    const r = localAnalyze(positions, 'USD', fx);
    expect(r.buckets.length).toBe(4);
});

test('demo missing-fx: CAD bucket has gross_home=0 + missing rate not in fx', () => {
    const positions = makeDemoPositions('missing-fx');
    const fx = makeDemoFx('missing-fx');
    expect(fx.CAD).toBeUndefined();
    const r = localAnalyze(positions, 'USD', fx);
    const cad = r.buckets.find(b => b.currency === 'CAD');
    expect(cad.gross_home).toBe(0);
});

test('demo short-hedged: USD net = 100k, EUR net = -20k * 1.10 = -22k → total_net 78k', () => {
    const r = localAnalyze(makeDemoPositions('short-hedged'), 'USD', makeDemoFx('short-hedged'));
    expect(r.total_net_home).toBeCloseTo(78000, 6);
});

test('demo eur-concentrated: EUR flagged overweight', () => {
    const r = localAnalyze(makeDemoPositions('eur-concentrated'), 'USD', makeDemoFx('eur-concentrated'));
    expect(r.overweight_currencies).toContain('EUR');
});

test('demo home-only: no overweight currencies', () => {
    const r = localAnalyze(makeDemoPositions('home-only'), 'USD', makeDemoFx('home-only'));
    expect(r.overweight_currencies).toEqual([]);
});

// ── formatters + colors ───────────────────────────────────────────

test('fmt helpers + non-finite guards', () => {
    expect(fmtUSD(1234)).toBe('$1234');
    expect(fmtUSDSigned(-100)).toBe('-$100');
    expect(fmtPct(0.25)).toBe('25.0%');
    expect(fmtNum(123.456, 1)).toBe('123.5');
    expect(fmtRate(1.10)).toBe('1.1000');
    expect(fmtUSD(NaN)).toBe('—');
});

test('ccyColor cycles palette, neg id → muted', () => {
    expect(ccyColor(0)).toBe('#00e5ff');
    expect(ccyColor(8)).toBe('#00e5ff');
    expect(ccyColor(-1)).toBe('#aab');
});
