// Order Staleness helpers: parser (4 or 5 tokens), validator, body
// shape, tier badge map, hour formatter, demo invariants.

import { test, expect } from 'vitest';
import {
    parseOrderBlob, validateInputs, buildBody,
    tierBadge, fmtHours, makeDemoData, nowIso,
} from '../js/_order_staleness_inputs.js';

// ── parseOrderBlob ─────────────────────────────────────────────────

test('parseOrderBlob accepts 4-token (no last_modified) and 5-token forms', () => {
    const r = parseOrderBlob(`
        A1 AAPL 2024-06-15T10:00:00Z buy
        B1 TSLA 2024-06-14T10:00:00Z 2024-06-15T08:00:00Z sell_stop
    `);
    expect(r.errors).toEqual([]);
    expect(r.orders.length).toBe(2);
    expect(r.orders[0].last_modified_at).toBe(null);
    expect(r.orders[1].last_modified_at).toBe('2024-06-15T08:00:00.000Z');
});

test('parseOrderBlob uppercases symbol but keeps order_id case', () => {
    const r = parseOrderBlob('a1 aapl 2024-06-15T10:00:00Z buy');
    expect(r.orders[0].symbol).toBe('AAPL');
    expect(r.orders[0].order_id).toBe('a1');
});

test('parseOrderBlob rejects wrong token count', () => {
    expect(parseOrderBlob('A1 AAPL').errors[0].message).toMatch(/expected 4 or 5 tokens/);
    expect(parseOrderBlob('A1 AAPL 2024-01-01T00:00:00Z X Y Z buy').errors[0].message).toMatch(/expected 4 or 5/);
});

test('parseOrderBlob rejects bad timestamp', () => {
    expect(parseOrderBlob('A1 AAPL not-a-date buy').errors[0].message).toMatch(/bad placed_at/);
    expect(parseOrderBlob('A1 AAPL 2024-06-15T10:00:00Z not-a-date sell').errors[0].message)
        .toMatch(/bad last_modified_at/);
});

test('parseOrderBlob rejects bad side', () => {
    expect(parseOrderBlob('A1 AAPL 2024-06-15T10:00:00Z bidirectional').errors[0].message)
        .toMatch(/side must be one of/);
});

test('parseOrderBlob accepts all 4 side enum values', () => {
    const r = parseOrderBlob(`
        A1 AAPL 2024-06-15T10:00:00Z buy
        A2 AAPL 2024-06-15T10:00:00Z sell
        A3 AAPL 2024-06-15T10:00:00Z buy_stop
        A4 AAPL 2024-06-15T10:00:00Z sell_stop
    `);
    expect(r.errors).toEqual([]);
    expect(r.orders.length).toBe(4);
});

test('parseOrderBlob non-string returns 1 error', () => {
    expect(parseOrderBlob(null).errors.length).toBe(1);
});

// ── validateInputs ────────────────────────────────────────────────

const okOrders = [{ order_id: 'A', symbol: 'X', placed_at: '2024-01-01T00:00:00Z', last_modified_at: null, side: 'buy' }];
const okThresh = { warn_hours: 24, stale_hours: 72, forgotten_hours: 168 };

test('validate accepts good inputs', () => {
    expect(validateInputs(okOrders, '2024-06-15T00:00:00Z', okThresh)).toBe(null);
});

test('validate rejects empty orders', () => {
    expect(validateInputs([], '2024-06-15T00:00:00Z', okThresh)).toMatch(/at least 1 order/);
});

test('validate rejects bad now', () => {
    expect(validateInputs(okOrders, '', okThresh)).toMatch(/now must be/);
    expect(validateInputs(okOrders, 'garbage', okThresh)).toMatch(/not a valid timestamp/);
});

test('validate enforces threshold ordering: warn < stale < forgotten', () => {
    expect(validateInputs(okOrders, '2024-06-15T00:00:00Z', { ...okThresh, warn_hours: 0 }))
        .toMatch(/warn_hours/);
    expect(validateInputs(okOrders, '2024-06-15T00:00:00Z', { ...okThresh, stale_hours: 24 }))
        .toMatch(/stale_hours/);
    expect(validateInputs(okOrders, '2024-06-15T00:00:00Z', { ...okThresh, forgotten_hours: 72 }))
        .toMatch(/forgotten_hours/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody emits backend StalenessBody shape', () => {
    expect(buildBody(okOrders, '2024-06-15T00:00:00Z', okThresh)).toEqual({
        orders: okOrders, now: '2024-06-15T00:00:00Z', thresholds: okThresh,
    });
});

// ── tierBadge ─────────────────────────────────────────────────────

test('tierBadge maps all 4 enum values', () => {
    expect(tierBadge('fresh').cls).toBe('pos');
    expect(tierBadge('aging').cls).toBe('');
    expect(tierBadge('stale').cls).toBe('neg');
    expect(tierBadge('forgotten').cls).toBe('neg');
});

test('tierBadge falls through for unknown tier', () => {
    expect(tierBadge('xyz').label).toBe('xyz');
    expect(tierBadge(null).label).toBe('—');
});

// ── fmtHours ──────────────────────────────────────────────────────

test('fmtHours uses h for <24, d for ≥24', () => {
    expect(fmtHours(5.4)).toBe('5.4h');
    expect(fmtHours(23.9)).toBe('23.9h');
    expect(fmtHours(48)).toBe('2.0d');
    expect(fmtHours(168)).toBe('7.0d');
    expect(fmtHours(NaN)).toBe('—');
});

// ── makeDemoData ──────────────────────────────────────────────────

test('makeDemoData produces 12 orders + valid now', () => {
    const { orders, now } = makeDemoData();
    expect(orders.length).toBe(12);
    expect(typeof now).toBe('string');
    expect(Number.isNaN(new Date(now).getTime())).toBe(false);
});

test('makeDemoData orders span all 4 tiers at default thresholds (24/72/168)', () => {
    const { orders, now } = makeDemoData();
    const nowMs = new Date(now).getTime();
    const ages = orders.map(o => {
        const touched = new Date(o.last_modified_at || o.placed_at).getTime();
        return (nowMs - touched) / 3600_000;
    });
    expect(ages.some(a => a < 24)).toBe(true);                        // fresh
    expect(ages.some(a => a >= 24 && a < 72)).toBe(true);             // aging
    expect(ages.some(a => a >= 72 && a < 168)).toBe(true);            // stale
    expect(ages.some(a => a >= 168)).toBe(true);                      // forgotten
});

test('makeDemoData modify-touch-resets-clock: AMD placed 100h ago but modified 48h ago lands aging not stale', () => {
    const { orders, now } = makeDemoData();
    const amd = orders.find(o => o.symbol === 'AMD');
    const nowMs = new Date(now).getTime();
    const ageHours = (nowMs - new Date(amd.last_modified_at).getTime()) / 3600_000;
    expect(ageHours).toBeGreaterThanOrEqual(24);
    expect(ageHours).toBeLessThan(72);
});

// ── nowIso ────────────────────────────────────────────────────────

test('nowIso returns a parseable ISO string', () => {
    const s = nowIso();
    expect(typeof s).toBe('string');
    expect(Number.isNaN(new Date(s).getTime())).toBe(false);
});
