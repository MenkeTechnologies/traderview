// Chande Volatility Index (CVI) helpers — % change in EMA of high-low range.
//
// Backend body: { bars: Bar[], ema_period: usize, roc_period: usize }
//   where Bar = { high, low }
// Returns: (number|null)[]  — CVI in %.

import { t } from './i18n.js';

export const DEFAULT_EMA = 10;
export const DEFAULT_ROC = 10;
export const MIN_EMA = 2;
export const MIN_ROC = 1;
export const MAX_PERIOD = 500;

export const DEFAULT_INPUTS = {
    bars: [],
    ema_period: DEFAULT_EMA,
    roc_period: DEFAULT_ROC,
};

export function validateInputs(input) {
    if (!Array.isArray(input.bars))                       return t('view.cvi.validate.bars_array');
    if (!Number.isInteger(input.ema_period) || input.ema_period < MIN_EMA || input.ema_period > MAX_PERIOD)
                                                           return t('view.cvi.validate.ema_range', { min: MIN_EMA, max: MAX_PERIOD });
    if (!Number.isInteger(input.roc_period) || input.roc_period < MIN_ROC || input.roc_period > MAX_PERIOD)
                                                           return t('view.cvi.validate.roc_range', { min: MIN_ROC, max: MAX_PERIOD });
    if (input.bars.length < input.ema_period + input.roc_period)
                                                           return t('view.cvi.validate.bars_min', { n: input.ema_period + input.roc_period });
    for (let i = 0; i < input.bars.length; i++) {
        const b = input.bars[i];
        if (!b)                                            return t('view.cvi.validate.bar_missing', { i });
        if (typeof b.high !== 'number' || typeof b.low !== 'number')
                                                            return t('view.cvi.validate.hl_numbers', { i });
        if (!Number.isFinite(b.high) || !Number.isFinite(b.low))
                                                            return t('view.cvi.validate.hl_finite', { i });
        if (b.high < b.low)                                return t('view.cvi.validate.high_lt_low', { i });
    }
    return null;
}

export function buildBody(input) {
    return {
        bars: input.bars.map(b => ({ high: b.high, low: b.low })),
        ema_period: input.ema_period,
        roc_period: input.roc_period,
    };
}

// Pure-JS mirror of crates/traderview-core/src/chande_volatility_index.rs::compute.
export function localCompute(bars, ema_period, roc_period) {
    const n = bars.length;
    const out = new Array(n).fill(null);
    if (ema_period < 2 || roc_period < 1 || n < ema_period + roc_period) return out;
    for (const b of bars) {
        if (!Number.isFinite(b.high) || !Number.isFinite(b.low)) return out;
    }
    const ranges = bars.map(b => b.high - b.low);
    const p_f = ema_period;
    const k = 2 / (p_f + 1);
    let sum = 0;
    for (let i = 0; i < ema_period; i++) sum += ranges[i];
    const seed = sum / p_f;
    const ema = new Array(n).fill(null);
    ema[ema_period - 1] = seed;
    let cur = seed;
    for (let i = ema_period; i < n; i++) {
        cur = ranges[i] * k + cur * (1 - k);
        ema[i] = cur;
    }
    for (let i = ema_period + roc_period - 1; i < n; i++) {
        const c = ema[i];
        const p = ema[i - roc_period];
        if (c != null && p != null && p !== 0) {
            out[i] = (c - p) / p * 100;
        }
    }
    return out;
}

// Parse "high low" 2-token-per-line blob.
export function parseBarsBlob(blob) {
    const out = { bars: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: t('common.parse.input_must_be_string') });
        return out;
    }
    const lines = blob.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = raw.split('#')[0].trim();
        if (!s) continue;
        const parts = s.split(/[\s,]+/).filter(Boolean);
        if (parts.length !== 2) {
            out.errors.push({ line_no: i + 1, message: `expected 2 tokens (high low), got ${parts.length}` });
            continue;
        }
        const h = Number(parts[0].replace(/\$/g, ''));
        const l = Number(parts[1].replace(/\$/g, ''));
        if (!Number.isFinite(h) || !Number.isFinite(l) || h <= 0 || l <= 0) {
            out.errors.push({ line_no: i + 1, message: `HL must be positive finite` });
            continue;
        }
        if (l > h) {
            out.errors.push({ line_no: i + 1, message: `low > high` });
            continue;
        }
        out.bars.push({ high: h, low: l });
    }
    return out;
}

export function barsToBlob(bars) {
    return bars.map(b => `${b.high} ${b.low}`).join('\n');
}

// 5-tier regime verdict on last CVI value.
export function regimeBadge(cvi_last) {
    if (cvi_last == null || !Number.isFinite(cvi_last)) {
        return { key: 'view.cvi.regime.unknown', cls: '' };
    }
    if (cvi_last > 30)   return { key: 'view.cvi.regime.expansion_strong', cls: 'neg' };
    if (cvi_last > 10)   return { key: 'view.cvi.regime.expansion',        cls: 'neg' };
    if (cvi_last > -10)  return { key: 'view.cvi.regime.steady',           cls: '' };
    if (cvi_last > -30)  return { key: 'view.cvi.regime.contraction',      cls: 'pos' };
    return { key: 'view.cvi.regime.contraction_strong', cls: 'pos' };
}

