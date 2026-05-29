// Brier Score helpers: parser, validator, body shape,
// localCompute Rust-mirror, badges, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_BINS,
    parsePairsBlob, pairsToBlob, validateInputs, buildBody, localCompute,
    brierBadge, skillScore, reliabilityBins,
    makeDemoInput,
    fmtBrier, fmtSkill, fmtPct, fmtInt,
} from '../js/_brier_score_inputs.js';

// ── constants ─────────────────────────────────────────────────────

test('DEFAULT_BINS = 10 (matches Rust)', () => {
    expect(DEFAULT_BINS).toBe(10);
});

// ── parser ────────────────────────────────────────────────────────

test('parsePairsBlob: 2 tokens per line, comments + blanks ignored', () => {
    const r = parsePairsBlob('0.75 1\n# day 2\n0.30, 0');
    expect(r.errors).toEqual([]);
    expect(r.probabilities).toEqual([0.75, 0.30]);
    expect(r.outcomes).toEqual([1, 0]);
});

test('parsePairsBlob: rejects wrong count / out-of-range prob / non-binary outcome', () => {
    expect(parsePairsBlob('0.5').errors[0].message).toMatch(/2 tokens/);
    expect(parsePairsBlob('1.5 1').errors[0].message).toMatch(/\[0, 1\]/);
    expect(parsePairsBlob('0.5 2').errors[0].message).toMatch(/0 or 1/);
});

