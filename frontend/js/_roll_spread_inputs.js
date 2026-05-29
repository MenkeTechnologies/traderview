// Roll (1984) effective bid-ask spread estimator helpers.
//
// Backend body: { prices: number[], window: number }
// Returns: (number | null)[] of length prices.length.
//
// Model: spread = 2·√(−cov(Δp_t, Δp_{t−1})) when cov < 0, else 0.
// Rolling-window estimate; first window-1 bars are null (warmup).

export const DEFAULT_WINDOW = 50;

export const DEFAULT_INPUTS = {
    prices: [],
    window: DEFAULT_WINDOW,
};

export function validateInputs(input) {
    if (!Array.isArray(input.prices))                  return 'prices must be an array';
    // NaN values inside prices are tolerated — the Rust compute() skips them
    // via its is_finite() guard. Reject only non-numeric (object, string).
    for (let i = 0; i < input.prices.length; i++) {
        const v = input.prices[i];
        if (typeof v !== 'number' && v != null)        return `prices[${i}] must be a number or null`;
    }
    if (!Number.isInteger(input.window))               return 'window must be an integer';
    if (input.window < 3)                              return 'window must be ≥ 3';
    return null;
}

export function buildBody(input) {
    return {
        prices: input.prices,
        window: input.window,
    };
}

// Pure-JS mirror of crates/traderview-core/src/roll_spread.rs::compute.
// Returns same shape, including null in warmup or count<2 windows.
export function localCompute(prices, window) {
    const n = prices.length;
    const out = new Array(n).fill(null);
    if (window < 3 || n < window) return out;
    const delta = new Array(n).fill(0);
    const have = new Array(n).fill(false);
    for (let i = 1; i < n; i++) {
        if (Number.isFinite(prices[i]) && Number.isFinite(prices[i - 1])) {
            delta[i] = prices[i] - prices[i - 1];
            have[i] = true;
        }
    }
    for (let i = window - 1; i < n; i++) {
        const lo = i + 1 - window;
        let sum_now = 0, sum_prev = 0, sum_prod = 0, count = 0;
        for (let tIdx = lo + 1; tIdx <= i; tIdx++) {
            if (!have[tIdx] || !have[tIdx - 1]) continue;
            sum_now += delta[tIdx];
            sum_prev += delta[tIdx - 1];
            sum_prod += delta[tIdx] * delta[tIdx - 1];
            count++;
        }
        if (count < 2) continue;
        const mean_now = sum_now / count;
        const mean_prev = sum_prev / count;
        const cov = sum_prod / count - mean_now * mean_prev;
        if (!Number.isFinite(cov)) continue;
        const spread = cov < 0 ? 2 * Math.sqrt(-cov) : 0;
        if (Number.isFinite(spread)) out[i] = spread;
    }
    return out;
}

// Parse one price per line; blanks + # comments + repeated whitespace skipped.
export function parsePricesBlob(blob) {
    const out = { prices: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: 'input must be a string' });
        return out;
    }
    const tokens = blob
        .split('\n')
        .map(l => l.split('#')[0])
        .join(' ')
        .split(/[\s,]+/)
        .filter(t => t.length > 0);
    for (let i = 0; i < tokens.length; i++) {
        const v = Number(tokens[i]);
        if (!Number.isFinite(v)) {
            out.errors.push({ line_no: i + 1, message: `token "${tokens[i]}" not finite` });
            continue;
        }
        out.prices.push(v);
    }
    return out;
}

export function pricesToBlob(prices) {
    return prices.join('\n');
}

// Summary stats over the non-null portion of a series.
export function summarize(series) {
    const valid = [];
    for (const v of series) if (v != null && Number.isFinite(v)) valid.push(v);
    if (valid.length === 0) return { count: 0, mean: NaN, min: NaN, max: NaN, last: NaN, zero_count: 0 };
    let sum = 0, mn = Infinity, mx = -Infinity, zc = 0;
    for (const v of valid) {
        sum += v;
        if (v < mn) mn = v;
        if (v > mx) mx = v;
        if (v === 0) zc++;
    }
    return {
        count: valid.length,
        mean: sum / valid.length,
        min: mn,
        max: mx,
        last: valid[valid.length - 1],
        zero_count: zc,
    };
}

