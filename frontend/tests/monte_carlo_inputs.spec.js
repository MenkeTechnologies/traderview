// Monte Carlo trade-sequence helpers: parser, validator, body shape,
// local simulator parity (Rust LCG / Lemire bounded-rand), histogram,
// demo presets, formatters.

import { test, expect } from 'vitest';
import {
    DEFAULT_CONFIG, parseRBlob, validateInputs, buildBody,
    localSimulate, localSimulateWithCurves, endingHistogram, pct, Lcg,
    makeDemoR, ruinBadge, fmtUSD, fmtPct, fmtNum,
} from '../js/_monte_carlo_inputs.js';

const baseCfg = (over = {}) => ({ ...DEFAULT_CONFIG, ...over });

// ── parseRBlob ────────────────────────────────────────────────────

test('parseRBlob accepts csv / whitespace / newline mix', () => {
    const r = parseRBlob('1.5,-1\n0.5 -0.5\n2.0');
    expect(r.errors).toEqual([]);
    expect(r.r).toEqual([1.5, -1, 0.5, -0.5, 2.0]);
});

test('parseRBlob: comments stripped, blanks ignored', () => {
    expect(parseRBlob('1.0 # win\n# note\n-1.0').r).toEqual([1.0, -1.0]);
});

test('parseRBlob: flags non-finite tokens with index', () => {
    const r = parseRBlob('1 abc 2');
    expect(r.errors.length).toBe(1);
    expect(r.r).toEqual([1, 2]);
});

test('parseRBlob: non-string returns 1 error', () => {
    expect(parseRBlob(null).errors.length).toBe(1);
});

// ── validateInputs / buildBody ────────────────────────────────────

test('validate accepts good inputs', () => {
    expect(validateInputs([1, -1], baseCfg())).toBe(null);
});

test('validate rejects empty R / non-finite / bad cfg', () => {
    expect(validateInputs([], baseCfg())).toMatch(/≥ 1 historical/);
    expect(validateInputs([NaN], baseCfg())).toMatch(/finite/);
    expect(validateInputs([1], baseCfg({ n_curves: 0 }))).toMatch(/n_curves/);
    expect(validateInputs([1], baseCfg({ trades_per_curve: 0 }))).toMatch(/trades_per_curve/);
    expect(validateInputs([1], baseCfg({ start_equity: 0 }))).toMatch(/start_equity/);
    expect(validateInputs([1], baseCfg({ ruin_threshold: -1 }))).toMatch(/ruin_threshold/);
});

test('validate enforces perf caps', () => {
    expect(validateInputs([1], baseCfg({ n_curves: 100_000 }))).toMatch(/50000/);
    expect(validateInputs([1], baseCfg({ trades_per_curve: 100_000 }))).toMatch(/10000/);
});

test('buildBody emits backend MonteCarloBody shape (defensive copies)', () => {
    const r = [1, -1];
    const body = buildBody(r, baseCfg());
    expect(body.historical_r).toEqual([1, -1]);
    r.push(99);
    expect(body.historical_r.length).toBe(2);  // not aliased
    expect(body.config).toEqual(DEFAULT_CONFIG);
});

// ── Lcg (Rust LCG / Lemire bounded-rand parity) ───────────────────

test('Lcg: deterministic stream for same seed', () => {
    const a = new Lcg(42n);
    const b = new Lcg(42n);
    for (let i = 0; i < 50; i++) expect(a.nextU64()).toBe(b.nextU64());
});

test('Lcg: different seeds produce different streams', () => {
    const a = new Lcg(0n);
    const b = new Lcg(1n);
    let same = 0;
    for (let i = 0; i < 10; i++) if (a.nextU64() === b.nextU64()) same++;
    expect(same).toBeLessThan(2);
});

test('Lcg.nextBounded: bound=1 always returns 0', () => {
    const r = new Lcg(7n);
    for (let i = 0; i < 20; i++) expect(r.nextBounded(1n)).toBe(0n);
});

