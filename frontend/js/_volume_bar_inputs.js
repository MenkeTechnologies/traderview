// Volume-bar aggregator helpers — emits an OHLC bar each time
// accumulated volume reaches volume_per_bar.
//
// Backend body: { prints: [{price, size}, ...], volume_per_bar }
// Returns: VolumeBar[] = { open, high, low, close, volume, tick_count }
//
// Trailing partial bars are NOT emitted (matches Rust contract).

import { t } from './i18n.js';

export const DEFAULT_INPUTS = {
    prints: [],
    volume_per_bar: 1000.0,
};

export function validateInputs(input) {
    if (!Array.isArray(input.prints))                          return t('view.volume_bar.validate.prints_array');
    for (let i = 0; i < input.prints.length; i++) {
        const p = input.prints[i];
        if (!p || typeof p !== 'object')                       return t('view.volume_bar.validate.print_object', { i });
        if (!Number.isFinite(p.price))                         return t('view.volume_bar.validate.price_finite', { i });
        if (p.price <= 0)                                      return t('view.volume_bar.validate.price_positive', { i });
        if (!Number.isFinite(p.size))                          return t('view.volume_bar.validate.size_finite', { i });
        if (p.size < 0)                                        return t('view.volume_bar.validate.size_negative', { i });
    }
    if (!Number.isFinite(input.volume_per_bar))                return t('view.volume_bar.validate.vpb_finite');
    if (input.volume_per_bar <= 0)                             return t('view.volume_bar.validate.vpb_positive');
    return null;
}

export function buildBody(input) {
    return {
        prints:         input.prints.map(p => ({ price: p.price, size: p.size })),
        volume_per_bar: input.volume_per_bar,
    };
}

// Pure-JS mirror of crates/traderview-core/src/volume_bar_chart.rs::compute.
export function localCompute(prints, volume_per_bar) {
    const out = [];
    if (!Array.isArray(prints) || prints.length === 0) return out;
    if (!Number.isFinite(volume_per_bar) || volume_per_bar <= 0) return out;
    for (const p of prints) {
        if (!Number.isFinite(p.price) || !Number.isFinite(p.size) || p.price <= 0 || p.size < 0) return out;
    }
    let open = prints[0].price;
    let high = prints[0].price;
    let low = prints[0].price;
    let volume = 0;
    let tick_count = 0;
    for (const p of prints) {
        if (tick_count === 0) {
            open = p.price;
            high = p.price;
            low = p.price;
            volume = 0;
        }
        if (p.price > high) high = p.price;
        if (p.price < low)  low = p.price;
        volume += p.size;
        tick_count++;
        if (volume >= volume_per_bar) {
            out.push({ open, high, low, close: p.price, volume, tick_count });
            tick_count = 0;
        }
    }
    return out;
}

// Parse "price size" per line.
export function parsePrintsBlob(blob) {
    const out = { prints: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: 'input must be a string' });
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

export function trendBadge(bars) {
    if (!Array.isArray(bars) || bars.length === 0) return { key: 'view.vol_bar.badge.flat', cls: '' };
    const last = bars[bars.length - 1];
    if (last.close > last.open) return { key: 'view.vol_bar.badge.uptrend',   cls: 'pos' };
    if (last.close < last.open) return { key: 'view.vol_bar.badge.downtrend', cls: 'neg' };
    return { key: 'view.vol_bar.badge.flat', cls: '' };
}

// Coverage = bars × volume_per_bar / total print volume.
export function coverageBadge(bars, totalVolume, volume_per_bar) {
    if (!Number.isFinite(totalVolume) || totalVolume <= 0
        || !Number.isFinite(volume_per_bar) || volume_per_bar <= 0)
        return { key: 'view.vol_bar.cov.unknown', cls: '' };
    const covered = bars.reduce((s, b) => s + b.volume, 0);
    const frac = covered / totalVolume;
    if (frac < 0.5) return { key: 'view.vol_bar.cov.low',    cls: 'neg' };
    if (frac < 0.9) return { key: 'view.vol_bar.cov.normal', cls: '' };
    if (frac < 1.0) return { key: 'view.vol_bar.cov.high',   cls: 'pos' };
    return { key: 'view.vol_bar.cov.full', cls: 'pos' };
}

