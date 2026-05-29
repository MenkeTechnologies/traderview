// Deflated Sharpe pure helpers: validator, body shape, confidence
// tiering, trials sweep, formatters.

import { test, expect } from 'vitest';
import {
    validateInputs, buildBody,
    confidenceTier, trialsSweep,
    fmtSR, fmtProb, fmtZ,
} from '../js/_deflated_sharpe_inputs.js';

const base = {
    observed_sharpe: 1.5,
    n_observations: 252,
    skewness: -0.3,
    kurtosis: 4.5,
    n_trials: 20,
};

// ── validateInputs ─────────────────────────────────────────────────

test('validate accepts canonical Bailey-LdP demo', () => {
    expect(validateInputs(base)).toBe(null);
});

test('validate rejects non-finite SR / skew / kurtosis', () => {
    expect(validateInputs({ ...base, observed_sharpe: NaN })).toMatch(/observed_sharpe/);
    expect(validateInputs({ ...base, skewness: Infinity })).toMatch(/skewness/);
    expect(validateInputs({ ...base, kurtosis: NaN })).toMatch(/kurtosis/);
});

test('validate enforces n_observations ≥ 4 integer (Mertens denom)', () => {
    expect(validateInputs({ ...base, n_observations: 3 })).toMatch(/n_observations/);
    expect(validateInputs({ ...base, n_observations: 252.5 })).toMatch(/n_observations/);
});

test('validate enforces n_trials ≥ 1 integer', () => {
    expect(validateInputs({ ...base, n_trials: 0 })).toMatch(/n_trials/);
    expect(validateInputs({ ...base, n_trials: 1.5 })).toMatch(/n_trials/);
});

test('validate catches non-positive Mertens denominator', () => {
    // Choose values that make 1 − γ3·SR + (γ4−1)/4·SR² ≤ 0:
    // SR=10, skew=100 → 1 − 1000 + (kurtosis-1)/4·100 = huge negative
    expect(validateInputs({ ...base, observed_sharpe: 10, skewness: 100 }))
        .toMatch(/Mertens/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody emits exact backend DeflatedSharpeBody shape', () => {
    expect(buildBody(base)).toEqual(base);
});

// ── confidenceTier ────────────────────────────────────────────────

test('confidenceTier buckets follow standard cutoffs', () => {
    expect(confidenceTier(0.995).label).toMatch(/very high/);
    expect(confidenceTier(0.96).label).toMatch(/high/);
    expect(confidenceTier(0.92).label).toMatch(/moderate/);
    expect(confidenceTier(0.60).label).toMatch(/weak/);
    expect(confidenceTier(0.30).label).toMatch(/overfit/);
});

test('confidenceTier flags weak/overfit as neg class', () => {
    expect(confidenceTier(0.40).cls).toBe('neg');
    expect(confidenceTier(0.60).cls).toBe('neg');
});

test('confidenceTier flags ≥95% as pos class', () => {
    expect(confidenceTier(0.96).cls).toBe('pos');
    expect(confidenceTier(0.99).cls).toBe('pos');
});

test('confidenceTier returns em-dash on non-finite', () => {
    expect(confidenceTier(NaN).label).toBe('—');
});

// ── trialsSweep ───────────────────────────────────────────────────

test('trialsSweep returns canonical ladder', () => {
    expect(trialsSweep(20)).toEqual([1, 5, 10, 20, 25, 50, 100, 250, 1000]);
});

test('trialsSweep deduplicates when base coincides with ladder', () => {
    expect(trialsSweep(50)).toEqual([1, 5, 10, 25, 50, 100, 250, 1000]);
});

test('trialsSweep falls back to default-10 on bad base', () => {
    expect(trialsSweep(0)).toEqual([1, 5, 10, 25, 50, 100, 250, 1000]);
    expect(trialsSweep(-1)).toEqual([1, 5, 10, 25, 50, 100, 250, 1000]);
});

// ── formatters ─────────────────────────────────────────────────────

test('fmtSR emits 3-decimal Sharpe', () => {
    expect(fmtSR(1.5)).toBe('1.500');
    expect(fmtSR(-0.123456)).toBe('-0.123');
    expect(fmtSR(NaN)).toBe('—');
});

test('fmtProb emits 2-decimal percentage', () => {
    expect(fmtProb(0.9876)).toBe('98.76%');
    expect(fmtProb(NaN)).toBe('—');
});

test('fmtZ emits signed σ', () => {
    expect(fmtZ(1.96)).toBe('+1.96σ');
    expect(fmtZ(-0.5)).toBe('-0.50σ');
    expect(fmtZ(NaN)).toBe('—');
});
