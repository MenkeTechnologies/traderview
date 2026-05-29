// Chaikin Oscillator helpers — MACD-style oscillator on cumulative ADL.
//
// Backend body: { bars: Bar[], fast: usize, slow: usize }
//   where Bar = { high, low, close, volume }
// Returns: (number|null)[]  — EMA(ADL, fast) − EMA(ADL, slow).

export const DEFAULT_FAST = 3;
export const DEFAULT_SLOW = 10;
export const MIN_PERIOD = 1;
export const MAX_PERIOD = 500;

export const DEFAULT_INPUTS = {
    bars: [],
    fast: DEFAULT_FAST,
    slow: DEFAULT_SLOW,
};

export function validateInputs(input) {
    if (!Array.isArray(input.bars))                       return 'bars must be an array';
    if (input.bars.length === 0)                          return 'bars cannot be empty';
    if (!Number.isInteger(input.fast) || input.fast < MIN_PERIOD || input.fast > MAX_PERIOD)
                                                           return `fast must be integer in [${MIN_PERIOD}, ${MAX_PERIOD}]`;
    if (!Number.isInteger(input.slow) || input.slow < MIN_PERIOD || input.slow > MAX_PERIOD)
                                                           return `slow must be integer in [${MIN_PERIOD}, ${MAX_PERIOD}]`;
    if (input.fast >= input.slow)                         return `fast (${input.fast}) must be < slow (${input.slow})`;
    if (input.bars.length < input.slow)                   return `need at least slow (${input.slow}) bars`;
    for (let i = 0; i < input.bars.length; i++) {
        const b = input.bars[i];
        if (!b)                                            return `bars[${i}] missing`;
        if (typeof b.high !== 'number' || typeof b.low !== 'number'
            || typeof b.close !== 'number' || typeof b.volume !== 'number')
                                                            return `bars[${i}] HLCV must be numbers`;
        if (b.volume < 0)                                  return `bars[${i}] volume cannot be negative`;
        if (Number.isFinite(b.high) && Number.isFinite(b.low) && b.high < b.low)
                                                            return `bars[${i}] high < low`;
    }
    return null;
}

export function buildBody(input) {
    return {
        bars: input.bars.map(b => ({ high: b.high, low: b.low, close: b.close, volume: b.volume })),
        fast: input.fast,
        slow: input.slow,
    };
}

// Pure-JS mirror of crates/traderview-core/src/accumulation_distribution_line.rs::compute
// (kept private here since chaikin_oscillator depends on it; the ADL view has its own).
function adlCompute(bars) {
    const n = bars.length;
    const out = new Array(n).fill(null);
    if (n === 0) return out;
    let adl = 0;
    for (let i = 0; i < n; i++) {
        const b = bars[i];
        if (!Number.isFinite(b.high) || !Number.isFinite(b.low)
            || !Number.isFinite(b.close) || !Number.isFinite(b.volume)) {
            out[i] = adl;
            continue;
        }
        const range = b.high - b.low;
        if (range > 0) {
            const mfm = ((b.close - b.low) - (b.high - b.close)) / range;
            adl += mfm * b.volume;
        }
        out[i] = adl;
    }
    return out;
}

// Pure-JS mirror of crates/traderview-core/src/chaikin_oscillator.rs::compute.
export function localCompute(bars, fast, slow) {
    const n = bars.length;
    const out = new Array(n).fill(null);
    if (n === 0 || fast === 0 || slow === 0 || fast >= slow) return out;
    const adl = adlCompute(bars);
    const fast_ema = ema(adl, fast);
    const slow_ema = ema(adl, slow);
    for (let i = 0; i < n; i++) {
        const f = fast_ema[i], s = slow_ema[i];
        if (f != null && s != null) out[i] = f - s;
    }
    return out;
}

// EMA over (number|null)[] — mirrors Rust impl: seed from SMA of first
// `period` values; if any of those is null, returns all null.
export function ema(series, period) {
    const n = series.length;
    const out = new Array(n).fill(null);
    if (period === 0 || n < period) return out;
    let seed_sum = 0;
    let have_seed = true;
    for (let i = 0; i < period; i++) {
        if (series[i] == null) { have_seed = false; break; }
        seed_sum += series[i];
    }
    if (!have_seed) return out;
    const k = 2 / (period + 1);
    let cur = seed_sum / period;
    out[period - 1] = cur;
    for (let i = period; i < n; i++) {
        if (series[i] != null) {
            cur = series[i] * k + cur * (1 - k);
        }
        out[i] = cur;
    }
    return out;
}

