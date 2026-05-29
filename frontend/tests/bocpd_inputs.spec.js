// BOCPD pure helpers: validator, top-K change-point selector,
// above-threshold counter, hazard formatter.

import { test, expect } from 'vitest';
import {
    validateInputs, buildBody,
    topChangePoints, countAboveThreshold, fmtHazardPct,
} from '../js/_bocpd_inputs.js';

// ── validateInputs ─────────────────────────────────────────────────

test('validate rejects fewer than 30 returns', () => {
    expect(validateInputs(Array(20).fill(0.01), 0.01)).toMatch(/at least 30/);
});

test('validate rejects non-finite returns', () => {
    const r = Array(50).fill(0.01); r[5] = NaN;
    expect(validateInputs(r, 0.01)).toMatch(/non-finite/);
});

test('validate rejects hazard outside (0, 1)', () => {
    expect(validateInputs(Array(50).fill(0.01), 0)).toMatch(/hazard/);
    expect(validateInputs(Array(50).fill(0.01), 1)).toMatch(/hazard/);
    expect(validateInputs(Array(50).fill(0.01), -0.1)).toMatch(/hazard/);
    expect(validateInputs(Array(50).fill(0.01), NaN)).toMatch(/hazard/);
});

test('validate accepts good input', () => {
    expect(validateInputs(Array(50).fill(0).map((_, i) => Math.sin(i / 5)), 0.01)).toBe(null);
});

// ── buildBody ──────────────────────────────────────────────────────

test('buildBody emits backend shape', () => {
    expect(buildBody([0.01, -0.02, 0.03], 0.05)).toEqual({
        returns: [0.01, -0.02, 0.03],
        hazard: 0.05,
    });
});

// ── topChangePoints ────────────────────────────────────────────────

test('topChangePoints returns highest-prob indices first', () => {
    const probs = [0.1, 0.85, 0.05, 0.92, 0.30];
    const top = topChangePoints(probs, 0.20, 3);
    expect(top.map(t => t.index)).toEqual([3, 1, 4]);
});

test('topChangePoints respects threshold', () => {
    const probs = [0.1, 0.4, 0.05, 0.6, 0.3];
    const top = topChangePoints(probs, 0.5, 10);
    expect(top.length).toBe(1);
    expect(top[0].index).toBe(3);
});

test('topChangePoints caps at topK', () => {
    const probs = [0.5, 0.6, 0.7, 0.8, 0.9];
    const top = topChangePoints(probs, 0.0, 2);
    expect(top.length).toBe(2);
    // Highest-first.
    expect(top[0].index).toBe(4);
    expect(top[1].index).toBe(3);
});

test('topChangePoints skips null / non-finite entries', () => {
    const probs = [0.5, null, NaN, 0.8, undefined];
    const top = topChangePoints(probs, 0.0, 10);
    expect(top.map(t => t.index)).toEqual([3, 0]);
});

test('topChangePoints returns empty on bad inputs', () => {
    expect(topChangePoints(null, 0.5, 3)).toEqual([]);
    expect(topChangePoints([0.5], NaN, 3)).toEqual([]);
    expect(topChangePoints([0.5], 0.5, 0)).toEqual([]);
    expect(topChangePoints([0.5], 0.5, 1.5)).toEqual([]);
});

// ── countAboveThreshold ────────────────────────────────────────────

test('countAboveThreshold tallies finite entries ≥ threshold', () => {
    expect(countAboveThreshold([0.1, 0.5, 0.8, 0.2, 0.9], 0.4)).toBe(3);
});

test('countAboveThreshold skips null / NaN', () => {
    expect(countAboveThreshold([0.5, null, NaN, 0.8], 0.4)).toBe(2);
});

test('countAboveThreshold returns 0 on bad inputs', () => {
    expect(countAboveThreshold(null, 0.5)).toBe(0);
    expect(countAboveThreshold([0.5], NaN)).toBe(0);
});

// ── fmtHazardPct ───────────────────────────────────────────────────

test('fmtHazardPct emits 2-decimal percent', () => {
    expect(fmtHazardPct(0.01)).toBe('1.00%');
    expect(fmtHazardPct(0.005)).toBe('0.50%');
});

test('fmtHazardPct returns "—" on non-finite', () => {
    expect(fmtHazardPct(NaN)).toBe('—');
    expect(fmtHazardPct(null)).toBe('—');
});
