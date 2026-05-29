// Pure helpers for the Signal Decomposition view.
//
// Three methods share one workflow:
//   * EMD     — adaptive sifting → IMFs + residual.
//   * Wavelet — Haar DWT → details (per level) + final approximation.
//   * SSA     — eigendecomposition → trend + noise.
//
// Each method's response is normalized into a uniform component list
// `[{ label, color, data }]` so the view can render them as stacked
// subplots without per-method branching.

import { parseFloatBlob } from './_paste_parser.js';

const PALETTE = [
    '#00e5ff', '#ff9f1a', '#a06bff', '#39ff14',
    '#ff3860', '#ffd84a', '#1ad1ff', '#ff66d9',
];

/** Method registry — id → { label, endpoint, fields[], buildBody,
 *  validateOpts, toComponents }. */
export const METHODS = {
    emd: {
        label: 'Empirical Mode Decomposition',
        endpoint: 'anlyEmpiricalModeDecomposition',
        fields: [
            { key: 'max_imfs',      label: 'Max IMFs',      default: 5,  min: 1,  max: 12, integer: true },
            { key: 'max_sift_iter', label: 'Max sift iter', default: 50, min: 1,  max: 500, integer: true },
        ],
        buildBody: (series, opts) => ({
            series,
            max_imfs: opts.max_imfs,
            max_sift_iter: opts.max_sift_iter,
        }),
        validateOpts: (opts) => {
            if (!Number.isInteger(opts.max_imfs) || opts.max_imfs < 1) {
                return 'max_imfs must be a positive integer';
            }
            if (!Number.isInteger(opts.max_sift_iter) || opts.max_sift_iter < 1) {
                return 'max_sift_iter must be a positive integer';
            }
            return null;
        },
        toComponents: (res) => {
            if (!res) return null;
            const out = [];
            (res.imfs || []).forEach((y, i) => {
                out.push({ label: `IMF ${i + 1}`, color: PALETTE[i % PALETTE.length], data: y });
            });
            if (Array.isArray(res.residual)) {
                out.push({ label: 'Residual', color: '#888', data: res.residual });
            }
            return out;
        },
    },

    wavelet: {
        label: 'Wavelet (Haar)',
        endpoint: 'anlyWaveletDecompositionHaar',
        fields: [
            { key: 'levels', label: 'Levels', default: 4, min: 1, max: 12, integer: true },
        ],
        buildBody: (series, opts) => ({ series, levels: opts.levels }),
        validateOpts: (opts) => {
            if (!Number.isInteger(opts.levels) || opts.levels < 1) {
                return 'levels must be a positive integer';
            }
            return null;
        },
        toComponents: (res) => {
            if (!res) return null;
            const out = [];
            // Details are ordered fine-to-coarse; reverse so the first
            // chart shows the lowest-frequency detail (cleanest visual).
            const details = res.details || [];
            details.forEach((y, i) => {
                out.push({ label: `Detail ${i + 1}`, color: PALETTE[i % PALETTE.length], data: y });
            });
            if (Array.isArray(res.approximation)) {
                out.push({ label: 'Approximation', color: '#888', data: res.approximation });
            }
            return out;
        },
    },

    ssa: {
        label: 'Singular Spectrum Analysis',
        endpoint: 'anlySingularSpectrumAnalysis',
        fields: [
            { key: 'window', label: 'Window L', default: 20, min: 2, max: 100, integer: true },
        ],
        buildBody: (series, opts) => ({ series, window: opts.window }),
        validateOpts: (opts) => {
            if (!Number.isInteger(opts.window) || opts.window < 2 || opts.window > 100) {
                return 'window must be an integer in [2, 100]';
            }
            return null;
        },
        toComponents: (res) => {
            if (!res) return null;
            return [
                { label: 'Trend', color: '#00e5ff', data: res.trend },
                { label: 'Noise', color: '#ff9f1a', data: res.noise },
            ];
        },
    },
};

/** Parse the textarea — delegate to the shared paste parser. */
export function parseSeries(text) {
    return parseFloatBlob(text);
}

/** Pre-flight validation against series length + method-specific opts. */
export function validateInputs(methodId, series, opts) {
    const method = METHODS[methodId];
    if (!method) return `unknown method "${methodId}"`;
    if (!Array.isArray(series) || series.length < 8) {
        return 'need at least 8 series values to decompose';
    }
    if (series.some(x => !Number.isFinite(x))) return 'series contains non-finite values';
    // SSA needs n >= 2·window. EMD needs >= 8. Wavelet needs >= 2^levels.
    if (methodId === 'ssa' && series.length < 2 * opts.window) {
        return `SSA needs series length ≥ 2 · window (got ${series.length} vs window=${opts.window})`;
    }
    if (methodId === 'wavelet' && series.length < (1 << opts.levels)) {
        return `Wavelet needs series length ≥ 2^${opts.levels} (= ${1 << opts.levels}) — got ${series.length}`;
    }
    return method.validateOpts(opts);
}

/** Fresh default options for a method (caller-mutable). */
export function defaultOpts(methodId) {
    const method = METHODS[methodId];
    if (!method) return {};
    const out = {};
    for (const f of method.fields) out[f.key] = f.default;
    return out;
}

/** Sum of every component's value at index i — useful for the
 *  "reconstruction quality" sanity card (EMD and Wavelet must
 *  approximately recover the original; SSA's trend+noise = original
 *  by construction). Returns null if components have mismatched
 *  lengths. */
export function reconstructionResidual(series, components) {
    if (!Array.isArray(series) || !Array.isArray(components) || components.length === 0) {
        return null;
    }
    // Use the FULL-length components only (some wavelet details are shorter).
    const fullLen = components.filter(c => Array.isArray(c.data) && c.data.length === series.length);
    if (fullLen.length === 0) return null;
    let maxAbsErr = 0;
    for (let i = 0; i < series.length; i++) {
        let sum = 0;
        for (const c of fullLen) sum += c.data[i];
        const err = Math.abs(series[i] - sum);
        if (err > maxAbsErr) maxAbsErr = err;
    }
    return maxAbsErr;
}
