// Sample Autocorrelation Function (ACF) helpers + Bartlett confidence bands.
//
// Backend body: { series: number[], max_lag: number }
// Returns: { lags, autocorrelations, confidence_band, significant_lags, n_observations } | null
//
// ρ̂(k) = Σ_{t=k+1..n} (x_t − x̄)(x_{t−k} − x̄) / Σ_{t=1..n} (x_t − x̄)²
// Bartlett 95% band = 1.96 / √n.
// significant_lags = k > 0 where |ρ̂(k)| > band.

import { t } from './i18n.js';

export const DEFAULT_MAX_LAG = 20;
export const BARTLETT_Z = 1.96;

export const DEFAULT_INPUTS = {
    series: [],
    max_lag: DEFAULT_MAX_LAG,
};

export function validateInputs(input) {
    if (!Array.isArray(input.series))                       return t('view.acf.validate.series_array');
    for (let i = 0; i < input.series.length; i++) {
        if (!Number.isFinite(input.series[i]))              return t('view.acf.validate.series_finite', { i });
    }
    if (!Number.isInteger(input.max_lag))                   return t('view.acf.validate.max_lag_int');
    if (input.max_lag < 1)                                  return t('view.acf.validate.max_lag_min');
    if (input.series.length < 5)                            return t('view.acf.validate.series_min');
    if (input.max_lag >= input.series.length)               return t('view.acf.validate.max_lag_lt_len', { maxLag: input.max_lag, len: input.series.length });
    return null;
}

export function buildBody(input) {
    return {
        series:  input.series,
        max_lag: input.max_lag,
    };
}

// Pure-JS mirror of crates/traderview-core/src/autocorrelation_function.rs::compute.
// Returns null on validation failure / degenerate denom.
export function localCompute(series, max_lag) {
    const n = series.length;
    if (n < 5 || max_lag === 0 || max_lag >= n) return null;
    for (const v of series) if (!Number.isFinite(v)) return null;
    let sum = 0;
    for (const v of series) sum += v;
    const mean = sum / n;
    let denom = 0;
    for (const v of series) { const d = v - mean; denom += d * d; }
    if (denom <= 0) return null;
    const lags = new Array(max_lag + 1);
    const acfs = new Array(max_lag + 1);
    for (let k = 0; k <= max_lag; k++) {
        let num = 0;
        for (let t = k; t < n; t++) num += (series[t] - mean) * (series[t - k] - mean);
        acfs[k] = num / denom;
        lags[k] = k;
    }
    const band = BARTLETT_Z / Math.sqrt(n);
    const significant_lags = [];
    for (let k = 1; k <= max_lag; k++) {
        if (Math.abs(acfs[k]) > band) significant_lags.push(k);
    }
    return {
        lags,
        autocorrelations: acfs,
        confidence_band: band,
        significant_lags,
        n_observations: n,
    };
}

// Parse comma/whitespace-separated series; comments + blanks ignored.
export function parseSeriesBlob(blob) {
    const out = { series: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: t('common.parse.input_must_be_string') });
        return out;
    }
    const tokens = blob
        .split('\n')
        .map(l => l.split('#')[0])
        .join(' ')
        .split(/[\s,]+/)
        .filter(t => t.length > 0);
    for (let i = 0; i < tokens.length; i++) {
        const v = Number(tokens[i]);
        if (!Number.isFinite(v)) {
            out.errors.push({ line_no: i + 1, message: `token "${tokens[i]}" not finite` });
            continue;
        }
        out.series.push(v);
    }
    return out;
}

export function seriesToBlob(series) {
    return series.join('\n');
}

// Verdict on lag-1 autocorrelation.
export function autocorrelationBadge(rho1, band) {
    if (rho1 == null || !Number.isFinite(rho1)) return { key: 'view.acf.badge.unknown', cls: '' };
    if (Number.isFinite(band) && Math.abs(rho1) <= band) return { key: 'view.acf.badge.white_noise', cls: 'pos' };
    if (rho1 > 0.8)  return { key: 'view.acf.badge.random_walk',     cls: 'neg' };
    if (rho1 > 0.3)  return { key: 'view.acf.badge.persistent',      cls: '' };
    if (rho1 > 0)    return { key: 'view.acf.badge.mild_persistence', cls: '' };
    if (rho1 > -0.3) return { key: 'view.acf.badge.mild_reversion',  cls: '' };
    return { key: 'view.acf.badge.mean_reverting', cls: 'pos' };
}

