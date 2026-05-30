// Weighted-midprice / Stoikov (2017) microprice helpers.
//
// Backend body: { quote?, quotes? }  — at least one must be set.
// Returns: { single?, series? } where each is MicropriceReport-shaped:
//   { midpoint, microprice, microprice_minus_midpoint, quote_imbalance,
//     spread, relative_spread }
//
// Stoikov microprice: bid_p · ask_size + ask_p · bid_size / total_size
// — biased toward the side with LESS size (the side likely to fill first).

import { t } from './i18n.js';

export const DEFAULT_INPUTS = {
    quotes: [],
};

export function validateInputs(input) {
    if (!Array.isArray(input.quotes))                                    return t('view.weighted_midprice.validate.quotes_array');
    if (input.quotes.length === 0)                                       return t('view.weighted_midprice.validate.quotes_empty');
    for (let i = 0; i < input.quotes.length; i++) {
        const q = input.quotes[i];
        if (!q || typeof q !== 'object')                                 return t('view.weighted_midprice.validate.quote_object', { i });
        for (const f of ['bid_price','bid_size','ask_price','ask_size']) {
            if (!Number.isFinite(q[f]))                                  return t('view.weighted_midprice.validate.field_finite', { i, f });
        }
    }
    return null;
}

export function buildBody(input) {
    return {
        quotes: input.quotes.map(q => ({
            bid_price: q.bid_price,
            bid_size:  q.bid_size,
            ask_price: q.ask_price,
            ask_size:  q.ask_size,
        })),
    };
}

// Pure-JS mirror of crates/traderview-core/src/weighted_midprice.rs::compute.
// Returns null when the quote fails validation (matches Rust Option<None>).
export function localCompute(q) {
    if (!q) return null;
    if (!Number.isFinite(q.bid_price) || q.bid_price <= 0) return null;
    if (!Number.isFinite(q.ask_price) || q.ask_price <= 0) return null;
    if (!Number.isFinite(q.bid_size)  || q.bid_size  <= 0) return null;
    if (!Number.isFinite(q.ask_size)  || q.ask_size  <= 0) return null;
    if (q.bid_price > q.ask_price) return null;
    const mid = (q.bid_price + q.ask_price) / 2;
    const total = q.bid_size + q.ask_size;
    if (total <= 0) return null;
    const micro = (q.bid_price * q.ask_size + q.ask_price * q.bid_size) / total;
    const imb = (q.bid_size - q.ask_size) / total;
    const spread = q.ask_price - q.bid_price;
    return {
        midpoint:                  mid,
        microprice:                micro,
        microprice_minus_midpoint: micro - mid,
        quote_imbalance:           imb,
        spread,
        relative_spread:           spread / mid,
    };
}

export function localSeries(quotes) {
    if (!Array.isArray(quotes)) return [];
    return quotes.map(localCompute);
}

// Parse "bid_price bid_size ask_price ask_size" per line; # comments + blanks.
export function parseQuotesBlob(blob) {
    const out = { quotes: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: t('common.parse.input_must_be_string') });
        return out;
    }
    const lines = blob.split('\n');
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i].split('#')[0].trim();
        if (!raw) continue;
        const toks = raw.split(/[\s,]+/).filter(t => t.length > 0);
        if (toks.length !== 4) {
            out.errors.push({ line_no: i + 1, message: t('view.weighted_midprice.parse.expected_4_tokens') });
            continue;
        }
        const [bid_price, bid_size, ask_price, ask_size] = toks.map(Number);
        if (![bid_price, bid_size, ask_price, ask_size].every(Number.isFinite)) {
            out.errors.push({ line_no: i + 1, message: t('common.parse.non_finite_token') });
            continue;
        }
        out.quotes.push({ bid_price, bid_size, ask_price, ask_size });
    }
    return out;
}

export function quotesToBlob(quotes) {
    return quotes.map(q => `${q.bid_price} ${q.bid_size} ${q.ask_price} ${q.ask_size}`).join('\n');
}

