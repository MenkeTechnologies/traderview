// Spread Tracker pure helpers: quote parser, validator, body shape,
// per-sample series computation, regime classifier, demo invariants,
// formatters.

import { test, expect } from 'vitest';
import {
    parseQuoteBlob, validateInputs, buildBody,
    computeSpreadSeries, classifyBps, makeDemoQuotes,
    REGIME_THRESHOLDS, REGIME_LABELS, REGIME_CSS,
    fmtBps, fmtN, fmtPct,
} from '../js/_spread_tracker_inputs.js';

// ── parseQuoteBlob ─────────────────────────────────────────────────

test('parseQuoteBlob handles whitespace + commas + comments', () => {
    const r = parseQuoteBlob('# header\n100.04 100.05\n100.05, 100.06');
    expect(r.errors).toEqual([]);
    expect(r.samples).toEqual([
        { bid: 100.04, ask: 100.05 },
        { bid: 100.05, ask: 100.06 },
    ]);
});

test('parseQuoteBlob rejects wrong token count', () => {
    expect(parseQuoteBlob('100.04').errors[0].message).toMatch(/expected 2 tokens/);
});

test('parseQuoteBlob rejects non-positive bid', () => {
    const r = parseQuoteBlob('0 0.01\n-1 1');
    expect(r.samples).toEqual([]);
    expect(r.errors.length).toBe(2);
});

test('parseQuoteBlob rejects ask < bid', () => {
    const r = parseQuoteBlob('100 99');
    expect(r.samples).toEqual([]);
    expect(r.errors[0].message).toMatch(/ask must be ≥ bid/);
});

test('parseQuoteBlob accepts ask == bid (zero-spread / crossed-mid edge)', () => {
    const r = parseQuoteBlob('100 100');
    expect(r.errors).toEqual([]);
    expect(r.samples).toEqual([{ bid: 100, ask: 100 }]);
});

test('parseQuoteBlob rejects non-finite tokens', () => {
    const r = parseQuoteBlob('abc 100\n100 def');
    expect(r.samples).toEqual([]);
    expect(r.errors.length).toBe(2);
});

test('parseQuoteBlob returns error on non-string input', () => {
    const r = parseQuoteBlob(42);
    expect(r.samples).toEqual([]);
    expect(r.errors.length).toBe(1);
});

// ── validateInputs ────────────────────────────────────────────────

test('validate accepts ≥5 samples', () => {
    expect(validateInputs(Array(5).fill({ bid: 100, ask: 100.01 }))).toBe(null);
});

test('validate rejects < 5 samples', () => {
    expect(validateInputs(Array(3).fill({ bid: 100, ask: 100.01 })))
        .toMatch(/at least 5 quote samples/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody emits backend SpreadTrackerBody shape', () => {
    const samples = [{ bid: 100, ask: 100.01 }];
    expect(buildBody(samples)).toEqual({ samples });
});

// ── computeSpreadSeries (parity with backend formula) ─────────────

test('computeSpreadSeries mirrors backend formula', () => {
    const samples = [
        { bid: 100, ask: 100.01 },    // mid 100.005, spread 0.01 → 0.9995 bps
        { bid: 100, ask: 101 },       // mid 100.5,   spread 1     → 99.50 bps
    ];
    const { bps, mids } = computeSpreadSeries(samples);
    expect(bps[0]).toBeCloseTo(0.999950, 4);
    expect(bps[1]).toBeCloseTo(99.502487, 4);
    expect(mids[0]).toBeCloseTo(100.005, 6);
    expect(mids[1]).toBeCloseTo(100.5, 6);
});

test('computeSpreadSeries emits null on invalid samples', () => {
    const { bps, mids } = computeSpreadSeries([
        { bid: 0, ask: 1 },
        { bid: 100, ask: 99 },
        { bid: NaN, ask: 100 },
    ]);
    expect(bps).toEqual([null, null, null]);
    expect(mids).toEqual([null, null, null]);
});

// ── classifyBps (matches backend tight/normal/wide/pathological cuts) ─

test('classifyBps boundaries', () => {
    expect(classifyBps(5)).toBe('tight');
    expect(classifyBps(5.0001)).toBe('normal');
    expect(classifyBps(25)).toBe('normal');
    expect(classifyBps(25.0001)).toBe('wide');
    expect(classifyBps(100)).toBe('wide');
    expect(classifyBps(100.0001)).toBe('pathological');
});

test('classifyBps NaN defaults to normal', () => {
    expect(classifyBps(NaN)).toBe('normal');
});

// ── label / css maps ──────────────────────────────────────────────

test('REGIME_LABELS + REGIME_CSS have all four regimes', () => {
    expect(Object.keys(REGIME_LABELS).sort()).toEqual(['normal', 'pathological', 'tight', 'wide']);
    expect(Object.keys(REGIME_CSS).sort()).toEqual(['normal', 'pathological', 'tight', 'wide']);
    expect(REGIME_CSS.tight).toBe('pos');
    expect(REGIME_CSS.pathological).toBe('neg');
});

test('REGIME_THRESHOLDS sorted ascending', () => {
    expect(REGIME_THRESHOLDS.tight).toBeLessThan(REGIME_THRESHOLDS.normal);
    expect(REGIME_THRESHOLDS.normal).toBeLessThan(REGIME_THRESHOLDS.wide);
});

// ── makeDemoQuotes ────────────────────────────────────────────────

test('makeDemoQuotes deterministic for fixed seed', () => {
    expect(makeDemoQuotes(42)).toEqual(makeDemoQuotes(42));
});

test('makeDemoQuotes emits exactly 300 quotes', () => {
    expect(makeDemoQuotes(1).length).toBe(300);
});

test('makeDemoQuotes all samples have ask > bid', () => {
    const q = makeDemoQuotes(7);
    expect(q.every(s => s.ask > s.bid && s.bid > 0)).toBe(true);
});

test('makeDemoQuotes pathological burst raises max spread above 100 bps', () => {
    const q = makeDemoQuotes(1);
    const burst = q.slice(250, 270);
    const baseline = q.slice(0, 200);
    const burstMaxBps = Math.max(...burst.map(s => (s.ask - s.bid) / ((s.bid + s.ask) / 2) * 10_000));
    const baselineMaxBps = Math.max(...baseline.map(s => (s.ask - s.bid) / ((s.bid + s.ask) / 2) * 10_000));
    expect(burstMaxBps).toBeGreaterThan(100);
    expect(baselineMaxBps).toBeLessThan(20);
});

// ── formatters ─────────────────────────────────────────────────────

test('fmtBps emits 1-decimal with bps suffix', () => {
    expect(fmtBps(12.345)).toBe('12.3 bps');
    expect(fmtBps(NaN)).toBe('—');
});

test('fmtN respects digit override', () => {
    expect(fmtN(1.23456, 2)).toBe('1.23');
    expect(fmtN(NaN)).toBe('—');
});

test('fmtPct emits 1-decimal percentage', () => {
    expect(fmtPct(0.064)).toBe('6.4%');
    expect(fmtPct(NaN)).toBe('—');
});
