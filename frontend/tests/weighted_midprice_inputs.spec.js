// Weighted-midprice helpers: parser, validator, body shape,
// localCompute Rust-mirror, summarize, badges, demos.

import { test, expect } from 'vitest';
import {
    parseQuotesBlob, quotesToBlob, validateInputs, buildBody,
    localCompute, localSeries, summarize, imbalanceBadge,
    makeDemoInput,
    fmtUSD, fmtUSDSigned, fmtBps, fmtImb, fmtNum, fmtInt,
} from '../js/_weighted_midprice_inputs.js';

const q = (bp, bs, ap, asz) => ({ bid_price: bp, bid_size: bs, ask_price: ap, ask_size: asz });

// ── parser ────────────────────────────────────────────────────────

test('parseQuotesBlob: 4 tokens per line; blanks + comments ignored', () => {
    const r = parseQuotesBlob('100.00 100 100.10 100\n# tick 2\n100.05, 200, 100.15, 50');
    expect(r.errors).toEqual([]);
    expect(r.quotes).toEqual([q(100, 100, 100.10, 100), q(100.05, 200, 100.15, 50)]);
});

test('parseQuotesBlob: rejects wrong token count + non-finite', () => {
    expect(parseQuotesBlob('100 100 100.10').errors[0].message).toMatch(/4 tokens/);
    expect(parseQuotesBlob('foo 100 100.10 100').errors[0].message).toMatch(/non-finite/);
});

