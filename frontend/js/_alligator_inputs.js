// Williams Alligator helpers shared by view + vitest.
//
// Backend body shape: { bars: [{high, low}, ...] }. Returns parallel
// AlligatorPoint[] with jaw/teeth/lips/sleeping per bar — UNSHIFTED.
// The view applies canonical forward shifts (jaw +8, teeth +5, lips +3)
// for the chart rendering only; backend math stays raw.

import { t } from './i18n.js';

const TOKEN_DELIM = /[\s,]+/;

// Two-token-per-line "high low".
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

export function validateInputs(bars) {
    // SMMA(13) needs at least 13 bars to seed; require ≥21 (13 + jaw shift 8)
    // so the shifted display has at least 1 plotted jaw value.
    if (!Array.isArray(bars) || bars.length < 21) return 'need at least 21 bars (13 SMMA + 8 jaw shift)';
    return null;
}

export function buildBody(bars) {
    return { bars };
}

// Williams's canonical forward shifts at display time.
export const SHIFTS = { jaw: 8, teeth: 5, lips: 3 };

// Apply the forward shift to each line: a point at index i is plotted
// at i + shift on the chart. Returns parallel arrays usable directly
// by uPlot: jaw[i] is the jaw value to plot at bar-index i. Out-of-range
// destination slots are null (uPlot draws gaps).
export function shiftLines(points, totalBars) {
    const out = {
        jaw:   new Array(totalBars).fill(null),
        teeth: new Array(totalBars).fill(null),
        lips:  new Array(totalBars).fill(null),
    };
    if (!Array.isArray(points)) return out;
    for (let i = 0; i < points.length; i++) {
        const p = points[i];
        if (!p) continue;
        const ji = i + SHIFTS.jaw;
        const ti = i + SHIFTS.teeth;
        const li = i + SHIFTS.lips;
        if (ji < totalBars && Number.isFinite(p.jaw)   && p.jaw   !== 0) out.jaw[ji]   = p.jaw;
        if (ti < totalBars && Number.isFinite(p.teeth) && p.teeth !== 0) out.teeth[ti] = p.teeth;
        if (li < totalBars && Number.isFinite(p.lips)  && p.lips  !== 0) out.lips[li]  = p.lips;
    }
    return out;
}

// Classifier mirrors backend `classify` enum exactly. Operates on a
// single AlligatorPoint to produce up / down / sleeping verdict.
export function classifyPoint(p) {
    if (!p || p.sleeping) return 'sleeping';
    if (p.lips > p.teeth && p.teeth > p.jaw) return 'up';
    if (p.lips < p.teeth && p.teeth < p.jaw) return 'down';
    return 'sleeping';
}

const BIAS_BADGES = {
    up:       { key: 'up',       cls: 'pos' },
    down:     { key: 'down',     cls: 'neg' },
    sleeping: { key: 'sleeping', cls: '' },
};
export function biasBadge(b) {
    const x = BIAS_BADGES[b];
    if (!x) return { label: String(b || '—'), cls: '', hint: '' };
    return {
        label: t(`view.alligator.bias.${x.key}.label`),
        cls: x.cls,
        hint: t(`view.alligator.bias.${x.key}.hint`),
    };
}

// Aggregate counts per bias across the full series.
export function biasCounts(points) {
    const counts = { up: 0, down: 0, sleeping: 0 };
    if (!Array.isArray(points)) return counts;
    for (const p of points) counts[classifyPoint(p)]++;
    return counts;
}

// 50-bar deterministic demo with explicit phases: 15 bars sleeping
// (tight chop around 100), 15 bars strong uptrend, 5 bars sleeping
// pause, 15 bars strong downtrend — guarantees the bias series shows
// all three regimes.
export function makeDemoBars() {
    const out = [];
    let price = 100;
    // Phase 1: 15 sleeping bars — tight oscillation.
    for (let i = 0; i < 15; i++) {
        const noise = (i % 2 === 0 ? 0.05 : -0.05);
        price += noise;
        out.push({ high: price + 0.2, low: price - 0.2 });
    }
    // Phase 2: 15 strong uptrend bars.
    for (let i = 0; i < 15; i++) {
        price += 0.8;
        out.push({ high: price + 0.5, low: price - 0.5 });
    }
    // Phase 3: 5 sleeping pause.
    for (let i = 0; i < 5; i++) {
        out.push({ high: price + 0.2, low: price - 0.2 });
    }
    // Phase 4: 15 strong downtrend bars.
    for (let i = 0; i < 15; i++) {
        price -= 0.8;
        out.push({ high: price + 0.5, low: price - 0.5 });
    }
    return out;
}

export function medianPrices(bars) {
    if (!Array.isArray(bars)) return [];
    return bars.map(b => (b.high + b.low) / 2);
}

export function fmtN(v, d = 3) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtPct(v) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(1) + '%';
}
