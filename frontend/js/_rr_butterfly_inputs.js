// Pure helpers for the Risk-Reversal / Butterfly Calculator view.
//
// Two modes share one endpoint via the backend's untagged enum:
//
//   Decompose:  σ_25C, σ_25P, σ_ATM   →   ATM, RR, BF, skew z-score
//   Reconstruct: ATM, RR, BF           →   σ_25C, σ_25P
//
// Math (FX vol-quote convention):
//   RR  = σ_25C − σ_25P
//   BF  = (σ_25C + σ_25P) / 2 − σ_ATM
//   skew_z = RR / σ_ATM
// Inversely:
//   σ_25C = σ_ATM + BF + RR/2
//   σ_25P = σ_ATM + BF − RR/2

/** Build the JSON body for /analytics/risk-reversal-25-delta-butterfly.
 *  `mode === 'decompose'` returns `{ sigma_25_call, sigma_25_put, sigma_atm }`;
 *  `mode === 'reconstruct'` returns `{ atm, rr, bf }`. The backend
 *  matches on field presence (serde untagged enum). */
import { t } from './i18n.js';

export function buildBody(mode, params) {
    if (mode === 'decompose') {
        return {
            sigma_25_call: params.sigma_25_call,
            sigma_25_put:  params.sigma_25_put,
            sigma_atm:     params.sigma_atm,
        };
    }
    if (mode === 'reconstruct') {
        return { atm: params.atm, rr: params.rr, bf: params.bf };
    }
    throw new Error(`unknown mode "${mode}"`);
}

/** Validate per-mode. Returns null on success or an error string. */
export function validateInputs(mode, params) {
    if (mode === 'decompose') {
        for (const k of ['sigma_25_call', 'sigma_25_put', 'sigma_atm']) {
            const v = params[k];
            if (!Number.isFinite(v)) return t('view.rr_butterfly.validate.field_finite', { field: k });
            if (v <= 0) return t('view.rr_butterfly.validate.field_positive', { field: k, value: v });
        }
        return null;
    }
    if (mode === 'reconstruct') {
        if (!Number.isFinite(params.atm) || params.atm <= 0) return t('view.rr_butterfly.validate.atm');
        if (!Number.isFinite(params.rr)) return t('view.rr_butterfly.validate.rr');
        if (!Number.isFinite(params.bf)) return t('view.rr_butterfly.validate.bf');
        // The wing reconstruction can produce negative IVs if RR and BF
        // are pathologically large — surface that pre-flight so the
        // user sees a friendlier error than the backend's null.
        const call = params.atm + params.bf + params.rr / 2;
        const put  = params.atm + params.bf - params.rr / 2;
        if (call <= 0) return t('view.rr_butterfly.validate.call_negative', { value: call.toFixed(6) });
        if (put  <= 0) return t('view.rr_butterfly.validate.put_negative',  { value: put.toFixed(6) });
        return null;
    }
    return t('view.rr_butterfly.validate.unknown_mode', { mode });
}

/** Local closed-form decompose. Used to power instant "what's the RR
 *  for these wings?" feedback without a round-trip per keystroke; the
 *  backend value is the canonical one and overwrites this on submit. */
export function decomposeLocal(sigma25call, sigma25put, sigmaAtm) {
    const wingAvg = 0.5 * (sigma25call + sigma25put);
    const rr = sigma25call - sigma25put;
    const bf = wingAvg - sigmaAtm;
    const skewZ = sigmaAtm > 0 ? rr / sigmaAtm : NaN;
    return { atm: sigmaAtm, rr, bf, skew_zscore: skewZ };
}

/** Local closed-form reconstruct. */
export function reconstructLocal(atm, rr, bf) {
    return {
        sigma_25_call: atm + bf + rr / 2,
        sigma_25_put:  atm + bf - rr / 2,
    };
}

/** Format an IV / RR / BF as a percent (FX vol quotes are conventionally
 *  in vol-points = percent). Null → "—". */
export function fmtVolPct(x, digits = 3) {
    if (!Number.isFinite(x)) return '—';
    return `${(x * 100).toFixed(digits)}%`;
}

/** Format a skew z-score (dimensionless ratio). */
export function fmtSkewZ(x, digits = 3) {
    if (!Number.isFinite(x)) return '—';
    return x.toFixed(digits);
}
