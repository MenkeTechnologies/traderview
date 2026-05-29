// Heikin-Ashi Reversal helpers shared by view + vitest.
//
// Backend body shape: { bars: HaBar[], config: FlipConfig }. The view
// accepts standard OHLC (what users actually have) and computes HA
// candles locally before posting — saves users from converting by hand
// AND lets us render both the raw price + the HA series with flip
// markers in one chart.

import { t } from './i18n.js';

const TOKEN_DELIM = /[\s,]+/;

// Four-token-per-line "open high low close".
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
        if (parts.length !== 4) {
            errors.push({ line_no: i + 1, raw, message: `expected 4 tokens (open high low close), got ${parts.length}` });
            continue;
        }
        const o = Number(parts[0]);
        const h = Number(parts[1]);
        const l = Number(parts[2]);
        const c = Number(parts[3]);
        if (![o, h, l, c].every(Number.isFinite) || o <= 0 || h <= 0 || l <= 0 || c <= 0) {
            errors.push({ line_no: i + 1, raw, message: `OHLC must be positive finite` });
            continue;
        }
        if (l > h + 1e-9) {
            errors.push({ line_no: i + 1, raw, message: `low > high` });
            continue;
        }
        if (o < l - 1e-9 || o > h + 1e-9 || c < l - 1e-9 || c > h + 1e-9) {
            errors.push({ line_no: i + 1, raw, message: `open / close outside [low, high]` });
            continue;
        }
        bars.push({ open: o, high: h, low: l, close: c });
    }
    return { bars, errors };
}

// Standard Heikin-Ashi formula. Returns parallel `{ha, color}` arrays.
// HA_close = (O+H+L+C) / 4
// HA_open  = (prev_HA_open + prev_HA_close) / 2  (first bar: (O+C)/2)
// HA_high  = max(H, HA_open, HA_close)
// HA_low   = min(L, HA_open, HA_close)
export function computeHeikinAshi(bars) {
    const out = [];
    if (!Array.isArray(bars) || bars.length === 0) return out;
    let prevHaOpen = (bars[0].open + bars[0].close) / 2;
    let prevHaClose = (bars[0].open + bars[0].high + bars[0].low + bars[0].close) / 4;
    out.push({
        open: prevHaOpen,
        close: prevHaClose,
        high: Math.max(bars[0].high, prevHaOpen, prevHaClose),
        low:  Math.min(bars[0].low,  prevHaOpen, prevHaClose),
    });
    for (let i = 1; i < bars.length; i++) {
        const b = bars[i];
        const haOpen = (prevHaOpen + prevHaClose) / 2;
        const haClose = (b.open + b.high + b.low + b.close) / 4;
        const haHigh = Math.max(b.high, haOpen, haClose);
        const haLow  = Math.min(b.low,  haOpen, haClose);
        out.push({ open: haOpen, high: haHigh, low: haLow, close: haClose });
        prevHaOpen = haOpen;
        prevHaClose = haClose;
    }
    return out;
}

export function validateInputs(bars, cfg) {
    if (!Array.isArray(bars) || bars.length < 2) return t('view.ha_reversal.validate.bars_min');
    if (!Number.isFinite(cfg.min_body_ratio) || cfg.min_body_ratio < 0 || cfg.min_body_ratio > 1)
        return t('view.ha_reversal.validate.min_body_ratio');
    if (!Number.isInteger(cfg.strong_streak) || cfg.strong_streak < 1)
        return t('view.ha_reversal.validate.strong_streak');
    if (!Number.isInteger(cfg.weak_streak)   || cfg.weak_streak < 1)
        return t('view.ha_reversal.validate.weak_streak');
    if (cfg.weak_streak > cfg.strong_streak)
        return t('view.ha_reversal.validate.weak_le_strong');
    return null;
}

export function buildBody(bars, config) {
    return { bars: computeHeikinAshi(bars), config };
}

