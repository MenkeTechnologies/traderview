// VWAP Slippage helpers: bar parser, validator, body shape, local VWAP
// parity, rolling VWAP, tagged-enum unwrap, Decimal-string coercion,
// demo invariants, formatters.

import { test, expect } from 'vitest';
import {
    parseBarBlob, validateInputs, buildBody,
    localVwap, rollingVwap, unwrapResponse, decToNum,
    makeDemoData, fmtN, fmtBps, fmtVol,
} from '../js/_vwap_slippage_inputs.js';

// ── parseBarBlob ───────────────────────────────────────────────────

test('parseBarBlob accepts whitespace + commas + comments', () => {
    const r = parseBarBlob('# header\n100.05 1200\n100.08, 850');
    expect(r.errors).toEqual([]);
    expect(r.bars).toEqual([
        { typical: 100.05, volume: 1200 },
        { typical: 100.08, volume: 850 },
    ]);
});

test('parseBarBlob rejects wrong token count', () => {
    expect(parseBarBlob('100').errors[0].message).toMatch(/expected 2 tokens/);
});

test('parseBarBlob rejects non-positive typical price', () => {
    expect(parseBarBlob('0 100').errors[0].message).toMatch(/typical price/);
    expect(parseBarBlob('-1 100').errors[0].message).toMatch(/typical price/);
});

test('parseBarBlob accepts volume = 0 (silent bar)', () => {
    const r = parseBarBlob('100 0');
    expect(r.errors).toEqual([]);
    expect(r.bars).toEqual([{ typical: 100, volume: 0 }]);
});

test('parseBarBlob rejects negative volume', () => {
    expect(parseBarBlob('100 -5').errors[0].message).toMatch(/volume/);
});

test('parseBarBlob non-string returns 1 error', () => {
    expect(parseBarBlob(null).errors.length).toBe(1);
});

// ── validateInputs ────────────────────────────────────────────────

test('validate accepts good long-side inputs', () => {
    expect(validateInputs('long', 100, [{ typical: 100, volume: 1000 }])).toBe(null);
});

test('validate rejects bad side', () => {
    expect(validateInputs('flat', 100, [{ typical: 100, volume: 1000 }]))
        .toMatch(/side must be long or short/);
});

test('validate rejects non-positive fill_price', () => {
    expect(validateInputs('long', 0, [{ typical: 100, volume: 1000 }]))
        .toMatch(/fill_price/);
});

test('validate rejects zero-total-volume bars (would div-by-zero backend)', () => {
    expect(validateInputs('long', 100, [{ typical: 100, volume: 0 }]))
        .toMatch(/total bar volume/);
});

test('validate rejects empty bars', () => {
    expect(validateInputs('long', 100, [])).toMatch(/at least 1 bar/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody stringifies Decimal scalars per backend contract', () => {
    const body = buildBody('long', 100.5, [{ typical: 100.05, volume: 1200 }]);
    expect(body).toEqual({
        side: 'long',
        fill_price: '100.5',
        bars: [{ typical: '100.05', volume: '1200' }],
    });
});

// ── localVwap (parity with backend formula) ───────────────────────

test('localVwap computes Σ(typical·volume) / Σ(volume)', () => {
    // typical=[10, 20, 30], volume=[100, 200, 300]
    // num = 10·100 + 20·200 + 30·300 = 1000 + 4000 + 9000 = 14000
    // den = 600 → 14000/600 = 23.333...
    const v = localVwap([
        { typical: 10, volume: 100 },
        { typical: 20, volume: 200 },
        { typical: 30, volume: 300 },
    ]);
    expect(v).toBeCloseTo(14000 / 600, 10);
});

test('localVwap drops bars with non-finite typical or non-positive volume', () => {
    const v = localVwap([
        { typical: 100, volume: 1000 },
        { typical: NaN, volume: 500 },
        { typical: 200, volume: -1 },
        { typical: 50,  volume: 500 },
    ]);
    expect(v).toBeCloseTo((100 * 1000 + 50 * 500) / 1500, 10);
});

test('localVwap returns NaN on empty or zero-volume input', () => {
    expect(localVwap([])).toBeNaN();
    expect(localVwap([{ typical: 100, volume: 0 }])).toBeNaN();
});

// ── rollingVwap ───────────────────────────────────────────────────

test('rollingVwap returns per-bar cumulative VWAP', () => {
    const r = rollingVwap([
        { typical: 10, volume: 100 },
        { typical: 20, volume: 100 },
        { typical: 30, volume: 100 },
    ]);
    expect(r[0]).toBeCloseTo(10, 10);
    expect(r[1]).toBeCloseTo(15, 10);
    expect(r[2]).toBeCloseTo(20, 10);
});

test('rollingVwap emits null for bad bars', () => {
    const r = rollingVwap([
        { typical: 100, volume: 1000 },
        { typical: NaN, volume: 500 },
    ]);
    expect(r[0]).toBeCloseTo(100, 10);
    expect(r[1]).toBe(null);
});

// ── unwrapResponse ────────────────────────────────────────────────

test('unwrapResponse handles "computed" tag', () => {
    const u = unwrapResponse({ kind: 'computed', vwap: '100' });
    expect(u.ok).toBe(true);
    expect(u.result.vwap).toBe('100');
});

test('unwrapResponse handles "empty" tag with reason passthrough', () => {
    const u = unwrapResponse({ kind: 'empty', reason: 'no bars' });
    expect(u.ok).toBe(false);
    expect(u.reason).toBe('no bars');
});

test('unwrapResponse rejects malformed shapes', () => {
    expect(unwrapResponse(null).ok).toBe(false);
    expect(unwrapResponse({ kind: 'xyz' }).ok).toBe(false);
});

// ── decToNum ──────────────────────────────────────────────────────

test('decToNum coerces Decimal-string + number passthrough + null safe', () => {
    expect(decToNum('100.5')).toBe(100.5);
    expect(decToNum(100.5)).toBe(100.5);
    expect(decToNum(null)).toBeNaN();
    expect(decToNum('garbage')).toBeNaN();
});

// ── makeDemoData ──────────────────────────────────────────────────

test('makeDemoData deterministic + exactly 200 bars + long side', () => {
    const a = makeDemoData(42);
    const b = makeDemoData(42);
    expect(a).toEqual(b);
    expect(a.bars.length).toBe(200);
    expect(a.side).toBe('long');
});

test('makeDemoData fill_price beats VWAP for long entry across seeds', () => {
    for (const seed of [1, 7, 42, 1337]) {
        const { fill_price, bars } = makeDemoData(seed);
        const vwap = localVwap(bars);
        expect(fill_price).toBeLessThan(vwap);   // long fill below VWAP = good
    }
});

// ── formatters ────────────────────────────────────────────────────

test('fmtN + fmtBps + fmtVol', () => {
    expect(fmtN(100.123456)).toBe('100.1235');
    expect(fmtN(NaN)).toBe('—');
    expect(fmtBps(12.3)).toBe('+12.3 bps');
    expect(fmtBps(-5.4)).toBe('-5.4 bps');
    expect(fmtBps(NaN)).toBe('—');
    expect(fmtVol(1234567)).toBe('1,234,567');
});
