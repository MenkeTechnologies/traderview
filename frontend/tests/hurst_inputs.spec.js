// Hurst Exponent pure helpers: chunk-size parser, validator, regime
// classifier + strength, CSS-class picker.

import { test, expect } from 'vitest';
import {
    parseChunkSizes, validateInputs, buildBody,
    regimeLabelKey, regimeStrengthKey, regimeCssClass,
} from '../js/_hurst_inputs.js';

// ── parseChunkSizes ────────────────────────────────────────────────

test('parseChunkSizes defaults canonical [10, 20, 50, 100, 250] on empty', () => {
    expect(parseChunkSizes('').value).toEqual([10, 20, 50, 100, 250]);
    expect(parseChunkSizes('   ').value).toEqual([10, 20, 50, 100, 250]);
});

test('parseChunkSizes handles space/comma separators and # comments', () => {
    const r = parseChunkSizes('# header\n10 20, 30\n# inline\n50');
    expect(r.errors).toEqual([]);
    expect(r.value).toEqual([10, 20, 30, 50]);
});

test('parseChunkSizes rejects non-integer / < 2 chunk sizes', () => {
    const r = parseChunkSizes('5 1 abc 10.5 20');
    expect(r.value).toEqual([5, 20]);
    expect(r.errors.length).toBe(3);
});

// ── validateInputs ─────────────────────────────────────────────────

test('validate rejects too few returns', () => {
    expect(validateInputs([1, 2, 3], [2])).toMatch(/at least 10/);
});

test('validate rejects non-finite returns', () => {
    const r = Array(20).fill(0.01); r[5] = NaN;
    expect(validateInputs(r, [10])).toMatch(/non-finite/);
});

test('validate requires at least 2 chunk sizes', () => {
    expect(validateInputs(Array(20).fill(0.01), [10])).toMatch(/at least 2 chunk/);
});

test('validate rejects non-integer / < 2 chunk sizes', () => {
    expect(validateInputs(Array(20).fill(0.01), [10, 1])).toMatch(/integer ≥ 2/);
    expect(validateInputs(Array(20).fill(0.01), [10, 5.5])).toMatch(/integer ≥ 2/);
});

test('validate rejects chunk sizes larger than series', () => {
    expect(validateInputs(Array(20).fill(0.01), [10, 25])).toMatch(/≤ series length/);
});

test('validate accepts good input', () => {
    expect(validateInputs(Array(100).fill(0.01).map((_, i) => Math.sin(i / 5)), [10, 20, 50]))
        .toBe(null);
});

// ── buildBody ──────────────────────────────────────────────────────

test('buildBody emits backend shape', () => {
    expect(buildBody([1, 2, 3], [10, 20])).toEqual({ returns: [1, 2, 3], chunk_sizes: [10, 20] });
});

// ── regimeLabelKey ─────────────────────────────────────────────────

test('regimeLabelKey: H < 0.45 → mean_reverting key', () => {
    expect(regimeLabelKey(0.30)).toBe('view.hurst.regime.mean_reverting');
    expect(regimeLabelKey(0.44)).toBe('view.hurst.regime.mean_reverting');
});

test('regimeLabelKey: H near 0.5 → random_walk key', () => {
    expect(regimeLabelKey(0.45)).toBe('view.hurst.regime.random_walk');
    expect(regimeLabelKey(0.50)).toBe('view.hurst.regime.random_walk');
    expect(regimeLabelKey(0.55)).toBe('view.hurst.regime.random_walk');
});

test('regimeLabelKey: H > 0.55 → trending key', () => {
    expect(regimeLabelKey(0.65)).toBe('view.hurst.regime.trending');
    expect(regimeLabelKey(0.95)).toBe('view.hurst.regime.trending');
});

test('regimeLabelKey returns unknown key on non-finite', () => {
    expect(regimeLabelKey(NaN)).toBe('view.hurst.regime.unknown');
    expect(regimeLabelKey(null)).toBe('view.hurst.regime.unknown');
});

// ── regimeStrengthKey ──────────────────────────────────────────────

test('regimeStrengthKey buckets: |Δ| < 0.05 weak key', () => {
    expect(regimeStrengthKey(0.52)).toBe('view.hurst.strength.weak');
    expect(regimeStrengthKey(0.49)).toBe('view.hurst.strength.weak');
});

test('regimeStrengthKey: 0.05 ≤ |Δ| < 0.15 moderate key', () => {
    expect(regimeStrengthKey(0.60)).toBe('view.hurst.strength.moderate');
    expect(regimeStrengthKey(0.40)).toBe('view.hurst.strength.moderate');
});

test('regimeStrengthKey: |Δ| ≥ 0.15 strong key', () => {
    expect(regimeStrengthKey(0.70)).toBe('view.hurst.strength.strong');
    expect(regimeStrengthKey(0.20)).toBe('view.hurst.strength.strong');
});

test('regimeStrengthKey returns unknown key on non-finite', () => {
    expect(regimeStrengthKey(NaN)).toBe('view.hurst.strength.unknown');
});

// ── regimeCssClass ─────────────────────────────────────────────────

test('regimeCssClass: trending → pos, mean-reverting → neg, else empty', () => {
    expect(regimeCssClass(0.65)).toBe('pos');
    expect(regimeCssClass(0.35)).toBe('neg');
    expect(regimeCssClass(0.50)).toBe('');
    expect(regimeCssClass(NaN)).toBe('');
});
