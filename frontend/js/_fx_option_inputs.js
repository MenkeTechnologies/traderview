// Pure helpers for the FX Option Calculator view.
//
// Mirrors the American Option helper structure: payload builder + input
// validator + local closed-form Garman-Kohlhagen so the sensitivity
// chart renders without N backend calls.
//
// Garman-Kohlhagen extends Black-Scholes for FX by treating the foreign
// interest rate as a continuous dividend yield. With:
//   r_d = domestic short rate
//   r_f = foreign short rate
// the call/put formulas are:
//   d1 = (ln(S/K) + (r_d − r_f + σ²/2) T) / (σ √T)
//   d2 = d1 − σ √T
//   call = S · e^(−r_f T) · N(d1) − K · e^(−r_d T) · N(d2)
//   put  = K · e^(−r_d T) · N(−d2) − S · e^(−r_f T) · N(−d1)

import { normCdf } from './_american_option_inputs.js';
import { t } from './i18n.js';

/** Build the JSON body for /analytics/garman-kohlhagen-fx-option. */
export function buildGkBody(p) {
    return {
        kind: p.kind,
        spot: p.spot,
        strike: p.strike,
        t_years: p.t_years,
        rate_dom: p.rate_dom,
        rate_for: p.rate_for,
        sigma: p.sigma,
    };
}

/** Validate the parameter block. Returns null on success, an error
 *  string with the offending field otherwise. */
export function validateGkParams(p) {
    if (p.kind !== 'call' && p.kind !== 'put') return t('view.fx_option.validate.kind');
    const positive = ['spot', 'strike', 't_years'];
    for (const k of positive) {
        if (!Number.isFinite(p[k]) || p[k] <= 0) return t('view.fx_option.validate.field_positive', { k });
    }
    for (const k of ['rate_dom', 'rate_for']) {
        if (!Number.isFinite(p[k])) return t('view.fx_option.validate.field_finite', { k });
    }
    if (!Number.isFinite(p.sigma) || p.sigma < 0) return t('view.fx_option.validate.sigma');
    return null;
}

/** Garman-Kohlhagen European FX option price. Collapses to intrinsic at
 *  t=0 or σ=0. */
export function garmanKohlhagenPrice(kind, spot, strike, t, rd, rf, sigma) {
    if (t <= 0 || sigma <= 0) {
        return kind === 'call'
            ? Math.max(spot - strike, 0)
            : Math.max(strike - spot, 0);
    }
    const sqrtT = Math.sqrt(t);
    const st = sigma * sqrtT;
    const d1 = (Math.log(spot / strike) + (rd - rf + 0.5 * sigma * sigma) * t) / st;
    const d2 = d1 - st;
    const discD = Math.exp(-rd * t);
    const discF = Math.exp(-rf * t);
    if (kind === 'call') {
        return spot * discF * normCdf(d1) - strike * discD * normCdf(d2);
    } else {
        return strike * discD * normCdf(-d2) - spot * discF * normCdf(-d1);
    }
}

/** Common dollar-money formatting for FX rates (4 decimals by default
 *  — typical for EURUSD/USDJPY-scale rates). */
export function fmtRate(x, digits = 4) {
    if (!Number.isFinite(x)) return '—';
    return x.toFixed(digits);
}

/** Greek formatting: keeps 6 decimals because gamma and vega often
 *  produce tiny magnitudes for non-leveraged FX positions. */
export function fmtGreek(x, digits = 6) {
    if (!Number.isFinite(x)) return '—';
    return x.toFixed(digits);
}
