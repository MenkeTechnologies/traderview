// Optimal-f pure helpers: validator, TWR formula, sweep, formatters.

import { test, expect } from 'vitest';
import {
    validateInputs, buildBody,
    twrAt, twrSweep,
    fmtPctF, fmtMoney, fmtMultiple,
} from '../js/_optimal_f_inputs.js';

// ── validateInputs ─────────────────────────────────────────────────

test('validate rejects too few trades', () => {
    expect(validateInputs([1, -1, 2])).toMatch(/at least 5/);
});

test('validate rejects non-finite values', () => {
    expect(validateInputs([1, 2, NaN, 3, 4])).toMatch(/non-finite/);
});

test('validate rejects all-winners (no worst_loss)', () => {
    expect(validateInputs([100, 200, 50, 75, 120])).toMatch(/losing trade/);
});

test('validate accepts good mixed input', () => {
    expect(validateInputs([100, -50, 80, -30, 120])).toBe(null);
});

// ── buildBody ──────────────────────────────────────────────────────

test('buildBody emits backend shape', () => {
    expect(buildBody([1, 2, 3])).toEqual({ returns: [1, 2, 3] });
});

// ── twrAt ──────────────────────────────────────────────────────────

test('twrAt(f=0) = 1 (no bet, no growth)', () => {
    expect(twrAt([100, -50, 75], 50, 0)).toBeCloseTo(1, 12);
});

test('twrAt computes geometric product over HPRs', () => {
    // worst_loss = 50; f = 0.5.
    //   hpr_1 = 1 + 0.5 · (100/50)  = 2.0
    //   hpr_2 = 1 + 0.5 · (-50/50)  = 0.5
    //   hpr_3 = 1 + 0.5 · (75/50)   = 1.75
    //   TWR = 2.0 · 0.5 · 1.75 = 1.75
    expect(twrAt([100, -50, 75], 50, 0.5)).toBeCloseTo(1.75, 9);
});

test('twrAt returns 0 when any HPR would go non-positive', () => {
    // worst_loss=50, f=1 → hpr on -50 trade = 1 + 1 · (-1) = 0 → wipe.
    expect(twrAt([100, -50, 75], 50, 1)).toBe(0);
});

test('twrAt with zero worst_loss returns 1 (no leverage defined)', () => {
    expect(twrAt([100, 50], 0, 0.5)).toBe(1);
});

// ── twrSweep ──────────────────────────────────────────────────────

test('twrSweep returns parallel xs and ys', () => {
    const { xs, ys } = twrSweep([100, -50, 75, -30, 120], 51);
    expect(xs.length).toBe(51);
    expect(ys.length).toBe(51);
});

test('twrSweep spans (0, 1]', () => {
    const { xs } = twrSweep([100, -50, 75], 100);
    expect(xs[0]).toBeGreaterThan(0);
    expect(xs[xs.length - 1]).toBeCloseTo(1, 9);
});

test('twrSweep has a maximum (TWR is concave between blowup points)', () => {
    const { ys } = twrSweep([100, -50, 75, -30, 120], 51);
    const max = Math.max(...ys);
    const maxIdx = ys.indexOf(max);
    // Strict interior maximum (not at 0 or 1).
    expect(maxIdx).toBeGreaterThan(0);
    expect(maxIdx).toBeLessThan(ys.length - 1);
});

test('twrSweep returns empty for all-winners input', () => {
    expect(twrSweep([100, 200, 50]).xs).toEqual([]);
});

// ── formatters ─────────────────────────────────────────────────────

test('fmtPctF emits 2-decimal percent', () => {
    expect(fmtPctF(0.123)).toBe('12.30%');
});

test('fmtPctF returns "—" on non-finite', () => {
    expect(fmtPctF(NaN)).toBe('—');
});

test('fmtMoney handles positive + negative', () => {
    expect(fmtMoney(123.45)).toBe('$123.45');
    expect(fmtMoney(-123.45)).toBe('-$123.45');
});

test('fmtMoney returns "—" on non-finite', () => {
    expect(fmtMoney(NaN)).toBe('—');
});

test('fmtMultiple appends ×', () => {
    expect(fmtMultiple(1.234)).toBe('1.23×');
});

test('fmtMultiple returns "—" on non-finite', () => {
    expect(fmtMultiple(NaN)).toBe('—');
});
