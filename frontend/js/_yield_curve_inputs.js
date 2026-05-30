// Yield-curve shape classifier helpers.
//
// Backend body: YieldCurve flat: { y3m, y2y, y5y, y10y, y30y }.
// Returns: CurveReport { shape, spread_10y_2y_bps, spread_10y_3m_bps, note }.
//
// Classification priority (mirror of yield_curve::classify):
//   1. y10y < y2y → Inverted (canonical recession)
//   2. Humped: y5y > y3m AND y5y > y30y AND y2y < y5y AND y10y < y5y
//   3. All four consecutive |spreads| < 0.0025 → Flat
//   4. All consecutive spreads ≥ -0.0001 → Normal (allows tiny noise)
//   5. else → Flat (mixed / non-monotonic)

import { t } from './i18n.js';

export const TENORS = ['y3m', 'y2y', 'y5y', 'y10y', 'y30y'];

// Approximate years (for chart x-axis). 3M = 0.25, etc.
export const TENOR_YEARS = { y3m: 0.25, y2y: 2, y5y: 5, y10y: 10, y30y: 30 };

export const TENOR_LABELS = { y3m: '3M', y2y: '2Y', y5y: '5Y', y10y: '10Y', y30y: '30Y' };

export const SHAPES = ['normal', 'flat', 'inverted', 'humped'];

export const DEFAULT_INPUTS = {
    y3m: 0.030, y2y: 0.035, y5y: 0.040, y10y: 0.045, y30y: 0.050,
};

export function validateInputs(curve) {
    for (const k of TENORS) {
        if (!Number.isFinite(curve[k])) return t('common.validate.field_must_be_finite', { field: k });
    }
    return null;
}

export function buildBody(curve) {
    return { y3m: curve.y3m, y2y: curve.y2y, y5y: curve.y5y, y10y: curve.y10y, y30y: curve.y30y };
}

// Mirror of crates/traderview-core/src/yield_curve.rs::classify.
// note_key returned (i18n) instead of formatted English string. The
// inverted note interpolates the magnitude — caller formats with t().
export function localClassify(c) {
    const spread_10_2  = (c.y10y - c.y2y) * 10_000;
    const spread_10_3m = (c.y10y - c.y3m) * 10_000;
    const isHumped = c.y5y > c.y3m && c.y5y > c.y30y && c.y2y < c.y5y && c.y10y < c.y5y;
    const spreads = [c.y2y - c.y3m, c.y5y - c.y2y, c.y10y - c.y5y, c.y30y - c.y10y];
    let shape;
    if (spread_10_2 < 0) {
        shape = 'inverted';
    } else if (isHumped) {
        shape = 'humped';
    } else if (spreads.every(s => Math.abs(s) < 0.0025)) {
        shape = 'flat';
    } else if (spreads.every(s => s >= -0.0001)) {
        shape = 'normal';
    } else {
        shape = 'flat';
    }
    let note_key = `view.yield_curve.note.${shape}`;
    let note_params;
    if (shape === 'inverted') {
        note_params = { bps: Math.round(-spread_10_2) };
    }
    return {
        shape,
        spread_10y_2y_bps: spread_10_2,
        spread_10y_3m_bps: spread_10_3m,
        note_key, note_params,
    };
}

const SHAPE_BADGES = {
    normal:   { key: 'view.yield_curve.badge.normal',   cls: 'pos' },
    flat:     { key: 'view.yield_curve.badge.flat',     cls: '' },
    inverted: { key: 'view.yield_curve.badge.inverted', cls: 'neg' },
    humped:   { key: 'view.yield_curve.badge.humped',   cls: '' },
};

export function shapeBadge(shape) {
    return SHAPE_BADGES[shape] || { key: 'view.yield_curve.badge.unknown', cls: '' };
}

// Per-tenor slope between adjacent tenors — drives the breakdown table.
export function consecutiveSpreads(curve) {
    return [
        { from: 'y3m',  to: 'y2y',  delta: curve.y2y  - curve.y3m },
        { from: 'y2y',  to: 'y5y',  delta: curve.y5y  - curve.y2y },
        { from: 'y5y',  to: 'y10y', delta: curve.y10y - curve.y5y },
        { from: 'y10y', to: 'y30y', delta: curve.y30y - curve.y10y },
    ];
}

// Demo presets matching Rust tests + historical analogues.
export function makeDemoCurve(kind = 'normal') {
    switch (kind) {
        case 'normal':
            return { y3m: 0.030, y2y: 0.035, y5y: 0.040, y10y: 0.045, y30y: 0.050 };
        case 'inverted':
            return { y3m: 0.040, y2y: 0.055, y5y: 0.050, y10y: 0.045, y30y: 0.045 };
        case 'flat':
            return { y3m: 0.040, y2y: 0.041, y5y: 0.042, y10y: 0.0425, y30y: 0.043 };
        case 'humped':
            return { y3m: 0.030, y2y: 0.040, y5y: 0.060, y10y: 0.045, y30y: 0.030 };
        case 'noisy-normal':
            // Tiny dip 10y→30y still classifies normal (tolerance).
            return { y3m: 0.030, y2y: 0.035, y5y: 0.040, y10y: 0.045, y30y: 0.0449 };
        case 'ust-2024-inverted':
            // Approximate UST end-2024: short rates high vs long flat ~4.3%.
            return { y3m: 0.0530, y2y: 0.0490, y5y: 0.0430, y10y: 0.0435, y30y: 0.0455 };
        case 'ust-2020-zirp':
            // COVID ZIRP curve.
            return { y3m: 0.0010, y2y: 0.0015, y5y: 0.0050, y10y: 0.0100, y30y: 0.0170 };
        case 'gfc-2008-flat':
            // Late-2008 flat near zero.
            return { y3m: 0.0010, y2y: 0.0030, y5y: 0.0050, y10y: 0.0250, y30y: 0.0310 };
        default:
            return makeDemoCurve('normal');
    }
}

export function fmtPct(v, d = 3) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtBpsSigned(v, d = 0) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + v.toFixed(d) + ' bps';
}

export function fmtSpreadPct(v, d = 4) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + (v * 100).toFixed(d) + '%';
}
