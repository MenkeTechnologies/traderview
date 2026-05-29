// Carhart 4-factor helpers: parser, validator, localCompute parity (OLS w/ SE), badges.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, MIN_OBS,
    parseSeriesBlob, seriesToBlob, validateInputs, buildBody, localCompute, olsWithSe,
    alphaBadge, styleBadge, fitBadge, marketBetaBadge, summarizeSeries,
    makeDemoInput,
    fmtBeta, fmtBetaSigned, fmtPct, fmtInt, fmtTStat,
} from '../js/_carhart4_inputs.js';

// ── parser ────────────────────────────────────────────────────────

test('parseSeriesBlob: 6-token lines', () => {
    const r = parseSeriesBlob('0.01 0.012 0.003 -0.001 0.005 0.00005\n# noise\n0.02 0.013 0.001 0.002 0.004 0.00005');
    expect(r.errors).toEqual([]);
    expect(r.portfolio_returns).toEqual([0.01, 0.02]);
    expect(r.market_excess).toEqual([0.012, 0.013]);
});

test('parseSeriesBlob: rejects wrong count', () => {
    expect(parseSeriesBlob('0.01 0.012 0.003').errors[0].message).toMatch(/6 tokens/);
});

test('parseSeriesBlob: rejects non-finite', () => {
    expect(parseSeriesBlob('0.01 NaN 0.003 -0.001 0.005 0').errors[0].message).toMatch(/finite/);
});

