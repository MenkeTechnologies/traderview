// Range Expansion helpers shared by view + vitest.
//
// Backend body shape: { bars: OhlcBar[], atr: f64[], config:
// {lookback, min_expansion_atrs, prior_atr_max} }. Bars need HLC only
// (no open); ATR must be parallel to bars. View computes ATR locally
// (Wilder smoothing) so the user only pastes raw bars.

import { t } from './i18n.js';

const TOKEN_DELIM = /[\s,]+/;

// Three-token-per-line "high low close".
export function parseBarBlob(text) {
    const bars = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { bars, errors: [{ line_no: 0, raw: '', message: 'input not a string' }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = raw.trim();
        if (!s || s.startsWith('#')) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 3) {
            errors.push({ line_no: i + 1, raw, message: `expected 3 tokens (high low close), got ${parts.length}` });
            continue;
        }
        const h = Number(parts[0]);
        const l = Number(parts[1]);
        const c = Number(parts[2]);
        if (![h, l, c].every(Number.isFinite) || h <= 0 || l <= 0 || c <= 0) {
            errors.push({ line_no: i + 1, raw, message: `HLC must be positive finite` });
            continue;
        }
        if (l > h + 1e-9) {
            errors.push({ line_no: i + 1, raw, message: `low > high` });
            continue;
        }
        if (c < l - 1e-9 || c > h + 1e-9) {
            errors.push({ line_no: i + 1, raw, message: `close outside [low, high]` });
            continue;
        }
        bars.push({ high: h, low: l, close: c });
    }
    return { bars, errors };
}

// True Range per bar — max of (H-L, |H - prev_close|, |L - prev_close|).
// First bar has no prior close, fall back to H-L.
export function trueRange(bars) {
    if (!Array.isArray(bars) || bars.length === 0) return [];
    const out = new Array(bars.length);
    out[0] = bars[0].high - bars[0].low;
    for (let i = 1; i < bars.length; i++) {
        const pc = bars[i - 1].close;
        const a = bars[i].high - bars[i].low;
        const b = Math.abs(bars[i].high - pc);
        const c = Math.abs(bars[i].low - pc);
        out[i] = Math.max(a, b, c);
    }
    return out;
}

// Wilder ATR: SMA of TR for the first `period` bars, then RMA recursion
// ATR_t = (ATR_{t-1} * (period - 1) + TR_t) / period.
export function computeAtr(bars, period = 14) {
    if (!Array.isArray(bars) || bars.length === 0 || !Number.isInteger(period) || period <= 0) return [];
    const tr = trueRange(bars);
    const out = new Array(bars.length).fill(NaN);
    if (bars.length < period) return out;
    let sum = 0;
    for (let i = 0; i < period; i++) sum += tr[i];
    out[period - 1] = sum / period;
    for (let i = period; i < bars.length; i++) {
        out[i] = (out[i - 1] * (period - 1) + tr[i]) / period;
    }
    return out;
}

export function validateInputs(bars, atr, cfg) {
    if (!Array.isArray(bars) || bars.length < cfg.lookback + 1)
        return `need at least ${cfg.lookback + 1} bars (lookback + 1)`;
    if (!Array.isArray(atr) || atr.length !== bars.length)
        return 'atr series length must equal bars length';
    if (!Number.isInteger(cfg.lookback) || cfg.lookback < 1)
        return 'lookback must be integer ≥ 1';
    if (!Number.isFinite(cfg.min_expansion_atrs) || cfg.min_expansion_atrs <= 0)
        return 'min_expansion_atrs must be > 0';
    if (!Number.isFinite(cfg.prior_atr_max) || cfg.prior_atr_max <= 0)
        return 'prior_atr_max must be > 0';
    if (cfg.prior_atr_max >= cfg.min_expansion_atrs)
        return 'prior_atr_max must be < min_expansion_atrs (compression < expansion)';
    return null;
}

export function buildBody(bars, atr, config) {
    return { bars, atr, config };
}

// Maps backend direction enum to label + color.
const DIR_BADGES = {
    up:   { key: 'up',   cls: 'pos' },
    down: { key: 'down', cls: 'neg' },
};
export function dirBadge(d) {
    const x = DIR_BADGES[d];
    if (!x) return { label: String(d || '—'), cls: '' };
    return { label: t(`view.range_expansion.dir.${x.key}`), cls: x.cls };
}

// Spreads events into parallel up/down null-padded series for uPlot
// markers. Up = above the bar's high, Down = below the bar's low.
export function eventMarkers(events, bars) {
    const up = new Array(bars.length).fill(null);
    const dn = new Array(bars.length).fill(null);
    if (!Array.isArray(events)) return { up, dn };
    for (const e of events) {
        if (!Number.isInteger(e.bar_index) || e.bar_index < 0 || e.bar_index >= bars.length) continue;
        const bar = bars[e.bar_index];
        if (!bar) continue;
        if (e.direction === 'up')   up[e.bar_index] = bar.high * 1.002;
        if (e.direction === 'down') dn[e.bar_index] = bar.low  * 0.998;
    }
    return { up, dn };
}

// Deterministic demo: 30-bar series with explicit compression bars
// 18-21 (very narrow), then a wide expansion bar at index 22 that
// breaks UP, plus a second compression-then-down break later.
export function makeDemoBars() {
    const out = [];
    let price = 100;
    // 18 bars of normal-volatility drift.
    for (let i = 0; i < 18; i++) {
        const delta = i % 3 === 0 ? 0.4 : -0.2;
        price += delta;
        const h = price + 0.5;
        const l = price - 0.5;
        out.push({ high: h, low: l, close: price });
    }
    // 4 narrow-range bars (compression).
    for (let i = 0; i < 4; i++) {
        const h = price + 0.1;
        const l = price - 0.1;
        out.push({ high: h, low: l, close: price });
    }
    // Wide-range expansion bar UP.
    {
        const h = price + 2.5;
        const l = price - 0.2;
        price = h - 0.1;
        out.push({ high: h, low: l, close: price });
    }
    // 4 more bars of normal drift.
    for (let i = 0; i < 4; i++) {
        price += 0.2;
        out.push({ high: price + 0.4, low: price - 0.4, close: price });
    }
    // 2 more narrow compression bars.
    for (let i = 0; i < 2; i++) {
        out.push({ high: price + 0.1, low: price - 0.1, close: price });
    }
    // Wide-range expansion bar DOWN.
    {
        const h = price + 0.2;
        const l = price - 2.5;
        price = l + 0.1;
        out.push({ high: h, low: l, close: price });
    }
    return out;
}

export function fmtN(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}
