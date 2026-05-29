// Pure helpers for the Hurst Exponent view.
//
// Reuses the shared paste parser for the return series. Adds chunk-
// sizes parsing (a small integer list) and a regime classifier:
//   H < 0.5   → mean-reverting (Ornstein-Uhlenbeck-like)
//   H ≈ 0.5   → random walk (no memory)
//   H > 0.5   → persistent / trending (long-memory)
// The interpretation strength scales with how far from 0.5 H sits.

import { parseFloatBlob } from './_paste_parser.js';

/** Parse the return-series textarea. */
export function parseReturns(text) {
    return parseFloatBlob(text);
}

/** Parse comma/space-separated positive integers for chunk sizes.
 *  Falls back to the canonical [10, 20, 50, 100, 250] if input is
 *  empty after parsing (matches the backend default). */
export function parseChunkSizes(text) {
    const errors = [];
    if (typeof text !== 'string' || !text.trim()) {
        return { value: [10, 20, 50, 100, 250], errors };
    }
    const out = [];
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const stripped = raw.trim();
        if (!stripped || stripped.startsWith('#')) continue;
        for (const tok of stripped.split(/[\s,]+/).filter(Boolean)) {
            const n = Number(tok);
            if (!Number.isInteger(n) || n < 2) {
                errors.push({ line_no: i + 1, raw, message: `bad chunk size "${tok}" (must be integer ≥ 2)` });
                continue;
            }
            out.push(n);
        }
    }
    return { value: out, errors };
}

/** Validate the combined inputs. */
export function validateInputs(returns, chunkSizes) {
    if (!Array.isArray(returns) || returns.length < 10) {
        return 'need at least 10 returns for a Hurst estimate';
    }
    if (returns.some(x => !Number.isFinite(x))) return 'returns contain non-finite values';
    if (!Array.isArray(chunkSizes) || chunkSizes.length < 2) {
        return 'need at least 2 chunk sizes for the regression';
    }
    if (chunkSizes.some(c => !Number.isInteger(c) || c < 2)) {
        return 'every chunk size must be an integer ≥ 2';
    }
    if (chunkSizes.some(c => c > returns.length)) {
        return `chunk sizes must be ≤ series length (${returns.length})`;
    }
    return null;
}

/** Build the JSON body for /analytics/hurst-exponent. */
export function buildBody(returns, chunkSizes) {
    return { returns, chunk_sizes: chunkSizes };
}

/** Three-bucket regime classification. Returns an i18n key so the caller
 *  can translate via `t()`; pure helper stays decoupled from i18n.js. */
export function regimeLabelKey(h) {
    if (!Number.isFinite(h)) return 'view.hurst.regime.unknown';
    if (h < 0.45) return 'view.hurst.regime.mean_reverting';
    if (h > 0.55) return 'view.hurst.regime.trending';
    return 'view.hurst.regime.random_walk';
}

/** Strength qualifier — distance from 0.5 mapped to a word. Returns an
 *  i18n key. */
export function regimeStrengthKey(h) {
    if (!Number.isFinite(h)) return 'view.hurst.strength.unknown';
    const d = Math.abs(h - 0.5);
    if (d < 0.05) return 'view.hurst.strength.weak';
    if (d < 0.15) return 'view.hurst.strength.moderate';
    return 'view.hurst.strength.strong';
}

/** Color class for the H value card based on regime — green for
 *  trending, red for mean-reverting, neutral for random walk. */
export function regimeCssClass(h) {
    if (!Number.isFinite(h)) return '';
    if (h > 0.55) return 'pos';
    if (h < 0.45) return 'neg';
    return '';
}
