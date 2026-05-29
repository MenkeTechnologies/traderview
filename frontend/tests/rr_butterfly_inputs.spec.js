// RR / BF calculator pure helpers: body shape per mode, validators,
// local closed-form decompose / reconstruct, formatters.

import { test, expect } from 'vitest';
import {
    buildBody, validateInputs,
    decomposeLocal, reconstructLocal,
    fmtVolPct, fmtSkewZ,
} from '../js/_rr_butterfly_inputs.js';

// ── buildBody ───────────────────────────────────────────────────────

test('buildBody decompose passes σ wing inputs', () => {
    const b = buildBody('decompose', { sigma_25_call: 0.085, sigma_25_put: 0.097, sigma_atm: 0.09 });
    expect(b).toEqual({ sigma_25_call: 0.085, sigma_25_put: 0.097, sigma_atm: 0.09 });
});

test('buildBody reconstruct passes ATM/RR/BF', () => {
    const b = buildBody('reconstruct', { atm: 0.09, rr: -0.012, bf: 0.001 });
    expect(b).toEqual({ atm: 0.09, rr: -0.012, bf: 0.001 });
});

test('buildBody throws on unknown mode', () => {
    expect(() => buildBody('greek', {})).toThrow(/unknown mode/);
});

// ── validateInputs ──────────────────────────────────────────────────

test('validate decompose rejects non-positive σ inputs', () => {
    expect(validateInputs('decompose', { sigma_25_call: 0, sigma_25_put: 0.1, sigma_atm: 0.1 }))
        .toMatch(/sigma_25_call/);
    expect(validateInputs('decompose', { sigma_25_call: 0.1, sigma_25_put: -1, sigma_atm: 0.1 }))
        .toMatch(/sigma_25_put/);
    expect(validateInputs('decompose', { sigma_25_call: 0.1, sigma_25_put: 0.1, sigma_atm: 0 }))
        .toMatch(/sigma_atm/);
});

test('validate decompose rejects non-finite σ inputs', () => {
    expect(validateInputs('decompose', { sigma_25_call: NaN, sigma_25_put: 0.1, sigma_atm: 0.1 }))
        .toMatch(/finite/);
});

test('validate decompose accepts a good set', () => {
    expect(validateInputs('decompose', { sigma_25_call: 0.085, sigma_25_put: 0.097, sigma_atm: 0.09 }))
        .toBe(null);
});

test('validate reconstruct rejects non-positive ATM', () => {
    expect(validateInputs('reconstruct', { atm: 0, rr: 0, bf: 0 })).toMatch(/ATM/);
});

test('validate reconstruct rejects non-finite RR or BF', () => {
    expect(validateInputs('reconstruct', { atm: 0.1, rr: NaN, bf: 0 })).toMatch(/RR/);
    expect(validateInputs('reconstruct', { atm: 0.1, rr: 0, bf: Infinity })).toMatch(/BF/);
});

test('validate reconstruct catches negative-wing pathology pre-flight', () => {
    // σ_25P would be 0.10 + 0 − 0.30/2 = -0.05.
    const err = validateInputs('reconstruct', { atm: 0.10, rr: 0.30, bf: 0 });
    expect(err).toMatch(/σ_25P would be ≤ 0/);
});

test('validate reconstruct accepts realistic FX-vol inputs', () => {
    expect(validateInputs('reconstruct', { atm: 0.09, rr: -0.012, bf: 0.001 })).toBe(null);
});

test('validate returns error on unknown mode', () => {
    expect(validateInputs('greek', {})).toMatch(/unknown mode/);
});

// ── decomposeLocal ──────────────────────────────────────────────────

test('decomposeLocal: RR = call − put', () => {
    const d = decomposeLocal(0.10, 0.08, 0.09);
    expect(d.rr).toBeCloseTo(0.02, 12);
});

test('decomposeLocal: BF = wing-avg − ATM', () => {
    const d = decomposeLocal(0.10, 0.08, 0.09);
    // wing avg = 0.09, BF = 0.09 - 0.09 = 0.
    expect(d.bf).toBeCloseTo(0, 12);
});

test('decomposeLocal: skew_zscore = RR / ATM', () => {
    const d = decomposeLocal(0.10, 0.08, 0.09);
    expect(d.skew_zscore).toBeCloseTo(0.02 / 0.09, 12);
});

test('decomposeLocal: zero ATM yields NaN skew (avoid divide-by-zero)', () => {
    const d = decomposeLocal(0.10, 0.08, 0);
    expect(Number.isNaN(d.skew_zscore)).toBe(true);
});

// ── reconstructLocal ────────────────────────────────────────────────

test('reconstructLocal: σ_25C = ATM + BF + RR/2', () => {
    const r = reconstructLocal(0.09, 0.02, 0.001);
    expect(r.sigma_25_call).toBeCloseTo(0.09 + 0.001 + 0.01, 12);
});

test('reconstructLocal: σ_25P = ATM + BF − RR/2', () => {
    const r = reconstructLocal(0.09, 0.02, 0.001);
    expect(r.sigma_25_put).toBeCloseTo(0.09 + 0.001 - 0.01, 12);
});

test('decompose ↔ reconstruct round-trip is identity', () => {
    const call = 0.105, put = 0.085, atm = 0.092;
    const d = decomposeLocal(call, put, atm);
    const r = reconstructLocal(d.atm, d.rr, d.bf);
    expect(r.sigma_25_call).toBeCloseTo(call, 12);
    expect(r.sigma_25_put).toBeCloseTo(put, 12);
});

// ── formatters ──────────────────────────────────────────────────────

test('fmtVolPct emits 3-decimal percent', () => {
    expect(fmtVolPct(0.085)).toBe('8.500%');
});

test('fmtVolPct returns "—" on non-finite', () => {
    expect(fmtVolPct(NaN)).toBe('—');
    expect(fmtVolPct(null)).toBe('—');
});

test('fmtSkewZ emits 3-decimal scalar', () => {
    expect(fmtSkewZ(-0.2222)).toBe('-0.222');
});

test('fmtSkewZ returns "—" on non-finite', () => {
    expect(fmtSkewZ(NaN)).toBe('—');
});
