// Breusch-Godfrey helpers: parser, validator, localTest parity (OLS + linear solve), badges.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, DEFAULT_LAG, MIN_LAG, MAX_LAG,
    parsePairsBlob, pairsToBlob, validateInputs, buildBody, localTest,
    solveLinear, chiSquaredUpperTail, chiSquared5pctCritical, standardNormalCdf, erf,
    verdictBadge, r2Badge, sampleBadge, summarizeData,
    makeDemoInput,
    fmtNum, fmtPVal, fmtPct, fmtInt,
} from '../js/_bg_inputs.js';

// ── parser ────────────────────────────────────────────────────────

test('parsePairsBlob: 2 tokens per line', () => {
    const r = parsePairsBlob('1 2\n# noise\n3, 4');
    expect(r.errors).toEqual([]);
    expect(r.x).toEqual([1, 3]);
    expect(r.y).toEqual([2, 4]);
});

test('parsePairsBlob: rejects wrong count / non-finite / non-string', () => {
    expect(parsePairsBlob('1').errors[0].message).toMatch(/2 tokens/);
    expect(parsePairsBlob('1 NaN').errors[0].message).toMatch(/finite/);
    expect(parsePairsBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts valid', () => {
    const arr = Array.from({ length: 15 }, (_, i) => i + 1);
    expect(validateInputs({ x: arr, y: arr, lag_order: 4 })).toBe(null);
});

test('validate rejects: bad arrays / bad lag / unequal / short / non-finite', () => {
    const ok = Array.from({ length: 15 }, (_, i) => i + 1);
    expect(validateInputs({ x: 'no', y: ok, lag_order: 4 })).toMatch(/x/);
    expect(validateInputs({ x: ok, y: 'no', lag_order: 4 })).toMatch(/y/);
    expect(validateInputs({ x: ok, y: ok, lag_order: 0 })).toMatch(/lag_order/);
    expect(validateInputs({ x: ok, y: ok, lag_order: 999 })).toMatch(/lag_order/);
    expect(validateInputs({ x: ok, y: ok.slice(0, 5), lag_order: 4 })).toMatch(/equal length/);
    expect(validateInputs({ x: ok.slice(0, 5), y: ok.slice(0, 5), lag_order: 4 })).toMatch(/12 pairs/);
    const bad = [...ok]; bad[2] = NaN;
    expect(validateInputs({ x: bad, y: ok, lag_order: 4 })).toMatch(/finite/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody copies + passes through', () => {
    expect(buildBody({ x: [1, 2], y: [3, 4], lag_order: 4 }))
        .toEqual({ x: [1, 2], y: [3, 4], lag_order: 4 });
});

// ── linear solver sanity ─────────────────────────────────────────

test('solveLinear: recovers known solution', () => {
    // 2x = 6, x = 3
    const r = solveLinear([[1, 1], [1, -1]], [5, 1]);
    expect(r[0]).toBeCloseTo(3, 9);
    expect(r[1]).toBeCloseTo(2, 9);
});

test('solveLinear: returns null on singular matrix', () => {
    expect(solveLinear([[1, 2], [2, 4]], [3, 6])).toBe(null);
});

// ── chi² helpers ──────────────────────────────────────────────────

test('chiSquaredUpperTail: monotone decreasing', () => {
    expect(chiSquaredUpperTail(1, 4)).toBeGreaterThan(chiSquaredUpperTail(5, 4));
});

test('chiSquared5pctCritical: 1..10 hardcoded', () => {
    expect(chiSquared5pctCritical(1)).toBe(3.841);
    expect(chiSquared5pctCritical(4)).toBe(9.488);
    expect(chiSquared5pctCritical(10)).toBe(18.307);
});

test('chiSquared5pctCritical: large-k approx', () => {
    expect(chiSquared5pctCritical(20)).toBeGreaterThan(20);
});

test('erf sanity', () => {
    expect(erf(0)).toBeCloseTo(0, 6);
    expect(erf(1)).toBeCloseTo(0.8427, 3);
});

test('standardNormalCdf(0) = 0.5', () => {
    expect(standardNormalCdf(0)).toBeCloseTo(0.5, 6);
});

// ── localTest parity (mirrors every Rust #[test]) ────────────────

test('local: too-short returns null', () => {
    expect(localTest(new Array(5).fill(1), new Array(5).fill(1), 2)).toBe(null);
});

test('local: zero lag returns null', () => {
    const x = Array.from({ length: 30 }, (_, i) => i);
    const y = x.map(xi => 2 * xi);
    expect(localTest(x, y, 0)).toBe(null);
});

test('local: mismatched returns null', () => {
    const x = Array.from({ length: 30 }, (_, i) => i);
    expect(localTest(x, new Array(10).fill(1), 2)).toBe(null);
});

test('local: NaN returns null', () => {
    const x = Array.from({ length: 30 }, (_, i) => i);
    const y = x.map(xi => 2 * xi);
    y[10] = NaN;
    expect(localTest(x, y, 2)).toBe(null);
});

function lcg(seed) {
    let s = BigInt(seed);
    const MASK = 0xFFFFFFFFFFFFFFFFn;
    return () => {
        s = (s * 6364136223846793005n + 1442695040888963407n) & MASK;
        return Number(s >> 32n) / 0xFFFFFFFF;
    };
}

test('local: iid residuals do not reject', () => {
    const rand = lcg(42);
    const x = Array.from({ length: 200 }, (_, i) => i);
    const y = x.map(xi => 2 * xi + (rand() - 0.5) * 0.5);
    const r = localTest(x, y, 4);
    expect(r.reject_at_5pct).toBe(false);
});

test('local: AR(1) residuals reject', () => {
    const rand = lcg(11);
    const x = Array.from({ length: 300 }, (_, i) => i);
    const e = new Array(300).fill(0);
    for (let i = 1; i < 300; i++) e[i] = 0.8 * e[i - 1] + (rand() - 0.5) * 5;
    const y = x.map((xi, i) => 2 * xi + e[i]);
    const r = localTest(x, y, 2);
    expect(r.reject_at_5pct).toBe(true);
});

test('local: p-value in [0, 1]', () => {
    const rand = lcg(99);
    const x = Array.from({ length: 100 }, (_, i) => i);
    const y = x.map(xi => 2 * xi + (rand() - 0.5) * 0.5);
    const r = localTest(x, y, 3);
    expect(r.p_value).toBeGreaterThanOrEqual(0);
    expect(r.p_value).toBeLessThanOrEqual(1);
});

test('local: deterministic', () => {
    const x = Array.from({ length: 30 }, (_, i) => i);
    const y = x.map(xi => 2 * xi + (xi % 3));
    expect(localTest(x, y, 4)).toEqual(localTest(x, y, 4));
});

test('local: n_observations + lag_order reported', () => {
    const x = Array.from({ length: 50 }, (_, i) => i);
    const y = x.map(xi => 2 * xi + (xi % 3));
    const r = localTest(x, y, 5);
    expect(r.n_observations).toBe(50);
    expect(r.lag_order).toBe(5);
});

// ── badges ────────────────────────────────────────────────────────

test('verdictBadge: 4 tiers', () => {
    const mk = (p) => ({ p_value: p, lm_statistic: 5, r_squared_auxiliary: 0.1,
                          lag_order: 4, n_observations: 100, reject_at_5pct: p < 0.05 });
    expect(verdictBadge(mk(0.005)).key).toMatch(/strong_reject/);
    expect(verdictBadge(mk(0.03)).key).toMatch(/reject/);
    expect(verdictBadge(mk(0.07)).key).toMatch(/borderline/);
    expect(verdictBadge(mk(0.5)).key).toMatch(/no_correlation/);
    expect(verdictBadge(null).key).toMatch(/unknown/);
});

test('r2Badge: 5 tiers', () => {
    expect(r2Badge(0.30).key).toMatch(/very_strong/);
    expect(r2Badge(0.15).key).toMatch(/strong/);
    expect(r2Badge(0.07).key).toMatch(/moderate/);
    expect(r2Badge(0.02).key).toMatch(/weak/);
    expect(r2Badge(0.005).key).toMatch(/negligible/);
    expect(r2Badge(NaN).key).toMatch(/unknown/);
});

test('sampleBadge: large / medium / small / too_small / unknown', () => {
    const mk = (n, p) => ({ n_observations: n, lag_order: p, lm_statistic: 0, p_value: 0.5,
                             r_squared_auxiliary: 0.01, reject_at_5pct: false });
    expect(sampleBadge(mk(500, 4)).key).toMatch(/large/);
    expect(sampleBadge(mk(100, 4)).key).toMatch(/medium/);
    expect(sampleBadge(mk(50, 4)).key).toMatch(/small/);
    expect(sampleBadge(mk(10, 4)).key).toMatch(/too_small/);
    expect(sampleBadge(mk(100, 0)).key).toMatch(/unknown/);
    expect(sampleBadge(null).key).toMatch(/unknown/);
});

// ── summarize ─────────────────────────────────────────────────────

test('summarizeData: counts / means / sds', () => {
    const s = summarizeData([1, 2, 3, 4, 5], [2, 4, 6, 8, 10]);
    expect(s.n).toBe(5);
    expect(s.x_mean).toBe(3);
    expect(s.y_mean).toBe(6);
    expect(s.x_sd).toBeCloseTo(Math.sqrt(2.5), 9);
});

test('summarizeData: empty → NaN', () => {
    const s = summarizeData([], []);
    expect(s.n).toBe(0);
    expect(Number.isNaN(s.x_mean)).toBe(true);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + tests cleanly', () => {
    for (const k of ['iid-residuals','ar1-residuals','ar2-residuals','mild-ar1',
                     'cyclical-residuals','high-lag','short-series','price-vs-return']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localTest(inp.x, inp.y, inp.lag_order);
        expect(r).not.toBe(null);
        expect(r.lag_order).toBe(inp.lag_order);
    }
});

test('demo iid does not reject', () => {
    const inp = makeDemoInput('iid-residuals');
    const r = localTest(inp.x, inp.y, inp.lag_order);
    expect(r.reject_at_5pct).toBe(false);
});

test('demo ar1 rejects', () => {
    const inp = makeDemoInput('ar1-residuals');
    const r = localTest(inp.x, inp.y, inp.lag_order);
    expect(r.reject_at_5pct).toBe(true);
});

test('demo high-lag uses lag_order=10', () => {
    const inp = makeDemoInput('high-lag');
    expect(inp.lag_order).toBe(10);
});

// ── formatters ────────────────────────────────────────────────────

test('pairsToBlob round-trips', () => {
    const x = [1, 2, 3];
    const y = [4, 5, 6];
    const back = parsePairsBlob(pairsToBlob(x, y));
    expect(back.errors).toEqual([]);
    expect(back.x).toEqual(x);
    expect(back.y).toEqual(y);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtNum(1.23456)).toBe('1.2346');
    expect(fmtPVal(0.000001)).toBe('< 0.0001');
    expect(fmtPct(0.0125)).toBe('1.25%');
    expect(fmtInt(42.9)).toBe('42');
    expect(fmtNum(NaN)).toBe('—');
});

test('DEFAULTS sanity', () => {
    expect(DEFAULT_INPUTS.x).toEqual([]);
    expect(DEFAULT_INPUTS.y).toEqual([]);
    expect(DEFAULT_INPUTS.lag_order).toBe(DEFAULT_LAG);
    expect(DEFAULT_LAG).toBe(4);
    expect(MIN_LAG).toBe(1);
    expect(MAX_LAG).toBe(50);
});
