// ACF (autocorrelation function) helpers: parser, validator, body shape,
// localCompute Rust-mirror, badges, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_MAX_LAG, BARTLETT_Z, DEFAULT_INPUTS,
    parseSeriesBlob, seriesToBlob, validateInputs, buildBody, localCompute,
    autocorrelationBadge, ar1PhiEstimate, summarize,
    makeDemoInput,
    fmtAcf, fmtBand, fmtInt, fmtNum,
} from '../js/_acf_inputs.js';

// ── constants ─────────────────────────────────────────────────────

test('Bartlett z = 1.96 (95% CI)', () => {
    expect(BARTLETT_Z).toBe(1.96);
});

test('DEFAULT_MAX_LAG = 20 (Rust default)', () => {
    expect(DEFAULT_MAX_LAG).toBe(20);
});

// ── parser ────────────────────────────────────────────────────────

test('parseSeriesBlob: whitespace + commas + comments', () => {
    const r = parseSeriesBlob('0.01, -0.02\n# noise\n0.005  0.012');
    expect(r.errors).toEqual([]);
    expect(r.series).toEqual([0.01, -0.02, 0.005, 0.012]);
});

test('parseSeriesBlob: rejects non-finite', () => {
    expect(parseSeriesBlob('0.01, foo').errors[0].message).toMatch(/foo/);
});