test('Lcg.nextBounded: stays inside [0, bound)', () => {
    const r = new Lcg(7n);
    for (let i = 0; i < 200; i++) {
        const v = r.nextBounded(7n);
        expect(v).toBeGreaterThanOrEqual(0n);
        expect(v).toBeLessThan(7n);
    }
});

test('Lcg.nextBounded: bound=0 returns 0 (guarded)', () => {
    expect(new Lcg(0n).nextBounded(0n)).toBe(0n);
});

// ── localSimulate ────────────────────────────────────────────────

test('localSimulate: empty R / zero curves / zero trades → null', () => {
    expect(localSimulate([], baseCfg())).toBeNull();
    expect(localSimulate([1], baseCfg({ n_curves: 0 }))).toBeNull();
    expect(localSimulate([1], baseCfg({ trades_per_curve: 0 }))).toBeNull();
});

test('localSimulate: deterministic for same seed (Rust LCG)', () => {
    const r = [1, -1, 1.5, -0.5];
    const cfg = baseCfg({ n_curves: 50, trades_per_curve: 30, seed: 12345 });
    const r1 = localSimulate(r, cfg);
    const r2 = localSimulate(r, cfg);
    expect(r1.mean_ending_equity).toBe(r2.mean_ending_equity);
    expect(r1.probability_of_ruin).toBe(r2.probability_of_ruin);
});

test('localSimulate: positive-edge demo → high P(profitable)', () => {
    const r = makeDemoR('positive-edge');
    const out = localSimulate(r, baseCfg({ n_curves: 500, trades_per_curve: 100, seed: 1 }));
    expect(out.probability_profitable).toBeGreaterThan(0.7);
});

test('localSimulate: negative-edge demo → low P(profitable)', () => {
    const r = makeDemoR('negative-edge');
    const out = localSimulate(r, baseCfg({ n_curves: 500, trades_per_curve: 100, seed: 1 }));
    expect(out.probability_profitable).toBeLessThan(0.3);
});

test('localSimulate: ending percentiles are monotonically increasing', () => {
    const r = [1, -1, 0.5, -0.5];
    const out = localSimulate(r, baseCfg({ n_curves: 200, seed: 9 }));
    expect(out.ending_equity_p05).toBeLessThanOrEqual(out.ending_equity_p25);
    expect(out.ending_equity_p25).toBeLessThanOrEqual(out.ending_equity_p50);
    expect(out.ending_equity_p50).toBeLessThanOrEqual(out.ending_equity_p75);
    expect(out.ending_equity_p75).toBeLessThanOrEqual(out.ending_equity_p95);
});

test('localSimulate: max-drawdown percentiles are monotonically increasing', () => {
    const r = [1, -1, 0.5, -0.5];
    const out = localSimulate(r, baseCfg({ n_curves: 200, seed: 9 }));
    expect(out.max_drawdown_p05).toBeLessThanOrEqual(out.max_drawdown_p50);
    expect(out.max_drawdown_p50).toBeLessThanOrEqual(out.max_drawdown_p95);
});

test('localSimulate: probabilities in [0, 1]', () => {
    const out = localSimulate(makeDemoR('fat-tail'),
        baseCfg({ n_curves: 200, trades_per_curve: 50, seed: 42 }));
    expect(out.probability_of_ruin).toBeGreaterThanOrEqual(0);
    expect(out.probability_of_ruin).toBeLessThanOrEqual(1);
    expect(out.probability_profitable).toBeGreaterThanOrEqual(0);
    expect(out.probability_profitable).toBeLessThanOrEqual(1);
});

test('localSimulate: lossy R with thin start equity → non-zero ruin probability', () => {
    // Pure-loss distribution; 100 starting equity vs 1R/trade losses
    // → every curve must hit ruin within 100 trades.
    const out = localSimulate([-1, -1, -1, -1],
        baseCfg({ n_curves: 50, trades_per_curve: 100, seed: 7,
                  start_equity: 100, ruin_threshold: 0 }));
    expect(out.probability_of_ruin).toBe(1);
});

