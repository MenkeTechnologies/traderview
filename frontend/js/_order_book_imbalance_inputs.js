// Order Book Imbalance helpers shared by view + vitest.
//
// Backend body shape: { bid_sizes: f64[], ask_sizes: f64[], levels: usize }.

import { parseFloatBlob } from './_paste_parser.js';

// Sizes are non-negative — reuse the shared parser with nonNegative gate.
export function parseSizes(text) {
    return parseFloatBlob(text, { nonNegative: true });
}

export function validateInputs(bidSizes, askSizes, levels) {
    if (!Array.isArray(bidSizes) || bidSizes.length === 0)
        return 'bid_sizes must have at least 1 level';
    if (!Array.isArray(askSizes) || askSizes.length === 0)
        return 'ask_sizes must have at least 1 level';
    if (!Number.isInteger(levels) || levels < 1 || levels > 50)
        return 'levels must be integer in [1, 50]';
    if (!bidSizes.every(v => Number.isFinite(v) && v >= 0))
        return 'bid_sizes must be non-negative finite';
    if (!askSizes.every(v => Number.isFinite(v) && v >= 0))
        return 'ask_sizes must be non-negative finite';
    return null;
}

export function buildBody(bidSizes, askSizes, levels) {
    return { bid_sizes: bidSizes, ask_sizes: askSizes, levels };
}

// Per-level rows for the table view. Pads the shorter side with 0 so
// the table aligns visually. Returns rows up to max(bidLen, askLen, levels).
export function alignLevels(bidSizes, askSizes, levels) {
    const showLevels = Math.min(
        Math.max(bidSizes.length, askSizes.length),
        Math.max(levels, 1),
    );
    const rows = [];
    for (let i = 0; i < showLevels; i++) {
        rows.push({
            level: i + 1,
            bid: i < bidSizes.length ? bidSizes[i] : 0,
            ask: i < askSizes.length ? askSizes[i] : 0,
        });
    }
    return rows;
}

// Maps the backend's snake_case bias enum to a UI badge.
const BIAS_BADGES = {
    strongly_bid: { label: 'STRONGLY BID', cls: 'pos', hint: 'heavy buying pressure on top of book' },
    bid:          { label: 'BID',          cls: 'pos', hint: 'moderate bid skew' },
    balanced:     { label: 'BALANCED',     cls: '',    hint: 'no directional pressure' },
    ask:          { label: 'ASK',          cls: 'neg', hint: 'moderate ask skew' },
    strongly_ask: { label: 'STRONGLY ASK', cls: 'neg', hint: 'heavy selling pressure on top of book' },
};

export function biasBadge(bias) {
    return BIAS_BADGES[bias] || { label: String(bias || '—'), cls: '', hint: '' };
}

// Deterministic preset books — used by the 3 demo buttons.
export function makeDemoBook(kind) {
    switch (kind) {
        case 'bid-pressure':
            return {
                bid_sizes: [500, 380, 290, 220, 180, 150, 120, 90, 70, 50],
                ask_sizes: [120, 100, 80, 60, 50, 40, 30, 20, 15, 10],
            };
        case 'ask-pressure':
            return {
                bid_sizes: [120, 100, 80, 60, 50, 40, 30, 20, 15, 10],
                ask_sizes: [500, 380, 290, 220, 180, 150, 120, 90, 70, 50],
            };
        case 'balanced':
        default:
            return {
                bid_sizes: [200, 180, 160, 140, 120, 100, 90, 80, 70, 60],
                ask_sizes: [200, 180, 160, 140, 120, 100, 90, 80, 70, 60],
            };
    }
}

export function fmtN(v) {
    if (!Number.isFinite(v)) return '—';
    return v.toLocaleString('en-US');
}

export function fmtImbalance(v) {
    if (!Number.isFinite(v)) return '—';
    const sign = v >= 0 ? '+' : '';
    return sign + v.toFixed(4);
}
