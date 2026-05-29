// Pure helpers for the VaR Calculator view.
//
// Parse a pasted return series (one return per token, whitespace OR
// comma OR newline separated; `#` and blank lines ignored). Compute a
// local histogram of the parsed series so the chart can overlay the 3
// backend VaR methods without a second round-trip per re-render.
//
// Conventions:
//   * Confidence c ∈ (0.5, 1) — slider value (e.g. 0.95 = 95% VaR).
//   * Tail alpha α = 1 − c    — what cornish_fisher::compute expects.
//   * VaR / ES are positive loss magnitudes (matches all three
//     backend endpoints).

import { parseFloatBlob } from './_paste_parser.js';

/** Parse the textarea blob into a flat array of finite numbers.
 *  Returns { value, errors } where errors are line-anchored. */
export function parseReturns(text) {
    return parseFloatBlob(text);
}

/** Validation gate before sending to any VaR endpoint. */
export function validateReturns(returns) {
    if (!Array.isArray(returns) || returns.length < 20) {
        return 'need at least 20 returns for a meaningful VaR estimate';
    }
    if (returns.some(x => !Number.isFinite(x))) return 'returns contain non-finite values';
    // Need some variation; a flat series has undefined VaR.
    const min = Math.min(...returns);
    const max = Math.max(...returns);
    if (max === min) return 'returns are constant — VaR is undefined';
    return null;
}

/** Convert confidence (0.95) to tail alpha (0.05). */
export function confidenceToAlpha(c) {
    return 1 - c;
}

/** Compute a histogram of returns with `nbins` equal-width bins over
 *  [min, max]. Returns { centers, counts, binWidth }. Used to render
 *  the empirical distribution underneath the VaR markers. */
export function histogram(returns, nbins = 40) {
    if (!Array.isArray(returns) || returns.length === 0 || nbins < 1) {
        return { centers: [], counts: [], binWidth: 0 };
    }
    let min = Infinity, max = -Infinity;
    for (const r of returns) {
        if (r < min) min = r;
        if (r > max) max = r;
    }
    if (min === max) {
        return { centers: [min], counts: [returns.length], binWidth: 0 };
    }
    const binWidth = (max - min) / nbins;
    const counts = new Array(nbins).fill(0);
    for (const r of returns) {
        let idx = Math.floor((r - min) / binWidth);
        if (idx >= nbins) idx = nbins - 1;
        if (idx < 0) idx = 0;
        counts[idx]++;
    }
    const centers = counts.map((_, i) => min + (i + 0.5) * binWidth);
    return { centers, counts, binWidth };
}

/** Format a positive-loss VaR/ES as a percentage with sign. The backend
 *  returns positive loss magnitudes; we render with a leading minus to
 *  reinforce that this is a loss. */
export function formatLoss(v, digits = 2) {
    if (!Number.isFinite(v)) return '—';
    return `-${(v * 100).toFixed(digits)}%`;
}
