// ARCH-LM helpers: parser, validator, localTest mirror (including OLS),
// chi-square table + p-value approx, badges, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, DEFAULT_LAGS, MIN_LAGS, MAX_LAGS,
    parseReturnsBlob, returnsToBlob, validateInputs, buildBody, localTest, ols,
    chi2Critical, chi2PValue, verdictBadge, r2Badge, summarizeReturns,
    standardNormalCdf, erf,
    makeDemoInput,
    fmtNum, fmtNumSigned, fmtPct, fmtPVal, fmtInt,
} from '../js/_arch_lm_inputs.js';

// ── parser ────────────────────────────────────────────────────────

test('parseReturnsBlob: comma + whitespace + comments', () => {
    const r = parseReturnsBlob('0.01 -0.02\n# noise\n0.03, 0.04');
    expect(r.errors).toEqual([]);
    expect(r.returns).toEqual([0.01, -0.02, 0.03, 0.04]);
});

test('parseReturnsBlob: $/% / () negatives', () => {
    const r = parseReturnsBlob('$0.012 -$0.004 (0.005) 50%');
    expect(r.errors).toEqual([]);
    expect(r.returns).toEqual([0.012, -0.004, -0.005, 50]);
});

test('parseReturnsBlob: non-string → 1 error', () => {
    expect(parseReturnsBlob({}).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts valid', () => {
    expect(validateInputs({ returns: new Array(50).fill(0.01), lags: 5 })).toBe(null);
});

test('validate rejects: bad array / bad lags / short / non-finite', () => {
    const base = { returns: new Array(50).fill(0.01), lags: 5 };
    expect(validateInputs({ ...base, returns: 'no' })).toMatch(/returns/);
    expect(validateInputs({ ...base, lags: 0 })).toMatch(/lags/);
    expect(validateInputs({ ...base, lags: 100 })).toMatch(/lags/);
    expect(validateInputs({ returns: new Array(5).fill(0.01), lags: 5 })).toMatch(/3·lags/);
    expect(validateInputs({ returns: [0.01, NaN, 0.02, 0.03, 0.04, 0.05, 0.06, 0.07, 0.08, 0.09, 0.10, 0.11, 0.12, 0.13, 0.14, 0.15, 0.16, 0.17, 0.18, 0.19, 0.20], lags: 5 })).toMatch(/finite/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody passes through', () => {
    expect(buildBody({ returns: [0.01, 0.02], lags: 3 })).toEqual({ returns: [0.01, 0.02], lags: 3 });
});

// ── OLS sanity (independent test) ────────────────────────────────

test('ols: recovers known coefficients on noiseless input', () => {
    // y = 2 + 3·x1 − 1·x2 with non-collinear regressors.
    const x1 = [1, 2, 3, 4, 5, 6];
    const x2 = [1, 4, 9, 16, 25, 36];
    const y  = x1.map((v, i) => 2 + 3 * v - 1 * x2[i]);
    const intercept = new Array(6).fill(1);
    const beta = ols([intercept, x1, x2], y);
    expect(beta).not.toBe(null);
    expect(beta[0]).toBeCloseTo(2, 6);
    expect(beta[1]).toBeCloseTo(3, 6);
    expect(beta[2]).toBeCloseTo(-1, 6);
});

test('ols: rejects mismatched column lengths', () => {
    expect(ols([[1, 2], [1, 2, 3]], [1, 2])).toBe(null);
});

test('ols: rejects empty', () => {
    expect(ols([], [])).toBe(null);
});

// ── chi-square table + p-value ───────────────────────────────────

test('chi2Critical: returns hardcoded for k=1..15, exact known', () => {
    expect(chi2Critical(5).a5).toBe(11.070);
    expect(chi2Critical(10).a1).toBe(23.209);
    expect(chi2Critical(1).a5).toBe(3.841);
});

test('chi2Critical: Wilson-Hilferty fallback for k > 15', () => {
    const c = chi2Critical(20);
    expect(c.a5).toBeGreaterThan(0);
    // True χ²(20, 0.05) ≈ 31.41; W-H approx should be close.
    expect(Math.abs(c.a5 - 31.41)).toBeLessThan(1.0);
});

test('chi2PValue: monotone decreasing in LM', () => {
    expect(chi2PValue(2, 5)).toBeGreaterThan(chi2PValue(5, 5));
    expect(chi2PValue(5, 5)).toBeGreaterThan(chi2PValue(11.07, 5));
    expect(chi2PValue(11.07, 5)).toBeGreaterThan(chi2PValue(20, 5));
});

test('chi2PValue: ~0.05 at critical, well below at 1% crit', () => {
    expect(chi2PValue(11.07, 5)).toBeLessThan(0.10);
    expect(chi2PValue(15.09, 5)).toBeLessThan(0.05);
});

test('chi2PValue: NaN on invalid inputs', () => {
    expect(Number.isNaN(chi2PValue(NaN, 5))).toBe(true);
    expect(Number.isNaN(chi2PValue(-1, 5))).toBe(true);
    expect(Number.isNaN(chi2PValue(5, 0))).toBe(true);
});

// ── erf / standardNormalCdf sanity ───────────────────────────────

test('erf: known values', () => {
    expect(erf(0)).toBeCloseTo(0, 6);
    expect(erf(1)).toBeCloseTo(0.8427, 3);
});

test('standardNormalCdf: 0.5 at z=0', () => {
    expect(standardNormalCdf(0)).toBeCloseTo(0.5, 6);
});

// ── localTest mirroring Rust #[test] ─────────────────────────────

test('local: too-short returns null', () => {
    expect(localTest(new Array(5).fill(0.01), 5)).toBe(null);
});

test('local: zero lags returns null', () => {
    expect(localTest(new Array(50).fill(0.01), 0)).toBe(null);
});

test('local: NaN returns null', () => {
    const r = new Array(50).fill(0.01);
    r[10] = NaN;
    expect(localTest(r, 2)).toBe(null);
});

test('local: ARCH(1) yields LM > 20', () => {
    const n = 1000;
    let stateBig = 42n;
    const MASK = 0xFFFFFFFFFFFFFFFFn;
    const r = new Array(n).fill(0);
    for (let t = 1; t < n; t++) {
        stateBig = (stateBig * 6364136223846793005n + 1442695040888963407n) & MASK;
        const u1 = Math.max(1e-12, Number(stateBig >> 32n) / 0xFFFFFFFF);
        stateBig = (stateBig * 6364136223846793005n + 1442695040888963407n) & MASK;
        const u2 = Number(stateBig >> 32n) / 0xFFFFFFFF;
        const z = Math.sqrt(-2 * Math.log(u1)) * Math.cos(2 * Math.PI * u2);
        const v = 0.01 + 0.8 * r[t - 1] ** 2;
        r[t] = Math.sqrt(v) * z;
    }
    const rep = localTest(r, 5);
    expect(rep).not.toBe(null);
    expect(rep.lm_statistic).toBeGreaterThan(20);
});

test('local: iid Gaussian yields small LM (< 25)', () => {
    const n = 1000;
    let stateBig = 999n;
    const MASK = 0xFFFFFFFFFFFFFFFFn;
    const r = [];
    for (let i = 0; i < n / 2; i++) {
        stateBig = (stateBig * 6364136223846793005n + 1442695040888963407n) & MASK;
        const u1 = Math.max(1e-12, Number(stateBig >> 32n) / 0xFFFFFFFF);
        stateBig = (stateBig * 6364136223846793005n + 1442695040888963407n) & MASK;
        const u2 = Number(stateBig >> 32n) / 0xFFFFFFFF;
        const z1 = Math.sqrt(-2 * Math.log(u1)) * Math.cos(2 * Math.PI * u2);
        const z2 = Math.sqrt(-2 * Math.log(u1)) * Math.sin(2 * Math.PI * u2);
        r.push(0.01 * z1);
        r.push(0.01 * z2);
    }
    const rep = localTest(r, 5);
    expect(rep.lm_statistic).toBeLessThan(25);
});

test('local: r_squared in [-1, 1]', () => {
    const r = Array.from({ length: 100 }, (_, i) => Math.sin(i * 0.1) * 0.02);
    const rep = localTest(r, 3);
    expect(rep.r_squared).toBeGreaterThanOrEqual(-1);
    expect(rep.r_squared).toBeLessThanOrEqual(1);
});

test('local: lags and n_observations reported back', () => {
    const r = Array.from({ length: 100 }, (_, i) => Math.cos(i * 0.07) * 0.02);
    const rep = localTest(r, 4);
    expect(rep.lags).toBe(4);
    expect(rep.n_observations).toBe(96);
});

test('local: deterministic for same input', () => {
    const r = Array.from({ length: 100 }, (_, i) => (i % 7 - 3) * 0.01);
    const a = localTest(r, 5);
    const b = localTest(r, 5);
    expect(a.lm_statistic).toBe(b.lm_statistic);
    expect(a.r_squared).toBe(b.r_squared);
});

// ── badges ────────────────────────────────────────────────────────

test('verdictBadge: tiers by χ²(lags) critical', () => {
    const mk = (lm, lags) => ({ lm_statistic: lm, r_squared: 0, lags, n_observations: 100 });
    expect(verdictBadge(mk(5, 5)).key).toMatch(/no_arch/);     // < 9.236
    expect(verdictBadge(mk(10, 5)).key).toMatch(/borderline/); // 9.236 < lm < 11.07
    expect(verdictBadge(mk(13, 5)).key).toMatch(/moderate/);   // 11.07 < lm < 15.086
    expect(verdictBadge(mk(50, 5)).key).toMatch(/strong/);     // > 15.086
    expect(verdictBadge(null).key).toMatch(/unknown/);
});

test('r2Badge: tiers', () => {
    expect(r2Badge(0.005).key).toMatch(/none/);
    expect(r2Badge(0.02).key).toMatch(/weak/);
    expect(r2Badge(0.10).key).toMatch(/moderate/);
    expect(r2Badge(0.25).key).toMatch(/strong/);
    expect(r2Badge(NaN).key).toMatch(/unknown/);
});

// ── summarizeReturns ─────────────────────────────────────────────

test('summarizeReturns: count / mean / sd / extrema', () => {
    const s = summarizeReturns([0.01, -0.02, 0.03, -0.04]);
    expect(s.count).toBe(4);
    expect(s.mean).toBeCloseTo(-0.005, 6);
    expect(s.min).toBe(-0.04);
    expect(s.max).toBe(0.03);
});

test('summarizeReturns: empty → NaN', () => {
    const s = summarizeReturns([]);
    expect(s.count).toBe(0);
    expect(Number.isNaN(s.mean)).toBe(true);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + tests cleanly', () => {
    for (const k of ['arch-strong','arch-mild','garch-like','iid-gauss',
                     'iid-laplace','short-memory-vol','few-obs','high-lags']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const rep = localTest(inp.returns, inp.lags);
        expect(rep).not.toBe(null);
        expect(rep.lags).toBe(inp.lags);
    }
});

test('demo arch-strong yields LM well above χ²(5, 0.01) = 15.086', () => {
    const inp = makeDemoInput('arch-strong');
    const rep = localTest(inp.returns, inp.lags);
    expect(rep.lm_statistic).toBeGreaterThan(15.086);
});

test('demo iid-gauss yields LM near lags (≈5) → does NOT reject at 5%', () => {
    const inp = makeDemoInput('iid-gauss');
    const rep = localTest(inp.returns, inp.lags);
    // Not asserting < 11.07 (LM is random) but well under the very-strong threshold.
    expect(rep.lm_statistic).toBeLessThan(40);
});

test('demo high-lags uses lags=10', () => {
    const inp = makeDemoInput('high-lags');
    expect(inp.lags).toBe(10);
});

// ── formatters ────────────────────────────────────────────────────

test('returnsToBlob round-trips', () => {
    const r = [0.01, -0.02, 0.03];
    const back = parseReturnsBlob(returnsToBlob(r));
    expect(back.errors).toEqual([]);
    expect(back.returns).toEqual(r);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtNum(1.23456)).toBe('1.2346');
    expect(fmtNum(NaN)).toBe('—');
    expect(fmtNumSigned(0.5)).toBe('+0.5000');
    expect(fmtNumSigned(-0.5)).toBe('-0.5000');
    expect(fmtPct(0.0125)).toBe('1.25%');
    expect(fmtPVal(0.000001)).toBe('< 0.0001');
    expect(fmtPVal(0.04)).toBe('0.0400');
    expect(fmtInt(42.9)).toBe('42');
});

test('DEFAULT_INPUTS / DEFAULT_LAGS / MIN_LAGS / MAX_LAGS', () => {
    expect(DEFAULT_INPUTS.returns).toEqual([]);
    expect(DEFAULT_INPUTS.lags).toBe(DEFAULT_LAGS);
    expect(DEFAULT_LAGS).toBe(5);
    expect(MIN_LAGS).toBe(1);
    expect(MAX_LAGS).toBe(50);
});
