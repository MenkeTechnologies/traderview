// Margin-runway helpers: validator, body shape, local Rust-mirror,
// runway badge, projection curves, demos, formatters.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, validateInputs, buildBody, localCompute,
    runwayBadge, projectionCurves, makeDemoInputs,
    fmtUSD, fmtUSDSigned, fmtPct, fmtMaintPct,
} from '../js/_margin_runway_inputs.js';

// ── validator / buildBody ─────────────────────────────────────────

test('validate accepts good inputs', () => {
    expect(validateInputs(50_000, 100_000, 0.25)).toBe(null);
});

test('validate accepts zero position (cash-only)', () => {
    expect(validateInputs(50_000, 0, 0.25)).toBe(null);
});

test('validate accepts negative equity (already underwater)', () => {
    expect(validateInputs(-5_000, 100_000, 0.25)).toBe(null);
});

test('validate rejects negative position', () => {
    expect(validateInputs(50_000, -1, 0.25)).toMatch(/position/);
});

test('validate rejects maint < 0 or ≥ 1', () => {
    expect(validateInputs(50_000, 100_000, -0.1)).toMatch(/maintenance/);
    expect(validateInputs(50_000, 100_000, 1.0)).toMatch(/maintenance/);
    expect(validateInputs(50_000, 100_000, 1.5)).toMatch(/maintenance/);
});

test('validate rejects non-finite', () => {
    expect(validateInputs(NaN, 100_000, 0.25)).toMatch(/account_equity/);
    expect(validateInputs(50_000, NaN, 0.25)).toMatch(/position_value/);
    expect(validateInputs(50_000, 100_000, NaN)).toMatch(/maintenance/);
});

test('buildBody mirrors backend MarginRunwayBody', () => {
    expect(buildBody(50_000, 100_000, 0.25)).toEqual({
        account_equity: 50_000, position_value: 100_000, maintenance_req_pct: 0.25,
    });
});

// ── localCompute parity (one test per Rust test case) ─────────────

test('local: position=0 → runway=0, no-call', () => {
    const r = localCompute(10_000, 0, 0.25);
    expect(r.runway_pct).toBe(0);
    expect(r.already_in_margin_call).toBe(false);
});

test('local: maint_pct ≥ 1 → zeroed (degenerate guard)', () => {
    const r = localCompute(50_000, 100_000, 1.0);
    expect(r.runway_pct).toBe(0);
});

test('local: $100k equity + $100k position @ 25% → runway = 1.0 (100%)', () => {
    const r = localCompute(100_000, 100_000, 0.25);
    expect(r.runway_pct).toBeCloseTo(1.0, 9);
    expect(r.already_in_margin_call).toBe(false);
    expect(r.equity_buffer_dollars).toBe(75_000);
});

test('local: $50k equity + $100k position @ 25% → runway = 1/3', () => {
    const r = localCompute(50_000, 100_000, 0.25);
    expect(r.runway_pct).toBeCloseTo(1 / 3, 9);
});

test('local: equity == maint_dollars → buffer=0 and runway=0', () => {
    const r = localCompute(25_000, 100_000, 0.25);
    expect(r.equity_buffer_dollars).toBe(0);
    expect(r.runway_pct).toBeCloseTo(0, 9);
});

test('local: equity < maint → already_in_call true, runway=0', () => {
    const r = localCompute(20_000, 100_000, 0.25);
    expect(r.already_in_margin_call).toBe(true);
    expect(r.runway_pct).toBe(0);
    expect(r.equity_buffer_dollars).toBeLessThan(0);
});

test('local: higher maint% → strictly lower runway (same equity+position)', () => {
    const lo = localCompute(50_000, 100_000, 0.25);
    const hi = localCompute(50_000, 100_000, 0.40);
    expect(hi.runway_pct).toBeLessThan(lo.runway_pct);
});

test('local: $40k equity + $100k position @ 25% → buffer = $15k', () => {
    const r = localCompute(40_000, 100_000, 0.25);
    expect(r.equity_buffer_dollars).toBe(15_000);
});

test('local: report echoes inputs (account_equity / position / maint_pct)', () => {
    const r = localCompute(40_000, 100_000, 0.25);
    expect(r.account_equity).toBe(40_000);
    expect(r.position_value).toBe(100_000);
    expect(r.maintenance_req_pct).toBe(0.25);
});

// ── runwayBadge ───────────────────────────────────────────────────

test('runwayBadge: in_call → in_call badge regardless of runway', () => {
    expect(runwayBadge({ runway_pct: 0, already_in_margin_call: true }).key).toMatch(/in_call/);
});

test('runwayBadge: < 5% → critical', () => {
    expect(runwayBadge({ runway_pct: 0.04 }).key).toMatch(/critical/);
});

test('runwayBadge: < 15% → tight', () => {
    expect(runwayBadge({ runway_pct: 0.10 }).key).toMatch(/tight/);
});

