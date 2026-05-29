// Accumulation / Distribution Line (Chaikin) helpers.
//
// Backend body: { bars: Bar[] } where Bar = { high, low, close, volume }
// Returns: (number|null)[]  — cumulative running ADL value per bar.

export const DEFAULT_INPUTS = { bars: [] };

export function validateInputs(input) {
    if (!Array.isArray(input.bars))                       return 'bars must be an array';
    if (input.bars.length === 0)                          return 'bars cannot be empty';
    for (let i = 0; i < input.bars.length; i++) {
        const b = input.bars[i];
        if (!b)                                            return `bars[${i}] missing`;
        // Allow non-finite — the Rust impl carries forward ADL through NaN bars.
        // But disallow non-finite volume in client validation? Carry-forward is intentional.
        // Match Rust behavior: accept NaN bars silently.
        if (typeof b.high !== 'number' || typeof b.low !== 'number'
            || typeof b.close !== 'number' || typeof b.volume !== 'number')
                                                           return `bars[${i}] HLCV must be numbers`;
        if (Number.isFinite(b.high) && Number.isFinite(b.low) && b.high < b.low)
                                                           return `bars[${i}] high < low`;
    }
    return null;
}

export function buildBody(input) {
    return {
        bars: input.bars.map(b => ({
            high: b.high, low: b.low, close: b.close, volume: b.volume,
        })),
    };
}

