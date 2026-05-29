// Execution Scheduler pure helpers: volume-curve parser, input
// validator, per-algo payload builders, schedule summarizer.

import { test, expect } from 'vitest';
import {
    parseVolumeCurve, validateExecInputs,
    buildPovBody, buildTwapBody, buildVwapBody,
    summarizeSchedule,
} from '../js/_execution_scheduler_inputs.js';

// ── parseVolumeCurve ─────────────────────────────────────────────────

test('parseVolumeCurve accepts one-per-line numbers', () => {
    expect(parseVolumeCurve('100\n200\n300').value).toEqual([100, 200, 300]);
});

test('parseVolumeCurve accepts mixed delimiters', () => {
    expect(parseVolumeCurve('100, 200 300\n400').value).toEqual([100, 200, 300, 400]);
});

test('parseVolumeCurve flags negative volumes', () => {
    const r = parseVolumeCurve('100\n-50\n200');
    expect(r.value).toEqual([100, 200]);
    expect(r.errors.length).toBe(1);
    expect(r.errors[0].message).toMatch(/negative/);
});

test('parseVolumeCurve flags non-numeric tokens', () => {
    const r = parseVolumeCurve('100\nfoo\n200');
    expect(r.value).toEqual([100, 200]);
    expect(r.errors.length).toBe(1);
});

test('parseVolumeCurve ignores blanks + # comments', () => {
    expect(parseVolumeCurve('# header\n\n100\n# inline').value).toEqual([100]);
});

// ── validateExecInputs ──────────────────────────────────────────────

test('validateExecInputs requires positive total order', () => {
    expect(validateExecInputs(0, [100], 0.1)).toMatch(/total order/);
    expect(validateExecInputs(-1, [100], 0.1)).toMatch(/total order/);
});

test('validateExecInputs requires non-empty curve', () => {
    expect(validateExecInputs(1000, [], 0.1)).toMatch(/at least one bar/);
});

test('validateExecInputs rejects curve with negative or non-finite values', () => {
    expect(validateExecInputs(1000, [100, -10, 200], 0.1)).toMatch(/invalid/);
    expect(validateExecInputs(1000, [100, NaN, 200], 0.1)).toMatch(/invalid/);
});

test('validateExecInputs rejects zero-sum curve', () => {
    expect(validateExecInputs(1000, [0, 0, 0], 0.1)).toMatch(/sums to 0/);
});

test('validateExecInputs rejects bad participation rate', () => {
    expect(validateExecInputs(1000, [100], 0)).toMatch(/participation/);
    expect(validateExecInputs(1000, [100], 1.5)).toMatch(/participation/);
});

test('validateExecInputs returns null on good inputs', () => {
    expect(validateExecInputs(1000, [100, 200, 300], 0.1)).toBe(null);
});

// ── buildPovBody / buildTwapBody / buildVwapBody ────────────────────

test('buildPovBody includes participation rate', () => {
    const b = buildPovBody(1000, [100, 200], 0.15);
    expect(b).toEqual({
        total_order_size: 1000,
        volume_curve: [100, 200],
        participation_rate: 0.15,
    });
});

test('buildTwapBody includes num_slices', () => {
    const b = buildTwapBody(1000, 5, [100, 200, 300, 400, 500]);
    expect(b.total_order_size).toBe(1000);
    expect(b.num_slices).toBe(5);
    expect(b.volume_curve).toEqual([100, 200, 300, 400, 500]);
});

test('buildTwapBody omits volume_curve when length mismatches num_slices', () => {
    // The TWAP endpoint treats it as None internally — frontend just
    // omits the key to keep payload tidy.
    const b = buildTwapBody(1000, 5, [100, 200]);
    expect(b.volume_curve).toBeUndefined();
});

test('buildTwapBody omits volume_curve when not provided', () => {
    const b = buildTwapBody(1000, 5);
    expect(b.volume_curve).toBeUndefined();
});

test('buildVwapBody has total + curve only', () => {
    const b = buildVwapBody(1000, [100, 200]);
    expect(b).toEqual({ total_order_size: 1000, volume_curve: [100, 200] });
});

// ── summarizeSchedule ───────────────────────────────────────────────

test('summarizeSchedule returns null for null response', () => {
    expect(summarizeSchedule(null)).toBe(null);
    expect(summarizeSchedule({})).toBe(null);
});

test('summarizeSchedule extracts totalFilled from cumulative_fill', () => {
    const r = { slices: [100, 200, 300], cumulative_fill: [100, 300, 600] };
    const s = summarizeSchedule(r);
    expect(s.totalFilled).toBe(600);
});

test('summarizeSchedule reports lastFillBar (last non-zero slice)', () => {
    const r = { slices: [100, 200, 0, 0], cumulative_fill: [100, 300, 300, 300] };
    const s = summarizeSchedule(r);
    expect(s.lastFillBar).toBe(1);
});

test('summarizeSchedule preserves POV shortfall + completionBar', () => {
    const r = {
        slices: [100, 0], cumulative_fill: [100, 100],
        shortfall: 50, completion_bar: 0,
    };
    const s = summarizeSchedule(r);
    expect(s.shortfall).toBe(50);
    expect(s.completionBar).toBe(0);
});

test('summarizeSchedule preserves max_participation_rate', () => {
    const r = {
        slices: [100, 200], cumulative_fill: [100, 300],
        max_participation_rate: 0.25,
    };
    expect(summarizeSchedule(r).maxParticipation).toBe(0.25);
});

test('summarizeSchedule returns null fields when backend omits them', () => {
    const r = { slices: [10, 20], cumulative_fill: [10, 30] };
    const s = summarizeSchedule(r);
    expect(s.shortfall).toBe(null);
    expect(s.completionBar).toBe(null);
    expect(s.maxParticipation).toBe(null);
});
