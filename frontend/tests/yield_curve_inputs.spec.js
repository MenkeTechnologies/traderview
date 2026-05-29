// Yield-curve helpers: constants, validator, body, localClassify
// Rust-mirror parity with priority/boundary tests, badges, demos.

import { test, expect } from 'vitest';
import {
    TENORS, TENOR_YEARS, TENOR_LABELS, SHAPES, DEFAULT_INPUTS,
    validateInputs, buildBody, localClassify,
    shapeBadge, consecutiveSpreads, makeDemoCurve,
    fmtPct, fmtBpsSigned, fmtSpreadPct,
} from '../js/_yield_curve_inputs.js';

const curve = (a, b, c, d, e) => ({ y3m: a, y2y: b, y5y: c, y10y: d, y30y: e });

// ── constants ─────────────────────────────────────────────────────

test('TENORS = the 5 Rust struct fields in canonical order', () => {
    expect(TENORS).toEqual(['y3m', 'y2y', 'y5y', 'y10y', 'y30y']);
});

test('TENOR_YEARS + TENOR_LABELS cover every tenor', () => {
    expect(TENOR_YEARS).toEqual({ y3m: 0.25, y2y: 2, y5y: 5, y10y: 10, y30y: 30 });
    expect(TENOR_LABELS).toEqual({ y3m: '3M', y2y: '2Y', y5y: '5Y', y10y: '10Y', y30y: '30Y' });
});

test('SHAPES = the 4 Rust enum variants', () => {
    expect(SHAPES).toEqual(['normal', 'flat', 'inverted', 'humped']);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts default', () => {
    expect(validateInputs(DEFAULT_INPUTS)).toBe(null);
});

test('validate accepts negative yields (recent EU / JP reality)', () => {
    expect(validateInputs(curve(-0.001, 0, 0.005, 0.01, 0.015))).toBe(null);
});

test('validate rejects non-finite tenor', () => {
    expect(validateInputs({ ...DEFAULT_INPUTS, y10y: NaN })).toMatch(/y10y/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody emits flat numeric body', () => {
    expect(buildBody(curve(0.03, 0.035, 0.04, 0.045, 0.05))).toEqual({
        y3m: 0.03, y2y: 0.035, y5y: 0.04, y10y: 0.045, y30y: 0.05,
    });
});

// ── localClassify parity (one test per Rust property) ─────────────

test('local: normal curve classified normal, spread > 0', () => {
    const r = localClassify(curve(0.03, 0.035, 0.04, 0.045, 0.05));
    expect(r.shape).toBe('normal');
    expect(r.spread_10y_2y_bps).toBeGreaterThan(0);
});

test('local: inverted (2Y > 10Y) → inverted, spread < 0', () => {
    const r = localClassify(curve(0.04, 0.055, 0.05, 0.045, 0.045));
    expect(r.shape).toBe('inverted');
    expect(r.spread_10y_2y_bps).toBeLessThan(0);
});

test('local: flat curve (all spreads < 25bps) → flat', () => {
    const r = localClassify(curve(0.04, 0.041, 0.042, 0.0425, 0.043));
    expect(r.shape).toBe('flat');
});

test('local: humped (5Y peak + both ends lower) → humped', () => {
    const r = localClassify(curve(0.03, 0.04, 0.06, 0.045, 0.03));
    expect(r.shape).toBe('humped');
});

test('local: spreads in basis points (× 10000)', () => {
    const r = localClassify(curve(0.03, 0.035, 0.04, 0.045, 0.05));
    expect(r.spread_10y_2y_bps).toBeCloseTo(100, 9);   // 4.5% - 3.5% = 1% = 100bps
    expect(r.spread_10y_3m_bps).toBeCloseTo(150, 9);   // 4.5% - 3% = 1.5% = 150bps
});

test('local: inverted note_params carries magnitude in bps', () => {
    const r = localClassify(curve(0.04, 0.055, 0.05, 0.045, 0.045));
    expect(r.note_key).toBe('view.yield_curve.note.inverted');
    expect(r.note_params.bps).toBe(100);
});

test('local: tiny 30Y noise still normal (≥ -1bps tolerance)', () => {
    const r = localClassify(curve(0.03, 0.035, 0.04, 0.045, 0.0449));
    expect(r.shape).toBe('normal');
});

// ── priority order (inverted > humped > flat > normal) ───────────

test('priority: inverted check wins over humped (10Y < 2Y always inverted)', () => {
    // y5y peak above both ends BUT y2y > y10y too → still inverted.
    const r = localClassify(curve(0.03, 0.06, 0.08, 0.05, 0.03));
    expect(r.shape).toBe('inverted');
});

test('priority: humped wins over flat / normal when 5Y peaks both ends', () => {
    const r = localClassify(curve(0.03, 0.04, 0.06, 0.045, 0.03));
    expect(r.shape).toBe('humped');
});

test('priority: mixed non-monotonic (not humped, not all-flat-spreads) → flat fallback', () => {
    // 5Y dips below 2Y but not enough for humped (10Y > 5Y); not all spreads
    // < 25bps either.
    const r = localClassify(curve(0.030, 0.045, 0.040, 0.050, 0.060));
    expect(r.shape).toBe('flat');
});

// ── shapeBadge ────────────────────────────────────────────────────

test('shapeBadge: normal=pos, inverted=neg, flat/humped=empty', () => {
    expect(shapeBadge('normal').cls).toBe('pos');
    expect(shapeBadge('flat').cls).toBe('');
    expect(shapeBadge('inverted').cls).toBe('neg');
    expect(shapeBadge('humped').cls).toBe('');
    expect(shapeBadge('bogus').key).toMatch(/unknown/);
});

// ── consecutiveSpreads ────────────────────────────────────────────

test('consecutiveSpreads: 4 rows of pairwise diffs', () => {
    const rows = consecutiveSpreads(curve(0.03, 0.035, 0.04, 0.045, 0.05));
    expect(rows.length).toBe(4);
    expect(rows[0].from).toBe('y3m');
    expect(rows[0].to).toBe('y2y');
    expect(rows[0].delta).toBeCloseTo(0.005, 9);
    expect(rows[3].from).toBe('y10y');
    expect(rows[3].to).toBe('y30y');
    expect(rows[3].delta).toBeCloseTo(0.005, 9);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset classifies into expected shape', () => {
    const matrix = {
        'normal':            'normal',
        'inverted':          'inverted',
        'flat':              'flat',
        'humped':            'humped',
        'noisy-normal':      'normal',
        'ust-2024-inverted': 'inverted',
        'ust-2020-zirp':     'normal',
        'gfc-2008-flat':     'normal',
    };
    for (const [k, expected] of Object.entries(matrix)) {
        expect(localClassify(makeDemoCurve(k)).shape).toBe(expected);
    }
});

test('demo ust-2024-inverted: spread_10y_2y < 0', () => {
    expect(localClassify(makeDemoCurve('ust-2024-inverted')).spread_10y_2y_bps).toBeLessThan(0);
});

// ── formatters ────────────────────────────────────────────────────

test('fmt helpers + non-finite guards', () => {
    expect(fmtPct(0.0425)).toBe('4.250%');
    expect(fmtBpsSigned(100)).toBe('+100 bps');
    expect(fmtBpsSigned(-50)).toBe('-50 bps');
    expect(fmtSpreadPct(0.005)).toBe('+0.5000%');
    expect(fmtSpreadPct(-0.001)).toBe('-0.1000%');
    expect(fmtPct(NaN)).toBe('—');
    expect(fmtBpsSigned(null)).toBe('—');
});
