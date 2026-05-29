// Market Impact pure helpers: trade parser, validator, body shape,
// bucket logic, demo-data invariants, formatters.

import { test, expect } from 'vitest';
import {
    parseTradeBlob, validateInputs, buildBody,
    bucketIndex, bucketParticipations,
    makeDemoTrades, BUCKET_LABELS, fmtBps, fmtN,
} from '../js/_market_impact_inputs.js';

// ── parseTradeBlob ─────────────────────────────────────────────────

test('parseTradeBlob handles whitespace + commas + comments', () => {
    const r = parseTradeBlob('# header\n2500 5000000 2.1\n120000, 5000000, 12.5');
    expect(r.errors).toEqual([]);
    expect(r.trades).toEqual([
        { qty: 2500, adv: 5_000_000, slippage_bps: 2.1 },
        { qty: 120000, adv: 5_000_000, slippage_bps: 12.5 },
    ]);
});

test('parseTradeBlob rejects wrong token count', () => {
    const r = parseTradeBlob('100 200');
    expect(r.errors.length).toBe(1);
    expect(r.errors[0].message).toMatch(/expected 3 tokens/);
});

test('parseTradeBlob rejects non-positive qty or adv', () => {
    const r = parseTradeBlob('0 100 5\n100 0 5\n-1 100 5\n100 -5 5');
    expect(r.trades).toEqual([]);
    expect(r.errors.length).toBe(4);
});

test('parseTradeBlob accepts negative slippage_bps (favorable fill)', () => {
    const r = parseTradeBlob('1000 100000 -3.5');
    expect(r.errors).toEqual([]);
    expect(r.trades[0].slippage_bps).toBe(-3.5);
});

test('parseTradeBlob rejects non-finite slippage', () => {
    const r = parseTradeBlob('1000 100000 abc');
    expect(r.trades).toEqual([]);
    expect(r.errors[0].message).toMatch(/slippage_bps must be finite/);
});

test('parseTradeBlob returns error on non-string input', () => {
    const r = parseTradeBlob(undefined);
    expect(r.trades).toEqual([]);
    expect(r.errors.length).toBe(1);
});

// ── validateInputs ─────────────────────────────────────────────────

test('validate accepts ≥5 trades + spike > 0', () => {
    const trades = Array(10).fill({ qty: 100, adv: 1e6, slippage_bps: 2 });
    expect(validateInputs(trades, 30)).toBe(null);
});

test('validate rejects < 5 trades', () => {
    expect(validateInputs(Array(2).fill({ qty: 1, adv: 1, slippage_bps: 0 }), 30))
        .toMatch(/at least 5 trades/);
});

test('validate rejects non-positive spike threshold', () => {
    const trades = Array(10).fill({ qty: 100, adv: 1e6, slippage_bps: 2 });
    expect(validateInputs(trades, 0)).toMatch(/spike_bps/);
    expect(validateInputs(trades, -5)).toMatch(/spike_bps/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody emits backend MarketImpactBody shape', () => {
    const trades = [{ qty: 100, adv: 1000, slippage_bps: 1 }];
    expect(buildBody(trades, 30)).toEqual({ trades, spike_bps: 30 });
});

// ── bucketIndex ───────────────────────────────────────────────────

test('bucketIndex respects canonical band caps (≤0.1%, ≤0.5%, ≤1%, ≤5%, ≤10%, >10%)', () => {
    expect(bucketIndex(0.0005)).toBe(0);    // 0.05%
    expect(bucketIndex(0.001)).toBe(0);     // boundary
    expect(bucketIndex(0.003)).toBe(1);     // 0.3%
    expect(bucketIndex(0.008)).toBe(2);     // 0.8%
    expect(bucketIndex(0.03)).toBe(3);      // 3%
    expect(bucketIndex(0.08)).toBe(4);      // 8%
    expect(bucketIndex(0.50)).toBe(5);      // 50%
});

test('bucketIndex labels match backend band order', () => {
    expect(BUCKET_LABELS).toEqual([
        '< 0.1% ADV', '0.1-0.5% ADV', '0.5-1% ADV',
        '1-5% ADV', '5-10% ADV', '> 10% ADV',
    ]);
});

// ── bucketParticipations ──────────────────────────────────────────

test('bucketParticipations counts trades per band correctly', () => {
    const trades = [
        { qty: 500, adv: 1e6, slippage_bps: 1 },        // 0.05% → bucket 0
        { qty: 5000, adv: 1e6, slippage_bps: 5 },       // 0.5%  → bucket 1
        { qty: 100_000, adv: 1e6, slippage_bps: 50 },   // 10%   → bucket 4
        { qty: 200_000, adv: 1e6, slippage_bps: 80 },   // 20%   → bucket 5
    ];
    expect(bucketParticipations(trades)).toEqual([1, 1, 0, 0, 1, 1]);
});

test('bucketParticipations skips trades with non-positive adv', () => {
    const trades = [
        { qty: 100, adv: 0, slippage_bps: 5 },
        { qty: 100, adv: NaN, slippage_bps: 5 },
        { qty: 100, adv: 1_000_000, slippage_bps: 5 },
    ];
    const counts = bucketParticipations(trades);
    expect(counts.reduce((a, b) => a + b, 0)).toBe(1);
});

// ── makeDemoTrades ────────────────────────────────────────────────

test('makeDemoTrades is deterministic for fixed seed', () => {
    const a = makeDemoTrades(42);
    const b = makeDemoTrades(42);
    expect(a).toEqual(b);
});

test('makeDemoTrades emits 400 trades with required fields', () => {
    const trades = makeDemoTrades(1);
    expect(trades.length).toBe(400);
    expect(trades.every(t =>
        Number.isFinite(t.qty) && t.qty > 0 &&
        Number.isFinite(t.adv) && t.adv > 0 &&
        Number.isFinite(t.slippage_bps))).toBe(true);
});

test('makeDemoTrades exhibits slippage cliff: avg(>5% bucket) > avg(<0.1% bucket)', () => {
    const trades = makeDemoTrades(1);
    const tiny = trades.filter(t => t.qty / t.adv <= 0.001);
    const big  = trades.filter(t => t.qty / t.adv > 0.05);
    if (tiny.length === 0 || big.length === 0) {
        throw new Error('demo seed produced no tiny or big trades');
    }
    const avgTiny = tiny.reduce((a, t) => a + t.slippage_bps, 0) / tiny.length;
    const avgBig  = big.reduce((a, t) => a + t.slippage_bps, 0) / big.length;
    expect(avgBig).toBeGreaterThan(avgTiny * 3);  // cliff = ≥3× cost
});

// ── formatters ─────────────────────────────────────────────────────

test('fmtBps emits 1-decimal with bps suffix', () => {
    expect(fmtBps(12.345)).toBe('12.3 bps');
    expect(fmtBps(NaN)).toBe('—');
});

test('fmtN locale-formats large integers', () => {
    expect(fmtN(1234567)).toBe('1,234,567');
    expect(fmtN(NaN)).toBe('—');
});
