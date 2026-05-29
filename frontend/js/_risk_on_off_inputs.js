// Risk-On / Risk-Off cross-asset signal helpers.
//
// Backend body: CrossAssetSnapshot flat (NOT nested):
//   { spy_change_pct, gold_change_pct, dxy_change_pct, ten_year_yield_bps_change }
// Returns: { regime: 'on'|'off'|'neutral', score, agreement_count, total_signals }.
//
// Per-signal scoring (mirrors crates/traderview-core/src/risk_on_off.rs):
//   SPY direction: +1 if change > 0.001, -1 if < -0.001, 0 else.
//   Gold (inverse): +1 if < -0.001, -1 if > 0.001.
//   DXY (inverse):  +1 if < -0.001, -1 if > 0.001.
//   Yields (positive correlation): +1 if Δbps > 1, -1 if < -1.
// Regime: score ≥ +2 = on; ≤ -2 = off; else neutral.

import { t } from './i18n.js';

export const SPY_THRESHOLD  = 0.001;
export const GOLD_THRESHOLD = 0.001;
export const DXY_THRESHOLD  = 0.001;
export const YIELD_THRESHOLD_BPS = 1.0;
export const REGIME_THRESHOLD = 2;

export const DEFAULT_INPUTS = {
    spy_change_pct: 0.01,
    gold_change_pct: -0.005,
    dxy_change_pct: -0.003,
    ten_year_yield_bps_change: 5,
};

export function validateInputs(snap) {
    for (const k of ['spy_change_pct','gold_change_pct','dxy_change_pct','ten_year_yield_bps_change']) {
        if (!Number.isFinite(snap[k])) return t('view.risk_on_off.validate.field_finite', { k });
    }
    return null;
}

export function buildBody(snap) {
    return {
        spy_change_pct:  snap.spy_change_pct,
        gold_change_pct: snap.gold_change_pct,
        dxy_change_pct:  snap.dxy_change_pct,
        ten_year_yield_bps_change: snap.ten_year_yield_bps_change,
    };
}

// Pure-JS mirror of risk_on_off::evaluate.
export function localEvaluate(snap) {
    let score = 0, agreement = 0;
    const total = 4;
    // SPY direction.
    if (snap.spy_change_pct >  SPY_THRESHOLD) { score += 1; agreement += 1; }
    else if (snap.spy_change_pct < -SPY_THRESHOLD) { score -= 1; agreement += 1; }
    // Gold (inverse of risk-on).
    if (snap.gold_change_pct < -GOLD_THRESHOLD) { score += 1; agreement += 1; }
    else if (snap.gold_change_pct > GOLD_THRESHOLD) { score -= 1; agreement += 1; }
    // Dollar (inverse of risk-on).
    if (snap.dxy_change_pct < -DXY_THRESHOLD) { score += 1; agreement += 1; }
    else if (snap.dxy_change_pct > DXY_THRESHOLD) { score -= 1; agreement += 1; }
    // Yields (positive correlation with risk-on).
    if (snap.ten_year_yield_bps_change > YIELD_THRESHOLD_BPS)  { score += 1; agreement += 1; }
    else if (snap.ten_year_yield_bps_change < -YIELD_THRESHOLD_BPS) { score -= 1; agreement += 1; }
    let regime;
    if (score >= REGIME_THRESHOLD) regime = 'on';
    else if (score <= -REGIME_THRESHOLD) regime = 'off';
    else regime = 'neutral';
    return { regime, score, agreement_count: agreement, total_signals: total };
}

// Return per-signal direction (+1/-1/0) with the dimension name + raw
// value so the view can render a breakdown table.
export function signalBreakdown(snap) {
    return [
        sig('spy',    snap.spy_change_pct,  SPY_THRESHOLD,  +1),
        sig('gold',   snap.gold_change_pct, GOLD_THRESHOLD, -1),
        sig('dxy',    snap.dxy_change_pct,  DXY_THRESHOLD,  -1),
        sig('yields', snap.ten_year_yield_bps_change, YIELD_THRESHOLD_BPS, +1),
    ];
}

function sig(name, value, threshold, riskOnSign) {
    let direction = 0;
    if (value > threshold)      direction = +1;
    else if (value < -threshold) direction = -1;
    const contribution = direction * riskOnSign;
    return { name, value, threshold, direction, contribution };
}

const REGIME_BADGES = {
    on:      { key: 'view.risk_on_off.badge.on',      cls: 'pos' },
    off:     { key: 'view.risk_on_off.badge.off',     cls: 'neg' },
    neutral: { key: 'view.risk_on_off.badge.neutral', cls: '' },
};

export function regimeBadge(r) {
    return REGIME_BADGES[r] || { key: 'view.risk_on_off.badge.unknown', cls: '' };
}

// Demo presets driving each regime + the noise-floor edge cases the
// Rust tests cover.
export function makeDemoSnap(kind = 'full-on') {
    switch (kind) {
        case 'full-on':
            // SPY +1%, gold -0.5%, dxy -0.3%, yields +5bps → score +4.
            return { spy_change_pct: 0.01, gold_change_pct: -0.005, dxy_change_pct: -0.003, ten_year_yield_bps_change: 5 };
        case 'full-off':
            return { spy_change_pct: -0.02, gold_change_pct: 0.01, dxy_change_pct: 0.005, ten_year_yield_bps_change: -8 };
        case 'majority-off':
            return { spy_change_pct: -0.01, gold_change_pct: 0.005, dxy_change_pct: 0.003, ten_year_yield_bps_change: 0 };
        case 'mixed-neutral':
            // SPY +1, gold -1, dxy +1, yields -1 → score 0.
            return { spy_change_pct: 0.01, gold_change_pct: 0.005, dxy_change_pct: -0.001, ten_year_yield_bps_change: -2 };
        case 'flat':
            return { spy_change_pct: 0, gold_change_pct: 0, dxy_change_pct: 0, ten_year_yield_bps_change: 0 };
        case 'minority-on':
            // Score +1 → still neutral.
            return { spy_change_pct: 0.01, gold_change_pct: 0, dxy_change_pct: 0, ten_year_yield_bps_change: 0 };
        case 'noisy-spy':
            // SPY below noise floor but the other three are clear ON.
            return { spy_change_pct: 0.0001, gold_change_pct: -0.01, dxy_change_pct: -0.005, ten_year_yield_bps_change: 5 };
        case 'bond-rally':
            // Yields drop sharply, equities flat — bond bid → off.
            return { spy_change_pct: 0, gold_change_pct: 0.003, dxy_change_pct: 0.002, ten_year_yield_bps_change: -10 };
        default:
            return makeDemoSnap('full-on');
    }
}

export function fmtPctSigned(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + (v * 100).toFixed(d) + '%';
}

export function fmtBpsSigned(v, d = 1) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + v.toFixed(d) + ' bps';
}

export function fmtScore(v) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + v.toString();
}

export function directionLabelKey(direction) {
    if (direction > 0) return 'view.risk_on_off.dir.up';
    if (direction < 0) return 'view.risk_on_off.dir.down';
    return 'view.risk_on_off.dir.flat';
}

export function contributionClass(contribution) {
    if (contribution > 0) return 'pos';
    if (contribution < 0) return 'neg';
    return '';
}
