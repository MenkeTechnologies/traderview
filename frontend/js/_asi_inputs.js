// Accumulation Swing Index (Wilder) helpers.
//
// Backend body: { bars: Bar[], limit_move: f64 }
//   where Bar = { open, high, low, close }
// Returns: (number|null)[]  — cumulative ASI per bar; first bar always 0.
//
// Cumulative running sum of Wilder's Swing Index. ASI breakouts of prior
// highs/lows confirm "real" trend changes. limit_move is the market's
// max allowable per-bar price move; for equities use ~10% of prior close.

export const DEFAULT_LIMIT_MOVE = 10.0;

export const DEFAULT_INPUTS = {
    bars: [],
    limit_move: DEFAULT_LIMIT_MOVE,
};

export function validateInputs(input) {
    if (!Array.isArray(input.bars))                                return 'bars must be an array';
    if (input.bars.length === 0)                                   return 'bars cannot be empty';
    if (!Number.isFinite(input.limit_move) || input.limit_move <= 0)
                                                                    return 'limit_move must be positive finite';
    for (let i = 0; i < input.bars.length; i++) {
        const b = input.bars[i];
        if (!b)                                                    return `bars[${i}] missing`;
        if (typeof b.open !== 'number' || typeof b.high !== 'number'
            || typeof b.low !== 'number' || typeof b.close !== 'number')
                                                                    return `bars[${i}] OHLC must be numbers`;
        if (!Number.isFinite(b.open) || !Number.isFinite(b.high)
            || !Number.isFinite(b.low) || !Number.isFinite(b.close))
                                                                    return `bars[${i}] OHLC must be finite`;
        if (b.high < b.low)                                        return `bars[${i}] high < low`;
        if (b.close < b.low || b.close > b.high)                   return `bars[${i}] close outside [low, high]`;
        if (b.open  < b.low || b.open  > b.high)                   return `bars[${i}] open outside [low, high]`;
    }
    return null;
}

export function buildBody(input) {
    return {
        bars: input.bars.map(b => ({ open: b.open, high: b.high, low: b.low, close: b.close })),
        limit_move: input.limit_move,
    };
}

// Pure-JS mirror of crates/traderview-core/src/accumulation_swing_index.rs::compute.
export function localCompute(bars, limit_move) {
    const n = bars.length;
    const out = new Array(n).fill(null);
    if (n === 0 || !Number.isFinite(limit_move) || limit_move <= 0) return out;
    for (const b of bars) {
        if (!Number.isFinite(b.open) || !Number.isFinite(b.high)
            || !Number.isFinite(b.low)  || !Number.isFinite(b.close)) return out;
    }
    let asi = 0;
    out[0] = asi;
    for (let i = 1; i < n; i++) {
        const prev = bars[i - 1];
        const cur  = bars[i];
        const a = Math.abs(cur.high - prev.close);
        const b = Math.abs(cur.low  - prev.close);
        const c = Math.abs(cur.high - cur.low);
        const d = Math.abs(prev.close - prev.open);
        let r;
        if (a >= b && a >= c)      r = a - 0.5 * b + 0.25 * d;
        else if (b >= a && b >= c) r = b - 0.5 * a + 0.25 * d;
        else                        r = c + 0.25 * d;
        if (r <= 0) { out[i] = asi; continue; }
        const k = Math.max(a, b);
        const numerator = (cur.close - prev.close)
            + 0.5  * (cur.close - cur.open)
            + 0.25 * (prev.close - prev.open);
        const si = 50 * numerator / r * k / limit_move;
        asi += si;
        out[i] = asi;
    }
    return out;
}

// Parse "open high low close" 4-token-per-line blob.
export function parseBarsBlob(blob) {
    const out = { bars: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: 'input must be a string' });
        return out;
    }
    const lines = blob.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = raw.split('#')[0].trim();
        if (!s) continue;
        const parts = s.split(/[\s,]+/).filter(Boolean);
        if (parts.length !== 4) {
            out.errors.push({ line_no: i + 1, message: `expected 4 tokens (open high low close), got ${parts.length}` });
            continue;
        }
        const o = Number(parts[0].replace(/\$/g, ''));
        const h = Number(parts[1].replace(/\$/g, ''));
        const l = Number(parts[2].replace(/\$/g, ''));
        const c = Number(parts[3].replace(/\$/g, ''));
        if (!Number.isFinite(o) || !Number.isFinite(h) || !Number.isFinite(l) || !Number.isFinite(c)
            || o <= 0 || h <= 0 || l <= 0 || c <= 0) {
            out.errors.push({ line_no: i + 1, message: `OHLC must be positive finite` });
            continue;
        }
        if (l > h) {
            out.errors.push({ line_no: i + 1, message: `low > high` });
            continue;
        }
        if (c < l || c > h) {
            out.errors.push({ line_no: i + 1, message: `close outside [low, high]` });
            continue;
        }
        if (o < l || o > h) {
            out.errors.push({ line_no: i + 1, message: `open outside [low, high]` });
            continue;
        }
        out.bars.push({ open: o, high: h, low: l, close: c });
    }
    return out;
}

