// Implementation Shortfall helpers: validator, body shape, component
// decomposition, sign-class picker, fill classifier, formatters.

import { test, expect } from 'vitest';
import {
    validateInputs, buildBody, decompose, costSignClass,
    fillKind, fmtUSD, fmtBps, fmtPct,
    COMPONENT_KEYS, COMPONENT_LABELS,
} from '../js/_implementation_shortfall_inputs.js';

const baseInput = {
    direction: 'buy',
    decision_mid: 100, arrival_mid: 100.05, vwap_fill: 100.08,
    final_mid: 100.20, half_spread_at_decision: 0.02,
    intended_qty: 10_000, filled_qty: 9_500,
};

// ── validateInputs ─────────────────────────────────────────────────

test('validate accepts canonical buy lifecycle', () => {
    expect(validateInputs(baseInput)).toBe(null);
});

test('validate accepts a full-fill sell', () => {
    expect(validateInputs({ ...baseInput, direction: 'sell', filled_qty: 10_000 })).toBe(null);
});

test('validate rejects bad direction', () => {
    expect(validateInputs({ ...baseInput, direction: 'short' })).toMatch(/direction/);
});

test('validate rejects non-positive mids', () => {
    expect(validateInputs({ ...baseInput, decision_mid: 0 })).toMatch(/decision_mid/);
    expect(validateInputs({ ...baseInput, arrival_mid: -1 })).toMatch(/arrival_mid/);
    expect(validateInputs({ ...baseInput, final_mid: 0 })).toMatch(/final_mid/);
});

test('validate allows zero vwap_fill (unfilled), rejects negative', () => {
    expect(validateInputs({ ...baseInput, vwap_fill: 0, filled_qty: 0 })).toBe(null);
    expect(validateInputs({ ...baseInput, vwap_fill: -0.01 })).toMatch(/vwap_fill/);
});

test('validate rejects negative half-spread', () => {
    expect(validateInputs({ ...baseInput, half_spread_at_decision: -0.01 }))
        .toMatch(/half_spread/);
});

test('validate enforces filled_qty ≤ intended_qty', () => {
    expect(validateInputs({ ...baseInput, filled_qty: 10_001 })).toMatch(/filled_qty/);
});

test('validate accepts equal filled = intended (full fill)', () => {
    expect(validateInputs({ ...baseInput, filled_qty: 10_000 })).toBe(null);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody emits exact backend ShortfallInput shape', () => {
    expect(buildBody(baseInput)).toEqual(baseInput);
});

// ── decompose ──────────────────────────────────────────────────────

test('decompose returns four components with labels and shares summing to 1 (signed-share variant)', () => {
    const report = {
        spread_cost: 1, timing_cost: 2, impact_cost: 1, opportunity_cost: -1,
        total_dollars: 3, total_bps: 30,
    };
    const items = decompose(report);
    expect(items.map(it => it.key)).toEqual(COMPONENT_KEYS);
    expect(items.map(it => it.label)).toEqual(COMPONENT_KEYS.map(k => COMPONENT_LABELS[k]));
    const absSumShares = items.reduce((a, it) => a + Math.abs(it.share), 0);
    expect(absSumShares).toBeCloseTo(1.0, 6);
});

test('decompose handles all-zero report (no NaN, all shares = 0)', () => {
    const items = decompose({
        spread_cost: 0, timing_cost: 0, impact_cost: 0, opportunity_cost: 0,
        total_dollars: 0, total_bps: 0,
    });
    expect(items.every(it => it.share === 0)).toBe(true);
});

test('decompose returns zero items when report is null', () => {
    const items = decompose(null);
    expect(items.length).toBe(4);
    expect(items.every(it => it.value === 0 && it.share === 0)).toBe(true);
});

// ── costSignClass ──────────────────────────────────────────────────

test('costSignClass: positive cost → neg (paid up), negative → pos (captured)', () => {
    expect(costSignClass(0.05)).toBe('neg');
    expect(costSignClass(-0.05)).toBe('pos');
    expect(costSignClass(0)).toBe('');
    expect(costSignClass(NaN)).toBe('');
});

// ── fillKind ──────────────────────────────────────────────────────

test('fillKind classifies full / partial / unfilled', () => {
    expect(fillKind(100, 100)).toBe('full');
    expect(fillKind(100, 50)).toBe('partial');
    expect(fillKind(100, 0)).toBe('unfilled');
});

test('fillKind treats float-jitter equality as full', () => {
    expect(fillKind(100, 100 - 1e-12)).toBe('full');
});

// ── formatters ─────────────────────────────────────────────────────

test('fmtUSD signs and 2-decimal', () => {
    expect(fmtUSD(12.345)).toBe('$12.35');
    expect(fmtUSD(-12.345)).toBe('-$12.35');
    expect(fmtUSD(NaN)).toBe('—');
});

test('fmtBps emits 1-decimal with bps suffix', () => {
    expect(fmtBps(5.678)).toBe('5.7 bps');
    expect(fmtBps(NaN)).toBe('—');
});

test('fmtPct emits 1-decimal percentage', () => {
    expect(fmtPct(0.054)).toBe('5.4%');
    expect(fmtPct(NaN)).toBe('—');
});
