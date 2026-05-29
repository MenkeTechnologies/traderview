// Equity-Regime helpers: parser, validator, body shape, local
// classifier (parity with Rust), badges, demo presets, formatters.

import { test, expect } from 'vitest';
import {
    DEFAULT_CONFIG, parseEquityBlob, validateInputs, buildBody, localEvaluate,
    regimeBadge, fitLine, makeDemoEquity,
    fmtUSD, fmtUSDSigned, fmtPct, fmtNum,
} from '../js/_regime_equity_inputs.js';

const cfg = () => ({ ...DEFAULT_CONFIG });

// ── parseEquityBlob ───────────────────────────────────────────────

test('parseEquityBlob accepts newline / CSV / whitespace mixes', () => {
    const r = parseEquityBlob('10000\n10100,10200 10300\n10400');
    expect(r.errors).toEqual([]);
    expect(r.equity).toEqual([10000, 10100, 10200, 10300, 10400]);
});

test('parseEquityBlob flags non-numeric tokens', () => {
    const r = parseEquityBlob('100,abc,200');
    expect(r.errors.length).toBe(1);
    expect(r.errors[0].message).toMatch(/abc/);
    expect(r.equity).toEqual([100, 200]);
});

test('parseEquityBlob non-string returns single error', () => {
    expect(parseEquityBlob(null).errors.length).toBe(1);
});

test('parseEquityBlob accepts negative values (margin call account)', () => {
    expect(parseEquityBlob('-50,100,200').equity).toEqual([-50, 100, 200]);
});

// ── validateInputs ────────────────────────────────────────────────

test('validate accepts ≥ 3 finite points', () => {
    expect(validateInputs([1, 2, 3], cfg())).toBe(null);
});

test('validate rejects < 3 points', () => {
    expect(validateInputs([1, 2], cfg())).toMatch(/≥ 3/);
});

test('validate rejects non-array equity', () => {
    expect(validateInputs(null, cfg())).toMatch(/array/);
});

test('validate rejects NaN / Inf in equity', () => {
    expect(validateInputs([1, 2, NaN], cfg())).toMatch(/finite/);
    expect(validateInputs([1, 2, Infinity], cfg())).toMatch(/finite/);
});

