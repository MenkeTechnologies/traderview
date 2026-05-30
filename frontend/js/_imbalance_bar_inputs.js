// Tick Imbalance Bar (TIB) helpers — López de Prado AFML signed-volume bar.
//
// Backend body: { prints: [{price, size}, ...], imbalance_threshold }
// Returns: ImbalanceBar[] = { open, high, low, close, volume, imbalance, tick_count }
//
// Sign rule: +1 if price > prev, -1 if price < prev, prior sign on tie.
// Bar closes when |Σ sign × size| ≥ imbalance_threshold.
// Trailing partial bars are NOT emitted.

import { t } from './i18n.js';

export const DEFAULT_INPUTS = {
    prints: [],
    imbalance_threshold: 1000,
};

export function validateInputs(input) {
    if (!Array.isArray(input.prints))                            return t('view.imbalance_bar.validate.prints_array');
    for (let i = 0; i < input.prints.length; i++) {
        const p = input.prints[i];
        if (!p || typeof p !== 'object')                         return t('view.imbalance_bar.validate.print_object', { i });
        if (!Number.isFinite(p.price))                           return t('view.imbalance_bar.validate.price_finite', { i });
        if (p.price <= 0)                                        return t('view.imbalance_bar.validate.price_positive', { i });
        if (!Number.isFinite(p.size))                            return t('view.imbalance_bar.validate.size_finite', { i });
        if (p.size < 0)                                          return t('view.imbalance_bar.validate.size_negative', { i });
    }
    if (!Number.isFinite(input.imbalance_threshold))             return t('view.imbalance_bar.validate.threshold_finite');
    if (input.imbalance_threshold <= 0)                          return t('view.imbalance_bar.validate.threshold_positive');
    return null;
}

export function buildBody(input) {
    return {
        prints:              input.prints.map(p => ({ price: p.price, size: p.size })),
        imbalance_threshold: input.imbalance_threshold,
    };
}

// Pure-JS mirror of crates/traderview-core/src/imbalance_bar_chart.rs::compute.
export function localCompute(prints, imbalance_threshold) {
    const out = [];
    if (!Array.isArray(prints) || prints.length === 0) return out;
    if (!Number.isFinite(imbalance_threshold) || imbalance_threshold <= 0) return out;
    for (const p of prints) {
        if (!Number.isFinite(p.price) || !Number.isFinite(p.size) || p.price <= 0 || p.size < 0) return out;
    }
    let open = prints[0].price;
    let high = prints[0].price;
    let low = prints[0].price;
    let volume = 0;
    let imbalance = 0;
    let tick_count = 0;
    let prev_sign = 1;
    let prev_price = prints[0].price;
    for (const p of prints) {
        if (tick_count === 0) {
            open = p.price;
            high = p.price;
            low = p.price;
            volume = 0;
            imbalance = 0;
        }
        let sign;
        if (p.price > prev_price)      sign = 1;
        else if (p.price < prev_price) sign = -1;
        else                            sign = prev_sign;
        prev_sign = sign;
        prev_price = p.price;
        if (p.price > high) high = p.price;
        if (p.price < low)  low = p.price;
        volume += p.size;
        imbalance += sign * p.size;
        tick_count++;
        if (Math.abs(imbalance) >= imbalance_threshold) {
            out.push({ open, high, low, close: p.price, volume, imbalance, tick_count });
            tick_count = 0;
        }
    }
    return out;
}

// Parse "price size" per line.
export function parsePrintsBlob(blob) {
    const out = { prints: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: t('common.parse.input_must_be_string') });
        return out;
    }
    const lines = blob.split('\n');
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i].split('#')[0].trim();
        if (!raw) continue;
        const toks = raw.split(/[\s,]+/).filter(t => t.length > 0);
        if (toks.length !== 2) {
            out.errors.push({ line_no: i + 1, message: 'expected 2 tokens (price size)' });
            continue;
        }
        const price = Number(toks[0]);
        const size = Number(toks[1]);
        if (!Number.isFinite(price) || price <= 0) {
            out.errors.push({ line_no: i + 1, message: 'price must be positive finite' });
            continue;
        }
        if (!Number.isFinite(size) || size < 0) {
            out.errors.push({ line_no: i + 1, message: 'size must be ≥ 0 finite' });
            continue;
        }
        out.prints.push({ price, size });
    }
    return out;
}

export function printsToBlob(prints) {
    return prints.map(p => `${p.price} ${p.size}`).join('\n');
}

// Last bar's signed imbalance verdict.
export function flowBadge(bars) {
    if (!Array.isArray(bars) || bars.length === 0) return { key: 'view.imb_bar.badge.no_signal', cls: '' };
    const last = bars[bars.length - 1];
    if (last.imbalance > 0)  return { key: 'view.imb_bar.badge.buy_dominant',  cls: 'pos' };
    if (last.imbalance < 0)  return { key: 'view.imb_bar.badge.sell_dominant', cls: 'neg' };
    return { key: 'view.imb_bar.badge.balanced', cls: '' };
}

