// VIX term-structure helpers: constants, validator, body, localAnalyze
// Rust-mirror parity with strict-< boundary tests, badges, demos.

import { test, expect } from 'vitest';
import {
    STATES, TENORS, TENOR_DAYS, DEFAULT_INPUTS,
    validateInputs, buildBody, localAnalyze,
    stateBadge, tenorContributions, makeDemoInput,
    fmtN, fmtSigned, fmtRatio,
} from '../js/_vix_term_structure_inputs.js';

const ts = (a, b, c, d, e) => ({ vix9d: a, vix: b, vix3m: c, vix6m: d, vix1y: e });

// ── constants ─────────────────────────────────────────────────────

test('STATES = the 5 Rust enum variants in canonical order', () => {
    expect(STATES).toEqual(['steep_contango', 'contango', 'flat', 'backwardation', 'severe_backwardation']);
});

test('TENORS + TENOR_DAYS match the 5-point curve', () => {
    expect(TENORS).toEqual(['vix9d', 'vix', 'vix3m', 'vix6m', 'vix1y']);
    expect(TENOR_DAYS).toEqual({ vix9d: 9, vix: 30, vix3m: 90, vix6m: 180, vix1y: 365 });
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts default snapshot', () => {
    expect(validateInputs(DEFAULT_INPUTS)).toBe(null);
});

test('validate rejects non-finite or negative tenor', () => {
    expect(validateInputs({ ...DEFAULT_INPUTS, vix9d: NaN })).toMatch(/vix9d/);
    expect(validateInputs({ ...DEFAULT_INPUTS, vix3m: -1 })).toMatch(/vix3m/);
});

test('validate accepts vix=0 (degenerate but valid)', () => {
    expect(validateInputs({ ...DEFAULT_INPUTS, vix: 0 })).toBe(null);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody emits flat numeric snapshot (no Decimal-as-string here)', () => {
    expect(buildBody(ts(10, 15, 18, 19, 20))).toEqual({
        vix9d: 10, vix: 15, vix3m: 18, vix6m: 19, vix1y: 20,
    });
});

// ── localAnalyze parity (one test per Rust property) ─────────────

test('local: normal contango (VIX 15, VIX3M 18 → 0.833)', () => {
    const r = localAnalyze(ts(13, 15, 18, 19, 20));
    expect(r.state).toBe('contango');
    expect(r.vix_to_vix3m_ratio).toBeCloseTo(15 / 18, 9);
});

test('local: steep contango (ratio < 0.80)', () => {
    expect(localAnalyze(ts(10, 12, 18, 19, 20)).state).toBe('steep_contango');
});

test('local: flat (VIX = VIX3M, ratio = 1.00)', () => {
    expect(localAnalyze(ts(20, 20, 20, 20, 20)).state).toBe('flat');
});

test('local: backwardation (VIX 25, VIX3M 23 → 1.087)', () => {
    expect(localAnalyze(ts(28, 25, 23, 22, 22)).state).toBe('backwardation');
});

test('local: severe backwardation at ≥ 1.20 (VIX 40, VIX3M 30 → 1.333)', () => {
    expect(localAnalyze(ts(45, 40, 30, 28, 26)).state).toBe('severe_backwardation');
});

test('local: slope positive in contango', () => {
    expect(localAnalyze(ts(13, 15, 18, 19, 20)).slope).toBeGreaterThan(0);
});

test('local: slope negative in severe backwardation', () => {
    expect(localAnalyze(ts(45, 40, 30, 28, 26)).slope).toBeLessThan(0);
});

test('local: zero VIX3M returns default report (no divide-by-zero)', () => {
    const r = localAnalyze(ts(15, 18, 0, 0, 0));
    expect(r.vix_to_vix3m_ratio).toBe(0);
});

test('local: note_key matches view.vix_term_structure.note.<state>', () => {
    expect(localAnalyze(ts(45, 40, 30, 28, 26)).note_key)
        .toBe('view.vix_term_structure.note.severe_backwardation');
});

