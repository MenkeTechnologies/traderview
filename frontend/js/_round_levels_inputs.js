// Round-number support/resistance helpers.
//
// Backend body: { current_price, atr: number | null,
//   config: { window, min_weight } }.
// min_weight enum (snake_case): 'major' | 'medium' | 'minor'.
// Returns: { levels: [{price, weight, distance, distance_atrs}],
//   nearest_above, nearest_below }.
//
// Classification (largest divisor wins):
//   Major  → ÷1000, ÷500, ÷100
//   Medium → ÷50, ÷25
//   Minor  → ÷10, ÷5, ÷1   (any integer)
// Non-integer prices return None.

import { t } from './i18n.js';

export const WEIGHTS = ['major', 'medium', 'minor'];
const WEIGHT_RANK = { major: 3, medium: 2, minor: 1 };

export const DEFAULT_CONFIG = { window: 50, min_weight: 'minor' };
export const DEFAULT_INPUTS = {
    current_price: 102.5,
    atr: 2.0,
    config: { ...DEFAULT_CONFIG },
};

// Window-size cap mirrors Rust's 100_000 guard.
export const MAX_INTEGER_SCAN = 100_000;

export function validateInputs(input) {
    if (!Number.isFinite(input.current_price)) return t('view.round_levels.validate.price_finite');
    if (input.current_price <= 0)              return t('view.round_levels.validate.price_positive');
    if (input.atr != null && (!Number.isFinite(input.atr) || input.atr < 0))
        return t('view.round_levels.validate.atr');
    if (!input.config)                          return t('view.round_levels.validate.config_required');
    if (!Number.isFinite(input.config.window))  return t('view.round_levels.validate.window_finite');
    if (input.config.window <= 0)               return t('view.round_levels.validate.window_positive');
    if (!WEIGHTS.includes(input.config.min_weight))
        return t('view.round_levels.validate.min_weight', { list: WEIGHTS.join(', ') });
    return null;
}

export function buildBody(input) {
    return {
        current_price: input.current_price,
        atr: input.atr,
        config: { window: input.config.window, min_weight: input.config.min_weight },
    };
}

export function classify(price) {
    if (!Number.isFinite(price)) return null;
    if (Math.abs(price - Math.round(price)) > 1e-9) return null;
    const p = Math.round(price);
    // Largest divisor wins — $1000 is Major even though it's also a Minor.
    if (p % 1000 === 0 || p % 500 === 0 || p % 100 === 0) return 'major';
    if (p % 50 === 0   || p % 25 === 0)                    return 'medium';
    if (p % 10 === 0   || p % 5 === 0  || p % 1 === 0)     return 'minor';
    return null;
}

export function weightRank(w) {
    return WEIGHT_RANK[w] || 0;
}

// Pure-JS mirror of crates/traderview-core/src/round_levels.rs::detect.
// Returns same shape; same window guard.
export function localDetect(current_price, atr, cfg) {
    const out = { levels: [], nearest_above: null, nearest_below: null };
    if (!Number.isFinite(current_price) || current_price <= 0 || cfg.window <= 0) return out;
    const lo = Math.max(0, Math.floor(current_price - cfg.window));
    const hi = Math.ceil(current_price + cfg.window);
    if (hi <= lo) return out;
    if (hi - lo > MAX_INTEGER_SCAN) return out;
    const minRank = weightRank(cfg.min_weight);
    const levels = [];
    for (let p = lo; p <= hi; p++) {
        const w = classify(p);
        if (!w) continue;
        if (weightRank(w) < minRank) continue;
        const distance = p - current_price;
        if (Math.abs(distance) > cfg.window) continue;
        const distance_atrs = (atr != null && atr > 0) ? distance / atr : null;
        levels.push({ price: p, weight: w, distance, distance_atrs });
    }
    out.levels = levels;
    let nearestAbove = null, nearestBelow = null;
    for (const l of levels) {
        if (l.distance > 0 && (nearestAbove == null || l.distance < nearestAbove.distance))
            nearestAbove = l;
        if (l.distance < 0 && (nearestBelow == null || l.distance > nearestBelow.distance))
            nearestBelow = l;
    }
    out.nearest_above = nearestAbove ? { ...nearestAbove } : null;
    out.nearest_below = nearestBelow ? { ...nearestBelow } : null;
    return out;
}

const WEIGHT_BADGES = {
    major:  { key: 'view.round_levels.weight.major',  cls: 'neg' },
    medium: { key: 'view.round_levels.weight.medium', cls: '' },
    minor:  { key: 'view.round_levels.weight.minor',  cls: '' },
};

export function weightBadge(w) {
    return WEIGHT_BADGES[w] || { key: 'view.round_levels.weight.unknown', cls: '' };
}

// Summary verdict — how confluent is the current price with strong levels?
export function pinningBadge(nearest_above, nearest_below, current_price) {
    if (!Number.isFinite(current_price)) return { key: 'view.round_levels.badge.unknown', cls: '' };
    const ahove = nearest_above ? Math.abs(nearest_above.distance) : Infinity;
    const below = nearest_below ? Math.abs(nearest_below.distance) : Infinity;
    const nearest = Math.min(ahove, below);
    if (!Number.isFinite(nearest)) return { key: 'view.round_levels.badge.no_levels', cls: '' };
    const pct = nearest / current_price;
    if (pct < 0.001) return { key: 'view.round_levels.badge.pinned',     cls: 'neg' };
    if (pct < 0.005) return { key: 'view.round_levels.badge.adjacent',   cls: 'neg' };
    if (pct < 0.02)  return { key: 'view.round_levels.badge.near',       cls: '' };
    return { key: 'view.round_levels.badge.clear', cls: 'pos' };
}

// Demo presets.
export function makeDemoInput(kind = 'aapl-near-180') {
    switch (kind) {
        case 'aapl-near-180':
            return { current_price: 180.50, atr: 3.5,
                     config: { window: 20, min_weight: 'minor' } };
        case 'spy-near-500':
            return { current_price: 498.75, atr: 5,
                     config: { window: 30, min_weight: 'minor' } };
        case 'tsla-near-250':
            return { current_price: 247.30, atr: 10,
                     config: { window: 50, min_weight: 'medium' } };
        case 'btc-near-100k':
            return { current_price: 99850, atr: 2500,
                     config: { window: 5000, min_weight: 'major' } };
        case 'penny-near-3':
            return { current_price: 3.15, atr: 0.4,
                     config: { window: 2, min_weight: 'minor' } };
        case 'pinned-at-100':
            return { current_price: 100.05, atr: 1.5,
                     config: { window: 10, min_weight: 'minor' } };
        case 'major-only':
            return { current_price: 175, atr: null,
                     config: { window: 100, min_weight: 'major' } };
        case 'no-atr':
            return { current_price: 250, atr: null,
                     config: { window: 50, min_weight: 'medium' } };
        default:
            return makeDemoInput('aapl-near-180');
    }
}

export function fmtUSD(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    const sign = v < 0 ? '-' : '';
    return sign + '$' + Math.abs(v).toFixed(d);
}

export function fmtUSDSigned(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '-') + '$' + Math.abs(v).toFixed(d);
}

export function fmtAtrs(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + v.toFixed(d) + ' ATR';
}

export function fmtPct(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function weightLabelKey(w) {
    return `view.round_levels.weight.${w || 'unknown'}`;
}
