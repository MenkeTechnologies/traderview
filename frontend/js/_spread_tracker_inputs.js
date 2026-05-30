// Spread Tracker helpers shared by view + vitest.
//
// Backend body shape: { samples: [{bid, ask}, ...] }.

const TOKEN_DELIM = /[\s,]+/;

import { t } from './i18n.js';

export const REGIME_THRESHOLDS = { tight: 5, normal: 25, wide: 100 };
const REGIME_KEYS_ARR = ['tight', 'normal', 'wide', 'pathological'];
const _regimeTarget = Object.fromEntries(REGIME_KEYS_ARR.map(k => [k, true]));
export const REGIME_LABELS = new Proxy(_regimeTarget, {
    get(_t, key) {
        if (typeof key !== 'string' || !REGIME_KEYS_ARR.includes(key)) return undefined;
        return t(`view.spread_tracker.regime.${key}`);
    },
});
export const REGIME_CSS = {
    tight:        'pos',
    normal:       '',
    wide:         'neg',
    pathological: 'neg',
};

export function parseQuoteBlob(text) {
    const samples = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { samples, errors: [{ line_no: 0, raw: '', message: t('common.parse.input_must_be_string') }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = raw.trim();
        if (!s || s.startsWith('#')) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 2) {
            errors.push({ line_no: i + 1, raw, message: `expected 2 tokens (bid ask), got ${parts.length}` });
            continue;
        }
        const bid = Number(parts[0]);
        const ask = Number(parts[1]);
        if (!Number.isFinite(bid) || bid <= 0) {
            errors.push({ line_no: i + 1, raw, message: `bid must be > 0` });
            continue;
        }
        if (!Number.isFinite(ask) || ask < bid) {
            errors.push({ line_no: i + 1, raw, message: `ask must be ≥ bid` });
            continue;
        }
        samples.push({ bid, ask });
    }
    return { samples, errors };
}

export function validateInputs(samples) {
    if (!Array.isArray(samples) || samples.length < 5)
        return t('view.spread_tracker.validate.samples_min');
    return null;
}

export function buildBody(samples) {
    return { samples };
}

// Computes per-sample spread bps + mid for charting. Mirrors backend
// formula exactly so chart values match the report scalars to 7dp.
export function computeSpreadSeries(samples) {
    const bps = [], mids = [];
    for (const s of samples) {
        if (!Number.isFinite(s.bid) || !Number.isFinite(s.ask) || s.bid <= 0 || s.ask < s.bid) {
            bps.push(null);
            mids.push(null);
            continue;
        }
        const mid = (s.bid + s.ask) / 2;
        if (mid <= 0) { bps.push(null); mids.push(null); continue; }
        bps.push((s.ask - s.bid) / mid * 10_000);
        mids.push(mid);
    }
    return { bps, mids };
}

// Classifies an arbitrary bps value into a regime — used to color
// individual sample dots on the chart, separately from the avg-based
// regime in the backend report.
export function classifyBps(bps) {
    if (!Number.isFinite(bps)) return 'normal';
    if (bps <= REGIME_THRESHOLDS.tight)  return 'tight';
    if (bps <= REGIME_THRESHOLDS.normal) return 'normal';
    if (bps <= REGIME_THRESHOLDS.wide)   return 'wide';
    return 'pathological';
}

// Deterministic demo: 300 samples with a "normal" baseline regime and
// a 20-sample pathological burst near the end (feed glitch / circuit-
// breaker style event). Demonstrates pathological_pct > 0.
export function makeDemoQuotes(seed = 1) {
    let s = seed;
    const rand = () => { s = (s * 1664525 + 1013904223) | 0; return ((s >>> 0) / 0xffffffff); };
    const out = new Array(300);
    let mid = 100;
    for (let i = 0; i < 300; i++) {
        mid = Math.max(0.01, mid * (1 + (rand() - 0.5) * 0.0008));
        const baseSpread = mid * 0.0008 + (rand() - 0.5) * mid * 0.0001;  // ~8 bps ± 1
        const pathological = (i >= 250 && i < 270);
        const halfSpread = (pathological
            ? mid * (0.012 + rand() * 0.008)   // 120-200 bps
            : baseSpread) / 2;
        const bid = Math.max(0.01, mid - halfSpread);
        const ask = mid + halfSpread;
        out[i] = { bid: Number(bid.toFixed(4)), ask: Number(ask.toFixed(4)) };
    }
    return out;
}

export function fmtBps(v) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(1) + ' bps';
}

export function fmtN(v, d = 4) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtPct(v) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(1) + '%';
}