// Zero-cross detector.
export function crossBadge(cvi) {
    if (!Array.isArray(cvi)) return { key: 'view.cvi.cross.unknown', cls: '' };
    let prev = null;
    let last_cross = null;
    let last_cross_idx = -1;
    for (let i = 0; i < cvi.length; i++) {
        const v = cvi[i];
        if (v == null || !Number.isFinite(v)) continue;
        if (prev != null) {
            if (prev <= 0 && v > 0)      { last_cross = 'up';   last_cross_idx = i; }
            else if (prev >= 0 && v < 0) { last_cross = 'down'; last_cross_idx = i; }
        }
        prev = v;
    }
    if (last_cross == null) return { key: 'view.cvi.cross.none', cls: '' };
    const barsAgo = cvi.length - 1 - last_cross_idx;
    if (last_cross === 'up') return { key: 'view.cvi.cross.up_recent', cls: 'neg', barsAgo };
    return { key: 'view.cvi.cross.down_recent', cls: 'pos', barsAgo };
}

// Trend over last N populated values.
export function trendBadge(cvi, lookback = 10) {
    if (!Array.isArray(cvi) || cvi.length === 0) {
        return { key: 'view.cvi.trend.unknown', cls: '' };
    }
    const tail = [];
    for (let i = cvi.length - 1; i >= 0 && tail.length < lookback; i--) {
        if (cvi[i] != null && Number.isFinite(cvi[i])) tail.unshift(cvi[i]);
    }
    if (tail.length < 2) return { key: 'view.cvi.trend.unknown', cls: '' };
    const slope = tail[tail.length - 1] - tail[0];
    const range = Math.max(...tail) - Math.min(...tail);
    if (range === 0)              return { key: 'view.cvi.trend.flat',          cls: '' };
    if (slope > range * 0.5)       return { key: 'view.cvi.trend.rising_fast',  cls: 'neg' };
    if (slope > range * 0.1)       return { key: 'view.cvi.trend.rising',       cls: 'neg' };
    if (slope < -range * 0.5)      return { key: 'view.cvi.trend.falling_fast', cls: 'pos' };
    if (slope < -range * 0.1)      return { key: 'view.cvi.trend.falling',      cls: 'pos' };
    return { key: 'view.cvi.trend.flat', cls: '' };
}

export function summarizeBars(bars) {
    if (!Array.isArray(bars) || bars.length === 0) {
        return { count: 0, mean_range: NaN, min_low: NaN, max_high: NaN };
    }
    let sumR = 0, mxH = -Infinity, mnL = Infinity;
    for (const b of bars) {
        sumR += b.high - b.low;
        if (b.high > mxH) mxH = b.high;
        if (b.low  < mnL) mnL = b.low;
    }
    return {
        count: bars.length,
        mean_range: sumR / bars.length,
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

function mkBar(mid, range) {
    return { high: mid + range / 2, low: mid - range / 2 };
}

export function makeDemoInput(kind = 'expanding') {
    switch (kind) {
        case 'expanding': {
            // 30 tight bars → 30 expanding bars.
            const bars = Array.from({ length: 30 }, () => mkBar(100, 2));
            for (let i = 0; i < 30; i++) bars.push(mkBar(100, 2 + i * 0.5));
            return { bars, ema_period: 10, roc_period: 10 };
        }
        case 'contracting': {
            const bars = Array.from({ length: 30 }, (_, i) => mkBar(100, 20 - i * 0.5));
            for (let i = 0; i < 30; i++) bars.push(mkBar(100, 1));
            return { bars, ema_period: 10, roc_period: 10 };
        }
        case 'steady': {
            const rand = lcg(42n);
            return {
                bars: Array.from({ length: 60 }, () => mkBar(100, 2 + (rand() - 0.5) * 0.2)),
                ema_period: 10, roc_period: 10,
            };
        }
        case 'spike': {
            // Quiet then sudden range spike.
            const bars = Array.from({ length: 40 }, () => mkBar(100, 1));
            for (let i = 0; i < 10; i++) bars.push(mkBar(100, 10));
            for (let i = 0; i < 10; i++) bars.push(mkBar(100, 1));
            return { bars, ema_period: 10, roc_period: 10 };
        }
        case 'oscillating': {
            return {
                bars: Array.from({ length: 80 }, (_, i) => mkBar(100, 3 + Math.sin(i * 0.3) * 2.5)),
                ema_period: 10, roc_period: 10,
            };
        }
        case 'long-ema': {
            const rand = lcg(11n);
            return {
                bars: Array.from({ length: 100 }, (_, i) => mkBar(100, 2 + i * 0.05 + (rand() - 0.5) * 0.2)),
                ema_period: 25, roc_period: 25,
            };
        }
        case 'short-roc': {
            const rand = lcg(13n);
            return {
                bars: Array.from({ length: 40 }, (_, i) => mkBar(100, 2 + i * 0.1 + (rand() - 0.5) * 0.2)),
                ema_period: 10, roc_period: 3,
            };
        }
        case 'climax-volatility': {
            // Quiet, then huge sustained vol, then quiet again.
            const bars = [];
            for (let i = 0; i < 30; i++) bars.push(mkBar(100, 1));
            for (let i = 0; i < 20; i++) bars.push(mkBar(100, 15));
            for (let i = 0; i < 30; i++) bars.push(mkBar(100, 2));
            return { bars, ema_period: 10, roc_period: 10 };
        }
        default: return makeDemoInput('expanding');
    }
}

export function fmtPct(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d) + '%';
}

export function fmtPctSigned(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + v.toFixed(d) + '%';
}

export function fmtPrice(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtInt(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}
