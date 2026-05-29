// Marginal/Component VaR helpers: parser, validator, body shape,
// localAnalyze Rust-mirror, badges, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_Z_ALPHA, Z_CONFIDENCE_LEVELS, DEFAULT_INPUTS,
    parsePortfolioBlob, portfolioToBlob, validateInputs, buildBody,
    localAnalyze, covFromVolsAndCorr,
    concentrationBadge, positionBadge,
    makeDemoInput,
    fmtPct, fmtPctNum, fmtNum, fmtInt, fmtSci, assetLabel,
} from '../js/_marginal_var_inputs.js';

// ── constants ─────────────────────────────────────────────────────

test('DEFAULT_Z_ALPHA = 1.645 (Rust 95%)', () => {
    expect(DEFAULT_Z_ALPHA).toBe(1.645);
});

test('Z_CONFIDENCE_LEVELS includes 95% / 99% / 99.9%', () => {
    const labels = Z_CONFIDENCE_LEVELS.map(c => c.label);
    expect(labels).toContain('95%');
    expect(labels).toContain('99%');
    expect(labels).toContain('99.9%');
});

// ── parser ────────────────────────────────────────────────────────

test('parsePortfolioBlob: weights then blank line then matrix', () => {
    const r = parsePortfolioBlob('SPY 0.3\nQQQ 0.4\nGLD 0.3\n\n0.04, 0.01, 0.0\n0.01, 0.09, 0.0\n0.0, 0.0, 0.16');
    expect(r.errors).toEqual([]);
    expect(r.labels).toEqual(['SPY', 'QQQ', 'GLD']);
    expect(r.weights).toEqual([0.3, 0.4, 0.3]);
    expect(r.covariance.length).toBe(3);
});

test('parsePortfolioBlob: single-token weight rows auto-label pos_N', () => {
    const r = parsePortfolioBlob('0.3\n0.4\n\n0.04, 0.01\n0.01, 0.09');
    // 2 weights and 2×2 matrix mismatches the labels — that's fine for parser; validator catches it.
    expect(r.errors).toEqual([]);
    expect(r.labels.slice(0, 2)).toEqual(['pos_1', 'pos_2']);
});

test('parsePortfolioBlob: needs 2 sections separated by blank line', () => {
    expect(parsePortfolioBlob('0.5 0.5').errors[0].message).toMatch(/2 sections/);
});

test('parsePortfolioBlob: non-finite cell rejected', () => {
    expect(parsePortfolioBlob('A 0.5\nB 0.5\n\n0.04 foo\n0.01 0.09').errors.length).toBeGreaterThan(0);
});

