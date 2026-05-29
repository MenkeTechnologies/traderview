// Black-Litterman helpers: parser, validator, body shape, localSolve
// Rust-mirror (Gauss-Jordan invert + matrix algebra), badges, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_TAU, DEFAULT_INPUTS,
    parseBlackLittermanBlob, blToBlob, validateInputs, buildBody, localSolve,
    confidenceBadge, tiltBadge,
    makeDemoInput,
    fmtPctSigned, fmtPct, fmtNum, fmtSci, fmtInt, assetLabel,
} from '../js/_black_litterman_inputs.js';

const baseInputs = () => ({
    covariance: [[0.04, 0.01], [0.01, 0.09]],
    equilibrium_returns: [0.05, 0.07],
    view_loadings: [[1, -1]],
    view_returns: [0.02],
    view_confidence: [[0.001]],
    tau: 0.05,
});

// ── constants ─────────────────────────────────────────────────────

test('DEFAULT_TAU = 0.05', () => {
    expect(DEFAULT_TAU).toBe(0.05);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts default', () => {
    expect(validateInputs(DEFAULT_INPUTS)).toBe(null);
});

test('validate rejects: missing inputs / bad equilibrium / dim mismatch / tau ≤ 0', () => {
    expect(validateInputs({})).toMatch(/inputs/);
    expect(validateInputs({ inputs: { ...baseInputs(), equilibrium_returns: [] } })).toMatch(/equilibrium_returns/);
    expect(validateInputs({ inputs: { ...baseInputs(), covariance: [[0.04]] } })).toMatch(/covariance/);
    expect(validateInputs({ inputs: { ...baseInputs(), tau: 0 } })).toMatch(/tau/);
    expect(validateInputs({ inputs: { ...baseInputs(), tau: NaN } })).toMatch(/tau/);
});

test('validate rejects: view dim mismatch / NaN / non-square confidence', () => {
    expect(validateInputs({ inputs: { ...baseInputs(), view_loadings: [[1, -1, 0]] } })).toMatch(/view_loadings/);
    expect(validateInputs({ inputs: { ...baseInputs(), view_confidence: [[NaN]] } })).toMatch(/view_confidence/);
});

test('validate accepts k=0 (no views)', () => {
    expect(validateInputs({ inputs: { ...baseInputs(),
        view_loadings: [], view_returns: [], view_confidence: [] } })).toBe(null);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: strips labels + view_labels but keeps numeric fields', () => {
    const body = buildBody({ inputs: { ...baseInputs(), labels: ['A','B'], view_labels: ['v1'] } });
    expect(body.inputs).toEqual(baseInputs());
});

// ── localSolve parity (mirrors every Rust #[test]) ───────────────

test('local: invalid inputs (empty cov / tau=0 / NaN tau) → null', () => {
    expect(localSolve({ ...baseInputs(), covariance: [] })).toBeNull();
    expect(localSolve({ ...baseInputs(), tau: 0 })).toBeNull();
    expect(localSolve({ ...baseInputs(), tau: NaN })).toBeNull();
});

test('local: dim mismatch → null', () => {
    expect(localSolve({ ...baseInputs(), equilibrium_returns: [0.05] })).toBeNull();
    expect(localSolve({ ...baseInputs(), view_loadings: [[1, -1, 0]] })).toBeNull();
});

test('local: no views (k=0) → posterior = prior', () => {
    const r = localSolve({ ...baseInputs(),
        view_loadings: [], view_returns: [], view_confidence: [] });
    expect(r.posterior_returns).toEqual([0.05, 0.07]);
    expect(r.posterior_covariance).toEqual([[0.04, 0.01], [0.01, 0.09]]);
});

test('local: very confident view (ω=1e-8) pulls posterior toward view', () => {
    const r = localSolve({ ...baseInputs(), view_confidence: [[1e-8]] });
    // View said "asset 1 − asset 2 = 0.02".
    const diff = r.posterior_returns[0] - r.posterior_returns[1];
    expect(Math.abs(diff - 0.02)).toBeLessThan(0.01);
});

