// Camarilla Pivot Points (Nick Stott) helpers.
//
// Backend body: { session: { high, low, close } }
// Returns: { h4, h3, h2, h1, pivot, l1, l2, l3, l4 } | null

import { t } from './i18n.js';

export const DEFAULT_INPUTS = {
    session: { high: 110, low: 100, close: 105 },
    current_price: 105,
};

export function validateInputs(input) {
    if (!input || !input.session)                          return t('view.camarilla.validate.session_missing');
    const { high, low, close } = input.session;
    if (typeof high !== 'number' || typeof low !== 'number' || typeof close !== 'number')
                                                            return t('view.camarilla.validate.hlc_numbers');
    if (!Number.isFinite(high) || !Number.isFinite(low) || !Number.isFinite(close))
                                                            return t('view.camarilla.validate.hlc_finite');
    if (high < low)                                         return t('view.camarilla.validate.high_lt_low');
    if (close < low || close > high)                        return t('view.camarilla.validate.close_outside');
    if (high <= 0 || low <= 0 || close <= 0)                return t('view.camarilla.validate.ohlc_positive');
    if (input.current_price != null
        && !Number.isFinite(input.current_price))           return t('view.camarilla.validate.current_finite');
    return null;
}

export function buildBody(input) {
    return { session: { high: input.session.high, low: input.session.low, close: input.session.close } };
}

// Pure-JS mirror of crates/traderview-core/src/camarilla_pivots.rs::compute.
export function localCompute(session) {
    if (!session
        || !Number.isFinite(session.high) || !Number.isFinite(session.low)
        || !Number.isFinite(session.close) || session.high < session.low) return null;
    const range = session.high - session.low;
    const k = range * 1.1;
    const pivot = (session.high + session.low + session.close) / 3;
    return {
        h4: session.close + k / 2,
        h3: session.close + k / 4,
        h2: session.close + k / 6,
        h1: session.close + k / 12,
        pivot,
        l1: session.close - k / 12,
        l2: session.close - k / 6,
        l3: session.close - k / 4,
        l4: session.close - k / 2,
    };
}

// Parse 3 tokens (h l c) plus optional current price.
// Format: "HIGH LOW CLOSE [CURRENT_PRICE]"
export function parseInputBlob(blob) {
    const out = { session: null, current_price: null, errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: 'input must be a string' });
        return out;
    }
    const tokens = blob
        .split('\n')
        .map(l => l.split('#')[0])
        .join(' ')
        .split(/[\s,]+/)
        .filter(t => t.length > 0)
        .map(t => Number(t.replace(/[\$,]/g, '')));
    if (tokens.length < 3) {
        out.errors.push({ line_no: 1, message: 'expected at least HIGH LOW CLOSE (3 tokens)' });
        return out;
    }
    for (let i = 0; i < tokens.length; i++) {
        if (!Number.isFinite(tokens[i])) {
            out.errors.push({ line_no: i + 1, message: `token ${i + 1} not finite` });
            return out;
        }
    }
    out.session = { high: tokens[0], low: tokens[1], close: tokens[2] };
    if (tokens.length >= 4) out.current_price = tokens[3];
    return out;
}

export function inputToBlob(input) {
    if (!input || !input.session) return '';
    const { high, low, close } = input.session;
    const base = `${high} ${low} ${close}`;
    if (input.current_price != null && Number.isFinite(input.current_price)) {
        return `${base} ${input.current_price}`;
    }
    return base;
}

// Where is price relative to the Camarilla band?
export function zoneBadge(levels, current_price) {
    if (!levels || current_price == null || !Number.isFinite(current_price)) {
        return { key: 'view.cam.zone.unknown', cls: '' };
    }
    if (current_price > levels.h4)  return { key: 'view.cam.zone.above_h4', cls: 'pos' };
    if (current_price > levels.h3)  return { key: 'view.cam.zone.h3_h4',    cls: 'pos' };
    if (current_price > levels.h2)  return { key: 'view.cam.zone.h2_h3',    cls: 'pos' };
    if (current_price > levels.h1)  return { key: 'view.cam.zone.h1_h2',    cls: '' };
    if (current_price > levels.l1)  return { key: 'view.cam.zone.pivot',    cls: '' };
    if (current_price > levels.l2)  return { key: 'view.cam.zone.l1_l2',    cls: '' };
    if (current_price > levels.l3)  return { key: 'view.cam.zone.l2_l3',    cls: 'neg' };
    if (current_price > levels.l4)  return { key: 'view.cam.zone.l3_l4',    cls: 'neg' };
    return { key: 'view.cam.zone.below_l4', cls: 'neg' };
}