test('parseSeriesBlob: non-string returns 1 error', () => {
    expect(parseSeriesBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts valid', () => {
    const arr = new Array(15).fill(0.01);
    expect(validateInputs({
        portfolio_returns: arr, market_excess: arr,
        smb: arr, hml: arr, wml: arr, risk_free: arr,
    })).toBe(null);
});

test('validate rejects: missing array / mismatched / short / non-number', () => {
    const arr = new Array(15).fill(0.01);
    const base = { portfolio_returns: arr, market_excess: arr, smb: arr,
                   hml: arr, wml: arr, risk_free: arr };
    expect(validateInputs({ ...base, smb: 'no' })).toMatch(/smb/);
    expect(validateInputs({ ...base, hml: arr.slice(0, 5) })).toMatch(/hml length/);
    expect(validateInputs({ ...base, portfolio_returns: arr.slice(0, 5), market_excess: arr.slice(0, 5),
                            smb: arr.slice(0, 5), hml: arr.slice(0, 5), wml: arr.slice(0, 5),
                            risk_free: arr.slice(0, 5) })).toMatch(/10 observations/);
    expect(validateInputs({ ...base, wml: arr.map((_, i) => i === 0 ? '0.1' : 0.01) })).toMatch(/wml\[0\]/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody copies all 6 arrays', () => {
    const inp = {
        portfolio_returns: [1], market_excess: [2], smb: [3], hml: [4], wml: [5], risk_free: [6],
    };
    const body = buildBody(inp);
    expect(body).toEqual(inp);
    expect(body.portfolio_returns).not.toBe(inp.portfolio_returns);
});

// ── OLS sanity ────────────────────────────────────────────────────

test('olsWithSe: recovers known coefficients on noiseless input', () => {
    // y = 1 + 2·x1 + 3·x2
    const x1 = [1, 2, 3, 4, 5, 6, 7];
    const x2 = [1, 4, 9, 16, 25, 36, 49];
    const y = x1.map((v, i) => 1 + 2 * v + 3 * x2[i]);
    const intercept = new Array(7).fill(1);
    const r = olsWithSe([intercept, x1, x2], y);
    expect(r.beta[0]).toBeCloseTo(1, 6);
    expect(r.beta[1]).toBeCloseTo(2, 6);
    expect(r.beta[2]).toBeCloseTo(3, 6);
    expect(r.se[0]).toBeLessThan(0.01);   // noiseless → tiny SE
});

test('olsWithSe: returns null on singular', () => {
    expect(olsWithSe([[1, 2], [2, 4]], [3, 6])).toBe(null);
});

test('olsWithSe: returns null on mismatched lengths', () => {
    expect(olsWithSe([[1, 2], [1, 2, 3]], [1, 2])).toBe(null);
});

// ── localCompute parity ──────────────────────────────────────────

test('local: dim mismatch returns null', () => {
    const arr = new Array(20).fill(0.01);
    expect(localCompute({
        portfolio_returns: arr, market_excess: arr, smb: arr,
        hml: arr, wml: arr, risk_free: arr.slice(0, 10),
    })).toBe(null);
});

test('local: too-short returns null', () => {
    const arr = new Array(5).fill(0.01);
    expect(localCompute({
        portfolio_returns: arr, market_excess: arr, smb: arr,
        hml: arr, wml: arr, risk_free: arr,
    })).toBe(null);
});

test('local: NaN rows skipped silently', () => {
    // Use varying SMB/HML/WML to avoid collinearity with intercept.
    const n = 100;
    const m = Array.from({ length: n }, (_, i) => Math.cos(i * 0.07) * 0.01);
    const s = Array.from({ length: n }, (_, i) => Math.sin(i * 0.13) * 0.005);
    const h = Array.from({ length: n }, (_, i) => Math.cos(i * 0.11) * 0.005);
    const w = Array.from({ length: n }, (_, i) => Math.sin(i * 0.17) * 0.004);
    const rf = new Array(n).fill(0);
    const p = m.map((mi, i) => 0.001 + 1.0 * mi + 0.3 * s[i] + 0.1 * h[i] + 0.5 * w[i]);
    p[20] = NaN;
    const r = localCompute({
        portfolio_returns: p, market_excess: m, smb: s, hml: h, wml: w, risk_free: rf,
    });
    expect(r).not.toBe(null);
    expect(r.n_observations).toBeLessThan(100);
});

function lcg(seed) {
    let s = BigInt(seed);
    const MASK = 0xFFFFFFFFFFFFFFFFn;
    return () => {
        s = (s * 6364136223846793005n + 1442695040888963407n) & MASK;
        return Number(s >> 32n) / 0xFFFFFFFF;
    };
}

test('local: recovers known beta_wml = 0.8 momentum tilt', () => {
    const rand = lcg(99);
    const n = 500;
    const m = [], s = [], h = [], w = [], rf = [], p = [];
    for (let i = 0; i < n; i++) {
        const mi = (rand() - 0.5) * 0.04;
        const si = (rand() - 0.5) * 0.03;
        const hi = (rand() - 0.5) * 0.025;
        const wi = (rand() - 0.5) * 0.02;
        const rfi = 0.00005;
        const eps = (rand() - 0.5) * 0.005;
        const pi = rfi + 0.001 + 0.9 * mi + 0 + 0 + 0.8 * wi + eps;
        m.push(mi); s.push(si); h.push(hi); w.push(wi); rf.push(rfi); p.push(pi);
    }
    const r = localCompute({
        portfolio_returns: p, market_excess: m, smb: s, hml: h, wml: w, risk_free: rf,
    });
    expect(Math.abs(r.beta_wml - 0.8)).toBeLessThan(0.1);
});

test('local: alpha + r_squared in plausible ranges', () => {
    const inp = makeDemoInput('positive-alpha');
    const r = localCompute(inp);
    expect(r.r_squared).toBeGreaterThan(0);
    expect(r.r_squared).toBeLessThanOrEqual(1);
});

test('local: deterministic', () => {
    const inp = makeDemoInput('market-only');
    expect(localCompute(inp)).toEqual(localCompute(inp));
});

test('local: alpha_tstat = alpha / alpha_se when se > 0', () => {
    const inp = makeDemoInput('positive-alpha');
    const r = localCompute(inp);
    if (r.alpha_se > 0) {
        expect(Math.abs(r.alpha_tstat - r.alpha / r.alpha_se)).toBeLessThan(1e-9);
    }
});

// ── badges ────────────────────────────────────────────────────────

test('alphaBadge: 5 tiers', () => {
    const mk = (a, t) => ({ alpha: a, alpha_tstat: t,
                             beta_mkt: 1, beta_smb: 0, beta_hml: 0, beta_wml: 0,
                             alpha_se: 0.001, beta_mkt_se: 0, beta_smb_se: 0, beta_hml_se: 0, beta_wml_se: 0,
                             r_squared: 0.5, n_observations: 100 });
    expect(alphaBadge(mk(0.01, 3.0)).key).toMatch(/strong_pos/);
    expect(alphaBadge(mk(0.01, 2.2)).key).toMatch(/significant_pos/);
    expect(alphaBadge(mk(-0.01, -3.0)).key).toMatch(/strong_neg/);
    expect(alphaBadge(mk(-0.01, -2.2)).key).toMatch(/significant_neg/);
    expect(alphaBadge(mk(0.001, 0.5)).key).toMatch(/insignificant/);
    expect(alphaBadge(null).key).toMatch(/unknown/);
});

test('styleBadge: each tilt + market_neutral + multi', () => {
    const mk = (smb, hml, wml) => ({ beta_smb: smb, beta_hml: hml, beta_wml: wml,
                                       alpha: 0, beta_mkt: 1, alpha_se: 0, beta_mkt_se: 0,
                                       beta_smb_se: 0, beta_hml_se: 0, beta_wml_se: 0,
                                       alpha_tstat: 0, r_squared: 0.5, n_observations: 100 });
    expect(styleBadge(mk(0, 0, 0)).key).toMatch(/market_neutral/);
    expect(styleBadge(mk(0.5, 0, 0)).key).toMatch(/small/);
    expect(styleBadge(mk(-0.5, 0, 0)).key).toMatch(/large/);
    expect(styleBadge(mk(0, 0.5, 0)).key).toMatch(/value/);
    expect(styleBadge(mk(0, -0.5, 0)).key).toMatch(/growth/);
    expect(styleBadge(mk(0, 0, 0.5)).key).toMatch(/momentum/);
    expect(styleBadge(mk(0, 0, -0.5)).key).toMatch(/contrarian/);
    expect(styleBadge(mk(0.5, 0.5, 0.5)).key).toMatch(/multi/);
    expect(styleBadge(null).key).toMatch(/unknown/);
});

test('fitBadge: 5 tiers', () => {
    expect(fitBadge(0.95).key).toMatch(/excellent/);
    expect(fitBadge(0.80).key).toMatch(/good/);
    expect(fitBadge(0.50).key).toMatch(/moderate/);
    expect(fitBadge(0.20).key).toMatch(/weak/);
    expect(fitBadge(0.05).key).toMatch(/poor/);
    expect(fitBadge(NaN).key).toMatch(/unknown/);
});

test('marketBetaBadge: 5 classes', () => {
    expect(marketBetaBadge(-1).key).toMatch(/inverse/);
    expect(marketBetaBadge(0.3).key).toMatch(/low/);
    expect(marketBetaBadge(1.0).key).toMatch(/market/);
    expect(marketBetaBadge(2.0).key).toMatch(/high/);
    expect(marketBetaBadge(3.0).key).toMatch(/leveraged/);
    expect(marketBetaBadge(NaN).key).toMatch(/unknown/);
});

// ── summarize ─────────────────────────────────────────────────────

test('summarizeSeries: counts + means', () => {
    const inp = {
        portfolio_returns: [0.01, 0.02], market_excess: [0.012, 0.013],
        smb: [0.001, 0.002], hml: [0, 0], wml: [0, 0], risk_free: [0.00005, 0.00005],
    };
    const s = summarizeSeries(inp);
    expect(s.n).toBe(2);
    expect(s.mean_p).toBeCloseTo(0.015, 6);
    expect(s.mean_m).toBeCloseTo(0.0125, 6);
});

test('summarizeSeries: empty → 0 + NaN', () => {
    const s = summarizeSeries({ portfolio_returns: [], market_excess: [],
                                  smb: [], hml: [], wml: [], risk_free: [] });
    expect(s.n).toBe(0);
    expect(Number.isNaN(s.mean_p)).toBe(true);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + computes', () => {
    for (const k of ['market-only','small-cap-tilt','value-tilt','momentum-tilt',
                     'growth-tilt','positive-alpha','negative-alpha','small-sample']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp);
        expect(r).not.toBe(null);
        expect(r.n_observations).toBe(inp.portfolio_returns.length);
    }
});

test('demo small-cap-tilt: beta_smb ≈ 0.7', () => {
    const inp = makeDemoInput('small-cap-tilt');
    const r = localCompute(inp);
    expect(Math.abs(r.beta_smb - 0.7)).toBeLessThan(0.15);
});

test('demo momentum-tilt: beta_wml ≈ 0.8', () => {
    const inp = makeDemoInput('momentum-tilt');
    const r = localCompute(inp);
    expect(Math.abs(r.beta_wml - 0.8)).toBeLessThan(0.15);
});

test('demo positive-alpha: alpha > 0', () => {
    const inp = makeDemoInput('positive-alpha');
    const r = localCompute(inp);
    expect(r.alpha).toBeGreaterThan(0);
});

test('demo small-sample uses n=15', () => {
    const inp = makeDemoInput('small-sample');
    expect(inp.portfolio_returns.length).toBe(15);
});

// ── formatters ────────────────────────────────────────────────────

test('seriesToBlob round-trips through parseSeriesBlob', () => {
    const inp = {
        portfolio_returns: [0.01], market_excess: [0.012], smb: [0.003],
        hml: [-0.001], wml: [0.005], risk_free: [0.00005],
    };
    const back = parseSeriesBlob(seriesToBlob(inp));
    expect(back.errors).toEqual([]);
    expect(back.portfolio_returns).toEqual(inp.portfolio_returns);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtBeta(0.7654)).toBe('0.7654');
    expect(fmtBetaSigned(1.5)).toBe('+1.5000');
    expect(fmtBetaSigned(-1.5)).toBe('-1.5000');
    expect(fmtPct(0.0125)).toBe('1.2500%');
    expect(fmtInt(42.9)).toBe('42');
    expect(fmtTStat(2.5)).toBe('2.50');
    expect(fmtBeta(NaN)).toBe('—');
});

test('DEFAULTS sanity', () => {
    expect(DEFAULT_INPUTS.portfolio_returns).toEqual([]);
    expect(MIN_OBS).toBe(10);
});
