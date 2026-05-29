// Second-Order Greeks — pure helpers shared by view + vitest.
//
// Mathematical references mirror the backend `second_order_greeks` crate
// (Abramowitz & Stegun 26.2.17 erf, BS continuous-compounding) so the
// client-side grid and the backend single-point card match to ~7 dp.
//
//   vanna = ∂Δ/∂σ
//   charm = ∂Δ/∂t   (sign convention: signed)
//   vomma = ∂vega/∂σ  (a.k.a. volga)
//   veta  = ∂vega/∂t

import { t } from './i18n.js';

export const METRICS = ['vanna', 'charm', 'vomma', 'veta'];

export function validateParams(p) {
    if (p.kind !== 'call' && p.kind !== 'put') return t('view.second_order_greeks.validate.kind');
    if (!Number.isFinite(p.strike) || p.strike <= 0) return t('view.second_order_greeks.validate.strike');
    if (!Number.isFinite(p.time_to_expiry) || p.time_to_expiry <= 0) return t('view.second_order_greeks.validate.tte');
    if (!Number.isFinite(p.risk_free)) return t('view.second_order_greeks.validate.risk_free');
    if (!Number.isFinite(p.dividend_yield) || p.dividend_yield < 0) return t('view.second_order_greeks.validate.div_yield');
    if (!Number.isFinite(p.sigma) || p.sigma <= 0) return t('view.second_order_greeks.validate.sigma');
    if (!Number.isFinite(p.spot_grid_low) || p.spot_grid_low <= 0) return t('view.second_order_greeks.validate.grid_low');
    if (!Number.isFinite(p.spot_grid_high) || p.spot_grid_high <= p.spot_grid_low)
        return t('view.second_order_greeks.validate.grid_high');
    if (!Number.isInteger(p.n_points) || p.n_points < 5 || p.n_points > 501)
        return t('view.second_order_greeks.validate.n_points');
    return null;
}

export function buildBody(p) {
    // Single-point body matches backend SecondOrderGreeksBody.
    return {
        spot: p.spot, strike: p.strike, time_to_expiry: p.time_to_expiry,
        risk_free: p.risk_free, dividend_yield: p.dividend_yield, sigma: p.sigma,
        kind: p.kind,
    };
}

export function defaultSpotGrid(strike) {
    if (!Number.isFinite(strike) || strike <= 0) return { low: 50, high: 150 };
    return { low: strike * 0.5, high: strike * 1.5 };
}

export function linspace(lo, hi, n) {
    if (n < 2) return [lo];
    const step = (hi - lo) / (n - 1);
    const xs = new Array(n);
    for (let i = 0; i < n; i++) xs[i] = lo + i * step;
    return xs;
}

// Abramowitz & Stegun 26.2.17 — mirrors backend exactly.
function normCdf(x) {
    const a1 =  0.254829592, a2 = -0.284496736, a3 =  1.421413741,
          a4 = -1.453152027, a5 =  1.061405429, p  =  0.3275911;
    const sign = x < 0 ? -1 : 1;
    const ax = Math.abs(x) / Math.SQRT2;
    const t = 1.0 / (1.0 + p * ax);
    const y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * Math.exp(-ax * ax);
    return 0.5 * (1.0 + sign * y);
}

function normPdf(x) {
    return Math.exp(-0.5 * x * x) / Math.sqrt(2.0 * Math.PI);
}

// Computes all four second-order greeks at one (s, k, t, r, q, σ, kind).
// Returns null if any input is invalid or any output non-finite — mirrors
// backend's `Option<Greeks2>` contract.
export function computePoint(s, k, t, r, q, sigma, kind) {
    if (![s, k, t, r, q, sigma].every(Number.isFinite)) return null;
    if (s <= 0 || k <= 0 || t <= 0 || sigma <= 0) return null;
    const sqrtT = Math.sqrt(t);
    const d1 = (Math.log(s / k) + (r - q + 0.5 * sigma * sigma) * t) / (sigma * sqrtT);
    const d2 = d1 - sigma * sqrtT;
    const pdfD1 = normPdf(d1);
    const expQT = Math.exp(-q * t);
    const vanna = -expQT * pdfD1 * d2 / sigma;
    const charmCommon = expQT * pdfD1 * (2.0 * (r - q) * t - d2 * sigma * sqrtT)
        / (2.0 * t * sigma * sqrtT);
    const nd1 = normCdf(d1);
    const charm = kind === 'call'
        ? q * expQT * nd1 - charmCommon
        : -q * expQT * (1.0 - nd1) - charmCommon;
    const vega = s * expQT * sqrtT * pdfD1;
    const vomma = vega * d1 * d2 / sigma;
    const veta = -s * expQT * pdfD1
        * (q + (r - q) * d1 / (sigma * sqrtT) - (1.0 + d1 * d2) / (2.0 * t));
    const g = { vanna, charm, vomma, veta };
    if (!METRICS.every(m => Number.isFinite(g[m]))) return null;
    return g;
}

// Walks the spot grid and returns six parallel arrays: spots + one per
// metric. Non-finite outputs at a given spot become null (uPlot draws a
// gap there).
export function computeGrid(params) {
    const xs = linspace(params.spot_grid_low, params.spot_grid_high, params.n_points);
    const out = { spots: xs };
    for (const m of METRICS) out[m] = new Array(xs.length).fill(null);
    for (let i = 0; i < xs.length; i++) {
        const g = computePoint(
            xs[i], params.strike, params.time_to_expiry,
            params.risk_free, params.dividend_yield, params.sigma, params.kind);
        if (!g) continue;
        for (const m of METRICS) out[m][i] = g[m];
    }
    return out;
}

// Picks the grid index whose spot is closest to `strike` — used to anchor
// the ATM marker on each mini-chart.
export function nearestAtmIndex(spots, strike) {
    if (!Array.isArray(spots) || spots.length === 0) return -1;
    let bestI = 0, bestD = Math.abs(spots[0] - strike);
    for (let i = 1; i < spots.length; i++) {
        const d = Math.abs(spots[i] - strike);
        if (d < bestD) { bestI = i; bestD = d; }
    }
    return bestI;
}

export function fmtN(v, digits = 6) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(digits);
}
