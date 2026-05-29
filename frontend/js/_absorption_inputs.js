// Absorption Detector helpers.
//
// Backend body: { bars: { high, low, close, volume }[], period, threshold, vol_multiplier }
// Returns:      { bullish: bool[], bearish: bool[], period, threshold, vol_multiplier }

import { t } from './i18n.js';

export const DEFAULT_PERIOD = 20;
export const DEFAULT_THRESHOLD = 0.5;
export const DEFAULT_VOL_MULTIPLIER = 1.5;
export const MIN_PERIOD = 2;
export const MAX_PERIOD = 500;

export const DEFAULT_INPUTS = {
    bars: [],
    period: DEFAULT_PERIOD,
    threshold: DEFAULT_THRESHOLD,
    vol_multiplier: DEFAULT_VOL_MULTIPLIER,
};

export function validateInputs(input) {
    if (!Array.isArray(input.bars))                           return t('view.absorption.validate.bars_array');
    if (!Number.isInteger(input.period) || input.period < MIN_PERIOD || input.period > MAX_PERIOD)
                                                                  return t('view.absorption.validate.period_range', { min: MIN_PERIOD, max: MAX_PERIOD });
    if (input.bars.length < input.period + 1)                 return t('view.absorption.validate.bars_min', { have: input.bars.length });
    if (!Number.isFinite(input.threshold) || input.threshold <= 0)
                                                                  return t('view.absorption.validate.threshold');
    if (!Number.isFinite(input.vol_multiplier) || input.vol_multiplier <= 0)
                                                                  return t('view.absorption.validate.vol_multiplier');
    for (let i = 0; i < input.bars.length; i++) {
        const b = input.bars[i];
        if (!b || typeof b !== 'object')                      return t('view.absorption.validate.bar_object', { i });
        for (const k of ['high', 'low', 'close', 'volume']) {
            if (typeof b[k] !== 'number' || !Number.isFinite(b[k]))
                                                                  return t('view.absorption.validate.bar_field_finite', { i, k });
        }
        if (b.high < b.low)                                   return t('view.absorption.validate.high_lt_low', { i });
        if (b.close > b.high || b.close < b.low)              return t('view.absorption.validate.close_outside', { i });
        if (b.volume <= 0)                                    return t('view.absorption.validate.volume', { i });
    }
    return null;
}

export function buildBody(input) {
    return {
        bars: input.bars.map(b => ({
            high: b.high, low: b.low, close: b.close, volume: b.volume,
        })),
        period:         input.period,
        threshold:      input.threshold,
        vol_multiplier: input.vol_multiplier,
    };
}

// Pure-JS mirror of crates/traderview-core/src/absorption_detector.rs::compute.
export function localCompute(bars, period, threshold, vol_multiplier) {
    const n = bars.length;
    const bullish = new Array(n).fill(false);
    const bearish = new Array(n).fill(false);
    if (period < 2 || !Number.isFinite(threshold) || threshold <= 0
        || !Number.isFinite(vol_multiplier) || vol_multiplier <= 0
        || n < period + 1) return { bullish, bearish, period, threshold, vol_multiplier };
    for (const b of bars) {
        if (!Number.isFinite(b.high) || !Number.isFinite(b.low)
            || !Number.isFinite(b.close) || !Number.isFinite(b.volume) || b.volume <= 0) {
            return { bullish, bearish, period, threshold, vol_multiplier };
        }
    }
    const rpv = bars.map(b => (b.high - b.low) / b.volume);
    for (let i = period; i < n; i++) {
        let rpvSum = 0, volSum = 0;
        for (let k = i - period; k < i; k++) { rpvSum += rpv[k]; volSum += bars[k].volume; }
        const rpvAvg = rpvSum / period;
        const volAvg = volSum / period;
        const cur = bars[i];
        const range = cur.high - cur.low;
        if (range <= 0) continue;
        const mid = cur.low + range / 2;
        const prevClose = bars[i - 1].close;
        const absorb = rpv[i] < rpvAvg * threshold && cur.volume > volAvg * vol_multiplier;
        if (absorb && cur.close > mid && cur.close > prevClose) bullish[i] = true;
        if (absorb && cur.close < mid && cur.close < prevClose) bearish[i] = true;
    }
    return { bullish, bearish, period, threshold, vol_multiplier };
}

