// Dollar-bar aggregator helpers — emits an OHLC bar each time
// accumulated notional (Σ price × size) reaches dollars_per_bar.
//
// Backend body: { prints: [{price, size}, ...], dollars_per_bar }
// Returns: DollarBar[] = { open, high, low, close, volume, notional, tick_count }
//
// Trailing partial bars are NOT emitted (matches Rust contract).

import { t } from './i18n.js';

export const DEFAULT_INPUTS = {
    prints: [],
    dollars_per_bar: 100_000,
};

export function validateInputs(input) {
    if (!Array.isArray(input.prints))                          return t('view.dollar_bar.validate.prints_array');
    for (let i = 0; i < input.prints.length; i++) {
        const p = input.prints[i];
        if (!p || typeof p !== 'object')                       return t('view.dollar_bar.validate.print_object', { i });
        if (!Number.isFinite(p.price))                         return t('view.dollar_bar.validate.price_finite', { i });
        if (p.price <= 0)                                      return t('view.dollar_bar.validate.price_positive', { i });
        if (!Number.isFinite(p.size))                          return t('view.dollar_bar.validate.size_finite', { i });
        if (p.size < 0)                                        return t('view.dollar_bar.validate.size_negative', { i });
    }
    if (!Number.isFinite(input.dollars_per_bar))               return t('view.dollar_bar.validate.dpb_finite');
    if (input.dollars_per_bar <= 0)                            return t('view.dollar_bar.validate.dpb_positive');
    return null;
}

export function buildBody(input) {
    return {
        prints:          input.prints.map(p => ({ price: p.price, size: p.size })),
        dollars_per_bar: input.dollars_per_bar,
    };
}

// Pure-JS mirror of crates/traderview-core/src/dollar_bar_chart.rs::compute.
export function localCompute(prints, dollars_per_bar) {
    const out = [];
    if (!Array.isArray(prints) || prints.length === 0) return out;
    if (!Number.isFinite(dollars_per_bar) || dollars_per_bar <= 0) return out;
    for (const p of prints) {
        if (!Number.isFinite(p.price) || !Number.isFinite(p.size) || p.price <= 0 || p.size < 0) return out;
    }
    let open = prints[0].price;
    let high = prints[0].price;
    let low = prints[0].price;
    let volume = 0;
    let notional = 0;
    let tick_count = 0;
    for (const p of prints) {
        if (tick_count === 0) {
            open = p.price;
            high = p.price;
            low = p.price;
            volume = 0;
            notional = 0;
        }
        if (p.price > high) high = p.price;
        if (p.price < low)  low = p.price;
        volume += p.size;
        notional += p.price * p.size;
        tick_count++;
        if (notional >= dollars_per_bar) {
            out.push({ open, high, low, close: p.price, volume, notional, tick_count });
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
            out.errors.push({ line_no: i + 1, message: t('common.parse.expected_price_size') });
            continue;
        }
        const price = Number(toks[0]);
        const size = Number(toks[1]);
        if (!Number.isFinite(price) || price <= 0) {
            out.errors.push({ line_no: i + 1, message: t('common.parse.price_must_be_positive') });
            continue;
        }
        if (!Number.isFinite(size) || size < 0) {
            out.errors.push({ line_no: i + 1, message: t('common.parse.size_must_be_non_negative') });
            continue;
        }
        out.prints.push({ price, size });
    }
    return out;
}

export function printsToBlob(prints) {
    return prints.map(p => `${p.price} ${p.size}`).join('\n');
}

export function trendBadge(bars) {
    if (!Array.isArray(bars) || bars.length === 0) return { key: 'view.dollar_bar.badge.flat', cls: '' };
    const last = bars[bars.length - 1];
    if (last.close > last.open) return { key: 'view.dollar_bar.badge.uptrend',   cls: 'pos' };
    if (last.close < last.open) return { key: 'view.dollar_bar.badge.downtrend', cls: 'neg' };
    return { key: 'view.dollar_bar.badge.flat', cls: '' };
}

// Coverage = bars × dollars_per_bar / total notional.
export function coverageBadge(bars, totalNotional, dollars_per_bar) {
    if (!Number.isFinite(totalNotional) || totalNotional <= 0
        || !Number.isFinite(dollars_per_bar) || dollars_per_bar <= 0)
        return { key: 'view.dollar_bar.cov.unknown', cls: '' };
    const covered = bars.reduce((s, b) => s + b.notional, 0);
    const frac = covered / totalNotional;
    if (frac < 0.5) return { key: 'view.dollar_bar.cov.low',    cls: 'neg' };
    if (frac < 0.9) return { key: 'view.dollar_bar.cov.normal', cls: '' };
    if (frac < 1.0) return { key: 'view.dollar_bar.cov.high',   cls: 'pos' };
    return { key: 'view.dollar_bar.cov.full', cls: 'pos' };
}

