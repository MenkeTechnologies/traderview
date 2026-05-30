// Chandelier Stop helpers shared by view + vitest.
//
// Backend body shape: { bars: [{high, low, close}, ...], atr: f64[],
// side: "long"|"short", config: {lookback, atr_multiplier} }. Returns
// StopPoint[] with {stop_price, triggered} per bar.
//
// Computes the ATR locally (Wilder smoothing) so the user only pastes
// raw HLC — same pattern as Range Expansion view.

import { t } from './i18n.js';

const TOKEN_DELIM = /[\s,]+/;

// Three-token-per-line "high low close" with full OHLC sanity.
export function parseBarBlob(text) {
    const bars = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { bars, errors: [{ line_no: 0, raw: '', message: t('common.parse.input_must_be_string') }] };
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

// True Range per bar (max of H-L, |H-prev_close|, |L-prev_close|).
export function trueRange(bars) {
    if (!Array.isArray(bars) || bars.length === 0) return [];
    const out = new Array(bars.length);
    out[0] = bars[0].high - bars[0].low;
    for (let i = 1; i < bars.length; i++) {
        const pc = bars[i - 1].close;
        out[i] = Math.max(
            bars[i].high - bars[i].low,
            Math.abs(bars[i].high - pc),
            Math.abs(bars[i].low - pc),
        );
    }
    return out;
}

// Wilder ATR. SMA seed for first `period` bars then RMA recursion.
// Mirrors Range Expansion view's helper — identical formula.
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

export function validateInputs(bars, atr, side, cfg) {
    if (!Array.isArray(bars) || bars.length === 0) return t('view.chandelier_stop.validate.need_bar');
    if (!Array.isArray(atr) || atr.length !== bars.length) return t('view.chandelier_stop.validate.atr_length');
    if (side !== 'long' && side !== 'short') return t('view.chandelier_stop.validate.side');
    if (!Number.isInteger(cfg.lookback) || cfg.lookback < 1) return t('view.chandelier_stop.validate.lookback');
    if (!Number.isFinite(cfg.atr_multiplier) || cfg.atr_multiplier <= 0)
        return t('view.chandelier_stop.validate.atr_multiplier');
    if (bars.length < cfg.lookback) return t('view.chandelier_stop.validate.bars_lt_lookback', { n: cfg.lookback });
    return null;
}

export function buildBody(bars, atr, side, cfg) {
    return { bars, atr, side, config: cfg };
}

// Splits the backend's StopPoint[] into a stop-price series + boolean
// trigger flags, with optional null-padding where the lookback warmup
// returns the default `{stop_price: 0, triggered: false}`. We treat
// `triggered=false && stop_price==0` as warmup and emit null so uPlot
// draws a gap instead of plotting at zero.
export function splitStops(stops) {
    const stopPrice = [];
    const triggers = [];
    if (!Array.isArray(stops)) return { stopPrice, triggers };
    for (const s of stops) {
        if (!s || (s.stop_price === 0 && !s.triggered)) {
            stopPrice.push(null);
            triggers.push(false);
        } else {
            stopPrice.push(s.stop_price);
            triggers.push(!!s.triggered);
        }
    }
    return { stopPrice, triggers };
}

// Per-bar trigger markers — non-null only where trigger fires. Used
// for the uPlot dot overlay.
export function triggerMarkers(stops, bars) {
    const out = new Array(bars.length).fill(null);
    if (!Array.isArray(stops) || !Array.isArray(bars)) return out;
    for (let i = 0; i < Math.min(stops.length, bars.length); i++) {
        if (stops[i] && stops[i].triggered) {
            out[i] = bars[i].close;
        }
    }
    return out;
}

// Summary scalars across the stop series.
export function summarize(stops, bars, side) {
    const out = { latestStop: NaN, latestClose: NaN, distancePct: NaN,
                  triggerCount: 0, firstTriggerIdx: -1 };
    if (!Array.isArray(stops) || !Array.isArray(bars) || bars.length === 0) return out;
    out.latestClose = bars[bars.length - 1].close;
    for (let i = 0; i < stops.length; i++) {
        if (stops[i] && stops[i].triggered) {
            out.triggerCount++;
            if (out.firstTriggerIdx < 0) out.firstTriggerIdx = i;
        }
    }
    // Find the most recent non-warmup stop price.
    for (let i = stops.length - 1; i >= 0; i--) {
        const s = stops[i];
        if (s && (s.stop_price !== 0 || s.triggered)) {
            out.latestStop = s.stop_price;
            break;
        }
    }
    if (Number.isFinite(out.latestStop) && out.latestClose > 0) {
        out.distancePct = side === 'long'
            ? (out.latestClose - out.latestStop) / out.latestClose
            : (out.latestStop - out.latestClose) / out.latestClose;
    }
    return out;
}

// Deterministic 60-bar demo: 40 bars of strong uptrend → 20 bars of
// reversal. With default config (lookback=22, multiplier=3.0) the
// long-chandelier stop trails up during the rally and triggers when
// the reversal pulls back through it.
export function makeDemoBars() {
    const out = [];
    let price = 100;
    for (let i = 0; i < 40; i++) {
        price += 0.5;
        out.push({ high: price + 0.4, low: price - 0.4, close: price });
    }
    for (let i = 0; i < 20; i++) {
        price -= 0.6;
        out.push({ high: price + 0.4, low: price - 0.4, close: price });
    }
    return out;
}

export function fmtN(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtPct(v) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(2) + '%';
}