export function lastSignalBadge(report) {
    if (!report) return { key: 'view.abs.last.unknown', cls: '' };
    const n = report.bullish.length;
    for (let i = n - 1; i >= 0; i--) {
        if (report.bullish[i]) return { key: 'view.abs.last.bullish',  cls: 'pos', barsAgo: n - 1 - i };
        if (report.bearish[i]) return { key: 'view.abs.last.bearish',  cls: 'neg', barsAgo: n - 1 - i };
    }
    return { key: 'view.abs.last.none', cls: '' };
}

export function biasBadge(report) {
    if (!report) return { key: 'view.abs.bias.unknown', cls: '' };
    const bull = report.bullish.filter(Boolean).length;
    const bear = report.bearish.filter(Boolean).length;
    if (bull === 0 && bear === 0) return { key: 'view.abs.bias.flat',       cls: '' };
    if (bull > 0 && bear === 0)   return { key: 'view.abs.bias.all_bull',   cls: 'pos' };
    if (bear > 0 && bull === 0)   return { key: 'view.abs.bias.all_bear',   cls: 'neg' };
    if (bull > bear)              return { key: 'view.abs.bias.bull_lean',  cls: 'pos' };
    if (bear > bull)              return { key: 'view.abs.bias.bear_lean',  cls: 'neg' };
    return { key: 'view.abs.bias.balanced', cls: '' };
}

export function intensityBadge(report) {
    if (!report || !report.bullish.length) return { key: 'view.abs.intensity.unknown', cls: '' };
    const total = report.bullish.length;
    const sigs  = report.bullish.filter(Boolean).length + report.bearish.filter(Boolean).length;
    const rate  = sigs / total;
    if (rate === 0)        return { key: 'view.abs.intensity.silent', cls: '' };
    if (rate < 0.02)       return { key: 'view.abs.intensity.rare',   cls: '' };
    if (rate < 0.05)       return { key: 'view.abs.intensity.normal', cls: '' };
    if (rate < 0.10)       return { key: 'view.abs.intensity.busy',   cls: 'pos' };
    return { key: 'view.abs.intensity.flooded', cls: 'neg' };
}

// ── parser ──
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
            out.errors.push({ line_no: i + 1, message: 'expected 4 tokens: high low close volume' });
            continue;
        }
        const [h, l, c, v] = parts.map(p => Number(p));
        if (![h, l, c, v].every(Number.isFinite)) {
            out.errors.push({ line_no: i + 1, message: 'token not finite' });
            continue;
        }
        if (h < l)              { out.errors.push({ line_no: i + 1, message: 'high < low' });            continue; }
        if (c > h || c < l)     { out.errors.push({ line_no: i + 1, message: 'close outside [low, high]' }); continue; }
        if (v <= 0)             { out.errors.push({ line_no: i + 1, message: 'volume must be > 0' });    continue; }
        out.bars.push({ high: h, low: l, close: c, volume: v });
    }
    return out;
}

export function barsToBlob(bars) {
    return bars.map(b => `${b.high} ${b.low} ${b.close} ${b.volume}`).join('\n');
}

export function summarizeBars(bars) {
    if (!Array.isArray(bars) || bars.length === 0) {
        return { count: 0, last_close: NaN, min_low: NaN, max_high: NaN, vol_min: NaN, vol_max: NaN, vol_avg: NaN };
    }
    let mn = Infinity, mx = -Infinity, vMin = Infinity, vMax = -Infinity, vSum = 0;
    for (const b of bars) {
        if (b.low < mn) mn = b.low;
        if (b.high > mx) mx = b.high;
        if (b.volume < vMin) vMin = b.volume;
        if (b.volume > vMax) vMax = b.volume;
        vSum += b.volume;
    }
    return {
        count: bars.length,
        last_close: bars[bars.length - 1].close,
        min_low: mn, max_high: mx,
        vol_min: vMin, vol_max: vMax, vol_avg: vSum / bars.length,
    };
}

