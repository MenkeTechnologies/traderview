// Bipower Variation helpers: parser, validator, body shape,
// localCompute Rust-mirror, badges, demos.

import { test, expect } from 'vitest';
import {
    MU1, THETA, DEFAULT_INPUTS,
    parseReturnsBlob, returnsToBlob, validateInputs, buildBody, localCompute,
    jumpBadge, jumpFractionBadge, jumpRatio,
    makeDemoInput,
    fmtVar, fmtZ, fmtP, fmtPct, fmtInt,
} from '../js/_bipower_variation_inputs.js';

// ── constants ─────────────────────────────────────────────────────

test('MU1 = √(2/π) (Rust constant)', () => {
    expect(MU1).toBeCloseTo(Math.sqrt(2 / Math.PI), 12);
});

test('THETA = π²/4 + π − 5 (Rust constant)', () => {
    expect(THETA).toBeCloseTo(Math.PI * Math.PI / 4 + Math.PI - 5, 12);
});

// ── parser ────────────────────────────────────────────────────────

test('parseReturnsBlob: decimals + pct-suffix + comments', () => {
    const r = parseReturnsBlob('0.012, -0.005\n# jump\n50%  0.001');
    expect(r.errors).toEqual([]);
    expect(r.returns).toEqual([0.012, -0.005, 0.50, 0.001]);
});

test('parseReturnsBlob: rejects non-finite', () => {
    expect(parseReturnsBlob('0.01, foo').errors[0].message).toMatch(/foo/);
});

