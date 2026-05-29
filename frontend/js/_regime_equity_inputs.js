// Equity-curve Regime helpers shared by view + vitest.
//
// Backend body shape: { equity: f64[], config: { trend_slope_pct,
//   clean_trend_rel_stdev } }. Backend serializes regime as snake_case
// enum: 'trending_up' | 'trending_down' | 'volatile_up' |
// 'volatile_down' | 'choppy'.

import { t } from './i18n.js';

export const DEFAULT_CONFIG = {
    trend_slope_pct: 0.001,
    clean_trend_rel_stdev: 0.02,
};

// Parse equity values from CSV, whitespace, newline, or any combination.
// Skip empty lines + non-numeric tokens are errors.
export function parseEquityBlob(s) {
    const errors = [];
    if (typeof s !== 'string') {
        return { equity: [], errors: [{ line: 0, message: 'expected string input' }] };
    }
    const tokens = s.split(/[\s,]+/).map(t => t.trim()).filter(Boolean);
    const equity = [];
    tokens.forEach((tok, i) => {
        const n = Number(tok);
        if (!Number.isFinite(n)) {
            errors.push({ line: i + 1, message: `"${tok}" is not finite` });
        } else {
            equity.push(n);
        }
    });
    return { equity, errors };
}

export function validateInputs(equity, cfg) {
    if (!Array.isArray(equity)) return 'equity must be an array';
    if (equity.length < 3) return 'need ≥ 3 equity points (linear fit requires at least 3)';
    if (equity.some(v => !Number.isFinite(v))) return 'all equity values must be finite numbers';
    if (!Number.isFinite(cfg.trend_slope_pct) || cfg.trend_slope_pct < 0)
        return 'trend_slope_pct must be ≥ 0';
    if (!Number.isFinite(cfg.clean_trend_rel_stdev) || cfg.clean_trend_rel_stdev < 0)
        return 'clean_trend_rel_stdev must be ≥ 0';
    return null;
}

export function buildBody(equity, cfg) {
    return { equity: equity.slice(), config: { ...cfg } };
}

// JS mirror of crates/traderview-core/src/equity_regime.rs::analyze.
export function localEvaluate(equity, cfg) {
    const n = equity.length;
    const empty = {
        n, slope_per_period: 0, residual_stdev: 0, r_squared: 0, regime: 'choppy',
        intercept: 0, mean_equity: 0,
    };
    if (n < 3) return empty;
    const meanT = (n - 1) / 2;
    const meanE = equity.reduce((a, b) => a + b, 0) / n;
    let num = 0, den = 0;
    for (let i = 0; i < n; i++) {
        const dt = i - meanT;
        num += dt * (equity[i] - meanE);
        den += dt * dt;
    }
    if (den === 0) return { ...empty, mean_equity: meanE };
    const slope = num / den;
    const intercept = meanE - slope * meanT;
    let ssRes = 0, ssTot = 0;
    for (let i = 0; i < n; i++) {
        const fit = intercept + slope * i;
        ssRes += (equity[i] - fit) ** 2;
        ssTot += (equity[i] - meanE) ** 2;
    }
    const rSquared = ssTot > 0 ? 1 - ssRes / ssTot : 0;
    const residualStdev = Math.sqrt(ssRes / n);
    const relSlope = meanE > 0 ? slope / meanE : 0;
    const relStdev = meanE > 0 ? residualStdev / meanE : 0;
    let regime;
    if (Math.abs(relSlope) < cfg.trend_slope_pct) {
        regime = 'choppy';
    } else if (relSlope > 0) {
        regime = relStdev <= cfg.clean_trend_rel_stdev ? 'trending_up' : 'volatile_up';
    } else {
        regime = relStdev <= cfg.clean_trend_rel_stdev ? 'trending_down' : 'volatile_down';
    }
    return {
        n, slope_per_period: slope, residual_stdev: residualStdev,
        r_squared: rSquared, regime, intercept, mean_equity: meanE,
    };
}

const REGIME_BADGES = {
    trending_up:   { key: 'trending_up',   cls: 'pos' },
    trending_down: { key: 'trending_down', cls: 'neg' },
    volatile_up:   { key: 'volatile_up',   cls: 'pos' },
    volatile_down: { key: 'volatile_down', cls: 'neg' },
    choppy:        { key: 'choppy',        cls: '' },
};

export function regimeBadge(r) {
    const x = REGIME_BADGES[r];
    if (!x) return { label: String(r || '—').toUpperCase(), cls: '', hint: '—' };
    return {
        label: t(`view.regime_equity.regime.${x.key}.label`),
        cls: x.cls,
        hint: t(`view.regime_equity.regime.${x.key}.hint`),
    };
}

// Build the fitted regression line as parallel x[] and y[] arrays
// (uPlot-friendly two-row layout: equity[0..n-1] = x indices,
// equity[1] = actual, equity[2] = fit).
export function fitLine(equity, local) {
    if (!Array.isArray(equity) || equity.length === 0) return [];
    const out = new Array(equity.length);
    for (let i = 0; i < equity.length; i++) {
        out[i] = local.intercept + local.slope_per_period * i;
    }
    return out;
}

// 5 demo presets — one per regime + a flat one with custom config.
// Each runs the same `analyze` and SHOULD classify into the named regime.
export function makeDemoEquity(kind = 'trending-up') {
    const len = 30;
    switch (kind) {
        case 'trending-up':
            return Array.from({ length: len }, (_, i) => 10_000 + i * 100);
        case 'trending-down':
            return Array.from({ length: len }, (_, i) => 20_000 - i * 120);
        case 'volatile-up':
            return Array.from({ length: len }, (_, i) => {
                const trend = 10_000 + i * 100;
                const noise = ((i * 73) % 31) * 100 - 1500;
                return trend + noise;
            });
        case 'volatile-down':
            return Array.from({ length: len }, (_, i) => {
                const trend = 20_000 - i * 120;
                const noise = ((i * 73) % 31) * 100 - 1500;
                return trend + noise;
            });
        case 'choppy':
            // tiny micro-noise around a flat line — rel_slope below threshold
            return Array.from({ length: len }, (_, i) => 10_000 + ((i * 7) % 5));
        case 'realistic':
            // 90-day random-walk equity curve with a slight positive drift
            return Array.from({ length: 90 }, (_, i) => {
                const drift = 50_000 + i * 80;
                const noise = Math.sin(i * 0.7) * 400 + Math.cos(i * 1.3) * 200;
                return drift + noise;
            });
        default:
            return makeDemoEquity('trending-up');
    }
}

export function fmtUSD(v, d = 0) {
    if (!Number.isFinite(v)) return '—';
    const sign = v < 0 ? '-' : '';
    return sign + '$' + Math.abs(v).toFixed(d);
}

export function fmtUSDSigned(v, d = 0) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '-') + '$' + Math.abs(v).toFixed(d);
}

export function fmtPct(v, d = 3) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtNum(v, d = 4) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}
