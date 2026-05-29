// Per-Symbol Slippage helpers: parser, validator, body shape, grading,
// worst/best lookups, demo invariants, formatters.

import { test, expect } from 'vitest';
import {
    parseRecordBlob, validateInputs, buildBody,
    executionGrade, worstSymbol, bestSymbol,
    makeDemoRecords, fmtBps, fmtPct, fmtN,
} from '../js/_per_symbol_slippage_inputs.js';

// ── parseRecordBlob ────────────────────────────────────────────────

test('parseRecordBlob uppercases symbols + accepts comma + comments', () => {
    const r = parseRecordBlob('# header\naapl -2.5\nspy, 7\nilqd -28');
    expect(r.errors).toEqual([]);
    expect(r.records).toEqual([
        { symbol: 'AAPL', slippage_bps: -2.5 },
        { symbol: 'SPY',  slippage_bps: 7 },
        { symbol: 'ILQD', slippage_bps: -28 },
    ]);
});

test('parseRecordBlob rejects wrong token count', () => {
    expect(parseRecordBlob('AAPL').errors[0].message).toMatch(/expected 2 tokens/);
});

test('parseRecordBlob rejects malformed symbol', () => {
    expect(parseRecordBlob('A!P -2.5').errors[0].message).toMatch(/bad symbol/);
});

test('parseRecordBlob rejects non-finite bps', () => {
    expect(parseRecordBlob('AAPL abc').errors[0].message).toMatch(/slippage_bps/);
});

test('parseRecordBlob accepts positive AND negative bps', () => {
    const r = parseRecordBlob('AAPL 5\nMSFT -5');
    expect(r.errors).toEqual([]);
    expect(r.records.map(x => x.slippage_bps)).toEqual([5, -5]);
});

test('parseRecordBlob non-string returns 1 error', () => {
    expect(parseRecordBlob(null).errors.length).toBe(1);
});

// ── validateInputs / buildBody ────────────────────────────────────

test('validate accepts ≥1 record, rejects empty', () => {
    expect(validateInputs([{ symbol: 'A', slippage_bps: 1 }])).toBe(null);
    expect(validateInputs([])).toMatch(/at least 1 record/);
});

test('buildBody emits backend PerSymbolSlippageBody shape', () => {
    const recs = [{ symbol: 'A', slippage_bps: 1 }];
    expect(buildBody(recs)).toEqual({ records: recs });
});

// ── executionGrade ────────────────────────────────────────────────

test('executionGrade buckets at -15 / -5 / 0 / +5', () => {
    expect(executionGrade(-20).label).toBe('TERRIBLE');
    expect(executionGrade(-20).cls).toBe('neg');
    expect(executionGrade(-10).label).toBe('POOR');
    expect(executionGrade(-2).label).toBe('NEUTRAL');
    expect(executionGrade(2).label).toBe('GOOD');
    expect(executionGrade(2).cls).toBe('pos');
    expect(executionGrade(10).label).toBe('EXCELLENT');
});

test('executionGrade boundary points (-5, -15, 0, 5)', () => {
    expect(executionGrade(-15).label).toBe('TERRIBLE');
    expect(executionGrade(-5).label).toBe('POOR');
    expect(executionGrade(0).label).toBe('NEUTRAL');
    expect(executionGrade(5).label).toBe('GOOD');
});

test('executionGrade non-finite → em-dash', () => {
    expect(executionGrade(NaN).label).toBe('—');
});

// ── worstSymbol / bestSymbol ──────────────────────────────────────

test('worstSymbol returns lowest mean_bps', () => {
    const report = [
        { symbol: 'A', mean_bps: 5 },
        { symbol: 'B', mean_bps: -10 },
        { symbol: 'C', mean_bps: -3 },
    ];
    expect(worstSymbol(report).symbol).toBe('B');
    expect(bestSymbol(report).symbol).toBe('A');
});

test('worstSymbol / bestSymbol return null on empty / non-array', () => {
    expect(worstSymbol([])).toBe(null);
    expect(worstSymbol(null)).toBe(null);
    expect(bestSymbol([])).toBe(null);
});

test('worstSymbol skips non-finite mean_bps when seeking minimum', () => {
    const report = [
        { symbol: 'A', mean_bps: 0 },
        { symbol: 'B', mean_bps: NaN },
        { symbol: 'C', mean_bps: -3 },
    ];
    expect(worstSymbol(report).symbol).toBe('C');
});

// ── makeDemoRecords ───────────────────────────────────────────────

test('makeDemoRecords deterministic + exactly 108 records spanning 6 symbols', () => {
    const a = makeDemoRecords(42);
    const b = makeDemoRecords(42);
    expect(a).toEqual(b);
    expect(a.length).toBe(108);
    const symbols = new Set(a.map(r => r.symbol));
    expect(symbols.size).toBe(6);
});

test('makeDemoRecords SPY mean is clearly positive (EXCELLENT) and ILQD clearly negative (TERRIBLE)', () => {
    for (const seed of [1, 7, 42, 1337]) {
        const recs = makeDemoRecords(seed);
        const spy  = recs.filter(r => r.symbol === 'SPY').map(r => r.slippage_bps);
        const ilqd = recs.filter(r => r.symbol === 'ILQD').map(r => r.slippage_bps);
        const spyMean  = spy.reduce((a, b) => a + b, 0) / spy.length;
        const ilqdMean = ilqd.reduce((a, b) => a + b, 0) / ilqd.length;
        expect(spyMean).toBeGreaterThan(0);
        expect(ilqdMean).toBeLessThan(-15);
    }
});

// ── formatters ────────────────────────────────────────────────────

test('fmtBps signs positive + 1-decimal', () => {
    expect(fmtBps(12.34)).toBe('+12.3 bps');
    expect(fmtBps(-5)).toBe('-5.0 bps');
    expect(fmtBps(NaN)).toBe('—');
});

test('fmtPct emits 0-decimal percent', () => {
    expect(fmtPct(0.654)).toBe('65%');
    expect(fmtPct(NaN)).toBe('—');
});

test('fmtN locale-formats integers', () => {
    expect(fmtN(1234)).toBe('1,234');
    expect(fmtN(NaN)).toBe('—');
});
