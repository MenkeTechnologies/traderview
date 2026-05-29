// Margin-call helpers: validator, body shape (Decimal-as-string),
// localEvaluate Rust-mirror, triggerLmv, cushion badge, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, validateInputs, buildBody, localEvaluate, dec,
    triggerLmv, cushionBadge, makeDemoInput,
    fmtUSD, fmtUSDSigned, fmtPct, fmtMaintPct,
} from '../js/_margin_call_inputs.js';

const snap = (over = {}) => ({ ...DEFAULT_INPUTS, ...over });

// ── validator ─────────────────────────────────────────────────────

test('validate accepts standard snapshot', () => {
    expect(validateInputs(snap())).toBe(null);
});

test('validate rejects non-finite / negative numerics', () => {
    expect(validateInputs(snap({ long_market_value: NaN }))).toMatch(/long_market_value/);
    expect(validateInputs(snap({ long_market_value: -1 }))).toMatch(/long_market_value/);
    expect(validateInputs(snap({ margin_debt: -1 }))).toMatch(/margin_debt/);
    expect(validateInputs(snap({ maintenance_pct: -0.1 }))).toMatch(/maintenance_pct/);
    expect(validateInputs(snap({ maintenance_pct: 1.5 }))).toMatch(/maintenance_pct/);
});

test('validate accepts maintenance_pct=0 and =1 (boundary)', () => {
    expect(validateInputs(snap({ maintenance_pct: 0 }))).toBe(null);
    expect(validateInputs(snap({ maintenance_pct: 1 }))).toBe(null);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody stringifies all Decimal fields', () => {
    const body = buildBody(snap({ long_market_value: 100_000, margin_debt: 60_000, maintenance_pct: 0.25 }));
    expect(body).toEqual({
        long_market_value: '100000',
        margin_debt: '60000',
        maintenance_pct: '0.25',
    });
});

// ── localEvaluate parity (one test per Rust property) ────────────

test('local: fully cash account has no margin call', () => {
    const r = localEvaluate(snap({ long_market_value: 50_000, margin_debt: 0, maintenance_pct: 0.25 }));
    expect(r.in_call).toBe(false);
    expect(r.dollar_cushion).toBe(50_000);
});

test('local: standard 25% maintenance, $100k LMV + $60k debt → $20k cushion', () => {
    const r = localEvaluate(snap({ long_market_value: 100_000, margin_debt: 60_000, maintenance_pct: 0.25 }));
    expect(r.in_call).toBe(false);
    expect(r.dollar_cushion).toBe(20_000);
});

test('local: in call when equity below maintenance (LMV 100k, debt 80k @ 25%)', () => {
    const r = localEvaluate(snap({ long_market_value: 100_000, margin_debt: 80_000, maintenance_pct: 0.25 }));
    expect(r.in_call).toBe(true);
    expect(r.dollar_cushion).toBeLessThan(0);
});

test('local: exactly at maintenance ($0 cushion) is NOT in call (strict <)', () => {
    const r = localEvaluate(snap({ long_market_value: 100_000, margin_debt: 75_000, maintenance_pct: 0.25 }));
    expect(r.dollar_cushion).toBeCloseTo(0, 9);
    expect(r.in_call).toBe(false);
});

test('local: higher maintenance % shrinks cushion (same LMV/debt)', () => {
    const lo = localEvaluate(snap({ long_market_value: 100_000, margin_debt: 60_000, maintenance_pct: 0.25 }));
    const hi = localEvaluate(snap({ long_market_value: 100_000, margin_debt: 60_000, maintenance_pct: 0.40 }));
    expect(hi.dollar_cushion).toBeLessThan(lo.dollar_cushion);
    expect(hi.dollar_cushion).toBeCloseTo(0, 9);
});

test('local: 100% maintenance + any debt → in call (cash-only enforced)', () => {
    const r = localEvaluate(snap({ long_market_value: 50_000, margin_debt: 1, maintenance_pct: 1.0 }));
    expect(r.in_call).toBe(true);
    expect(r.dollar_cushion).toBe(-1);
});

test('local: zero LMV with zero debt → no call, equity = 0', () => {
    const r = localEvaluate(snap({ long_market_value: 0, margin_debt: 0, maintenance_pct: 0.25 }));
    expect(r.in_call).toBe(false);
    expect(r.current_equity).toBe(0);
});

