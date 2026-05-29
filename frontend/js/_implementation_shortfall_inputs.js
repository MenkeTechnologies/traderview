// Implementation Shortfall helpers shared by view + vitest.
//
// Backend body shape mirrors `ShortfallInput` in
// `crates/traderview-core/src/implementation_shortfall.rs`.

import { t } from './i18n.js';

export const COMPONENT_KEYS = ['spread_cost', 'timing_cost', 'impact_cost', 'opportunity_cost'];

export const COMPONENT_LABELS = new Proxy({}, {
    get(_t, key) {
        if (typeof key !== 'string') return undefined;
        return t(`view.implementation_shortfall.component.${key}`);
    },
});

export function validateInputs(p) {
    if (p.direction !== 'buy' && p.direction !== 'sell') return t('view.implementation_shortfall.validate.direction');
    if (!Number.isFinite(p.decision_mid) || p.decision_mid <= 0) return t('view.implementation_shortfall.validate.decision_mid');
    if (!Number.isFinite(p.arrival_mid)  || p.arrival_mid <= 0)  return t('view.implementation_shortfall.validate.arrival_mid');
    if (!Number.isFinite(p.vwap_fill)    || p.vwap_fill < 0)     return t('view.implementation_shortfall.validate.vwap_fill');
    if (!Number.isFinite(p.final_mid)    || p.final_mid <= 0)    return t('view.implementation_shortfall.validate.final_mid');
    if (!Number.isFinite(p.half_spread_at_decision) || p.half_spread_at_decision < 0)
        return t('view.implementation_shortfall.validate.half_spread');
    if (!Number.isFinite(p.intended_qty) || p.intended_qty <= 0) return t('view.implementation_shortfall.validate.intended_qty');
    if (!Number.isFinite(p.filled_qty)   || p.filled_qty < 0)    return t('view.implementation_shortfall.validate.filled_qty');
    if (p.filled_qty > p.intended_qty + 1e-9) return t('view.implementation_shortfall.validate.filled_le_intended');
    return null;
}

export function buildBody(p) {
    return {
        direction:               p.direction,
        decision_mid:            p.decision_mid,
        arrival_mid:             p.arrival_mid,
        vwap_fill:               p.vwap_fill,
        final_mid:               p.final_mid,
        half_spread_at_decision: p.half_spread_at_decision,
        intended_qty:            p.intended_qty,
        filled_qty:              p.filled_qty,
    };
}

// Decomposes the four components into a {label, value, share} array for
// the bar chart + cost-attribution list. `share` is signed-component /
// sum-of-abs-components — robust when components have opposing signs.
export function decompose(report) {
    if (!report || typeof report !== 'object') {
        return COMPONENT_KEYS.map(k => ({ key: k, label: COMPONENT_LABELS[k], value: 0, share: 0 }));
    }
    const absSum = COMPONENT_KEYS.reduce((a, k) => a + Math.abs(report[k] || 0), 0);
    return COMPONENT_KEYS.map(k => {
        const v = Number(report[k]) || 0;
        return {
            key: k,
            label: COMPONENT_LABELS[k],
            value: v,
            share: absSum > 0 ? v / absSum : 0,
        };
    });
}

// CSS class for a cost cell — green when the trader CAPTURED liquidity
// (negative dollar cost), red when paid up. Total includes both signs.
export function costSignClass(v) {
    if (!Number.isFinite(v) || v === 0) return '';
    return v > 0 ? 'neg' : 'pos';
}

export function fillKind(intended, filled) {
    if (filled <= 0) return 'unfilled';
    if (filled >= intended - 1e-9) return 'full';
    return 'partial';
}

export function fmtUSD(v) {
    if (!Number.isFinite(v)) return '—';
    const sign = v < 0 ? '-$' : '$';
    return sign + Math.abs(v).toFixed(2);
}

export function fmtBps(v) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(1) + ' bps';
}

export function fmtPct(v) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(1) + '%';
}