test('parseReturnsBlob: non-string returns 1 error', () => {
    expect(parseReturnsBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts ≥ 4 returns', () => {
    expect(validateInputs({ returns: [0.01, -0.02, 0.005, 0.012] })).toBe(null);
});

test('validate rejects: bad array / too short / NaN', () => {
    expect(validateInputs({ returns: 'no' })).toMatch(/returns/);
    expect(validateInputs({ returns: [0.01, -0.02] })).toMatch(/at least 4/);
    expect(validateInputs({ returns: [0.01, NaN, 0.02, 0.01] })).toMatch(/finite/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: forwards returns verbatim', () => {
    expect(buildBody({ returns: [0.01] })).toEqual({ returns: [0.01] });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: too short → null', () => {
    expect(localCompute([0.01, -0.02])).toBeNull();
});

test('local: NaN input → null', () => {
    expect(localCompute([0.01, NaN, 0.02, 0.01])).toBeNull();
});

test('local: no-jump smooth path → BPV tracks RV within 30%', () => {
    let state = 12345n;
    const r = [];
    for (let i = 0; i < 500; i++) {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        const u1 = Math.max(1e-12, Number(state >> 32n) / 0xFFFFFFFF);
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        const u2 = Number(state >> 32n) / 0xFFFFFFFF;
        r.push(Math.sqrt(-2 * Math.log(u1)) * Math.cos(2 * Math.PI * u2) * 0.01);
    }
    const rep = localCompute(r);
    const rel = Math.abs(rep.realized_variance - rep.bipower_variation) / rep.realized_variance;
    expect(rel).toBeLessThan(0.30);
});

test('local: single big jump → RV > BPV, jump_variation > 0', () => {
    const r = new Array(200).fill(0.001);
    r[100] = 0.50;
    const rep = localCompute(r);
    expect(rep.realized_variance).toBeGreaterThan(rep.bipower_variation);
    expect(rep.jump_variation).toBeGreaterThan(0);
});

test('local: jumpy path → Huang-Tauchen z > 0', () => {
    const r = new Array(200).fill(0.001);
    r[100] = 0.50;
    const rep = localCompute(r);
    expect(rep.jump_test_z).toBeGreaterThan(0);
});

test('local: flat zero series → all components zero', () => {
    const rep = localCompute(new Array(100).fill(0));
    expect(rep.realized_variance).toBe(0);
    expect(rep.bipower_variation).toBe(0);
    expect(rep.jump_variation).toBe(0);
});

test('local: p-value in [0, 1]', () => {
    const r = [];
    for (let i = 0; i < 100; i++) r.push(Math.sin(i * 0.1) * 0.01);
    const rep = localCompute(r);
    expect(rep.jump_test_p_value).toBeGreaterThanOrEqual(0);
    expect(rep.jump_test_p_value).toBeLessThanOrEqual(1);
});

test('local: jump_variation = max(0, RV − BPV)', () => {
    const r = new Array(200).fill(0.001);
    r[100] = 0.50;
    const rep = localCompute(r);
    expect(rep.jump_variation).toBeCloseTo(Math.max(0, rep.realized_variance - rep.bipower_variation), 12);
});

test('local: n_observations equals input length', () => {
    const rep = localCompute(new Array(50).fill(0.01));
    expect(rep.n_observations).toBe(50);
});

test('local: BPV uses |r|·|r-1| pairs (validates first pair contribution)', () => {
    // 4 returns of magnitude 1 → BPV = (π/2)·(1·1 + 1·1 + 1·1) = (π/2)·3 ≈ 4.712.
    const rep = localCompute([1, 1, 1, 1]);
    expect(rep.bipower_variation).toBeCloseTo(Math.PI / 2 * 3, 9);
});

// ── jumpBadge / jumpFractionBadge / jumpRatio ────────────────────

test('jumpBadge: 5-tier by p-value', () => {
    expect(jumpBadge(0.0005).key).toMatch(/strong_jumps/);
    expect(jumpBadge(0.005).key).toMatch(/significant/);
    expect(jumpBadge(0.03).key).toMatch(/weak/);
    expect(jumpBadge(0.08).key).toMatch(/marginal/);
    expect(jumpBadge(0.5).key).toMatch(/no_jumps/);
    expect(jumpBadge(null).key).toMatch(/unknown/);
});

test('jumpFractionBadge: tiers by jump/RV', () => {
    expect(jumpFractionBadge(1.0, 0.6).key).toMatch(/dominant/);
    expect(jumpFractionBadge(1.0, 0.3).key).toMatch(/substantial/);
    expect(jumpFractionBadge(1.0, 0.10).key).toMatch(/moderate/);
    expect(jumpFractionBadge(1.0, 0.01).key).toMatch(/minor/);
    expect(jumpFractionBadge(1.0, 0).key).toMatch(/none/);
    expect(jumpFractionBadge(0, 0).key).toMatch(/unknown/);
});

test('jumpRatio: jump / rv (0 when rv ≤ 0)', () => {
    expect(jumpRatio(1, 0.25)).toBe(0.25);
    expect(jumpRatio(0, 0.5)).toBe(0);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + computes non-null report', () => {
    for (const k of ['no-jumps','single-big-jump','multi-small-jumps','flat-zero',
                     'high-vol-no-jumps','crash-down','short-series','persistent-vol']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.returns);
        expect(r).not.toBeNull();
    }
});

test('demo single-big-jump: jump_variation > 0', () => {
    const inp = makeDemoInput('single-big-jump');
    const r = localCompute(inp.returns);
    expect(r.jump_variation).toBeGreaterThan(0);
});

test('demo single-big-jump: z > 0', () => {
    const inp = makeDemoInput('single-big-jump');
    const r = localCompute(inp.returns);
    expect(r.jump_test_z).toBeGreaterThan(0);
});

test('demo no-jumps: |RV − BPV| / RV < 0.30', () => {
    const inp = makeDemoInput('no-jumps');
    const r = localCompute(inp.returns);
    const rel = Math.abs(r.realized_variance - r.bipower_variation) / r.realized_variance;
    expect(rel).toBeLessThan(0.30);
});

test('demo flat-zero: all components zero', () => {
    const inp = makeDemoInput('flat-zero');
    const r = localCompute(inp.returns);
    expect(r.realized_variance).toBe(0);
    expect(r.bipower_variation).toBe(0);
    expect(r.jump_variation).toBe(0);
});

test('demo crash-down: jump test significant (z > 1)', () => {
    const inp = makeDemoInput('crash-down');
    const r = localCompute(inp.returns);
    expect(r.jump_test_z).toBeGreaterThan(1);
});

test('demo short-series: minimal 5-bar input runs without error', () => {
    const inp = makeDemoInput('short-series');
    expect(inp.returns.length).toBe(5);
    const r = localCompute(inp.returns);
    expect(r).not.toBeNull();
});

// ── round-trip + formatters ──────────────────────────────────────

test('returnsToBlob round-trips through parseReturnsBlob', () => {
    const rs = [0.01, -0.02, 0.5];
    const back = parseReturnsBlob(returnsToBlob(rs));
    expect(back.errors).toEqual([]);
    expect(back.returns).toEqual(rs);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtVar(0.001)).toBe('0.001000');
    expect(fmtVar(1e-6)).toMatch(/e/i);
    expect(fmtZ(2.5)).toBe('+2.500');
    expect(fmtZ(-2.5)).toBe('-2.500');
    expect(fmtP(0.05)).toBe('0.0500');
    expect(fmtP(1e-6)).toMatch(/e/i);
    expect(fmtPct(0.25)).toBe('25.00%');
    expect(fmtInt(7.9)).toBe('7');
    expect(fmtVar(null)).toBe('—');
    expect(fmtZ(NaN)).toBe('—');
});
