// Order Flow Classify/Aggregate helpers shared by view + vitest.
//
// Backend body shape: { ticks: [{price, volume, bid, ask}, ...] }.
// Two endpoints share the body — `classify` returns per-tick sides,
// `aggregate` returns the rolled-up imbalance scalars. View calls both
// in parallel.

const TOKEN_DELIM = /[\s,]+/;

// Parses four-token-per-line: "price volume bid ask".
export function parseTickBlob(text) {
    const ticks = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { ticks, errors: [{ line_no: 0, raw: '', message: 'input not a string' }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = raw.trim();
        if (!s || s.startsWith('#')) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 4) {
            errors.push({ line_no: i + 1, raw, message: `expected 4 tokens (price volume bid ask), got ${parts.length}` });
            continue;
        }
        const price = Number(parts[0]);
        const volume = Number(parts[1]);
        const bid = Number(parts[2]);
        const ask = Number(parts[3]);
        if (!Number.isFinite(price) || price <= 0) {
            errors.push({ line_no: i + 1, raw, message: `price must be > 0` });
            continue;
        }
        if (!Number.isFinite(volume) || volume <= 0) {
            errors.push({ line_no: i + 1, raw, message: `volume must be > 0` });
            continue;
        }
        if (!Number.isFinite(bid) || bid <= 0) {
            errors.push({ line_no: i + 1, raw, message: `bid must be > 0` });
            continue;
        }
        if (!Number.isFinite(ask) || ask < bid) {
            errors.push({ line_no: i + 1, raw, message: `ask must be ≥ bid` });
            continue;
        }
        ticks.push({ price, volume, bid, ask });
    }
    return { ticks, errors };
}

export function validateInputs(ticks) {
    if (!Array.isArray(ticks) || ticks.length < 5) return 'need at least 5 ticks';
    return null;
}

export function buildBody(ticks) {
    return { ticks };
}

// Maps the backend snake_case Side enum to UI label + class. Buy → cyan
// (pos), Sell → red (neg), Uncertain → muted neutral.
export function sideBadge(side) {
    switch (side) {
        case 'buy':       return { label: 'BUY',       cls: 'pos' };
        case 'sell':      return { label: 'SELL',      cls: 'neg' };
        case 'uncertain':
        default:          return { label: 'UNCERTAIN', cls: '' };
    }
}

// Splits classified ticks into 3 parallel cumulative-volume arrays
// indexed by tick number. Buy values accumulate signed-positive, sell
// values signed-negative, uncertain stays at last seen — gives a clean
// cumulative-net-volume curve for charting.
export function cumulativeFlow(classified) {
    const xs = [], buy = [], sell = [], net = [];
    let cumBuy = 0, cumSell = 0;
    if (!Array.isArray(classified)) return { xs, buy, sell, net };
    for (let i = 0; i < classified.length; i++) {
        const c = classified[i];
        if (c && c.side === 'buy')  cumBuy  += Number(c.volume) || 0;
        if (c && c.side === 'sell') cumSell += Number(c.volume) || 0;
        xs.push(i);
        buy.push(cumBuy);
        sell.push(-cumSell);   // negate for divergent display
        net.push(cumBuy - cumSell);
    }
    return { xs, buy, sell, net };
}

// Sum-of-volumes-by-side from already-classified ticks. Mirrors backend
// aggregate logic locally for sanity-check against the network response.
export function localAggregate(classified) {
    let buy = 0, sell = 0, uncertain = 0;
    if (!Array.isArray(classified)) return { buy, sell, uncertain, net: 0, imbalance: 0 };
    for (const c of classified) {
        if (!c || !Number.isFinite(c.volume)) continue;
        if      (c.side === 'buy')       buy += c.volume;
        else if (c.side === 'sell')      sell += c.volume;
        else if (c.side === 'uncertain') uncertain += c.volume;
    }
    const net = buy - sell;
    const denom = buy + sell;
    const imbalance = denom > 0 ? net / denom : 0;
    return { buy, sell, uncertain, net, imbalance };
}

// Deterministic 400-tick demo. Volume-weighted toward aggressive BUY:
// most prints land at ask, a few mid-spread, a handful at bid.
export function makeDemoTicks(seed = 1) {
    let s = seed;
    const rand = () => { s = (s * 1664525 + 1013904223) | 0; return ((s >>> 0) / 0xffffffff); };
    const out = new Array(400);
    let mid = 100;
    for (let i = 0; i < 400; i++) {
        mid = Math.max(0.5, mid + (rand() - 0.45) * 0.05);
        const halfSpread = 0.01;
        const bid = mid - halfSpread;
        const ask = mid + halfSpread;
        // Volume-weighted aggressor classification: 65% buy-at-ask, 20%
        // sell-at-bid, 15% mid-spread (uncertain). Tuned so the at-ask /
        // at-bid ratio cleanly exceeds 2× regardless of LCG seed jitter.
        const u = rand();
        let price;
        if      (u < 0.65) price = ask;
        else if (u < 0.85) price = bid;
        else               price = mid;
        const volume = Math.round(100 + rand() * 400);
        out[i] = {
            price: Number(price.toFixed(4)),
            volume,
            bid: Number(bid.toFixed(4)),
            ask: Number(ask.toFixed(4)),
        };
    }
    return out;
}

export function fmtN(v) {
    if (!Number.isFinite(v)) return '—';
    return v.toLocaleString('en-US');
}

export function fmtImbalance(v) {
    if (!Number.isFinite(v)) return '—';
    const sign = v >= 0 ? '+' : '';
    return sign + v.toFixed(4);
}
