// Anderson-Darling normality helpers: parser, validator, localTest mirror,
// erf accuracy, badges, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, MIN_OBS,
    parseSampleBlob, sampleToBlob, validateInputs, buildBody, localTest,
    erf, standardNormalCdf, verdictBadge, approxPValue, summarizeSample,
    makeDemoInput,
    fmtNum, fmtNumSigned, fmtPVal, fmtInt,
} from '../js/_ad_normality_inputs.js';

// ── parser ────────────────────────────────────────────────────────

test('parseSampleBlob: comma + whitespace, comments ignored', () => {
    const r = parseSampleBlob('1 2\n# noise\n3, 4');
    expect(r.errors).toEqual([]);
    expect(r.sample).toEqual([1, 2, 3, 4]);
});

test('parseSampleBlob: $/% / () for negatives', () => {
    const r = parseSampleBlob('$0.01 -$0.02 (0.05)');
    expect(r.errors).toEqual([]);
    expect(r.sample).toEqual([0.01, -0.02, -0.05]);
});

test('parseSampleBlob: non-string returns 1 error', () => {
    expect(parseSampleBlob(undefined).errors.length).toBe(1);
});

test('parseSampleBlob: bad token tagged with line_no', () => {
    expect(parseSampleBlob('1 zzz 3').errors[0].line_no).toBe(2);
});

// ── validator ─────────────────────────────────────────────────────

test('validate: accepts valid 8-element sample', () => {
    expect(validateInputs({ sample: [1, 2, 3, 4, 5, 6, 7, 8] })).toBe(null);
});