// ── deterministic LCG noise ──
function lcg(seed) {
    let s = BigInt(seed) & 0xFFFFFFFFFFFFFFFFn;
    return () => {
        s = (s * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(s >> 32n) / 0xFFFFFFFF;
    };
}

export function makeDemoInput(kind = 'bullish-absorb') {
    const base = { period: DEFAULT_PERIOD, threshold: DEFAULT_THRESHOLD, vol_multiplier: DEFAULT_VOL_MULTIPLIER };
    switch (kind) {
        case 'flat': {
            const bars = Array.from({ length: 30 }, () => ({ high: 101, low: 99, close: 100, volume: 1000 }));
            return { ...base, bars };
        }
        case 'bullish-absorb': {
            const bars = Array.from({ length: 25 }, () => ({ high: 101, low: 99, close: 100, volume: 1000 }));
            bars.push({ high: 100.9, low: 99.9, close: 100.9, volume: 10000 });
            return { ...base, bars };
        }
        case 'bearish-absorb': {
            const bars = Array.from({ length: 25 }, () => ({ high: 101, low: 99, close: 100, volume: 1000 }));
            bars.push({ high: 100.1, low: 99.1, close: 99.1, volume: 10000 });
            return { ...base, bars };
        }
        case 'normal-volume': {
            const bars = Array.from({ length: 25 }, () => ({ high: 101, low: 99, close: 100, volume: 1000 }));
            bars.push({ high: 100.9, low: 99.9, close: 100.9, volume: 1000 });   // not absorption
            return { ...base, bars };
        }
        case 'multi-absorb': {
            const bars = Array.from({ length: 22 }, () => ({ high: 101, low: 99, close: 100, volume: 1000 }));
            bars.push({ high: 100.9, low: 99.9, close: 100.9, volume: 10000 });   // bull
            bars.push({ high: 101.0, low: 99.0, close: 100, volume: 1000 });
            bars.push({ high: 100.1, low: 99.1, close: 99.1, volume: 10000 });    // bear
            return { ...base, bars };
        }
        case 'noisy': {
            const rng = lcg(42);
            const bars = Array.from({ length: 60 }, () => {
                const u = rng();
                const c = 100 + (u - 0.5) * 0.5;
                const h = c + 0.5 + u * 0.5;
                const l = c - 0.5 - (1 - u) * 0.5;
                return { high: h, low: l, close: c, volume: 800 + u * 600 };
            });
            // Sprinkle a couple absorption bars.
            bars[40] = { high: 100.2, low: 99.8, close: 100.2, volume: 9000 };
            bars[55] = { high: 100.2, low: 99.8, close: 99.8, volume: 9000 };
            return { ...base, bars };
        }
        case 'short-period': {
            const bars = Array.from({ length: 12 }, () => ({ high: 101, low: 99, close: 100, volume: 1000 }));
            bars.push({ high: 100.9, low: 99.9, close: 100.9, volume: 10000 });
            return { ...base, period: 10, bars };
        }
        case 'tight-thresh': {
            // Tighter threshold (0.3) and stricter vol multiplier (3.0).
            const bars = Array.from({ length: 25 }, () => ({ high: 101, low: 99, close: 100, volume: 1000 }));
            bars.push({ high: 100.9, low: 99.9, close: 100.9, volume: 15000 });
            return { period: DEFAULT_PERIOD, threshold: 0.3, vol_multiplier: 3.0, bars };
        }
        default: return makeDemoInput('bullish-absorb');
    }
}

// ── formatters ──
export function fmtPrice(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}
export function fmtInt(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}
export function fmtPct(v, d = 1) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}
export function fmtRatio(v, d = 3) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}
