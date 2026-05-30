// DeMarker Oscillator helpers shared by view + vitest.
//
// Backend body shape: { highs: f64[], lows: f64[], period: usize }.
// Returns Vec<Option<f64>> — null until warmup, then values in [0, 1].

import { t } from './i18n.js';

const TOKEN_DELIM = /[\s,]+/;

// Two-token-per-line "high low" — same convention as the Alligator view
// to keep the input format consistent across HL-only indicators.
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
        if (parts.length !== 2) {
            errors.push({ line_no: i + 1, raw, message: `expected 2 tokens (high low), got ${parts.length}` });
            continue;
        }
        const h = Number(parts[0]);
        const l = Number(parts[1]);
        if (!Number.isFinite(h) || !Number.isFinite(l) || h <= 0 || l <= 0) {
            errors.push({ line_no: i + 1, raw, message: `HL must be positive finite` });
            continue;
        }
        if (l > h + 1e-9) {
            errors.push({ line_no: i + 1, raw, message: `low > high` });
            continue;
        }
        bars.push({ high: h, low: l });
    }
    return { bars, errors };
}

export function validateInputs(bars, period) {
    if (!Array.isArray(bars) || bars.length === 0) return t('view.demarker.validate.bars_empty');
    if (!Number.isInteger(period) || period < 2) return t('view.demarker.validate.period_min');
    if (bars.length < period + 1) return t('view.demarker.validate.bars_min', { n: period + 1 });
    return null;
}

export function buildBody(bars, period) {
    return {
        highs: bars.map(b => b.high),
        lows:  bars.map(b => b.low),
        period,
    };
}

// Threshold constants — match Tom DeMark's published cuts (0.7 / 0.3).
export const OB_THRESHOLD = 0.7;
export const OS_THRESHOLD = 0.3;

// Single-value regime classifier.
export function regimeOf(v) {
    if (!Number.isFinite(v)) return 'unknown';
    if (v >= OB_THRESHOLD) return 'overbought';
    if (v <= OS_THRESHOLD) return 'oversold';
    return 'neutral';
}

const REGIME_BADGES = {
    overbought: { key: 'overbought', cls: 'neg' },
    oversold:   { key: 'oversold',   cls: 'pos' },
    neutral:    { key: 'neutral',    cls: '' },
    unknown:    { key: 'unknown',    cls: '' },
};

export function regimeBadge(r) {
    const x = REGIME_BADGES[r];
    if (!x) return { label: String(r || '—'), cls: '', hint: '' };
    return {
        label: t(`view.demarker.regime.${x.key}.label`),
        cls: x.cls,
        hint: t(`view.demarker.regime.${x.key}.hint`),
    };
}

// Aggregate counts across the series (including unknown for warmup).
export function regimeCounts(values) {
    const out = { overbought: 0, oversold: 0, neutral: 0, unknown: 0 };
    if (!Array.isArray(values)) return out;
    for (const v of values) out[regimeOf(v)]++;
    return out;
}

// Detects each crossing into/out of the OB/OS regions. An event is the
// FIRST bar where the regime changes from neutral/unknown into ob or os.
// Used to surface "alert candidates" in the event log.
export function detectCrossings(values) {
    const events = [];
    if (!Array.isArray(values)) return events;
    let prev = 'unknown';
    for (let i = 0; i < values.length; i++) {
        const cur = regimeOf(values[i]);
        if ((cur === 'overbought' || cur === 'oversold') && cur !== prev) {
            events.push({ bar_index: i, regime: cur, value: values[i] });
        }
        prev = cur;
    }
    return events;
}

// Pulls the most recent finite value + its index. Used by the "current"
// summary card so the trader sees the most-recent reading at a glance
// even after a warmup-padded series.
export function latestValue(values) {
    if (!Array.isArray(values)) return { index: -1, value: NaN };
    for (let i = values.length - 1; i >= 0; i--) {
        if (Number.isFinite(values[i])) return { index: i, value: values[i] };
    }
    return { index: -1, value: NaN };
}

// Deterministic 60-bar demo that produces clear OB and OS readings under
// default period 14. Phases: 20 bars of strong uptrend (drives DeMarker
// → OB), 20 bars of sideways pullback, 20 bars of strong downtrend
// (drives DeMarker → OS).
export function makeDemoBars() {
    const out = [];
    let price = 100;
    for (let i = 0; i < 20; i++) {
        price += 0.8;
        out.push({ high: price + 0.3, low: price - 0.3 });
    }
    for (let i = 0; i < 20; i++) {
        const delta = i % 2 === 0 ? 0.1 : -0.1;
        price += delta;
        out.push({ high: price + 0.3, low: price - 0.3 });
    }
    for (let i = 0; i < 20; i++) {
        price -= 0.8;
        out.push({ high: price + 0.3, low: price - 0.3 });
    }
    return out;
}

export function fmtN(v, d = 4) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtPct(v) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(1) + '%';
}
