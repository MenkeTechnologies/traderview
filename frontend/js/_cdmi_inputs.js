// Chande Dynamic Momentum Index (DMI / DyMOI) helpers.
//
// Backend body: {
//   closes: number[], td_const: usize, std_period: usize, td_min: usize, td_max: usize
// }
// Returns: (number|null)[]  — RSI-like 0..100 with adaptive lookback.

import { t } from './i18n.js';

export const DEFAULT_TD_CONST = 14;
export const DEFAULT_STD_PERIOD = 5;
export const DEFAULT_TD_MIN = 5;
export const DEFAULT_TD_MAX = 30;
export const MIN_PERIOD = 2;
export const MAX_PERIOD = 500;

export const DEFAULT_INPUTS = {
    closes: [],
    td_const: DEFAULT_TD_CONST,
    std_period: DEFAULT_STD_PERIOD,
    td_min: DEFAULT_TD_MIN,
    td_max: DEFAULT_TD_MAX,
};

export function validateInputs(input) {
    if (!Array.isArray(input.closes))                       return t('view.cdmi.validate.closes_array');
    if (!Number.isInteger(input.td_const) || input.td_const < MIN_PERIOD || input.td_const > MAX_PERIOD)
                                                             return t('view.cdmi.validate.td_const_range', { min: MIN_PERIOD, max: MAX_PERIOD });
    if (!Number.isInteger(input.std_period) || input.std_period < MIN_PERIOD || input.std_period > MAX_PERIOD)
                                                             return t('view.cdmi.validate.std_period_range', { min: MIN_PERIOD, max: MAX_PERIOD });
    if (!Number.isInteger(input.td_min) || input.td_min < MIN_PERIOD)
                                                             return t('view.cdmi.validate.td_min', { min: MIN_PERIOD });
    if (!Number.isInteger(input.td_max) || input.td_max < input.td_min)
                                                             return t('view.cdmi.validate.td_max_ge_min');
    if (input.td_max < input.td_const)                      return t('view.cdmi.validate.td_max_ge_const');
    const required = 2 * input.std_period + input.td_max;
    if (input.closes.length < required)                     return t('view.cdmi.validate.closes_min', { n: required });
    for (let i = 0; i < input.closes.length; i++) {
        if (!Number.isFinite(input.closes[i]))              return t('view.cdmi.validate.close_finite', { i });
    }
    return null;
}

export function buildBody(input) {
    return {
        closes: input.closes.slice(),
        td_const:   input.td_const,
        std_period: input.std_period,
        td_min:     input.td_min,
        td_max:     input.td_max,
    };
}

// Pure-JS mirror of crates/traderview-core/src/chande_dynamic_momentum_index.rs::compute.
export function localCompute(closes, td_const, std_period, td_min, td_max) {
    const n = closes.length;
    const out = new Array(n).fill(null);
    if (td_const < 2 || std_period < 2 || td_min < 2 || td_max < td_min
        || td_max < td_const || n < 2 * std_period + td_max) return out;
    for (const v of closes) if (!Number.isFinite(v)) return out;
    const p_f = std_period;
    const std_series = new Array(n).fill(null);
    for (let i = std_period - 1; i < n; i++) {
        let sum = 0;
        for (let j = i + 1 - std_period; j <= i; j++) sum += closes[j];
        const mean = sum / p_f;
        let v_acc = 0;
        for (let j = i + 1 - std_period; j <= i; j++) v_acc += (closes[j] - mean) ** 2;
        const variance = v_acc / p_f;
        std_series[i] = Math.sqrt(Math.max(0, variance));
    }
    const avg_std = smaOpt(std_series, std_period);
    const td_c = td_const;
    for (let i = 0; i < n; i++) {
        const s = std_series[i];
        const a = avg_std[i];
        if (s == null || a == null || a <= 0 || s <= 0) continue;
        const vi = s / a;
        const tdRaw = Math.round(td_c / vi);
        const td = Math.max(td_min, Math.min(td_max, tdRaw));
        if (i < td) continue;
        const rsi = wilderRsiAt(closes, i, td);
        if (rsi != null) out[i] = rsi;
    }
    return out;
}

export function wilderRsiAt(closes, i, td) {
    if (i < td) return null;
    const p_f = td;
    let sum_gain = 0, sum_loss = 0;
    const start = i + 1 - td;
    for (let k = start + 1; k <= i; k++) {
        const diff = closes[k] - closes[k - 1];
        if (diff > 0) sum_gain += diff;
        else          sum_loss -= diff;
    }
    const avg_gain = sum_gain / p_f;
    const avg_loss = sum_loss / p_f;
    if (avg_loss <= 0) return avg_gain <= 0 ? 50 : 100;
    const rs = avg_gain / avg_loss;
    return 100 - 100 / (1 + rs);
}