export function barsToBlob(bars) {
    return bars.map(b => `${b.open} ${b.high} ${b.low} ${b.close}`).join('\n');
}

// 5-tier trend verdict over last `lookback` ASI values (slope vs range).
export function trendBadge(asi, lookback = 10) {
    if (!Array.isArray(asi) || asi.length === 0) {
        return { key: 'view.asi.trend.unknown', cls: '' };
    }
    const tail = [];
    for (let i = asi.length - 1; i >= 0 && tail.length < lookback; i--) {
        const v = asi[i];
        if (v != null && Number.isFinite(v)) tail.unshift(v);
    }
    if (tail.length < 2) return { key: 'view.asi.trend.unknown', cls: '' };
    const slope = tail[tail.length - 1] - tail[0];
    const range = Math.max(...tail) - Math.min(...tail);
    if (range === 0)              return { key: 'view.asi.trend.flat',          cls: '' };
    if (slope > range * 0.6)       return { key: 'view.asi.trend.strong_up',    cls: 'pos' };
    if (slope > range * 0.1)       return { key: 'view.asi.trend.up',           cls: 'pos' };
    if (slope < -range * 0.6)      return { key: 'view.asi.trend.strong_down',  cls: 'neg' };
    if (slope < -range * 0.1)      return { key: 'view.asi.trend.down',         cls: 'neg' };
    return { key: 'view.asi.trend.flat', cls: '' };
}

// Wilder breakout verdict: does latest ASI exceed prior ASI extreme over last N bars?
export function breakoutBadge(asi, lookback = 20) {
    if (!Array.isArray(asi) || asi.length < 2) return { key: 'view.asi.breakout.unknown', cls: '' };
    const populated = [];
    for (const v of asi) if (v != null && Number.isFinite(v)) populated.push(v);
    if (populated.length < 2) return { key: 'view.asi.breakout.unknown', cls: '' };
    const last = populated[populated.length - 1];
    const window = populated.slice(Math.max(0, populated.length - lookback - 1), populated.length - 1);
    if (window.length === 0) return { key: 'view.asi.breakout.unknown', cls: '' };
    const mx = Math.max(...window);
    const mn = Math.min(...window);
    if (last > mx)  return { key: 'view.asi.breakout.up',   cls: 'pos' };
    if (last < mn)  return { key: 'view.asi.breakout.down', cls: 'neg' };
    return { key: 'view.asi.breakout.none', cls: '' };
}

// Sign verdict (cumulative bias).
export function biasBadge(asi_last) {
    if (asi_last == null || !Number.isFinite(asi_last)) {
        return { key: 'view.asi.bias.unknown', cls: '' };
    }
    if (asi_last > 0) return { key: 'view.asi.bias.bullish', cls: 'pos' };
    if (asi_last < 0) return { key: 'view.asi.bias.bearish', cls: 'neg' };
    return { key: 'view.asi.bias.neutral', cls: '' };
}

export function summarizeBars(bars) {
    if (!Array.isArray(bars) || bars.length === 0) {
        return { count: 0, last_close: NaN, min_low: NaN, max_high: NaN,
                 mean_close: NaN, up_bars: 0, down_bars: 0 };
    }
    let sumC = 0, mxH = -Infinity, mnL = Infinity;
    let up = 0, down = 0;
    for (const b of bars) {
        sumC += b.close;
        if (b.high > mxH) mxH = b.high;
        if (b.low  < mnL) mnL = b.low;
        if (b.close > b.open)  up++;
        else if (b.close < b.open) down++;
    }
    return {
        count: bars.length,
        last_close: bars[bars.length - 1].close,
        mean_close: sumC / bars.length,
        min_low: Number.isFinite(mnL) ? mnL : NaN,
        max_high: Number.isFinite(mxH) ? mxH : NaN,
        up_bars: up, down_bars: down,
    };
}

