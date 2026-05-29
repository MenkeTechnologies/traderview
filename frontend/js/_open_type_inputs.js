// Open Type (Dalton) classifier helpers shared by view + vitest.
//
// Backend body shape: OpenInput {open_price, opening_range_high/low/close,
// prior_day_high/low/vah/val} — flat, no wrapper.

import { t } from './i18n.js';

export function validateInputs(p) {
    for (const k of ['open_price', 'opening_range_high', 'opening_range_low', 'opening_range_close',
                     'prior_day_high', 'prior_day_low', 'prior_day_vah', 'prior_day_val']) {
        if (!Number.isFinite(p[k]) || p[k] <= 0) return `${k} must be > 0`;
    }
    if (p.opening_range_high < p.opening_range_low) return 'opening_range_high must be ≥ opening_range_low';
    if (p.opening_range_close < p.opening_range_low || p.opening_range_close > p.opening_range_high)
        return 'opening_range_close must be in [low, high]';
    if (p.prior_day_high < p.prior_day_low) return 'prior_day_high must be ≥ prior_day_low';
    if (p.prior_day_vah < p.prior_day_val) return 'prior_day_vah must be ≥ prior_day_val';
    if (p.prior_day_val < p.prior_day_low || p.prior_day_vah > p.prior_day_high)
        return 'value area must lie within prior-day range';
    return null;
}

export function buildBody(p) {
    return {
        open_price:           p.open_price,
        opening_range_high:   p.opening_range_high,
        opening_range_low:    p.opening_range_low,
        opening_range_close:  p.opening_range_close,
        prior_day_high:       p.prior_day_high,
        prior_day_low:        p.prior_day_low,
        prior_day_vah:        p.prior_day_vah,
        prior_day_val:        p.prior_day_val,
    };
}

const TYPE_BADGES = {
    open_drive:             { key: 'open_drive',             cls: 'pos' },
    open_test_drive:        { key: 'open_test_drive',        cls: 'pos' },
    open_rejection_reverse: { key: 'open_rejection_reverse', cls: 'neg' },
    open_auction:           { key: 'open_auction',           cls: '' },
};

export function typeBadge(tag) {
    const x = TYPE_BADGES[tag];
    if (!x) return { label: String(tag || '—'), cls: '', hint: '' };
    return {
        label: t(`view.open_type.type.${x.key}.label`),
        cls: x.cls,
        hint: t(`view.open_type.type.${x.key}.hint`),
    };
}

// Four preset OpenInputs matching the four enum variants. Each is
// hand-tuned to make the classifier return the matching enum without
// ambiguity at default tolerances.
export function makeDemoData(kind) {
    const baseline = {
        prior_day_high: 102, prior_day_low: 98,
        prior_day_vah:  101, prior_day_val: 99,
    };
    switch (kind) {
        case 'drive-up':
            return { ...baseline, open_price: 103, opening_range_high: 105, opening_range_low: 102.5, opening_range_close: 105 };
        case 'drive-down':
            return { ...baseline, open_price:  97, opening_range_high:  98, opening_range_low:  95,   opening_range_close:  95 };
        case 'test-drive-up':
            return { ...baseline, open_price: 101, opening_range_high: 103, opening_range_low: 100.5, opening_range_close: 103 };
        case 'rejection-up':
            return { ...baseline, open_price: 101, opening_range_high: 103, opening_range_low: 100,   opening_range_close: 100 };
        case 'auction':
        default:
            return { ...baseline, open_price: 100, opening_range_high: 101, opening_range_low:  99,   opening_range_close: 100.5 };
    }
}

// Derives the {min, max} y-axis span for the schematic chart so all
// reference levels (prior HL + VAH/VAL + OR HL + open) fit comfortably.
export function chartSpan(p) {
    const vals = [
        p.opening_range_high, p.opening_range_low, p.opening_range_close, p.open_price,
        p.prior_day_high, p.prior_day_low, p.prior_day_vah, p.prior_day_val,
    ].filter(Number.isFinite);
    if (vals.length === 0) return { min: 0, max: 1 };
    const min = Math.min(...vals);
    const max = Math.max(...vals);
    const pad = (max - min) * 0.1 || 1;
    return { min: min - pad, max: max + pad };
}

export function fmtN(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function yesNo(b) {
    return b ? 'YES' : 'NO';
}