test('validate: rejects non-array / short / non-finite', () => {
    expect(validateInputs({ sample: 'no' })).toMatch(/sample/);
    expect(validateInputs({ sample: [1, 2, 3] })).toMatch(/8 obs/);
    expect(validateInputs({ sample: [1, 2, 3, 4, NaN, 6, 7, 8] })).toMatch(/finite/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: passes through', () => {
    expect(buildBody({ sample: [1, 2, 3], extra: 'x' })).toEqual({ sample: [1, 2, 3] });
});

// ── erf + standardNormalCdf ──────────────────────────────────────

test('erf: known values', () => {
    expect(erf(0)).toBeCloseTo(0, 6);
    expect(erf(1)).toBeCloseTo(0.8427, 3);
    expect(erf(-1)).toBeCloseTo(-0.8427, 3);
    expect(erf(2)).toBeCloseTo(0.9953, 3);
});

test('standardNormalCdf: Φ(0)=0.5, Φ(1)≈0.8413', () => {
    expect(standardNormalCdf(0)).toBeCloseTo(0.5, 6);
    expect(standardNormalCdf(1)).toBeCloseTo(0.8413, 3);
    expect(standardNormalCdf(-2)).toBeCloseTo(0.0228, 3);
});

// ── localTest mirroring Rust #[test] cases ───────────────────────

test('local: too-short returns null', () => {
    expect(localTest([0, 0, 0, 0, 0])).toBe(null);
});

test('local: NaN returns null', () => {
    expect(localTest([0, NaN, 1, 2, 3, 4, 5, 6])).toBe(null);
});

test('local: flat (var=0) returns null', () => {
    expect(localTest(new Array(50).fill(1))).toBe(null);
});

function boxMuller(n, seed) {
    let state = BigInt(seed);
    const MASK = 0xFFFFFFFFFFFFFFFFn;
    const out = [];
    for (let i = 0; i < n; i++) {
        state = (state * 6364136223846793005n + 1442695040888963407n) & MASK;
        const u1 = Math.max(1e-12, Number(state >> 32n) / 0xFFFFFFFF);
        state = (state * 6364136223846793005n + 1442695040888963407n) & MASK;
        const u2 = Number(state >> 32n) / 0xFFFFFFFF;
        out.push(Math.sqrt(-2 * Math.log(u1)) * Math.cos(2 * Math.PI * u2));
    }
    return out;
}

test('local: Gaussian (n=2000) does not reject at 1%', () => {
    const s = boxMuller(2000, 42);
    const r = localTest(s);
    expect(r).not.toBe(null);
    expect(r.reject_at_1pct).toBe(false);
});

test('local: |Gaussian| (right-skew, n=500) rejects at 5%', () => {
    const s = boxMuller(500, 7).map(Math.abs);
    const r = localTest(s);
    expect(r.reject_at_5pct).toBe(true);
});

test('local: heavy-tail mixture rejects at 1%', () => {
    let state = 11n;
    const MASK = 0xFFFFFFFFFFFFFFFFn;
    const s = [];
    for (let i = 0; i < 2000; i++) {
        state = (state * 6364136223846793005n + 1442695040888963407n) & MASK;
        const u = Number(state >> 32n) / 0xFFFFFFFF;
        state = (state * 6364136223846793005n + 1442695040888963407n) & MASK;
        const u1 = Math.max(1e-12, Number(state >> 32n) / 0xFFFFFFFF);
        state = (state * 6364136223846793005n + 1442695040888963407n) & MASK;
        const u2 = Number(state >> 32n) / 0xFFFFFFFF;
        const z = Math.sqrt(-2 * Math.log(u1)) * Math.cos(2 * Math.PI * u2);
        s.push(u < 0.1 ? z * 5 : z);
    }
    const r = localTest(s);
    expect(r.reject_at_1pct).toBe(true);
});

test('local: adjustment factor inflates statistic for small n', () => {
    const s = boxMuller(20, 99);
    const r = localTest(s);
    expect(r.a_squared_adjusted).toBeGreaterThanOrEqual(r.a_squared);
});

test('local: reports n_observations', () => {
    const s = boxMuller(100, 3);
    const r = localTest(s);
    expect(r.n_observations).toBe(100);
});

test('local: deterministic — same input twice → same A²*', () => {
    const s = boxMuller(200, 5);
    const r1 = localTest(s);
    const r2 = localTest(s);
    expect(r1.a_squared_adjusted).toBe(r2.a_squared_adjusted);
});

// ── badges ────────────────────────────────────────────────────────

test('verdictBadge: normal / borderline / reject5 / reject1 / unknown', () => {
    const mk = (a) => ({
        a_squared: a, a_squared_adjusted: a,
        reject_at_5pct: a > 0.752, reject_at_1pct: a > 1.035, n_observations: 100,
    });
    expect(verdictBadge(mk(0.3)).key).toMatch(/normal/);
    expect(verdictBadge(mk(0.7)).key).toMatch(/borderline/);
    expect(verdictBadge(mk(0.9)).key).toMatch(/reject_5pct/);
    expect(verdictBadge(mk(1.2)).key).toMatch(/reject_strong/);
    expect(verdictBadge(null).key).toMatch(/unknown/);
});

test('approxPValue: monotone-decreasing in A²*', () => {
    expect(approxPValue(0.1)).toBeGreaterThan(approxPValue(0.5));
    expect(approxPValue(0.5)).toBeGreaterThan(approxPValue(1.0));
    expect(approxPValue(1.0)).toBeGreaterThan(approxPValue(2.0));
    expect(approxPValue(2.0)).toBeGreaterThan(approxPValue(5.0));
});

test('approxPValue: NaN for non-finite or non-positive', () => {
    expect(Number.isNaN(approxPValue(NaN))).toBe(true);
    expect(Number.isNaN(approxPValue(0))).toBe(true);
    expect(Number.isNaN(approxPValue(-1))).toBe(true);
});

test('approxPValue: returns 0 for very large A²*', () => {
    expect(approxPValue(20)).toBe(0);
});

// ── summarizeSample ──────────────────────────────────────────────

test('summarizeSample: count / mean / sd / min / max', () => {
    const s = summarizeSample([1, 2, 3, 4, 5]);
    expect(s.count).toBe(5);
    expect(s.mean).toBe(3);
    expect(s.sd).toBeCloseTo(Math.sqrt(2.5), 6);
    expect(s.min).toBe(1);
    expect(s.max).toBe(5);
});

test('summarizeSample: skew & kurt are finite for non-flat input', () => {
    const s = summarizeSample([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    expect(Number.isFinite(s.skew)).toBe(true);
    expect(Number.isFinite(s.kurt)).toBe(true);
});

test('summarizeSample: empty → NaN extrema', () => {
    const s = summarizeSample([]);
    expect(s.count).toBe(0);
    expect(Number.isNaN(s.mean)).toBe(true);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + tests cleanly', () => {
    for (const k of ['gaussian','heavy-tail','right-skew','left-skew','uniform',
                     'bimodal','exponential','small-sample']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localTest(inp.sample);
        expect(r).not.toBe(null);
        expect(r.n_observations).toBe(inp.sample.length);
    }
});

test('demo gaussian does not reject at 1%', () => {
    const inp = makeDemoInput('gaussian');
    const r = localTest(inp.sample);
    expect(r.reject_at_1pct).toBe(false);
});

test('demo heavy-tail rejects at 1%', () => {
    const inp = makeDemoInput('heavy-tail');
    const r = localTest(inp.sample);
    expect(r.reject_at_1pct).toBe(true);
});

test('demo right-skew rejects at 5%', () => {
    const inp = makeDemoInput('right-skew');
    const r = localTest(inp.sample);
    expect(r.reject_at_5pct).toBe(true);
});

test('demo exponential rejects at 1%', () => {
    const inp = makeDemoInput('exponential');
    const r = localTest(inp.sample);
    expect(r.reject_at_1pct).toBe(true);
});

// ── round-trip + formatters ──────────────────────────────────────

test('sampleToBlob round-trips through parseSampleBlob', () => {
    const sample = [1.5, -2.5, 3.5];
    const back = parseSampleBlob(sampleToBlob(sample));
    expect(back.errors).toEqual([]);
    expect(back.sample).toEqual(sample);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtNum(1.23456)).toBe('1.2346');
    expect(fmtNum(NaN)).toBe('—');
    expect(fmtNumSigned(0.5)).toBe('+0.5000');
    expect(fmtNumSigned(-0.5)).toBe('-0.5000');
    expect(fmtPVal(0.000001)).toBe('< 0.0001');
    expect(fmtPVal(0.04)).toBe('0.0400');
    expect(fmtPVal(NaN)).toBe('—');
    expect(fmtInt(42.9)).toBe('42');
    expect(fmtInt(NaN)).toBe('—');
});

test('DEFAULT_INPUTS / MIN_OBS sanity', () => {
    expect(DEFAULT_INPUTS.sample).toEqual([]);
    expect(MIN_OBS).toBe(8);
});