// Pure-JS mirror of crates/traderview-core/src/accumulation_distribution_line.rs::compute.
export function localCompute(bars) {
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
        const v = Number(parts[3].replace(/[\$,kKmMbB]/g, ''));
        if (!Number.isFinite(h) || !Number.isFinite(l) || !Number.isFinite(c)
            || !Number.isFinite(v) || h <= 0 || l <= 0 || c <= 0 || v < 0) {
            out.errors.push({ line_no: i + 1, message: `HLCV must be finite (HLC positive, volume non-negative)` });
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

// Direction verdict based on slope over last N values.
export function trendBadge(adl, lookback = 10) {
    if (!Array.isArray(adl) || adl.length === 0) {
        return { key: 'view.adl.trend.unknown', cls: '' };
    }
    const tail = [];
    for (let i = adl.length - 1; i >= 0 && tail.length < lookback; i--) {
        const v = adl[i];
        if (v != null && Number.isFinite(v)) tail.unshift(v);
    }
    if (tail.length < 2) return { key: 'view.adl.trend.unknown', cls: '' };
    const slope = tail[tail.length - 1] - tail[0];
    const range = Math.max(...tail) - Math.min(...tail);
    if (range === 0)              return { key: 'view.adl.trend.flat',          cls: '' };
    if (slope > range * 0.6)       return { key: 'view.adl.trend.strong_accum', cls: 'pos' };
    if (slope > range * 0.1)       return { key: 'view.adl.trend.accum',        cls: 'pos' };
    if (slope < -range * 0.6)      return { key: 'view.adl.trend.strong_dist',  cls: 'neg' };
    if (slope < -range * 0.1)      return { key: 'view.adl.trend.dist',         cls: 'neg' };
    return { key: 'view.adl.trend.flat', cls: '' };
}

// Divergence detector: ADL last-vs-first vs close last-vs-first over last N bars.
export function divergenceBadge(adl, bars, lookback = 20) {
    if (!Array.isArray(adl) || !Array.isArray(bars)
        || adl.length === 0 || bars.length === 0) {
        return { key: 'view.adl.div.unknown', cls: '' };
    }
    const adlTail = [], closeTail = [];
    for (let i = adl.length - 1; i >= 0 && adlTail.length < lookback; i--) {
        if (adl[i] != null && Number.isFinite(adl[i])) adlTail.unshift(adl[i]);
        if (bars[i] && Number.isFinite(bars[i].close)) closeTail.unshift(bars[i].close);
    }
    if (adlTail.length < 3 || closeTail.length < 3) return { key: 'view.adl.div.unknown', cls: '' };
    const adlDelta = adlTail[adlTail.length - 1] - adlTail[0];
    const closeDelta = closeTail[closeTail.length - 1] - closeTail[0];
    const sameSign = (Math.sign(adlDelta) === Math.sign(closeDelta)) && adlDelta !== 0 && closeDelta !== 0;
    if (sameSign)                            return { key: 'view.adl.div.confirms', cls: 'pos' };
    if (adlDelta > 0 && closeDelta < 0)      return { key: 'view.adl.div.bullish',  cls: 'pos' };
    if (adlDelta < 0 && closeDelta > 0)      return { key: 'view.adl.div.bearish',  cls: 'neg' };
    return { key: 'view.adl.div.neutral', cls: '' };
}

// Last-value sign verdict (accumulation phase vs distribution phase).
export function phaseBadge(adl_last) {
    if (adl_last == null || !Number.isFinite(adl_last)) {
        return { key: 'view.adl.phase.unknown', cls: '' };
    }
    if (adl_last > 0) return { key: 'view.adl.phase.accumulation', cls: 'pos' };
    if (adl_last < 0) return { key: 'view.adl.phase.distribution', cls: 'neg' };
    return { key: 'view.adl.phase.neutral', cls: '' };
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
        min_low: Number.isFinite(mnL) ? mnL : NaN,
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

// Bars favoring close-near-high (accumulation).
function mkAccumBar(price, range, vol, rand) {
    const noise = (rand() - 0.5) * 0.1;
    return {
        high:  price + range / 2,
        low:   price - range / 2,
        close: price + range * (0.35 + noise),    // near high
        volume: vol,
    };
}

// Bars favoring close-near-low (distribution).
function mkDistBar(price, range, vol, rand) {
    const noise = (rand() - 0.5) * 0.1;
    return {
        high:  price + range / 2,
        low:   price - range / 2,
        close: price - range * (0.35 + noise),    // near low
        volume: vol,
    };
}

function mkNeutralBar(price, range, vol, rand) {
    const r = rand();
    return {
        high:  price + range * r,
        low:   price - range * (1 - r),
        close: price,
        volume: vol,
    };
}

export function makeDemoInput(kind = 'accumulation') {
    switch (kind) {
        case 'accumulation': {
            // Closes near highs throughout → ADL rises.
            const rand = lcg(42n);
            return { bars: Array.from({ length: 60 }, (_, i) => mkAccumBar(100 + i * 0.3, 2, 1000 + rand() * 200, rand)) };
        }
        case 'distribution': {
            const rand = lcg(7n);
            return { bars: Array.from({ length: 60 }, (_, i) => mkDistBar(140 - i * 0.3, 2, 1000 + rand() * 200, rand)) };
        }
        case 'bull-divergence': {
            // Price falling, ADL rising — bullish divergence (buyers absorbing).
            const rand = lcg(11n);
            const bars = [];
            for (let i = 0; i < 60; i++) {
                const p = 130 - i * 0.3;
                // Use accum-flavored bars but price drops → divergence.
                bars.push(mkAccumBar(p, 2, 1500 + rand() * 500, rand));
            }
            return { bars };
        }
        case 'bear-divergence': {
            // Price rising, ADL falling — distribution into strength.
            const rand = lcg(13n);
            const bars = [];
            for (let i = 0; i < 60; i++) {
                const p = 100 + i * 0.3;
                bars.push(mkDistBar(p, 2, 1500 + rand() * 500, rand));
            }
            return { bars };
        }
        case 'sideways': {
            const rand = lcg(21n);
            return { bars: Array.from({ length: 60 }, () => mkNeutralBar(100, 1.5, 1000 + rand() * 200, rand)) };
        }
        case 'climax-volume': {
            // Quiet then volume burst at end on close-near-high.
            const rand = lcg(33n);
            const bars = [];
            for (let i = 0; i < 50; i++) bars.push(mkAccumBar(100 + i * 0.1, 1, 500 + rand() * 100, rand));
            for (let i = 0; i < 10; i++) bars.push(mkAccumBar(105 + i * 0.5, 2, 5000 + rand() * 1000, rand));
            return { bars };
        }
        case 'doji-cluster': {
            // Range bars with zero MFM contribution interspersed.
            const rand = lcg(57n);
            return { bars: Array.from({ length: 50 }, () => ({ high: 100, low: 100, close: 100, volume: 1000 + rand() * 200 })) };
        }
        case 'small-volume': {
            // Verify ADL magnitudes are dominated by volume.
            const rand = lcg(99n);
            return { bars: Array.from({ length: 40 }, (_, i) => mkAccumBar(100 + i * 0.5, 1, 1, rand)) };
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
