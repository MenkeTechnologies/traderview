// Risk-On / Risk-Off helpers: thresholds, validator, body shape,
// localEvaluate Rust-mirror parity (one per Rust test case), signal
// breakdown, badges, demos, formatters.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, SPY_THRESHOLD, GOLD_THRESHOLD, DXY_THRESHOLD,
    YIELD_THRESHOLD_BPS, REGIME_THRESHOLD,
    validateInputs, buildBody, localEvaluate, signalBreakdown,
    regimeBadge, makeDemoSnap,
    fmtPctSigned, fmtBpsSigned, fmtScore,
    directionLabelKey, contributionClass,
} from '../js/_risk_on_off_inputs.js';

const snap = (spy, gold, dxy, yld) => ({
    spy_change_pct: spy, gold_change_pct: gold,
    dxy_change_pct: dxy, ten_year_yield_bps_change: yld,
});

// ── thresholds match Rust constants ───────────────────────────────

test('thresholds match Rust: 0.001 / 0.001 / 0.001 / 1.0 / ±2', () => {
    expect(SPY_THRESHOLD).toBe(0.001);
    expect(GOLD_THRESHOLD).toBe(0.001);
    expect(DXY_THRESHOLD).toBe(0.001);
    expect(YIELD_THRESHOLD_BPS).toBe(1.0);
    expect(REGIME_THRESHOLD).toBe(2);
});

// ── validator / buildBody ─────────────────────────────────────────

test('validate accepts finite snapshot', () => {
    expect(validateInputs(snap(0.01, 0, 0, 5))).toBe(null);
});

test('validate rejects non-finite in any field', () => {
    expect(validateInputs(snap(NaN, 0, 0, 0))).toMatch(/spy/);
    expect(validateInputs(snap(0, Infinity, 0, 0))).toMatch(/gold/);
});

test('buildBody mirrors backend flat snapshot', () => {
    const s = snap(0.01, -0.005, -0.003, 5);
    expect(buildBody(s)).toEqual(s);
});

// ── localEvaluate parity (one test per Rust test) ─────────────────

test('local: full risk-on (SPY +1%, gold -0.5%, dxy -0.3%, yields +5bps) → score +4', () => {
    const r = localEvaluate(snap(0.01, -0.005, -0.003, 5));
    expect(r.regime).toBe('on');
    expect(r.score).toBe(4);
});

test('local: full risk-off (SPY -2%, gold +1%, dxy +0.5%, yields -8bps) → score -4', () => {
    const r = localEvaluate(snap(-0.02, 0.01, 0.005, -8));
    expect(r.regime).toBe('off');
    expect(r.score).toBe(-4);
});

test('local: mixed signals (SPY +1, gold -1, dxy +1, yields -1) → score 0 → neutral', () => {
    const r = localEvaluate(snap(0.01, 0.005, -0.001, -2));
    expect(r.regime).toBe('neutral');
});

test('local: flat snapshot → neutral, score 0', () => {
    const r = localEvaluate(snap(0, 0, 0, 0));
    expect(r.regime).toBe('neutral');
    expect(r.score).toBe(0);
});

test('local: minority risk-on (1 signal) → still neutral (need ≥ ±2)', () => {
    const r = localEvaluate(snap(0.01, 0, 0, 0));
    expect(r.regime).toBe('neutral');
    expect(r.score).toBe(1);
});

test('local: majority risk-off (SPY -1, gold +1, dxy +1, yields 0) → off', () => {
    const r = localEvaluate(snap(-0.01, 0.005, 0.003, 0));
    expect(r.regime).toBe('off');
});

test('local: agreement_count excludes noisy signals below threshold', () => {
    const r = localEvaluate(snap(0.0001, -0.01, -0.005, 5));
    expect(r.agreement_count).toBe(3);
});

test('local: yields threshold is bps, not %', () => {
    const r = localEvaluate(snap(0, 0, 0, 0.5));
    expect(r.score).toBe(0);
});

// ── boundary tests ────────────────────────────────────────────────

test('local: SPY at exactly +0.001 → no signal (strict >)', () => {
    const r = localEvaluate(snap(0.001, 0, 0, 0));
    expect(r.score).toBe(0);
});

test('local: yields at exactly +1bp → no signal (strict >)', () => {
    const r = localEvaluate(snap(0, 0, 0, 1.0));
    expect(r.score).toBe(0);
});

