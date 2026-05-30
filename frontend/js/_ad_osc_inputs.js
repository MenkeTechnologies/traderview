// Accumulation/Distribution Oscillator helpers.
//
// Backend body: { bars: Bar[], period: usize }
//   where Bar = { high, low, close, volume }
// Returns: {
//   per_bar: (number|null)[], ema: (number|null)[], period
// }
//
// Per-bar CLV × volume + EMA smoothing. Distinct from cumulative ADL —
// this oscillates around 0 and reads as "current buying pressure".

import { t } from './i18n.js';

export const DEFAULT_PERIOD = 14;
export const MIN_PERIOD = 2;
export const MAX_PERIOD = 500;

export const DEFAULT_INPUTS = {
    bars: [],
    period: DEFAULT_PERIOD,
};

export function validateInputs(input) {
    if (!Array.isArray(input.bars))                       return t('view.ad_osc.validate.bars_array');
    if (!Number.isInteger(input.period))                  return t('view.ad_osc.validate.period_int');
    if (input.period < MIN_PERIOD || input.period > MAX_PERIOD)
                                                           return t('view.ad_osc.validate.period_range', { min: MIN_PERIOD, max: MAX_PERIOD });
    if (input.bars.length < input.period)                 return t('view.ad_osc.validate.bars_min', { period: input.period });
    for (let i = 0; i < input.bars.length; i++) {
        const b = input.bars[i];
        if (!b)                                            return t('view.ad_osc.validate.bar_missing', { i });
        if (typeof b.high !== 'number' || typeof b.low !== 'number'
            || typeof b.close !== 'number' || typeof b.volume !== 'number')
                                                            return t('view.ad_osc.validate.hlcv_numbers', { i });
        if (!Number.isFinite(b.high) || !Number.isFinite(b.low)
            || !Number.isFinite(b.close) || !Number.isFinite(b.volume))
                                                            return t('view.ad_osc.validate.hlcv_finite', { i });
        if (b.volume < 0)                                  return t('view.ad_osc.validate.volume_negative', { i });
        if (b.high < b.low)                                return t('view.ad_osc.validate.high_lt_low', { i });
        if (b.close < b.low || b.close > b.high)           return t('view.ad_osc.validate.close_outside', { i });
    }
    return null;
}

export function buildBody(input) {
    return {
        bars: input.bars.map(b => ({ high: b.high, low: b.low, close: b.close, volume: b.volume })),
        period: input.period,
    };
}

// Pure-JS mirror of crates/traderview-core/src/accumulation_distribution_oscillator.rs::compute.
export function localCompute(bars, period) {
    const n = bars.length;
    const report = {
        per_bar: new Array(n).fill(null),
        ema:     new Array(n).fill(null),
        period,
    };
    if (period < 2 || n < period) return report;
    for (const b of bars) {
        if (!Number.isFinite(b.high) || !Number.isFinite(b.low)
            || !Number.isFinite(b.close) || !Number.isFinite(b.volume) || b.volume < 0) return report;
    }
    const raw = new Array(n);
    for (let i = 0; i < n; i++) {
        const range = bars[i].high - bars[i].low;
        const per = range > 0
            ? ((bars[i].close - bars[i].low) - (bars[i].high - bars[i].close)) / range * bars[i].volume
            : 0;
        raw[i] = per;
        report.per_bar[i] = per;
    }
    const p_f = period;
    const k = 2 / (p_f + 1);
    let sum = 0;
    for (let i = 0; i < period; i++) sum += raw[i];
    const seed = sum / p_f;
    report.ema[period - 1] = seed;
    let cur = seed;
    for (let i = period; i < n; i++) {
        cur = raw[i] * k + cur * (1 - k);
        report.ema[i] = cur;
    }
    return report;
}

// Parse "high low close volume" 4-token-per-line blob.
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
            out.errors.push({ line_no: i + 1, message: `HLCV must be finite (HLC positive, volume ≥ 0)` });
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

// Pressure verdict on most recent EMA value (vs typical volume magnitudes).
export function pressureBadge(ema_last, volume_scale) {
    if (ema_last == null || !Number.isFinite(ema_last)) {
        return { key: 'view.ado.pressure.unknown', cls: '' };
    }
    if (!Number.isFinite(volume_scale) || volume_scale <= 0) {
        if (ema_last > 0) return { key: 'view.ado.pressure.buying',  cls: 'pos' };
        if (ema_last < 0) return { key: 'view.ado.pressure.selling', cls: 'neg' };
        return { key: 'view.ado.pressure.neutral', cls: '' };
    }
    const ratio = ema_last / volume_scale;
    if (ratio > 0.40)  return { key: 'view.ado.pressure.strong_buy',  cls: 'pos' };
    if (ratio > 0.05)  return { key: 'view.ado.pressure.buying',     cls: 'pos' };
    if (ratio < -0.40) return { key: 'view.ado.pressure.strong_sell', cls: 'neg' };
    if (ratio < -0.05) return { key: 'view.ado.pressure.selling',    cls: 'neg' };
    return { key: 'view.ado.pressure.neutral', cls: '' };
}

// EMA-crossing verdict: when did EMA last cross zero, and what direction?
export function crossBadge(ema) {
    if (!Array.isArray(ema)) return { key: 'view.ado.cross.unknown', cls: '' };
    let prev = null;
    let last_cross = null;
    let last_cross_idx = -1;
    for (let i = 0; i < ema.length; i++) {
        const v = ema[i];
        if (v == null || !Number.isFinite(v)) continue;
        if (prev != null) {
            if (prev <= 0 && v > 0) { last_cross = 'up'; last_cross_idx = i; }
            else if (prev >= 0 && v < 0) { last_cross = 'down'; last_cross_idx = i; }
        }
        prev = v;
    }
    if (last_cross == null) return { key: 'view.ado.cross.none', cls: '' };
    const barsAgo = ema.length - 1 - last_cross_idx;
    if (last_cross === 'up')   return { key: 'view.ado.cross.up_recent',   cls: 'pos', barsAgo };
    return { key: 'view.ado.cross.down_recent', cls: 'neg', barsAgo };
}

