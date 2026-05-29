// Order Flow helpers: tick parser, validator, body shape, side badge,
// cumulative-flow + local-aggregate (sanity-check vs network),
// demo invariants, formatters.

import { test, expect } from 'vitest';
import {
    parseTickBlob, validateInputs, buildBody,
    sideBadge, cumulativeFlow, localAggregate,
    makeDemoTicks, fmtN, fmtImbalance,
} from '../js/_order_flow_inputs.js';

// ── parseTickBlob ──────────────────────────────────────────────────

test('parseTickBlob handles whitespace + commas + comments', () => {
    const r = parseTickBlob('# header\n100.05 250 100.04 100.05\n100.04, 1200, 100.04, 100.05');
    expect(r.errors).toEqual([]);
    expect(r.ticks).toEqual([
        { price: 100.05, volume: 250, bid: 100.04, ask: 100.05 },
        { price: 100.04, volume: 1200, bid: 100.04, ask: 100.05 },
    ]);
});

test('parseTickBlob rejects wrong token count', () => {
    expect(parseTickBlob('100 250 100').errors[0].message).toMatch(/expected 4 tokens/);
});

test('parseTickBlob rejects non-positive price/volume/bid', () => {
    const r = parseTickBlob('0 1 1 1\n1 0 1 1\n1 1 0 1');
    expect(r.ticks).toEqual([]);
    expect(r.errors.length).toBe(3);
});

test('parseTickBlob rejects ask < bid', () => {
    expect(parseTickBlob('100 100 100 99').errors[0].message).toMatch(/ask must be ≥ bid/);
});

test('parseTickBlob accepts ask == bid (crossed-mid edge)', () => {
    const r = parseTickBlob('100 100 100 100');
    expect(r.errors).toEqual([]);
    expect(r.ticks).toEqual([{ price: 100, volume: 100, bid: 100, ask: 100 }]);
});

test('parseTickBlob non-string returns 1 error', () => {
    expect(parseTickBlob(null).errors.length).toBe(1);
});

// ── validateInputs / buildBody ────────────────────────────────────

test('validate accepts ≥5 ticks, rejects fewer', () => {
    expect(validateInputs(Array(5).fill({ price: 1, volume: 1, bid: 1, ask: 1 }))).toBe(null);
    expect(validateInputs(Array(3).fill({ price: 1, volume: 1, bid: 1, ask: 1 }))).toMatch(/at least 5/);
});

test('buildBody emits backend OrderFlowBody shape', () => {
    expect(buildBody([{ price: 1, volume: 1, bid: 1, ask: 1 }]))
        .toEqual({ ticks: [{ price: 1, volume: 1, bid: 1, ask: 1 }] });
});

// ── sideBadge ─────────────────────────────────────────────────────

test('sideBadge maps buy/sell/uncertain/unknown', () => {
    expect(sideBadge('buy').cls).toBe('pos');
    expect(sideBadge('sell').cls).toBe('neg');
    expect(sideBadge('uncertain').cls).toBe('');
    expect(sideBadge('garbage').cls).toBe('');
});

// ── cumulativeFlow ────────────────────────────────────────────────

test('cumulativeFlow accumulates buy + sell + net correctly', () => {
    const classified = [
        { volume: 100, side: 'buy' },
        { volume: 50,  side: 'sell' },
        { volume: 200, side: 'buy' },
        { volume: 30,  side: 'uncertain' },
    ];
    const { xs, buy, sell, net } = cumulativeFlow(classified);
    expect(xs).toEqual([0, 1, 2, 3]);
    expect(buy).toEqual([100, 100, 300, 300]);
    expect(sell).toEqual([-0, -50, -50, -50]);
    expect(net).toEqual([100, 50, 250, 250]);
});

test('cumulativeFlow empty / non-array safe', () => {
    expect(cumulativeFlow([])).toEqual({ xs: [], buy: [], sell: [], net: [] });
    expect(cumulativeFlow(null)).toEqual({ xs: [], buy: [], sell: [], net: [] });
});

// ── localAggregate ────────────────────────────────────────────────

test('localAggregate sums per-side and computes imbalance', () => {
    const a = localAggregate([
        { volume: 100, side: 'buy' },
        { volume: 60,  side: 'sell' },
        { volume: 40,  side: 'uncertain' },
    ]);
    expect(a.buy).toBe(100);
    expect(a.sell).toBe(60);
    expect(a.uncertain).toBe(40);
    expect(a.net).toBe(40);
    expect(a.imbalance).toBeCloseTo(40 / 160, 8);
});

test('localAggregate handles all-uncertain with zero imbalance', () => {
    const a = localAggregate([{ volume: 100, side: 'uncertain' }]);
    expect(a.imbalance).toBe(0);
});

test('localAggregate drops malformed entries', () => {
    const a = localAggregate([null, { volume: NaN, side: 'buy' }, { volume: 50, side: 'buy' }]);
    expect(a.buy).toBe(50);
});

// ── makeDemoTicks ─────────────────────────────────────────────────

test('makeDemoTicks deterministic + exactly 400 ticks', () => {
    const a = makeDemoTicks(42);
    const b = makeDemoTicks(42);
    expect(a).toEqual(b);
    expect(a.length).toBe(400);
});

test('makeDemoTicks all bid > 0, ask >= bid, volume > 0', () => {
    const t = makeDemoTicks(7);
    expect(t.every(x => x.bid > 0 && x.ask >= x.bid && x.volume > 0)).toBe(true);
});

test('makeDemoTicks at-ask print fraction clearly > at-bid (engineered buy pressure)', () => {
    // Engineered 65/20/15 mix → atAsk/atBid expected ~3.25. Assert ≥ 2×
    // to allow LCG seed jitter while still proving the directional bias.
    for (const seed of [1, 7, 42, 1337]) {
        const t = makeDemoTicks(seed);
        const atAsk = t.filter(x => Math.abs(x.price - x.ask) < 1e-9).length;
        const atBid = t.filter(x => Math.abs(x.price - x.bid) < 1e-9).length;
        expect(atAsk).toBeGreaterThan(atBid * 2);
    }
});

// ── formatters ────────────────────────────────────────────────────

test('fmtN locale-formats large integers', () => {
    expect(fmtN(1234567)).toBe('1,234,567');
    expect(fmtN(NaN)).toBe('—');
});

test('fmtImbalance signs positive + 4-decimal', () => {
    expect(fmtImbalance(0.4567)).toBe('+0.4567');
    expect(fmtImbalance(-0.123)).toBe('-0.1230');
    expect(fmtImbalance(NaN)).toBe('—');
});
