// ATR-cone helpers: validator, body shape, localProject Rust-mirror,
// width helpers, daysToReachOffset, badges, demos.

import { test, expect } from 'vitest';
import {
    MAX_HORIZON_DAYS, DEFAULT_INPUTS,
    validateInputs, buildBody, localProject,
    widthAtHorizon, widthPctAtHorizon, noiseBadge, daysToReachOffset,
    makeDemoInput, fmtUSD, fmtUSDSigned, fmtPct, fmtDays,
} from '../js/_atr_cone_inputs.js';

// ── constants ─────────────────────────────────────────────────────

test('MAX_HORIZON_DAYS = 1000 (matches Rust cap)', () => {
    expect(MAX_HORIZON_DAYS).toBe(1000);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts default', () => {
    expect(validateInputs(DEFAULT_INPUTS)).toBe(null);
});

test('validate rejects non-finite / non-positive entry / negative atr / bad horizon', () => {
    expect(validateInputs({ ...DEFAULT_INPUTS, entry: NaN })).toMatch(/entry/);
    expect(validateInputs({ ...DEFAULT_INPUTS, entry: 0 })).toMatch(/entry/);
    expect(validateInputs({ ...DEFAULT_INPUTS, daily_atr: -1 })).toMatch(/daily_atr/);
    expect(validateInputs({ ...DEFAULT_INPUTS, horizon_days: -1 })).toMatch(/horizon_days/);
    expect(validateInputs({ ...DEFAULT_INPUTS, horizon_days: 1.5 })).toMatch(/horizon_days/);
});

test('validate accepts daily_atr=0 (flat-cone edge case)', () => {
    expect(validateInputs({ ...DEFAULT_INPUTS, daily_atr: 0 })).toBe(null);
});

test('validate accepts horizon_days=0 (entry-only point)', () => {
    expect(validateInputs({ ...DEFAULT_INPUTS, horizon_days: 0 })).toBe(null);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody flat-passes the 3 inputs', () => {
    expect(buildBody(DEFAULT_INPUTS)).toEqual({
        entry: 100, daily_atr: 2, horizon_days: 20,
    });
});

// ── localProject parity (one test per Rust property) ─────────────

test('local: zero horizon collapses all bands to entry (single point)', () => {
    const out = localProject(100, 2, 0);
    expect(out.length).toBe(1);
    expect(out[0]).toEqual({
        days_forward: 0, upper_2sd: 100, upper_1sd: 100,
        center: 100, lower_1sd: 100, lower_2sd: 100,
    });
});

test('local: day 1 σ = ATR (√1 = 1)', () => {
    const out = localProject(100, 2, 1);
    expect(out[1].upper_1sd).toBe(102);
    expect(out[1].lower_1sd).toBe(98);
    expect(out[1].upper_2sd).toBe(104);
    expect(out[1].lower_2sd).toBe(96);
});

test('local: day 4 σ = 2×ATR (√4 = 2)', () => {
    const out = localProject(100, 2, 4);
    expect(out[4].upper_1sd).toBe(104);
    expect(out[4].lower_2sd).toBe(92);
});

test('local: cone widens monotonically', () => {
    const out = localProject(100, 2, 10);
    for (let i = 1; i < out.length; i++) {
        expect(out[i].upper_2sd).toBeGreaterThan(out[i - 1].upper_2sd);
        expect(out[i].lower_2sd).toBeLessThan(out[i - 1].lower_2sd);
    }
});

test('local: cone symmetric around entry (upper/lower offsets equal)', () => {
    const out = localProject(100, 2, 10);
    for (const p of out) {
        expect(p.upper_2sd - p.center).toBeCloseTo(p.center - p.lower_2sd, 12);
        expect(p.upper_1sd - p.center).toBeCloseTo(p.center - p.lower_1sd, 12);
    }
});

test('local: zero ATR → flat cone forever', () => {
    const out = localProject(100, 0, 10);
    for (const p of out) {
        expect(p.upper_2sd).toBe(100);
        expect(p.lower_2sd).toBe(100);
    }
});

test('local: larger ATR widens cone proportionally', () => {
    const small = localProject(100, 1, 10);
    const big   = localProject(100, 2, 10);
    for (let d = 1; d <= 10; d++) {
        const smallOffset = small[d].upper_1sd - 100;
        const bigOffset   = big[d].upper_1sd - 100;
        expect(bigOffset / smallOffset).toBeCloseTo(2, 9);
    }
});

test('local: series length = horizon + 1; days_forward 0..horizon', () => {
    const out = localProject(100, 1, 30);
    expect(out.length).toBe(31);
    expect(out[0].days_forward).toBe(0);
    expect(out[30].days_forward).toBe(30);
});

