// Block bootstrap helpers: parser, validator, body shape,
// localBootstrap Rust-mirror, badges, summarize, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, STATISTICS, MIN_RESAMPLES, MAX_RESAMPLES,
    parseDataBlob, dataToBlob, validateInputs, buildBody, localBootstrap,
    computeStatistic, ciBadge, biasBadge, signifBadge, summarizeData,
    makeDemoInput,
    fmtNum, fmtNumSigned, fmtPct, fmtInt,
} from '../js/_block_bootstrap_inputs.js';

// ── parser ────────────────────────────────────────────────────────

test('parseDataBlob: comma + whitespace, comments + blanks ignored', () => {
    const r = parseDataBlob('0.01 -0.02\n# noise\n0.03, 0.04');
    expect(r.errors).toEqual([]);
    expect(r.data).toEqual([0.01, -0.02, 0.03, 0.04]);
});

test('parseDataBlob: $/% prefixes stripped, ()→neg', () => {
    const r = parseDataBlob('$0.012 -$0.004 (0.005) 50%');
    expect(r.errors).toEqual([]);
    expect(r.data).toEqual([0.012, -0.004, -0.005, 50]);
});

test('parseDataBlob: non-string returns 1 error', () => {
    expect(parseDataBlob(null).errors.length).toBe(1);
});

test('parseDataBlob: bad token reported with line_no', () => {
    expect(parseDataBlob('1 NaNzzz 3').errors[0].line_no).toBe(2);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts valid input', () => {
    expect(validateInputs({
        data: new Array(100).fill(0.01),
        block_size: 10, n_resamples: 500, statistic: 'mean', seed: 0n,
    })).toBe(null);
});