export function smaOpt(series, period) {
    const n = series.length;
    const out = new Array(n).fill(null);
    if (period === 0 || n < period) return out;
    const p_f = period;
    for (let i = period - 1; i < n; i++) {
        let anyNull = false;
        let sum = 0;
        for (let j = i + 1 - period; j <= i; j++) {
            if (series[j] == null) { anyNull = true; break; }
            sum += series[j];
        }
        if (!anyNull) out[i] = sum / p_f;
    }
    return out;
}

// Parse positive prices.
export function parseClosesBlob(blob) {
    const out = { closes: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: t('common.parse.input_must_be_string') });
        return out;
    }
    const tokens = blob
        .split('\n')
        .map(l => l.split('#')[0])
        .join(' ')
        .split(/[\s,]+/)
        .filter(t => t.length > 0);
    for (let i = 0; i < tokens.length; i++) {
        const v = Number(tokens[i].replace(/[\$,]/g, ''));
        if (!Number.isFinite(v) || v <= 0) {
            out.errors.push({ line_no: i + 1, message: `token "${tokens[i]}" not a positive finite price` });
            continue;
        }
        out.closes.push(v);
    }
    return out;
}

export function closesToBlob(closes) {
    return closes.join('\n');
}

// 5-tier zone verdict on most recent DMI value.
export function zoneBadge(dmi_last) {
    if (dmi_last == null || !Number.isFinite(dmi_last)) {
        return { key: 'view.cdmi.zone.unknown', cls: '' };
    }
    if (dmi_last >= 80) return { key: 'view.cdmi.zone.overbought',  cls: 'neg' };
    if (dmi_last >= 60) return { key: 'view.cdmi.zone.strong_buy',  cls: 'pos' };
    if (dmi_last > 40)  return { key: 'view.cdmi.zone.neutral',     cls: '' };
    if (dmi_last >  20) return { key: 'view.cdmi.zone.strong_sell', cls: 'neg' };
    return { key: 'view.cdmi.zone.oversold', cls: 'pos' };
}

// Cross detector on classic RSI thresholds (70 / 30).
export function crossBadge(dmi) {
    if (!Array.isArray(dmi)) return { key: 'view.cdmi.cross.unknown', cls: '' };
    let prev = null;
    let last_cross = null;
    let last_cross_idx = -1;
    for (let i = 0; i < dmi.length; i++) {
        const v = dmi[i];
        if (v == null || !Number.isFinite(v)) continue;
        if (prev != null) {
            if (prev <= 70 && v > 70)        { last_cross = 'into_overbought'; last_cross_idx = i; }
            else if (prev >= 70 && v < 70)   { last_cross = 'out_of_overbought'; last_cross_idx = i; }
            else if (prev >= 30 && v < 30)   { last_cross = 'into_oversold'; last_cross_idx = i; }
            else if (prev <= 30 && v > 30)   { last_cross = 'out_of_oversold'; last_cross_idx = i; }
        }
        prev = v;
    }
    if (last_cross == null) return { key: 'view.cdmi.cross.none', cls: '' };
    const barsAgo = dmi.length - 1 - last_cross_idx;
    const map = {
        into_overbought:   { key: 'view.cdmi.cross.into_overbought',   cls: 'neg' },
        out_of_overbought: { key: 'view.cdmi.cross.out_of_overbought', cls: 'neg' },
        into_oversold:     { key: 'view.cdmi.cross.into_oversold',     cls: 'pos' },
        out_of_oversold:   { key: 'view.cdmi.cross.out_of_oversold',   cls: 'pos' },
    };
    return { ...map[last_cross], barsAgo };
}

// Trend over last N populated values.
export function trendBadge(dmi, lookback = 10) {
    if (!Array.isArray(dmi) || dmi.length === 0) {
        return { key: 'view.cdmi.trend.unknown', cls: '' };
    }
    const tail = [];
    for (let i = dmi.length - 1; i >= 0 && tail.length < lookback; i--) {
        if (dmi[i] != null && Number.isFinite(dmi[i])) tail.unshift(dmi[i]);
    }
    if (tail.length < 2) return { key: 'view.cdmi.trend.unknown', cls: '' };
    const slope = tail[tail.length - 1] - tail[0];
    const range = Math.max(...tail) - Math.min(...tail);
    if (range === 0)              return { key: 'view.cdmi.trend.flat',         cls: '' };
    if (slope > range * 0.5)       return { key: 'view.cdmi.trend.rising_fast', cls: 'pos' };
    if (slope > range * 0.1)       return { key: 'view.cdmi.trend.rising',      cls: 'pos' };
    if (slope < -range * 0.5)      return { key: 'view.cdmi.trend.falling_fast', cls: 'neg' };
    if (slope < -range * 0.1)      return { key: 'view.cdmi.trend.falling',     cls: 'neg' };
    return { key: 'view.cdmi.trend.flat', cls: '' };
}

