// Yield Curve PCA pure helpers: validator, factor naming, tenor
// normalization, body shape, color cycling.

import { test, expect } from 'vitest';
import {
    validatePcaInputs, factorName, normalizeTenors,
    buildBody, factorColor,
} from '../js/_yield_curve_pca_inputs.js';

test('validate accepts a clean 5×3 input with topK=3', () => {
    const c = Array.from({ length: 5 }, (_, i) => [i, i + 1, i + 2]);
    expect(validatePcaInputs(c, 3)).toBe(null);
});

test('validate rejects fewer than 5 dates', () => {
    expect(validatePcaInputs([[1, 2], [3, 4]], 1)).toMatch(/at least 5/);
});

test('validate rejects fewer than 2 tenors', () => {
    const c = Array.from({ length: 6 }, () => [1.0]);
    expect(validatePcaInputs(c, 1)).toMatch(/at least 2 tenors/);
});

test('validate rejects ragged matrix', () => {
    const c = [
        [1, 2, 3], [1, 2, 3], [1, 2, 3], [1, 2, 3], [1, 2],
    ];
    expect(validatePcaInputs(c, 2)).toMatch(/row 5/);
});

test('validate rejects non-finite cells', () => {
    const c = [
        [1, 2, 3], [1, 2, 3], [1, 2, NaN], [1, 2, 3], [1, 2, 3],
    ];
    expect(validatePcaInputs(c, 2)).toMatch(/row 3/);
});

test('validate rejects out-of-range topK', () => {
    const c = Array.from({ length: 5 }, () => [1, 2, 3]);
    expect(validatePcaInputs(c, 0)).toMatch(/top_k/);
    expect(validatePcaInputs(c, 4)).toMatch(/top_k/);
    expect(validatePcaInputs(c, 2.5)).toMatch(/top_k/);
});

test('factorName uses Litterman-Scheinkman labels for first 3', () => {
    expect(factorName(0)).toBe('Level');
    expect(factorName(1)).toBe('Slope');
    expect(factorName(2)).toBe('Curvature');
});

test('factorName falls back to PCN for higher indices', () => {
    expect(factorName(3)).toBe('PC4');
    expect(factorName(10)).toBe('PC11');
});

test('normalizeTenors uses Tn defaults when no labels supplied', () => {
    expect(normalizeTenors([], 3)).toEqual(['T1', 'T2', 'T3']);
    expect(normalizeTenors(null, 2)).toEqual(['T1', 'T2']);
});

test('normalizeTenors pads short label lists', () => {
    expect(normalizeTenors(['1Y'], 3)).toEqual(['1Y', 'T2', 'T3']);
});

test('normalizeTenors trims long label lists', () => {
    expect(normalizeTenors(['1Y', '2Y', '5Y', '10Y'], 2)).toEqual(['1Y', '2Y']);
});

test('buildBody emits both fields', () => {
    const c = [[1, 2], [3, 4]];
    expect(buildBody(c, 2)).toEqual({ curves: c, top_k: 2 });
});

test('factorColor cycles after 6 factors', () => {
    expect(factorColor(0)).toBe(factorColor(6));
    expect(factorColor(1)).toBe(factorColor(7));
});

test('factorColor returns valid hex strings', () => {
    for (let i = 0; i < 10; i++) {
        expect(factorColor(i)).toMatch(/^#[0-9a-f]{6}$/i);
    }
});