test('parsePairsBlob: non-string returns 1 error', () => {
    expect(parsePairsBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts non-empty defaults', () => {
    expect(validateInputs({ probabilities: [0.5], outcomes: [1], n_bins: 10 })).toBe(null);
});

test('validate rejects: bad arrays / length mismatch / out-of-range / bad bins', () => {
    expect(validateInputs({ probabilities: 'no', outcomes: [1], n_bins: 10 })).toMatch(/probabilities/);
    expect(validateInputs({ probabilities: [0.5], outcomes: [1, 0], n_bins: 10 })).toMatch(/length/);
    expect(validateInputs({ probabilities: [1.5], outcomes: [1], n_bins: 10 })).toMatch(/\[0, 1\]/);
    expect(validateInputs({ probabilities: [0.5], outcomes: [2], n_bins: 10 })).toMatch(/0 or 1/);
    expect(validateInputs({ probabilities: [0.5], outcomes: [1], n_bins: 0 })).toMatch(/n_bins/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: forwards all 3 fields', () => {
    expect(buildBody({ probabilities: [0.5], outcomes: [1], n_bins: 5 }))
        .toEqual({ probabilities: [0.5], outcomes: [1], n_bins: 5 });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: empty → null', () => {
    expect(localCompute([], [], 10)).toBeNull();
});

test('local: length mismatch → null', () => {
    expect(localCompute([0.5, 0.5], [1], 10)).toBeNull();
});

test('local: invalid probability → null', () => {
    expect(localCompute([1.1, 0.5], [1, 0], 10)).toBeNull();
    expect(localCompute([-0.1, 0.5], [1, 0], 10)).toBeNull();
    expect(localCompute([NaN, 0.5], [1, 0], 10)).toBeNull();
});

test('local: invalid outcome → null', () => {
    expect(localCompute([0.5, 0.5], [1, 2], 10)).toBeNull();
});

test('local: perfect forecast → BS=0', () => {
    const r = localCompute([1, 0, 1, 0, 1], [1, 0, 1, 0, 1], 10);
    expect(r.brier_score).toBeCloseTo(0, 12);
});

test('local: random 0.5 forecasts on balanced sample → BS=0.25=uncertainty', () => {
    const r = localCompute(new Array(10).fill(0.5), [1, 0, 1, 0, 1, 0, 1, 0, 1, 0], 10);
    expect(r.brier_score).toBeCloseTo(0.25, 12);
    expect(r.uncertainty).toBeCloseTo(0.25, 12);
});

test('local: Murphy decomposition BS = reliability − resolution + uncertainty (one-per-bin)', () => {
    const probs = Array.from({ length: 10 }, (_, i) => 0.05 + i * 0.1);
    const outcomes = [0, 0, 1, 0, 1, 1, 0, 1, 1, 1];
    const r = localCompute(probs, outcomes, 10);
    const recomposed = r.reliability - r.resolution + r.uncertainty;
    expect(Math.abs(r.brier_score - recomposed)).toBeLessThan(1e-9);
});

test('local: base rate = mean of outcomes', () => {
    const r = localCompute(new Array(10).fill(0.5),
        [1, 1, 1, 0, 0, 0, 0, 0, 0, 0], 10);
    expect(r.base_rate).toBeCloseTo(0.3, 12);
});

test('local: n_observations reported', () => {
    const r = localCompute(new Array(7).fill(0.5), new Array(7).fill(1), 5);
    expect(r.n_observations).toBe(7);
});

test('local: probability of exactly 1.0 buckets into last bin (clamped)', () => {
    const r = localCompute([1.0, 1.0, 0], [1, 1, 0], 10);
    // No null exception, no out-of-bounds.
    expect(r.brier_score).toBeGreaterThanOrEqual(0);
});

test('local: reliability is ≥ 0', () => {
    const r = localCompute([0.1, 0.5, 0.9], [0, 1, 1], 10);
    expect(r.reliability).toBeGreaterThanOrEqual(0);
});

test('local: resolution is ≥ 0', () => {
    const r = localCompute([0.1, 0.5, 0.9], [0, 1, 1], 10);
    expect(r.resolution).toBeGreaterThanOrEqual(0);
});

// ── brierBadge / skillScore / reliabilityBins ────────────────────

test('brierBadge: tiers (perfect / excellent / good / useful / coin_flip / worse)', () => {
    expect(brierBadge(0.005, 0.25).key).toMatch(/perfect/);
    expect(brierBadge(0.05, 0.25).key).toMatch(/excellent/);
    expect(brierBadge(0.15, 0.25).key).toMatch(/good/);
    expect(brierBadge(0.22, 0.25).key).toMatch(/useful/);
    expect(brierBadge(0.25, 0.25).key).toMatch(/coin_flip/);
    expect(brierBadge(0.35, 0.25).key).toMatch(/worse_than_random/);
    expect(brierBadge(null, 0.25).key).toMatch(/unknown/);
});

test('skillScore: 1 − BS/uncertainty', () => {
    expect(skillScore(0.125, 0.25)).toBeCloseTo(0.5, 9);
    expect(skillScore(0.25, 0.25)).toBeCloseTo(0, 9);
    expect(Number.isNaN(skillScore(0.1, 0))).toBe(true);
});

test('reliabilityBins: counts + mean_pred + mean_actual per bin', () => {
    const probs = [0.05, 0.15, 0.85, 0.95];
    const outcomes = [0, 0, 1, 1];
    const bins = reliabilityBins(probs, outcomes, 10);
    expect(bins.length).toBe(10);
    expect(bins[0].count).toBe(1);
    expect(bins[1].count).toBe(1);
    expect(bins[8].count).toBe(1);
    expect(bins[9].count).toBe(1);
    expect(bins[0].mean_pred).toBeCloseTo(0.05, 12);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + scores to non-null report', () => {
    for (const k of ['perfect','random-coin-flip','well-calibrated','overconfident',
                     'underconfident','flipped-sign','rare-event','fine-bins']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.probabilities, inp.outcomes, inp.n_bins);
        expect(r).not.toBeNull();
        expect(r.brier_score).toBeGreaterThanOrEqual(0);
        expect(r.brier_score).toBeLessThanOrEqual(1);
    }
});

test('demo perfect: BS = 0', () => {
    const inp = makeDemoInput('perfect');
    const r = localCompute(inp.probabilities, inp.outcomes, inp.n_bins);
    expect(r.brier_score).toBeCloseTo(0, 12);
});

test('demo random-coin-flip: BS = 0.25', () => {
    const inp = makeDemoInput('random-coin-flip');
    const r = localCompute(inp.probabilities, inp.outcomes, inp.n_bins);
    expect(r.brier_score).toBeCloseTo(0.25, 12);
});

test('demo flipped-sign: BS > uncertainty (worse than coin flip)', () => {
    const inp = makeDemoInput('flipped-sign');
    const r = localCompute(inp.probabilities, inp.outcomes, inp.n_bins);
    expect(r.brier_score).toBeGreaterThan(r.uncertainty);
});

test('demo well-calibrated: BS < uncertainty (informative)', () => {
    const inp = makeDemoInput('well-calibrated');
    const r = localCompute(inp.probabilities, inp.outcomes, inp.n_bins);
    expect(r.brier_score).toBeLessThan(r.uncertainty);
});

test('demo rare-event: base_rate close to 5%', () => {
    const inp = makeDemoInput('rare-event');
    const r = localCompute(inp.probabilities, inp.outcomes, inp.n_bins);
    expect(Math.abs(r.base_rate - 0.05)).toBeLessThan(0.03);
});

test('demo fine-bins: n_bins = 50', () => {
    const inp = makeDemoInput('fine-bins');
    expect(inp.n_bins).toBe(50);
});

// ── round-trip + formatters ──────────────────────────────────────

test('pairsToBlob round-trips through parsePairsBlob', () => {
    const probs = [0.75, 0.30];
    const outs = [1, 0];
    const back = parsePairsBlob(pairsToBlob(probs, outs));
    expect(back.errors).toEqual([]);
    expect(back.probabilities).toEqual(probs);
    expect(back.outcomes).toEqual(outs);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtBrier(0.1234)).toBe('0.1234');
    expect(fmtSkill(0.5)).toBe('+0.5000');
    expect(fmtSkill(-0.5)).toBe('-0.5000');
    expect(fmtPct(0.05)).toBe('5.00%');
    expect(fmtInt(7.9)).toBe('7');
    expect(fmtBrier(null)).toBe('—');
    expect(fmtSkill(NaN)).toBe('—');
});
