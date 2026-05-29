// Footprint helpers: tick parser, validator, body shape, delta cls,
// summarize, hotspots ranker, demo invariants, formatters.

import { test, expect } from 'vitest';
import {
    parseTickBlob, validateInputs, buildBody,
    deltaCls, summarize, imbalanceHotspots,
    makeDemoTicks, fmtN, fmtPrice, fmtSigned,
} from '../js/_footprint_inputs.js';

// ── parseTickBlob ────────────────────────────────────────────────

test('parseTickBlob accepts 4-token rows + case-insensitive side', () => {
    const r = parseTickBlob('0 100.05 50 BUY\n0 100.00 75 sell\n1 99.95 30 Uncertain');
    expect(r.errors).toEqual([]);
    expect(r.ticks.map(t => t.classified.side)).toEqual(['buy', 'sell', 'uncertain']);
    expect(r.ticks[0]).toEqual({
        bar_id: 0, price: 100.05,
        classified: { volume: 50, side: 'buy' },
    });
});

test('parseTickBlob rejects wrong token count', () => {
    expect(parseTickBlob('0 100 50').errors[0].message).toMatch(/expected 4 tokens/);
});

test('parseTickBlob rejects non-integer or negative bar_id', () => {
    expect(parseTickBlob('1.5 100 50 buy').errors[0].message).toMatch(/bar_id/);
    expect(parseTickBlob('-1 100 50 buy').errors[0].message).toMatch(/bar_id/);
});

test('parseTickBlob rejects non-positive price + non-positive volume', () => {
    expect(parseTickBlob('0 0 50 buy').errors[0].message).toMatch(/price/);
    expect(parseTickBlob('0 100 0 buy').errors[0].message).toMatch(/volume/);
});

test('parseTickBlob rejects bad side enum', () => {
    expect(parseTickBlob('0 100 50 long').errors[0].message).toMatch(/side must be/);
});

test('parseTickBlob comments + non-string safety', () => {
    const r = parseTickBlob('# header\n\n0 100 50 buy');
    expect(r.ticks.length).toBe(1);
    expect(parseTickBlob(null).errors.length).toBe(1);
});

// ── validateInputs / buildBody ───────────────────────────────────

test('validate accepts good inputs', () => {
    const ticks = [{ bar_id: 0, price: 100, classified: { volume: 50, side: 'buy' } }];
    expect(validateInputs(ticks, 0.05)).toBe(null);
});

test('validate rejects empty ticks + non-positive tick_size', () => {
    expect(validateInputs([], 0.05)).toMatch(/at least 1 tick/);
    expect(validateInputs([{}], 0)).toMatch(/tick_size/);
});

test('buildBody emits backend FootprintBody shape', () => {
    const t = [{ bar_id: 0, price: 100, classified: { volume: 50, side: 'buy' } }];
    expect(buildBody(t, 0.05)).toEqual({ ticks: t, tick_size: 0.05 });
});

// ── deltaCls ─────────────────────────────────────────────────────

test('deltaCls: positive → pos, negative → neg, zero/NaN → empty', () => {
    expect(deltaCls(100)).toBe('pos');
    expect(deltaCls(-100)).toBe('neg');
    expect(deltaCls(0)).toBe('');
    expect(deltaCls(NaN)).toBe('');
});

// ── summarize ────────────────────────────────────────────────────

const sampleReport = {
    tick_size: 0.05,
    bars: [
        {
            bar_id: 0,
            cells: [
                { price: 100.00, bid_volume: 50, ask_volume: 50, delta: 0 },
                { price: 100.05, bid_volume: 100, ask_volume: 150, delta: 50 },
            ],
            total_volume: 350,
            total_delta: 50,
            poc_price: 100.05,
        },
        {
            bar_id: 1,
            cells: [
                { price: 99.95, bid_volume: 200, ask_volume: 50, delta: -150 },
                { price: 100.00, bid_volume: 100, ask_volume: 100, delta: 0 },
            ],
            total_volume: 450,
            total_delta: -150,
            poc_price: 99.95,
        },
    ],
};