test('localSimulateWithCurves: ending array length == n_curves', () => {
    const cfg = baseCfg({ n_curves: 50, trades_per_curve: 20, seed: 3 });
    const { report, ending, maxDds } = localSimulateWithCurves([1, -1], cfg);
    expect(report).not.toBeNull();
    expect(ending.length).toBe(50);
    expect(maxDds.length).toBe(50);
});

// ── pct + endingHistogram ─────────────────────────────────────────

test('pct: q=0 → min, q=1 → max', () => {
    const s = [1, 2, 3, 4, 5];
    expect(pct(s, 0)).toBe(1);
    expect(pct(s, 1)).toBe(5);
    expect(pct(s, 0.5)).toBe(3);
});

test('pct: empty array returns 0', () => {
    expect(pct([], 0.5)).toBe(0);
});

test('endingHistogram: bins all values; counts sum to n', () => {
    const vals = [100, 120, 150, 180, 200];
    const h = endingHistogram(vals, 10);
    expect(h.centers.length).toBe(10);
    expect(h.counts.reduce((a, b) => a + b, 0)).toBe(5);
});

test('endingHistogram: degenerate all-same input emits single bucket', () => {
    const h = endingHistogram([100, 100, 100], 10);
    expect(h.centers).toEqual([100]);
    expect(h.counts).toEqual([3]);
});

test('endingHistogram: empty / zero-position safe', () => {
    expect(endingHistogram([], 10)).toEqual({ centers: [], counts: [] });
});

// ── ruinBadge ─────────────────────────────────────────────────────

test('ruinBadge: thresholds 0 / >0 / ≥2% / ≥10%', () => {
    expect(ruinBadge(0).key).toMatch(/no_ruin/);
    expect(ruinBadge(0.005).key).toMatch(/low_ruin/);
    expect(ruinBadge(0.05).key).toMatch(/moderate_ruin/);
    expect(ruinBadge(0.20).key).toMatch(/high_ruin/);
    expect(ruinBadge(NaN).key).toMatch(/unknown/);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset returns exactly 100 finite R values', () => {
    for (const k of ['positive-edge', 'negative-edge', 'fat-tail', 'lumpy-winner', 'random']) {
        const r = makeDemoR(k);
        expect(r.length).toBe(100);
        for (const v of r) expect(Number.isFinite(v)).toBe(true);
    }
});

test('demo positive-edge mean > 0', () => {
    const r = makeDemoR('positive-edge');
    const m = r.reduce((a, b) => a + b, 0) / r.length;
    expect(m).toBeGreaterThan(0);
});

test('demo negative-edge mean < 0', () => {
    const r = makeDemoR('negative-edge');
    const m = r.reduce((a, b) => a + b, 0) / r.length;
    expect(m).toBeLessThan(0);
});

test('demo fat-tail: includes at least one large negative outlier', () => {
    const r = makeDemoR('fat-tail');
    expect(Math.min(...r)).toBeLessThanOrEqual(-4);
});

// ── DEFAULT_CONFIG / formatters ───────────────────────────────────

test('DEFAULT_CONFIG matches backend defaults shape', () => {
    expect(DEFAULT_CONFIG).toEqual({
        n_curves: 1000, trades_per_curve: 100,
        start_equity: 10_000, ruin_threshold: 5_000, seed: 42,
    });
});

test('formatters: USD/pct/num + non-finite guards', () => {
    expect(fmtUSD(1234, 0)).toBe('$1234');
    expect(fmtUSD(-50, 0)).toBe('-$50');
    expect(fmtPct(0.05, 2)).toBe('5.00%');
    expect(fmtNum(3.14159, 2)).toBe('3.14');
    expect(fmtUSD(NaN)).toBe('—');
    expect(fmtPct(null)).toBe('—');
});
