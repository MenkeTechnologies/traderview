// IV Rank pure helpers: history parser, validator, body shape,
// environment classifier, rank-vs-percentile divergence note, demo
// generator, formatters.

import { test, expect } from 'vitest';
import {
    parseHistory, validateInputs, buildBody,
    rankEnvironment, rankVsPercentileNote,
    makeDemoHistory, fmtIv, fmtRank,
} from '../js/_iv_rank_inputs.js';

// ── parseHistory (delegates to shared parseFloatBlob w/ nonNegative) ─

test('parseHistory accepts decimals and flags negatives via nonNegative gate', () => {
    const r = parseHistory('0.22\n0.24\n-0.1\n# comment\n0.30');
    expect(r.value).toEqual([0.22, 0.24, 0.30]);
    expect(r.errors.length).toBe(1);
    expect(r.errors[0].message).toMatch(/negative/);
});

// ── validateInputs ─────────────────────────────────────────────────

test('validate accepts good current_iv + ≥10 history', () => {
    const hist = Array(20).fill(0.25);
    expect(validateInputs(0.30, hist)).toBe(null);
});

test('validate rejects negative or non-finite current_iv', () => {
    expect(validateInputs(NaN, [0.1])).toMatch(/current_iv/);
    expect(validateInputs(-0.1, [0.1])).toMatch(/current_iv/);
});

test('validate rejects < 10 history observations', () => {
    expect(validateInputs(0.25, Array(5).fill(0.20))).toMatch(/at least 10/);
});

test('validate rejects history containing non-finite or negative values', () => {
    const hist = [0.2, 0.21, 0.19, 0.22, 0.21, 0.20, 0.23, 0.22, 0.21, NaN];
    expect(validateInputs(0.25, hist)).toMatch(/non-negative finite/);
    const hist2 = [0.2, 0.21, 0.19, 0.22, 0.21, 0.20, 0.23, 0.22, 0.21, -0.1];
    expect(validateInputs(0.25, hist2)).toMatch(/non-negative finite/);
});

test('validate accepts a non-array history with proper message', () => {
    expect(validateInputs(0.25, null)).toMatch(/history/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody emits backend IvRankBody shape', () => {
    expect(buildBody(0.30, [0.2, 0.3])).toEqual({ current_iv: 0.30, history: [0.2, 0.3] });
});

// ── rankEnvironment ───────────────────────────────────────────────

test('rankEnvironment buckets follow trader convention (25 / 75 cuts)', () => {
    expect(rankEnvironment(10).label).toBe('LOW');
    expect(rankEnvironment(10).cls).toBe('neg');
    expect(rankEnvironment(50).label).toBe('NORMAL');
    expect(rankEnvironment(50).cls).toBe('');
    expect(rankEnvironment(90).label).toBe('HIGH');
    expect(rankEnvironment(90).cls).toBe('pos');
});

test('rankEnvironment boundary points (25 → normal, 75 → normal)', () => {
    expect(rankEnvironment(25).label).toBe('NORMAL');
    expect(rankEnvironment(75).label).toBe('NORMAL');
});

test('rankEnvironment hint flips for sell-premium vs buy-premium guidance', () => {
    expect(rankEnvironment(10).hint).toMatch(/long premium|debit/);
    expect(rankEnvironment(90).hint).toMatch(/short premium|credit/);
});

test('rankEnvironment returns em-dash on non-finite', () => {
    expect(rankEnvironment(NaN).label).toBe('—');
});

// ── rankVsPercentileNote ──────────────────────────────────────────

test('rankVsPercentileNote: tight agreement → "agree closely"', () => {
    expect(rankVsPercentileNote(50, 55)).toMatch(/agree closely/);
});

test('rankVsPercentileNote: mild divergence (10-20pt)', () => {
    expect(rankVsPercentileNote(50, 65)).toMatch(/mild divergence/);
});

test('rankVsPercentileNote: large divergence (≥20pt) prefers percentile', () => {
    expect(rankVsPercentileNote(85, 30)).toMatch(/prefer percentile/);
});

test('rankVsPercentileNote returns empty string when non-finite', () => {
    expect(rankVsPercentileNote(NaN, 50)).toBe('');
});

// ── makeDemoHistory ───────────────────────────────────────────────

test('makeDemoHistory is deterministic for fixed seed', () => {
    expect(makeDemoHistory(42)).toEqual(makeDemoHistory(42));
});

test('makeDemoHistory has 252 values', () => {
    expect(makeDemoHistory(1).length).toBe(252);
});

test('makeDemoHistory all values are non-negative finite', () => {
    const hist = makeDemoHistory(7);
    expect(hist.every(v => Number.isFinite(v) && v >= 0)).toBe(true);
});

test('makeDemoHistory injects an earnings spike (max ≥ 0.50 in days 240-244)', () => {
    const hist = makeDemoHistory(1);
    const spike = Math.max(...hist.slice(240, 245));
    const baseline = Math.max(...hist.slice(0, 200));
    expect(spike).toBeGreaterThan(baseline * 1.5);
});

// ── formatters ─────────────────────────────────────────────────────

test('fmtIv emits 2-decimal percentage', () => {
    expect(fmtIv(0.2533)).toBe('25.33%');
    expect(fmtIv(NaN)).toBe('—');
});

test('fmtRank emits 1-decimal', () => {
    expect(fmtRank(67.834)).toBe('67.8');
    expect(fmtRank(NaN)).toBe('—');
});
