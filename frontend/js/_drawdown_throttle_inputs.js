// Drawdown Throttle helpers shared by view + vitest.
//
// Backend body shape: { equity_history: f64[],
//   config: { tiers: [{min_dd, multiplier}, ...] } }
// Returns ThrottleReport with current/peak/drawdown_pct/active_multiplier.

import { parseFloatBlob } from './_paste_parser.js';
import { t } from './i18n.js';

// Equity values are positive — reuse the shared parser with nonNegative
// gate. Returns `{value, errors}`.
export function parseEquity(text) {
    return parseFloatBlob(text, { nonNegative: true });
}

export const DEFAULT_TIERS = [
    { min_dd: 0.00, multiplier: 1.00 },
    { min_dd: 0.05, multiplier: 0.75 },
    { min_dd: 0.10, multiplier: 0.50 },
    { min_dd: 0.15, multiplier: 0.25 },
    { min_dd: 0.20, multiplier: 0.10 },
];

export function validateInputs(equity, tiers) {
    if (!Array.isArray(equity) || equity.length === 0) return t('view.drawdown_throttle.validate.need_equity');
    if (!equity.every(v => Number.isFinite(v) && v > 0))
        return t('view.drawdown_throttle.validate.equity_positive');
    if (!Array.isArray(tiers) || tiers.length === 0) return t('view.drawdown_throttle.validate.need_tier');
    for (const tier of tiers) {
        if (!Number.isFinite(tier.min_dd) || tier.min_dd < 0 || tier.min_dd > 1)
            return t('view.drawdown_throttle.validate.tier_min_dd');
        if (!Number.isFinite(tier.multiplier) || tier.multiplier < 0 || tier.multiplier > 5)
            return t('view.drawdown_throttle.validate.tier_multiplier');
    }
    // Tiers must be sorted ascending by min_dd (backend assumption).
    for (let i = 1; i < tiers.length; i++) {
        if (tiers[i].min_dd < tiers[i - 1].min_dd) return t('view.drawdown_throttle.validate.tiers_sorted');
    }
    return null;
}

export function buildBody(equity, tiers) {
    return { equity_history: equity, config: { tiers } };
}

// Pure-JS mirror of the backend evaluator. Used as instant pre-flight
// verdict + parity check.
export function localEvaluate(equity, tiers) {
    if (!Array.isArray(equity) || equity.length === 0) {
        return { current_equity: NaN, peak_equity: NaN, drawdown_pct: 0,
                 active_multiplier: 1.0, tier_min_dd: 0 };
    }
    const current = equity[equity.length - 1];
    const peak = Math.max(...equity);
    const dd = peak > 0 ? Math.max(0, (peak - current) / peak) : 0;
    let chosen = tiers[0] || { min_dd: 0, multiplier: 1 };
    for (const t of tiers || []) {
        if (dd >= t.min_dd) chosen = t;
    }
    return {
        current_equity: current,
        peak_equity: peak,
        drawdown_pct: dd,
        active_multiplier: chosen.multiplier,
        tier_min_dd: chosen.min_dd,
    };
}

// Returns the active tier from a tiers array given a current DD %.
// Standalone (used by the chart-overlay legend builder).
export function activeTier(tiers, dd) {
    if (!Array.isArray(tiers) || tiers.length === 0) return null;
    let chosen = tiers[0];
    for (const t of tiers) if (dd >= t.min_dd) chosen = t;
    return chosen;
}

// Per-bar rolling peak + drawdown — drives the equity-curve chart's
// peak line and underwater-DD shaded series.
export function rollingDrawdown(equity) {
    const peaks = [];
    const dds = [];
    if (!Array.isArray(equity)) return { peaks, dds };
    let peak = -Infinity;
    for (const v of equity) {
        if (Number.isFinite(v) && v > peak) peak = v;
        peaks.push(peak === -Infinity ? null : peak);
        dds.push(peak > 0 && Number.isFinite(v)
            ? -((peak - v) / peak)   // negative for underwater chart convention
            : null);
    }
    return { peaks, dds };
}

// Tier color → CSS class for the multiplier badge.
export function multiplierCls(mult) {
    if (!Number.isFinite(mult)) return '';
    if (mult >= 0.95) return 'pos';
    if (mult >= 0.50) return '';
    return 'neg';
}

// Deterministic 50-bar equity curve: starts at $10k, rises to $11k, then
// draws down to a configurable target. 4 demo presets — one per
// post-zero tier (5%, 10%, 15%, 20%+).
export function makeDemoEquity(kind = 'mid') {
    const targetDd = {
        'shallow': 0.03,  // OK tier
        'mild':    0.07,  // 0.75x tier
        'mid':     0.12,  // 0.50x tier
        'deep':    0.17,  // 0.25x tier
        'crisis':  0.25,  // 0.10x tier
    }[kind] || 0.12;
    const peak = 11_000;
    const target = peak * (1 - targetDd);
    const out = [];
    let v = 10_000;
    // Rise 25 bars.
    for (let i = 0; i < 25; i++) {
        v = 10_000 + (peak - 10_000) * (i / 24);
        out.push(Math.round(v));
    }
    // Drawdown 25 bars.
    for (let i = 0; i < 25; i++) {
        v = peak - (peak - target) * ((i + 1) / 25);
        out.push(Math.round(v));
    }
    return out;
}

export function fmtUSD(v) {
    if (!Number.isFinite(v)) return '—';
    const sign = v < 0 ? '-$' : '$';
    return sign + Math.abs(v).toFixed(0);
}

export function fmtPct(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtMult(v) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(2) + 'x';
}