// Trend verdict over last N EMA values.
export function trendBadge(ema, lookback = 10) {
    if (!Array.isArray(ema) || ema.length === 0) {
        return { key: 'view.ado.trend.unknown', cls: '' };
    }
    const tail = [];
    for (let i = ema.length - 1; i >= 0 && tail.length < lookback; i--) {
        if (ema[i] != null && Number.isFinite(ema[i])) tail.unshift(ema[i]);
    }
    if (tail.length < 2) return { key: 'view.ado.trend.unknown', cls: '' };
    const slope = tail[tail.length - 1] - tail[0];
    const range = Math.max(...tail) - Math.min(...tail);
    if (range === 0)              return { key: 'view.ado.trend.flat',         cls: '' };
    if (slope > range * 0.6)       return { key: 'view.ado.trend.strong_up',   cls: 'pos' };
    if (slope > range * 0.1)       return { key: 'view.ado.trend.up',          cls: 'pos' };
    if (slope < -range * 0.6)      return { key: 'view.ado.trend.strong_down', cls: 'neg' };
    if (slope < -range * 0.1)      return { key: 'view.ado.trend.down',        cls: 'neg' };
    return { key: 'view.ado.trend.flat', cls: '' };
}

export function summarizeBars(bars) {
    if (!Array.isArray(bars) || bars.length === 0) {
        return { count: 0, last_close: NaN, mean_volume: NaN, total_volume: NaN,
                 min_low: NaN, max_high: NaN };
    }
    let sumV = 0, mxH = -Infinity, mnL = Infinity;
    for (const b of bars) {
        sumV += b.volume;
        if (b.high > mxH) mxH = b.high;
        if (b.low  < mnL) mnL = b.low;
    }
    return {
        count: bars.length,
        last_close: bars[bars.length - 1].close,
        mean_volume: sumV / bars.length,
        total_volume: sumV,
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

function mkAccum(price, range, vol, rand) {
    return {
        high:  price + range / 2,
        low:   price - range / 2,
        close: price + range * 0.35 * (1 + (rand() - 0.5) * 0.1),
        volume: vol,
    };
}

function mkDist(price, range, vol, rand) {
    return {
        high:  price + range / 2,
        low:   price - range / 2,
        close: price - range * 0.35 * (1 + (rand() - 0.5) * 0.1),
        volume: vol,
    };
}

function mkNeutral(price, range, vol) {
    return { high: price + range / 2, low: price - range / 2, close: price, volume: vol };
}

export function makeDemoInput(kind = 'buying') {
    switch (kind) {
        case 'buying': {
            const rand = lcg(42n);
            return { bars: Array.from({ length: 60 }, (_, i) => mkAccum(100 + i * 0.3, 2, 1000 + rand() * 200, rand)),
                     period: 14 };
        }
        case 'selling': {
            const rand = lcg(7n);
            return { bars: Array.from({ length: 60 }, (_, i) => mkDist(140 - i * 0.3, 2, 1000 + rand() * 200, rand)),
                     period: 14 };
        }
        case 'neutral': {
            const rand = lcg(11n);
            return { bars: Array.from({ length: 60 }, () => mkNeutral(100 + (rand() - 0.5) * 2, 1.5, 1000 + rand() * 200)),
                     period: 14 };
        }
        case 'cross-up': {
            // EMA crosses from negative to positive.
            const rand = lcg(13n);
            const bars = [];
            for (let i = 0; i < 30; i++) bars.push(mkDist(120, 2, 1000 + rand() * 200, rand));
            for (let i = 0; i < 30; i++) bars.push(mkAccum(120, 2, 1500 + rand() * 300, rand));
            return { bars, period: 14 };
        }
        case 'cross-down': {
            const rand = lcg(21n);
            const bars = [];
            for (let i = 0; i < 30; i++) bars.push(mkAccum(100, 2, 1000 + rand() * 200, rand));
            for (let i = 0; i < 30; i++) bars.push(mkDist(100, 2, 1500 + rand() * 300, rand));
            return { bars, period: 14 };
        }
        case 'climax-buy': {
            // Quiet then huge volume spike with close-at-high.
            const rand = lcg(33n);
            const bars = [];
            for (let i = 0; i < 50; i++) bars.push(mkNeutral(100, 1, 500 + rand() * 100));
            for (let i = 0; i < 10; i++) bars.push(mkAccum(100, 2, 5000 + rand() * 1000, rand));
            return { bars, period: 14 };
        }
        case 'zero-range': {
            // All doji bars → all per_bar = 0 → EMA = 0.
            return { bars: Array.from({ length: 30 }, () => mkNeutral(100, 0, 1000)),
                     period: 14 };
        }
        case 'short-period': {
            // Period 5 instead of 14 — faster EMA reaction.
            const rand = lcg(57n);
            return { bars: Array.from({ length: 30 }, (_, i) => mkAccum(100 + i * 0.5, 2, 1000 + rand() * 200, rand)),
                     period: 5 };
        }
        default: return makeDemoInput('buying');
    }
}

export function fmtNum(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    const abs = Math.abs(v);
    if (abs >= 1e9) return (v / 1e9).toFixed(2) + 'B';
    if (abs >= 1e6) return (v / 1e6).toFixed(2) + 'M';
    if (abs >= 1e3) return (v / 1e3).toFixed(2) + 'k';
    return v.toFixed(d);
}

export function fmtSigned(v, d = 2) {
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
