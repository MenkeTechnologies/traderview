// Anchored Momentum helpers — ROC vs an anchor bar (earnings, FOMC, etc.).
//
// Backend body: { closes: number[], anchor: number, smooth_period: number }
// Returns: (number | null)[] of length closes.length.
//
// raw_i  = (close_i − close_anchor) / close_anchor   for i ≥ anchor
// out_i  = WMA(raw, smooth_period) starting at i = anchor + smooth_period − 1
//          with weights 1..smooth_period (sum = n(n+1)/2)
// smooth_period=1 returns the raw series.

import { t } from './i18n.js';

export const DEFAULT_SMOOTH = 5;

export const DEFAULT_INPUTS = {
    closes: [],
    anchor: 0,
    smooth_period: DEFAULT_SMOOTH,
};

export function validateInputs(input) {
    if (!Array.isArray(input.closes))                                    return t('view.anchored_momentum.validate.closes_array');
    for (let i = 0; i < input.closes.length; i++) {
        // NaN tolerated mid-series (Rust skips them per-bar).
        if (typeof input.closes[i] !== 'number')                          return t('view.anchored_momentum.validate.close_number', { i });
    }
    if (!Number.isInteger(input.anchor))                                  return t('view.anchored_momentum.validate.anchor_int');
    if (input.anchor < 0)                                                  return t('view.anchored_momentum.validate.anchor_min');
    if (!Number.isInteger(input.smooth_period))                           return t('view.anchored_momentum.validate.smooth_int');
    if (input.smooth_period < 1)                                          return t('view.anchored_momentum.validate.smooth_min');
    if (input.closes.length > 0 && input.anchor >= input.closes.length)   return t('view.anchored_momentum.validate.anchor_lt_len', { anchor: input.anchor, len: input.closes.length });
    return null;
}

export function buildBody(input) {
    return {
        closes:        input.closes,
        anchor:        input.anchor,
        smooth_period: input.smooth_period,
    };
}

// Pure-JS mirror of crates/traderview-core/src/anchored_momentum.rs::compute.
export function localCompute(closes, anchor, smooth_period) {
    const n = closes.length;
    const out = new Array(n).fill(null);
    if (anchor >= n || smooth_period === 0) return out;
    const anchor_close = closes[anchor];
    if (!Number.isFinite(anchor_close) || anchor_close <= 0) return out;
    const raw = new Array(n).fill(null);
    for (let i = anchor; i < n; i++) {
        const c = closes[i];
        if (!Number.isFinite(c)) continue;
        const v = (c - anchor_close) / anchor_close;
        if (Number.isFinite(v)) raw[i] = v;
    }
    if (smooth_period === 1) return raw;
    const max_eligible = n - anchor;
    if (smooth_period > max_eligible) return out;
    const weight_sum = smooth_period * (smooth_period + 1) / 2;
    for (let i = 0; i < n; i++) {
        if (i < anchor + smooth_period - 1) continue;
        const lo = i + 1 - smooth_period;
        if (lo < anchor) continue;
        let numer = 0;
        let ok = true;
        for (let k = 0; k < smooth_period; k++) {
            const j = lo + k;
            const v = raw[j];
            if (v == null) { ok = false; break; }
            numer += v * (k + 1);
        }
        if (ok && Number.isFinite(numer)) out[i] = numer / weight_sum;
    }
    return out;
}

// Parse comma/whitespace-separated closes; blanks + # comments ignored.
export function parseClosesBlob(blob) {
    const out = { closes: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: t('common.parse.input_must_be_string') });
        return out;
    }
    const tokens = blob
        .split('\n')
        .map(l => l.split('#')[0])
        .join(' ')
        .split(/[\s,]+/)
        .filter(t => t.length > 0);
    for (let i = 0; i < tokens.length; i++) {
        const tok = tokens[i].toLowerCase();
        if (tok === 'nan') {
            out.closes.push(NaN);
            continue;
        }
        const v = Number(tokens[i]);
        if (!Number.isFinite(v)) {
            out.errors.push({ line_no: i + 1, message: `token "${tokens[i]}" not finite` });
            continue;
        }
        out.closes.push(v);
    }
    return out;
}

export function closesToBlob(closes) {
    return closes.map(v => Number.isFinite(v) ? v : 'NaN').join('\n');
}