test('validate rejects: bad data / NaN / zero block / short data / bad resamples / bad stat / bad seed', () => {
    const base = { data: new Array(100).fill(0.01), block_size: 10, n_resamples: 500, statistic: 'mean', seed: 0n };
    expect(validateInputs({ ...base, data: 'no' })).toMatch(/data/);
    expect(validateInputs({ ...base, data: [0.01, NaN, 0.02] })).toMatch(/finite|block_size/);
    expect(validateInputs({ ...base, block_size: 0 })).toMatch(/block_size/);
    expect(validateInputs({ ...base, data: new Array(5).fill(0.01), block_size: 10 })).toMatch(/block_size \+ 2/);
    expect(validateInputs({ ...base, n_resamples: 10 })).toMatch(/n_resamples/);
    expect(validateInputs({ ...base, n_resamples: 100_000 })).toMatch(/n_resamples/);
    expect(validateInputs({ ...base, statistic: 'bogus' })).toMatch(/statistic/);
    expect(validateInputs({ ...base, seed: 'abc' })).toMatch(/seed/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: bigint seed → Number, fields passed through', () => {
    const body = buildBody({
        data: [0.01, 0.02], block_size: 5, n_resamples: 100, statistic: 'mean', seed: 42n,
    });
    expect(body).toEqual({
        data: [0.01, 0.02], block_size: 5, n_resamples: 100, statistic: 'mean', seed: 42,
    });
});

// ── computeStatistic ──────────────────────────────────────────────

test('computeStatistic: mean / stdev / sharpe / maxdd', () => {
    expect(computeStatistic([1, 2, 3, 4, 5], 'mean')).toBeCloseTo(3, 9);
    // sample stdev of 1..5 with n-1 = (sqrt(2.5)) ≈ 1.5811
    expect(computeStatistic([1, 2, 3, 4, 5], 'stdev')).toBeCloseTo(1.5811388, 5);
    expect(computeStatistic([0.01, 0.02], 'sharpe_ratio')).toBeGreaterThan(0);
    // max drawdown of cumulative [1, -2, 0.5]: equity [1, -1, -0.5]; peak=1 → dd=2
    expect(computeStatistic([1, -2, 0.5], 'max_drawdown')).toBeCloseTo(2, 9);
});

test('computeStatistic: edge cases', () => {
    expect(computeStatistic([], 'mean')).toBe(null);
    expect(computeStatistic([1], 'stdev')).toBe(null);
    expect(computeStatistic([1], 'sharpe_ratio')).toBe(null);
    expect(computeStatistic([0, 0, 0], 'sharpe_ratio')).toBe(null);
});

// ── localBootstrap parity (mirrors every Rust #[test]) ───────────

test('local: too-short returns null', () => {
    expect(localBootstrap(new Array(5).fill(0.01), 10, 100, 'mean', 42n)).toBe(null);
});

test('local: zero block_size returns null', () => {
    expect(localBootstrap(new Array(100).fill(0.01), 0, 100, 'mean', 42n)).toBe(null);
});

test('local: invalid resample count returns null', () => {
    expect(localBootstrap(new Array(100).fill(0.01), 10, 10, 'mean', 42n)).toBe(null);
    expect(localBootstrap(new Array(100).fill(0.01), 10, 100_000, 'mean', 42n)).toBe(null);
});

test('local: NaN input returns null', () => {
    const d = new Array(100).fill(0.01);
    d[5] = NaN;
    expect(localBootstrap(d, 10, 100, 'mean', 42n)).toBe(null);
});

test('local: bootstrap mean ≈ original for iid-like deterministic series', () => {
    const data = Array.from({ length: 1000 }, (_, i) => Math.sin(i * 0.07) * 0.01);
    const r = localBootstrap(data, 10, 500, 'mean', 42n);
    expect(r).not.toBe(null);
    expect(Math.abs(r.bootstrap_mean - r.original_statistic)).toBeLessThan(0.005);
});

test('local: ci_lower ≤ ci_upper', () => {
    const data = Array.from({ length: 500 }, (_, i) => Math.cos(i * 0.03) * 0.02);
    const r = localBootstrap(data, 20, 500, 'stdev', 999n);
    expect(r).not.toBe(null);
    expect(r.ci_lower_2_5_pct).toBeLessThanOrEqual(r.ci_upper_97_5_pct);
});

test('local: 95% CI brackets original for well-behaved noise', () => {
    let state = 7n;
    const MASK = 0xFFFFFFFFFFFFFFFFn;
    const data = Array.from({ length: 500 }, () => {
        state = (state * 6364136223846793005n + 1442695040888963407n) & MASK;
        return (Number(state >> 32n) / 0xFFFFFFFF - 0.5) * 0.05;
    });
    const r = localBootstrap(data, 20, 1000, 'mean', 42n);
    expect(r).not.toBe(null);
    expect(r.ci_lower_2_5_pct).toBeLessThanOrEqual(r.original_statistic);
    expect(r.ci_upper_97_5_pct).toBeGreaterThanOrEqual(r.original_statistic);
});

test('local: sharpe finite for near-constant input or null cleanly', () => {
    const data = new Array(100).fill(0.01);
    const r = localBootstrap(data, 10, 500, 'sharpe_ratio', 42n);
    if (r != null) expect(Number.isFinite(r.original_statistic)).toBe(true);
});

test('local: max_drawdown non-negative', () => {
    let state = 42n;
    const MASK = 0xFFFFFFFFFFFFFFFFn;
    const data = Array.from({ length: 500 }, () => {
        state = (state * 6364136223846793005n + 1442695040888963407n) & MASK;
        return (Number(state >> 32n) / 0xFFFFFFFF - 0.5) * 0.05;
    });
    const r = localBootstrap(data, 20, 200, 'max_drawdown', 42n);
    expect(r).not.toBe(null);
    expect(r.original_statistic).toBeGreaterThanOrEqual(0);
    expect(r.bootstrap_mean).toBeGreaterThanOrEqual(0);
});

test('local: deterministic for same seed', () => {
    const data = Array.from({ length: 200 }, (_, i) => (i % 3 - 1) * 0.02);
    const a = localBootstrap(data, 10, 200, 'mean', 7n);
    const b = localBootstrap(data, 10, 200, 'mean', 7n);
    expect(a.bootstrap_mean).toBe(b.bootstrap_mean);
    expect(a.ci_lower_2_5_pct).toBe(b.ci_lower_2_5_pct);
    expect(a.ci_upper_97_5_pct).toBe(b.ci_upper_97_5_pct);
});

test('local: different seeds give different bootstrap distributions', () => {
    const data = Array.from({ length: 200 }, (_, i) => (i % 3 - 1) * 0.02);
    const a = localBootstrap(data, 10, 200, 'mean', 1n);
    const b = localBootstrap(data, 10, 200, 'mean', 999n);
    // Originals identical, bootstrap_mean almost certainly differs.
    expect(a.original_statistic).toBe(b.original_statistic);
    expect(a.bootstrap_mean).not.toBe(b.bootstrap_mean);
});

test('local: reports block_size and n_resamples back', () => {
    const data = Array.from({ length: 200 }, (_, i) => i * 0.001);
    const r = localBootstrap(data, 15, 250, 'mean', 0n);
    expect(r.block_size).toBe(15);
    expect(r.n_resamples).toBe(250);
});

// ── badges ────────────────────────────────────────────────────────

test('ciBadge: tight / moderate / wide / extreme / unknown', () => {
    const mk = (lo, hi, orig) => ({
        ci_lower_2_5_pct: lo, ci_upper_97_5_pct: hi, original_statistic: orig,
        bootstrap_mean: orig, bootstrap_stdev: 0, n_resamples: 100, block_size: 10,
    });
    expect(ciBadge(mk(0.9, 1.1, 1.0)).key).toMatch(/tight/);    // ratio 0.2
    expect(ciBadge(mk(0.5, 1.5, 1.0)).key).toMatch(/moderate/); // ratio 1
    expect(ciBadge(mk(-2, 4, 1.0)).key).toMatch(/wide/);        // ratio 6
    expect(ciBadge(mk(-5, 6, 1.0)).key).toMatch(/extreme/);     // ratio 11
    expect(ciBadge(mk(0.9, 1.1, 0)).key).toMatch(/unknown/);    // |orig|=0
    expect(ciBadge(null).key).toMatch(/unknown/);
});

test('biasBadge: tiers', () => {
    const mk = (boot, orig) => ({
        ci_lower_2_5_pct: 0, ci_upper_97_5_pct: 0, original_statistic: orig,
        bootstrap_mean: boot, bootstrap_stdev: 0, n_resamples: 100, block_size: 10,
    });
    expect(biasBadge(mk(1.0,  1.0 )).key).toMatch(/negligible/);    // 0%
    expect(biasBadge(mk(1.1,  1.0 )).key).toMatch(/small/);         // 10%
    expect(biasBadge(mk(1.3,  1.0 )).key).toMatch(/notable/);       // 30%
    expect(biasBadge(mk(1.6,  1.0 )).key).toMatch(/large/);         // 60%
    expect(biasBadge(mk(1.0,  0   )).key).toMatch(/unknown/);
});

test('signifBadge: positive / negative / spans_zero / unknown', () => {
    const mk = (lo, hi) => ({
        ci_lower_2_5_pct: lo, ci_upper_97_5_pct: hi, original_statistic: 0,
        bootstrap_mean: 0, bootstrap_stdev: 0, n_resamples: 100, block_size: 10,
    });
    expect(signifBadge(mk(0.1, 0.5)).key).toMatch(/positive/);
    expect(signifBadge(mk(-0.5, -0.1)).key).toMatch(/negative/);
    expect(signifBadge(mk(-0.2, 0.3)).key).toMatch(/spans_zero/);
    expect(signifBadge(mk(NaN, 1)).key).toMatch(/unknown/);
    expect(signifBadge(null).key).toMatch(/unknown/);
});

// ── summarize ─────────────────────────────────────────────────────

test('summarizeData: counts / sum / mean / extrema', () => {
    const s = summarizeData([1, -2, 3, -4, 5]);
    expect(s.count).toBe(5);
    expect(s.sum).toBe(3);
    expect(s.mean).toBeCloseTo(0.6, 9);
    expect(s.max).toBe(5);
    expect(s.min).toBe(-4);
});

test('summarizeData: empty → count 0, NaN extrema', () => {
    const s = summarizeData([]);
    expect(s.count).toBe(0);
    expect(Number.isNaN(s.max)).toBe(true);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + computes without error', () => {
    for (const k of ['mean-revert','momentum','volatility-cluster','sharpe-strategy',
                     'drawdown-tail','iid-noise','small-sample','fat-tail']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localBootstrap(inp.data, inp.block_size, inp.n_resamples, inp.statistic, inp.seed);
        expect(r).not.toBe(null);
        expect(r.n_resamples).toBeGreaterThan(0);
    }
});

test('demo drawdown-tail uses max_drawdown statistic and returns ≥ 0', () => {
    const inp = makeDemoInput('drawdown-tail');
    expect(inp.statistic).toBe('max_drawdown');
    const r = localBootstrap(inp.data, inp.block_size, inp.n_resamples, inp.statistic, inp.seed);
    expect(r.original_statistic).toBeGreaterThanOrEqual(0);
    expect(r.bootstrap_mean).toBeGreaterThanOrEqual(0);
});

test('demo sharpe-strategy uses sharpe_ratio statistic', () => {
    const inp = makeDemoInput('sharpe-strategy');
    expect(inp.statistic).toBe('sharpe_ratio');
    const r = localBootstrap(inp.data, inp.block_size, inp.n_resamples, inp.statistic, inp.seed);
    expect(r).not.toBe(null);
});

test('demo small-sample passes block_size + 2 constraint', () => {
    const inp = makeDemoInput('small-sample');
    expect(inp.data.length).toBeGreaterThanOrEqual(inp.block_size + 2);
});

// ── round-trip + formatters ──────────────────────────────────────

test('dataToBlob round-trips through parseDataBlob', () => {
    const data = [0.01, -0.02, 0.03];
    const back = parseDataBlob(dataToBlob(data));
    expect(back.errors).toEqual([]);
    expect(back.data).toEqual(data);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtNum(1.23456789)).toBe('1.2346');
    expect(fmtNum(NaN)).toBe('—');
    expect(fmtNumSigned(0.5)).toBe('+0.5000');
    expect(fmtNumSigned(-0.5)).toBe('-0.5000');
    expect(fmtPct(0.0125)).toBe('1.25%');
    expect(fmtInt(42.9)).toBe('42');
    expect(fmtInt(NaN)).toBe('—');
});

test('STATISTICS enum has 4 variants matching Rust', () => {
    expect(STATISTICS).toEqual(['mean', 'stdev', 'sharpe_ratio', 'max_drawdown']);
});

test('DEFAULT_INPUTS has zero-length data (forces user to provide)', () => {
    expect(DEFAULT_INPUTS.data).toEqual([]);
    expect(DEFAULT_INPUTS.block_size).toBeGreaterThan(0);
    expect(DEFAULT_INPUTS.n_resamples).toBeGreaterThanOrEqual(MIN_RESAMPLES);
    expect(DEFAULT_INPUTS.n_resamples).toBeLessThanOrEqual(MAX_RESAMPLES);
});
