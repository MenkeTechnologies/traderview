// IV Backtest (earnings-straddle) helpers shared by view + vitest.
//
// Backend body shape: { implied_move_pct: f64, realized_pcts: f64[] }.
// Realized %s are taken absolute by the backend (long-straddle PnL is
// symmetric); the view also displays signed for direction context.

import { parseFloatBlob } from './_paste_parser.js';
import { t } from './i18n.js';

// Realized history can include sign (direction of post-event move).
// Negative values are kept as-is — backend abs()'es them.
export function parseRealized(text) {
    return parseFloatBlob(text);
}

export function validateInputs(implied, realized) {
    if (!Number.isFinite(implied) || implied <= 0)
        return 'implied_move_pct must be > 0';
    if (implied > 100) return 'implied_move_pct looks like raw bps — enter as %, e.g. 5.5 for 5.5%';
    if (!Array.isArray(realized) || realized.length < 4)
        return 'need at least 4 realized observations';
    if (!realized.every(Number.isFinite))
        return 'realized history must contain only finite values';
    return null;
}

export function buildBody(implied, realized) {
    return { implied_move_pct: implied, realized_pcts: realized };
}

// Three-tier label from the backend's "long"/"short"/"neutral" string +
// the edge magnitude. Display badge + color + action hint.
export function recommendationBadge(rec, edgePct) {
    const sign = Number.isFinite(edgePct) ? (edgePct >= 0 ? '+' : '') : '';
    const edgeStr = Number.isFinite(edgePct) ? t('view.iv_backtest.edge_fmt', { sign, pct: edgePct.toFixed(2) }) : '';
    switch (rec) {
        case 'long':  return { label: t('view.iv_backtest.rec.long.label',    { edge: edgeStr }), cls: 'pos',
                                hint: t('view.iv_backtest.rec.long.hint') };
        case 'short': return { label: t('view.iv_backtest.rec.short.label',   { edge: edgeStr }), cls: 'neg',
                                hint: t('view.iv_backtest.rec.short.hint') };
        case 'neutral':
        default:      return { label: t('view.iv_backtest.rec.neutral.label', { edge: edgeStr }), cls: '',
                                hint: t('view.iv_backtest.rec.neutral.hint') };
    }
}

// Equal-width histogram for the realized distribution chart. Splits the
// |realized| range into nBins buckets and returns parallel xs/ys arrays
// suitable for uPlot's bar plot.
export function histogram(values, nBins = 20) {
    if (!Array.isArray(values) || values.length === 0)
        return { centers: [], counts: [] };
    const abs = values.map(v => Math.abs(v)).filter(Number.isFinite);
    if (abs.length === 0) return { centers: [], counts: [] };
    const lo = 0;
    const hi = Math.max(...abs);
    if (hi <= lo) return { centers: [hi], counts: [abs.length] };
    const width = (hi - lo) / nBins;
    const counts = new Array(nBins).fill(0);
    const centers = new Array(nBins);
    for (let i = 0; i < nBins; i++) centers[i] = lo + (i + 0.5) * width;
    for (const v of abs) {
        let i = Math.floor((v - lo) / width);
        if (i >= nBins) i = nBins - 1;
        if (i < 0) i = 0;
        counts[i]++;
    }
    return { centers, counts };
}

// Deterministic 16-quarter demo history for an event where realized
// systematically beats implied — pushes the backend toward "long".
export function makeDemoData() {
    const implied_move_pct = 4.5;
    const realized_pcts = [
        7.2, -8.5, 5.1, 6.0,   // year 1
        -9.3, 3.2, 11.4, -7.0, // year 2
        4.8, -5.5, 8.1, 9.2,   // year 3
        -10.5, 6.7, -7.8, 4.0, // year 4
    ];
    return { implied_move_pct, realized_pcts };
}

export function fmtPct(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d) + '%';
}

export function fmtPnl(v) {
    if (!Number.isFinite(v)) return '—';
    const sign = v >= 0 ? '+' : '';
    return sign + (v * 100).toFixed(1) + '% per $1';
}

export function fmtWinRate(v) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(0) + '%';
}
