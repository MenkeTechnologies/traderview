// Pure helpers for the Microprice (Stoikov) calculator.
//
// Stoikov 2018 microprice: a quote-adjusted fair mid that weighs the
// opposite side by your queue's imbalance.
//   imbalance      = bid_size / (bid_size + ask_size)
//   microprice     = bid · (1 - imbalance) + ask · imbalance
//                  = bid · (ask_size / total_size) + ask · (bid_size / total_size)
// When the bid queue is much larger than the ask queue, the next print
// is more likely to lift the ask → microprice biases toward the ask.

import { t } from './i18n.js';

/** Build the JSON body for /analytics/microprice-stoikov. The backend
 *  accepts an array (time series); we wrap the user's single snapshot
 *  in a 1-element array. */
export function buildBody(quote) {
    return { quotes: [{
        bid: quote.bid, ask: quote.ask,
        bid_size: quote.bid_size, ask_size: quote.ask_size,
    }] };
}

/** Validate inputs. Returns null on success or a friendly error string. */
export function validateQuote(q) {
    if (!Number.isFinite(q.bid) || q.bid <= 0) return t('view.microprice.validate.bid');
    if (!Number.isFinite(q.ask) || q.ask <= 0) return t('view.microprice.validate.ask');
    if (q.bid > q.ask) return t('view.microprice.validate.crossed');
    if (!Number.isFinite(q.bid_size) || q.bid_size < 0) return t('view.microprice.validate.bid_size');
    if (!Number.isFinite(q.ask_size) || q.ask_size < 0) return t('view.microprice.validate.ask_size');
    if (q.bid_size + q.ask_size <= 0) return t('view.microprice.validate.need_size');
    return null;
}

/** Local closed-form for instant feedback. Mirrors the Rust module so
 *  the chart can sweep across an imbalance range without N round-trips. */
export function microprice(bid, ask, bidSize, askSize) {
    const total = bidSize + askSize;
    if (!(total > 0)) return null;
    const imbalance = bidSize / total;
    return bid * (1 - imbalance) + ask * imbalance;
}

/** Generate a sweep of microprices across all possible imbalances (from
 *  pure-ask-side queue, 0.0, to pure-bid-side queue, 1.0). Useful as a
 *  reference chart showing where the user's current point falls on the
 *  full imbalance → microprice line. Returns parallel arrays. */
export function imbalanceSweep(bid, ask, points = 101) {
    if (!(ask > bid)) return { xs: [], ys: [] };
    const xs = new Array(points);
    const ys = new Array(points);
    for (let i = 0; i < points; i++) {
        const imbalance = i / (points - 1);
        xs[i] = imbalance;
        ys[i] = bid * (1 - imbalance) + ask * imbalance;
    }
    return { xs, ys };
}

/** Format a price with the given decimals; "—" for non-finite. */
export function fmtPrice(x, digits = 4) {
    if (!Number.isFinite(x)) return '—';
    return x.toFixed(digits);
}

/** Format a basis-points number signed. */
export function fmtBps(x, digits = 2) {
    if (!Number.isFinite(x)) return '—';
    const sign = x > 0 ? '+' : (x < 0 ? '−' : '');
    const abs = Math.abs(x).toFixed(digits);
    return `${sign}${abs} bps`;
}

/** Format an imbalance fraction as a percent. */
export function fmtImbalance(x) {
    if (!Number.isFinite(x)) return '—';
    return `${(x * 100).toFixed(2)}%`;
}