// Maps backend flip enum + strength into UI badges.
const DIR_LABEL = {
    bullish_to_bearish: { key: 'bull_to_bear', cls: 'neg' },
    bearish_to_bullish: { key: 'bear_to_bull', cls: 'pos' },
};
const STRENGTH_LABEL = {
    strong: { key: 'strong', cls: 'pos' },
    weak:   { key: 'weak',   cls: '' },
};

export function dirBadge(d) {
    const x = DIR_LABEL[d];
    if (!x) return { label: String(d || '—'), cls: '' };
    return { label: t(`view.ha_reversal.dir.${x.key}`), cls: x.cls };
}
export function strengthBadge(s) {
    const x = STRENGTH_LABEL[s];
    if (!x) return { label: String(s || '—'), cls: '' };
    return { label: t(`view.ha_reversal.strength.${x.key}`), cls: x.cls };
}

// Splits backend events into parallel up/down arrays anchored at the
// bar_index for uPlot marker plotting (null elsewhere).
export function eventMarkers(events, haBars) {
    const up = new Array(haBars.length).fill(null);
    const dn = new Array(haBars.length).fill(null);
    if (!Array.isArray(events)) return { up, dn };
    for (const e of events) {
        if (!Number.isInteger(e.bar_index) || e.bar_index < 0 || e.bar_index >= haBars.length) continue;
        const haBar = haBars[e.bar_index];
        if (!haBar) continue;
        if (e.direction === 'bearish_to_bullish') up[e.bar_index] = haBar.low * 0.998;
        if (e.direction === 'bullish_to_bearish') dn[e.bar_index] = haBar.high * 1.002;
    }
    return { up, dn };
}

// Deterministic demo: 30-bar OHLC series with 3 distinct regimes — long
// bullish run, sharp bearish reversal, recovery bullish run. Engineered
// to fire at least one Strong flip in each direction at default config.
export function makeDemoBars(seed = 1) {
    let s = seed;
    const rand = () => { s = (s * 1664525 + 1013904223) | 0; return ((s >>> 0) / 0xffffffff); };
    const noise = (amp) => (rand() * 2 - 1) * amp;
    const out = [];
    let price = 100;
    // 10 bars of strong uptrend.
    for (let i = 0; i < 10; i++) {
        const drift = 0.6 + noise(0.15);
        const o = price;
        const c = o + drift;
        const h = Math.max(o, c) + Math.abs(noise(0.3));
        const l = Math.min(o, c) - Math.abs(noise(0.3));
        out.push({ open: round2(o), high: round2(h), low: round2(l), close: round2(c) });
        price = c;
    }
    // 1 large reversal bar — explicit large bearish body to trigger a Strong flip.
    {
        const o = price;
        const c = o - 4.5;
        const h = o + 0.1;
        const l = c - 0.2;
        out.push({ open: round2(o), high: round2(h), low: round2(l), close: round2(c) });
        price = c;
    }
    // 9 bars of strong downtrend.
    for (let i = 0; i < 9; i++) {
        const drift = -0.55 + noise(0.15);
        const o = price;
        const c = o + drift;
        const h = Math.max(o, c) + Math.abs(noise(0.3));
        const l = Math.min(o, c) - Math.abs(noise(0.3));
        out.push({ open: round2(o), high: round2(h), low: round2(l), close: round2(c) });
        price = c;
    }
    // 1 large reversal-back-up bar.
    {
        const o = price;
        const c = o + 4.5;
        const h = c + 0.2;
        const l = o - 0.1;
        out.push({ open: round2(o), high: round2(h), low: round2(l), close: round2(c) });
        price = c;
    }
    // 9 bars of recovery uptrend.
    for (let i = 0; i < 9; i++) {
        const drift = 0.55 + noise(0.15);
        const o = price;
        const c = o + drift;
        const h = Math.max(o, c) + Math.abs(noise(0.3));
        const l = Math.min(o, c) - Math.abs(noise(0.3));
        out.push({ open: round2(o), high: round2(h), low: round2(l), close: round2(c) });
        price = c;
    }
    return out;
}

function round2(v) { return Math.round(v * 100) / 100; }

export function fmtN(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtPct(v) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(1) + '%';
}
