// Pure helpers for the Bayesian Change Point Detector view.
//
// Reuses the shared paste parser for the return series. Adds hazard-
// rate validation (must be in (0, 1) — a probability per bar) and a
// detector that finds the top-K change-points by probability.

import { parseFloatBlob } from './_paste_parser.js';
import { t } from './i18n.js';

/** Parse the return-series textarea. */
export function parseReturns(text) {
    return parseFloatBlob(text);
}

/** Validate inputs. The backend needs ≥ 1 return (it produces an
 *  empty report for shorter inputs); we require ≥ 30 to keep the view
 *  useful (any fewer and the change-point posterior is uninformative).
 *  Hazard rate is the per-bar prior probability of a regime shift —
 *  must be strictly in (0, 1). */
export function validateInputs(returns, hazard) {
    if (!Array.isArray(returns) || returns.length < 30) {
        return t('view.bocpd.validate.need_30');
    }
    if (returns.some(x => !Number.isFinite(x))) return t('view.bocpd.validate.non_finite');
    if (!Number.isFinite(hazard) || hazard <= 0 || hazard >= 1) {
        return t('view.bocpd.validate.hazard');
    }
    return null;
}

/** Build the JSON body for /analytics/bayesian-change-point. */
export function buildBody(returns, hazard) {
    return { returns, hazard };
}

/** Find the top-K indices where change_point_probability exceeded the
 *  threshold, sorted by descending probability. Used to populate the
 *  "detected change points" summary card. */
export function topChangePoints(probArray, threshold, topK) {
    const out = [];
    if (!Array.isArray(probArray) || !Number.isFinite(threshold) || !Number.isInteger(topK) || topK < 1) {
        return out;
    }
    for (let i = 0; i < probArray.length; i++) {
        const p = probArray[i];
        if (Number.isFinite(p) && p >= threshold) out.push({ index: i, probability: p });
    }
    out.sort((a, b) => b.probability - a.probability);
    return out.slice(0, topK);
}

/** Count bars whose change-point probability exceeded the threshold —
 *  useful as a "how chatty is this hazard rate" diagnostic. A high
 *  count at hazard=0.01 means the data is genuinely full of regime
 *  shifts (or the model is over-sensitive). */
export function countAboveThreshold(probArray, threshold) {
    if (!Array.isArray(probArray) || !Number.isFinite(threshold)) return 0;
    let n = 0;
    for (const p of probArray) if (Number.isFinite(p) && p >= threshold) n++;
    return n;
}

/** Format hazard as "x.xx%" for the summary. */
export function fmtHazardPct(h) {
    if (!Number.isFinite(h)) return '—';
    return `${(h * 100).toFixed(2)}%`;
}