// Liquidity verdict — calibrated for equity-like spreads (price-units).
// Pair with the last price to get the bps interpretation.
export function liquidityBadge(spread, price) {
    if (spread == null || !Number.isFinite(spread)) return { key: 'view.roll.badge.unknown', cls: '' };
    if (spread === 0)                                return { key: 'view.roll.badge.trending', cls: '' };
    if (!Number.isFinite(price) || price <= 0)       return { key: 'view.roll.badge.unknown', cls: '' };
    const bps = (spread / price) * 10_000;
    if (bps < 1)   return { key: 'view.roll.badge.tight',     cls: 'pos' };
    if (bps < 5)   return { key: 'view.roll.badge.normal',    cls: '' };
    if (bps < 20)  return { key: 'view.roll.badge.wide',      cls: 'neg' };
    return { key: 'view.roll.badge.extreme', cls: 'neg' };
}

// Trend verdict — Roll's estimator collapses to 0 under monotonic flow.
export function regimeBadge(series, totalCount) {
    if (!Array.isArray(series) || series.length === 0)
        return { key: 'view.roll.regime.unknown', cls: '' };
    const s = summarize(series);
    if (s.count === 0)            return { key: 'view.roll.regime.unknown',   cls: '' };
    const zeroRate = s.zero_count / s.count;
    if (zeroRate > 0.5)           return { key: 'view.roll.regime.directional', cls: 'neg' };
    if (zeroRate > 0.15)          return { key: 'view.roll.regime.mixed',       cls: '' };
    return { key: 'view.roll.regime.random_walk', cls: 'pos' };
    void totalCount;
}

// Convert price-unit spread to basis points.
export function spreadToBps(spread, price) {
    if (!Number.isFinite(spread) || !Number.isFinite(price) || price <= 0) return NaN;
    return (spread / price) * 10_000;
}

// Synthetic demos — designed to be deterministic for tests.
export function makeDemoInput(kind = 'random-bounce') {
    switch (kind) {
        case 'random-bounce': {
            // Pseudo-random bid/ask bounce around 100 ± 0.05 (10 bps spread).
            const bid = 99.95, ask = 100.05;
            const p = [];
            let state = BigInt(7919);
            for (let i = 0; i < 1000; i++) {
                state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
                const u = Number(state >> 32n) / 0xFFFFFFFF;
                p.push(u < 0.5 ? bid : ask);
            }
            return { prices: p, window: 100 };
        }
        case 'trending': {
            // Strictly monotonic → spread = 0.
            const p = [];
            for (let i = 0; i < 200; i++) p.push(100 + i);
            return { prices: p, window: 50 };
        }
        case 'flat': {
            return { prices: Array(200).fill(100), window: 50 };
        }
        case 'tight-bounce':
            // 1 bp spread — pseudo-random 50/50 draws (LCG).
            return { prices: rngBounce(99.995, 100.005, 500, 1n), window: 80 };
        case 'wide-bounce':
            // 50 bp spread — same RNG, wider bid/ask.
            return { prices: rngBounce(99.75, 100.25, 500, 31n), window: 80 };
        case 'regime-shift': {
            // First 200 bars: tight bounce; next 200 bars: trending.
            const tight = makeDemoInput('tight-bounce').prices.slice(0, 200);
            const trend = [];
            const last = tight[tight.length - 1];
            for (let i = 0; i < 200; i++) trend.push(last + (i + 1) * 0.05);
            return { prices: [...tight, ...trend], window: 60 };
        }
        case 'spotty-nan': {
            const p = makeDemoInput('random-bounce').prices.slice(0, 500);
            p[100] = NaN;
            p[200] = NaN;
            return { prices: p, window: 100 };
        }
        case 'huge-window': {
            return { prices: Array(60).fill(100), window: 500 };
        }
        default:
            return makeDemoInput('random-bounce');
    }
}

// Deterministic 50/50 bid/ask draw via splitmix64-style LCG. Used by demos
// to ensure the demo data is regression-stable across runs/tests.
function rngBounce(bid, ask, n, seed) {
    const p = [];
    let state = BigInt(7919) + BigInt(seed);
    for (let i = 0; i < n; i++) {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        const u = Number(state >> 32n) / 0xFFFFFFFF;
        p.push(u < 0.5 ? bid : ask);
    }
    return p;
}

export function fmtUSD(v, d = 4) {
    if (!Number.isFinite(v)) return '—';
    return '$' + v.toFixed(d);
}

export function fmtBps(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d) + ' bps';
}

export function fmtNum(v, d = 6) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtInt(v) {
    if (!Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}

export function fmtPct(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}
