// Pure helpers for the American Option Pricer view.
//
// Two responsibilities:
//   1. Shape + validate the LSMC backend payload.
//   2. Compute a Black-Scholes European reference price LOCALLY so the
//      early-exercise premium (American − European) renders without a
//      second network round-trip. The BS formula is short and the
//      Abramowitz-Stegun 26.2.17 normal-CDF approximation matches what
//      the Rust modules already use (error < 7.5e-8).

import { t } from './i18n.js';

/** Build the JSON body for /analytics/american-option-lsmc. */
export function buildLsmcBody(p) {
    return {
        kind: p.kind,
        spot: p.spot,
        strike: p.strike,
        t_years: p.t_years,
        rate: p.rate,
        dividend: p.dividend,
        sigma: p.sigma,
        steps: p.steps,
        paths: p.paths,
        seed: p.seed,
    };
}

/** Validate the parameter block. Returns null on success or a friendly
 *  error string with the offending field. Mirrors the backend's compute
 *  rejections so the UI surfaces problems pre-round-trip. */
export function validateLsmcParams(p) {
    if (p.kind !== 'call' && p.kind !== 'put') return t('view.american_option.validate.kind');
    const positive = ['spot', 'strike', 't_years'];
    for (const k of positive) {
        if (!Number.isFinite(p[k]) || p[k] <= 0) return t('view.american_option.validate.field_positive', { k });
    }
    if (!Number.isFinite(p.rate)) return t('view.american_option.validate.rate');
    if (!Number.isFinite(p.dividend) || p.dividend < 0) return t('view.american_option.validate.dividend');
    if (!Number.isFinite(p.sigma) || p.sigma < 0) return t('view.american_option.validate.sigma');
    if (!Number.isInteger(p.steps) || p.steps < 2) return t('view.american_option.validate.steps');
    if (!Number.isInteger(p.paths) || p.paths < 10) return t('view.american_option.validate.paths');
    if (!Number.isInteger(p.seed) || p.seed < 0) return t('view.american_option.validate.seed');
    return null;
}

/** Black-Scholes European option price. Handles the t=0 and σ=0 limits
 *  (collapses to intrinsic). Returns the discounted option value, in
 *  the same units as `spot` and `strike`. */
export function blackScholesEuropean(kind, spot, strike, t, r, q, sigma) {
    if (t <= 0 || sigma <= 0) {
        const intrinsic = kind === 'call'
            ? Math.max(spot - strike, 0)
            : Math.max(strike - spot, 0);
        return intrinsic;
    }
    const sqrtT = Math.sqrt(t);
    const st = sigma * sqrtT;
    const d1 = (Math.log(spot / strike) + (r - q + 0.5 * sigma * sigma) * t) / st;
    const d2 = d1 - st;
    const discQ = Math.exp(-q * t);
    const discR = Math.exp(-r * t);
    if (kind === 'call') {
        return spot * discQ * normCdf(d1) - strike * discR * normCdf(d2);
    } else {
        return strike * discR * normCdf(-d2) - spot * discQ * normCdf(-d1);
    }
}

/** Abramowitz-Stegun 26.2.17 normal CDF. Error < 7.5e-8. Matches the
 *  approximation used in the Rust pricing modules so AB-tested values
 *  line up exactly. */
export function normCdf(x) {
    const a1 = 0.254829592, a2 = -0.284496736, a3 = 1.421413741;
    const a4 = -1.453152027, a5 = 1.061405429, p = 0.3275911;
    const sign = x < 0 ? -1 : 1;
    const xAbs = Math.abs(x) / Math.SQRT2;
    const t = 1 / (1 + p * xAbs);
    const y = 1 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1)
              * t * Math.exp(-xAbs * xAbs);
    return 0.5 * (1 + sign * y);
}

/** Early-exercise premium = American price - European reference.
 *  Returns null on bad inputs. */
export function earlyExercisePremium(american, european) {
    if (!Number.isFinite(american) || !Number.isFinite(european)) return null;
    return american - european;
}

/** Format a money-style number with a fixed number of decimals; falls
 *  back to "—" for non-finite. */
export function fmtMoney(x, digits = 4) {
    if (!Number.isFinite(x)) return '—';
    return x.toFixed(digits);
}

/** Standard-error → 95% confidence-interval width (half-width = 1.96·SE). */
export function ciHalfWidth(se) {
    return Number.isFinite(se) ? 1.96 * se : NaN;
}
