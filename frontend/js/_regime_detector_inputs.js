// Pure helpers for the Regime Detector view.
//
// Parser delegates to the shared paste-parser. Validation enforces the
// backend's Markov-switching minimum (≥ 30 observations and non-zero
// variance). Post-processing canonicalizes the response into UI-ready
// derived stats — annualized vol per state, mean-of-state-1 prob, count
// of bars where state-1 prob exceeded a threshold.

import { parseFloatBlob } from './_paste_parser.js';
import { t } from './i18n.js';

/** Parse a pasted return series. */
export function parseReturns(text) {
    return parseFloatBlob(text);
}

/** Validate inputs against the backend's constraints. */
export function validateReturns(returns) {
    if (!Array.isArray(returns) || returns.length < 30) {
        return t('view.regime_detector.validate.need_30');
    }
    if (returns.some(x => !Number.isFinite(x))) return t('view.regime_detector.validate.non_finite');
    // Variance > 0 — backend bails on a flat series.
    const mean = returns.reduce((a, b) => a + b, 0) / returns.length;
    const sse = returns.reduce((a, b) => a + (b - mean) ** 2, 0);
    if (sse < 1e-18) return t('view.regime_detector.validate.flat');
    return null;
}

/** Annualize a per-bar stdev given `bars_per_year` (typically 252 for
 *  daily, 78 for 5-min, etc.). Returns σ_annual = σ_bar · √bpy. */
export function annualizeStdev(sigma, barsPerYear) {
    if (!Number.isFinite(sigma) || !Number.isFinite(barsPerYear) || barsPerYear <= 0) {
        return NaN;
    }
    return sigma * Math.sqrt(barsPerYear);
}

/** Annualize a per-bar mean given `bars_per_year`. μ_annual = μ_bar · bpy. */
export function annualizeMean(mu, barsPerYear) {
    if (!Number.isFinite(mu) || !Number.isFinite(barsPerYear) || barsPerYear <= 0) {
        return NaN;
    }
    return mu * barsPerYear;
}

/** Stationary distribution of a 2-state Markov chain from its transition
 *  probabilities. Returns { p_state0, p_state1 } — the long-run fraction
 *  of time spent in each state. For [p00, 1-p00; 1-p11, p11]:
 *      π_1 = (1 - p00) / (2 - p00 - p11)
 *      π_0 = 1 - π_1
 *  Returns { p_state0: 0.5, p_state1: 0.5 } if the system is degenerate. */
export function stationaryDistribution(p00, p11) {
    if (!Number.isFinite(p00) || !Number.isFinite(p11)) {
        return { p_state0: 0.5, p_state1: 0.5 };
    }
    const denom = 2 - p00 - p11;
    if (Math.abs(denom) < 1e-12) return { p_state0: 0.5, p_state1: 0.5 };
    const p1 = (1 - p00) / denom;
    const p0 = 1 - p1;
    return { p_state0: clamp(p0, 0, 1), p_state1: clamp(p1, 0, 1) };
}

function clamp(x, lo, hi) { return Math.min(Math.max(x, lo), hi); }

/** Expected dwell time in a state (Markov geometric mean):
 *  E[dwell | state k] = 1 / (1 - p_kk). Returns Infinity for absorbing
 *  states (p_kk = 1) and 1 for instantaneous-exit (p_kk = 0). */
export function expectedDwell(pkk) {
    if (!Number.isFinite(pkk) || pkk < 0 || pkk > 1) return NaN;
    if (pkk >= 1 - 1e-12) return Infinity;
    return 1 / (1 - pkk);
}

/** Fraction of bars whose state-1 probability exceeded `threshold`. */
export function highVolBarFraction(probState1, threshold = 0.5) {
    if (!Array.isArray(probState1) || probState1.length === 0) return 0;
    let n = 0;
    for (const p of probState1) if (Number.isFinite(p) && p > threshold) n++;
    return n / probState1.length;
}
