// IV Rank helpers shared by view + vitest.
//
// Backend body shape: { current_iv: f64, history: f64[] }.
// IV is decimal (0.25 = 25% annualized) by convention; the view leaves
// units to the user.

import { parseFloatBlob } from './_paste_parser.js';
import { t } from './i18n.js';

export function parseHistory(text) {
    return parseFloatBlob(text, { nonNegative: true });
}

export function validateInputs(currentIv, history) {
    if (!Number.isFinite(currentIv) || currentIv < 0)
        return 'current_iv must be ≥ 0';
    if (!Array.isArray(history) || history.length < 10)
        return 'history must have at least 10 observations';
    if (!history.every(v => Number.isFinite(v) && v >= 0))
        return 'history must contain only non-negative finite values';
    return null;
}

export function buildBody(currentIv, history) {
    return { current_iv: currentIv, history };
}

// Two narrative buckets following the canonical trader convention. The
// backend's `classify` enum cuts at 25/75; UI shows the same cuts plus
// "low / normal / high" labels with action hints.
export function rankEnvironment(rank) {
    if (!Number.isFinite(rank)) return { label: '—', cls: '', hint: '' };
    if (rank < 25)  return { label: t('view.iv_rank.env.low.label'),    cls: 'neg', hint: t('view.iv_rank.env.low.hint') };
    if (rank > 75)  return { label: t('view.iv_rank.env.high.label'),   cls: 'pos', hint: t('view.iv_rank.env.high.hint') };
    return            { label: t('view.iv_rank.env.normal.label'), cls: '',    hint: t('view.iv_rank.env.normal.hint') };
}

// Two-tier check on whether IV rank and IV percentile agree. When they
// disagree by ≥20 points the underlying IV series is skewed (e.g. a
// single earnings spike pulled the range), and IV percentile is the
// more honest metric.
export function rankVsPercentileNote(rank, pct) {
    if (!Number.isFinite(rank) || !Number.isFinite(pct)) return '';
    const delta = Math.abs(rank - pct);
    if (delta < 10) return 'rank and percentile agree closely — trust either';
    if (delta < 20) return 'mild divergence between rank and percentile';
    return 'rank and percentile diverge ≥20pts — series is skewed, prefer percentile';
}

// Synthesizes a deterministic 252-day IV history with one earnings-style
// spike near the end. Demonstrates the rank-vs-percentile divergence.
export function makeDemoHistory(seed = 1) {
    let s = seed;
    const rand = () => { s = (s * 1664525 + 1013904223) | 0; return ((s >>> 0) / 0xffffffff); };
    const out = new Array(252);
    for (let i = 0; i < 252; i++) {
        let iv = 0.22 + 0.05 * Math.sin(i / 18) + (rand() - 0.5) * 0.02;
        // Earnings spike at day 240 lifts IV to ~0.65 for 5 days.
        if (i >= 240 && i < 245) iv += 0.40;
        out[i] = Math.max(0.05, Number(iv.toFixed(4)));
    }
    return out;
}

export function fmtIv(v) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(2) + '%';
}

export function fmtRank(v) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(1);
}
