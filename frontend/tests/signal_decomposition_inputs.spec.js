// Signal-decomposition helpers: validator, defaults, residual, method registry.

import { test, expect } from 'vitest';
import {
    METHODS, validateInputs, defaultOpts, reconstructionResidual,
    parseSeries,
} from '../js/_signal_decomposition_inputs.js';

// ── METHODS registry ──────────────────────────────────────────────

test('METHODS has emd, wavelet, ssa', () => {
    expect(Object.keys(METHODS).sort()).toEqual(['emd', 'ssa', 'wavelet']);
});

test('emd buildBody passes series + max_imfs + max_sift_iter', () => {
    expect(METHODS.emd.buildBody([1, 2, 3], { max_imfs: 4, max_sift_iter: 50 }))
        .toEqual({ series: [1, 2, 3], max_imfs: 4, max_sift_iter: 50 });
});

test('wavelet buildBody passes series + levels only', () => {
    expect(METHODS.wavelet.buildBody([1, 2, 3], { levels: 3 }))
        .toEqual({ series: [1, 2, 3], levels: 3 });
});

test('ssa buildBody passes series + window only', () => {
    expect(METHODS.ssa.buildBody([1, 2, 3], { window: 8 }))
        .toEqual({ series: [1, 2, 3], window: 8 });
});

// ── validateInputs: cross-method gates ────────────────────────────

test('rejects unknown method id', () => {
    const err = validateInputs('zonk', [1, 2, 3, 4, 5, 6, 7, 8], {});
    expect(err).toMatch(/zonk/);
});

test('rejects series shorter than 8', () => {
    expect(validateInputs('emd', [1, 2, 3], { max_imfs: 5, max_sift_iter: 50 }))
        .toMatch(/8/);
});

test('rejects non-finite series', () => {
    const series = [1, 2, 3, 4, 5, 6, 7, NaN];
    expect(validateInputs('emd', series, { max_imfs: 5, max_sift_iter: 50 }))
        .toMatch(/finite/);
});

// ── method-specific gates ─────────────────────────────────────────

test('ssa requires n >= 2·window — n=10 / window=8 fails', () => {
    // 10 < 16, so SSA rejects
    expect(validateInputs('ssa', new Array(10).fill(0), { window: 8 }))
        .toMatch(/8/);
});

test('ssa accepts n = 2·window exactly (n=16 / window=8)', () => {
    expect(validateInputs('ssa', new Array(16).fill(0), { window: 8 }))
        .toBeNull();
});

test('wavelet requires n >= 2^levels — n=8 / levels=4 fails (need 16)', () => {
    expect(validateInputs('wavelet', new Array(8).fill(0), { levels: 4 }))
        .toMatch(/16/);
});

test('wavelet accepts n = 2^levels (n=16 / levels=4)', () => {
    expect(validateInputs('wavelet', new Array(16).fill(0), { levels: 4 }))
        .toBeNull();
});

test('emd rejects max_imfs < 1', () => {
    expect(validateInputs('emd', new Array(8).fill(0), { max_imfs: 0, max_sift_iter: 50 }))
        .toMatch(/max_imfs/);
});

test('emd rejects max_sift_iter < 1', () => {
    expect(validateInputs('emd', new Array(8).fill(0), { max_imfs: 5, max_sift_iter: 0 }))
        .toMatch(/max_sift_iter/);
});

test('ssa rejects window < 2 or > 100', () => {
    expect(validateInputs('ssa', new Array(200).fill(0), { window: 1 }))
        .toMatch(/window/);
    expect(validateInputs('ssa', new Array(300).fill(0), { window: 101 }))
        .toMatch(/window/);
});

test('wavelet rejects levels < 1', () => {
    expect(validateInputs('wavelet', new Array(8).fill(0), { levels: 0 }))
        .toMatch(/levels/);
});

// ── defaultOpts ───────────────────────────────────────────────────

test('defaultOpts(emd) = {max_imfs: 5, max_sift_iter: 50}', () => {
    expect(defaultOpts('emd')).toEqual({ max_imfs: 5, max_sift_iter: 50 });
});

test('defaultOpts(wavelet) = {levels: 4}', () => {
    expect(defaultOpts('wavelet')).toEqual({ levels: 4 });
});

test('defaultOpts(ssa) = {window: 20}', () => {
    expect(defaultOpts('ssa')).toEqual({ window: 20 });
});

test('defaultOpts(unknown) = {}', () => {
    expect(defaultOpts('zonk')).toEqual({});
});

// ── reconstructionResidual ────────────────────────────────────────

test('reconstructionResidual returns null on null inputs', () => {
    expect(reconstructionResidual(null, [])).toBeNull();
    expect(reconstructionResidual([1, 2], null)).toBeNull();
    expect(reconstructionResidual([1, 2], [])).toBeNull();
});

test('reconstructionResidual: SSA-style perfect decomposition → max-abs-err = 0', () => {
    // trend + noise = series exactly; max abs err = 0
    const series = [1, 2, 3, 4];
    const components = [
        { data: [0.5, 1.0, 1.5, 2.0] },
        { data: [0.5, 1.0, 1.5, 2.0] },
    ];
    expect(reconstructionResidual(series, components)).toBe(0);
});

test('reconstructionResidual: max-abs-err reflects worst single-index drift', () => {
    // series[2]=10 but components sum to 7; err=3
    const series = [1, 2, 10, 4];
    const components = [
        { data: [0.5, 1.0, 3.5, 2.0] },
        { data: [0.5, 1.0, 3.5, 2.0] },
    ];
    expect(reconstructionResidual(series, components)).toBe(3);
});

test('reconstructionResidual: skips components shorter than series, returns null when none qualify', () => {
    const series = [1, 2, 3, 4];
    const components = [{ data: [0.5, 1.0, 1.5] }];  // 3 != 4 → filtered out
    expect(reconstructionResidual(series, components)).toBeNull();
});

// ── toComponents normalization ────────────────────────────────────

test('emd toComponents: IMFs + residual → labeled list', () => {
    const res = { imfs: [[1, 2], [3, 4]], residual: [0.1, 0.2] };
    const out = METHODS.emd.toComponents(res);
    expect(out.length).toBe(3);
    expect(out[0].data).toEqual([1, 2]);
    expect(out[1].data).toEqual([3, 4]);
    expect(out[2].data).toEqual([0.1, 0.2]);
});

test('wavelet toComponents: details + approximation', () => {
    const res = { details: [[1, 2], [3, 4]], approximation: [10, 20] };
    const out = METHODS.wavelet.toComponents(res);
    expect(out.length).toBe(3);
    expect(out[2].data).toEqual([10, 20]);
});

test('ssa toComponents: trend + noise', () => {
    const res = { trend: [1, 2, 3], noise: [0.1, 0.2, 0.3] };
    const out = METHODS.ssa.toComponents(res);
    expect(out.length).toBe(2);
    expect(out[0].data).toEqual([1, 2, 3]);
    expect(out[1].data).toEqual([0.1, 0.2, 0.3]);
});

test('toComponents returns null when res is null', () => {
    expect(METHODS.emd.toComponents(null)).toBeNull();
    expect(METHODS.wavelet.toComponents(null)).toBeNull();
    expect(METHODS.ssa.toComponents(null)).toBeNull();
});

// ── parseSeries (delegated to paste_parser) ───────────────────────

test('parseSeries: csv blob → {value, errors:[]}', () => {
    const r = parseSeries('1, 2, 3');
    expect(r.errors).toEqual([]);
    expect(r.value).toEqual([1, 2, 3]);
});
