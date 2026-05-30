// AlphaTrend (Kivanc Ozbilgic 2021) helpers.
//
// Backend body: { bars: Bar[], period: usize, multiplier: f64 }
//   where Bar = { high, low, close }
// Returns: {
//   alpha: (number|null)[], direction: (number|null)[],
//   period, multiplier
// }
//
// Trailing trend line built from ATR (SMA-of-TR) + Wilder RSI gate:
//   r ≥ 50 → max(prev_alpha, low − mult·atr)   (ratchet up)
//   r < 50 → min(prev_alpha, high + mult·atr)  (ratchet down)

import { t } from './i18n.js';

export const DEFAULT_PERIOD = 14;
export const DEFAULT_MULTIPLIER = 1.0;
export const MIN_PERIOD = 2;
export const MAX_PERIOD = 500;

export const DEFAULT_INPUTS = {
    bars: [],
    period: DEFAULT_PERIOD,
    multiplier: DEFAULT_MULTIPLIER,
};

export function validateInputs(input) {
    if (!Array.isArray(input.bars))                  return t('view.alphatrend.validate.bars_array');
    if (!Number.isInteger(input.period))             return t('view.alphatrend.validate.period_int');
    if (input.period < MIN_PERIOD || input.period > MAX_PERIOD)
                                                      return t('view.alphatrend.validate.period_range', { min: MIN_PERIOD, max: MAX_PERIOD });
    if (!Number.isFinite(input.multiplier) || input.multiplier <= 0)
                                                      return t('view.alphatrend.validate.multiplier');
    if (input.bars.length < input.period + 1)         return t('view.alphatrend.validate.bars_min', { n: input.period + 1 });
    for (let i = 0; i < input.bars.length; i++) {
        const b = input.bars[i];
        if (!b || !Number.isFinite(b.high) || !Number.isFinite(b.low) || !Number.isFinite(b.close))
                                                      return t('view.alphatrend.validate.bar_finite', { i });
        if (b.high < b.low)                           return t('view.alphatrend.validate.high_lt_low', { i });
        if (b.close < b.low || b.close > b.high)      return t('view.alphatrend.validate.close_outside', { i });
    }
    return null;
}

export function buildBody(input) {
    return {
        bars: input.bars.map(b => ({ high: b.high, low: b.low, close: b.close })),
        period: input.period,
        multiplier: input.multiplier,
    };
}

// Pure-JS mirror of crates/traderview-core/src/alphatrend.rs::compute.
export function localCompute(bars, period, multiplier) {
    const n = bars.length;
    const report = {
        alpha: new Array(n).fill(null),
        direction: new Array(n).fill(null),
        period, multiplier,
    };
    if (period < 2 || !Number.isFinite(multiplier) || multiplier <= 0 || n < period + 1) return report;
    for (const b of bars) {
        if (!b || !Number.isFinite(b.high) || !Number.isFinite(b.low) || !Number.isFinite(b.close)) return report;
    }
    const p_f = period;
    // ATR via SMA of TR.
    const tr = new Array(n);
    tr[0] = bars[0].high - bars[0].low;
    for (let i = 1; i < n; i++) {
        const pc = bars[i - 1].close;
        tr[i] = Math.max(
            bars[i].high - bars[i].low,
            Math.abs(bars[i].high - pc),
            Math.abs(bars[i].low - pc),
        );
    }
    const atr = new Array(n).fill(null);
    let sum = 0;
    for (let i = 0; i < period; i++) sum += tr[i];
    atr[period - 1] = sum / p_f;
    for (let i = period; i < n; i++) {
        sum += tr[i] - tr[i - period];
        atr[i] = sum / p_f;
    }
    const rsi = wilderRsi(bars.map(b => b.close), period);
    let last = null;
    for (let i = 0; i < n; i++) {
        const a = atr[i], r = rsi[i];
        if (a == null || r == null) continue;
        const up = bars[i].low - multiplier * a;
        const dn = bars[i].high + multiplier * a;
        let next;
        if (last == null) {
            next = r >= 50 ? up : dn;
        } else {
            next = r >= 50 ? Math.max(up, last) : Math.min(dn, last);
        }
        const dir = last == null ? 0 : (next > last ? 1 : next < last ? -1 : 0);
        report.alpha[i] = next;
        report.direction[i] = dir;
        last = next;
    }
    return report;
}

