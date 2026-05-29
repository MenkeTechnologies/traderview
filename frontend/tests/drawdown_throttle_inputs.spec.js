// Drawdown Throttle helpers: equity parser, validator, body shape,
// local evaluator (backend parity), active tier, rolling drawdown,
// demo invariants, formatters.

import { test, expect } from 'vitest';
import {
    parseEquity, DEFAULT_TIERS, validateInputs, buildBody,
    localEvaluate, activeTier, rollingDrawdown, multiplierCls,
    makeDemoEquity, fmtUSD, fmtPct, fmtMult,
} from '../js/_drawdown_throttle_inputs.js';

// ── parseEquity ─────────────────────────────────────────────────

test('parseEquity rejects negatives via shared nonNegative gate', () => {
    const r = parseEquity('10000\n10500\n-100\n# comment\n11000');
    expect(r.value).toEqual([10000, 10500, 11000]);
    expect(r.errors.length).toBe(1);
});

// ── DEFAULT_TIERS ────────────────────────────────────────────────

test('DEFAULT_TIERS shape matches backend defaults (5-tier ladder)', () => {
    expect(DEFAULT_TIERS.length).toBe(5);
    expect(DEFAULT_TIERS[0]).toEqual({ min_dd: 0.00, multiplier: 1.00 });
    expect(DEFAULT_TIERS[4]).toEqual({ min_dd: 0.20, multiplier: 0.10 });
});

// ── validateInputs ──────────────────────────────────────────────

const okTiers = DEFAULT_TIERS.map(t => ({ ...t }));

test('validate accepts canonical', () => {
    expect(validateInputs([10000, 11000, 10500], okTiers)).toBe(null);
});

test('validate rejects empty equity / non-positive entries', () => {
    expect(validateInputs([], okTiers)).toMatch(/at least 1/);
    expect(validateInputs([10000, 0, 11000], okTiers)).toMatch(/all equity values must be > 0/);
    expect(validateInputs([10000, -100, 11000], okTiers)).toMatch(/all equity values/);
});

test('validate rejects empty tiers', () => {
    expect(validateInputs([10000], [])).toMatch(/at least 1 tier/);
});

test('validate rejects tier min_dd out of [0, 1]', () => {
    expect(validateInputs([10000], [{ min_dd: -0.01, multiplier: 1 }])).toMatch(/min_dd/);
    expect(validateInputs([10000], [{ min_dd: 1.5, multiplier: 1 }])).toMatch(/min_dd/);
});

test('validate rejects tier multiplier out of [0, 5]', () => {
    expect(validateInputs([10000], [{ min_dd: 0, multiplier: -0.1 }])).toMatch(/multiplier/);
    expect(validateInputs([10000], [{ min_dd: 0, multiplier: 10 }])).toMatch(/multiplier/);
});

test('validate enforces tier ordering ascending by min_dd', () => {
    expect(validateInputs([10000], [
        { min_dd: 0.10, multiplier: 0.5 },
        { min_dd: 0.05, multiplier: 0.75 },
    ])).toMatch(/sorted ascending/);
});

// ── buildBody ────────────────────────────────────────────────────

test('buildBody emits backend DdThrottleBody shape', () => {
    expect(buildBody([10000, 11000], okTiers)).toEqual({
        equity_history: [10000, 11000],
        config: { tiers: okTiers },
    });
});

// ── localEvaluate (mirrors backend) ─────────────────────────────

test('localEvaluate empty history → multiplier 1.0', () => {
    expect(localEvaluate([], okTiers).active_multiplier).toBe(1.0);
});

test('localEvaluate flat equity → no throttle', () => {
    const r = localEvaluate([10000, 10000, 10000], okTiers);
    expect(r.drawdown_pct).toBe(0);
    expect(r.active_multiplier).toBe(1.0);
});

test('localEvaluate at peak → no throttle', () => {
    const r = localEvaluate([10000, 11000, 12000], okTiers);
    expect(r.drawdown_pct).toBe(0);
    expect(r.active_multiplier).toBe(1.0);
});

test('localEvaluate 3% DD → 1.0x (under 5% tier)', () => {
    const r = localEvaluate([10000, 9700], okTiers);
    expect(r.drawdown_pct).toBeCloseTo(0.03, 8);
    expect(r.active_multiplier).toBe(1.0);
});

