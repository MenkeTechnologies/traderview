// DeMarker Oscillator helpers: bar parser, validator, body shape,
// regime classifier, crossing detector, latest-finite, demo invariants,
// formatters.

import { test, expect } from 'vitest';
import {
    parseBarBlob, validateInputs, buildBody,
    OB_THRESHOLD, OS_THRESHOLD,
    regimeOf, regimeBadge, regimeCounts, detectCrossings, latestValue,
    makeDemoBars, fmtN, fmtPct,
} from '../js/_demarker_inputs.js';

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

test('parseBarBlob rejects non-positive HL + low>high', () => {
    expect(parseBarBlob('0 1').errors[0].message).toMatch(/HL/);
    expect(parseBarBlob('99 100').errors[0].message).toMatch(/low > high/);
});

test('parseBarBlob non-string returns 1 error', () => {
    expect(parseBarBlob(null).errors.length).toBe(1);
});

// ── validateInputs ────────────────────────────────────────────────

test('validate accepts canonical inputs', () => {
    expect(validateInputs(Array(20).fill({ high: 100, low: 99 }), 14)).toBe(null);
});

test('validate rejects empty bars', () => {
    expect(validateInputs([], 14)).toMatch(/at least 1 bar/);
});

test('validate enforces integer period ≥ 2', () => {
    expect(validateInputs([{}, {}], 1)).toMatch(/period/);
    expect(validateInputs([{}, {}], 1.5)).toMatch(/period/);
});

test('validate enforces ≥ period+1 bars with computed message', () => {
    expect(validateInputs(Array(10).fill({}), 14)).toMatch(/at least 15 bars/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody splits bars into parallel highs/lows arrays', () => {
    const bars = [{ high: 100, low: 99 }, { high: 101, low: 100 }];
    expect(buildBody(bars, 14)).toEqual({
        highs: [100, 101], lows: [99, 100], period: 14,
    });
});

// ── constants ─────────────────────────────────────────────────────

test('OB / OS thresholds match Tom DeMark canonical 0.70 / 0.30', () => {
    expect(OB_THRESHOLD).toBe(0.7);
    expect(OS_THRESHOLD).toBe(0.3);
});

// ── regimeOf ──────────────────────────────────────────────────────

test('regimeOf at boundaries (≥0.70 OB, ≤0.30 OS, between neutral)', () => {
    expect(regimeOf(0.70)).toBe('overbought');
    expect(regimeOf(0.7001)).toBe('overbought');
    expect(regimeOf(0.6999)).toBe('neutral');
    expect(regimeOf(0.30)).toBe('oversold');
    expect(regimeOf(0.3001)).toBe('neutral');
    expect(regimeOf(0.50)).toBe('neutral');
});

test('regimeOf returns unknown on null/NaN', () => {
    expect(regimeOf(null)).toBe('unknown');
    expect(regimeOf(NaN)).toBe('unknown');
});

// ── regimeBadge ───────────────────────────────────────────────────

test('regimeBadge: OB → neg, OS → pos, neutral → empty, unknown → em-dash', () => {
    expect(regimeBadge('overbought').cls).toBe('neg');
    expect(regimeBadge('overbought').hint).toMatch(/selling pressure/);
    expect(regimeBadge('oversold').cls).toBe('pos');
    expect(regimeBadge('oversold').hint).toMatch(/buying pressure/);
    expect(regimeBadge('neutral').cls).toBe('');
    expect(regimeBadge('unknown').label).toBe('—');
});

test('regimeBadge unknown enum falls through', () => {
    expect(regimeBadge('garbage').label).toBe('garbage');
});

// ── regimeCounts ──────────────────────────────────────────────────

test('regimeCounts aggregates including unknown for warmup nulls', () => {
    const values = [null, null, 0.2, 0.5, 0.8, 0.75, 0.4, NaN];
    expect(regimeCounts(values)).toEqual({
        overbought: 2, oversold: 1, neutral: 2, unknown: 3,
    });
});

test('regimeCounts safe on non-array', () => {
    expect(regimeCounts(null)).toEqual({ overbought: 0, oversold: 0, neutral: 0, unknown: 0 });
});

// ── detectCrossings ───────────────────────────────────────────────

test('detectCrossings flags only entry crossings into OB/OS', () => {
    // Sequence: warmup-warmup-neutral-OB-OB-neutral-OS-OS
    const values = [null, null, 0.5, 0.8, 0.85, 0.5, 0.2, 0.15];
    const events = detectCrossings(values);
    expect(events.length).toBe(2);
    expect(events[0]).toEqual({ bar_index: 3, regime: 'overbought', value: 0.8 });
    expect(events[1]).toEqual({ bar_index: 6, regime: 'oversold',   value: 0.2 });
});

test('detectCrossings: consecutive OB bars produce only one event', () => {
    const values = [null, 0.85, 0.90, 0.95];
    const events = detectCrossings(values);
    expect(events.length).toBe(1);
    expect(events[0].bar_index).toBe(1);
});

test('detectCrossings: leaving extreme back into neutral does NOT fire', () => {
    const values = [0.5, 0.8, 0.5];
    const events = detectCrossings(values);
    expect(events.length).toBe(1);   // only the entry into OB at bar 1
});

test('detectCrossings safe on non-array', () => {
    expect(detectCrossings(null)).toEqual([]);
});

// ── latestValue ───────────────────────────────────────────────────

test('latestValue returns most recent finite + its index', () => {
    expect(latestValue([null, 0.5, 0.7, null])).toEqual({ index: 2, value: 0.7 });
});

test('latestValue all-null returns -1 / NaN', () => {
    const r = latestValue([null, NaN, null]);
    expect(r.index).toBe(-1);
    expect(Number.isNaN(r.value)).toBe(true);
});

test('latestValue safe on non-array', () => {
    expect(latestValue(null)).toEqual({ index: -1, value: NaN });
});

// ── makeDemoBars ──────────────────────────────────────────────────

test('makeDemoBars returns exactly 60 bars', () => {
    expect(makeDemoBars().length).toBe(60);
});

test('makeDemoBars phase 1 (bars 0-19) has rising highs (uptrend)', () => {
    const bars = makeDemoBars();
    const firstHigh = bars[0].high;
    const lastHighPhase1 = bars[19].high;
    expect(lastHighPhase1 - firstHigh).toBeGreaterThan(10);
});

test('makeDemoBars phase 3 (bars 40-59) has falling lows (downtrend)', () => {
    const bars = makeDemoBars();
    expect(bars[40].low - bars[59].low).toBeGreaterThan(10);
});

test('makeDemoBars all bars have low ≤ high and positive', () => {
    const bars = makeDemoBars();
    expect(bars.every(b => b.low <= b.high && b.high > 0 && b.low > 0)).toBe(true);
});

// ── formatters ────────────────────────────────────────────────────

test('formatters', () => {
    expect(fmtN(0.85432)).toBe('0.8543');
    expect(fmtN(NaN)).toBe('—');
    expect(fmtPct(0.234)).toBe('23.4%');
    expect(fmtPct(NaN)).toBe('—');
});
