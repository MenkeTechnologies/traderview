// Beta Shrinkage helpers: parser, validator, localShrink mirror + OLS, badges, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, MIN_OBS,
    parseAssetsBlob, assetsToBlob, parseMarketBlob, marketToBlob,
    validateInputs, buildBody, localShrink, olsBeta,
    weightBadge, betaBadge, dispersionBadge,
    makeDemoInput,
    fmtNum, fmtNumSigned, fmtPct, fmtInt,
} from '../js/_beta_shrink_inputs.js';

// ── parser ────────────────────────────────────────────────────────

test('parseAssetsBlob: SYMBOL r1 r2 r3', () => {
    const r = parseAssetsBlob('AAPL 0.01 -0.02 0.03\n# noise\nMSFT 0.005, -0.01, 0.02');
    expect(r.errors).toEqual([]);
    expect(r.assets).toEqual([
        { symbol: 'AAPL', asset_returns: [0.01, -0.02, 0.03] },
        { symbol: 'MSFT', asset_returns: [0.005, -0.01, 0.02] },
    ]);
});

test('parseAssetsBlob: rejects 1-token line / non-finite token', () => {
    expect(parseAssetsBlob('AAPL').errors[0].message).toMatch(/SYMBOL/);
    expect(parseAssetsBlob('AAPL 0.01 NaNzzz').errors[0].message).toMatch(/finite/);
});

test('parseMarketBlob: numbers separated by ws/comma', () => {
    const r = parseMarketBlob('0.01 -0.02 0.03, 0.04');
    expect(r.errors).toEqual([]);
    expect(r.market_returns).toEqual([0.01, -0.02, 0.03, 0.04]);
});

test('parseMarketBlob: rejects non-finite', () => {
    expect(parseMarketBlob('0.01 zzz 0.02').errors[0].line_no).toBe(2);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts valid', () => {
    expect(validateInputs({
        assets: [{ symbol: 'A', asset_returns: [0.01, 0.02, 0.03, 0.04, 0.05] }],
        market_returns: [0.01, 0.02, 0.03, 0.04, 0.05],
    })).toBe(null);
});