export function wilderRsi(closes, period) {
    const n = closes.length;
    const out = new Array(n).fill(null);
    if (period === 0 || n < period + 1) return out;
    const p_f = period;
    let sum_g = 0, sum_l = 0;
    for (let i = 1; i <= period; i++) {
        const d = closes[i] - closes[i - 1];
        if (d > 0) sum_g += d; else sum_l -= d;
    }
    let avg_g = sum_g / p_f;
    let avg_l = sum_l / p_f;
    out[period] = rsiOf(avg_g, avg_l);
    for (let i = period + 1; i < n; i++) {
        const d = closes[i] - closes[i - 1];
        const g = Math.max(0, d);
        const l = Math.max(0, -d);
        avg_g = (avg_g * (p_f - 1) + g) / p_f;
        avg_l = (avg_l * (p_f - 1) + l) / p_f;
        out[i] = rsiOf(avg_g, avg_l);
    }
    return out;
}

function rsiOf(g, l) {
    if (l <= 0) return g <= 0 ? 50 : 100;
    const rs = g / l;
    return 100 - 100 / (1 + rs);
}

// Parse "high low close" 3-token-per-line OHL→HLC blob.
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
        if (parts.length !== 3) {
            out.errors.push({ line_no: i + 1, message: `expected 3 tokens (high low close), got ${parts.length}` });
            continue;
        }
        const h = Number(parts[0].replace(/\$/g, ''));
        const l = Number(parts[1].replace(/\$/g, ''));
        const c = Number(parts[2].replace(/\$/g, ''));
        if (!Number.isFinite(h) || !Number.isFinite(l) || !Number.isFinite(c) || h <= 0 || l <= 0 || c <= 0) {
            out.errors.push({ line_no: i + 1, message: `HLC must be positive finite` });
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
        out.bars.push({ high: h, low: l, close: c });
    }
    return out;
}

export function barsToBlob(bars) {
    return bars.map(b => `${b.high} ${b.low} ${b.close}`).join('\n');
}

// Direction verdict on most recent bar.
export function dirBadge(direction) {
    if (!Array.isArray(direction) || direction.length === 0) {
        return { key: 'view.atrend.dir.unknown', cls: '' };
    }
    let last = null;
    for (let i = direction.length - 1; i >= 0; i--) {
        if (direction[i] != null) { last = direction[i]; break; }
    }
    if (last == null) return { key: 'view.atrend.dir.unknown', cls: '' };
    if (last > 0)  return { key: 'view.atrend.dir.up',   cls: 'pos' };
    if (last < 0)  return { key: 'view.atrend.dir.down', cls: 'neg' };
    return { key: 'view.atrend.dir.flat', cls: '' };
}

// Trend strength over last N bars by ratio of up vs down ticks.
export function trendBadge(direction, lookback = 10) {
    if (!Array.isArray(direction) || direction.length === 0) {
        return { key: 'view.atrend.trend.unknown', cls: '' };
    }
    const tail = [];
    for (let i = direction.length - 1; i >= 0 && tail.length < lookback; i--) {
        if (direction[i] != null) tail.unshift(direction[i]);
    }
    if (tail.length === 0) return { key: 'view.atrend.trend.unknown', cls: '' };
    let up = 0, down = 0, flat = 0;
    for (const d of tail) {
        if (d > 0) up++;
        else if (d < 0) down++;
        else flat++;
    }
    const total = tail.length;
    if (up / total >= 0.8)   return { key: 'view.atrend.trend.strong_up',   cls: 'pos' };
    if (up / total >= 0.6)   return { key: 'view.atrend.trend.up',          cls: 'pos' };
    if (down / total >= 0.8) return { key: 'view.atrend.trend.strong_down', cls: 'neg' };
    if (down / total >= 0.6) return { key: 'view.atrend.trend.down',        cls: 'neg' };
    if (flat / total >= 0.5) return { key: 'view.atrend.trend.flat',        cls: '' };
    return { key: 'view.atrend.trend.mixed', cls: '' };
}

// Close vs last alpha (above/below the trailing line).
export function positionBadge(close, alpha_last) {
    if (close == null || alpha_last == null
        || !Number.isFinite(close) || !Number.isFinite(alpha_last)) {
        return { key: 'view.atrend.pos.unknown', cls: '' };
    }
    if (alpha_last === 0) return { key: 'view.atrend.pos.unknown', cls: '' };
    const rel = (close - alpha_last) / Math.abs(alpha_last);
    if (rel > 0.02)  return { key: 'view.atrend.pos.well_above', cls: 'pos' };
    if (rel > 0)     return { key: 'view.atrend.pos.above',      cls: 'pos' };
    if (rel < -0.02) return { key: 'view.atrend.pos.well_below', cls: 'neg' };
    if (rel < 0)     return { key: 'view.atrend.pos.below',      cls: 'neg' };
    return { key: 'view.atrend.pos.at', cls: '' };
}

