// Risk-parity helpers: parser, validator, body shape, Rust-mirror,
// equal-weight, dispersion + concentration diagnostics, demos.

import { test, expect } from 'vitest';
import {
    parseAssetBlob, validateInputs, buildBody, localAllocate,
    equalWeightAllocation, riskContribDispersion, maxConcentration,
    makeDemoAssets, fmtPct, fmtVol, fmtNum, symbolColor,
} from '../js/_risk_parity_inputs.js';

const a = (sym, vol) => ({ symbol: sym, vol });

// ── parser ────────────────────────────────────────────────────────

test('parser accepts 2 tokens + %-suffix + comments + upcases', () => {
    const r = parseAssetBlob('spy 15%\n# bond\nagg 0.05');
    expect(r.errors).toEqual([]);
    expect(r.assets).toEqual([a('SPY', 0.15), a('AGG', 0.05)]);
});

test('parser accepts zero vol (cash equivalent)', () => {
    expect(parseAssetBlob('CASH 0').errors).toEqual([]);
});

test('parser rejects negative vol / non-finite / wrong token count', () => {
    expect(parseAssetBlob('A -0.1').errors[0].message).toMatch(/vol/);
    expect(parseAssetBlob('A abc').errors[0].message).toMatch(/vol/);
    expect(parseAssetBlob('A').errors[0].message).toMatch(/2 tokens/);
});

test('parser rejects duplicate symbol (case-insensitive)', () => {
    const r = parseAssetBlob('SPY 0.15\nspy 0.20');
    expect(r.errors[0].message).toMatch(/duplicate/);
    expect(r.assets.length).toBe(1);
});

test('parser non-string returns 1 error', () => {
    expect(parseAssetBlob(null).errors.length).toBe(1);
});

// ── validator / buildBody ─────────────────────────────────────────

test('validate accepts ≥ 1 asset; rejects empty', () => {
    expect(validateInputs([a('SPY', 0.15)])).toBe(null);
    expect(validateInputs([])).toMatch(/≥ 1 asset/);
});

test('buildBody mirrors backend RiskParityBody', () => {
    expect(buildBody([a('SPY', 0.15)])).toEqual({ assets: [a('SPY', 0.15)] });
});

// ── localAllocate parity (one test per Rust property) ─────────────

test('local: empty → empty allocations + total_weight=0', () => {
    expect(localAllocate([])).toEqual({ allocations: [], total_weight: 0 });
});

test('local: equal-vol → equal weights', () => {
    const r = localAllocate([a('A', 0.20), a('B', 0.20), a('C', 0.20)]);
    for (const w of r.allocations) expect(w.weight).toBeCloseTo(1 / 3, 9);
});

test('local: high-vol asset gets smaller weight (proportional to inv-vol)', () => {
    const r = localAllocate([a('LOW', 0.10), a('HIGH', 0.40)]);
    const low = r.allocations.find(x => x.symbol === 'LOW');
    const high = r.allocations.find(x => x.symbol === 'HIGH');
    expect(low.weight).toBeGreaterThan(high.weight);
    expect(low.weight / high.weight).toBeCloseTo(4, 9);
});

test('local: weights sum to 1 (when at least one positive vol)', () => {
    const r = localAllocate([a('A', 0.10), a('B', 0.20), a('C', 0.30)]);
    expect(r.total_weight).toBeCloseTo(1, 9);
});

test('local: risk_contribution equal across all assets (definition of risk parity)', () => {
    const r = localAllocate([a('A', 0.10), a('B', 0.20), a('C', 0.30)]);
    const first = r.allocations[0].risk_contribution;
    for (const w of r.allocations) expect(w.risk_contribution).toBeCloseTo(first, 12);
});

test('local: zero-vol asset → 0 weight, others absorb its share', () => {
    const r = localAllocate([a('CASH', 0), a('SPY', 0.20)]);
    const cash = r.allocations.find(x => x.symbol === 'CASH');
    const spy  = r.allocations.find(x => x.symbol === 'SPY');
    expect(cash.weight).toBe(0);
    expect(spy.weight).toBe(1);
});

test('local: single asset → 100% weight', () => {
    expect(localAllocate([a('ONLY', 0.25)]).allocations[0].weight).toBe(1);
});

test('local: all-zero-vol → empty report (total inv-vol = 0)', () => {
    const r = localAllocate([a('A', 0), a('B', 0)]);
    expect(r.allocations).toEqual([]);
    expect(r.total_weight).toBe(0);
});

// ── equalWeightAllocation ─────────────────────────────────────────

