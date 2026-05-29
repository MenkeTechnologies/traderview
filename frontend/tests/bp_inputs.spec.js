// Breusch-Pagan helpers: parser, validator, localTest parity, badges.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, MIN_OBS,
    parsePairsBlob, pairsToBlob,
    validateInputs, buildBody, localTest,
    chiSquaredUpperTail, standardNormalCdf, erf,
    verdictBadge, r2Badge, sampleBadge, summarizeData,
    makeDemoInput,
    fmtNum, fmtPVal, fmtPct, fmtInt,
} from '../js/_bp_inputs.js';

// ── parser ────────────────────────────────────────────────────────

test('parsePairsBlob: 2 tokens per line', () => {
    const r = parsePairsBlob('1 2\n# noise\n3, 4');
    expect(r.errors).toEqual([]);
    expect(r.x).toEqual([1, 3]);
    expect(r.y).toEqual([2, 4]);
});

test('parsePairsBlob: rejects wrong count', () => {
    expect(parsePairsBlob('1').errors[0].message).toMatch(/2 tokens/);
});

test('parsePairsBlob: rejects non-finite', () => {
    expect(parsePairsBlob('1 NaN').errors[0].message).toMatch(/finite/);
});

test('parsePairsBlob: non-string returns 1 error', () => {
    expect(parsePairsBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts valid', () => {
    const arr = Array.from({ length: 10 }, (_, i) => i + 1);
    expect(validateInputs({ x: arr, y: arr })).toBe(null);
});

test('validate rejects: bad arrays / unequal / short / non-finite', () => {
    const ok = Array.from({ length: 10 }, (_, i) => i + 1);
    expect(validateInputs({ x: 'no', y: ok })).toMatch(/x/);
    expect(validateInputs({ x: ok, y: 'no' })).toMatch(/y/);
    expect(validateInputs({ x: ok, y: ok.slice(0, 5) })).toMatch(/equal length/);
    expect(validateInputs({ x: ok.slice(0, 5), y: ok.slice(0, 5) })).toMatch(/10 pairs/);
    const bad = [...ok]; bad[2] = NaN;
    expect(validateInputs({ x: bad, y: ok })).toMatch(/finite/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody copies arrays', () => {
    const inp = { x: [1, 2], y: [3, 4] };
    const body = buildBody(inp);
    expect(body).toEqual({ x: [1, 2], y: [3, 4] });
    expect(body.x).not.toBe(inp.x);
});

// ── chi² helpers ──────────────────────────────────────────────────

test('chiSquaredUpperTail: monotone decreasing in x', () => {
    expect(chiSquaredUpperTail(1, 1)).toBeGreaterThan(chiSquaredUpperTail(5, 1));
});

test('chiSquaredUpperTail: x ≤ 0 → 1', () => {
    expect(chiSquaredUpperTail(0, 1)).toBe(1);
    expect(chiSquaredUpperTail(-1, 1)).toBe(1);
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
    const x = new Array(5).fill(1);
    expect(localTest(x, x)).toBe(null);
});

test('local: mismatched returns null', () => {
    expect(localTest(new Array(20).fill(1), new Array(10).fill(1))).toBe(null);
});

test('local: NaN returns null', () => {
    const x = new Array(30).fill(1);
    const y = new Array(30).fill(1);
    y[5] = NaN;
    expect(localTest(x, y)).toBe(null);
});

test('local: flat predictor returns null', () => {
    const x = new Array(30).fill(1);
    const y = Array.from({ length: 30 }, (_, i) => i);
    expect(localTest(x, y)).toBe(null);
});

function lcg(seed) {
    let s = BigInt(seed);
    const MASK = 0xFFFFFFFFFFFFFFFFn;
    return () => {
        s = (s * 6364136223846793005n + 1442695040888963407n) & MASK;
        return Number(s >> 32n) / 0xFFFFFFFF;
    };
}

test('local: homoskedastic data does NOT reject at 5%', () => {
    const rand = lcg(42);
    const x = Array.from({ length: 300 }, (_, i) => i);
    const y = x.map(xi => 2 * xi + (rand() - 0.5) * 1.0);
    const r = localTest(x, y);
    expect(r).not.toBe(null);
    expect(r.reject_at_5pct).toBe(false);
});

test('local: variance ∝ x → rejects at 5%', () => {
    const rand = lcg(11);
    const x = Array.from({ length: 300 }, (_, i) => i + 1);
    const y = x.map(xi => 2 * xi + (rand() - 0.5) * (xi / 30));
    const r = localTest(x, y);
    expect(r.reject_at_5pct).toBe(true);
});

test('local: p-value in [0, 1]', () => {
    const rand = lcg(99);
    const x = Array.from({ length: 100 }, (_, i) => i);
    const y = x.map(xi => 2 * xi + (rand() - 0.5) * 0.5);
    const r = localTest(x, y);
    expect(r.p_value).toBeGreaterThanOrEqual(0);
    expect(r.p_value).toBeLessThanOrEqual(1);
});

test('local: r_squared_auxiliary ≥ 0 (clamped at 0 if negative)', () => {
    const x = Array.from({ length: 30 }, (_, i) => i);
    const y = x.map(xi => 2 * xi + 0.5);   // perfect fit → resid_sq all 0 → tss ≈ 0
    const r = localTest(x, y);
    if (r) expect(r.r_squared_auxiliary).toBeGreaterThanOrEqual(0);
});

test('local: deterministic', () => {
    const x = Array.from({ length: 20 }, (_, i) => i);
    const y = x.map(xi => 2 * xi + (xi % 3));
    const a = localTest(x, y);
    const b = localTest(x, y);
    expect(a.lm_statistic).toBe(b.lm_statistic);
    expect(a.p_value).toBe(b.p_value);
});

test('local: n_observations reported back', () => {
    const x = Array.from({ length: 50 }, (_, i) => i);
    const y = x.map(xi => 2 * xi + (xi % 3));
    expect(localTest(x, y).n_observations).toBe(50);
});

// ── badges ────────────────────────────────────────────────────────

test('verdictBadge: 4 tiers', () => {
    const mk = (p) => ({ p_value: p, lm_statistic: 5, r_squared_auxiliary: 0.1,
                          n_observations: 50, reject_at_5pct: p < 0.05, reject_at_1pct: p < 0.01 });
    expect(verdictBadge(mk(0.005)).key).toMatch(/strong_reject/);
    expect(verdictBadge(mk(0.03)).key).toMatch(/reject/);
    expect(verdictBadge(mk(0.07)).key).toMatch(/borderline/);
    expect(verdictBadge(mk(0.5)).key).toMatch(/homoskedastic/);
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
    expect(sampleBadge(500).key).toMatch(/large/);
    expect(sampleBadge(100).key).toMatch(/medium/);
    expect(sampleBadge(15).key).toMatch(/small/);
    expect(sampleBadge(5).key).toMatch(/too_small/);
    expect(sampleBadge(NaN).key).toMatch(/unknown/);
});

// ── summarize ─────────────────────────────────────────────────────

test('summarizeData: counts / means / sds', () => {
    const s = summarizeData([1, 2, 3, 4, 5], [2, 4, 6, 8, 10]);
    expect(s.n).toBe(5);
    expect(s.x_mean).toBe(3);
    expect(s.y_mean).toBe(6);
    expect(s.x_sd).toBeCloseTo(Math.sqrt(2.5), 9);
    expect(s.y_sd).toBeCloseTo(Math.sqrt(10), 9);
});

test('summarizeData: empty → 0 + NaN', () => {
    const s = summarizeData([], []);
    expect(s.n).toBe(0);
    expect(Number.isNaN(s.x_mean)).toBe(true);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + tests cleanly', () => {
    for (const k of ['homoskedastic','variance-increasing-in-x','variance-decreasing-in-x',
                     'v-shape-variance','narrow-then-wide','small-sample',
                     'returns-vs-vol','extreme-spike-residuals']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localTest(inp.x, inp.y);
        expect(r).not.toBe(null);
        expect(r.n_observations).toBe(inp.x.length);
    }
});

test('demo homoskedastic does not reject', () => {
    const inp = makeDemoInput('homoskedastic');
    const r = localTest(inp.x, inp.y);
    expect(r.reject_at_5pct).toBe(false);
});

test('demo variance-increasing rejects at 5%', () => {
    const inp = makeDemoInput('variance-increasing-in-x');
    const r = localTest(inp.x, inp.y);
    expect(r.reject_at_5pct).toBe(true);
});

test('demo small-sample uses n=12', () => {
    const inp = makeDemoInput('small-sample');
    expect(inp.x.length).toBe(12);
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
    expect(fmtPVal(0.04)).toBe('0.0400');
    expect(fmtPct(0.0125)).toBe('1.25%');
    expect(fmtInt(42.9)).toBe('42');
    expect(fmtNum(NaN)).toBe('—');
});

test('DEFAULTS sanity', () => {
    expect(DEFAULT_INPUTS.x).toEqual([]);
    expect(DEFAULT_INPUTS.y).toEqual([]);
    expect(MIN_OBS).toBe(10);
});