// Bar-series summary (close-side stats).
export function summarizeBars(bars) {
    if (!Array.isArray(bars) || bars.length === 0) {
        return { count: 0, last_close: NaN, min_low: NaN, max_high: NaN, mean_close: NaN };
    }
    let sum = 0, mxH = -Infinity, mnL = Infinity;
    for (const b of bars) {
        sum += b.close;
        if (b.high > mxH) mxH = b.high;
        if (b.low  < mnL) mnL = b.low;
    }
    return {
        count: bars.length,
        last_close: bars[bars.length - 1].close,
        min_low: Number.isFinite(mnL) ? mnL : NaN,
        max_high: Number.isFinite(mxH) ? mxH : NaN,
        mean_close: sum / bars.length,
    };
}

function lcg(seed) {
    let state = BigInt(7919) + seed;
    return () => {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(state >> 32n) / 0xFFFFFFFF;
    };
}

function mkBar(close, range = 1.0, rand) {
    const r = rand ? rand() : 0.5;
    const h = close + range * r;
    const l = close - range * (1 - r);
    return { high: h, low: l, close };
}

export function makeDemoInput(kind = 'uptrend') {
    switch (kind) {
        case 'uptrend': {
            const rand = lcg(42n);
            return {
                bars: Array.from({ length: 60 }, (_, i) => mkBar(100 + i * 0.8, 0.6, rand)),
                period: 14, multiplier: 1.0,
            };
        }
        case 'downtrend': {
            const rand = lcg(7n);
            return {
                bars: Array.from({ length: 60 }, (_, i) => mkBar(160 - i * 0.8, 0.6, rand)),
                period: 14, multiplier: 1.0,
            };
        }
        case 'reversal-up': {
            const rand = lcg(11n);
            const bars = [];
            for (let i = 0; i < 30; i++) bars.push(mkBar(120 - i * 0.6, 0.5, rand));
            for (let i = 0; i < 30; i++) bars.push(mkBar(102 + i * 0.8, 0.5, rand));
            return { bars, period: 14, multiplier: 1.0 };
        }
        case 'reversal-down': {
            const rand = lcg(13n);
            const bars = [];
            for (let i = 0; i < 30; i++) bars.push(mkBar(100 + i * 0.6, 0.5, rand));
            for (let i = 0; i < 30; i++) bars.push(mkBar(118 - i * 0.8, 0.5, rand));
            return { bars, period: 14, multiplier: 1.0 };
        }
        case 'sideways': {
            const rand = lcg(21n);
            return {
                bars: Array.from({ length: 60 }, () => mkBar(100 + (rand() - 0.5) * 1.5, 0.4, rand)),
                period: 14, multiplier: 1.0,
            };
        }
        case 'volatile': {
            const rand = lcg(33n);
            return {
                bars: Array.from({ length: 60 }, (_, i) => mkBar(100 + Math.sin(i * 0.3) * 6, 1.5, rand)),
                period: 14, multiplier: 1.5,
            };
        }
        case 'high-mult': {
            // Wider trail — fewer direction flips, harder to trip.
            const rand = lcg(57n);
            return {
                bars: Array.from({ length: 60 }, (_, i) => mkBar(100 + i * 0.5 + Math.sin(i * 0.5) * 2, 1, rand)),
                period: 14, multiplier: 3.0,
            };
        }
        case 'low-mult': {
            // Tight trail — frequent direction flips.
            const rand = lcg(99n);
            return {
                bars: Array.from({ length: 60 }, (_, i) => mkBar(100 + i * 0.5 + Math.sin(i * 0.5) * 2, 1, rand)),
                period: 14, multiplier: 0.3,
            };
        }
        default: return makeDemoInput('uptrend');
    }
}

export function fmtPrice(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtPriceSigned(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + v.toFixed(d);
}

export function fmtPct(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtInt(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}

export function fmtDir(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    if (v > 0) return '↑';
    if (v < 0) return '↓';
    return '·';
}