test('parseQuotesBlob: non-string returns 1 error', () => {
    expect(parseQuotesBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts non-empty quotes', () => {
    expect(validateInputs({ quotes: [q(100, 100, 100.10, 100)] })).toBe(null);
});

test('validate rejects: bad array / empty / bad field', () => {
    expect(validateInputs({ quotes: 'no' })).toMatch(/quotes/);
    expect(validateInputs({ quotes: [] })).toMatch(/non-empty/);
    expect(validateInputs({ quotes: [{ bid_price: NaN, bid_size: 100, ask_price: 100.10, ask_size: 100 }] }))
        .toMatch(/bid_price/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: emits plain-shape quotes (strips extras)', () => {
    const body = buildBody({ quotes: [{ ...q(100, 100, 100.10, 100), extra: 'x' }] });
    expect(body.quotes[0]).toEqual(q(100, 100, 100.10, 100));
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: bad single fields → null', () => {
    for (const bad of [0, -1, NaN]) {
        expect(localCompute(q(bad, 100, 100.10, 100))).toBeNull();
        expect(localCompute(q(100, bad, 100.10, 100))).toBeNull();
        expect(localCompute(q(100, 100, bad,    100))).toBeNull();
        expect(localCompute(q(100, 100, 100.10, bad))).toBeNull();
    }
});

test('local: crossed book (bid > ask) → null', () => {
    expect(localCompute(q(100.10, 100, 100.00, 100))).toBeNull();
});

test('local: null input → null', () => {
    expect(localCompute(null)).toBeNull();
});

test('local: balanced book → micro = midpoint, imbalance = 0', () => {
    const r = localCompute(q(100, 100, 100.10, 100));
    expect(r.microprice).toBeCloseTo(r.midpoint, 12);
    expect(Math.abs(r.quote_imbalance)).toBeLessThan(1e-12);
});

test('local: larger bid size pushes microprice toward ask', () => {
    const r = localCompute(q(100, 1000, 100.10, 100));
    expect(r.microprice).toBeGreaterThan(r.midpoint);
    expect(r.quote_imbalance).toBeGreaterThan(0);
});

test('local: larger ask size pushes microprice toward bid', () => {
    const r = localCompute(q(100, 100, 100.10, 1000));
    expect(r.microprice).toBeLessThan(r.midpoint);
    expect(r.quote_imbalance).toBeLessThan(0);
});

test('local: extreme bid size caps microprice at ask', () => {
    const r = localCompute(q(100, 1e9, 100.10, 1));
    expect(Math.abs(r.microprice - 100.10)).toBeLessThan(1e-6);
});

test('local: extreme ask size caps microprice at bid', () => {
    const r = localCompute(q(100, 1, 100.10, 1e9));
    expect(Math.abs(r.microprice - 100.00)).toBeLessThan(1e-6);
});

test('local: imbalance bounded in [-1, +1]', () => {
    const a = localCompute(q(100, 1e9, 100.10, 1));
    const b = localCompute(q(100, 1, 100.10, 1e9));
    expect(Math.abs(a.quote_imbalance - 1)).toBeLessThan(1e-6);
    expect(Math.abs(b.quote_imbalance + 1)).toBeLessThan(1e-6);
    expect(a.quote_imbalance).toBeGreaterThanOrEqual(-1);
    expect(a.quote_imbalance).toBeLessThanOrEqual(1);
    expect(b.quote_imbalance).toBeGreaterThanOrEqual(-1);
    expect(b.quote_imbalance).toBeLessThanOrEqual(1);
});

test('local: relative_spread = spread / midpoint', () => {
    const r = localCompute(q(100, 100, 100.10, 100));
    expect(r.relative_spread).toBeCloseTo(0.10 / 100.05, 9);
});

test('local: spread + microprice_minus_midpoint computed correctly', () => {
    const r = localCompute(q(100, 100, 100.10, 100));
    expect(r.spread).toBeCloseTo(0.10, 9);
    expect(r.microprice_minus_midpoint).toBeCloseTo(0, 9);
});

// ── localSeries ──────────────────────────────────────────────────

test('localSeries: one entry per input; preserves nulls for bad quotes', () => {
    const series = localSeries([
        q(100, 100, 100.10, 100),
        q(0, 100, 100.10, 100),     // bad → null
        q(99, 200, 99.05, 200),
    ]);
    expect(series.length).toBe(3);
    expect(series[1]).toBeNull();
    expect(series[0]).not.toBeNull();
    expect(series[2]).not.toBeNull();
});

test('localSeries: non-array → []', () => {
    expect(localSeries(null)).toEqual([]);
});

// ── summarize ─────────────────────────────────────────────────────

test('summarize: aggregates over non-null reports', () => {
    const series = [
        localCompute(q(100, 1000, 100.10, 100)),
        localCompute(q(100, 100, 100.10, 1000)),
        localCompute(q(100, 100, 100.10, 100)),
    ];
    const s = summarize(series);
    expect(s.count).toBe(3);
    expect(Number.isFinite(s.mean_micro)).toBe(true);
    expect(Number.isFinite(s.mean_imb)).toBe(true);
    // max_abs_dev > 0 because two reports are imbalanced.
    expect(s.max_abs_dev).toBeGreaterThan(0);
});

test('summarize: empty / all-null → count 0, NaN aggregates', () => {
    const s = summarize([null, null]);
    expect(s.count).toBe(0);
    expect(Number.isNaN(s.mean_micro)).toBe(true);
});

// ── imbalanceBadge ───────────────────────────────────────────────

test('imbalanceBadge: 5-tier classification', () => {
    const mk = (i) => ({ quote_imbalance: i });
    expect(imbalanceBadge(mk(0.8)).key).toMatch(/heavy_bid/);
    expect(imbalanceBadge(mk(0.4)).key).toMatch(/bid_lean/);
    expect(imbalanceBadge(mk(0)).key).toMatch(/balanced/);
    expect(imbalanceBadge(mk(-0.4)).key).toMatch(/ask_lean/);
    expect(imbalanceBadge(mk(-0.8)).key).toMatch(/heavy_ask/);
    expect(imbalanceBadge(null).key).toMatch(/unknown/);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + produces a series with no nulls', () => {
    for (const k of ['balanced','heavy-bid','heavy-ask','extreme-bid','extreme-ask',
                     'evolving-imbalance','tight-spread','wide-spread']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const series = localSeries(inp.quotes);
        expect(series.length).toBe(inp.quotes.length);
        for (const r of series) expect(r).not.toBeNull();
    }
});

test('demo balanced: each microprice equals its midpoint', () => {
    const inp = makeDemoInput('balanced');
    const series = localSeries(inp.quotes);
    for (const r of series) expect(r.microprice).toBeCloseTo(r.midpoint, 12);
});

test('demo heavy-bid: every microprice > midpoint', () => {
    const inp = makeDemoInput('heavy-bid');
    const series = localSeries(inp.quotes);
    for (const r of series) expect(r.microprice).toBeGreaterThan(r.midpoint);
});

test('demo heavy-ask: every microprice < midpoint', () => {
    const inp = makeDemoInput('heavy-ask');
    const series = localSeries(inp.quotes);
    for (const r of series) expect(r.microprice).toBeLessThan(r.midpoint);
});

test('demo evolving-imbalance: imbalance sign flips from positive to negative across series', () => {
    const inp = makeDemoInput('evolving-imbalance');
    const series = localSeries(inp.quotes);
    expect(series[0].quote_imbalance).toBeGreaterThan(0);
    expect(series[series.length - 1].quote_imbalance).toBeLessThan(0);
});

test('demo extreme-bid: imbalance ≈ +1', () => {
    const inp = makeDemoInput('extreme-bid');
    const series = localSeries(inp.quotes);
    expect(Math.abs(series[0].quote_imbalance - 1)).toBeLessThan(1e-5);
});

// ── round-trip + formatters ──────────────────────────────────────

test('quotesToBlob round-trips through parseQuotesBlob', () => {
    const quotes = [q(100, 100, 100.10, 100), q(99.99, 250, 100.01, 250)];
    const back = parseQuotesBlob(quotesToBlob(quotes));
    expect(back.errors).toEqual([]);
    expect(back.quotes).toEqual(quotes);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtUSD(100.05)).toBe('$100.0500');
    expect(fmtUSDSigned(0.05)).toBe('+$0.0500');
    expect(fmtUSDSigned(-0.05)).toBe('-$0.0500');
    expect(fmtBps(0.001)).toBe('10.00 bps');
    expect(fmtImb(0.5)).toBe('+0.5000');
    expect(fmtImb(-0.5)).toBe('-0.5000');
    expect(fmtNum(42.123, 1)).toBe('42.1');
    expect(fmtInt(42.7)).toBe('42');
    expect(fmtUSD(NaN)).toBe('—');
});
