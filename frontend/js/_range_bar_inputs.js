// Range-bar aggregator helpers — emits a new bar each time the
// running high-low spread reaches target_range.
//
// Backend body: { prints: [{price, size}, ...], target_range }
// Returns: RangeBar[] = { open, high, low, close, volume, tick_count }
//
// Partial trailing bars are NOT emitted (matches Rust contract).

export const DEFAULT_INPUTS = {
    prints: [],
    target_range: 5.0,
};

export function validateInputs(input) {
    if (!Array.isArray(input.prints))                          return 'prints must be an array';
    for (let i = 0; i < input.prints.length; i++) {
        const p = input.prints[i];
        if (!p || typeof p !== 'object')                       return `prints[${i}] must be an object`;
        if (!Number.isFinite(p.price))                         return `prints[${i}].price not finite`;
        if (p.price <= 0)                                      return `prints[${i}].price must be > 0`;
        if (!Number.isFinite(p.size))                          return `prints[${i}].size not finite`;
        if (p.size < 0)                                        return `prints[${i}].size must be ≥ 0`;
    }
    if (!Number.isFinite(input.target_range))                  return 'target_range must be finite';
    if (input.target_range <= 0)                               return 'target_range must be > 0';
    return null;
}

export function buildBody(input) {
    return {
        prints: input.prints.map(p => ({ price: p.price, size: p.size })),
        target_range: input.target_range,
    };
}

// Pure-JS mirror of crates/traderview-core/src/range_bar_chart.rs::compute.
// Trailing partial bar is NOT emitted (matches Rust).
export function localCompute(prints, target_range) {
    const out = [];
    if (!Array.isArray(prints) || prints.length === 0) return out;
    if (!Number.isFinite(target_range) || target_range <= 0) return out;
    for (const p of prints) {
        if (!Number.isFinite(p.price) || !Number.isFinite(p.size) || p.price <= 0 || p.size < 0) return out;
    }
    let open = prints[0].price;
    let high = prints[0].price;
    let low = prints[0].price;
    let volume = prints[0].size;
    let tick_count = 1;
    for (let i = 1; i < prints.length; i++) {
        const p = prints[i];
        if (p.price > high) high = p.price;
        if (p.price < low)  low = p.price;
        volume += p.size;
        tick_count += 1;
        if (high - low >= target_range) {
            out.push({ open, high, low, close: p.price, volume, tick_count });
            open = p.price;
            high = p.price;
            low = p.price;
            volume = 0;
            tick_count = 0;
        }
    }
    return out;
}

// Parse "price size" per line, blanks + # comments ignored.
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
    if (!Array.isArray(bars) || bars.length === 0) return { key: 'view.range_bar.badge.flat', cls: '' };
    const last = bars[bars.length - 1];
    if (last.close > last.open) return { key: 'view.range_bar.badge.uptrend',   cls: 'pos' };
    if (last.close < last.open) return { key: 'view.range_bar.badge.downtrend', cls: 'neg' };
    return { key: 'view.range_bar.badge.flat', cls: '' };
}

// Activity verdict — bars-per-print density.
export function activityBadge(bars, printCount) {
    if (!Array.isArray(bars) || !Number.isFinite(printCount) || printCount <= 0)
        return { key: 'view.range_bar.activity.unknown', cls: '' };
    const density = bars.length / printCount;
    if (density === 0)         return { key: 'view.range_bar.activity.quiet',     cls: '' };
    if (density < 0.05)        return { key: 'view.range_bar.activity.normal',    cls: '' };
    if (density < 0.2)         return { key: 'view.range_bar.activity.active',    cls: 'pos' };
    return { key: 'view.range_bar.activity.volatile', cls: 'neg' };
}

// Aggregate stats.
export function summarize(bars) {
    if (!Array.isArray(bars) || bars.length === 0) {
        return { count: 0, total_volume: 0, total_ticks: 0,
                 avg_volume: NaN, avg_ticks: NaN,
                 ups: 0, downs: 0, doji: 0, last_close: NaN };
    }
    let sumV = 0, sumT = 0, ups = 0, downs = 0, doji = 0;
    for (const b of bars) {
        sumV += b.volume;
        sumT += b.tick_count;
        if (b.close > b.open)      ups++;
        else if (b.close < b.open) downs++;
        else                       doji++;
    }
    return {
        count: bars.length,
        total_volume: sumV,
        total_ticks: sumT,
        avg_volume: sumV / bars.length,
        avg_ticks: sumT / bars.length,
        ups, downs, doji,
        last_close: bars[bars.length - 1].close,
    };
}

// Synthetic demos. Deterministic walks driven by a small LCG.
export function makeDemoInput(kind = 'uptrend') {
    switch (kind) {
        case 'uptrend':       return uptrend(100, 50, 0.5, 5.0);
        case 'downtrend':     return downtrend(110, 50, 0.5, 5.0);
        case 'chop':          return chop(100, 100, 2.0, 5.0);
        case 'flat':          return flat(100, 50, 5.0);
        case 'big-prints':    return bigPrints(100, 20, 5.0);
        case 'small-range':   return { prints: tinyBounce(100, 20, 0.05), target_range: 1.0 };
        case 'wide-range':    return uptrend(100, 50, 1.0, 10.0);
        case 'noisy-walk':    return noisyWalk(100, 200, 1n, 5.0);
        default:              return makeDemoInput('uptrend');
    }
}

function uptrend(start, n, step, target_range) {
    const prints = [];
    for (let i = 0; i < n; i++) prints.push({ price: round(start + i * step), size: 10 });
    return { prints, target_range };
}

function downtrend(start, n, step, target_range) {
    const prints = [];
    for (let i = 0; i < n; i++) prints.push({ price: round(start - i * step), size: 10 });
    return { prints, target_range };
}

function chop(center, n, amp, target_range) {
    const prints = [];
    for (let i = 0; i < n; i++) prints.push({ price: round(center + Math.sin(i * 0.4) * amp + Math.cos(i * 1.1) * amp * 0.5), size: 10 });
    return { prints, target_range };
}

function flat(price, n, target_range) {
    const prints = [];
    for (let i = 0; i < n; i++) prints.push({ price, size: 10 });
    return { prints, target_range };
}

function bigPrints(start, n, target_range) {
    const prints = [];
    for (let i = 0; i < n; i++) prints.push({ price: round(start + i), size: 1000 + i * 50 });
    return { prints, target_range };
}

function tinyBounce(center, n, amp) {
    const prints = [];
    for (let i = 0; i < n; i++) prints.push({ price: round(center + (i % 2 === 0 ? amp : -amp)), size: 10 });
    return prints;
}

function noisyWalk(start, n, seed, target_range) {
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
    return { prints, target_range };
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
