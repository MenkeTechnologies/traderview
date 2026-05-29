// Pyramid plan helpers: tranche parser, validator, body shape,
// direction pre-flight, Decimal-string coercion, demo invariants,
// avg-cost series extraction, formatters.

import { test, expect } from 'vitest';
import {
    parseTrancheBlob, validateInputs, buildBody,
    directionMisordered, decToNum, makeDemoData,
    avgCostSeries, fmtN, fmtInt, fmtUSD,
} from '../js/_pyramid_inputs.js';

// ── parseTrancheBlob ───────────────────────────────────────────────

test('parseTrancheBlob accepts whitespace + commas + comments', () => {
    const r = parseTrancheBlob('# header\n105 75\n110, 50');
    expect(r.errors).toEqual([]);
    expect(r.tranches).toEqual([
        { trigger_price: 105, qty: 75 },
        { trigger_price: 110, qty: 50 },
    ]);
});

test('parseTrancheBlob rejects wrong token count', () => {
    expect(parseTrancheBlob('105').errors[0].message).toMatch(/expected 2 tokens/);
});

test('parseTrancheBlob rejects non-positive trigger_price', () => {
    expect(parseTrancheBlob('0 100').errors[0].message).toMatch(/trigger_price/);
    expect(parseTrancheBlob('-1 100').errors[0].message).toMatch(/trigger_price/);
});

test('parseTrancheBlob rejects non-positive qty', () => {
    expect(parseTrancheBlob('100 0').errors[0].message).toMatch(/qty/);
});

test('parseTrancheBlob non-string returns 1 error', () => {
    expect(parseTrancheBlob(null).errors.length).toBe(1);
});

// ── validateInputs ────────────────────────────────────────────────

const okPlan = {
    kind: 'pyramid_up', side: 'long',
    initial_qty: 100, initial_entry: 100,
    tranches: [{ trigger_price: 105, qty: 75 }],
};

test('validate accepts canonical', () => {
    expect(validateInputs(okPlan)).toBe(null);
});

test('validate rejects bad enums', () => {
    expect(validateInputs({ ...okPlan, kind: 'martingale' })).toMatch(/kind/);
    expect(validateInputs({ ...okPlan, side: 'flat' })).toMatch(/side/);
});

test('validate rejects non-positive initial_qty / initial_entry', () => {
    expect(validateInputs({ ...okPlan, initial_qty: 0 })).toMatch(/initial_qty/);
    expect(validateInputs({ ...okPlan, initial_entry: 0 })).toMatch(/initial_entry/);
});

test('validate rejects empty tranches', () => {
    expect(validateInputs({ ...okPlan, tranches: [] })).toMatch(/at least 1 tranche/);
});

test('validate rejects tranche with non-positive trigger / qty', () => {
    expect(validateInputs({ ...okPlan, tranches: [{ trigger_price: 0, qty: 50 }] })).toMatch(/trigger_price/);
    expect(validateInputs({ ...okPlan, tranches: [{ trigger_price: 100, qty: 0 }] })).toMatch(/qty/);
});

// ── buildBody (Decimal-as-string) ─────────────────────────────────

test('buildBody stringifies all numeric scalars (Decimal contract)', () => {
    const body = buildBody({
        ...okPlan,
        initial_qty: 100, initial_entry: 50.5,
        tranches: [{ trigger_price: 55.5, qty: 75 }],
    });
    expect(body).toEqual({
        kind: 'pyramid_up', side: 'long',
        initial_qty: '100', initial_entry: '50.5',
        tranches: [{ trigger_price: '55.5', qty: '75' }],
    });
});

// ── directionMisordered ────────────────────────────────────────────

test('directionMisordered: pyramid_up long requires tranche > initial_entry', () => {
    expect(directionMisordered('pyramid_up', 'long', 100, [{ trigger_price: 105, qty: 1 }])).toBe(false);
    expect(directionMisordered('pyramid_up', 'long', 100, [{ trigger_price: 95,  qty: 1 }])).toBe(true);
});

