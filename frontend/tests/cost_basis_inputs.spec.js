// Cost-basis helpers: parser, validator, body shape, localClose
// Rust-mirror, suggestMethod logic, badges, demos.

import { test, expect } from 'vitest';
import {
    METHODS, parseLotBlob, validateInputs, buildBody, localClose, dec,
    isValidDate, realizedBadge, suggestMethod, methodLabelKey,
    makeDemoLots, makeDemoQtyPrice,
    fmtUSD, fmtUSDSigned, fmtNum,
} from '../js/_cost_basis_inputs.js';

const lot = (id, date, qty, cost) => ({ lot_id: id, acquired: date, qty, cost_per_share: cost });

// ── constants ─────────────────────────────────────────────────────

test('METHODS exposes the four Rust enum values', () => {
    expect(METHODS).toEqual(['fifo', 'lifo', 'hifo', 'lofo']);
});

// ── parser ────────────────────────────────────────────────────────

test('parseLotBlob: 4 tokens; ignores comments', () => {
    const r = parseLotBlob('A 2024-01-15 100 100\n# note\nB 2024-06-10 100 150');
    expect(r.errors).toEqual([]);
    expect(r.lots).toEqual([lot('A', '2024-01-15', 100, 100), lot('B', '2024-06-10', 100, 150)]);
});

test('parseLotBlob: rejects bad date / non-positive qty / negative cost / dup lot_id / wrong token count', () => {
    expect(parseLotBlob('A 2024/01/15 100 100').errors[0].message).toMatch(/acquired/);
    expect(parseLotBlob('A 2024-01-15 0 100').errors[0].message).toMatch(/qty/);
    expect(parseLotBlob('A 2024-01-15 100 -1').errors[0].message).toMatch(/cost_per_share/);
    expect(parseLotBlob('A 2024-01-15 100').errors[0].message).toMatch(/4 tokens/);
    const dup = parseLotBlob('A 2024-01-15 50 100\nA 2024-06-01 50 150');
    expect(dup.errors[0].message).toMatch(/duplicate/);
});

test('parseLotBlob: accepts cost=0 (worthless basis edge case)', () => {
    expect(parseLotBlob('A 2024-01-15 100 0').errors).toEqual([]);
});

test('parseLotBlob: non-string returns 1 error', () => {
    expect(parseLotBlob(null).errors.length).toBe(1);
});

// ── date helper ───────────────────────────────────────────────────

test('isValidDate strict YYYY-MM-DD; rejects malformed', () => {
    expect(isValidDate('2024-01-15')).toBe(true);
    expect(isValidDate('2024-13-01')).toBe(false);
    expect(isValidDate('2024/01/15')).toBe(false);
});

// ── validator / buildBody ─────────────────────────────────────────

test('validate accepts good inputs', () => {
    expect(validateInputs([lot('A', '2024-01-15', 100, 100)], 50, 200, 'fifo')).toBe(null);
});

test('validate rejects bad method / negative qty/price / non-finite', () => {
    expect(validateInputs([], 50, 200, 'sma')).toMatch(/method/);
    expect(validateInputs([], -1, 200, 'fifo')).toMatch(/qty_to_close/);
    expect(validateInputs([], 50, -1, 'fifo')).toMatch(/price_per_share/);
    expect(validateInputs([], NaN, 200, 'fifo')).toMatch(/qty_to_close/);
});

test('buildBody: stringifies Decimal fields per rust_decimal contract', () => {
    const body = buildBody([lot('A', '2024-01-15', 100, 100)], 50, 200, 'hifo');
    expect(body.lots[0]).toEqual({
        lot_id: 'A', acquired: '2024-01-15', qty: '100', cost_per_share: '100',
    });
    expect(body.qty_to_close).toBe('50');
    expect(body.price_per_share).toBe('200');
    expect(body.method).toBe('hifo');
});

// ── localClose parity (one test per Rust property) ────────────────

const LOTS = [
    lot('A', '2024-01-15', 100, 100),
    lot('B', '2024-06-10', 100, 150),
    lot('C', '2025-03-05', 100, 125),
];

test('local: empty lots → empty closes', () => {
    expect(localClose([], 100, 200, 'fifo').closes).toEqual([]);
});

test('local: FIFO takes oldest first (lot A)', () => {
    const r = localClose(LOTS, 100, 200, 'fifo');
    expect(r.closes[0].lot_id).toBe('A');
    expect(r.total_realized).toBe(10000);
});

test('local: LIFO takes newest first (lot C)', () => {
    const r = localClose(LOTS, 100, 200, 'lifo');
    expect(r.closes[0].lot_id).toBe('C');
    expect(r.total_realized).toBe(7500);
});

test('local: HIFO takes highest cost first (lot B @ $150)', () => {
    const r = localClose(LOTS, 100, 200, 'hifo');
    expect(r.closes[0].lot_id).toBe('B');
    expect(r.total_realized).toBe(5000);
});

test('local: LOFO takes lowest cost first (lot A @ $100), maximizing gain', () => {
    const r = localClose(LOTS, 100, 200, 'lofo');
    expect(r.closes[0].lot_id).toBe('A');
    expect(r.total_realized).toBe(10000);
});

test('local: closing 250 spans 3 lots, totals = $18,750', () => {
    const r = localClose(LOTS, 250, 200, 'fifo');
    expect(r.closes.length).toBe(3);
    expect(r.closes[0].qty_closed).toBe(100);
    expect(r.closes[1].qty_closed).toBe(100);
    expect(r.closes[2].qty_closed).toBe(50);
    expect(r.total_realized).toBe(18750);
});