// Imbalance verdict — 5-tier classification of where micro sits relative to mid.
export function imbalanceBadge(r) {
    if (!r || !Number.isFinite(r.quote_imbalance)) return { key: 'view.wmp.badge.unknown', cls: '' };
    const i = r.quote_imbalance;
    if (i >  0.6) return { key: 'view.wmp.badge.heavy_bid',     cls: 'pos' };
    if (i >  0.2) return { key: 'view.wmp.badge.bid_lean',      cls: 'pos' };
    if (i >= -0.2) return { key: 'view.wmp.badge.balanced',     cls: '' };
    if (i >= -0.6) return { key: 'view.wmp.badge.ask_lean',     cls: 'neg' };
    return { key: 'view.wmp.badge.heavy_ask', cls: 'neg' };
}

// Aggregate over a series of reports (skipping nulls).
export function summarize(reports) {
    const valid = (reports || []).filter(Boolean);
    if (valid.length === 0) {
        return { count: 0, mean_micro: NaN, mean_mid: NaN, mean_dev: NaN,
                 mean_imb: NaN, mean_spread: NaN, max_abs_dev: NaN };
    }
    let sM = 0, sMid = 0, sDev = 0, sImb = 0, sSp = 0, maxDev = 0;
    for (const r of valid) {
        sM += r.microprice;
        sMid += r.midpoint;
        sDev += r.microprice_minus_midpoint;
        sImb += r.quote_imbalance;
        sSp += r.spread;
        const a = Math.abs(r.microprice_minus_midpoint);
        if (a > maxDev) maxDev = a;
    }
    const n = valid.length;
    return {
        count: n,
        mean_micro: sM / n, mean_mid: sMid / n,
        mean_dev: sDev / n, mean_imb: sImb / n,
        mean_spread: sSp / n,
        max_abs_dev: maxDev,
    };
}

// Synthetic demos.
export function makeDemoInput(kind = 'balanced') {
    switch (kind) {
        case 'balanced': {
            // 8 ticks, perfectly balanced sides.
            const qs = [];
            for (let i = 0; i < 8; i++) qs.push(q(100.00, 100, 100.10, 100));
            return { quotes: qs };
        }
        case 'heavy-bid': {
            // Bid-stacked book → microprice leans up.
            const qs = [];
            for (let i = 0; i < 8; i++) qs.push(q(100.00, 1000, 100.10, 100));
            return { quotes: qs };
        }
        case 'heavy-ask': {
            // Ask-stacked → microprice leans down.
            const qs = [];
            for (let i = 0; i < 8; i++) qs.push(q(100.00, 100, 100.10, 1000));
            return { quotes: qs };
        }
        case 'extreme-bid': {
            return { quotes: [q(100.00, 1e6, 100.10, 1)] };
        }
        case 'extreme-ask': {
            return { quotes: [q(100.00, 1, 100.10, 1e6)] };
        }
        case 'evolving-imbalance': {
            // Imbalance migrates from heavy bid to heavy ask across 10 quotes.
            const qs = [];
            for (let i = 0; i < 10; i++) {
                const t = i / 9;     // 0..1
                const bidSize = 1000 * (1 - t) + 100 * t;
                const askSize = 100 * (1 - t) + 1000 * t;
                qs.push(q(100.00, bidSize, 100.10, askSize));
            }
            return { quotes: qs };
        }
        case 'tight-spread': {
            // 1-tick spread, 50/50 size.
            return { quotes: [q(100.00, 500, 100.01, 500)] };
        }
        case 'wide-spread': {
            // 50-cent spread.
            return { quotes: [q(99.75, 100, 100.25, 100)] };
        }
        default:
            return makeDemoInput('balanced');
    }
}

function q(bid_price, bid_size, ask_price, ask_size) {
    return { bid_price, bid_size, ask_price, ask_size };
}

export function fmtUSD(v, d = 4) {
    if (!Number.isFinite(v)) return '—';
    const sign = v < 0 ? '-' : '';
    return sign + '$' + Math.abs(v).toFixed(d);
}

export function fmtUSDSigned(v, d = 4) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '-') + '$' + Math.abs(v).toFixed(d);
}

export function fmtBps(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v * 10_000).toFixed(d) + ' bps';
}

export function fmtImb(v, d = 4) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + v.toFixed(d);
}

export function fmtNum(v, d = 0) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtInt(v) {
    if (!Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}
