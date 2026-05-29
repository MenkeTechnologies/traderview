// VPIN helpers: tick parser, validator, body shape, series extractor,
// summarize, demo-data generator, formatters.

import { test, expect } from 'vitest';
import {
    parseTickBlob, validateInputs, buildBody,
    extractFinishedVpin, summarize, makeDemoTicks,
    fmtN, fmtPct,
} from '../js/_vpin_inputs.js';

// ── parseTickBlob ──────────────────────────────────────────────────

test('parseTickBlob accepts whitespace and comma separators', () => {
    const r = parseTickBlob('100.05 250\n100.06, 1200\n100.04\t500');
    expect(r.errors).toEqual([]);
    expect(r.ticks).toEqual([
        { price: 100.05, volume: 250 },
        { price: 100.06, volume: 1200 },
        { price: 100.04, volume: 500 },
    ]);
});

test('parseTickBlob skips comments and blanks', () => {
    const r = parseTickBlob('# header\n\n100 10\n# inline\n101 20');
    expect(r.errors).toEqual([]);
    expect(r.ticks.length).toBe(2);
});

test('parseTickBlob rejects lines with wrong token count', () => {
    const r = parseTickBlob('100 10\n101\n102 20 30');
    expect(r.ticks.length).toBe(1);
    expect(r.errors.length).toBe(2);
    expect(r.errors[0].message).toMatch(/expected 2 tokens/);
});

test('parseTickBlob rejects non-positive price and negative volume', () => {
    const r = parseTickBlob('0 10\n-1 5\n100 -10\n100 abc');
    expect(r.ticks).toEqual([]);
    expect(r.errors.length).toBe(4);
});

test('parseTickBlob returns error on non-string input', () => {
    const r = parseTickBlob(null);
    expect(r.ticks).toEqual([]);
    expect(r.errors.length).toBe(1);
});

// ── validateInputs ─────────────────────────────────────────────────

const okConfig = { volume_per_bucket: 1000, window_buckets: 5, return_window: 10 };

test('validate accepts ≥10 ticks + sensible config', () => {
    const ticks = Array.from({ length: 20 }, (_, i) => ({ price: 100 + i * 0.01, volume: 500 }));
    expect(validateInputs(ticks, okConfig)).toBe(null);
});

test('validate rejects < 10 ticks', () => {
    expect(validateInputs(Array(5).fill({ price: 100, volume: 100 }), okConfig))
        .toMatch(/10 ticks/);
});

test('validate rejects bad config scalars', () => {
    const ticks = Array(20).fill({ price: 100, volume: 500 });
    expect(validateInputs(ticks, { ...okConfig, volume_per_bucket: 0 })).toMatch(/volume_per_bucket/);
    expect(validateInputs(ticks, { ...okConfig, window_buckets: 0 })).toMatch(/window_buckets/);
    expect(validateInputs(ticks, { ...okConfig, window_buckets: 2001 })).toMatch(/window_buckets/);
    expect(validateInputs(ticks, { ...okConfig, return_window: 1 })).toMatch(/return_window/);
});

test('validate catches volume_per_bucket too large for total volume', () => {
    const ticks = Array(20).fill({ price: 100, volume: 10 });
    expect(validateInputs(ticks, { ...okConfig, volume_per_bucket: 1e6 }))
        .toMatch(/volume_per_bucket too large/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody emits backend VpinBody shape', () => {
    const ticks = [{ price: 100, volume: 10 }];
    const body = buildBody(ticks, okConfig);
    expect(body).toEqual({ ticks, config: okConfig });
});

// ── extractFinishedVpin ──────────────────────────────────────────

test('extractFinishedVpin drops null warmup entries and indexes the rest', () => {
    const report = { vpin: [null, null, null, 0.25, 0.40, 0.60, null, 0.55] };
    const { xs, ys } = extractFinishedVpin(report);
    expect(xs).toEqual([3, 4, 5, 7]);
    expect(ys).toEqual([0.25, 0.40, 0.60, 0.55]);
});

test('extractFinishedVpin returns empty arrays on missing report', () => {
    const { xs, ys } = extractFinishedVpin(null);
    expect(xs).toEqual([]);
    expect(ys).toEqual([]);
});

// ── summarize ─────────────────────────────────────────────────────

test('summarize computes nBuckets, max, avg, toxic count + pct, buy/sell skew', () => {
    const report = {
        vpin: [null, null, 0.20, 0.30, 0.60, 0.70],
        toxic_buckets: [4, 5],
        bucket_buy_volume:  [1, 2, 3, 4, 5, 6],
        bucket_sell_volume: [6, 5, 4, 3, 2, 1],
    };
    const s = summarize(report);
    expect(s.nBuckets).toBe(6);
    expect(s.maxVpin).toBeCloseTo(0.70, 6);
    expect(s.avgVpin).toBeCloseTo((0.2 + 0.3 + 0.6 + 0.7) / 4, 6);
    expect(s.toxicCount).toBe(2);
    expect(s.toxicPct).toBeCloseTo(2 / 6, 6);
    expect(s.totalBuy).toBe(21);
    expect(s.totalSell).toBe(21);
    expect(s.buySellSkew).toBe(0);
});

test('summarize returns null on null report', () => {
    expect(summarize(null)).toBe(null);
});

// ── makeDemoTicks ─────────────────────────────────────────────────

test('makeDemoTicks is deterministic for fixed seed (regression-safe)', () => {
    const a = makeDemoTicks(50, 42);
    const b = makeDemoTicks(50, 42);
    expect(a).toEqual(b);
});

test('makeDemoTicks emits valid (price, volume) shape', () => {
    const ticks = makeDemoTicks(100, 1);
    expect(ticks.length).toBe(100);
    expect(ticks.every(t => Number.isFinite(t.price) && t.price > 0
        && Number.isInteger(t.volume) && t.volume >= 0)).toBe(true);
});

test('makeDemoTicks toxic regime has higher avg volume than benign regime', () => {
    const ticks = makeDemoTicks(1000, 7);
    const benign = ticks.slice(0, 750);
    const toxic  = ticks.slice(750);
    const avgB = benign.reduce((a, t) => a + t.volume, 0) / benign.length;
    const avgT = toxic.reduce((a, t) => a + t.volume, 0) / toxic.length;
    expect(avgT).toBeGreaterThan(avgB);
});

// ── formatters ─────────────────────────────────────────────────────

test('fmtN handles non-finite', () => {
    expect(fmtN(NaN)).toBe('—');
    expect(fmtN(0.12345)).toBe('0.123');
});

test('fmtPct emits 1-decimal percentage', () => {
    expect(fmtPct(0.234)).toBe('23.4%');
    expect(fmtPct(NaN)).toBe('—');
});