// Parse "high low close volume" 4-token-per-line blob.
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
            out.errors.push({ line_no: i + 1, message: `expected 4 tokens (high low close volume), got ${parts.length}` });
            continue;
        }
        const h = Number(parts[0].replace(/\$/g, ''));
        const l = Number(parts[1].replace(/\$/g, ''));
        const c = Number(parts[2].replace(/\$/g, ''));
        const v = Number(parts[3].replace(/[\$,]/g, ''));
        if (!Number.isFinite(h) || !Number.isFinite(l) || !Number.isFinite(c)
            || !Number.isFinite(v) || h <= 0 || l <= 0 || c <= 0 || v < 0) {
            out.errors.push({ line_no: i + 1, message: `HLCV must be finite (HLC positive, vol ≥ 0)` });
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
        out.bars.push({ high: h, low: l, close: c, volume: v });
    }
    return out;
}

export function barsToBlob(bars) {
    return bars.map(b => `${b.high} ${b.low} ${b.close} ${b.volume}`).join('\n');
}

// 5-tier oscillator-sign verdict on most recent value.
export function signBadge(co_last) {
    if (co_last == null || !Number.isFinite(co_last)) {
        return { key: 'view.chosc.sign.unknown', cls: '' };
    }
    if (co_last > 0)        return { key: 'view.chosc.sign.bullish', cls: 'pos' };
    if (co_last < 0)        return { key: 'view.chosc.sign.bearish', cls: 'neg' };
    return { key: 'view.chosc.sign.neutral', cls: '' };
}

// Recent zero-cross detector.
export function crossBadge(co) {
    if (!Array.isArray(co)) return { key: 'view.chosc.cross.unknown', cls: '' };
    let prev = null;
    let last_cross = null;
    let last_cross_idx = -1;
    for (let i = 0; i < co.length; i++) {
        const v = co[i];
        if (v == null || !Number.isFinite(v)) continue;
        if (prev != null) {
            if (prev <= 0 && v > 0)      { last_cross = 'up';   last_cross_idx = i; }
            else if (prev >= 0 && v < 0) { last_cross = 'down'; last_cross_idx = i; }
        }
        prev = v;
    }
    if (last_cross == null) return { key: 'view.chosc.cross.none', cls: '' };
    const barsAgo = co.length - 1 - last_cross_idx;
    if (last_cross === 'up') return { key: 'view.chosc.cross.up_recent',   cls: 'pos', barsAgo };
    return { key: 'view.chosc.cross.down_recent', cls: 'neg', barsAgo };
}

// Trend over last N populated values.
export function trendBadge(co, lookback = 10) {
    if (!Array.isArray(co) || co.length === 0) {
        return { key: 'view.chosc.trend.unknown', cls: '' };
    }
    const tail = [];
    for (let i = co.length - 1; i >= 0 && tail.length < lookback; i--) {
        if (co[i] != null && Number.isFinite(co[i])) tail.unshift(co[i]);
    }
    if (tail.length < 2) return { key: 'view.chosc.trend.unknown', cls: '' };
    const slope = tail[tail.length - 1] - tail[0];
    const range = Math.max(...tail) - Math.min(...tail);
    if (range === 0)              return { key: 'view.chosc.trend.flat',          cls: '' };
    if (slope > range * 0.5)       return { key: 'view.chosc.trend.rising_fast',  cls: 'pos' };
    if (slope > range * 0.1)       return { key: 'view.chosc.trend.rising',       cls: 'pos' };
    if (slope < -range * 0.5)      return { key: 'view.chosc.trend.falling_fast', cls: 'neg' };
    if (slope < -range * 0.1)      return { key: 'view.chosc.trend.falling',      cls: 'neg' };
    return { key: 'view.chosc.trend.flat', cls: '' };
}

// Divergence vs price (close) over last `lookback` bars.
export function divergenceBadge(co, bars, lookback = 20) {
    if (!Array.isArray(co) || !Array.isArray(bars)) {
        return { key: 'view.chosc.div.unknown', cls: '' };
    }
    const coTail = [], closeTail = [];
    for (let i = co.length - 1; i >= 0 && coTail.length < lookback; i--) {
        if (co[i] != null && Number.isFinite(co[i])) coTail.unshift(co[i]);
        if (bars[i] && Number.isFinite(bars[i].close)) closeTail.unshift(bars[i].close);
    }
    if (coTail.length < 3 || closeTail.length < 3) return { key: 'view.chosc.div.unknown', cls: '' };
    const coDelta = coTail[coTail.length - 1] - coTail[0];
    const closeDelta = closeTail[closeTail.length - 1] - closeTail[0];
    if (Math.sign(coDelta) === Math.sign(closeDelta) && coDelta !== 0 && closeDelta !== 0) {
        return { key: 'view.chosc.div.confirms', cls: 'pos' };
    }
    if (coDelta > 0 && closeDelta < 0) return { key: 'view.chosc.div.bullish', cls: 'pos' };
    if (coDelta < 0 && closeDelta > 0) return { key: 'view.chosc.div.bearish', cls: 'neg' };
    return { key: 'view.chosc.div.neutral', cls: '' };
}