test('validate rejects bad config', () => {
    expect(validateInputs([1, 2, 3], { ...cfg(), trend_slope_pct: -1 })).toMatch(/trend_slope/);
    expect(validateInputs([1, 2, 3], { ...cfg(), clean_trend_rel_stdev: NaN })).toMatch(/clean_trend/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody returns { equity[], config{} } shape', () => {
    const eq = [1, 2, 3];
    const body = buildBody(eq, cfg());
    expect(body).toEqual({ equity: [1, 2, 3], config: cfg() });
    // Defensive copy: mutating source should not affect body.
    eq.push(99);
    expect(body.equity.length).toBe(3);
});

// ── localEvaluate ─────────────────────────────────────────────────

test('localEvaluate: n < 3 → choppy default', () => {
    const r = localEvaluate([1, 2], cfg());
    expect(r.regime).toBe('choppy');
    expect(r.n).toBe(2);
});

test('localEvaluate: perfect line ⇒ R² = 1.0, slope = step', () => {
    const eq = Array.from({ length: 30 }, (_, i) => 10_000 + i * 100);
    const r = localEvaluate(eq, cfg());
    expect(r.r_squared).toBeCloseTo(1, 9);
    expect(r.slope_per_period).toBeCloseTo(100, 9);
    expect(r.regime).toBe('trending_up');
});

test('localEvaluate: steady downtrend classified trending_down', () => {
    const eq = Array.from({ length: 30 }, (_, i) => 20_000 - i * 120);
    const r = localEvaluate(eq, cfg());
    expect(r.slope_per_period).toBeLessThan(0);
    expect(r.regime).toBe('trending_down');
});

test('localEvaluate: tiny micro-noise (rel_slope < threshold) → choppy', () => {
    const eq = Array.from({ length: 30 }, (_, i) => 10_000 + ((i * 7) % 5));
    const r = localEvaluate(eq, cfg());
    expect(r.regime).toBe('choppy');
});

test('localEvaluate: positive slope + noisy → volatile_up', () => {
    const eq = makeDemoEquity('volatile-up');
    const r = localEvaluate(eq, cfg());
    expect(r.slope_per_period).toBeGreaterThan(0);
    // demo deliberately exceeds clean_trend_rel_stdev
    expect(['volatile_up', 'trending_up']).toContain(r.regime);
});

test('localEvaluate: lax config promotes tiny slope to trending_up', () => {
    const eq = Array.from({ length: 30 }, (_, i) => 10_000 + i * 0.01);
    const lax = { trend_slope_pct: 0.0000001, clean_trend_rel_stdev: 1.0 };
    expect(localEvaluate(eq, lax).regime).toBe('trending_up');
});

test('localEvaluate: intercept + slope reconstructs first/last fit values', () => {
    const eq = Array.from({ length: 30 }, (_, i) => 10_000 + i * 100);
    const r = localEvaluate(eq, cfg());
    // First fit value = intercept; last = intercept + slope*(n-1)
    expect(r.intercept).toBeCloseTo(10_000, 6);
    expect(r.intercept + r.slope_per_period * (eq.length - 1)).toBeCloseTo(10_000 + 29 * 100, 6);
});

test('localEvaluate: residual_stdev is 0 on a perfect line', () => {
    const eq = Array.from({ length: 30 }, (_, i) => 10_000 + i * 100);
    expect(localEvaluate(eq, cfg()).residual_stdev).toBeCloseTo(0, 9);
});

test('localEvaluate: mean_equity=0 edge case → choppy (rel_slope=0)', () => {
    const eq = [-1, 0, 1, 0, -1, 0, 1]; // sum=0, mean=0
    const r = localEvaluate(eq, cfg());
    expect(r.mean_equity).toBeCloseTo(0, 9);
    expect(r.regime).toBe('choppy');
});

// Parity: every demo preset's local classification must equal what the
// preset name claims (these would catch JS/Rust algorithm drift).

test('demo presets self-classify into their named regime', () => {
    const c = cfg();
    expect(localEvaluate(makeDemoEquity('trending-up'),   c).regime).toBe('trending_up');
    expect(localEvaluate(makeDemoEquity('trending-down'), c).regime).toBe('trending_down');
    expect(localEvaluate(makeDemoEquity('volatile-up'),   c).regime).toMatch(/^(volatile_up|trending_up)$/);
    expect(localEvaluate(makeDemoEquity('volatile-down'), c).regime).toMatch(/^(volatile_down|trending_down)$/);
    expect(localEvaluate(makeDemoEquity('choppy'),        c).regime).toBe('choppy');
});

// ── presentation helpers ──────────────────────────────────────────

test('regimeBadge maps every known regime to a label + class', () => {
    expect(regimeBadge('trending_up').cls).toBe('pos');
    expect(regimeBadge('trending_down').cls).toBe('neg');
    expect(regimeBadge('volatile_up').cls).toBe('pos');
    expect(regimeBadge('volatile_down').cls).toBe('neg');
    expect(regimeBadge('choppy').cls).toBe('');
    expect(regimeBadge('unknown_state').label).toBe('UNKNOWN_STATE');
});

test('fitLine emits one y-value per equity sample', () => {
    const eq = [10, 20, 30, 40];
    const local = localEvaluate(eq, cfg());
    const line = fitLine(eq, local);
    expect(line.length).toBe(4);
    // For perfect line, fit == data.
    line.forEach((y, i) => expect(y).toBeCloseTo(eq[i], 9));
});

test('fitLine empty input returns empty array', () => {
    expect(fitLine([], localEvaluate([], cfg()))).toEqual([]);
});

test('fmt helpers: USD / signed / pct / num + non-finite guards', () => {
    expect(fmtUSD(1234.56, 2)).toBe('$1234.56');
    expect(fmtUSD(-50, 0)).toBe('-$50');
    expect(fmtUSDSigned(100, 0)).toBe('+$100');
    expect(fmtUSDSigned(-100, 0)).toBe('-$100');
    expect(fmtPct(0.05, 2)).toBe('5.00%');
    expect(fmtNum(0.1234, 2)).toBe('0.12');
    expect(fmtUSD(NaN)).toBe('—');
    expect(fmtPct(NaN)).toBe('—');
    expect(fmtNum(Infinity)).toBe('—');
});

test('DEFAULT_CONFIG matches backend (0.001 / 0.02)', () => {
    expect(DEFAULT_CONFIG).toEqual({
        trend_slope_pct: 0.001, clean_trend_rel_stdev: 0.02,
    });
});
