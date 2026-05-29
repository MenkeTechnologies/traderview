// Pure helpers for the Cov Matrix Denoiser (Marchenko-Pastur) view.
//
// Reuses the portfolio-allocator matrix parser + validator. Adds two
// MP-specific helpers:
//   * marchenkoPasturBulk(sigma, q) — Bulk edges [λ_min, λ_max] given
//     the assumed white-noise variance σ² and q = N/T. Used to draw
//     the bulk-shaded band on the eigenvalue chart locally without a
//     round-trip per re-render.
//   * formatCovDelta — pretty stats on what changed cell-by-cell.

import { parseMatrix } from './_portfolio_allocator_inputs.js';
import { validateCovariance } from './_portfolio_allocator_inputs.js';

/** Parse the cov matrix textarea. Same shape as portfolio-allocator. */
export function parseCovariance(text) {
    return parseMatrix(text);
}

/** Validate before sending. Adds an MP-specific check: num_observations
 *  must be ≥ N (matrix size) — that's the well-defined regime of the
 *  bulk formula. */
export function validateInputs(cov, numObservations) {
    const baseErr = validateCovariance(cov);
    if (baseErr) return baseErr;
    if (!Number.isInteger(numObservations) || numObservations < 1) {
        return 'num observations T must be a positive integer';
    }
    if (numObservations < cov.length) {
        return `T (${numObservations}) must be ≥ N (${cov.length}) — q = N/T must be ≤ 1`;
    }
    return null;
}

/** Build backend payload. */
export function buildBody(cov, numObservations) {
    return { covariance: cov, num_observations: numObservations };
}

/** Marchenko-Pastur bulk edges for an N×T sample-cov from i.i.d. zero-
 *  mean noise with variance σ². q = N/T < 1.
 *    λ_min = σ² · (1 − √q)²
 *    λ_max = σ² · (1 + √q)²
 *  Used to visualize the noise-bulk band on the eigenvalue chart even
 *  before the backend responds. */
export function marchenkoPasturBulk(sigmaSq, q) {
    if (!Number.isFinite(sigmaSq) || sigmaSq <= 0) return null;
    if (!Number.isFinite(q) || q <= 0 || q > 1) return null;
    const root = Math.sqrt(q);
    return {
        lambda_min: sigmaSq * (1 - root) * (1 - root),
        lambda_max: sigmaSq * (1 + root) * (1 + root),
    };
}

/** Maximum absolute change between two square matrices of the same
 *  size. Returns null if shapes mismatch. */
export function maxAbsDelta(a, b) {
    if (!Array.isArray(a) || !Array.isArray(b) || a.length !== b.length) return null;
    let m = 0;
    for (let i = 0; i < a.length; i++) {
        if (!Array.isArray(a[i]) || !Array.isArray(b[i]) || a[i].length !== b[i].length) return null;
        for (let j = 0; j < a[i].length; j++) {
            const d = Math.abs(a[i][j] - b[i][j]);
            if (d > m) m = d;
        }
    }
    return m;
}

/** Frobenius-norm relative delta: ||A - B||_F / ||A||_F. Useful as a
 *  scalar "how much did the matrix change" summary. */
export function frobeniusRelDelta(orig, clean) {
    if (!Array.isArray(orig) || !Array.isArray(clean) || orig.length !== clean.length) return null;
    let sseDelta = 0;
    let sseOrig = 0;
    for (let i = 0; i < orig.length; i++) {
        for (let j = 0; j < orig[i].length; j++) {
            const d = orig[i][j] - clean[i][j];
            sseDelta += d * d;
            sseOrig  += orig[i][j] * orig[i][j];
        }
    }
    if (sseOrig <= 0) return null;
    return Math.sqrt(sseDelta) / Math.sqrt(sseOrig);
}