export function summarizeBars(bars) {
    if (!Array.isArray(bars) || bars.length === 0) {
        return { count: 0, last_close: NaN, total_volume: NaN, mean_close: NaN,
                 min_low: NaN, max_high: NaN };
    }
    let sumC = 0, sumV = 0, mxH = -Infinity, mnL = Infinity;
    for (const b of bars) {
        sumC += b.close;
        sumV += b.volume;
        if (b.high > mxH) mxH = b.high;
        if (b.low  < mnL) mnL = b.low;
    }
    return {
        count: bars.length,
        last_close: bars[bars.length - 1].close,
        total_volume: sumV,
        mean_close: sumC / bars.length,
        min_low:  Number.isFinite(mnL) ? mnL : NaN,
        max_high: Number.isFinite(mxH) ? mxH : NaN,
    };
}

function lcg(seed) {
    let state = BigInt(7919) + seed;
    return () => {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(state >> 32n) / 0xFFFFFFFF;
    };
}

function mkAccum(price, range, vol, rand) {
    return { high: price + range / 2, low: price - range / 2,
             close: price + range * 0.35 * (1 + (rand() - 0.5) * 0.1), volume: vol };
}
function mkDist(price, range, vol, rand) {
    return { high: price + range / 2, low: price - range / 2,
             close: price - range * 0.35 * (1 + (rand() - 0.5) * 0.1), volume: vol };
}
function mkNeutral(price, range, vol) {
    return { high: price + range / 2, low: price - range / 2, close: price, volume: vol };
}

export function makeDemoInput(kind = 'accumulation') {
    switch (kind) {
        case 'accumulation': {
            const rand = lcg(42n);
            return { bars: Array.from({ length: 60 }, (_, i) => mkAccum(100 + i * 0.3, 2, 1000 + rand() * 200, rand)),
                     fast: 3, slow: 10 };
        }
        case 'distribution': {
            const rand = lcg(7n);
            return { bars: Array.from({ length: 60 }, (_, i) => mkDist(140 - i * 0.3, 2, 1000 + rand() * 200, rand)),
                     fast: 3, slow: 10 };
        }
        case 'sideways-neutral': {
            const rand = lcg(11n);
            return { bars: Array.from({ length: 60 }, () => mkNeutral(100, 1.5, 1000 + rand() * 200)),
                     fast: 3, slow: 10 };
        }
        case 'bull-divergence': {
            // Price falling, accumulation rising → bullish divergence.
            const rand = lcg(13n);
            const bars = [];
            for (let i = 0; i < 60; i++) {
                const p = 130 - i * 0.3;
                bars.push(mkAccum(p, 2, 1500 + rand() * 500, rand));
            }
            return { bars, fast: 3, slow: 10 };
        }
        case 'bear-divergence': {
            const rand = lcg(21n);
            const bars = [];
            for (let i = 0; i < 60; i++) {
                const p = 100 + i * 0.3;
                bars.push(mkDist(p, 2, 1500 + rand() * 500, rand));
            }
            return { bars, fast: 3, slow: 10 };
        }
        case 'cross-up': {
            // First half distribution, second half accumulation → CO crosses zero up.
            const rand = lcg(33n);
            const bars = [];
            for (let i = 0; i < 30; i++) bars.push(mkDist(120, 2, 1000 + rand() * 200, rand));
            for (let i = 0; i < 30; i++) bars.push(mkAccum(120, 2, 1500 + rand() * 300, rand));
            return { bars, fast: 3, slow: 10 };
        }
        case 'wide-fast-slow': {
            // Fast=5 slow=20 — slower oscillator, less noisy.
            const rand = lcg(57n);
            return { bars: Array.from({ length: 80 }, (_, i) => mkAccum(100 + i * 0.3, 2, 1000 + rand() * 200, rand)),
                     fast: 5, slow: 20 };
        }
        case 'flat-zero': {
            // Midpoint closes → ADL flat at 0 → CO = 0.
            return { bars: Array.from({ length: 40 }, () => ({ high: 101, low: 99, close: 100, volume: 1000 })),
                     fast: 3, slow: 10 };
        }
        default: return makeDemoInput('accumulation');
    }
}

export function fmtNum(v, d = 0) {
    if (v == null || !Number.isFinite(v)) return '—';
    const abs = Math.abs(v);
    if (abs >= 1e9) return (v / 1e9).toFixed(2) + 'B';
    if (abs >= 1e6) return (v / 1e6).toFixed(2) + 'M';
    if (abs >= 1e3) return (v / 1e3).toFixed(2) + 'k';
    return v.toFixed(d);
}

export function fmtSigned(v, d = 0) {
    if (v == null || !Number.isFinite(v)) return '—';
    const formatted = fmtNum(Math.abs(v), d);
    return (v >= 0 ? '+' : '-') + formatted;
}

export function fmtPrice(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtInt(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}
