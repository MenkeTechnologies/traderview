// VaR estimator helpers: parser (with % suffix), validator, body shape,
// historical + parametric-Gaussian Rust-mirrors, inverse-normal,
// distribution stats, loss histogram, comparison, demos, formatters.

import { test, expect } from 'vitest';
import {
    parseReturnsBlob, validateInputs, buildBody,
    localHistorical, localParametricGaussian, inverseNormal, phi,
    distributionStats, lossHistogram, compareMethods, makeDemoReturns,
    fmtUSD, fmtUSDSigned, fmtPct, fmtN, methodColor,
} from '../js/_var_estimator_inputs.js';

// ── parseReturnsBlob ──────────────────────────────────────────────

test('parser accepts CSV / whitespace / newline mix', () => {
    const r = parseReturnsBlob('0.005,-0.01\n0.002 0.003\n-0.015');
    expect(r.errors).toEqual([]);
    expect(r.returns).toEqual([0.005, -0.01, 0.002, 0.003, -0.015]);
});

test('parser %-suffix auto-divides by 100', () => {
    const r = parseReturnsBlob('1% -0.5% 2%');
    expect(r.errors).toEqual([]);
    expect(r.returns[0]).toBeCloseTo(0.01,  10);
    expect(r.returns[1]).toBeCloseTo(-0.005, 10);
    expect(r.returns[2]).toBeCloseTo(0.02,  10);
});

test('parser strips #-comments', () => {
    expect(parseReturnsBlob('0.01 # win\n# pure note\n-0.02').returns).toEqual([0.01, -0.02]);
});

test('parser flags non-finite tokens with index', () => {
    const r = parseReturnsBlob('0.01 abc 0.02');
    expect(r.errors.length).toBe(1);
    expect(r.returns).toEqual([0.01, 0.02]);
});

test('parser non-string returns 1 error', () => {
    expect(parseReturnsBlob(null).errors.length).toBe(1);
});

// ── validateInputs ────────────────────────────────────────────────

test('validate accepts 10+ returns + positive position + valid confidence', () => {
    expect(validateInputs(new Array(10).fill(0), 1000, 0.95)).toBe(null);
});

test('validate rejects < 10 returns', () => {
    expect(validateInputs([0, 0, 0], 1000, 0.95)).toMatch(/≥ 10/);
});

test('validate rejects bad confidence (must be exclusive open interval)', () => {
    expect(validateInputs(new Array(10).fill(0), 1000, 0)).toMatch(/confidence/);
    expect(validateInputs(new Array(10).fill(0), 1000, 1)).toMatch(/confidence/);
    expect(validateInputs(new Array(10).fill(0), 1000, 1.5)).toMatch(/confidence/);
});

test('validate rejects non-positive position', () => {
    expect(validateInputs(new Array(10).fill(0), 0, 0.95)).toMatch(/position_value/);
});

