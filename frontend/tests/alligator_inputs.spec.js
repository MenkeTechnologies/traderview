// Williams Alligator helpers: bar parser, validator, body shape, shift
// math, classifier, bias counts, demo invariants, median price, formatters.

import { test, expect } from 'vitest';
import {
    parseBarBlob, validateInputs, buildBody,
    SHIFTS, shiftLines, classifyPoint, biasBadge, biasCounts,
    makeDemoBars, medianPrices, fmtN, fmtPct,
} from '../js/_alligator_inputs.js';

// ── parseBarBlob ───────────────────────────────────────────────────

test('parseBarBlob accepts whitespace + commas + comments', () => {
    const r = parseBarBlob('# h\n100.5 99.5\n100.8, 99.8');
    expect(r.errors).toEqual([]);
    expect(r.bars).toEqual([
        { high: 100.5, low: 99.5 },
        { high: 100.8, low: 99.8 },
    ]);
});

test('parseBarBlob rejects wrong token count', () => {
    expect(parseBarBlob('100').errors[0].message).toMatch(/expected 2 tokens/);
});

test('parseBarBlob rejects non-positive HL', () => {
    expect(parseBarBlob('0 1').errors[0].message).toMatch(/HL/);
});

test('parseBarBlob rejects low > high', () => {
    expect(parseBarBlob('99 100').errors[0].message).toMatch(/low > high/);
});

test('parseBarBlob non-string returns 1 error', () => {
    expect(parseBarBlob(null).errors.length).toBe(1);
});

// ── validateInputs ────────────────────────────────────────────────

test('validate requires ≥ 21 bars (13 SMMA + 8 jaw shift)', () => {
    expect(validateInputs(Array(20).fill({}))).toMatch(/at least 21/);
    expect(validateInputs(Array(21).fill({}))).toBe(null);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody emits backend AlligatorBody shape', () => {
    const bars = [{ high: 100, low: 99 }];
    expect(buildBody(bars)).toEqual({ bars });
});

// ── SHIFTS ────────────────────────────────────────────────────────

test('SHIFTS match Williams canonical values', () => {
    expect(SHIFTS).toEqual({ jaw: 8, teeth: 5, lips: 3 });
});

// ── shiftLines ────────────────────────────────────────────────────

test('shiftLines pushes each line forward by its shift', () => {
    const points = [
        { jaw: 10, teeth: 20, lips: 30, sleeping: false },
        { jaw: 11, teeth: 21, lips: 31, sleeping: false },
    ];
    const { jaw, teeth, lips } = shiftLines(points, 12);
    expect(jaw[8]).toBe(10);   // 0+8
    expect(jaw[9]).toBe(11);   // 1+8
    expect(jaw[0]).toBe(null);
    expect(teeth[5]).toBe(20); // 0+5
    expect(teeth[6]).toBe(21); // 1+5
    expect(lips[3]).toBe(30);  // 0+3
    expect(lips[4]).toBe(31);  // 1+3
});

test('shiftLines drops cells beyond chart end', () => {
    const points = [{ jaw: 10, teeth: 20, lips: 30, sleeping: false }];
    // jaw shift = 8 → destination = 8; with totalBars = 5, destination is past end.
    const { jaw } = shiftLines(points, 5);
    expect(jaw.every(v => v === null)).toBe(true);
});

test('shiftLines skips zero-seeded points (pre-period nulls)', () => {
    const points = [
        { jaw: 0, teeth: 0, lips: 0, sleeping: false },
        { jaw: 10, teeth: 20, lips: 30, sleeping: false },
    ];
    const { jaw, lips } = shiftLines(points, 12);
    // First point is zero (SMMA warmup) → must not appear in destination.
    expect(jaw[8]).toBe(null);
    expect(jaw[9]).toBe(10);
    expect(lips[3]).toBe(null);
    expect(lips[4]).toBe(30);
});

