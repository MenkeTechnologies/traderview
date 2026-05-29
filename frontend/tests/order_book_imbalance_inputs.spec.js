// Order Book Imbalance helpers: size parser, validator, body shape,
// level alignment, bias badge mapping, demo presets, formatters.

import { test, expect } from 'vitest';
import {
    parseSizes, validateInputs, buildBody,
    alignLevels, biasBadge, makeDemoBook,
    fmtN, fmtImbalance,
} from '../js/_order_book_imbalance_inputs.js';

// ── parseSizes (delegates to shared nonNegative parser) ───────────

test('parseSizes accepts non-negative + rejects negatives', () => {
    const r = parseSizes('100\n200\n-1\n300');
    expect(r.value).toEqual([100, 200, 300]);
    expect(r.errors.length).toBe(1);
    expect(r.errors[0].message).toMatch(/negative/);
});

// ── validateInputs ────────────────────────────────────────────────

test('validate accepts good inputs', () => {
    expect(validateInputs([100, 80], [90, 70], 2)).toBe(null);
});

test('validate rejects empty bid or ask', () => {
    expect(validateInputs([], [1], 1)).toMatch(/bid_sizes/);
    expect(validateInputs([1], [], 1)).toMatch(/ask_sizes/);
});

test('validate enforces integer levels in [1, 50]', () => {
    expect(validateInputs([1], [1], 0)).toMatch(/levels/);
    expect(validateInputs([1], [1], 51)).toMatch(/levels/);
    expect(validateInputs([1], [1], 1.5)).toMatch(/levels/);
});

test('validate rejects non-finite or negative size entries', () => {
    expect(validateInputs([1, NaN], [1], 1)).toMatch(/bid_sizes/);
    expect(validateInputs([1], [1, -1], 1)).toMatch(/ask_sizes/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody emits backend ObiBody shape', () => {
    expect(buildBody([100, 80], [90, 70], 2)).toEqual({
        bid_sizes: [100, 80], ask_sizes: [90, 70], levels: 2,
    });
});

// ── alignLevels ───────────────────────────────────────────────────

test('alignLevels pads shorter side with zeros', () => {
    const rows = alignLevels([100, 80, 60], [90], 5);
    expect(rows.length).toBe(3);
    expect(rows[1]).toEqual({ level: 2, bid: 80, ask: 0 });
    expect(rows[2]).toEqual({ level: 3, bid: 60, ask: 0 });
});

test('alignLevels caps row count at requested levels', () => {
    const rows = alignLevels([1, 2, 3, 4, 5], [1, 2, 3, 4, 5], 2);
    expect(rows.length).toBe(2);
});

test('alignLevels surfaces all side levels when levels > side length', () => {
    // levels=10 but only 3 entries either side → still returns 3 rows
    const rows = alignLevels([1, 2, 3], [4, 5, 6], 10);
    expect(rows.length).toBe(3);
});

test('alignLevels empty input returns empty', () => {
    expect(alignLevels([], [], 5)).toEqual([]);
});

// ── biasBadge ─────────────────────────────────────────────────────

test('biasBadge covers all 5 backend enum variants', () => {
    expect(biasBadge('strongly_bid').label).toBe('STRONGLY BID');
    expect(biasBadge('strongly_bid').cls).toBe('pos');
    expect(biasBadge('bid').cls).toBe('pos');
    expect(biasBadge('balanced').cls).toBe('');
    expect(biasBadge('ask').cls).toBe('neg');
    expect(biasBadge('strongly_ask').cls).toBe('neg');
});

test('biasBadge unknown bias falls through gracefully', () => {
    const b = biasBadge('garbage');
    expect(b.label).toBe('garbage');
    expect(b.cls).toBe('');
});

test('biasBadge null/undefined → em-dash', () => {
    expect(biasBadge(null).label).toBe('—');
    expect(biasBadge(undefined).label).toBe('—');
});

// ── makeDemoBook ──────────────────────────────────────────────────

test('makeDemoBook(balanced) has equal bid/ask totals', () => {
    const { bid_sizes, ask_sizes } = makeDemoBook('balanced');
    const bidSum = bid_sizes.reduce((a, b) => a + b, 0);
    const askSum = ask_sizes.reduce((a, b) => a + b, 0);
    expect(bidSum).toBe(askSum);
});

test('makeDemoBook(bid-pressure) has bid total > 2× ask total', () => {
    const { bid_sizes, ask_sizes } = makeDemoBook('bid-pressure');
    const bidSum = bid_sizes.reduce((a, b) => a + b, 0);
    const askSum = ask_sizes.reduce((a, b) => a + b, 0);
    expect(bidSum).toBeGreaterThan(askSum * 2);
});

test('makeDemoBook(ask-pressure) has ask total > 2× bid total', () => {
    const { bid_sizes, ask_sizes } = makeDemoBook('ask-pressure');
    const bidSum = bid_sizes.reduce((a, b) => a + b, 0);
    const askSum = ask_sizes.reduce((a, b) => a + b, 0);
    expect(askSum).toBeGreaterThan(bidSum * 2);
});

test('makeDemoBook unknown kind falls back to balanced', () => {
    const { bid_sizes, ask_sizes } = makeDemoBook('weird');
    const bidSum = bid_sizes.reduce((a, b) => a + b, 0);
    const askSum = ask_sizes.reduce((a, b) => a + b, 0);
    expect(bidSum).toBe(askSum);
});

test('makeDemoBook all sides return 10 levels', () => {
    for (const k of ['balanced', 'bid-pressure', 'ask-pressure']) {
        const { bid_sizes, ask_sizes } = makeDemoBook(k);
        expect(bid_sizes.length).toBe(10);
        expect(ask_sizes.length).toBe(10);
    }
});

// ── formatters ────────────────────────────────────────────────────

test('fmtN handles non-finite', () => {
    expect(fmtN(12345)).toBe('12,345');
    expect(fmtN(NaN)).toBe('—');
});

test('fmtImbalance signs positive + 4-decimal', () => {
    expect(fmtImbalance(0.4567)).toBe('+0.4567');
    expect(fmtImbalance(-0.123)).toBe('-0.1230');
    expect(fmtImbalance(NaN)).toBe('—');
});