test('parsePortfolioBlob: non-string → 1 error', () => {
    expect(parsePortfolioBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts default', () => {
    expect(validateInputs(DEFAULT_INPUTS)).toBe(null);
});

test('validate rejects: missing portfolio / empty weights / dim mismatch / NaN / bad z_alpha', () => {
    expect(validateInputs({ z_alpha: 1.645 })).toMatch(/portfolio/);
    expect(validateInputs({ portfolio: { weights: [], covariance: [] }, z_alpha: 1.645 })).toMatch(/weights/);
    expect(validateInputs({ portfolio: { weights: [0.5, 0.5], covariance: [[1, 0]] }, z_alpha: 1.645 })).toMatch(/covariance/);
    expect(validateInputs({ portfolio: { weights: [NaN], covariance: [[0.04]] }, z_alpha: 1.645 })).toMatch(/weights/);
    expect(validateInputs({ portfolio: { weights: [0.5, 0.5], covariance: [[NaN, 0], [0, 1]] }, z_alpha: 1.645 })).toMatch(/covariance/);
    expect(validateInputs({ portfolio: { weights: [1], covariance: [[0.04]] }, z_alpha: 0 })).toMatch(/z_alpha/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: strips labels, keeps weights/covariance/z_alpha', () => {
    const body = buildBody({
        portfolio: { weights: [0.5, 0.5], covariance: [[0.04, 0], [0, 0.09]], labels: ['A', 'B'] },
        z_alpha: 1.96,
    });
    expect(body).toEqual({
        portfolio: { weights: [0.5, 0.5], covariance: [[0.04, 0], [0, 0.09]] },
        z_alpha: 1.96,
    });
});

// ── localAnalyze parity (mirrors every Rust #[test]) ─────────────

test('local: empty → null', () => {
    expect(localAnalyze({ weights: [], covariance: [] }, 1.645)).toBeNull();
});

test('local: dim mismatch → null', () => {
    expect(localAnalyze({ weights: [0.5, 0.5], covariance: [[1, 0]] }, 1.645)).toBeNull();
});

test('local: invalid z → null', () => {
    expect(localAnalyze({ weights: [1], covariance: [[0.04]] }, 0)).toBeNull();
    expect(localAnalyze({ weights: [1], covariance: [[0.04]] }, -1)).toBeNull();
    expect(localAnalyze({ weights: [1], covariance: [[0.04]] }, NaN)).toBeNull();
});

test('local: NaN inputs → null', () => {
    expect(localAnalyze({ weights: [NaN], covariance: [[0.04]] }, 1.645)).toBeNull();
    expect(localAnalyze({ weights: [1], covariance: [[NaN]] }, 1.645)).toBeNull();
});

test('local: single asset → VaR = σ · z, 100% contribution', () => {
    const r = localAnalyze({ weights: [1], covariance: [[0.04]] }, 1.645);
    expect(r.portfolio_var).toBeCloseTo(0.329, 3);
    expect(r.pct_contribution[0]).toBeCloseTo(100, 9);
});

test('local: 50/50 uncorrelated — sum of components = total, higher-vol contributes more', () => {
    const r = localAnalyze({
        weights: [0.5, 0.5],
        covariance: [[0.04, 0], [0, 0.09]],
    }, 1.645);
    const sum = r.component_var.reduce((s, v) => s + v, 0);
    expect(sum).toBeCloseTo(r.portfolio_var, 9);
    expect(r.pct_contribution[1]).toBeGreaterThan(r.pct_contribution[0]);
});

test('local: fully-hedged portfolio → all-zero (vol=0 branch)', () => {
    const r = localAnalyze({
        weights: [1, -1],
        covariance: [[0.04, 0.04], [0.04, 0.04]],
    }, 1.645);
    expect(r.portfolio_var).toBeCloseTo(0, 12);
    for (const v of r.pct_contribution) expect(Math.abs(v)).toBeLessThan(1e-12);
});

test('local: Σ component = portfolio_var (decomposition identity)', () => {
    const r = localAnalyze({
        weights: [0.3, 0.4, 0.3],
        covariance: [
            [0.04, 0.01, 0.005],
            [0.01, 0.09, 0.02],
            [0.005, 0.02, 0.16],
        ],
    }, 2.326);
    const sum = r.component_var.reduce((s, v) => s + v, 0);
    expect(sum).toBeCloseTo(r.portfolio_var, 9);
});

test('local: Σ pct = 100 (decomposition identity)', () => {
    const r = localAnalyze({
        weights: [0.3, 0.4, 0.3],
        covariance: [
            [0.04, 0.01, 0.005],
            [0.01, 0.09, 0.02],
            [0.005, 0.02, 0.16],
        ],
    }, 2.326);
    const sum = r.pct_contribution.reduce((s, v) => s + v, 0);
    expect(sum).toBeCloseTo(100, 9);
});

test('local: marginal_var = z · (Σw)_i / vol', () => {
    const r = localAnalyze({
        weights: [0.5, 0.5],
        covariance: [[0.04, 0], [0, 0.09]],
    }, 1.645);
    const sigma_w0 = 0.04 * 0.5 + 0 * 0.5;
    expect(r.marginal_var[0]).toBeCloseTo(1.645 * sigma_w0 / r.portfolio_vol, 9);
});

test('local: component_var[i] = w_i · marginal_var[i]', () => {
    const r = localAnalyze({
        weights: [0.5, 0.5],
        covariance: [[0.04, 0], [0, 0.09]],
    }, 1.645);
    for (let i = 0; i < 2; i++) {
        expect(r.component_var[i]).toBeCloseTo(0.5 * r.marginal_var[i], 9);
    }
});

test('local: higher z_alpha → larger portfolio_var (linear)', () => {
    const p = { weights: [1], covariance: [[0.04]] };
    const r95 = localAnalyze(p, 1.645);
    const r99 = localAnalyze(p, 2.326);
    expect(r99.portfolio_var / r95.portfolio_var).toBeCloseTo(2.326 / 1.645, 6);
});

// ── covFromVolsAndCorr ──────────────────────────────────────────

test('covFromVolsAndCorr: σ_i σ_j ρ_ij', () => {
    const c = covFromVolsAndCorr([0.2, 0.3], [[1, 0.5], [0.5, 1]]);
    expect(c[0][0]).toBeCloseTo(0.04, 9);
    expect(c[1][1]).toBeCloseTo(0.09, 9);
    expect(c[0][1]).toBeCloseTo(0.2 * 0.3 * 0.5, 9);
});

// ── concentrationBadge / positionBadge ───────────────────────────

test('concentrationBadge: 5-tier on max |pct|', () => {
    expect(concentrationBadge([10, 10, 10]).key).toMatch(/well_diversified/);
    expect(concentrationBadge([30, 30, 30]).key).toMatch(/balanced/);
    expect(concentrationBadge([50, 30, 20]).key).toMatch(/tilted/);
    expect(concentrationBadge([70, 20, 10]).key).toMatch(/concentrated/);
    expect(concentrationBadge([90, 5, 5]).key).toMatch(/extreme/);
    expect(concentrationBadge([]).key).toMatch(/unknown/);
});

test('positionBadge: under / fair / over / dominant', () => {
    // n=4 → equal = 25%
    expect(positionBadge(10, 4).key).toMatch(/under/);
    expect(positionBadge(25, 4).key).toMatch(/fair/);
    expect(positionBadge(50, 4).key).toMatch(/over/);
    expect(positionBadge(85, 4).key).toMatch(/dominant/);
    expect(positionBadge(NaN, 4).key).toMatch(/unknown/);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + analyzes to a non-null report', () => {
    for (const k of ['mixed-3','equal-uncorr','concentrated','hedged-pair',
                     'two-asset-corr','diversifier','99-pct-vad','tight-99-9']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localAnalyze(inp.portfolio, inp.z_alpha);
        expect(r).not.toBeNull();
        expect(r.component_var.length).toBe(inp.portfolio.weights.length);
    }
});

test('demo equal-uncorr: contributions are equal (≈ 33.33% each)', () => {
    const inp = makeDemoInput('equal-uncorr');
    const r = localAnalyze(inp.portfolio, inp.z_alpha);
    for (const p of r.pct_contribution) expect(p).toBeCloseTo(100 / 3, 6);
});

test('demo concentrated: max-pct position is the heaviest weight', () => {
    const inp = makeDemoInput('concentrated');
    const r = localAnalyze(inp.portfolio, inp.z_alpha);
    let maxI = 0;
    for (let i = 1; i < r.pct_contribution.length; i++) {
        if (r.pct_contribution[i] > r.pct_contribution[maxI]) maxI = i;
    }
    expect(maxI).toBe(0);   // first asset has 70% weight
});

test('demo hedged-pair: portfolio_var = 0 + every contribution = 0', () => {
    const inp = makeDemoInput('hedged-pair');
    const r = localAnalyze(inp.portfolio, inp.z_alpha);
    expect(r.portfolio_var).toBe(0);
    for (const p of r.pct_contribution) expect(p).toBe(0);
});

test('demo 99-pct-vad vs mixed-3: tighter z → larger VaR', () => {
    const a = localAnalyze(makeDemoInput('mixed-3').portfolio, 1.645);
    const b = localAnalyze(makeDemoInput('99-pct-vad').portfolio, 2.326);
    expect(b.portfolio_var).toBeGreaterThan(a.portfolio_var);
});

// ── round-trip + helpers ─────────────────────────────────────────

test('portfolioToBlob round-trips through parsePortfolioBlob', () => {
    const labels = ['SPY', 'QQQ'];
    const weights = [0.6, 0.4];
    const cov = [[0.04, 0.01], [0.01, 0.09]];
    const back = parsePortfolioBlob(portfolioToBlob(labels, weights, cov));
    expect(back.errors).toEqual([]);
    expect(back.labels).toEqual(labels);
    expect(back.weights).toEqual(weights);
    expect(back.covariance).toEqual(cov);
});

test('assetLabel: prefers supplied labels, falls back to A..Z, then A1..', () => {
    expect(assetLabel(['SPY','QQQ'], 0)).toBe('SPY');
    expect(assetLabel(null, 0)).toBe('A');
    expect(assetLabel(null, 25)).toBe('Z');
    expect(assetLabel(null, 26)).toBe('A1');
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtPct(25)).toBe('25.00%');
    expect(fmtPctNum(0.25)).toBe('25.00%');
    expect(fmtNum(1.23456789)).toBe('1.234568');
    expect(fmtInt(42.7)).toBe('42');
    expect(fmtSci(1e-6)).toMatch(/e/i);
    expect(fmtSci(0)).toBe('0');
    expect(fmtSci(NaN)).toBe('—');
    expect(fmtPct(NaN)).toBe('—');
});