test('local: total_signals always 4 regardless of input', () => {
    expect(localEvaluate(snap(0, 0, 0, 0)).total_signals).toBe(4);
    expect(localEvaluate(snap(1, 1, 1, 100)).total_signals).toBe(4);
});

// ── signalBreakdown ───────────────────────────────────────────────

test('signalBreakdown: emits 4 entries named spy/gold/dxy/yields', () => {
    const sigs = signalBreakdown(snap(0.01, -0.005, -0.003, 5));
    expect(sigs.map(s => s.name)).toEqual(['spy', 'gold', 'dxy', 'yields']);
});

test('signalBreakdown: SPY positive value + positive direction + positive contribution', () => {
    const [spy] = signalBreakdown(snap(0.01, 0, 0, 0));
    expect(spy.direction).toBe(1);
    expect(spy.contribution).toBe(1);
});

test('signalBreakdown: Gold up → -1 risk-on contribution (inverse sign)', () => {
    const sigs = signalBreakdown(snap(0, 0.01, 0, 0));
    const gold = sigs.find(s => s.name === 'gold');
    expect(gold.direction).toBe(1);     // gold went up
    expect(gold.contribution).toBe(-1); // bad for risk-on
});

test('signalBreakdown: noisy SPY → direction 0, contribution 0', () => {
    const sigs = signalBreakdown(snap(0.0001, 0, 0, 0));
    const spy = sigs.find(s => s.name === 'spy');
    expect(spy.direction).toBe(0);
    expect(spy.contribution).toBe(0);
});

// ── regimeBadge ───────────────────────────────────────────────────

test('regimeBadge: on/off/neutral map to pos/neg/empty class', () => {
    expect(regimeBadge('on').cls).toBe('pos');
    expect(regimeBadge('off').cls).toBe('neg');
    expect(regimeBadge('neutral').cls).toBe('');
    expect(regimeBadge('bogus').key).toMatch(/unknown/);
});

// ── demos invariants ──────────────────────────────────────────────

test('demos: every preset classifies into expected regime', () => {
    expect(localEvaluate(makeDemoSnap('full-on')).regime).toBe('on');
    expect(localEvaluate(makeDemoSnap('full-off')).regime).toBe('off');
    expect(localEvaluate(makeDemoSnap('majority-off')).regime).toBe('off');
    expect(localEvaluate(makeDemoSnap('mixed-neutral')).regime).toBe('neutral');
    expect(localEvaluate(makeDemoSnap('flat')).regime).toBe('neutral');
    expect(localEvaluate(makeDemoSnap('minority-on')).regime).toBe('neutral');
    expect(localEvaluate(makeDemoSnap('noisy-spy')).regime).toBe('on');
    expect(localEvaluate(makeDemoSnap('bond-rally')).regime).toBe('off');
});

test('demo noisy-spy: only 3 signals contribute (SPY noise-filtered)', () => {
    const r = localEvaluate(makeDemoSnap('noisy-spy'));
    expect(r.agreement_count).toBe(3);
});

// ── formatters ────────────────────────────────────────────────────

test('fmtPctSigned: + sign for positive; - for negative; "—" for non-finite', () => {
    expect(fmtPctSigned(0.0123)).toBe('+1.23%');
    expect(fmtPctSigned(-0.05, 1)).toBe('-5.0%');
    expect(fmtPctSigned(NaN)).toBe('—');
});

test('fmtBpsSigned: bps unit', () => {
    expect(fmtBpsSigned(5)).toBe('+5.0 bps');
    expect(fmtBpsSigned(-12.5)).toBe('-12.5 bps');
});

test('fmtScore: + for non-negative integers', () => {
    expect(fmtScore(0)).toBe('+0');
    expect(fmtScore(2)).toBe('+2');
    expect(fmtScore(-3)).toBe('-3');
});

test('directionLabelKey: matches view.risk_on_off.dir.* keys', () => {
    expect(directionLabelKey(1)).toBe('view.risk_on_off.dir.up');
    expect(directionLabelKey(-1)).toBe('view.risk_on_off.dir.down');
    expect(directionLabelKey(0)).toBe('view.risk_on_off.dir.flat');
});

test('contributionClass: pos/neg/empty', () => {
    expect(contributionClass(1)).toBe('pos');
    expect(contributionClass(-1)).toBe('neg');
    expect(contributionClass(0)).toBe('');
});

// ── DEFAULT_INPUTS ────────────────────────────────────────────────

test('DEFAULT_INPUTS is a sensible full-on snapshot', () => {
    expect(localEvaluate(DEFAULT_INPUTS).regime).toBe('on');
});