// Current adaptive period estimate at last populated bar.
export function currentTdInfo(closes, td_const, std_period, td_min, td_max) {
    const n = closes.length;
    if (n === 0) return { td: null, vi: null };
    const sp = std_period;
    for (let i = n - 1; i >= sp - 1; i--) {
        let sum = 0;
        for (let j = i + 1 - sp; j <= i; j++) sum += closes[j];
        const mean = sum / sp;
        let var_acc = 0;
        for (let j = i + 1 - sp; j <= i; j++) var_acc += (closes[j] - mean) ** 2;
        const stdv = Math.sqrt(Math.max(0, var_acc / sp));
        // Avg of stdev needs another sp-window of stdev values — for the
        // dashboard estimate we approximate avg_std by stdev over a 2x window.
        const wider = i + 1 >= 2 * sp;
        if (!wider) continue;
        let sum2 = 0, cnt = 0;
        for (let j = i + 1 - 2 * sp; j <= i; j++) sum2 += closes[j];
        const mean2 = sum2 / (2 * sp);
        let var2 = 0;
        for (let j = i + 1 - 2 * sp; j <= i; j++) var2 += (closes[j] - mean2) ** 2;
        const avg = Math.sqrt(Math.max(0, var2 / (2 * sp)));
        if (avg <= 0 || stdv <= 0) return { td: null, vi: null };
        const vi = stdv / avg;
        const td = Math.max(td_min, Math.min(td_max, Math.round(td_const / vi)));
        return { td, vi };
    }
    return { td: null, vi: null };
}

export function summarizeCloses(closes) {
    if (!Array.isArray(closes) || closes.length === 0) {
        return { count: 0, last: NaN, min: NaN, max: NaN, mean: NaN };
    }
    let sum = 0, mx = -Infinity, mn = Infinity;
    for (const v of closes) {
        sum += v;
        if (v > mx) mx = v;
        if (v < mn) mn = v;
    }
    return {
        count: closes.length,
        last: closes[closes.length - 1],
        min: Number.isFinite(mn) ? mn : NaN,
        max: Number.isFinite(mx) ? mx : NaN,
        mean: sum / closes.length,
    };
}

function lcg(seed) {
    let state = BigInt(7919) + seed;
    return () => {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(state >> 32n) / 0xFFFFFFFF;
    };
}

export function makeDemoInput(kind = 'uptrend') {
    switch (kind) {
        case 'uptrend': {
            const rand = lcg(42n);
            return {
                closes: Array.from({ length: 250 }, (_, i) => 100 + i + (rand() - 0.5) * 0.5),
                td_const: 14, std_period: 5, td_min: 5, td_max: 30,
            };
        }
        case 'downtrend': {
            const rand = lcg(7n);
            return {
                closes: Array.from({ length: 250 }, (_, i) => 350 - i + (rand() - 0.5) * 0.5),
                td_const: 14, std_period: 5, td_min: 5, td_max: 30,
            };
        }
        case 'quiet-market': {
            // Low vol → period stretches to td_max → slower DMI.
            const rand = lcg(11n);
            return {
                closes: Array.from({ length: 250 }, () => 100 + (rand() - 0.5) * 0.2),
                td_const: 14, std_period: 5, td_min: 5, td_max: 30,
            };
        }
        case 'volatile-market': {
            // High vol → period shrinks to td_min → faster DMI.
            const rand = lcg(13n);
            return {
                closes: Array.from({ length: 250 }, () => 100 + (rand() - 0.5) * 10),
                td_const: 14, std_period: 5, td_min: 5, td_max: 30,
            };
        }
        case 'choppy-range': {
            const rand = lcg(21n);
            return {
                closes: Array.from({ length: 250 }, (_, i) => 100 + Math.sin(i * 0.3) * 5 + (rand() - 0.5) * 0.5),
                td_const: 14, std_period: 5, td_min: 5, td_max: 30,
            };
        }
        case 'reversal-up': {
            const rand = lcg(33n);
            const c = [];
            for (let i = 0; i < 125; i++) c.push(200 - i * 0.5 + (rand() - 0.5) * 0.5);
            for (let i = 0; i < 125; i++) c.push(140 + i * 0.5 + (rand() - 0.5) * 0.5);
            return { closes: c, td_const: 14, std_period: 5, td_min: 5, td_max: 30 };
        }
        case 'reversal-down': {
            const rand = lcg(57n);
            const c = [];
            for (let i = 0; i < 125; i++) c.push(100 + i * 0.5 + (rand() - 0.5) * 0.5);
            for (let i = 0; i < 125; i++) c.push(160 - i * 0.5 + (rand() - 0.5) * 0.5);
            return { closes: c, td_const: 14, std_period: 5, td_min: 5, td_max: 30 };
        }
        case 'short-bounds': {
            // td_min = td_max = td_const → fixed-period RSI behavior.
            const rand = lcg(99n);
            return {
                closes: Array.from({ length: 250 }, (_, i) => 100 + i * 0.5 + (rand() - 0.5) * 1),
                td_const: 14, std_period: 5, td_min: 14, td_max: 14,
            };
        }
        default: return makeDemoInput('uptrend');
    }
}

export function fmtNum(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtPrice(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtInt(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}