test('runwayBadge: < 30% → moderate', () => {
    expect(runwayBadge({ runway_pct: 0.25 }).key).toMatch(/moderate/);
});

test('runwayBadge: ≥ 30% → safe', () => {
    expect(runwayBadge({ runway_pct: 0.40 }).key).toMatch(/safe/);
    expect(runwayBadge({ runway_pct: 1.0  }).key).toMatch(/safe/);
});

test('runwayBadge: malformed → unknown', () => {
    expect(runwayBadge(null).key).toMatch(/unknown/);
    expect(runwayBadge({ runway_pct: NaN }).key).toMatch(/unknown/);
});

// ── projectionCurves ──────────────────────────────────────────────

test('projectionCurves: equity strictly decreases with decline (long position)', () => {
    const c = projectionCurves(100_000, 100_000, 0.25, 20, 0.5);
    for (let i = 1; i < c.equityCurve.length; i++) {
        expect(c.equityCurve[i]).toBeLessThan(c.equityCurve[i - 1]);
    }
});

test('projectionCurves: maint also decreases with decline (lower position value)', () => {
    const c = projectionCurves(100_000, 100_000, 0.25, 20, 0.5);
    for (let i = 1; i < c.maintCurve.length; i++) {
        expect(c.maintCurve[i]).toBeLessThanOrEqual(c.maintCurve[i - 1]);
    }
});

test('projectionCurves: buffer crossover at runway_pct', () => {
    // Runway 33% — at 33% decline, equity ≈ maintenance (buffer ≈ 0).
    const eq = 50_000, pos = 100_000, mp = 0.25;
    const c = projectionCurves(eq, pos, mp, 100, 0.5);
    const idxAt33 = Math.round((1 / 3) / 0.5 * 100);
    // Pick a wider tolerance — projection grid is discrete.
    expect(Math.abs(c.bufferCurve[idxAt33])).toBeLessThan(500);
});

test('projectionCurves: empty / zero-position safe', () => {
    expect(projectionCurves(0, 0, 0.25).xs).toEqual([]);
    expect(projectionCurves(NaN, 100, 0.25).xs).toEqual([]);
});

test('projectionCurves: xs span [0, maxDeclinePct]', () => {
    const c = projectionCurves(100_000, 100_000, 0.25, 10, 0.3);
    expect(c.xs[0]).toBe(0);
    expect(c.xs[c.xs.length - 1]).toBeCloseTo(0.3, 9);
});

// ── demos invariants ──────────────────────────────────────────────

test('demos: each preset produces valid validateInputs', () => {
    for (const k of ['safe','moderate','tight','critical','in-call','pdt-leveraged','concentrated','cash-only']) {
        const d = makeDemoInputs(k);
        expect(validateInputs(d.account_equity, d.position_value, d.maintenance_req_pct)).toBe(null);
    }
});

test('demo safe: runway ≈ 100%', () => {
    const d = makeDemoInputs('safe');
    const r = localCompute(d.account_equity, d.position_value, d.maintenance_req_pct);
    expect(r.runway_pct).toBeCloseTo(1.0, 9);
});

test('demo moderate: runway ≈ 33%', () => {
    const d = makeDemoInputs('moderate');
    const r = localCompute(d.account_equity, d.position_value, d.maintenance_req_pct);
    expect(r.runway_pct).toBeCloseTo(1 / 3, 9);
});

test('demo critical: runway < 5%, badge=critical', () => {
    const d = makeDemoInputs('critical');
    const r = localCompute(d.account_equity, d.position_value, d.maintenance_req_pct);
    expect(r.runway_pct).toBeLessThan(0.05);
    expect(runwayBadge(r).key).toMatch(/critical/);
});

test('demo in-call: badge=in_call', () => {
    const d = makeDemoInputs('in-call');
    const r = localCompute(d.account_equity, d.position_value, d.maintenance_req_pct);
    expect(r.already_in_margin_call).toBe(true);
    expect(runwayBadge(r).key).toMatch(/in_call/);
});

test('demo cash-only: no position, no risk', () => {
    const d = makeDemoInputs('cash-only');
    expect(d.position_value).toBe(0);
});

// ── DEFAULT_INPUTS / formatters ───────────────────────────────────

test('DEFAULT_INPUTS matches sensible defaults', () => {
    expect(DEFAULT_INPUTS).toEqual({
        account_equity: 50_000, position_value: 100_000, maintenance_req_pct: 0.25,
    });
});

test('formatters: USD / signed / pct / maintPct + non-finite guards', () => {
    expect(fmtUSD(1234)).toBe('$1234');
    expect(fmtUSD(-100)).toBe('-$100');
    expect(fmtUSDSigned(100)).toBe('+$100');
    expect(fmtUSDSigned(-100)).toBe('-$100');
    expect(fmtPct(0.0123, 2)).toBe('1.23%');
    expect(fmtMaintPct(0.25)).toBe('25%');
    expect(fmtUSD(NaN)).toBe('—');
    expect(fmtPct(null)).toBe('—');
});
