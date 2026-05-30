// Balance of Power (Igor Livshin) helpers.
//
// Backend body: { bars: [{open, high, low, close}, ...], smoothing_period }
// Returns: { raw_bop, smoothed_bop, smoothing_period }
//
// BOP_t = (close − open) / (high − low), clamped to [-1, +1].
// Smoothed = SMA over smoothing_period bars.

import { t } from './i18n.js';

export const DEFAULT_SMOOTHING = 14;

export const DEFAULT_INPUTS = {
    bars: [],
    smoothing_period: DEFAULT_SMOOTHING,
};

export function validateInputs(input) {
    if (!Array.isArray(input.bars))                              return t('view.balance_of_power.validate.bars_array');
    for (let i = 0; i < input.bars.length; i++) {
        const b = input.bars[i];
        if (!b || typeof b !== 'object')                         return t('view.balance_of_power.validate.bar_object', { i });
        for (const f of ['open', 'high', 'low', 'close']) {
            if (!Number.isFinite(b[f]))                          return t('view.balance_of_power.validate.bar_field_finite', { i, f });
        }
        if (b.high < b.low)                                      return t('view.balance_of_power.validate.bar_high_low', { i });
    }
    if (!Number.isInteger(input.smoothing_period))               return t('view.balance_of_power.validate.smoothing_int');
    if (input.smoothing_period < 1)                              return t('view.balance_of_power.validate.smoothing_min');
    return null;
}

export function buildBody(input) {
    return {
        bars: input.bars.map(b => ({ open: b.open, high: b.high, low: b.low, close: b.close })),
        smoothing_period: input.smoothing_period,
    };
}

// Pure-JS mirror of crates/traderview-core/src/balance_of_power.rs::compute.
export function localCompute(bars, smoothing_period) {
    const n = bars.length;
    const raw = new Array(n).fill(null);
    let smoothed = new Array(n).fill(null);
    if (n === 0 || smoothing_period === 0) {
        return { raw_bop: raw, smoothed_bop: smoothed, smoothing_period };
    }
    for (const b of bars) {
        if (!Number.isFinite(b.open) || !Number.isFinite(b.high)
            || !Number.isFinite(b.low) || !Number.isFinite(b.close)) {
            return { raw_bop: raw, smoothed_bop: smoothed, smoothing_period };
        }
    }
    for (let i = 0; i < n; i++) {
        const bar = bars[i];
        const range = bar.high - bar.low;
        if (range > 0) {
            const v = (bar.close - bar.open) / range;
            raw[i] = Math.max(-1, Math.min(1, v));
        } else {
            raw[i] = 0;
        }
    }
    if (smoothing_period > 1 && n >= smoothing_period) {
        for (let i = smoothing_period - 1; i < n; i++) {
            let sum = 0;
            for (let k = i + 1 - smoothing_period; k <= i; k++) sum += raw[k];
            smoothed[i] = sum / smoothing_period;
        }
    } else {
        smoothed = raw.slice();
    }
    return { raw_bop: raw, smoothed_bop: smoothed, smoothing_period };
}

// Parse "open high low close" per line; blanks + # comments ignored.
export function parseBarsBlob(blob) {
    const out = { bars: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: t('common.parse.input_must_be_string') });
        return out;
    }
    const lines = blob.split('\n');
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i].split('#')[0].trim();
        if (!raw) continue;
        const toks = raw.split(/[\s,]+/).filter(t => t.length > 0);
        if (toks.length !== 4) {
            out.errors.push({ line_no: i + 1, message: 'expected 4 tokens (open high low close)' });
            continue;
        }
        const [open, high, low, close] = toks.map(Number);
        if (![open, high, low, close].every(Number.isFinite)) {
            out.errors.push({ line_no: i + 1, message: 'non-finite token' });
            continue;
        }
        if (high < low) {
            out.errors.push({ line_no: i + 1, message: 'high < low' });
            continue;
        }
        out.bars.push({ open, high, low, close });
    }
    return out;
}

export function barsToBlob(bars) {
    return bars.map(b => `${b.open} ${b.high} ${b.low} ${b.close}`).join('\n');
}

// 5-tier verdict on smoothed BOP (last value).
export function regimeBadge(v) {
    if (v == null || !Number.isFinite(v))   return { key: 'view.bop.badge.unknown', cls: '' };
    if (v >= 0.5)                            return { key: 'view.bop.badge.strong_bull', cls: 'pos' };
    if (v >= 0.1)                            return { key: 'view.bop.badge.bullish',     cls: 'pos' };
    if (v > -0.1)                            return { key: 'view.bop.badge.balanced',    cls: '' };
    if (v > -0.5)                            return { key: 'view.bop.badge.bearish',     cls: 'neg' };
    return { key: 'view.bop.badge.strong_bear', cls: 'neg' };
}

