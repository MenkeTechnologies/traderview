// Open Type helpers: validator, body shape, type badge map, demo
// presets, chart-span helper, formatters.

import { test, expect } from 'vitest';
import {
    validateInputs, buildBody, typeBadge,
    makeDemoData, chartSpan, fmtN, yesNo,
} from '../js/_open_type_inputs.js';

// ── validateInputs ────────────────────────────────────────────────

const ok = {
    open_price: 100,
    opening_range_high: 101, opening_range_low: 99, opening_range_close: 100.5,
    prior_day_high: 102, prior_day_low: 98,
    prior_day_vah: 101, prior_day_val: 99,
};

test('validate accepts canonical baseline', () => {
    expect(validateInputs(ok)).toBe(null);
});

test('validate rejects each field at ≤ 0 or non-finite', () => {
    for (const k of Object.keys(ok)) {
        expect(validateInputs({ ...ok, [k]: 0 })).toMatch(new RegExp(k));
        expect(validateInputs({ ...ok, [k]: NaN })).toMatch(new RegExp(k));
    }
});

test('validate rejects OR-high < OR-low', () => {
    expect(validateInputs({ ...ok, opening_range_high: 98, opening_range_low: 99 }))
        .toMatch(/opening_range_high/);
});

test('validate rejects OR-close outside [OR-low, OR-high]', () => {
    expect(validateInputs({ ...ok, opening_range_close: 90 })).toMatch(/opening_range_close/);
    expect(validateInputs({ ...ok, opening_range_close: 110 })).toMatch(/opening_range_close/);
});

test('validate rejects prior-high < prior-low', () => {
    expect(validateInputs({ ...ok, prior_day_high: 90, prior_day_low: 100 }))
        .toMatch(/prior_day_high/);
});

test('validate rejects vah < val', () => {
    expect(validateInputs({ ...ok, prior_day_vah: 95, prior_day_val: 100 }))
        .toMatch(/prior_day_vah/);
});

test('validate rejects value area outside prior range', () => {
    expect(validateInputs({ ...ok, prior_day_val: 90 })).toMatch(/value area/);
    expect(validateInputs({ ...ok, prior_day_vah: 110 })).toMatch(/value area/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody passes all 8 fields through', () => {
    expect(buildBody(ok)).toEqual(ok);
});

// ── typeBadge ─────────────────────────────────────────────────────

test('typeBadge maps all 4 enum variants with color + hint', () => {
    expect(typeBadge('open_drive').cls).toBe('pos');
    expect(typeBadge('open_drive').label).toBe('OPEN DRIVE');
    expect(typeBadge('open_drive').hint).toMatch(/strong trend/);
    expect(typeBadge('open_test_drive').cls).toBe('pos');
    expect(typeBadge('open_rejection_reverse').cls).toBe('neg');
    expect(typeBadge('open_rejection_reverse').hint).toMatch(/fade/);
    expect(typeBadge('open_auction').cls).toBe('');
});

test('typeBadge handles unknown enum + null gracefully', () => {
    expect(typeBadge('garbage').label).toBe('garbage');
    expect(typeBadge(null).label).toBe('—');
});

// ── makeDemoData ──────────────────────────────────────────────────

test('makeDemoData(auction) returns inputs that the validator accepts', () => {
    const d = makeDemoData('auction');
    expect(validateInputs(d)).toBe(null);
});

test('all 5 demo kinds produce valid inputs', () => {
    for (const k of ['drive-up', 'drive-down', 'test-drive-up', 'rejection-up', 'auction']) {
        expect(validateInputs(makeDemoData(k))).toBe(null);
    }
});

test('makeDemoData unknown kind falls back to auction', () => {
    expect(makeDemoData('weird')).toEqual(makeDemoData('auction'));
});

test('drive-up demo opens above prior_day_high', () => {
    const d = makeDemoData('drive-up');
    expect(d.open_price).toBeGreaterThanOrEqual(d.prior_day_high);
});

test('drive-down demo opens below prior_day_low', () => {
    const d = makeDemoData('drive-down');
    expect(d.open_price).toBeLessThanOrEqual(d.prior_day_low);
});

// ── chartSpan ─────────────────────────────────────────────────────

test('chartSpan pads [min, max] by 10% of range', () => {
    const span = chartSpan({
        opening_range_high: 105, opening_range_low: 95, opening_range_close: 100, open_price: 100,
        prior_day_high: 102, prior_day_low: 98, prior_day_vah: 101, prior_day_val: 99,
    });
    expect(span.min).toBeCloseTo(95 - 1, 6);     // (105-95)*0.1 = 1
    expect(span.max).toBeCloseTo(105 + 1, 6);
});

test('chartSpan returns sensible defaults when input degenerate', () => {
    const span = chartSpan({});
    expect(span.min).toBe(0);
    expect(span.max).toBe(1);
});

test('chartSpan uses pad=1 when min == max', () => {
    const span = chartSpan({
        open_price: 100, opening_range_high: 100, opening_range_low: 100, opening_range_close: 100,
        prior_day_high: 100, prior_day_low: 100, prior_day_vah: 100, prior_day_val: 100,
    });
    expect(span.min).toBe(99);
    expect(span.max).toBe(101);
});

// ── formatters ────────────────────────────────────────────────────

test('fmtN + yesNo', () => {
    expect(fmtN(1.234)).toBe('1.23');
    expect(fmtN(NaN)).toBe('—');
    expect(yesNo(true)).toBe('YES');
    expect(yesNo(false)).toBe('NO');
});