test('local: huge horizon capped at MAX_HORIZON_DAYS (no OOM)', () => {
    const out = localProject(100, 1, 100_000);
    expect(out.length).toBe(MAX_HORIZON_DAYS + 1);
    expect(out[out.length - 1].days_forward).toBe(MAX_HORIZON_DAYS);
});

test('local: 10_000 horizon silently truncates to 1_001 entries', () => {
    expect(localProject(100, 1, 10_000).length).toBe(MAX_HORIZON_DAYS + 1);
});

// ── widthAtHorizon / widthPctAtHorizon ────────────────────────────

test('widthAtHorizon: 2 × ATR × √N', () => {
    expect(widthAtHorizon(2, 4)).toBe(8);   // 2 × 2 × 2
    expect(widthAtHorizon(2, 9)).toBe(12);  // 2 × 2 × 3
    expect(widthAtHorizon(0, 100)).toBe(0);
});

test('widthAtHorizon: caps at MAX_HORIZON_DAYS', () => {
    expect(widthAtHorizon(1, 100_000)).toBeCloseTo(2 * Math.sqrt(MAX_HORIZON_DAYS), 6);
});

test('widthAtHorizon: bad inputs → NaN', () => {
    expect(widthAtHorizon(NaN, 10)).toBeNaN();
    expect(widthAtHorizon(2, -1)).toBeNaN();
});

test('widthPctAtHorizon: width / entry', () => {
    // ATR 5, horizon 4, entry 100 → width 20, pct 0.20.
    expect(widthPctAtHorizon(100, 5, 4)).toBeCloseTo(0.20, 9);
});

test('widthPctAtHorizon: bad entry → NaN', () => {
    expect(widthPctAtHorizon(0, 5, 4)).toBeNaN();
});

// ── noiseBadge tiers ─────────────────────────────────────────────

test('noiseBadge: flat / quiet / normal / loud / extreme by width%', () => {
    // pct = 2×ATR×√N / entry
    expect(noiseBadge(100, 0, 5).key).toMatch(/flat/);             // pct = 0
    expect(noiseBadge(100, 0.4, 1).key).toMatch(/quiet/);          // pct = 0.008 (<2%)
    expect(noiseBadge(100, 2, 1).key).toMatch(/normal/);           // pct = 0.04
    expect(noiseBadge(100, 4, 1).key).toMatch(/loud/);             // pct = 0.08
    expect(noiseBadge(100, 10, 1).key).toMatch(/extreme/);         // pct = 0.20
});

test('noiseBadge: bad inputs → unknown', () => {
    expect(noiseBadge(0, 5, 5).key).toMatch(/unknown/);
});

// ── daysToReachOffset ─────────────────────────────────────────────

test('daysToReachOffset: N = (offset / ATR)²', () => {
    expect(daysToReachOffset(2, 2)).toBe(1);   // (2/2)² = 1
    expect(daysToReachOffset(2, 4)).toBe(4);   // (4/2)² = 4
    expect(daysToReachOffset(1, 5)).toBe(25);  // (5/1)² = 25
});

test('daysToReachOffset: offset = 0 → 0 days', () => {
    expect(daysToReachOffset(2, 0)).toBe(0);
});

test('daysToReachOffset: ATR = 0 → Infinity (never reaches)', () => {
    expect(daysToReachOffset(0, 5)).toBe(Infinity);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset is valid input', () => {
    for (const k of ['spy-normal', 'aapl-medium', 'tsla-loud', 'penny-extreme',
                     'long-horizon', 'zero-atr', 'huge-horizon', 'es-futures']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
    }
});

test('demo huge-horizon: localProject caps at MAX_HORIZON_DAYS+1', () => {
    const inp = makeDemoInput('huge-horizon');
    expect(localProject(inp.entry, inp.daily_atr, inp.horizon_days).length).toBe(MAX_HORIZON_DAYS + 1);
});

test('demo penny-extreme: badge = extreme (20% width at day 5)', () => {
    const inp = makeDemoInput('penny-extreme');
    expect(noiseBadge(inp.entry, inp.daily_atr, inp.horizon_days).key).toMatch(/extreme/);
});

test('demo zero-atr: badge = flat', () => {
    const inp = makeDemoInput('zero-atr');
    expect(noiseBadge(inp.entry, inp.daily_atr, inp.horizon_days).key).toMatch(/flat/);
});

// ── formatters ────────────────────────────────────────────────────

test('fmt helpers + non-finite guards', () => {
    expect(fmtUSD(1234.5)).toBe('$1234.50');
    expect(fmtUSDSigned(-100)).toBe('-$100.00');
    expect(fmtPct(0.20)).toBe('20.00%');
    expect(fmtDays(2.5)).toBe('2.50 d');
    expect(fmtUSD(NaN)).toBe('—');
    expect(fmtDays(null)).toBe('—');
});