test('directionMisordered: pyramid_up short requires tranche < initial_entry', () => {
    expect(directionMisordered('pyramid_up', 'short', 100, [{ trigger_price: 95,  qty: 1 }])).toBe(false);
    expect(directionMisordered('pyramid_up', 'short', 100, [{ trigger_price: 105, qty: 1 }])).toBe(true);
});

test('directionMisordered: scale_in long requires tranche < initial_entry', () => {
    expect(directionMisordered('scale_in', 'long', 100, [{ trigger_price: 95,  qty: 1 }])).toBe(false);
    expect(directionMisordered('scale_in', 'long', 100, [{ trigger_price: 105, qty: 1 }])).toBe(true);
});

test('directionMisordered: scale_in short requires tranche > initial_entry', () => {
    expect(directionMisordered('scale_in', 'short', 100, [{ trigger_price: 105, qty: 1 }])).toBe(false);
    expect(directionMisordered('scale_in', 'short', 100, [{ trigger_price: 95,  qty: 1 }])).toBe(true);
});

test('directionMisordered: any single bad tranche flags the whole plan', () => {
    expect(directionMisordered('pyramid_up', 'long', 100, [
        { trigger_price: 105, qty: 1 },   // OK
        { trigger_price: 110, qty: 1 },   // OK
        { trigger_price: 99,  qty: 1 },   // BAD
    ])).toBe(true);
});

test('directionMisordered: non-array tranches returns false (no plan = no violations)', () => {
    expect(directionMisordered('pyramid_up', 'long', 100, null)).toBe(false);
});

// ── decToNum ──────────────────────────────────────────────────────

test('decToNum coerces Decimal-string + number + null', () => {
    expect(decToNum('100.5')).toBe(100.5);
    expect(decToNum(100.5)).toBe(100.5);
    expect(decToNum(null)).toBeNaN();
    expect(decToNum('garbage')).toBeNaN();
});

// ── makeDemoData ──────────────────────────────────────────────────

test('makeDemoData returns valid plan for every (kind, side) combination', () => {
    for (const kind of ['pyramid_up', 'scale_in']) {
        for (const side of ['long', 'short']) {
            const d = makeDemoData(kind, side);
            expect(validateInputs(d)).toBe(null);
            expect(directionMisordered(kind, side, d.initial_entry, d.tranches)).toBe(false);
        }
    }
});

test('makeDemoData(pyramid_up, long) tranches all above initial_entry', () => {
    const d = makeDemoData('pyramid_up', 'long');
    expect(d.tranches.every(t => t.trigger_price > d.initial_entry)).toBe(true);
});

test('makeDemoData(scale_in, long) tranches all below initial_entry', () => {
    const d = makeDemoData('scale_in', 'long');
    expect(d.tranches.every(t => t.trigger_price < d.initial_entry)).toBe(true);
});

// ── avgCostSeries ─────────────────────────────────────────────────

test('avgCostSeries extracts parallel state-index and avg-cost arrays', () => {
    const report = {
        states: [
            { avg_cost: '100' },
            { avg_cost: '102.857' },
            { avg_cost: '105.5' },
        ],
    };
    const { xs, ys } = avgCostSeries(report);
    expect(xs).toEqual([0, 1, 2]);
    expect(ys[1]).toBeCloseTo(102.857, 6);
});

test('avgCostSeries handles null / non-array states', () => {
    expect(avgCostSeries(null)).toEqual({ xs: [], ys: [] });
    expect(avgCostSeries({ states: null })).toEqual({ xs: [], ys: [] });
});

// ── formatters ────────────────────────────────────────────────────

test('formatters', () => {
    expect(fmtN(105.123)).toBe('105.12');
    expect(fmtN(NaN)).toBe('—');
    expect(fmtInt(1500)).toBe('1,500');
    expect(fmtUSD(1234.5)).toBe('$1234.50');
    expect(fmtUSD(-50)).toBe('-$50.00');
});
