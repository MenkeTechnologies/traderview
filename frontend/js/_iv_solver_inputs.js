// Pure helpers for the Implied Volatility Solver view.
//
// Reuses the closed-form Black-Scholes from the American Option helper
// so the "BS price at solved σ" sanity reference and the sensitivity
// chart both have a single source of truth.

import { blackScholesEuropean } from './_american_option_inputs.js';
import { t } from './i18n.js';

/** Build the JSON body for /options/calc/iv-solver. */
export function buildBody(p) {
    return {
        market_price:    p.market_price,
        spot:            p.spot,
        strike:          p.strike,
        time_to_expiry:  p.time_to_expiry,
        risk_free:       p.risk_free,
        dividend_yield:  p.dividend_yield,
        kind:            p.kind,
    };
}

/** Validate inputs. The IV-solver fails when the market price is
 *  outside the BS no-arb bounds (call: max(S·e^{-qT} − K·e^{-rT}, 0)
 *  ≤ C ≤ S·e^{-qT}; put: max(K·e^{-rT} − S·e^{-qT}, 0) ≤ P ≤
 *  K·e^{-rT}). We pre-check those bounds so the user sees a friendly
 *  error before the round-trip. */
export function validateParams(p) {
    if (p.kind !== 'call' && p.kind !== 'put') return t('view.iv_solver.validate.kind');
    if (!Number.isFinite(p.spot) || p.spot <= 0)   return t('view.iv_solver.validate.spot');
    if (!Number.isFinite(p.strike) || p.strike <= 0) return t('view.iv_solver.validate.strike');
    if (!Number.isFinite(p.time_to_expiry) || p.time_to_expiry <= 0) {
        return t('view.iv_solver.validate.time');
    }
    if (!Number.isFinite(p.risk_free))      return t('view.iv_solver.validate.risk_free');
    if (!Number.isFinite(p.dividend_yield) || p.dividend_yield < 0) {
        return t('view.iv_solver.validate.div_yield');
    }
    if (!Number.isFinite(p.market_price) || p.market_price < 0) {
        return t('view.iv_solver.validate.market_price');
    }
    const bounds = arbBounds(p);
    if (p.market_price < bounds.lower - 1e-9 || p.market_price > bounds.upper + 1e-9) {
        return t('view.iv_solver.validate.no_arb_band', { price: p.market_price.toFixed(4), lower: bounds.lower.toFixed(4), upper: bounds.upper.toFixed(4) });
    }
    return null;
}

/** No-arb bounds on the option price for a given strike, spot, T, r, q.
 *  Used both as a validation check and to display the band to the user. */
export function arbBounds(p) {
    const discR = Math.exp(-p.risk_free * p.time_to_expiry);
    const discQ = Math.exp(-p.dividend_yield * p.time_to_expiry);
    const forward = p.spot * discQ;
    if (p.kind === 'call') {
        return {
            lower: Math.max(forward - p.strike * discR, 0),
            upper: forward,
        };
    }
    return {
        lower: Math.max(p.strike * discR - forward, 0),
        upper: p.strike * discR,
    };
}

/** Generate a BS price vs σ sweep over [0.005, max_sigma]. Caller plots
 *  this curve with a horizontal line at market_price + a vertical
 *  marker at the solved σ — visually verifies the solver hit the
 *  intersection. */
export function priceVsSigmaSweep(p, maxSigma = 2.0, points = 121) {
    if (!(maxSigma > 0)) return { xs: [], ys: [] };
    const xs = new Array(points);
    const ys = new Array(points);
    const lo = 0.005;
    for (let i = 0; i < points; i++) {
        const sigma = lo + (maxSigma - lo) * i / (points - 1);
        xs[i] = sigma;
        ys[i] = blackScholesEuropean(
            p.kind, p.spot, p.strike, p.time_to_expiry,
            p.risk_free, p.dividend_yield, sigma,
        );
    }
    return { xs, ys };
}

/** Format a vol as "x.xx%" for display. */
export function fmtVolPct(v, digits = 2) {
    if (!Number.isFinite(v)) return '—';
    return `${(v * 100).toFixed(digits)}%`;
}

/** Format a price with 4 decimals. */
export function fmtPrice(p, digits = 4) {
    if (!Number.isFinite(p)) return '—';
    return p.toFixed(digits);
}