test('summarize aggregates total volume / delta / max-abs-delta / last POC', () => {
    const s = summarize(sampleReport);
    expect(s.barCount).toBe(2);
    expect(s.totalVolume).toBe(800);
    expect(s.totalDelta).toBe(-100);
    expect(s.maxAbsDelta).toBe(150);
    expect(s.lastPoc).toBe(99.95);
});

test('summarize null-report safe', () => {
    const s = summarize(null);
    expect(s.barCount).toBe(0);
    expect(s.lastPoc).toBe(null);
});

// ── imbalanceHotspots ────────────────────────────────────────────

test('imbalanceHotspots returns top-N by abs(delta)', () => {
    const hots = imbalanceHotspots(sampleReport, 2);
    expect(hots.length).toBe(2);
    // Largest abs = 150 (bar 1, 99.95) then 50 (bar 0, 100.05).
    expect(Math.abs(hots[0].delta)).toBeGreaterThanOrEqual(Math.abs(hots[1].delta));
    expect(hots[0].price).toBe(99.95);
});

test('imbalanceHotspots null-report safe', () => {
    expect(imbalanceHotspots(null)).toEqual([]);
});

test('imbalanceHotspots default top-N = 5', () => {
    const hots = imbalanceHotspots(sampleReport);
    expect(hots.length).toBe(4);   // sample has 4 cells total
});

// ── makeDemoTicks ────────────────────────────────────────────────

test('makeDemoTicks: 4 bars + every tick has valid shape', () => {
    const t = makeDemoTicks();
    const barIds = new Set(t.map(x => x.bar_id));
    expect([...barIds].sort()).toEqual([0, 1, 2, 3]);
    expect(t.every(x =>
        Number.isInteger(x.bar_id) && x.bar_id >= 0 &&
        Number.isFinite(x.price) && x.price > 0 &&
        Number.isFinite(x.classified.volume) && x.classified.volume > 0 &&
        ['buy', 'sell', 'uncertain'].includes(x.classified.side)
    )).toBe(true);
});

test('makeDemoTicks: bar 1 (absorption) has more buy volume at low than at top', () => {
    const t = makeDemoTicks().filter(x => x.bar_id === 1);
    const lowBuys = t.filter(x => x.price === 99.85 && x.classified.side === 'buy')
        .reduce((a, x) => a + x.classified.volume, 0);
    const topSells = t.filter(x => x.price === 100.00 && x.classified.side === 'sell')
        .reduce((a, x) => a + x.classified.volume, 0);
    expect(lowBuys).toBeGreaterThan(topSells);
});

test('makeDemoTicks: bar 2 (drive up) is dominated by buys', () => {
    const t = makeDemoTicks().filter(x => x.bar_id === 2);
    const buys = t.filter(x => x.classified.side === 'buy').reduce((a, x) => a + x.classified.volume, 0);
    const sells = t.filter(x => x.classified.side === 'sell').reduce((a, x) => a + x.classified.volume, 0);
    expect(buys).toBeGreaterThan(sells * 3);
});

test('makeDemoTicks: bar 3 (rejection at high) is dominated by sells', () => {
    const t = makeDemoTicks().filter(x => x.bar_id === 3);
    const buys = t.filter(x => x.classified.side === 'buy').reduce((a, x) => a + x.classified.volume, 0);
    const sells = t.filter(x => x.classified.side === 'sell').reduce((a, x) => a + x.classified.volume, 0);
    expect(sells).toBeGreaterThan(buys * 3);
});

// ── Formatters ───────────────────────────────────────────────────

test('fmtN default 0 decimals', () => {
    expect(fmtN(123.456)).toBe('123');
    expect(fmtN(NaN)).toBe('—');
});

test('fmtPrice picks decimals from tick_size magnitude', () => {
    expect(fmtPrice(100.5, 0.01)).toBe('100.50');
    expect(fmtPrice(100.5, 0.001)).toBe('100.500');
    expect(fmtPrice(100.5, 1)).toBe('101');     // log10(1)=0, no decimals
    expect(fmtPrice(NaN, 0.01)).toBe('—');
});

test('fmtSigned formats with + prefix', () => {
    expect(fmtSigned(50)).toBe('+50');
    expect(fmtSigned(-50)).toBe('-50');
    expect(fmtSigned(NaN)).toBe('—');
});
