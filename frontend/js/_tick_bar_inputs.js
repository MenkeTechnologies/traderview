// Tick-bar aggregator helpers — emits one OHLC bar per N prints (time
// ignored).
//
// Backend body: { prints: [{price, size}, ...], ticks_per_bar }
// Returns: TickBar[] = { open, high, low, close, volume, tick_count }
//
// Trailing partial bars are NOT emitted (matches Rust contract).

export const DEFAULT_INPUTS = {
    prints: [],
    ticks_per_bar: 10,
};

export function validateInputs(input) {
    if (!Array.isArray(input.prints))                            return 'prints must be an array';
    for (let i = 0; i < input.prints.length; i++) {
        const p = input.prints[i];
        if (!p || typeof p !== 'object')                         return `prints[${i}] must be an object`;
        if (!Number.isFinite(p.price))                           return `prints[${i}].price not finite`;
        if (p.price <= 0)                                        return `prints[${i}].price must be > 0`;
        if (!Number.isFinite(p.size))                            return `prints[${i}].size not finite`;
        if (p.size < 0)                                          return `prints[${i}].size must be ≥ 0`;
    }
    if (!Number.isInteger(input.ticks_per_bar))                  return 'ticks_per_bar must be an integer';
    if (input.ticks_per_bar < 1)                                 return 'ticks_per_bar must be ≥ 1';
    return null;
}

export function buildBody(input) {
    return {
        prints:        input.prints.map(p => ({ price: p.price, size: p.size })),
        ticks_per_bar: input.ticks_per_bar,
    };
}

// Pure-JS mirror of crates/traderview-core/src/tick_bar_chart.rs::compute.
export function localCompute(prints, ticks_per_bar) {
    const out = [];
    if (!Array.isArray(prints) || prints.length === 0) return out;
    if (!Number.isInteger(ticks_per_bar) || ticks_per_bar < 1) return out;
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
        if (tick_count >= ticks_per_bar) {
            out.push({ open, high, low, close: p.price, volume, tick_count });
            tick_count = 0;
        }
    }
    return out;
}

// Parse "price size" per line — same shape as range-bar parser.
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

// Trend verdict from last bar (close vs open).
export function trendBadge(bars) {
    if (!Array.isArray(bars) || bars.length === 0) return { key: 'view.tick_bar.badge.flat', cls: '' };
    const last = bars[bars.length - 1];
    if (last.close > last.open) return { key: 'view.tick_bar.badge.uptrend',   cls: 'pos' };
    if (last.close < last.open) return { key: 'view.tick_bar.badge.downtrend', cls: 'neg' };
    return { key: 'view.tick_bar.badge.flat', cls: '' };
}

// Coverage verdict — how much of the print stream is captured by full bars.
export function coverageBadge(bars, printCount, ticks_per_bar) {
    if (!Number.isFinite(printCount) || printCount <= 0
        || !Number.isInteger(ticks_per_bar) || ticks_per_bar <= 0)
        return { key: 'view.tick_bar.cov.unknown', cls: '' };
    const covered = bars.length * ticks_per_bar;
    const frac = covered / printCount;
    if (frac < 0.5)  return { key: 'view.tick_bar.cov.low',     cls: 'neg' };
    if (frac < 0.9)  return { key: 'view.tick_bar.cov.normal',  cls: '' };
    if (frac < 1.0)  return { key: 'view.tick_bar.cov.high',    cls: 'pos' };
    return { key: 'view.tick_bar.cov.full', cls: 'pos' };
}

// Aggregate stats.
export function summarize(bars) {
    if (!Array.isArray(bars) || bars.length === 0) {
        return { count: 0, total_volume: 0, total_ticks: 0,
                 avg_volume: NaN, avg_range: NaN,
                 ups: 0, downs: 0, doji: 0, last_close: NaN };
    }
    let sumV = 0, sumR = 0, sumT = 0, ups = 0, downs = 0, doji = 0;
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
        avg_volume: sumV / n,
        avg_range:  sumR / n,
        ups, downs, doji,
        last_close: bars[bars.length - 1].close,
    };
}

// Synthetic demos. Deterministic LCG-driven walks for stability.
export function makeDemoInput(kind = 'uptrend') {
    switch (kind) {
        case 'uptrend':       return uptrend(100, 60, 0.05, 10);
        case 'downtrend':     return downtrend(110, 60, 0.05, 10);
        case 'flat':          return flat(100, 50, 10);
        case 'noisy':         return noisyWalk(100, 200, 1n, 20);
        case 'small-bars':    return uptrend(100, 60, 0.1, 5);     // very small bars
        case 'large-bars':    return uptrend(100, 60, 0.1, 30);    // few bars
        case 'partial':       return uptrend(100, 23, 0.1, 10);    // 2 full + partial drop
        case 'one-tick':      return uptrend(100, 5, 1, 1);        // every tick = 1 bar
        default:              return makeDemoInput('uptrend');
    }
}

function uptrend(start, n, step, ticks_per_bar) {
    const prints = [];
    for (let i = 0; i < n; i++) prints.push({ price: round(start + i * step), size: 10 });
    return { prints, ticks_per_bar };
}

function downtrend(start, n, step, ticks_per_bar) {
    const prints = [];
    for (let i = 0; i < n; i++) prints.push({ price: round(start - i * step), size: 10 });
    return { prints, ticks_per_bar };
}

function flat(price, n, ticks_per_bar) {
    const prints = [];
    for (let i = 0; i < n; i++) prints.push({ price, size: 10 });
    return { prints, ticks_per_bar };
}

function noisyWalk(start, n, seed, ticks_per_bar) {
    const prints = [];
    let state = BigInt(7919) + seed;
    let price = start;
    for (let i = 0; i < n; i++) {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        const u = Number(state >> 32n) / 0xFFFFFFFF;
        price = round(price + (u - 0.5) * 2);
        if (price <= 0) price = round(start);
        prints.push({ price, size: 10 + Math.trunc(u * 50) });
    }
    return { prints, ticks_per_bar };
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