test('validate rejects: bad arrays / empty / short market / non-finite / missing symbol', () => {
    expect(validateInputs({ assets: 'no', market_returns: [0.01, 0.02, 0.03, 0.04, 0.05] })).toMatch(/assets/);
    expect(validateInputs({ assets: [], market_returns: [0.01, 0.02, 0.03, 0.04, 0.05] })).toMatch(/at least one/);
    expect(validateInputs({ assets: [{ symbol: 'A', asset_returns: [0.01] }], market_returns: [0.01, 0.02] })).toMatch(/market_returns/);
    expect(validateInputs({ assets: [{ symbol: 'A', asset_returns: [0.01] }], market_returns: [0.01, NaN, 0.02, 0.03, 0.04] })).toMatch(/finite/);
    expect(validateInputs({ assets: [{ symbol: '', asset_returns: [0.01] }], market_returns: [0.01, 0.02, 0.03, 0.04, 0.05] })).toMatch(/symbol/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody copies + trims', () => {
    const inp = { assets: [{ symbol: 'A', asset_returns: [0.01, 0.02], extra: 'x' }],
                  market_returns: [0.01, 0.02] };
    const body = buildBody(inp);
    expect(body).toEqual({ assets: [{ symbol: 'A', asset_returns: [0.01, 0.02] }],
                          market_returns: [0.01, 0.02] });
});

// ── OLS sanity ────────────────────────────────────────────────────

test('olsBeta: recovers slope = 1 on identical y = x', () => {
    const x = [1, 2, 3, 4, 5];
    const y = [1, 2, 3, 4, 5];
    const r = olsBeta(y, x);
    expect(r.beta).toBeCloseTo(1, 9);
    expect(r.se).toBeCloseTo(0, 9);
});

test('olsBeta: recovers slope = 2 on y = 2x', () => {
    const x = [1, 2, 3, 4, 5, 6, 7];
    const y = x.map(v => 2 * v);
    const r = olsBeta(y, x);
    expect(r.beta).toBeCloseTo(2, 9);
});

test('olsBeta: returns null for flat x (no variance)', () => {
    expect(olsBeta([1, 2, 3, 4, 5], [1, 1, 1, 1, 1])).toBe(null);
});

test('olsBeta: returns null for too-short series', () => {
    expect(olsBeta([1, 2, 3], [1, 2, 3])).toBe(null);
});

test('olsBeta: returns null for mismatched lengths', () => {
    expect(olsBeta([1, 2, 3, 4, 5], [1, 2, 3])).toBe(null);
});

// ── localShrink parity (mirrors every Rust #[test]) ──────────────

test('local: empty assets returns null', () => {
    expect(localShrink([], [0.01, 0.02, 0.03, 0.04, 0.05])).toBe(null);
});

test('local: short market returns null', () => {
    const a = [{ symbol: 'X', asset_returns: [0.01, 0.02, 0.03] }];
    expect(localShrink(a, [0.01, 0.02, 0.03])).toBe(null);
});

test('local: NaN market returns null', () => {
    const m = [0.01, NaN, 0.02, 0.01, 0.02];
    const a = [{ symbol: 'X', asset_returns: [0.01, 0.02, 0.01, 0.02, 0.01] }];
    expect(localShrink(a, m)).toBe(null);
});

test('local: shrinkage_weight in [0, 1]', () => {
    const m = Array.from({ length: 30 }, (_, i) => Math.sin(i * 0.1) * 0.01);
    const assets = [
        { symbol: 'A', asset_returns: m.map(x => x * 1.2 + 0.001) },
        { symbol: 'B', asset_returns: m.map(x => x * 0.8 + 0.0005) },
        { symbol: 'C', asset_returns: m.map(x => x * 1.0 + 0.002) },
    ];
    const r = localShrink(assets, m);
    expect(r).not.toBe(null);
    for (const s of r.assets) {
        expect(s.shrinkage_weight).toBeGreaterThanOrEqual(0);
        expect(s.shrinkage_weight).toBeLessThanOrEqual(1);
    }
});

test('local: high SE shrinks more (tighter fit keeps more OLS weight)', () => {
    let state = 7n;
    const MASK = 0xFFFFFFFFFFFFFFFFn;
    const rand = () => {
        state = (state * 6364136223846793005n + 1442695040888963407n) & MASK;
        return Number(state >> 32n) / 0xFFFFFFFF - 0.5;
    };
    const m = Array.from({ length: 100 }, () => rand() * 0.02);
    const tight = m.map(x => x * 1.5 + rand() * 0.0005);
    const noisy = m.map(x => x * 1.5 + rand() * 0.05);
    const r = localShrink([{ symbol: 'TIGHT', asset_returns: tight },
                            { symbol: 'NOISY', asset_returns: noisy }], m);
    const t = r.assets.find(s => s.symbol === 'TIGHT').shrinkage_weight;
    const n = r.assets.find(s => s.symbol === 'NOISY').shrinkage_weight;
    expect(t).toBeGreaterThan(n);
});

test('local: beta_shrunk between beta_ols and prior_beta', () => {
    const m = Array.from({ length: 50 }, (_, i) => Math.sin(i * 0.1) * 0.01);
    const assets = [
        { symbol: 'HIGH', asset_returns: m.map(x => x * 2.0) },
        { symbol: 'LOW',  asset_returns: m.map(x => x * 0.5) },
        { symbol: 'MID',  asset_returns: m.map(x => x * 1.0) },
    ];
    const r = localShrink(assets, m);
    for (const s of r.assets) {
        const lo = Math.min(s.beta_ols, r.prior_beta);
        const hi = Math.max(s.beta_ols, r.prior_beta);
        expect(s.beta_shrunk).toBeGreaterThanOrEqual(lo - 1e-9);
        expect(s.beta_shrunk).toBeLessThanOrEqual(hi + 1e-9);
    }
});

test('local: mismatched lengths skipped', () => {
    const m = new Array(20).fill(0.01);
    const good = { symbol: 'OK', asset_returns: new Array(20).fill(0.01) };
    const bad  = { symbol: 'BAD', asset_returns: new Array(10).fill(0.01) };
    // Both assets have flat returns → OLS beta undefined (s_xx = 0 on market too).
    // Use sine market instead.
    const m2 = Array.from({ length: 20 }, (_, i) => Math.sin(i * 0.2) * 0.01);
    const good2 = { symbol: 'OK', asset_returns: m2.map(x => x * 1.5) };
    const bad2  = { symbol: 'BAD', asset_returns: m2.slice(0, 10) };
    const r = localShrink([good2, bad2], m2);
    expect(r.assets.length).toBe(1);
    expect(r.assets[0].symbol).toBe('OK');
});

test('local: flat market returns null (no x variance)', () => {
    const m = new Array(20).fill(0);
    const a = [{ symbol: 'X', asset_returns: new Array(20).fill(0.01) }];
    expect(localShrink(a, m)).toBe(null);
});

test('local: single asset → cs_var = 0 → w = 0 → shrunk = prior = beta_ols', () => {
    const m = Array.from({ length: 50 }, (_, i) => Math.sin(i * 0.1) * 0.01);
    const a = [{ symbol: 'ONLY', asset_returns: m.map(x => x * 1.2) }];
    const r = localShrink(a, m);
    expect(r.cross_sectional_variance).toBe(0);
    expect(r.assets[0].shrinkage_weight).toBe(0);
    // shrunk = 0 * beta + 1 * prior = prior; with single asset prior = beta_ols.
    expect(Math.abs(r.assets[0].beta_shrunk - r.assets[0].beta_ols)).toBeLessThan(1e-9);
});

test('local: deterministic', () => {
    const m = Array.from({ length: 30 }, (_, i) => Math.sin(i * 0.2) * 0.01);
    const a = [{ symbol: 'A', asset_returns: m.map(x => x * 1.3) }];
    expect(localShrink(a, m)).toEqual(localShrink(a, m));
});

// ── badges ────────────────────────────────────────────────────────

test('weightBadge: tiers', () => {
    expect(weightBadge(0.9).key).toMatch(/high/);
    expect(weightBadge(0.6).key).toMatch(/moderate/);
    expect(weightBadge(0.3).key).toMatch(/low/);
    expect(weightBadge(0.1).key).toMatch(/very_low/);
    expect(weightBadge(NaN).key).toMatch(/unknown/);
});

test('betaBadge: inverse / low / market / high / leveraged', () => {
    expect(betaBadge(-1.5).key).toMatch(/inverse/);
    expect(betaBadge(0.3).key).toMatch(/low/);
    expect(betaBadge(1.0).key).toMatch(/market/);
    expect(betaBadge(1.8).key).toMatch(/high/);
    expect(betaBadge(3.0).key).toMatch(/leveraged/);
    expect(betaBadge(NaN).key).toMatch(/unknown/);
});

test('dispersionBadge: tight / moderate / wide / very_wide / unknown', () => {
    expect(dispersionBadge(0.005, 5).key).toMatch(/tight/);     // sd ≈ 0.07
    expect(dispersionBadge(0.05, 5).key).toMatch(/moderate/);   // sd ≈ 0.22
    expect(dispersionBadge(0.20, 5).key).toMatch(/wide/);       // sd ≈ 0.45
    expect(dispersionBadge(1.0, 5).key).toMatch(/very_wide/);   // sd ≈ 1.0
    expect(dispersionBadge(0.1, 1).key).toMatch(/unknown/);     // n=1
    expect(dispersionBadge(NaN, 5).key).toMatch(/unknown/);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + shrinks (or null for short-series)', () => {
    for (const k of ['mixed','tight-vs-noisy','all-similar','sector-mix',
                     'inverse','short-series','mismatched','single']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localShrink(inp.assets, inp.market_returns);
        expect(r).not.toBe(null);
    }
});

test('demo single: single-asset report has w = 0 and shrunk = beta_ols', () => {
    const inp = makeDemoInput('single');
    const r = localShrink(inp.assets, inp.market_returns);
    expect(r.assets.length).toBe(1);
    expect(r.assets[0].shrinkage_weight).toBe(0);
    expect(Math.abs(r.assets[0].beta_shrunk - r.assets[0].beta_ols)).toBeLessThan(1e-9);
});

test('demo mismatched skips the bad asset', () => {
    const inp = makeDemoInput('mismatched');
    const r = localShrink(inp.assets, inp.market_returns);
    expect(r.assets.length).toBe(1);
    expect(r.assets[0].symbol).toBe('OK');
});

test('demo inverse: SH has negative beta_ols', () => {
    const inp = makeDemoInput('inverse');
    const r = localShrink(inp.assets, inp.market_returns);
    const sh = r.assets.find(s => s.symbol === 'SH');
    expect(sh.beta_ols).toBeLessThan(0);
});

// ── formatters & round-trip ──────────────────────────────────────

test('marketToBlob + assetsToBlob round-trip', () => {
    const market = [0.01, -0.02, 0.03];
    const assets = [{ symbol: 'A', asset_returns: [0.01, 0.02, 0.03] }];
    expect(parseMarketBlob(marketToBlob(market)).market_returns).toEqual(market);
    expect(parseAssetsBlob(assetsToBlob(assets)).assets).toEqual(assets);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtNum(1.23456)).toBe('1.2346');
    expect(fmtNumSigned(1.5)).toBe('+1.5000');
    expect(fmtNumSigned(-1.5)).toBe('-1.5000');
    expect(fmtPct(0.0125)).toBe('1.25%');
    expect(fmtInt(42.9)).toBe('42');
    expect(fmtNum(NaN)).toBe('—');
});

test('DEFAULT_INPUTS sanity', () => {
    expect(DEFAULT_INPUTS.assets).toEqual([]);
    expect(DEFAULT_INPUTS.market_returns).toEqual([]);
    expect(MIN_OBS).toBe(5);
});
