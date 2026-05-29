// DeMark Pivots helpers: validator, body shape, X-base direction info,
// local X computation, trade bias, demo presets, chart span, formatters.

import { test, expect } from 'vitest';
import {
    validateInputs, buildBody, xBaseInfo, computeX,
    tradeBias, makeDemoSession, chartSpan, fmtN,
} from '../js/_demark_pivots_inputs.js';

// ── validateInputs ────────────────────────────────────────────────

const ok = { open: 102, high: 110, low: 100, close: 108 };

test('validate accepts canonical bullish session', () => {
    expect(validateInputs(ok)).toBe(null);
});

test('validate rejects each field at ≤0 or non-finite', () => {
    for (const k of Object.keys(ok)) {
        expect(validateInputs({ ...ok, [k]: 0 })).toMatch(new RegExp(k));
        expect(validateInputs({ ...ok, [k]: NaN })).toMatch(new RegExp(k));
    }
});

test('validate rejects high < low', () => {
    expect(validateInputs({ ...ok, high: 99, low: 100 })).toMatch(/high must be/);
});

test('validate rejects open / close outside [low, high]', () => {
    expect(validateInputs({ ...ok, open: 50 })).toMatch(/open/);
    expect(validateInputs({ ...ok, close: 200 })).toMatch(/close/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody wraps under "session" per backend DemarkPivotsBody', () => {
    expect(buildBody(ok)).toEqual({ session: ok });
});

// ── xBaseInfo ─────────────────────────────────────────────────────

test('xBaseInfo: bearish session (close < open) → low-heavy', () => {
    const info = xBaseInfo({ open: 108, high: 110, low: 100, close: 102 });
    expect(info.label).toMatch(/BEARISH/);
    expect(info.cls).toBe('neg');
    expect(info.formula).toBe('X = H + 2·L + C');
});

test('xBaseInfo: bullish session (close > open) → high-heavy', () => {
    const info = xBaseInfo(ok);
    expect(info.label).toMatch(/BULLISH/);
    expect(info.cls).toBe('pos');
    expect(info.formula).toBe('X = 2·H + L + C');
});

test('xBaseInfo: doji (close == open) → close-heavy', () => {
    const info = xBaseInfo({ open: 100, high: 105, low: 95, close: 100 });
    expect(info.label).toMatch(/NEUTRAL/);
    expect(info.formula).toBe('X = H + L + 2·C');
});

// ── computeX (parity with backend math) ───────────────────────────

test('computeX bearish: X = H + 2L + C', () => {
    // Backend canonical example: H=110, L=100, C=102, O=108 → X=412
    expect(computeX({ open: 108, high: 110, low: 100, close: 102 })).toBe(412);
});

test('computeX bullish: X = 2H + L + C', () => {
    // H=110, L=100, C=108, O=102 → X = 220+100+108 = 428
    expect(computeX(ok)).toBe(428);
});

test('computeX doji: X = H + L + 2C', () => {
    // H=105, L=95, C=100, O=100 → X = 105+95+200 = 400
    expect(computeX({ open: 100, high: 105, low: 95, close: 100 })).toBe(400);
});

test('computeX returns NaN on non-finite', () => {
    expect(Number.isNaN(computeX({ open: NaN, high: 1, low: 1, close: 1 }))).toBe(true);
});

// ── tradeBias ─────────────────────────────────────────────────────

const levels = { r1: 106, pivot: 103, s1: 96 };

test('tradeBias: above R1 → momentum-long bias (neg cls = resistance broken)', () => {
    const b = tradeBias(110, levels);
    expect(b.label).toBe('ABOVE R1');
    expect(b.cls).toBe('neg');
    expect(b.hint).toMatch(/breakout/);
});

test('tradeBias: below S1 → momentum-short bias', () => {
    const b = tradeBias(90, levels);
    expect(b.label).toBe('BELOW S1');
    expect(b.cls).toBe('pos');
    expect(b.hint).toMatch(/breakdown/);
});

test('tradeBias: between pivot and R1 → upper band long-to-R1', () => {
    const b = tradeBias(104, levels);
    expect(b.label).toMatch(/PIVOT → R1/);
});

test('tradeBias: between S1 and pivot → lower band short-to-S1', () => {
    const b = tradeBias(98, levels);
    expect(b.label).toMatch(/S1 → PIVOT/);
});

test('tradeBias safe on non-finite or missing levels', () => {
    expect(tradeBias(NaN, levels).label).toBe('—');
    expect(tradeBias(100, null).label).toBe('—');
});

// ── makeDemoSession ───────────────────────────────────────────────

test('all demo presets pass the validator', () => {
    for (const k of ['bullish', 'bearish', 'doji', 'inside']) {
        expect(validateInputs(makeDemoSession(k))).toBe(null);
    }
});

test('bullish demo has close > open', () => {
    const d = makeDemoSession('bullish');
    expect(d.close).toBeGreaterThan(d.open);
});

test('bearish demo has close < open', () => {
    const d = makeDemoSession('bearish');
    expect(d.close).toBeLessThan(d.open);
});

test('doji demo has close == open', () => {
    const d = makeDemoSession('doji');
    expect(d.close).toBe(d.open);
});

test('unknown demo kind falls through to inside-day default', () => {
    const d = makeDemoSession('garbage');
    // Inside-day default: open=103, high=106, low=102, close=104
    expect(d).toEqual({ open: 103, high: 106, low: 102, close: 104 });
});

// ── chartSpan ─────────────────────────────────────────────────────

test('chartSpan pads [min, max] by 5%', () => {
    const session = { open: 100, high: 110, low: 100, close: 105 };
    const lvl = { r1: 108, pivot: 105, s1: 102 };
    const span = chartSpan(session, lvl);
    // Range = 110 − 100 = 10 → pad = 0.5
    expect(span.min).toBeCloseTo(99.5, 6);
    expect(span.max).toBeCloseTo(110.5, 6);
});

test('chartSpan degenerate-equal uses pad=1', () => {
    const session = { open: 100, high: 100, low: 100, close: 100 };
    const span = chartSpan(session, null);
    expect(span.min).toBe(99);
    expect(span.max).toBe(101);
});

test('chartSpan empty fallback', () => {
    expect(chartSpan({}, null)).toEqual({ min: 0, max: 1 });
});

// ── fmtN ──────────────────────────────────────────────────────────

test('fmtN handles non-finite + digit override', () => {
    expect(fmtN(123.4567)).toBe('123.46');
    expect(fmtN(123.4567, 4)).toBe('123.4567');
    expect(fmtN(NaN)).toBe('—');
});