test('local: very loose view (ω=1e8) leaves posterior close to prior', () => {
    const r = localSolve({ ...baseInputs(), view_confidence: [[1e8]] });
    expect(Math.abs(r.posterior_returns[0] - 0.05)).toBeLessThan(0.001);
    expect(Math.abs(r.posterior_returns[1] - 0.07)).toBeLessThan(0.001);
});

test('local: posterior covariance has correct dimensions', () => {
    const r = localSolve(baseInputs());
    expect(r.posterior_covariance.length).toBe(2);
    expect(r.posterior_covariance[0].length).toBe(2);
});

test('local: singular Ω → null', () => {
    expect(localSolve({ ...baseInputs(), view_confidence: [[0]] })).toBeNull();
});

test('local: posterior cov = Σ + A⁻¹ (must be ≥ Σ elementwise on diag)', () => {
    const r = localSolve(baseInputs());
    expect(r.posterior_covariance[0][0]).toBeGreaterThanOrEqual(baseInputs().covariance[0][0]);
    expect(r.posterior_covariance[1][1]).toBeGreaterThanOrEqual(baseInputs().covariance[1][1]);
});

test('local: posterior_returns has length n', () => {
    const r = localSolve(baseInputs());
    expect(r.posterior_returns.length).toBe(2);
});

test('local: 3-asset, 2-view solve produces finite output', () => {
    const r = localSolve({
        covariance: [[0.04, 0.005, 0.005], [0.005, 0.01, 0], [0.005, 0, 0.03]],
        equilibrium_returns: [0.06, 0.03, 0.04],
        view_loadings: [[1, -1, 0], [0, 0, 1]],
        view_returns: [0.04, 0.06],
        view_confidence: [[0.0005, 0], [0, 0.001]],
        tau: 0.05,
    });
    expect(r.posterior_returns.length).toBe(3);
    for (const v of r.posterior_returns) expect(Number.isFinite(v)).toBe(true);
});

// ── confidenceBadge / tiltBadge ──────────────────────────────────

test('confidenceBadge: very_high / high / medium / low / no_views', () => {
    expect(confidenceBadge([]).key).toMatch(/no_views/);
    expect(confidenceBadge([[1e-7]]).key).toMatch(/very_high/);
    expect(confidenceBadge([[1e-4]]).key).toMatch(/high/);
    expect(confidenceBadge([[1e-2]]).key).toMatch(/medium/);
    expect(confidenceBadge([[1.0]]).key).toMatch(/low/);
});

test('tiltBadge: strong_up / up / unchanged / down / strong_down', () => {
    expect(tiltBadge(0.02).key).toMatch(/strong_up/);
    expect(tiltBadge(0.005).key).toMatch(/up/);
    expect(tiltBadge(0).key).toMatch(/unchanged/);
    expect(tiltBadge(-0.005).key).toMatch(/down/);
    expect(tiltBadge(-0.02).key).toMatch(/strong_down/);
    expect(tiltBadge(NaN).key).toMatch(/unknown/);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + solves to a non-null report', () => {
    for (const k of ['two-asset-view','no-views','very-confident','very-loose',
                     'three-asset','two-views-conflict','low-tau','large-tau']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localSolve(inp.inputs);
        expect(r).not.toBeNull();
        expect(r.posterior_returns.length).toBe(inp.inputs.equilibrium_returns.length);
    }
});

test('demo no-views: posterior = prior exactly', () => {
    const inp = makeDemoInput('no-views');
    const r = localSolve(inp.inputs);
    expect(r.posterior_returns).toEqual(inp.inputs.equilibrium_returns);
});

test('demo very-confident: diff close to view (0.02)', () => {
    const inp = makeDemoInput('very-confident');
    const r = localSolve(inp.inputs);
    const diff = r.posterior_returns[0] - r.posterior_returns[1];
    expect(Math.abs(diff - 0.02)).toBeLessThan(0.01);
});

test('demo very-loose: posterior ≈ prior', () => {
    const inp = makeDemoInput('very-loose');
    const r = localSolve(inp.inputs);
    expect(Math.abs(r.posterior_returns[0] - 0.05)).toBeLessThan(0.001);
});

test('demo three-asset: posterior shifts every asset in some direction', () => {
    const inp = makeDemoInput('three-asset');
    const r = localSolve(inp.inputs);
    let anyShift = false;
    for (let i = 0; i < 3; i++) {
        if (Math.abs(r.posterior_returns[i] - inp.inputs.equilibrium_returns[i]) > 1e-6) anyShift = true;
    }
    expect(anyShift).toBe(true);
});