// Aggregate stats.
export function summarize(bars) {
    if (!Array.isArray(bars) || bars.length === 0) {
        return { count: 0, total_volume: 0, total_ticks: 0,
                 avg_ticks: NaN, avg_range: NaN,
                 ups: 0, downs: 0, doji: 0, last_close: NaN };
    }
    let sumV = 0, sumT = 0, sumR = 0, ups = 0, downs = 0, doji = 0;
    for (const b of bars) {
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
        total_volume: sumV,
        total_ticks: sumT,
        avg_ticks: sumT / n,
        avg_range:  sumR / n,
        ups, downs, doji,
        last_close: bars[bars.length - 1].close,
    };
}

// Demos.
export function makeDemoInput(kind = 'uptrend-large') {
    switch (kind) {
        case 'uptrend-large':   return walk(100, 50, 0.2, 200, 1n, 1000);
        case 'downtrend-large': return walkDown(110, 50, 0.2, 200, 2n, 1000);
        case 'flat-volume':     return flatVol(100, 50, 200, 1000);
        case 'spiky-volume':    return spiky(100, 30, 5n, 1000);
        case 'tiny-target':     return walk(100, 30, 0.1, 100, 7n, 50);
        case 'huge-target':     return walk(100, 50, 0.1, 100, 11n, 5000);
        case 'partial-trail':   return partialTrail();
        case 'noisy-walk':      return walk(100, 200, 0.5, 100, 21n, 500);
        default:                return makeDemoInput('uptrend-large');
    }
}

function walk(start, n, step, base_size, seed, volume_per_bar) {
    const prints = [];
    let state = BigInt(7919) + seed;
    for (let i = 0; i < n; i++) {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        const u = Number(state >> 32n) / 0xFFFFFFFF;
        prints.push({ price: round(start + i * step + (u - 0.5) * 0.05),
                      size: round(base_size + (u - 0.5) * base_size * 0.4) });
    }
    return { prints, volume_per_bar };
}

function walkDown(start, n, step, base_size, seed, volume_per_bar) {
    const prints = [];
    let state = BigInt(7919) + seed;
    for (let i = 0; i < n; i++) {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        const u = Number(state >> 32n) / 0xFFFFFFFF;
        prints.push({ price: round(start - i * step + (u - 0.5) * 0.05),
                      size: round(base_size + (u - 0.5) * base_size * 0.4) });
    }
    return { prints, volume_per_bar };
}

function flatVol(price, n, size, volume_per_bar) {
    const prints = [];
    for (let i = 0; i < n; i++) prints.push({ price, size });
    return { prints, volume_per_bar };
}

function spiky(center, n, seed, volume_per_bar) {
    const prints = [];
    let state = BigInt(7919) + seed;
    for (let i = 0; i < n; i++) {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        const u = Number(state >> 32n) / 0xFFFFFFFF;
        // Most prints small, occasional 10x.
        const size = u > 0.85 ? round(2000 + u * 2000) : round(50 + u * 100);
        prints.push({ price: round(center + Math.sin(i * 0.4) * 1.5), size });
    }
    return { prints, volume_per_bar };
}

function partialTrail() {
    // 5 prints × 200 = 1000 exactly → 1 bar. Then 2 prints × 100 = 200 partial → dropped.
    const prints = [];
    for (let i = 0; i < 5; i++) prints.push({ price: round(100 + i), size: 200 });
    for (let i = 0; i < 2; i++) prints.push({ price: round(105 + i), size: 100 });
    return { prints, volume_per_bar: 1000 };
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