test('local: over-close (500 vs 300 available) leaves 200 remaining', () => {
    const r = localClose(LOTS, 500, 200, 'fifo');
    expect(r.qty_remaining_to_close).toBe(200);
});

test('local: partial-lot close keeps remaining state up to the caller', () => {
    const r = localClose(LOTS, 50, 200, 'fifo');
    expect(r.closes.length).toBe(1);
    expect(r.closes[0].qty_closed).toBe(50);
});

test('local: HIFO < LOFO on gains (HIFO minimizes realized)', () => {
    const hifo = localClose(LOTS, 100, 200, 'hifo');
    const lofo = localClose(LOTS, 100, 200, 'lofo');
    expect(hifo.total_realized).toBeLessThan(lofo.total_realized);
});

test('local: qty_to_close=0 → no closes, no realized', () => {
    const r = localClose(LOTS, 0, 200, 'fifo');
    expect(r.closes).toEqual([]);
    expect(r.total_realized).toBe(0);
    expect(r.qty_remaining_to_close).toBe(0);
});

test('local: each close emits realized_per_share = price - cost', () => {
    const r = localClose(LOTS, 100, 200, 'fifo');
    expect(r.closes[0].realized_per_share).toBe(100);  // 200 - 100
    expect(r.closes[0].realized_total).toBe(10000);    // × 100
});

// ── suggestMethod ─────────────────────────────────────────────────

test('suggestMethod: all-gain → HIFO (minimize realized)', () => {
    const gainLots = [lot('X', '2024-01-15', 50, 50), lot('Y', '2024-06-10', 50, 75)];
    expect(suggestMethod(gainLots, 50, 200)).toBe('hifo');
});

test('suggestMethod: all-loss → HIFO (closes highest cost first → biggest loss magnitude)', () => {
    const lossLots = [lot('X', '2024-01-15', 50, 200), lot('Y', '2024-06-10', 50, 250)];
    expect(suggestMethod(lossLots, 50, 100)).toBe('hifo');
});

test('suggestMethod: mixed → FIFO (IRS default)', () => {
    // LOTS has some lots above and below $200 sale price.
    expect(suggestMethod(LOTS, 100, 130)).toBe('fifo');
});

test('suggestMethod: empty lots → FIFO fallback', () => {
    expect(suggestMethod([], 100, 200)).toBe('fifo');
});

// ── realizedBadge ─────────────────────────────────────────────────

test('realizedBadge: gain = neg (taxable), loss = pos (harvestable), scratch = empty', () => {
    expect(realizedBadge(1000).cls).toBe('neg');
    expect(realizedBadge(-500).cls).toBe('pos');
    expect(realizedBadge(0).cls).toBe('');
    expect(realizedBadge(NaN).key).toMatch(/unknown/);
});

// ── methodLabelKey ────────────────────────────────────────────────

test('methodLabelKey: returns view.cost_basis.method.<m>', () => {
    expect(methodLabelKey('fifo')).toBe('view.cost_basis.method.fifo');
    expect(methodLabelKey()).toBe('view.cost_basis.method.unknown');
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset produces a valid close', () => {
    for (const k of ['classic', 'gain-only', 'loss-only', 'many-lots', 'single-lot', 'overclose']) {
        const lots = makeDemoLots(k);
        const { qty_to_close, price_per_share } = makeDemoQtyPrice(k);
        const r = localClose(lots, qty_to_close, price_per_share, 'fifo');
        expect(Number.isFinite(r.total_realized)).toBe(true);
    }
});

test('demo gain-only: HIFO realizes less than LOFO', () => {
    const lots = makeDemoLots('gain-only');
    const { qty_to_close, price_per_share } = makeDemoQtyPrice('gain-only');
    const hi = localClose(lots, qty_to_close, price_per_share, 'hifo').total_realized;
    const lo = localClose(lots, qty_to_close, price_per_share, 'lofo').total_realized;
    expect(hi).toBeLessThan(lo);
});

test('demo loss-only: HIFO realizes the LARGEST loss (most negative)', () => {
    // For losses: bigger cost → bigger |loss|. HIFO closes high-cost lots first.
    const lots = makeDemoLots('loss-only');
    const { qty_to_close, price_per_share } = makeDemoQtyPrice('loss-only');
    const hi = localClose(lots, qty_to_close, price_per_share, 'hifo').total_realized;
    const lo = localClose(lots, qty_to_close, price_per_share, 'lofo').total_realized;
    expect(hi).toBeLessThan(lo);
});

test('demo overclose: 100 remaining after 200 sold against 100-total lots', () => {
    const lots = makeDemoLots('overclose');
    const { qty_to_close, price_per_share } = makeDemoQtyPrice('overclose');
    const r = localClose(lots, qty_to_close, price_per_share, 'fifo');
    expect(r.qty_remaining_to_close).toBe(100);
});

// ── dec / formatters ──────────────────────────────────────────────

test('dec coerces strings + guards null/garbage', () => {
    expect(dec('100.5')).toBe(100.5);
    expect(dec(null)).toBe(0);
    expect(dec('abc')).toBe(0);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtUSD(1234)).toBe('$1234.00');
    expect(fmtUSDSigned(-100)).toBe('-$100.00');
    expect(fmtNum(123.456, 2)).toBe('123.46');
    expect(fmtUSD(NaN)).toBe('—');
});