test('shiftLines safe on non-array points', () => {
    const out = shiftLines(null, 5);
    expect(out.jaw).toEqual([null, null, null, null, null]);
});

// ── classifyPoint (backend parity) ────────────────────────────────

test('classifyPoint: sleeping flag short-circuits', () => {
    expect(classifyPoint({ jaw: 100, teeth: 99, lips: 98, sleeping: true })).toBe('sleeping');
});

test('classifyPoint: lips > teeth > jaw → up', () => {
    expect(classifyPoint({ jaw: 100, teeth: 101, lips: 102, sleeping: false })).toBe('up');
});

test('classifyPoint: lips < teeth < jaw → down', () => {
    expect(classifyPoint({ jaw: 102, teeth: 101, lips: 100, sleeping: false })).toBe('down');
});

test('classifyPoint: non-monotonic → sleeping fallback', () => {
    expect(classifyPoint({ jaw: 101, teeth: 100, lips: 102, sleeping: false })).toBe('sleeping');
});

test('classifyPoint: null point → sleeping', () => {
    expect(classifyPoint(null)).toBe('sleeping');
});

// ── biasBadge / biasCounts ────────────────────────────────────────

test('biasBadge labels include hint with strategy guidance', () => {
    expect(biasBadge('up').hint).toMatch(/trend bias up/);
    expect(biasBadge('down').hint).toMatch(/trend bias down/);
    expect(biasBadge('sleeping').hint).toMatch(/no trade/);
});

test('biasBadge unknown / null fallthrough', () => {
    expect(biasBadge('garbage').label).toBe('garbage');
    expect(biasBadge(null).label).toBe('—');
});

test('biasCounts aggregates across point series', () => {
    const points = [
        { jaw: 100, teeth: 99, lips: 98, sleeping: true },   // sleeping
        { jaw: 100, teeth: 101, lips: 102, sleeping: false }, // up
        { jaw: 102, teeth: 101, lips: 100, sleeping: false }, // down
        { jaw: 100, teeth: 101, lips: 99, sleeping: false },  // non-monotonic → sleeping
    ];
    expect(biasCounts(points)).toEqual({ up: 1, down: 1, sleeping: 2 });
});

test('biasCounts safe on non-array', () => {
    expect(biasCounts(null)).toEqual({ up: 0, down: 0, sleeping: 0 });
});

// ── makeDemoBars ──────────────────────────────────────────────────

test('makeDemoBars returns exactly 50 bars (15+15+5+15 phases)', () => {
    expect(makeDemoBars().length).toBe(50);
});

test('makeDemoBars first 15 are tight (max range ≤ 0.5)', () => {
    const bars = makeDemoBars();
    const tight = bars.slice(0, 15);
    expect(tight.every(b => (b.high - b.low) <= 0.5)).toBe(true);
});

test('makeDemoBars uptrend phase (bars 15-29) has rising medians', () => {
    const bars = makeDemoBars();
    const meds = medianPrices(bars).slice(15, 30);
    expect(meds[meds.length - 1]).toBeGreaterThan(meds[0] + 5);
});

test('makeDemoBars downtrend phase (bars 35-49) has falling medians', () => {
    const bars = makeDemoBars();
    const meds = medianPrices(bars).slice(35, 50);
    expect(meds[meds.length - 1]).toBeLessThan(meds[0] - 5);
});

// ── medianPrices ──────────────────────────────────────────────────

test('medianPrices computes (H+L)/2 per bar', () => {
    expect(medianPrices([{ high: 102, low: 98 }, { high: 105, low: 95 }]))
        .toEqual([100, 100]);
});

test('medianPrices safe on non-array', () => {
    expect(medianPrices(null)).toEqual([]);
});

// ── formatters ────────────────────────────────────────────────────

test('formatters', () => {
    expect(fmtN(1.234567)).toBe('1.235');
    expect(fmtN(NaN)).toBe('—');
    expect(fmtPct(0.234)).toBe('23.4%');
    expect(fmtPct(NaN)).toBe('—');
});
