// CUSUM (Page-Hinkley) change-point detector helpers shared by view + vitest.
//
// Backend body shape: { series: f64[], config: {reference_mean,
// reference_stdev, threshold_stdevs, slack} }.

import { parseFloatBlob } from './_paste_parser.js';

// CUSUM operates on returns (signed) — no nonNegative gate.
export function parseSeries(text) {
    return parseFloatBlob(text);
}

export function validateInputs(series, cfg) {
    if (!Array.isArray(series) || series.length < 10) return 'series must have at least 10 observations';
    if (!series.every(Number.isFinite)) return 'series must be finite';
    if (!Number.isFinite(cfg.reference_mean)) return 'reference_mean must be finite';
    if (!Number.isFinite(cfg.reference_stdev) || cfg.reference_stdev <= 0)
        return 'reference_stdev must be > 0';
    if (!Number.isFinite(cfg.threshold_stdevs) || cfg.threshold_stdevs <= 0)
        return 'threshold_stdevs must be > 0';
    if (!Number.isFinite(cfg.slack) || cfg.slack < 0)
        return 'slack must be ≥ 0';
    return null;
}

export function buildBody(series, config) {
    return { series, config };
}

// Welford-stable mean + sample stdev of a finite series. Used by the
// "auto-fit from series" button — saves the user from computing stats by hand.
export function meanStdev(values) {
    const xs = (values || []).filter(Number.isFinite);
    const n = xs.length;
    if (n < 2) return { mean: NaN, stdev: NaN };
    let mean = 0, m2 = 0;
    for (let i = 0; i < n; i++) {
        const x = xs[i];
        const delta = x - mean;
        mean += delta / (i + 1);
        m2 += delta * (x - mean);
    }
    const variance = m2 / (n - 1);
    return { mean, stdev: Math.sqrt(variance) };
}

// Splits events into parallel up/down null-padded series for uPlot
// marker plotting — null elsewhere, value at the event bar_index.
export function eventMarkers(events, length) {
    const up = new Array(length).fill(null);
    const dn = new Array(length).fill(null);
    if (!Array.isArray(events)) return { up, dn };
    for (const e of events) {
        if (!Number.isInteger(e.bar_index) || e.bar_index < 0 || e.bar_index >= length) continue;
        if (e.direction === 'up')   up[e.bar_index] = e.cusum_value;
        if (e.direction === 'down') dn[e.bar_index] = e.cusum_value;
    }
    return { up, dn };
}

// Deterministic 200-bar demo: 100 bars at mean +0.5%, then 100 bars at
// mean −0.7%. The CUSUM detector with default threshold (5 stdevs) should
// fire one or two Down events shortly after bar 100.
export function makeDemoSeries(seed = 1) {
    let s = seed;
    const rand = () => { s = (s * 1664525 + 1013904223) | 0; return ((s >>> 0) / 0xffffffff); };
    const out = new Array(200);
    for (let i = 0; i < 200; i++) {
        const driftMean = i < 100 ? 0.005 : -0.007;
        const noise = (rand() - 0.5) * 0.02;
        out[i] = Number((driftMean + noise).toFixed(6));
    }
    return out;
}

export function fmtN(v, d = 4) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function dirCss(dir) {
    if (dir === 'up')   return 'pos';
    if (dir === 'down') return 'neg';
    return '';
}