// Crossover detection (raw vs smoothed) — useful momentum signal.
export function lastCrossover(report) {
    if (!report || !Array.isArray(report.raw_bop)) return null;
    const r = report.raw_bop;
    const s = report.smoothed_bop;
    let lastIdx = -1, kind = null;
    for (let i = 1; i < r.length; i++) {
        if (r[i] == null || r[i - 1] == null || s[i] == null || s[i - 1] == null) continue;
        const prev = r[i - 1] - s[i - 1];
        const cur  = r[i] - s[i];
        if (prev <= 0 && cur > 0)      { lastIdx = i; kind = 'bull'; }
        else if (prev >= 0 && cur < 0) { lastIdx = i; kind = 'bear'; }
    }
    return lastIdx >= 0 ? { idx: lastIdx, kind } : null;
}

// Aggregate stats.
export function summarize(report) {
    if (!report || !Array.isArray(report.raw_bop) || report.raw_bop.length === 0) {
        return { count: 0, populated: 0, last_raw: NaN, last_smoothed: NaN,
                 mean_raw: NaN, bull_bars: 0, bear_bars: 0 };
    }
    const r = report.raw_bop;
    const s = report.smoothed_bop;
    let populated = 0, lastR = NaN, lastS = NaN, sumR = 0;
    let bull = 0, bear = 0;
    for (let i = 0; i < r.length; i++) {
        if (r[i] != null && Number.isFinite(r[i])) {
            populated++;
            lastR = r[i];
            sumR += r[i];
            if (r[i] > 0) bull++;
            else if (r[i] < 0) bear++;
        }
        if (s[i] != null && Number.isFinite(s[i])) lastS = s[i];
    }
    return {
        count: r.length,
        populated,
        last_raw: lastR,
        last_smoothed: lastS,
        mean_raw: populated > 0 ? sumR / populated : NaN,
        bull_bars: bull,
        bear_bars: bear,
    };
}

// LCG for stable demos.
function lcg(seed) {
    let state = BigInt(7919) + seed;
    return () => {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(state >> 32n) / 0xFFFFFFFF;
    };
}

export function makeDemoInput(kind = 'strong-bull') {
    switch (kind) {
        case 'strong-bull': {
            // Marubozu bars: open=low, close=high → BOP=+1 every bar.
            const bars = [];
            for (let i = 0; i < 40; i++) {
                const p = 100 + i;
                bars.push({ open: p, high: p + 2, low: p, close: p + 2 });
            }
            return { ...DEFAULT_INPUTS, bars };
        }
        case 'strong-bear': {
            // Open=high, close=low → BOP=-1.
            const bars = [];
            for (let i = 0; i < 40; i++) {
                const p = 110 - i;
                bars.push({ open: p, high: p, low: p - 2, close: p - 2 });
            }
            return { ...DEFAULT_INPUTS, bars };
        }
        case 'balanced': {
            const bars = [];
            for (let i = 0; i < 40; i++) {
                const m = 100 + Math.sin(i * 0.2) * 0.5;
                bars.push({ open: m, high: m + 1, low: m - 1, close: m });
            }
            return { ...DEFAULT_INPUTS, bars };
        }
        case 'choppy-noise': {
            const rand = lcg(42n);
            const bars = [];
            for (let i = 0; i < 60; i++) {
                const m = 100 + Math.sin(i * 0.4) * 2;
                const range = 1 + rand() * 2;
                const dir = rand() > 0.5 ? 1 : -1;
                bars.push({ open: m - dir * range * 0.3, high: m + range, low: m - range, close: m + dir * range * 0.3 });
            }
            return { ...DEFAULT_INPUTS, bars };
        }
        case 'bull-then-bear': {
            const bars = [];
            for (let i = 0; i < 20; i++) {
                const p = 100 + i;
                bars.push({ open: p, high: p + 2, low: p, close: p + 2 });
            }
            for (let i = 0; i < 20; i++) {
                const p = 120 - i;
                bars.push({ open: p, high: p, low: p - 2, close: p - 2 });
            }
            return { ...DEFAULT_INPUTS, bars };
        }
        case 'zero-range': {
            // Doji-only series — every bar has high=low=open=close.
            const bars = [];
            for (let i = 0; i < 20; i++) bars.push({ open: 100, high: 100, low: 100, close: 100 });
            return { ...DEFAULT_INPUTS, bars };
        }
        case 'short-smoothing': {
            // smoothing_period=3 on a small bullish series.
            const bars = [];
            for (let i = 0; i < 10; i++) {
                const p = 100 + i;
                bars.push({ open: p, high: p + 1, low: p, close: p + 1 });
            }
            return { ...DEFAULT_INPUTS, bars, smoothing_period: 3 };
        }
        case 'no-smoothing': {
            // smoothing_period=1 → smoothed == raw.
            const bars = [];
            for (let i = 0; i < 20; i++) {
                const p = 100 + Math.sin(i * 0.3);
                bars.push({ open: p, high: p + 1, low: p - 1, close: p + Math.cos(i * 0.4) });
            }
            return { ...DEFAULT_INPUTS, bars, smoothing_period: 1 };
        }
        default: return makeDemoInput('strong-bull');
    }
}

export function fmtBop(v, d = 4) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + v.toFixed(d);
}

export function fmtUSD(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return '$' + v.toFixed(d);
}

export function fmtInt(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}