test('local: pct_cushion = $cushion / LMV (e.g. 20%)', () => {
    const r = localEvaluate(snap({ long_market_value: 100_000, margin_debt: 60_000, maintenance_pct: 0.25 }));
    expect(r.pct_cushion).toBeCloseTo(0.20, 9);
});

test('local: current_equity_pct = (LMV - debt) / LMV', () => {
    const r = localEvaluate(snap({ long_market_value: 100_000, margin_debt: 60_000, maintenance_pct: 0.25 }));
    expect(r.current_equity_pct).toBeCloseTo(0.40, 9);
});

// ── triggerLmv ────────────────────────────────────────────────────

test('triggerLmv: debt / (1 - maint_pct) at 25% on $60k → $80k', () => {
    expect(triggerLmv({ margin_debt: 60_000, maintenance_pct: 0.25 })).toBeCloseTo(80_000, 9);
});

test('triggerLmv: maintenance_pct = 1.0 → Infinity (no leverage tolerable)', () => {
    expect(triggerLmv({ margin_debt: 1, maintenance_pct: 1.0 })).toBe(Infinity);
});

// ── cushionBadge ──────────────────────────────────────────────────

test('cushionBadge: in_call wins', () => {
    expect(cushionBadge({ in_call: true, pct_cushion: 1 }).key).toMatch(/in_call/);
});

test('cushionBadge: tiered by pct_cushion (5% / 15% / 30%)', () => {
    expect(cushionBadge({ in_call: false, pct_cushion: 0.04 }).key).toMatch(/critical/);
    expect(cushionBadge({ in_call: false, pct_cushion: 0.10 }).key).toMatch(/tight/);
    expect(cushionBadge({ in_call: false, pct_cushion: 0.20 }).key).toMatch(/moderate/);
    expect(cushionBadge({ in_call: false, pct_cushion: 0.50 }).key).toMatch(/safe/);
});

test('cushionBadge: null / NaN → unknown', () => {
    expect(cushionBadge(null).key).toMatch(/unknown/);
    expect(cushionBadge({ in_call: false, pct_cushion: NaN }).key).toMatch(/unknown/);
});

// ── demos ─────────────────────────────────────────────────────────

test('demo standard: cushion $20k, not in call', () => {
    const r = localEvaluate(makeDemoInput('standard'));
    expect(r.dollar_cushion).toBe(20_000);
    expect(r.in_call).toBe(false);
});

test('demo in-call: dollar_cushion < 0', () => {
    expect(localEvaluate(makeDemoInput('in-call')).in_call).toBe(true);
});

test('demo at-line: cushion = $0, not in call', () => {
    const r = localEvaluate(makeDemoInput('at-line'));
    expect(r.dollar_cushion).toBeCloseTo(0, 9);
    expect(r.in_call).toBe(false);
});

test('demo high-maint: cushion = $0 with same numbers but 40% maint', () => {
    const r = localEvaluate(makeDemoInput('high-maint'));
    expect(r.dollar_cushion).toBeCloseTo(0, 9);
});

test('demo cash-only-with-debt: in call (100% maint + $1 debt)', () => {
    expect(localEvaluate(makeDemoInput('cash-only-with-debt')).in_call).toBe(true);
});

test('demo no-positions: no call, no equity', () => {
    const r = localEvaluate(makeDemoInput('no-positions'));
    expect(r.in_call).toBe(false);
    expect(r.current_equity).toBe(0);
});

test('demo leveraged-bull: $100k cushion, 20% of $500k LMV', () => {
    const r = localEvaluate(makeDemoInput('leveraged-bull'));
    expect(r.dollar_cushion).toBe(100_000);
    expect(r.pct_cushion).toBeCloseTo(0.20, 9);
});

// ── dec / formatters ──────────────────────────────────────────────

test('dec coerces strings + guards', () => {
    expect(dec('100.5')).toBe(100.5);
    expect(dec(null)).toBe(0);
    expect(dec('abc')).toBe(0);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtUSD(1234)).toBe('$1234');
    expect(fmtUSDSigned(-100)).toBe('-$100');
    expect(fmtPct(0.25)).toBe('25.00%');
    expect(fmtMaintPct(0.25)).toBe('25%');
    expect(fmtUSD(NaN)).toBe('—');
});

// ── DEFAULT_INPUTS ────────────────────────────────────────────────

test('DEFAULT_INPUTS is the "standard" preset', () => {
    expect(DEFAULT_INPUTS).toEqual(makeDemoInput('standard'));
});
