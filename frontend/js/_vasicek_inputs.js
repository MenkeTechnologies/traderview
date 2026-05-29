// Pure helpers for the Vasicek Short-Rate Simulator view.
//
// Vasicek SDE: dr = a·(b − r)·dt + σ·dW
//   r0    initial short rate
//   a     mean-reversion speed (> 0)
//   b     long-run mean rate
//   σ     instantaneous volatility (≥ 0)
//   dt    time step (years)
//   steps number of time steps per path
//   paths number of Monte Carlo paths
//
// Closed-form properties (for the summary cards):
//   half_life       = ln(2) / a            (years)
//   long_run_stdev  = σ / √(2a)            (terminal stdev as t → ∞)
//   long_run_var    = σ² / (2a)
//
// The view also draws a normal-approximation density of the terminal-
// rate distribution using (mean, stdev) from the backend response. For
// large t the simulated distribution converges to the long-run normal,
// so the approximation is asymptotically exact.

import { t } from './i18n.js';

/** Build the JSON body for /analytics/vasicek-short-rate-simulator. */
export function buildBody(p) {
    return {
        r0: p.r0,
        a:  p.a,
        b:  p.b,
        sigma: p.sigma,
        dt:    p.dt,
        steps: p.steps,
        paths: p.paths,
        seed:  p.seed,
    };
}

/** Validate inputs. Returns null on success or a friendly error string
 *  with the offending field. */
export function validateParams(p) {
    if (!Number.isFinite(p.r0)) return t('view.vasicek.validate.r0');
    if (!Number.isFinite(p.a) || p.a <= 0) return t('view.vasicek.validate.a');
    if (!Number.isFinite(p.b)) return t('view.vasicek.validate.b');
    if (!Number.isFinite(p.sigma) || p.sigma < 0) return t('view.vasicek.validate.sigma');
    if (!Number.isFinite(p.dt) || p.dt <= 0) return t('view.vasicek.validate.dt');
    if (!Number.isInteger(p.steps) || p.steps < 1) return t('view.vasicek.validate.steps');
    if (!Number.isInteger(p.paths) || p.paths < 10) return t('view.vasicek.validate.paths');
    if (!Number.isInteger(p.seed) || p.seed < 0) return t('view.vasicek.validate.seed');
    return null;
}

/** Mean-reversion half-life in years. ln(2)/a. */
export function halfLifeYears(a) {
    if (!Number.isFinite(a) || a <= 0) return null;
    return Math.LN2 / a;
}

/** Long-run terminal stdev — σ / √(2a). */
export function longRunStdev(a, sigma) {
    if (!Number.isFinite(a) || a <= 0) return null;
    if (!Number.isFinite(sigma) || sigma < 0) return null;
    return sigma / Math.sqrt(2 * a);
}

/** Total horizon in years = steps · dt. */
export function horizonYears(steps, dt) {
    if (!Number.isInteger(steps) || steps < 1) return null;
    if (!Number.isFinite(dt) || dt <= 0) return null;
    return steps * dt;
}

/** Normal-approximation density curve over [mean − 4σ, mean + 4σ]. */
export function normalDensityCurve(mean, stdev, points = 121) {
    if (!Number.isFinite(mean) || !Number.isFinite(stdev) || stdev <= 0) {
        return { xs: [], ys: [] };
    }
    const xs = [];
    const ys = [];
    const span = 8 * stdev;
    const start = mean - 4 * stdev;
    const step = span / (points - 1);
    const norm = 1 / (stdev * Math.sqrt(2 * Math.PI));
    for (let i = 0; i < points; i++) {
        const x = start + i * step;
        const z = (x - mean) / stdev;
        xs.push(x);
        ys.push(norm * Math.exp(-0.5 * z * z));
    }
    return { xs, ys };
}

/** Format a rate as a 4-decimal percent. */
export function fmtRatePct(r, digits = 4) {
    if (!Number.isFinite(r)) return '—';
    return `${(r * 100).toFixed(digits)}%`;
}

/** Format a years-duration concisely (days vs years depending on size). */
export function fmtYears(y) {
    if (!Number.isFinite(y)) return '—';
    if (y < 1.0) return `${(y * 365).toFixed(1)} days`;
    return `${y.toFixed(2)} years`;
}
