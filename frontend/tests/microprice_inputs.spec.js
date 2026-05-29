// Microprice (Stoikov) pure helpers: payload shape, validator,
// local closed-form, imbalance sweep, formatters.

import { test, expect } from 'vitest';
import {
    buildBody, validateQuote, microprice,
    imbalanceSweep, fmtPrice, fmtBps, fmtImbalance,
} from '../js/_microprice_inputs.js';

const baseQuote = { bid: 100.00, ask: 100.05, bid_size: 1500, ask_size: 400 };

// ── buildBody ──────────────────────────────────────────────────────

test('buildBody wraps single quote into 1-element array', () => {
    const b = buildBody(baseQuote);
    expect(Array.isArray(b.quotes)).toBe(true);
    expect(b.quotes.length).toBe(1);
    expect(b.quotes[0]).toEqual(baseQuote);
});

// ── validateQuote ──────────────────────────────────────────────────

test('validate rejects non-positive bid/ask', () => {
    expect(validateQuote({ ...baseQuote, bid: 0 })).toMatch(/bid/);
    expect(validateQuote({ ...baseQuote, ask: -1 })).toMatch(/ask/);
});

test('validate rejects crossed market', () => {
    expect(validateQuote({ ...baseQuote, bid: 101, ask: 100 })).toMatch(/bid must be ≤ ask/);
});

test('validate rejects negative sizes', () => {
    expect(validateQuote({ ...baseQuote, bid_size: -1 })).toMatch(/bid_size/);
    expect(validateQuote({ ...baseQuote, ask_size: -1 })).toMatch(/ask_size/);
});

test('validate rejects zero total size', () => {
    expect(validateQuote({ ...baseQuote, bid_size: 0, ask_size: 0 })).toMatch(/at least one side/);
});

test('validate accepts zero size on ONE side', () => {
    // A quote with all liquidity on one side is unusual but legal.
    expect(validateQuote({ ...baseQuote, ask_size: 0 })).toBe(null);
});

test('validate rejects non-finite inputs', () => {
    expect(validateQuote({ ...baseQuote, bid: NaN })).toMatch(/bid/);
    expect(validateQuote({ ...baseQuote, bid_size: Infinity })).toMatch(/bid_size/);
});

test('validate accepts a good default quote', () => {
    expect(validateQuote(baseQuote)).toBe(null);
});

// ── microprice (local closed-form) ─────────────────────────────────

test('microprice with balanced sizes returns midpoint', () => {
    const mp = microprice(100, 101, 1000, 1000);
    expect(mp).toBeCloseTo(100.5, 12);
});

test('microprice with bid-heavy queue biases toward ask', () => {
    // bid_size much > ask_size → next print likely lifts ask.
    const mp = microprice(100, 101, 9000, 1000);
    expect(mp).toBeGreaterThan(100.5);
});

test('microprice with ask-heavy queue biases toward bid', () => {
    const mp = microprice(100, 101, 1000, 9000);
    expect(mp).toBeLessThan(100.5);
});

test('microprice at imbalance=0 returns bid', () => {
    const mp = microprice(100, 101, 0, 1000);
    expect(mp).toBeCloseTo(100, 12);
});

test('microprice at imbalance=1 returns ask', () => {
    const mp = microprice(100, 101, 1000, 0);
    expect(mp).toBeCloseTo(101, 12);
});

test('microprice with zero total returns null', () => {
    expect(microprice(100, 101, 0, 0)).toBe(null);
});

// ── imbalanceSweep ─────────────────────────────────────────────────

test('imbalanceSweep returns parallel xs and ys', () => {
    const { xs, ys } = imbalanceSweep(100, 101, 11);
    expect(xs.length).toBe(11);
    expect(ys.length).toBe(11);
});

test('imbalanceSweep covers 0 to 1', () => {
    const { xs } = imbalanceSweep(100, 101, 5);
    expect(xs[0]).toBeCloseTo(0, 12);
    expect(xs[xs.length - 1]).toBeCloseTo(1, 12);
});

test('imbalanceSweep ys: y[0] = bid, y[last] = ask', () => {
    const { ys } = imbalanceSweep(100, 101, 5);
    expect(ys[0]).toBeCloseTo(100, 12);
    expect(ys[ys.length - 1]).toBeCloseTo(101, 12);
});

test('imbalanceSweep is monotone increasing when ask > bid', () => {
    const { ys } = imbalanceSweep(100, 101, 21);
    for (let i = 1; i < ys.length; i++) {
        expect(ys[i]).toBeGreaterThanOrEqual(ys[i - 1]);
    }
});

test('imbalanceSweep returns empty arrays when bid ≥ ask', () => {
    expect(imbalanceSweep(101, 100).xs).toEqual([]);
    expect(imbalanceSweep(100, 100).xs).toEqual([]);
});

// ── formatters ─────────────────────────────────────────────────────

test('fmtPrice emits 4 decimals by default', () => {
    expect(fmtPrice(100.123456)).toBe('100.1235');
});

test('fmtPrice returns "—" on non-finite', () => {
    expect(fmtPrice(NaN)).toBe('—');
    expect(fmtPrice(null)).toBe('—');
});

test('fmtBps emits sign and "bps" suffix', () => {
    expect(fmtBps(2.5)).toBe('+2.50 bps');
    expect(fmtBps(-2.5)).toBe('−2.50 bps');
    expect(fmtBps(0)).toBe('0.00 bps');
});

test('fmtImbalance returns 2-decimal percent', () => {
    expect(fmtImbalance(0.789)).toBe('78.90%');
});

test('fmtImbalance returns "—" on non-finite', () => {
    expect(fmtImbalance(NaN)).toBe('—');
});