// Verdict on the last smoothed momentum value.
export function momentumBadge(v) {
    if (v == null || !Number.isFinite(v)) return { key: 'view.anch_mom.badge.unknown', cls: '' };
    if (v >= 0.20)  return { key: 'view.anch_mom.badge.strong_up',   cls: 'pos' };
    if (v >= 0.05)  return { key: 'view.anch_mom.badge.up',          cls: 'pos' };
    if (v > -0.05)  return { key: 'view.anch_mom.badge.flat',        cls: '' };
    if (v > -0.20)  return { key: 'view.anch_mom.badge.down',        cls: 'neg' };
    return { key: 'view.anch_mom.badge.strong_down', cls: 'neg' };
}

// Aggregate stats from the smoothed series.
export function summarize(series) {
    if (!Array.isArray(series) || series.length === 0)
        return { count: 0, populated: 0, last: NaN, mean: NaN, min: NaN, max: NaN };
    let populated = 0, last = NaN, sum = 0, mn = Infinity, mx = -Infinity;
    for (const v of series) {
        if (v != null && Number.isFinite(v)) {
            populated++;
            last = v;
            sum += v;
            if (v < mn) mn = v;
            if (v > mx) mx = v;
        }
    }
    return {
        count: series.length,
        populated,
        last,
        mean: populated > 0 ? sum / populated : NaN,
        min: Number.isFinite(mn) ? mn : NaN,
        max: Number.isFinite(mx) ? mx : NaN,
    };
}

// LCG for stable demos.
function lcg(seed) {
    let state = BigInt(7919) + seed;
    return () => {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(state >> 32n) / 0xFFFFFFFF - 0.5;
    };
}

export function makeDemoInput(kind = 'post-earnings-rally') {
    switch (kind) {
        case 'post-earnings-rally': {
            // Flat → earnings event → ramp up.
            const closes = [];
            for (let i = 0; i < 20; i++) closes.push(100);
            for (let i = 0; i < 30; i++) closes.push(100 + i * 0.5);
            return { closes, anchor: 19, smooth_period: 5 };
        }
        case 'post-news-crash': {
            const closes = [];
            for (let i = 0; i < 20; i++) closes.push(100);
            for (let i = 0; i < 30; i++) closes.push(100 - i * 0.5);
            return { closes, anchor: 19, smooth_period: 5 };
        }
        case 'flat-after-anchor': {
            const closes = new Array(40).fill(100);
            return { closes, anchor: 10, smooth_period: 5 };
        }
        case 'pre-anchor-clipped': {
            // Anchor at index 25 — bars before are nulls in the output.
            const closes = [];
            for (let i = 0; i < 25; i++) closes.push(100 + Math.sin(i * 0.3));
            for (let i = 0; i < 25; i++) closes.push(105 + i * 0.2);
            return { closes, anchor: 25, smooth_period: 5 };
        }
        case 'raw-only': {
            // smooth_period=1 → raw series.
            const closes = [];
            for (let i = 0; i < 20; i++) closes.push(100 + i);
            return { closes, anchor: 0, smooth_period: 1 };
        }
        case 'long-smoothing': {
            // smooth_period=10 — slow but stable.
            const closes = [];
            for (let i = 0; i < 60; i++) closes.push(100 + i * 0.3);
            return { closes, anchor: 0, smooth_period: 10 };
        }
        case 'with-nan-gap': {
            const closes = [];
            for (let i = 0; i < 20; i++) closes.push(100 + i * 0.2);
            closes[10] = NaN;
            return { closes, anchor: 0, smooth_period: 3 };
        }
        case 'fomc-volatile': {
            const rand = lcg(42n);
            const closes = [];
            for (let i = 0; i < 20; i++) closes.push(100 + rand() * 1);
            // FOMC bar at index 20 then volatility.
            closes.push(100);
            for (let i = 0; i < 30; i++) closes.push(closes[closes.length - 1] + rand() * 4);
            return { closes, anchor: 20, smooth_period: 5 };
        }
        default: return makeDemoInput('post-earnings-rally');
    }
}

export function fmtPctSigned(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + (v * 100).toFixed(d) + '%';
}

export function fmtUSD(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return '$' + v.toFixed(d);
}

export function fmtInt(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}