// Net buy/sell tilt across all emitted bars.
export function tiltBadge(bars) {
    if (!Array.isArray(bars) || bars.length === 0) return { key: 'view.imb_bar.tilt.unknown', cls: '' };
    let buy = 0, sell = 0;
    for (const b of bars) {
        if (b.imbalance > 0) buy++;
        else if (b.imbalance < 0) sell++;
    }
    if (buy === 0 && sell === 0) return { key: 'view.imb_bar.tilt.balanced', cls: '' };
    const ratio = buy / (buy + sell);
    if (ratio >= 0.75) return { key: 'view.imb_bar.tilt.strong_buy',  cls: 'pos' };
    if (ratio >= 0.55) return { key: 'view.imb_bar.tilt.buy_tilt',   cls: 'pos' };
    if (ratio <= 0.25) return { key: 'view.imb_bar.tilt.strong_sell', cls: 'neg' };
    if (ratio <= 0.45) return { key: 'view.imb_bar.tilt.sell_tilt',  cls: 'neg' };
    return { key: 'view.imb_bar.tilt.balanced', cls: '' };
}

// Aggregate stats.
export function summarize(bars) {
    if (!Array.isArray(bars) || bars.length === 0) {
        return { count: 0, total_volume: 0, total_ticks: 0, buy_bars: 0, sell_bars: 0,
                 abs_imbalance_sum: 0, max_abs_imb: NaN, last_close: NaN };
    }
    let sumV = 0, sumT = 0, buy = 0, sell = 0, sumAbs = 0, maxAbs = -Infinity;
    for (const b of bars) {
        sumV += b.volume;
        sumT += b.tick_count;
        if (b.imbalance > 0) buy++;
        else if (b.imbalance < 0) sell++;
        const a = Math.abs(b.imbalance);
        sumAbs += a;
        if (a > maxAbs) maxAbs = a;
    }
    return {
        count: bars.length,
        total_volume: sumV,
        total_ticks: sumT,
        buy_bars: buy,
        sell_bars: sell,
        abs_imbalance_sum: sumAbs,
        max_abs_imb: Number.isFinite(maxAbs) ? maxAbs : NaN,
        last_close: bars[bars.length - 1].close,
    };
}

// Synthetic demos.
export function makeDemoInput(kind = 'uptrend') {
    switch (kind) {
        case 'uptrend':       return uptick(100, 30, 0.1, 10, 100);    // 10 upticks → +100 imb
        case 'downtrend':     return downtick(110, 30, 0.1, 10, 100);
        case 'balanced':      return alternate(100, 40, 0.5, 10, 100);
        case 'flat':          return flat(100, 20, 10, 100);
        case 'aggressive-buy': {
            // 5 upticks → +50; bigger threshold → fewer bars.
            return uptick(100, 50, 0.05, 100, 1000);
        }
        case 'climax-burst': {
            const a = uptick(100, 20, 0.05, 10, 100).prints;
            const b = downtick(102, 20, 0.05, 10, 100).prints;
            const c = uptick(101, 30, 0.1, 20, 100).prints;
            return { prints: [...a, ...b, ...c], imbalance_threshold: 100 };
        }
        case 'partial-trail': {
            // 2 ticks × 10 = 20 imbalance, threshold 100 → no bar.
            return { prints: [{ price: 100, size: 10 }, { price: 101, size: 10 }],
                     imbalance_threshold: 100 };
        }
        case 'tie-runs': {
            // Repeated equal prices → sign comes from prior_sign (test that path).
            const prints = [
                { price: 100, size: 10 },
                { price: 101, size: 20 },
                { price: 101, size: 30 },   // tie → +1 prior_sign
                { price: 101, size: 40 },   // tie → +1 prior_sign
            ];
            return { prints, imbalance_threshold: 50 };
        }
        default: return makeDemoInput('uptrend');
    }
}

function uptick(start, n, step, size, threshold) {
    const prints = [];
    for (let i = 0; i < n; i++) prints.push({ price: round(start + i * step), size });
    return { prints, imbalance_threshold: threshold };
}

function downtick(start, n, step, size, threshold) {
    const prints = [];
    for (let i = 0; i < n; i++) prints.push({ price: round(start - i * step), size });
    return { prints, imbalance_threshold: threshold };
}

function alternate(center, n, amp, size, threshold) {
    const prints = [];
    for (let i = 0; i < n; i++) prints.push({ price: round(center + (i % 2 === 0 ? amp : -amp)), size });
    return { prints, imbalance_threshold: threshold };
}

function flat(price, n, size, threshold) {
    const prints = [];
    for (let i = 0; i < n; i++) prints.push({ price, size });
    return { prints, imbalance_threshold: threshold };
}

function round(v) { return Math.round(v * 10000) / 10000; }

export function fmtUSD(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return '$' + v.toFixed(d);
}

export function fmtSigned(v, d = 0) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + v.toFixed(d);
}

export function fmtMove(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '-') + '$' + Math.abs(v).toFixed(d);
}

export function fmtInt(v) {
    if (!Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}

export function fmtVol(v) {
    if (!Number.isFinite(v)) return '—';
    if (Math.abs(v) >= 1e6) return (v / 1e6).toFixed(2) + 'M';
    if (Math.abs(v) >= 1e3) return (v / 1e3).toFixed(2) + 'k';
    return v.toFixed(0);
}