test('demo low-tau (0.001): weaker view pull than default τ=0.05 (posterior closer to prior)', () => {
    const lowT = localSolve(makeDemoInput('low-tau').inputs);
    const def = localSolve(makeDemoInput('two-asset-view').inputs);
    const lowShift = Math.abs(lowT.posterior_returns[0] - 0.05);
    const defShift = Math.abs(def.posterior_returns[0] - 0.05);
    // Lower tau = stronger prior = smaller shift away from equilibrium.
    expect(lowShift).toBeLessThan(defShift);
});

// ── parser ────────────────────────────────────────────────────────

test('parseBlackLittermanBlob: full 5-section round-trip', () => {
    const blob = 'A B\n\n0.05 0.07\n\n0.04, 0.01\n0.01, 0.09\n\ntau 0.05\n\nview_1 1 -1 0.02 0.001';
    const r = parseBlackLittermanBlob(blob);
    expect(r.errors).toEqual([]);
    expect(r.labels).toEqual(['A', 'B']);
    expect(r.equilibrium_returns).toEqual([0.05, 0.07]);
    expect(r.covariance).toEqual([[0.04, 0.01], [0.01, 0.09]]);
    expect(r.tau).toBe(0.05);
    expect(r.view_loadings).toEqual([[1, -1]]);
    expect(r.view_returns).toEqual([0.02]);
    expect(r.view_confidence).toEqual([[0.001]]);
    expect(r.view_labels).toEqual(['view_1']);
});

test('parseBlackLittermanBlob: 4 sections (no views) is accepted', () => {
    const blob = 'A B\n\n0.05 0.07\n\n0.04, 0.01\n0.01, 0.09\n\ntau 0.05';
    const r = parseBlackLittermanBlob(blob);
    expect(r.errors).toEqual([]);
    expect(r.view_returns).toEqual([]);
});

test('parseBlackLittermanBlob: too few sections → error', () => {
    expect(parseBlackLittermanBlob('A B\n\n0.05 0.07').errors.length).toBeGreaterThan(0);
});

test('parseBlackLittermanBlob: non-string returns 1 error', () => {
    expect(parseBlackLittermanBlob(null).errors.length).toBe(1);
});

// ── round-trip + formatters ──────────────────────────────────────

test('blToBlob round-trips through parseBlackLittermanBlob', () => {
    const inp = {
        labels: ['A', 'B'],
        equilibrium_returns: [0.05, 0.07],
        covariance: [[0.04, 0.01], [0.01, 0.09]],
        view_loadings: [[1, -1]],
        view_returns: [0.02],
        view_confidence: [[0.001]],
        tau: 0.05,
        view_labels: ['view_1'],
    };
    const back = parseBlackLittermanBlob(blToBlob(inp));
    expect(back.errors).toEqual([]);
    expect(back.labels).toEqual(inp.labels);
    expect(back.equilibrium_returns).toEqual(inp.equilibrium_returns);
    expect(back.covariance).toEqual(inp.covariance);
    expect(back.tau).toBe(inp.tau);
    expect(back.view_loadings).toEqual(inp.view_loadings);
    expect(back.view_returns).toEqual(inp.view_returns);
    expect(back.view_confidence).toEqual(inp.view_confidence);
});

test('assetLabel + fmt helpers + non-finite guards', () => {
    expect(assetLabel(['SPY', 'AGG'], 0)).toBe('SPY');
    expect(assetLabel(null, 0)).toBe('A');
    expect(assetLabel(null, 1)).toBe('B');
    expect(fmtPct(0.05)).toBe('5.00%');
    expect(fmtPctSigned(0.05)).toBe('+5.00%');
    expect(fmtPctSigned(-0.05)).toBe('-5.00%');
    expect(fmtNum(1.234567)).toBe('1.234567');
    expect(fmtSci(1e-6)).toMatch(/e/i);
    expect(fmtSci(0)).toBe('0');
    expect(fmtInt(7.9)).toBe('7');
    expect(fmtPct(NaN)).toBe('—');
    expect(fmtSci(NaN)).toBe('—');
});
