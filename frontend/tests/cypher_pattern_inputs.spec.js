// Cypher Pattern helpers: pivot parser, validator, body shape,
// direction badge, ratio-quality score, grade, demo invariants, formatters.

import { test, expect } from 'vitest';
import {
    parsePivotBlob, validateInputs, buildBody,
    dirBadge, patternQuality, qualityGrade,
    makeDemoPivots, DEMO_TOLERANCE,
    fmtN, fmtRatio,
} from '../js/_cypher_pattern_inputs.js';

// ── parsePivotBlob ─────────────────────────────────────────────────

test('parsePivotBlob accepts H/L letters, case-insensitive', () => {
    const r = parsePivotBlob('0 100 L\n10 130 H\n20 115 l\n30 134 h');
    expect(r.errors).toEqual([]);
    expect(r.pivots.map(p => p.is_high)).toEqual([false, true, false, true]);
});

test('parsePivotBlob accepts high/low/true/false words', () => {
    const r = parsePivotBlob('0 100 low\n10 130 HIGH\n20 115 true\n30 134 false');
    expect(r.errors).toEqual([]);
    expect(r.pivots.map(p => p.is_high)).toEqual([false, true, true, false]);
});

test('parsePivotBlob rejects wrong token count', () => {
    expect(parsePivotBlob('0 100').errors[0].message).toMatch(/expected 3 tokens/);
});

test('parsePivotBlob rejects non-integer / negative index', () => {
    expect(parsePivotBlob('1.5 100 H').errors[0].message).toMatch(/index/);
    expect(parsePivotBlob('-1 100 H').errors[0].message).toMatch(/index/);
});

test('parsePivotBlob rejects non-positive price', () => {
    expect(parsePivotBlob('0 0 H').errors[0].message).toMatch(/price/);
});

test('parsePivotBlob rejects bad H/L token', () => {
    expect(parsePivotBlob('0 100 X').errors[0].message).toMatch(/H\/L must be/);
});

test('parsePivotBlob non-string returns 1 error', () => {
    expect(parsePivotBlob(null).errors.length).toBe(1);
});

// ── validateInputs ────────────────────────────────────────────────

const okPivots = Array.from({ length: 5 }, (_, i) => ({
    index: i * 10, price: 100 + i, is_high: i % 2 === 1,
}));

test('validate accepts ≥5 pivots + good tolerance', () => {
    expect(validateInputs(okPivots, 0.05)).toBe(null);
});

test('validate rejects < 5 pivots', () => {
    expect(validateInputs(okPivots.slice(0, 4), 0.05)).toMatch(/at least 5 pivots/);
});

test('validate rejects non-positive tolerance', () => {
    expect(validateInputs(okPivots, 0)).toMatch(/tolerance/);
    expect(validateInputs(okPivots, -0.1)).toMatch(/tolerance/);
});

test('validate warns if tolerance > 0.5 (would match almost anything)', () => {
    expect(validateInputs(okPivots, 0.6)).toMatch(/0.5/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody emits backend HarmonicPatternBody shape', () => {
    expect(buildBody(okPivots, 0.05)).toEqual({ pivots: okPivots, tolerance: 0.05 });
});

// ── dirBadge ──────────────────────────────────────────────────────

test('dirBadge bullish=pos, bearish=neg, fallthrough', () => {
    expect(dirBadge('bullish').cls).toBe('pos');
    expect(dirBadge('bearish').cls).toBe('neg');
    expect(dirBadge('garbage').label).toBe('garbage');
    expect(dirBadge(null).label).toBe('—');
});

// ── patternQuality ────────────────────────────────────────────────

test('patternQuality: perfect match returns 1.0', () => {
    const match = {
        ab_ratio: 0.5, bc_ratio: 1.272, cd_to_xc_ratio: 1.5, ad_ratio: 0.786,
    };
    expect(patternQuality(match)).toBeCloseTo(1.0, 6);
});

test('patternQuality: edge-of-tolerance returns ≈ 0', () => {
    // AB at far edge (0.382 → ideal 0.5, tol 0.118) → score ≈ 0
    const match = {
        ab_ratio: 0.382, bc_ratio: 1.272, cd_to_xc_ratio: 1.5, ad_ratio: 0.786,
    };
    const q = patternQuality(match);
    // 3 perfect + 1 zero → 0.75 average
    expect(q).toBeCloseTo(0.75, 6);
});

test('patternQuality: null match returns NaN', () => {
    expect(Number.isNaN(patternQuality(null))).toBe(true);
});

// ── qualityGrade ──────────────────────────────────────────────────

test('qualityGrade buckets A/B/C/D/F at 0.85/0.70/0.50/0.30 cuts', () => {
    expect(qualityGrade(0.90).label).toBe('A');
    expect(qualityGrade(0.80).label).toBe('B');
    expect(qualityGrade(0.60).label).toBe('C');
    expect(qualityGrade(0.40).label).toBe('D');
    expect(qualityGrade(0.20).label).toBe('F');
});

test('qualityGrade non-finite → em-dash', () => {
    expect(qualityGrade(NaN).label).toBe('—');
});

test('qualityGrade A/B are pos, D/F are neg, C is empty', () => {
    expect(qualityGrade(0.95).cls).toBe('pos');
    expect(qualityGrade(0.55).cls).toBe('');
    expect(qualityGrade(0.40).cls).toBe('neg');
});

// ── makeDemoPivots ────────────────────────────────────────────────

test('makeDemoPivots returns 5 alternating-direction pivots', () => {
    const ps = makeDemoPivots();
    expect(ps.length).toBe(5);
    for (let i = 1; i < ps.length; i++) {
        expect(ps[i].is_high).not.toBe(ps[i - 1].is_high);
    }
});

test('makeDemoPivots starts with X as low (bullish setup)', () => {
    expect(makeDemoPivots()[0].is_high).toBe(false);
});

test('makeDemoPivots indices are strictly increasing', () => {
    const ps = makeDemoPivots();
    for (let i = 1; i < ps.length; i++) {
        expect(ps[i].index).toBeGreaterThan(ps[i - 1].index);
    }
});

test('makeDemoPivots C extends past A (Cypher signature)', () => {
    const ps = makeDemoPivots();
    // Bullish: A is high at index 1, C is high at index 3, C should be > A
    expect(ps[3].price).toBeGreaterThan(ps[1].price);
});

test('DEMO_TOLERANCE is the more permissive 0.10 retail setting', () => {
    expect(DEMO_TOLERANCE).toBe(0.10);
});

// ── formatters ────────────────────────────────────────────────────

test('fmtN / fmtRatio', () => {
    expect(fmtN(1.23456)).toBe('1.2346');
    expect(fmtN(NaN)).toBe('—');
    expect(fmtRatio(1.272)).toBe('1.272×');
    expect(fmtRatio(NaN)).toBe('—');
});