test('validate rejects non-finite returns', () => {
    expect(validateInputs([NaN, ...new Array(10).fill(0)], 1000, 0.95)).toMatch(/finite/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody emits backend VarBody shape', () => {
    const r = [0.01, -0.02];
    expect(buildBody(r, 5000, 0.99)).toEqual({
        daily_returns: r, position_value: 5000, confidence: 0.99,
    });
});

// ── localHistorical parity ────────────────────────────────────────

test('historical: < 10 obs returns zeroed default', () => {
    const r = localHistorical([0.01, -0.02], 10_000, 0.95);
    expect(r.var_dollars).toBe(0);
    expect(r.method).toBe('historical');
});

test('historical: invalid confidence returns zeroed default', () => {
    const r = localHistorical(new Array(30).fill(-0.01), 10_000, 1.5);
    expect(r.var_dollars).toBe(0);
});

test('historical: 95% VaR ≈ 5th-percentile loss × position', () => {
    // 95 zeros + 5 at -1% → VaR_95 ≈ $100 on $10k.
    const returns = [...new Array(95).fill(0), ...new Array(5).fill(-0.01)];
    const r = localHistorical(returns, 10_000, 0.95);
    expect(r.var_dollars).toBeCloseTo(100, 1);
});

test('historical: ES ≥ VaR (always — mean of worst tail)', () => {
    const returns = Array.from({ length: 100 }, (_, i) => (i - 50) * 0.001);
    const r = localHistorical(returns, 10_000, 0.95);
    expect(r.expected_shortfall_dollars).toBeGreaterThanOrEqual(r.var_dollars);
});

test('historical: VaR scales linearly with position value', () => {
    const returns = [...new Array(95).fill(0), ...new Array(5).fill(-0.01)];
    const small = localHistorical(returns, 1_000, 0.95);
    const big   = localHistorical(returns, 10_000, 0.95);
    expect(big.var_dollars / small.var_dollars).toBeCloseTo(10, 9);
});

test('historical: 99% more severe than 95% (or equal at tiny n)', () => {
    const returns = Array.from({ length: 1000 }, (_, i) =>
        -Math.min((i ** 2) / 1_000_000, 0.10));
    const r95 = localHistorical(returns, 10_000, 0.95);
    const r99 = localHistorical(returns, 10_000, 0.99);
    expect(r99.var_dollars).toBeGreaterThanOrEqual(r95.var_dollars);
});

// ── localParametricGaussian parity ────────────────────────────────

test('gaussian: zero volatility → zero VaR', () => {
    const r = localParametricGaussian(new Array(30).fill(0.01), 10_000, 0.95);
    expect(r.var_dollars).toBe(0);
});

test('gaussian: positive VaR when stdev > 0', () => {
    const returns = Array.from({ length: 50 }, (_, i) => (i - 25) / 25 * 0.01);
    const r = localParametricGaussian(returns, 10_000, 0.95);
    expect(r.var_dollars).toBeGreaterThan(0);
});

test('gaussian: 99% more severe than 95%', () => {
    const returns = Array.from({ length: 50 }, (_, i) => (i - 25) / 25 * 0.01);
    const r95 = localParametricGaussian(returns, 10_000, 0.95);
    const r99 = localParametricGaussian(returns, 10_000, 0.99);
    expect(r99.var_dollars).toBeGreaterThan(r95.var_dollars);
});

test('gaussian: < 2 obs returns zeroed default', () => {
    expect(localParametricGaussian([0.01], 10_000, 0.95).var_dollars).toBe(0);
});

test('gaussian: method label = parametric_gaussian', () => {
    expect(localParametricGaussian(new Array(30).fill(0.01), 10_000, 0.95).method)
        .toBe('parametric_gaussian');
});

// ── inverseNormal ─────────────────────────────────────────────────

test('inverseNormal: known z-scores for 90/95/99/99.9', () => {
    expect(inverseNormal(0.90)).toBeCloseTo(1.282, 3);
    expect(inverseNormal(0.95)).toBeCloseTo(1.645, 3);
    expect(inverseNormal(0.99)).toBeCloseTo(2.326, 3);
    expect(inverseNormal(0.999)).toBeCloseTo(3.090, 3);
});

test('inverseNormal: BSM fallback for arbitrary confidence', () => {
    // 0.975 ≈ 1.96 (standard 2-tail 95%).
    expect(inverseNormal(0.975)).toBeCloseTo(1.96, 1);
});

test('inverseNormal: symmetric below 0.5 (negative z)', () => {
    expect(inverseNormal(0.05)).toBeCloseTo(-1.645, 3);
});

test('phi: standard normal PDF integrates to ≈ 1 over wide range', () => {
    // φ(0) = 1/√(2π) ≈ 0.3989.
    expect(phi(0)).toBeCloseTo(1 / Math.sqrt(2 * Math.PI), 9);
    expect(phi(1.645)).toBeCloseTo(0.1031, 3);
});

// ── distributionStats ─────────────────────────────────────────────

test('distributionStats: symmetric returns → ~zero skew, ~zero kurtosis', () => {
    const returns = Array.from({ length: 100 }, (_, i) => (i - 50) * 0.001);
    const s = distributionStats(returns);
    expect(s.n).toBe(100);
    expect(Math.abs(s.skewness)).toBeLessThan(0.1);
});

test('distributionStats: fattest_left_tail counts σ-multiples below mean', () => {
    const returns = [...new Array(20).fill(0.001), -0.05];
    const s = distributionStats(returns);
    expect(s.fattest_left_tail).toBeLessThan(-3);
});

test('distributionStats: empty input safe', () => {
    expect(distributionStats([]).n).toBe(0);
    expect(Number.isNaN(distributionStats([]).mean)).toBe(true);
});

// ── lossHistogram ─────────────────────────────────────────────────

test('lossHistogram: counts sum to returns.length', () => {
    const returns = Array.from({ length: 100 }, (_, i) => (i - 50) * 0.001);
    const h = lossHistogram(returns, 10_000, 20);
    expect(h.counts.reduce((a, b) => a + b, 0)).toBe(100);
});

test('lossHistogram: edges length = nBuckets + 1', () => {
    const returns = Array.from({ length: 100 }, (_, i) => (i - 50) * 0.001);
    const h = lossHistogram(returns, 10_000, 30);
    expect(h.edges.length).toBe(31);
});

test('lossHistogram: empty / zero-position / degenerate range safe', () => {
    expect(lossHistogram([], 10_000, 30)).toEqual({ edges: [], counts: [] });
    expect(lossHistogram([0.01], 0, 30)).toEqual({ edges: [], counts: [] });
});

// ── compareMethods ────────────────────────────────────────────────

test('compareMethods: positive diff = historical worse than gaussian (fat tails)', () => {
    const cmp = compareMethods({ var_dollars: 150 }, { var_dollars: 100 });
    expect(cmp.diff).toBe(50);
    expect(cmp.pct).toBeCloseTo(0.5, 9);
});

test('compareMethods: zero-gauss → pct is 0 not Inf', () => {
    expect(compareMethods({ var_dollars: 50 }, { var_dollars: 0 }).pct).toBe(0);
});

// ── demos invariants ──────────────────────────────────────────────

test('demos: each preset emits ≥ 250 finite returns', () => {
    for (const k of ['normal', 'fat-tail', 'crisis', 'low-vol', 'random-walk']) {
        const r = makeDemoReturns(k);
        expect(r.length).toBeGreaterThanOrEqual(250);
        for (const v of r) expect(Number.isFinite(v)).toBe(true);
    }
});

test('demo fat-tail: historical VaR > Gaussian VaR (empirical tail wins)', () => {
    const returns = makeDemoReturns('fat-tail');
    const h = localHistorical(returns, 100_000, 0.95);
    const g = localParametricGaussian(returns, 100_000, 0.95);
    expect(h.var_dollars).toBeGreaterThan(g.var_dollars);
});

test('demo crisis: historical 99% VaR sees the embedded losses', () => {
    const r = localHistorical(makeDemoReturns('crisis'), 100_000, 0.99);
    expect(r.var_dollars).toBeGreaterThan(0);
});

// ── formatters / colors ───────────────────────────────────────────

test('fmt helpers + non-finite guards', () => {
    expect(fmtUSD(1234.5)).toBe('$1234.50');
    expect(fmtUSD(-100)).toBe('-$100.00');
    expect(fmtUSDSigned(100)).toBe('+$100.00');
    expect(fmtPct(0.05, 2)).toBe('5.00%');
    expect(fmtN(0.1234, 2)).toBe('0.12');
    expect(fmtUSD(NaN)).toBe('—');
});

test('methodColor picks distinct hues per method', () => {
    expect(methodColor('historical')).toBe('#00e5ff');
    expect(methodColor('parametric_gaussian')).toBe('#ffd84a');
});