function lcg(seed) {
    let state = BigInt(7919) + seed;
    return () => {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(state >> 32n) / 0xFFFFFFFF;
    };
}

function mkUp(open, range, rand) {
    const noise = rand() * 0.2;
    const close = open + range * (0.4 + noise);
    const high  = Math.max(open, close) + range * 0.2;
    const low   = Math.min(open, close) - range * 0.2;
    return { open, high, low, close };
}

function mkDown(open, range, rand) {
    const noise = rand() * 0.2;
    const close = open - range * (0.4 + noise);
    const high  = Math.max(open, close) + range * 0.2;
    const low   = Math.min(open, close) - range * 0.2;
    return { open, high, low, close };
}

function mkFlat(price, range, rand) {
    const high  = price + range * (0.5 + rand() * 0.2);
    const low   = price - range * (0.5 + rand() * 0.2);
    return { open: price, high, low, close: price };
}

export function makeDemoInput(kind = 'uptrend') {
    switch (kind) {
        case 'uptrend': {
            const rand = lcg(42n);
            const bars = [];
            let price = 100;
            for (let i = 0; i < 50; i++) {
                bars.push(mkUp(price, 1.5, rand));
                price = bars[bars.length - 1].close;
            }
            return { bars, limit_move: 10 };
        }
        case 'downtrend': {
            const rand = lcg(7n);
            const bars = [];
            let price = 150;
            for (let i = 0; i < 50; i++) {
                bars.push(mkDown(price, 1.5, rand));
                price = bars[bars.length - 1].close;
            }
            return { bars, limit_move: 10 };
        }
        case 'sideways': {
            const rand = lcg(11n);
            return {
                bars: Array.from({ length: 50 }, () => mkFlat(100 + (rand() - 0.5), 1, rand)),
                limit_move: 10,
            };
        }
        case 'reversal-up': {
            const rand = lcg(13n);
            const bars = [];
            let price = 130;
            for (let i = 0; i < 25; i++) {
                bars.push(mkDown(price, 1.5, rand));
                price = bars[bars.length - 1].close;
            }
            for (let i = 0; i < 25; i++) {
                bars.push(mkUp(price, 1.5, rand));
                price = bars[bars.length - 1].close;
            }
            return { bars, limit_move: 10 };
        }
        case 'reversal-down': {
            const rand = lcg(21n);
            const bars = [];
            let price = 100;
            for (let i = 0; i < 25; i++) {
                bars.push(mkUp(price, 1.5, rand));
                price = bars[bars.length - 1].close;
            }
            for (let i = 0; i < 25; i++) {
                bars.push(mkDown(price, 1.5, rand));
                price = bars[bars.length - 1].close;
            }
            return { bars, limit_move: 10 };
        }
        case 'wide-bars': {
            // Larger ranges → larger SI magnitudes.
            const rand = lcg(33n);
            const bars = [];
            let price = 100;
            for (let i = 0; i < 50; i++) {
                bars.push(mkUp(price, 5, rand));
                price = bars[bars.length - 1].close;
            }
            return { bars, limit_move: 10 };
        }
        case 'tight-limit': {
            // Same series as uptrend but smaller limit_move → larger ASI magnitudes.
            const rand = lcg(42n);
            const bars = [];
            let price = 100;
            for (let i = 0; i < 50; i++) {
                bars.push(mkUp(price, 1.5, rand));
                price = bars[bars.length - 1].close;
            }
            return { bars, limit_move: 1 };
        }
        case 'flat-doji': {
            // open == close on every bar → SI numerator = 0 → ASI stays at 0.
            return {
                bars: Array.from({ length: 30 }, () => ({ open: 100, high: 100.5, low: 99.5, close: 100 })),
                limit_move: 10,
            };
        }
        default: return makeDemoInput('uptrend');
    }
}

export function fmtNum(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    const abs = Math.abs(v);
    if (abs >= 1e6) return (v / 1e6).toFixed(2) + 'M';
    if (abs >= 1e3) return (v / 1e3).toFixed(2) + 'k';
    return v.toFixed(d);
}

export function fmtSigned(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + v.toFixed(d);
}

export function fmtPrice(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtInt(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}