test('localEvaluate 7% DD → 0.75x (5% tier)', () => {
    const r = localEvaluate([10000, 9300], okTiers);
    expect(r.active_multiplier).toBe(0.75);
    expect(r.tier_min_dd).toBe(0.05);
});

test('localEvaluate 12% DD → 0.50x (10% tier)', () => {
    expect(localEvaluate([10000, 8800], okTiers).active_multiplier).toBe(0.50);
});

test('localEvaluate 17% DD → 0.25x (15% tier)', () => {
    expect(localEvaluate([10000, 8300], okTiers).active_multiplier).toBe(0.25);
});

test('localEvaluate 25% DD → 0.10x (20%+ tier)', () => {
    expect(localEvaluate([10000, 7500], okTiers).active_multiplier).toBe(0.10);
});

test('localEvaluate exact-5%-boundary uses 0.75 (inclusive lower bound)', () => {
    expect(localEvaluate([10000, 9500], okTiers).active_multiplier).toBe(0.75);
});

test('localEvaluate recovery above old peak resets DD to 0', () => {
    const r = localEvaluate([10000, 9000, 11000], okTiers);
    expect(r.drawdown_pct).toBe(0);
    expect(r.active_multiplier).toBe(1.0);
});

// ── activeTier ──────────────────────────────────────────────────

test('activeTier picks largest min_dd that DD ≥', () => {
    expect(activeTier(okTiers, 0.07).min_dd).toBe(0.05);
    expect(activeTier(okTiers, 0.12).min_dd).toBe(0.10);
    expect(activeTier(okTiers, 0.0).min_dd).toBe(0.0);
});

test('activeTier null on empty', () => {
    expect(activeTier([], 0.1)).toBe(null);
});

// ── rollingDrawdown ─────────────────────────────────────────────

test('rollingDrawdown emits per-bar peak + signed-negative DD', () => {
    const { peaks, dds } = rollingDrawdown([10000, 11000, 10500, 12000]);
    expect(peaks).toEqual([10000, 11000, 11000, 12000]);
    // dds are -((peak - current) / peak)
    expect(dds[0]).toBeCloseTo(0, 10);
    expect(dds[1]).toBeCloseTo(0, 10);
    expect(dds[2]).toBeCloseTo(-(500 / 11000), 8);
    expect(dds[3]).toBeCloseTo(0, 10);
});

test('rollingDrawdown non-array safe', () => {
    expect(rollingDrawdown(null)).toEqual({ peaks: [], dds: [] });
});

// ── multiplierCls ───────────────────────────────────────────────

test('multiplierCls: ≥0.95=pos / ≥0.50=neutral / <0.50=neg', () => {
    expect(multiplierCls(1.0)).toBe('pos');
    expect(multiplierCls(0.75)).toBe('');
    expect(multiplierCls(0.50)).toBe('');
    expect(multiplierCls(0.25)).toBe('neg');
    expect(multiplierCls(NaN)).toBe('');
});

// ── makeDemoEquity ──────────────────────────────────────────────

test('makeDemoEquity: 50-bar curve with peak around index 25', () => {
    const eq = makeDemoEquity('mid');
    expect(eq.length).toBe(50);
    const peak = Math.max(...eq);
    const peakIdx = eq.indexOf(peak);
    expect(peakIdx).toBeGreaterThan(15);
    expect(peakIdx).toBeLessThan(35);
});

test('makeDemoEquity each preset lands in its target tier', () => {
    const cases = [
        ['shallow', 1.00],   // 3% < 5%
        ['mild',    0.75],   // 7%
        ['mid',     0.50],   // 12%
        ['deep',    0.25],   // 17%
        ['crisis',  0.10],   // 25%
    ];
    for (const [kind, expectedMult] of cases) {
        const eq = makeDemoEquity(kind);
        const r = localEvaluate(eq, DEFAULT_TIERS);
        expect(r.active_multiplier).toBe(expectedMult);
    }
});

// ── formatters ──────────────────────────────────────────────────

test('formatters', () => {
    expect(fmtUSD(1234)).toBe('$1234');
    expect(fmtUSD(-100)).toBe('-$100');
    expect(fmtPct(0.0712)).toBe('7.12%');
    expect(fmtMult(0.75)).toBe('0.75x');
});
