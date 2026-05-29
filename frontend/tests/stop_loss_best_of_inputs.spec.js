// Stop-loss best-of helpers: parser, validator, body shape, method
// badges, candidate descriptions, best-by-* lookups, defaults, demo,
// formatters.

import { test, expect } from 'vitest';
import {
    parseTradeBlob, validateInputs, buildBody,
    VALID_METHODS, methodBadge, describeCandidate,
    bestByTotal, bestByAvg, defaultCandidates,
    makeDemoTrades, fmtN, fmtSigned,
} from '../js/_stop_loss_best_of_inputs.js';

// ── parseTradeBlob ────────────────────────────────────────────────

test('parseTradeBlob accepts 4-token trades', () => {
    const r = parseTradeBlob('100 0.8 2.5 101.7\n100.5 2.1 0.5 99.2');
    expect(r.errors).toEqual([]);
    expect(r.trades.length).toBe(2);
});

test('parseTradeBlob rejects wrong token count', () => {
    expect(parseTradeBlob('100 0.5').errors[0].message).toMatch(/expected 4 tokens/);
});

test('parseTradeBlob rejects bad entry / negative excursions / non-positive exit', () => {
    expect(parseTradeBlob('0 0.5 0.5 100').errors[0].message).toMatch(/entry/);
    expect(parseTradeBlob('100 -0.1 0.5 100').errors[0].message).toMatch(/mae/);
    expect(parseTradeBlob('100 0.5 -0.1 100').errors[0].message).toMatch(/mfe/);
    expect(parseTradeBlob('100 0.5 0.5 0').errors[0].message).toMatch(/actual_exit/);
});

test('parseTradeBlob accepts zero MAE / MFE', () => {
    const r = parseTradeBlob('100 0 0 101');
    expect(r.errors).toEqual([]);
});

// ── validateInputs ────────────────────────────────────────────────

const okT = [{ entry: 100, mae: 0.5, mfe: 1.0, actual_exit: 101 }];
const okC = [{ method: 'fixed_dollar', value: 1, atr: 0 }];

test('validate accepts good inputs', () => {
    expect(validateInputs(okT, okC, true)).toBe(null);
});

test('validate rejects empty trades / candidates', () => {
    expect(validateInputs([], okC, true)).toMatch(/at least 1 trade/);
    expect(validateInputs(okT, [], true)).toMatch(/at least 1 candidate/);
});

test('validate rejects bad candidate method / non-numeric value', () => {
    expect(validateInputs(okT, [{ method: 'martingale', value: 1, atr: 0 }], true))
        .toMatch(/bad candidate/);
    expect(validateInputs(okT, [{ method: 'fixed_dollar', value: NaN, atr: 0 }], true))
        .toMatch(/value/);
});

test('validate rejects non-boolean side_long', () => {
    expect(validateInputs(okT, okC, 'long')).toMatch(/boolean/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody emits backend StopBestOfBody shape', () => {
    expect(buildBody(okT, okC, true)).toEqual({
        trades: okT, candidates: okC, side_long: true,
    });
});

// ── VALID_METHODS + methodBadge ──────────────────────────────────

test('VALID_METHODS lists exactly the 4 backend enums', () => {
    expect([...VALID_METHODS].sort()).toEqual(['atr_multiple', 'fixed_dollar', 'fixed_pct', 'none']);
});

test('methodBadge covers all enums + unknown fallthrough', () => {
    expect(methodBadge('none').cls).toBe('neg');
    expect(methodBadge('atr_multiple').cls).toBe('pos');
    expect(methodBadge('garbage').label).toBe('garbage');
});

// ── describeCandidate ────────────────────────────────────────────

test('describeCandidate per-method human strings', () => {
    expect(describeCandidate({ method: 'none', value: 0, atr: 0 })).toMatch(/No stop/);
    expect(describeCandidate({ method: 'fixed_dollar', value: 2.5, atr: 0 })).toMatch(/\$2\.50/);
    expect(describeCandidate({ method: 'fixed_pct', value: 0.015, atr: 0 })).toMatch(/1\.50%/);
    expect(describeCandidate({ method: 'atr_multiple', value: 2, atr: 1.5 })).toMatch(/2\.00 × ATR\(1\.50\)/);
    expect(describeCandidate(null)).toBe('—');
});

// ── bestByTotal / bestByAvg ──────────────────────────────────────

test('bestByTotal picks highest total_realized', () => {
    const results = [
        { method: 'fixed_pct', value: 0.01, total_realized: 50, avg_realized: 2 },
        { method: 'none',      value: 0,    total_realized: 100, avg_realized: 5 },
    ];
    expect(bestByTotal(results).method).toBe('none');
});

test('bestByAvg picks highest avg_realized', () => {
    const results = [
        { method: 'a', value: 1, total_realized: 100, avg_realized: 2 },
        { method: 'b', value: 1, total_realized: 50, avg_realized: 5 },
    ];
    expect(bestByAvg(results).method).toBe('b');
});

test('bestByTotal / bestByAvg null on empty', () => {
    expect(bestByTotal([])).toBe(null);
    expect(bestByAvg(null)).toBe(null);
});

// ── defaultCandidates ────────────────────────────────────────────

test('defaultCandidates covers all 4 methods + varied values', () => {
    const cs = defaultCandidates(1.0);
    expect(cs.length).toBe(9);
    expect(new Set(cs.map(c => c.method)).size).toBe(4);
});

// ── Demo ─────────────────────────────────────────────────────────

test('makeDemoTrades deterministic + exactly 20 trades', () => {
    const a = makeDemoTrades(42);
    const b = makeDemoTrades(42);
    expect(a).toEqual(b);
    expect(a.length).toBe(20);
});

test('makeDemoTrades has alternating winner/loser pattern', () => {
    const t = makeDemoTrades(1);
    expect(t[0].actual_exit).toBeGreaterThan(t[0].entry);   // winner
    expect(t[1].actual_exit).toBeLessThan(t[1].entry);      // loser
});

// ── Formatters ───────────────────────────────────────────────────

test('fmtN + fmtSigned', () => {
    expect(fmtN(1.234)).toBe('1.23');
    expect(fmtN(NaN)).toBe('—');
    expect(fmtSigned(2.5)).toBe('+2.50');
    expect(fmtSigned(-1.5)).toBe('-1.50');
    expect(fmtSigned(NaN)).toBe('—');
});