// AR(1) phi estimate (just lag-1 ACF — quick eyeball).
export function ar1PhiEstimate(report) {
    if (!report || !Array.isArray(report.autocorrelations)) return NaN;
    return report.autocorrelations[1];
}

// Aggregate stats.
export function summarize(report) {
    if (!report || !Array.isArray(report.autocorrelations) || report.autocorrelations.length === 0) {
        return { lag_count: 0, sig_count: 0, max_abs_acf: NaN, max_abs_lag: -1,
                 rho1: NaN, rho5: NaN, rho10: NaN };
    }
    let mx = 0, mxLag = -1;
    for (let k = 1; k < report.autocorrelations.length; k++) {
        const a = Math.abs(report.autocorrelations[k]);
        if (a > mx) { mx = a; mxLag = k; }
    }
    return {
        lag_count:   report.autocorrelations.length - 1,
        sig_count:   report.significant_lags.length,
        max_abs_acf: mx,
        max_abs_lag: mxLag,
        rho1:        report.autocorrelations[1] ?? NaN,
        rho5:        report.autocorrelations[5] ?? NaN,
        rho10:       report.autocorrelations[10] ?? NaN,
    };
}

function lcg(seed) {
    let state = BigInt(7919) + seed;
    return () => {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(state >> 32n) / 0xFFFFFFFF - 0.5;
    };
}

export function makeDemoInput(kind = 'ar1-0.8') {
    switch (kind) {
        case 'white-noise': {
            const rand = lcg(11n);
            const s = new Array(500);
            for (let i = 0; i < s.length; i++) s[i] = rand() * 2;
            return { series: s, max_lag: 20 };
        }
        case 'random-walk': {
            const rand = lcg(42n);
            const s = new Array(200).fill(0);
            for (let i = 1; i < s.length; i++) s[i] = s[i - 1] + rand() * 2;
            return { series: s, max_lag: 20 };
        }
        case 'ar1-0.8': {
            // Strongly persistent AR(1).
            const rand = lcg(7n);
            const s = new Array(1000).fill(0);
            for (let i = 1; i < s.length; i++) s[i] = 0.8 * s[i - 1] + rand() * 0.5;
            return { series: s, max_lag: 10 };
        }
        case 'ar1-neg0.6': {
            // Negative AR(1) → alternating sign ACF.
            const rand = lcg(31n);
            const s = new Array(500).fill(0);
            for (let i = 1; i < s.length; i++) s[i] = -0.6 * s[i - 1] + rand() * 0.5;
            return { series: s, max_lag: 15 };
        }
        case 'sinusoid': {
            // Pure sine — ACF should be sinusoidal too.
            const s = new Array(400);
            for (let i = 0; i < s.length; i++) s[i] = Math.sin(i * 0.3);
            return { series: s, max_lag: 30 };
        }
        case 'trending': {
            // Strong trend → very persistent ACF.
            const s = new Array(200);
            for (let i = 0; i < s.length; i++) s[i] = 100 + i * 0.5 + Math.sin(i * 0.4);
            return { series: s, max_lag: 20 };
        }
        case 'wide-lags': {
            // 50 lags on a 500-bar series.
            const rand = lcg(99n);
            const s = new Array(500).fill(0);
            for (let i = 1; i < s.length; i++) s[i] = 0.5 * s[i - 1] + rand() * 0.5;
            return { series: s, max_lag: 50 };
        }
        case 'short-series': {
            // Just over the minimum length.
            const rand = lcg(3n);
            const s = new Array(20);
            for (let i = 0; i < s.length; i++) s[i] = rand();
            return { series: s, max_lag: 5 };
        }
        default: return makeDemoInput('ar1-0.8');
    }
}

export function fmtAcf(v, d = 4) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + v.toFixed(d);
}

export function fmtBand(v, d = 4) {
    if (v == null || !Number.isFinite(v)) return '—';
    return '±' + v.toFixed(d);
}

export function fmtInt(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}

export function fmtNum(v, d = 4) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}