// Aggregate stats.
export function summarize(bars) {
    if (!Array.isArray(bars) || bars.length === 0) {
        return { count: 0, total_notional: 0, total_volume: 0, total_ticks: 0,
                 avg_ticks: NaN, avg_range: NaN, avg_notional: NaN,
                 ups: 0, downs: 0, doji: 0, last_close: NaN };
    }
    let sumN = 0, sumV = 0, sumT = 0, sumR = 0, ups = 0, downs = 0, doji = 0;
    for (const b of bars) {
        sumN += b.notional;
        sumV += b.volume;
        sumT += b.tick_count;
        sumR += (b.high - b.low);
        if (b.close > b.open)      ups++;
        else if (b.close < b.open) downs++;
        else                       doji++;
    }
    const n = bars.length;
    return {
        count: n,
        total_notional: sumN,
        total_volume: sumV,
        total_ticks: sumT,
        avg_ticks: sumT / n,
        avg_range: sumR / n,
        avg_notional: sumN / n,
        ups, downs, doji,
        last_close: bars[bars.length - 1].close,
    };
}

// Demos.
export function makeDemoInput(kind = 'mid-cap-uptrend') {
    switch (kind) {
        case 'mid-cap-uptrend':   return walk(100, 50, 0.5, 200, 1n, 100_000);
        case 'mid-cap-downtrend': return walkDown(110, 50, 0.5, 200, 2n, 100_000);
        case 'flat-market':       return flat(100, 30, 500, 100_000);
        case 'penny-stock': {
            // Low price needs many shares to hit notional.
            return walk(3, 80, 0.05, 5000, 3n, 50_000);
        }
        case 'large-cap': {
            // High price, fewer shares per bar.
            return walk(450, 50, 0.5, 100, 4n, 200_000);
        }
        case 'partial-trail': {
            // 3 prints × 100×100 = 30_000 only — well below 50k target → 0 bars.
            const prints = Array.from({ length: 3 }, () => ({ price: 100, size: 100 }));
            return { prints, dollars_per_bar: 50_000 };
        }
        case 'spiky-notional': {
            // Mix of small + occasional huge prints.
            return spiky(100, 30, 5n, 50_000);
        }
        case 'noisy-walk': {
            return walk(100, 200, 0.5, 200, 21n, 25_000);
        }
        default: return makeDemoInput('mid-cap-uptrend');
    }
}

function walk(start, n, step, base_size, seed, dollars_per_bar) {
    const prints = [];
    let state = BigInt(7919) + seed;
    for (let i = 0; i < n; i++) {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        const u = Number(state >> 32n) / 0xFFFFFFFF;
        prints.push({
            price: round(start + i * step + (u - 0.5) * 0.1),
            size:  round(base_size + (u - 0.5) * base_size * 0.4),
        });
    }
    return { prints, dollars_per_bar };
}

function walkDown(start, n, step, base_size, seed, dollars_per_bar) {
    const prints = [];
    let state = BigInt(7919) + seed;
    for (let i = 0; i < n; i++) {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        const u = Number(state >> 32n) / 0xFFFFFFFF;
        prints.push({
            price: round(start - i * step + (u - 0.5) * 0.1),
            size:  round(base_size + (u - 0.5) * base_size * 0.4),
        });
    }
    return { prints, dollars_per_bar };
}

function flat(price, n, size, dollars_per_bar) {
    const prints = [];
    for (let i = 0; i < n; i++) prints.push({ price, size });
    return { prints, dollars_per_bar };
}

function spiky(center, n, seed, dollars_per_bar) {
    const prints = [];
    let state = BigInt(7919) + seed;
    for (let i = 0; i < n; i++) {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        const u = Number(state >> 32n) / 0xFFFFFFFF;
        const size = u > 0.85 ? round(3000 + u * 5000) : round(50 + u * 100);
        prints.push({ price: round(center + Math.sin(i * 0.4) * 2), size });
    }
    return { prints, dollars_per_bar };
}

function round(v) { return Math.round(v * 10000) / 10000; }

export function fmtUSD(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return '$' + v.toFixed(d);
}

export function fmtMove(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '-') + '$' + Math.abs(v).toFixed(d);
}

export function fmtNum(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
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

export function fmtNotional(v, d = 0) {
    if (!Number.isFinite(v)) return '—';
    if (Math.abs(v) >= 1e9) return '$' + (v / 1e9).toFixed(2) + 'B';
    if (Math.abs(v) >= 1e6) return '$' + (v / 1e6).toFixed(2) + 'M';
    if (Math.abs(v) >= 1e3) return '$' + (v / 1e3).toFixed(2) + 'k';
    return '$' + v.toFixed(d);
}