// Camarilla trade rule: long L3 / short H3, breakout above H4 / breakdown below L4.
export function ruleBadge(levels, current_price) {
    if (!levels || current_price == null || !Number.isFinite(current_price)) {
        return { key: 'view.cam.rule.unknown', cls: '' };
    }
    if (current_price > levels.h4) return { key: 'view.cam.rule.breakout_long',  cls: 'pos' };
    if (current_price < levels.l4) return { key: 'view.cam.rule.breakdown_short', cls: 'neg' };
    if (current_price >= levels.h3 - (levels.h3 - levels.h2) * 0.1
     && current_price <= levels.h3 + (levels.h4 - levels.h3) * 0.1)
                                    return { key: 'view.cam.rule.short_reversal', cls: 'neg' };
    if (current_price >= levels.l3 - (levels.l3 - levels.l4) * 0.1
     && current_price <= levels.l3 + (levels.l2 - levels.l3) * 0.1)
                                    return { key: 'view.cam.rule.long_reversal',  cls: 'pos' };
    return { key: 'view.cam.rule.no_signal', cls: '' };
}

// Width verdict: H4-L4 as % of close.
export function widthBadge(levels) {
    if (!levels || !Number.isFinite(levels.h4) || !Number.isFinite(levels.l4)
        || !Number.isFinite(levels.pivot)) {
        return { key: 'view.cam.width.unknown', cls: '' };
    }
    const close_est = (levels.h1 + levels.l1) / 2;   // symmetric → equals close
    if (close_est === 0) return { key: 'view.cam.width.unknown', cls: '' };
    const w = (levels.h4 - levels.l4) / Math.abs(close_est);
    if (w < 0.005) return { key: 'view.cam.width.tight',  cls: '' };
    if (w < 0.02)  return { key: 'view.cam.width.normal', cls: '' };
    if (w < 0.05)  return { key: 'view.cam.width.wide',   cls: 'neg' };
    return { key: 'view.cam.width.very_wide', cls: 'neg' };
}

// Distance (signed) to nearest Camarilla level — informational helper.
export function nearestLevelInfo(levels, current_price) {
    if (!levels || current_price == null || !Number.isFinite(current_price)) {
        return { name: null, value: NaN, distance: NaN, distance_pct: NaN };
    }
    const candidates = [
        ['H4', levels.h4], ['H3', levels.h3], ['H2', levels.h2], ['H1', levels.h1],
        ['Pivot', levels.pivot],
        ['L1', levels.l1], ['L2', levels.l2], ['L3', levels.l3], ['L4', levels.l4],
    ];
    let best = null;
    let best_dist = Infinity;
    for (const [name, value] of candidates) {
        const d = Math.abs(current_price - value);
        if (d < best_dist) { best_dist = d; best = { name, value }; }
    }
    if (!best) return { name: null, value: NaN, distance: NaN, distance_pct: NaN };
    return {
        name: best.name,
        value: best.value,
        distance: current_price - best.value,
        distance_pct: best.value !== 0 ? (current_price - best.value) / Math.abs(best.value) : NaN,
    };
}

export function makeDemoInput(kind = 'standard-range') {
    switch (kind) {
        case 'standard-range':   return { session: { high: 110, low: 100, close: 105 }, current_price: 105 };
        case 'breakout-long':    return { session: { high: 110, low: 100, close: 109 }, current_price: 116 };
        case 'breakdown-short':  return { session: { high: 110, low: 100, close: 101 }, current_price: 94 };
        case 'short-reversal':   return { session: { high: 110, low: 100, close: 105 }, current_price: 107.75 };
        case 'long-reversal':    return { session: { high: 110, low: 100, close: 105 }, current_price: 102.25 };
        case 'tight-range':      return { session: { high: 100.5, low: 99.5, close: 100 }, current_price: 100 };
        case 'wide-range':       return { session: { high: 130, low: 80, close: 105 }, current_price: 105 };
        case 'flat-session':     return { session: { high: 100, low: 100, close: 100 }, current_price: 100 };
        default: return makeDemoInput('standard-range');
    }
}

export function fmtPrice(v, d = 4) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtPriceSigned(v, d = 4) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + v.toFixed(d);
}

export function fmtPct(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtInt(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}
