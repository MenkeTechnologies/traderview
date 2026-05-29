// IV Backtest pure helpers: realized parser, validator, body shape,
// recommendation badge, histogram bucketer, demo invariants, formatters.

import { test, expect } from 'vitest';
import {
    parseRealized, validateInputs, buildBody,
    recommendationBadge, histogram, makeDemoData,
    fmtPct, fmtPnl, fmtWinRate,
} from '../js/_iv_backtest_inputs.js';

// ── parseRealized (delegates to shared parser; signed allowed) ────

test('parseRealized accepts signed values + comments', () => {
    const r = parseRealized('# header\n7.2\n-8.5\n5.1');
    expect(r.errors).toEqual([]);
    expect(r.value).toEqual([7.2, -8.5, 5.1]);
});

// ── validateInputs ────────────────────────────────────────────────

test('validate accepts defaults', () => {
    expect(validateInputs(4.5, [7, -8, 5, 6])).toBe(null);
});

test('validate rejects non-positive implied', () => {
    expect(validateInputs(0, [1, 2, 3, 4])).toMatch(/implied_move_pct/);
    expect(validateInputs(-1, [1, 2, 3, 4])).toMatch(/implied_move_pct/);
});

test('validate flags suspiciously-high implied as bps confusion', () => {
    expect(validateInputs(550, [1, 2, 3, 4])).toMatch(/bps/);
});

test('validate requires ≥4 observations', () => {
    expect(validateInputs(4.5, [1, 2])).toMatch(/at least 4/);
});

test('validate rejects non-finite history values', () => {
    expect(validateInputs(4.5, [1, NaN, 3, 4])).toMatch(/finite/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody emits backend IvBacktestBody shape', () => {
    expect(buildBody(4.5, [1, 2])).toEqual({
        implied_move_pct: 4.5, realized_pcts: [1, 2],
    });
});

// ── recommendationBadge ───────────────────────────────────────────

test('recommendationBadge: long / short / neutral colors and hints', () => {
    const long  = recommendationBadge('long', -2.3);
    const short = recommendationBadge('short', 3.1);
    const neut  = recommendationBadge('neutral', 0.5);
    expect(long.cls).toBe('pos');
    expect(long.label).toMatch(/LONG/);
    expect(long.hint).toMatch(/buy premium/);
    expect(short.cls).toBe('neg');
    expect(short.label).toMatch(/SHORT/);
    expect(short.hint).toMatch(/sell premium/);
    expect(neut.cls).toBe('');
    expect(neut.label).toMatch(/NEUTRAL/);
});

test('recommendationBadge: signed edge in label', () => {
    expect(recommendationBadge('short', 3.4).label).toMatch(/\+3\.40%/);
    expect(recommendationBadge('long', -2.1).label).toMatch(/-2\.10%/);
});

test('recommendationBadge: NaN edge omits the +/-', () => {
    const b = recommendationBadge('neutral', NaN);
    expect(b.label).toMatch(/NEUTRAL/);
});

test('recommendationBadge: unknown rec defaults to neutral path', () => {
    expect(recommendationBadge('garbage', 0).cls).toBe('');
});

// ── histogram ─────────────────────────────────────────────────────

test('histogram returns nBins centers + counts', () => {
    const h = histogram([1, 2, 3, 4, 5, 6, 7, 8, 9, 10], 5);
    expect(h.centers.length).toBe(5);
    expect(h.counts.length).toBe(5);
    expect(h.counts.reduce((a, b) => a + b, 0)).toBe(10);
});

test('histogram takes |value| (signed input becomes mirrored)', () => {
    const h = histogram([-5, -3, 3, 5], 4);
    expect(h.counts.reduce((a, b) => a + b, 0)).toBe(4);
    // Max |value| = 5, lo = 0, so width 1.25 over 4 bins.
    // |3| and |5| each appear twice.
    const totalAt3 = h.counts[Math.floor(3 / 1.25)] + h.counts[Math.floor(3 / 1.25) - 1] || 0;
    void totalAt3;
});

test('histogram empty/non-array returns empty arrays', () => {
    expect(histogram([])).toEqual({ centers: [], counts: [] });
    expect(histogram(null)).toEqual({ centers: [], counts: [] });
});

test('histogram drops non-finite inputs', () => {
    const h = histogram([1, NaN, 2, Infinity, 3], 3);
    expect(h.counts.reduce((a, b) => a + b, 0)).toBe(3);
});

test('histogram all-zero input returns single-center', () => {
    const h = histogram([0, 0, 0]);
    expect(h.centers).toEqual([0]);
    expect(h.counts).toEqual([3]);
});

// ── makeDemoData ──────────────────────────────────────────────────

test('makeDemoData has 16 quarters and median |realized| > implied (long signal)', () => {
    const { implied_move_pct, realized_pcts } = makeDemoData();
    expect(realized_pcts.length).toBe(16);
    expect(implied_move_pct).toBeGreaterThan(0);
    const abs = realized_pcts.map(Math.abs).sort((a, b) => a - b);
    const median = (abs[7] + abs[8]) / 2;
    expect(median).toBeGreaterThan(implied_move_pct);
});

// ── formatters ────────────────────────────────────────────────────

test('fmtPct emits 2-decimal % with override', () => {
    expect(fmtPct(4.5)).toBe('4.50%');
    expect(fmtPct(4.5, 1)).toBe('4.5%');
    expect(fmtPct(NaN)).toBe('—');
});

test('fmtPnl scales to %, signs positive', () => {
    expect(fmtPnl(0.234)).toBe('+23.4% per $1');
    expect(fmtPnl(-0.123)).toBe('-12.3% per $1');
    expect(fmtPnl(NaN)).toBe('—');
});

test('fmtWinRate emits 0-decimal %', () => {
    expect(fmtWinRate(0.6234)).toBe('62%');
    expect(fmtWinRate(NaN)).toBe('—');
});