test('equal-weight: 1/n per asset', () => {
    const r = equalWeightAllocation([a('A', 0.10), a('B', 0.20)]);
    expect(r[0].weight).toBe(0.5);
    expect(r[1].weight).toBe(0.5);
});

test('equal-weight: risk contribution = (1/n) × vol (NOT equal for mixed vols)', () => {
    const r = equalWeightAllocation([a('LOW', 0.10), a('HIGH', 0.40)]);
    expect(r[0].risk_contribution).toBeCloseTo(0.05, 9);
    expect(r[1].risk_contribution).toBeCloseTo(0.20, 9);
});

test('equal-weight: empty safe', () => {
    expect(equalWeightAllocation([])).toEqual([]);
});

// ── diagnostics ───────────────────────────────────────────────────

test('riskContribDispersion: ~0 for risk-parity allocation', () => {
    const r = localAllocate([a('A', 0.10), a('B', 0.20), a('C', 0.30)]);
    expect(riskContribDispersion(r.allocations)).toBeLessThan(1e-9);
});

test('riskContribDispersion: > 0 for equal-weight on mixed vols', () => {
    const eq = equalWeightAllocation([a('LOW', 0.10), a('HIGH', 0.40)]);
    expect(riskContribDispersion(eq)).toBeGreaterThan(0);
});

test('riskContribDispersion: empty safe', () => {
    expect(riskContribDispersion([])).toBe(0);
});

test('maxConcentration: returns largest single weight', () => {
    const r = localAllocate([a('CASH', 0), a('SPY', 0.20)]);
    expect(maxConcentration(r.allocations)).toBe(1);
});

test('maxConcentration: equal-vol 3-asset → 1/3', () => {
    const r = localAllocate([a('A', 0.20), a('B', 0.20), a('C', 0.20)]);
    expect(maxConcentration(r.allocations)).toBeCloseTo(1 / 3, 9);
});

// ── demo invariants ───────────────────────────────────────────────

test('demos: each preset has ≥ 1 valid asset', () => {
    for (const k of ['classic-60-40', 'five-asset', 'equal-vol', 'extreme-vol', 'single', 'zero-vol-mixed']) {
        const assets = makeDemoAssets(k);
        expect(assets.length).toBeGreaterThanOrEqual(1);
        for (const x of assets) {
            expect(typeof x.symbol).toBe('string');
            expect(x.vol).toBeGreaterThanOrEqual(0);
        }
    }
});

test('demo classic-60-40: AGG (low-vol) gets the bigger weight', () => {
    const r = localAllocate(makeDemoAssets('classic-60-40'));
    const spy = r.allocations.find(x => x.symbol === 'SPY');
    const agg = r.allocations.find(x => x.symbol === 'AGG');
    expect(agg.weight).toBeGreaterThan(spy.weight);
    // Vols are 0.15 vs 0.05 → ratio 3 → AGG weight = 3 × SPY weight.
    expect(agg.weight / spy.weight).toBeCloseTo(3, 9);
});

test('demo extreme-vol: STEADY gets 10× the VOLATILE weight (0.05 vs 0.50)', () => {
    const r = localAllocate(makeDemoAssets('extreme-vol'));
    const s = r.allocations.find(x => x.symbol === 'STEADY');
    const v = r.allocations.find(x => x.symbol === 'VOLATILE');
    expect(s.weight / v.weight).toBeCloseTo(10, 9);
});

test('demo zero-vol-mixed: CASH gets 0% weight, risk-asset weights still sum to 1', () => {
    const r = localAllocate(makeDemoAssets('zero-vol-mixed'));
    const cash = r.allocations.find(x => x.symbol === 'CASH');
    expect(cash.weight).toBe(0);
    expect(r.total_weight).toBeCloseTo(1, 9);
});

test('demo equal-vol: dispersion ≈ 0 AND equal-weight equals risk-parity', () => {
    const assets = makeDemoAssets('equal-vol');
    const rp = localAllocate(assets);
    const eq = equalWeightAllocation(assets);
    for (let i = 0; i < rp.allocations.length; i++) {
        expect(rp.allocations[i].weight).toBeCloseTo(eq[i].weight, 9);
    }
});

// ── formatters ────────────────────────────────────────────────────

test('fmt helpers + non-finite guards', () => {
    expect(fmtPct(0.25)).toBe('25.00%');
    expect(fmtVol(0.15)).toBe('15.0%');
    expect(fmtNum(0.123456, 4)).toBe('0.1235');
    expect(fmtPct(NaN)).toBe('—');
});

test('symbolColor cycles palette, neg id → muted', () => {
    expect(symbolColor(0)).toBe('#00e5ff');
    expect(symbolColor(7)).toBe('#00e5ff');
    expect(symbolColor(-1)).toBe('#aab');
});
