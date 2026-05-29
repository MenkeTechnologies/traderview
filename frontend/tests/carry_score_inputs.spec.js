// Carry-score helpers: validator, body shape, localScore Rust-mirror
// with boundary + tier-priority tests, tierBadge, demos, formatters.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, TIERS, validateInputs, buildBody, localScore,
    tierBadge, noteKeyForTier, makeDemoInput,
    fmtPct, fmtPctSigned, fmtScore,
} from '../js/_carry_score_inputs.js';

// ── validator ─────────────────────────────────────────────────────

test('TIERS exposes the four enum values', () => {
    expect(TIERS).toEqual(['strong', 'okay', 'poor', 'negative']);
});

test('validate accepts default', () => {
    expect(validateInputs(DEFAULT_INPUTS)).toBe(null);
});

test('validate rejects non-finite / negative vol', () => {
    expect(validateInputs({ ...DEFAULT_INPUTS, long_rate: NaN })).toMatch(/long_rate/);
    expect(validateInputs({ ...DEFAULT_INPUTS, funding_rate: Infinity })).toMatch(/funding_rate/);
    expect(validateInputs({ ...DEFAULT_INPUTS, annualized_vol: -1 })).toMatch(/annualized_vol/);
});

test('validate accepts vol=0 (will produce score=0)', () => {
    expect(validateInputs({ ...DEFAULT_INPUTS, annualized_vol: 0 })).toBe(null);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody passes through (no Decimal-as-string here — f64s)', () => {
    expect(buildBody(DEFAULT_INPUTS)).toEqual({
        long_rate: 0.05, funding_rate: 0.01, annualized_vol: 0.10,
    });
});

// ── localScore parity (one test per Rust property) ────────────────

test('local: positive diff low vol → strong (score > 1)', () => {
    const r = localScore(0.05, 0.01, 0.03);
    expect(r.carry_score).toBeGreaterThan(1);
    expect(r.tier).toBe('strong');
});

test('local: positive diff high vol → poor (score < 0.5)', () => {
    const r = localScore(0.05, 0.01, 0.20);
    expect(r.carry_score).toBeLessThan(0.5);
    expect(r.tier).toBe('poor');
});

test('local: middling score → okay (0.5 ≤ score < 1.0)', () => {
    const r = localScore(0.04, 0.01, 0.04);  // 0.75
    expect(r.tier).toBe('okay');
});

test('local: negative differential → negative tier (regardless of score)', () => {
    const r = localScore(0.01, 0.05, 0.10);
    expect(r.rate_differential).toBeLessThan(0);
    expect(r.tier).toBe('negative');
});

test('local: zero-vol → score = 0 (no divide-by-zero)', () => {
    const r = localScore(0.05, 0.01, 0);
    expect(r.carry_score).toBe(0);
});

test('local: zero-vol with positive diff → still poor (score=0 < 0.5)', () => {
    expect(localScore(0.05, 0.01, 0).tier).toBe('poor');
});

test('local: zero-vol with negative diff → negative tier (diff check wins)', () => {
    expect(localScore(0.01, 0.05, 0).tier).toBe('negative');
});

test('local: rate_differential = long - funding', () => {
    expect(localScore(0.07, 0.02, 0.10).rate_differential).toBeCloseTo(0.05, 12);
});

// ── boundary tests (≥ in Rust) ────────────────────────────────────

test('boundary: score = 1.0 exactly → strong (≥ 1.0)', () => {
    const r = localScore(0.05, 0, 0.05);
    expect(r.carry_score).toBeCloseTo(1.0, 12);
    expect(r.tier).toBe('strong');
});

test('boundary: score = 0.5 exactly → okay (≥ 0.5)', () => {
    const r = localScore(0.025, 0, 0.05);
    expect(r.carry_score).toBeCloseTo(0.5, 12);
    expect(r.tier).toBe('okay');
});

test('boundary: score = 0.499 → poor (< 0.5)', () => {
    const r = localScore(0.02495, 0, 0.05);
    expect(r.carry_score).toBeLessThan(0.5);
    expect(r.tier).toBe('poor');
});

test('boundary: diff = 0 → score = 0 → poor (NOT negative)', () => {
    expect(localScore(0.05, 0.05, 0.10).tier).toBe('poor');
});

// ── echoed fields ─────────────────────────────────────────────────

test('local: report echoes all inputs', () => {
    const r = localScore(0.07, 0.02, 0.15);
    expect(r.long_rate).toBe(0.07);
    expect(r.funding_rate).toBe(0.02);
    expect(r.annualized_vol).toBe(0.15);
});

// ── tierBadge / noteKeyForTier ────────────────────────────────────

test('tierBadge: strong/okay/poor/negative map to expected cls', () => {
    expect(tierBadge('strong').cls).toBe('pos');
    expect(tierBadge('okay').cls).toBe('');
    expect(tierBadge('poor').cls).toBe('neg');
    expect(tierBadge('negative').cls).toBe('neg');
    expect(tierBadge('bogus').key).toMatch(/unknown/);
});

test('noteKeyForTier: returns view.carry_score.note.<tier>', () => {
    expect(noteKeyForTier('strong')).toBe('view.carry_score.note.strong');
    expect(noteKeyForTier()).toBe('view.carry_score.note.unknown');
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset classifies into expected tier', () => {
    const matrix = {
        'strong-mxn-jpy':       'strong',
        'okay-aud-jpy':         'okay',
        'poor-high-vol':        'poor',
        'negative-anti-carry':  'negative',
        'boundary-strong':      'strong',
        'boundary-okay':        'okay',
        'zero-vol':             'poor',
        'eur-vs-usd-2024':      'negative',
    };
    for (const [k, expected] of Object.entries(matrix)) {
        const d = makeDemoInput(k);
        expect(localScore(d.long_rate, d.funding_rate, d.annualized_vol).tier).toBe(expected);
    }
});

test('demo strong-mxn-jpy: score > 1.3 (positive carry, low vol)', () => {
    const d = makeDemoInput('strong-mxn-jpy');
    expect(localScore(d.long_rate, d.funding_rate, d.annualized_vol).carry_score).toBeGreaterThan(1.3);
});

// ── formatters ────────────────────────────────────────────────────

test('fmt helpers + non-finite guards', () => {
    expect(fmtPct(0.05)).toBe('5.00%');
    expect(fmtPctSigned(0.04)).toBe('+4.00%');
    expect(fmtPctSigned(-0.04)).toBe('-4.00%');
    expect(fmtScore(1.234)).toBe('1.234');
    expect(fmtPct(NaN)).toBe('—');
    expect(fmtScore(null)).toBe('—');
});