test('local: slope equals sum of consecutive deltas', () => {
    const r = localAnalyze(ts(13, 15, 18, 19, 20));
    const expected = (15-13) + (18-15) + (19-18) + (20-19);
    expect(r.slope).toBeCloseTo(expected, 9);
});

// ── boundary tests (strict < at every transition) ─────────────────

test('boundary: ratio = 0.80 exactly → contango (not steep)', () => {
    expect(localAnalyze({ vix9d: 0, vix: 8, vix3m: 10, vix6m: 0, vix1y: 0 }).state).toBe('contango');
});

test('boundary: ratio = 1.00 exactly → flat (not contango)', () => {
    expect(localAnalyze({ vix9d: 0, vix: 10, vix3m: 10, vix6m: 0, vix1y: 0 }).state).toBe('flat');
});

test('boundary: ratio = 1.05 exactly → backwardation (not flat)', () => {
    expect(localAnalyze({ vix9d: 0, vix: 10.5, vix3m: 10, vix6m: 0, vix1y: 0 }).state).toBe('backwardation');
});

test('boundary: ratio = 1.20 exactly → severe (not backwardation)', () => {
    expect(localAnalyze({ vix9d: 0, vix: 12, vix3m: 10, vix6m: 0, vix1y: 0 }).state).toBe('severe_backwardation');
});

// ── stateBadge ────────────────────────────────────────────────────

test('stateBadge: contango/steep = pos, backwardation/severe = neg, flat = empty', () => {
    expect(stateBadge('steep_contango').cls).toBe('pos');
    expect(stateBadge('contango').cls).toBe('pos');
    expect(stateBadge('flat').cls).toBe('');
    expect(stateBadge('backwardation').cls).toBe('neg');
    expect(stateBadge('severe_backwardation').cls).toBe('neg');
    expect(stateBadge('bogus').key).toMatch(/unknown/);
});

// ── tenorContributions ────────────────────────────────────────────

test('tenorContributions: 4 rows of consecutive-tenor deltas', () => {
    const rows = tenorContributions(ts(13, 15, 18, 19, 20));
    expect(rows.length).toBe(4);
    expect(rows.map(r => [r.from, r.to])).toEqual([
        ['vix9d', 'vix'], ['vix', 'vix3m'], ['vix3m', 'vix6m'], ['vix6m', 'vix1y'],
    ]);
    expect(rows[0].delta).toBe(2);
    expect(rows[3].delta).toBe(1);
});

test('tenorContributions: empty input safe', () => {
    expect(tenorContributions(null)).toEqual([]);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset classifies into expected state', () => {
    const matrix = {
        'steep-contango':   'steep_contango',
        'normal-contango':  'contango',
        'flat':             'flat',
        'backwardation':    'backwardation',
        'severe':           'severe_backwardation',
        'covid-spike':      'severe_backwardation',
        'gfc-bear':         'severe_backwardation',
        'low-vol-regime':   'steep_contango',
    };
    for (const [k, expected] of Object.entries(matrix)) {
        expect(localAnalyze(makeDemoInput(k)).state).toBe(expected);
    }
});

test('demo low-vol-regime: positive slope (contango through tenors)', () => {
    expect(localAnalyze(makeDemoInput('low-vol-regime')).slope).toBeGreaterThan(0);
});

test('demo covid-spike: negative slope (backwardation through tenors)', () => {
    expect(localAnalyze(makeDemoInput('covid-spike')).slope).toBeLessThan(0);
});

// ── formatters ────────────────────────────────────────────────────

test('fmt helpers + non-finite guards', () => {
    expect(fmtN(15.234, 2)).toBe('15.23');
    expect(fmtSigned(2.5)).toBe('+2.50');
    expect(fmtSigned(-3)).toBe('-3.00');
    expect(fmtRatio(0.8333)).toBe('0.833');
    expect(fmtN(NaN)).toBe('—');
    expect(fmtRatio(null)).toBe('—');
});
