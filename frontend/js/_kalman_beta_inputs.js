// Pure helpers for the Kalman Dynamic Beta view.
//
// Kalman filter assumes β evolves as a random walk in time:
//   β_t = β_{t-1} + w_t,   w_t ~ N(0, Q)
//   r_asset_t = β_t · r_bench_t + v_t,   v_t ~ N(0, R)
//
// Lower Q → β trace is smoother (assumes slow drift).
// Higher Q → β adapts faster (assumes regime changes).
// Lower R → trust observations more (β tracks noise).
// Higher R → trust prior more (β changes reluctantly).

import { parseFloatBlob } from './_paste_parser.js';
import { t } from './i18n.js';

/** Parse the textarea — same delegation pattern as the other views. */
export function parseSeries(text) {
    return parseFloatBlob(text);
}

/** Validate the two-series + hyperparameter inputs. */
export function validateInputs(asset, bench, params) {
    if (!Array.isArray(asset) || asset.length < 10) return t('view.kalman_beta.validate.asset_min');
    if (!Array.isArray(bench) || bench.length < 10) return t('view.kalman_beta.validate.bench_min');
    if (asset.length !== bench.length) {
        return t('view.kalman_beta.validate.length_mismatch', { a: asset.length, b: bench.length });
    }
    if (asset.some(x => !Number.isFinite(x))) return t('view.kalman_beta.validate.asset_finite');
    if (bench.some(x => !Number.isFinite(x))) return t('view.kalman_beta.validate.bench_finite');
    if (!Number.isFinite(params.process_noise_q) || params.process_noise_q < 0) {
        return t('view.kalman_beta.validate.q');
    }
    if (!Number.isFinite(params.obs_noise_r) || params.obs_noise_r <= 0) {
        return t('view.kalman_beta.validate.r');
    }
    if (!Number.isFinite(params.beta0)) return t('view.kalman_beta.validate.beta0');
    if (!Number.isFinite(params.p0) || params.p0 <= 0) return t('view.kalman_beta.validate.p0');
    return null;
}

/** Build backend payload. */
export function buildBody(asset, bench, params) {
    return {
        asset, bench,
        process_noise_q: params.process_noise_q,
        obs_noise_r:     params.obs_noise_r,
        beta0:           params.beta0,
        p0:              params.p0,
    };
}

/** Summary stats of the β trace. Returns null for empty / all-null. */
export function summarizeBetaTrace(betas) {
    if (!Array.isArray(betas)) return null;
    const finite = betas.filter(b => Number.isFinite(b));
    if (finite.length === 0) return null;
    const n = finite.length;
    let sum = 0, min = Infinity, max = -Infinity;
    for (const b of finite) {
        sum += b;
        if (b < min) min = b;
        if (b > max) max = b;
    }
    const mean = sum / n;
    // Stdev (population).
    let sse = 0;
    for (const b of finite) sse += (b - mean) ** 2;
    const stdev = Math.sqrt(sse / n);
    // Last finite β.
    let latest = NaN;
    for (let i = betas.length - 1; i >= 0; i--) {
        if (Number.isFinite(betas[i])) { latest = betas[i]; break; }
    }
    // Drift = latest - first.
    let first = NaN;
    for (let i = 0; i < betas.length; i++) {
        if (Number.isFinite(betas[i])) { first = betas[i]; break; }
    }
    const drift = Number.isFinite(latest) && Number.isFinite(first) ? latest - first : NaN;
    return { mean, stdev, min, max, latest, first, drift, count: n };
}

/** Round to a sensible number of decimals for display. */
export function fmtBeta(x, digits = 4) {
    if (!Number.isFinite(x)) return '—';
    return x.toFixed(digits);
}