test('parseSeriesBlob: non-string returns 1 error', () => {
    expect(parseSeriesBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts default', () => {
    expect(validateInputs({ series: new Array(50).fill(0).map((_, i) => i), max_lag: 10 })).toBe(null);
});

test('validate rejects: bad array / NaN / max_lag < 1 / short series / max_lag ≥ n', () => {
    expect(validateInputs({ series: 'no', max_lag: 5 })).toMatch(/series/);
    expect(validateInputs({ series: [1, NaN, 3, 4, 5], max_lag: 2 })).toMatch(/finite/);
    expect(validateInputs({ series: [1, 2, 3, 4, 5], max_lag: 0 })).toMatch(/max_lag/);
    expect(validateInputs({ series: [1, 2, 3], max_lag: 1 })).toMatch(/observations/);
    expect(validateInputs({ series: [1, 2, 3, 4, 5, 6], max_lag: 10 })).toMatch(/max_lag/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: forwards series + max_lag', () => {
    expect(buildBody({ series: [1, 2, 3], max_lag: 2 })).toEqual({ series: [1, 2, 3], max_lag: 2 });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: too short returns null', () => {
    expect(localCompute(new Array(3).fill(0.01), 1)).toBeNull();
});

test('local: max_lag=0 returns null', () => {
    const s = Array.from({ length: 50 }, (_, i) => i);
    expect(localCompute(s, 0)).toBeNull();
});

test('local: NaN input returns null', () => {
    const s = Array.from({ length: 50 }, (_, i) => i);
    s[10] = NaN;
    expect(localCompute(s, 5)).toBeNull();
});

test('local: flat series returns null (zero denominator)', () => {
    expect(localCompute(new Array(50).fill(100), 5)).toBeNull();
});

test('local: lag-0 ACF is always 1.0', () => {
    const s = Array.from({ length: 50 }, (_, i) => Math.sin(i * 0.1) * 5);
    const r = localCompute(s, 10);
    expect(r.autocorrelations[0]).toBeCloseTo(1.0, 12);
});

test('local: random walk → lag-1 ACF > 0.8', () => {
    let state = 42n;
    const s = new Array(200).fill(0);
    for (let i = 1; i < s.length; i++) {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        const step = (Number(state >> 32n) / 0xFFFFFFFF - 0.5) * 2;
        s[i] = s[i - 1] + step;
    }
    const r = localCompute(s, 10);
    expect(r.autocorrelations[1]).toBeGreaterThan(0.8);
});

test('local: white noise → most lags inside Bartlett bands', () => {
    let state = 11n;
    const s = new Array(500);
    for (let i = 0; i < s.length; i++) {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        s[i] = (Number(state >> 32n) / 0xFFFFFFFF - 0.5) * 2;
    }
    const r = localCompute(s, 10);
    let outside = 0;
    for (let k = 1; k <= 10; k++) if (Math.abs(r.autocorrelations[k]) > r.confidence_band) outside++;
    expect(outside).toBeLessThanOrEqual(3);
});

test('local: AR(1) φ=0.8 → lag-1 ACF ≈ 0.8', () => {
    let state = 7n;
    const phi = 0.8;
    const s = new Array(1000).fill(0);
    for (let i = 1; i < s.length; i++) {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        const eps = (Number(state >> 32n) / 0xFFFFFFFF - 0.5) * 0.5;
        s[i] = phi * s[i - 1] + eps;
    }
    const r = localCompute(s, 5);
    expect(Math.abs(r.autocorrelations[1] - phi)).toBeLessThan(0.05);
});

test('local: Bartlett band scales as 1/√n', () => {
    const s50  = Array.from({ length: 50  }, (_, i) => i);
    const s500 = Array.from({ length: 500 }, (_, i) => i);
    const r50  = localCompute(s50,  5);
    const r500 = localCompute(s500, 5);
    const ratio = r50.confidence_band / r500.confidence_band;
    expect(Math.abs(ratio - Math.sqrt(10))).toBeLessThan(0.01);
});

test('local: significant_lags only includes k > 0 lags outside band', () => {
    const s = Array.from({ length: 100 }, (_, i) => Math.sin(i * 0.3));
    const r = localCompute(s, 20);
    for (const k of r.significant_lags) {
        expect(k).toBeGreaterThan(0);
        expect(Math.abs(r.autocorrelations[k])).toBeGreaterThan(r.confidence_band);
    }
});

test('local: confidence_band exact = 1.96 / sqrt(n)', () => {
    const s = Array.from({ length: 100 }, (_, i) => Math.sin(i * 0.2));
    const r = localCompute(s, 5);
    expect(r.confidence_band).toBeCloseTo(1.96 / Math.sqrt(100), 12);
});

// ── badges + helpers ─────────────────────────────────────────────

test('autocorrelationBadge: white_noise inside band, else tier ladder', () => {
    expect(autocorrelationBadge(0.05, 0.1).key).toMatch(/white_noise/);
    expect(autocorrelationBadge(0.9, 0.1).key).toMatch(/random_walk/);
    expect(autocorrelationBadge(0.5, 0.1).key).toMatch(/persistent/);
    expect(autocorrelationBadge(0.15, 0.1).key).toMatch(/mild_persistence/);
    expect(autocorrelationBadge(-0.15, 0.1).key).toMatch(/mild_reversion/);
    expect(autocorrelationBadge(-0.5, 0.1).key).toMatch(/mean_reverting/);
    expect(autocorrelationBadge(null, 0.1).key).toMatch(/unknown/);
});

test('ar1PhiEstimate: returns lag-1 ACF', () => {
    expect(ar1PhiEstimate({ autocorrelations: [1, 0.75, 0.5] })).toBe(0.75);
    expect(Number.isNaN(ar1PhiEstimate(null))).toBe(true);
});

test('summarize: lag_count / sig_count / rho1 / rho5 / rho10', () => {
    const s = summarize({
        autocorrelations: [1, 0.8, 0.5, 0.2, 0.1, 0.05, 0.02, 0.01, 0, -0.01, -0.05],
        significant_lags: [1, 2],
    });
    expect(s.lag_count).toBe(10);
    expect(s.sig_count).toBe(2);
    expect(s.rho1).toBe(0.8);
    expect(s.rho5).toBe(0.05);
    expect(s.rho10).toBe(-0.05);
    expect(s.max_abs_lag).toBe(1);
});

test('summarize: empty / null → lag_count 0', () => {
    expect(summarize(null).lag_count).toBe(0);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + computes a non-null report', () => {
    for (const k of ['white-noise','random-walk','ar1-0.8','ar1-neg0.6',
                     'sinusoid','trending','wide-lags','short-series']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.series, inp.max_lag);
        expect(r).not.toBeNull();
        expect(r.autocorrelations.length).toBe(inp.max_lag + 1);
    }
});

test('demo random-walk: rho1 > 0.8', () => {
    const inp = makeDemoInput('random-walk');
    const r = localCompute(inp.series, inp.max_lag);
    expect(r.autocorrelations[1]).toBeGreaterThan(0.8);
});

test('demo ar1-0.8: rho1 within 0.05 of 0.8', () => {
    const inp = makeDemoInput('ar1-0.8');
    const r = localCompute(inp.series, inp.max_lag);
    expect(Math.abs(r.autocorrelations[1] - 0.8)).toBeLessThan(0.05);
});

test('demo ar1-neg0.6: rho1 within 0.1 of -0.6 (negative AR)', () => {
    const inp = makeDemoInput('ar1-neg0.6');
    const r = localCompute(inp.series, inp.max_lag);
    expect(Math.abs(r.autocorrelations[1] - (-0.6))).toBeLessThan(0.1);
});

test('demo trending: ρ̂(1) very high (≥ 0.95) from monotone drift', () => {
    const inp = makeDemoInput('trending');
    const r = localCompute(inp.series, inp.max_lag);
    expect(r.autocorrelations[1]).toBeGreaterThan(0.95);
});

test('demo white-noise: |ρ̂(1)| within ±0.3 (random)', () => {
    const inp = makeDemoInput('white-noise');
    const r = localCompute(inp.series, inp.max_lag);
    expect(Math.abs(r.autocorrelations[1])).toBeLessThan(0.3);
});

test('demo wide-lags: produces 50 lags', () => {
    const inp = makeDemoInput('wide-lags');
    expect(inp.max_lag).toBe(50);
    const r = localCompute(inp.series, inp.max_lag);
    expect(r.autocorrelations.length).toBe(51);
});

test('demo short-series: works with 20 bars', () => {
    const inp = makeDemoInput('short-series');
    expect(inp.series.length).toBe(20);
    const r = localCompute(inp.series, inp.max_lag);
    expect(r).not.toBeNull();
});

// ── round-trip + formatters ──────────────────────────────────────

test('seriesToBlob round-trips through parseSeriesBlob', () => {
    const s = [0.01, -0.02, 0.005, 0.012];
    const back = parseSeriesBlob(seriesToBlob(s));
    expect(back.errors).toEqual([]);
    expect(back.series).toEqual(s);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtAcf(0.5)).toBe('+0.5000');
    expect(fmtAcf(-0.5)).toBe('-0.5000');
    expect(fmtBand(0.087)).toBe('±0.0870');
    expect(fmtInt(7.9)).toBe('7');
    expect(fmtNum(1.2345)).toBe('1.2345');
    expect(fmtAcf(null)).toBe('—');
    expect(fmtAcf(NaN)).toBe('—');
});
