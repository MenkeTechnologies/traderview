// Pure helpers for the Series Smoother view.
//
// Each smoother takes a different payload shape, so build payloads via
// per-smoother helpers and post-process Theil-Sen (which returns slope
// + intercept, not a smoothed vector) into an evaluated y[] for chart
// overlay.

import { parseFloatBlob } from './_paste_parser.js';
import { t } from './i18n.js';

/** Parse a pasted series into a numeric array. */
export function parseSeries(text) {
    return parseFloatBlob(text);
}

/** Validation gate. Min length is the largest of the smoothers we run:
 *  Kalman-RTS needs at least 1, LOWESS needs at least 5, polynomial
 *  needs degree+2. Pick 10 as the universal floor — anything smaller
 *  isn't worth smoothing visually anyway. */
export function validateSeries(series, minLen = 10) {
    if (!Array.isArray(series) || series.length < minLen) {
        return t('view.series_smoother.validate.series_min', { n: minLen });
    }
    if (series.some(x => !Number.isFinite(x))) return t('view.series_smoother.validate.series_finite');
    return null;
}

/** Generate the implicit x-axis (0..n-1) used by the LOWESS, Theil-Sen,
 *  and polynomial endpoints. */
export function indexAxis(n) {
    return Array.from({ length: n }, (_, i) => i);
}

/** Build payloads for each backend. Returns an object keyed by smoother
 *  id with a `body` ready to JSON.stringify. */
export function buildSmootherPayloads(series, opts) {
    const xs = indexAxis(series.length);
    return {
        lowess: { x: xs, y: series, frac: opts.lowess_frac, robustness_iter: opts.lowess_robust },
        kalman_rts: {
            observations: series,
            process_noise_q: opts.kalman_q,
            obs_noise_r: opts.kalman_r,
            x0: series[0],
            p0: 1.0,
        },
        theil_sen: { x: xs, y: series },
        polynomial: { x: xs, y: series, degree: opts.poly_degree },
    };
}

/** Evaluate the Theil-Sen line at every x in `xs`. The backend returns
 *  { slope, intercept, n_pairs } — we generate the fitted y[] locally
 *  for chart overlay. */
export function theilSenFittedY(xs, slope, intercept) {
    return xs.map(x => slope * x + intercept);
}

/** Default smoother options. */
export function defaultOptions() {
    return {
        lowess_frac: 0.3,
        lowess_robust: 0,
        kalman_q: 1e-3,
        kalman_r: 1e-1,
        poly_degree: 3,
    };
}

/** Validate the option block. Returns null on success or an error
 *  string. Each field has a domain the backend enforces; we surface
 *  friendly errors before the round trip. */
export function validateOptions(opts) {
    if (!Number.isFinite(opts.lowess_frac) || opts.lowess_frac <= 0 || opts.lowess_frac > 1) {
        return t('view.series_smoother.validate.lowess_frac');
    }
    if (!Number.isInteger(opts.lowess_robust) || opts.lowess_robust < 0) {
        return t('view.series_smoother.validate.lowess_robust');
    }
    if (!Number.isFinite(opts.kalman_q) || opts.kalman_q < 0) {
        return t('view.series_smoother.validate.kalman_q');
    }
    if (!Number.isFinite(opts.kalman_r) || opts.kalman_r <= 0) {
        return t('view.series_smoother.validate.kalman_r');
    }
    if (!Number.isInteger(opts.poly_degree) || opts.poly_degree < 1) {
        return t('view.series_smoother.validate.poly_degree');
    }
    return null;
}
