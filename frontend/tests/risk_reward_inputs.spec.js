// Risk-reward helpers: validator, body shape (Decimal-as-string),
// localCompute with verbatim Rust error strings, rrBadge tiers, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, validateInputs, buildBody, localCompute, dec,
    rrBadge, makeDemoInput,
    fmtUSD, fmtUSDSigned, fmtNum, fmtPct, fmtR, fmtFraction,
} from '../js/_risk_reward_inputs.js';

const long3R = () => ({
    side: 'long', entry: 100, stop: 99, target: 103,
    risk_budget: 100, multiplier: 1,
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts well-formed long', () => {
    expect(validateInputs(long3R())).toBe(null);
});

test('validate rejects bad side', () => {
    expect(validateInputs({ ...long3R(), side: 'cover' })).toMatch(/side/);
});

test('validate rejects non-finite', () => {
    for (const k of ['entry', 'stop', 'target', 'risk_budget', 'multiplier']) {
        const bad = { ...long3R(), [k]: NaN };
        expect(validateInputs(bad)).toMatch(k);
    }
});

test('validate rejects non-positive on each numeric', () => {
    expect(validateInputs({ ...long3R(), entry: 0 })).toMatch(/entry/);
    expect(validateInputs({ ...long3R(), stop: 0 })).toMatch(/stop/);
    expect(validateInputs({ ...long3R(), target: 0 })).toMatch(/target/);
    expect(validateInputs({ ...long3R(), risk_budget: 0 })).toMatch(/risk_budget/);
    expect(validateInputs({ ...long3R(), multiplier: 0 })).toMatch(/multiplier/);
});

// ── buildBody Decimal-as-string contract ──────────────────────────

test('buildBody stringifies Decimal fields', () => {
    const body = buildBody(long3R());
    expect(body.side).toBe('long');
    expect(body.entry).toBe('100');
    expect(body.stop).toBe('99');
    expect(body.risk_budget).toBe('100');
    expect(body.multiplier).toBe('1');
});

// ── localCompute parity (one test per Rust property) ──────────────

test('local: long 3R computes r_multiple/qty/risk/reward/breakeven', () => {
    const r = localCompute(long3R());
    expect(r.ok).toBe(true);
    expect(r.report.r_multiple).toBeCloseTo(3, 9);
    expect(r.report.qty).toBeCloseTo(100, 9);
    expect(r.report.dollar_risk).toBeCloseTo(100, 9);
    expect(r.report.dollar_reward).toBeCloseTo(300, 9);
    expect(r.report.breakeven_win_rate).toBeCloseTo(0.25, 9);
});

test('local: short 3R works with inverted geometry', () => {
    const r = localCompute({ side: 'short', entry: 100, stop: 101, target: 97, risk_budget: 100, multiplier: 1 });
    expect(r.ok).toBe(true);
    expect(r.report.r_multiple).toBeCloseTo(3, 9);
    expect(r.report.qty).toBeCloseTo(100, 9);
});

test('local: zero stop distance returns verbatim Rust error', () => {
    const r = localCompute({ ...long3R(), stop: 100 });
    expect(r.ok).toBe(false);
    expect(r.error).toMatch(/stop equals entry/);
});

test('local: long with target below entry returns Rust error string', () => {
    const r = localCompute({ ...long3R(), target: 99 });
    expect(r.ok).toBe(false);
    expect(r.error).toBe('long requires target > entry > stop');
});

test('local: long with stop above entry returns Rust error string', () => {
    const r = localCompute({ ...long3R(), stop: 101 });
    expect(r.ok).toBe(false);
    expect(r.error).toBe('long requires target > entry > stop');
});

test('local: short with target above entry returns Rust error string', () => {
    const r = localCompute({ side: 'short', entry: 100, stop: 101, target: 103, risk_budget: 100, multiplier: 1 });
    expect(r.ok).toBe(false);
    expect(r.error).toBe('short requires target < entry < stop');
});

test('local: options multiplier reduces qty proportionally', () => {
    const r = localCompute({ side: 'long', entry: 5, stop: 4, target: 8, risk_budget: 100, multiplier: 100 });
    expect(r.ok).toBe(true);
    expect(r.report.qty).toBeCloseTo(1, 9);
    expect(r.report.dollar_risk).toBeCloseTo(100, 9);
    expect(r.report.dollar_reward).toBeCloseTo(300, 9);
});

test('local: breakeven_win_rate = 1 / (1 + r_multiple)', () => {
    const r2 = localCompute({ ...long3R(), target: 102 });    // 2R
    expect(r2.report.r_multiple).toBeCloseTo(2, 9);
    expect(r2.report.breakeven_win_rate).toBeCloseTo(1 / 3, 9);
});

test('local: scale-out levels at 1R / 2R / target (long)', () => {
    const r = localCompute(long3R());
    expect(r.report.scale_outs.length).toBe(3);
    expect(r.report.scale_outs[0].price).toBeCloseTo(101, 9);
    expect(r.report.scale_outs[1].price).toBeCloseTo(102, 9);
    expect(r.report.scale_outs[2].price).toBeCloseTo(103, 9);
    const total = r.report.scale_outs.reduce((s, x) => s + x.fraction, 0);
    expect(total).toBeCloseTo(1, 9);
});

test('local: short scale-outs use inverted geometry', () => {
    const r = localCompute({ side: 'short', entry: 100, stop: 101, target: 97, risk_budget: 100, multiplier: 1 });
    expect(r.report.scale_outs[0].price).toBeCloseTo(99, 9);
    expect(r.report.scale_outs[1].price).toBeCloseTo(98, 9);
    expect(r.report.scale_outs[2].price).toBeCloseTo(97, 9);
});

test('local: scale-out labels are "1R", "2R", "target"', () => {
    const r = localCompute(long3R());
    expect(r.report.scale_outs.map(s => s.label)).toEqual(['1R', '2R', 'target']);
});

// ── rrBadge tiers ─────────────────────────────────────────────────

test('rrBadge: ≤ 0 → none, < 1 → poor, < 2 → fair, < 3 → good, ≥ 3 → excellent', () => {
    expect(rrBadge(0).key).toMatch(/none/);
    expect(rrBadge(0.5).key).toMatch(/poor/);
    expect(rrBadge(1.5).key).toMatch(/fair/);
    expect(rrBadge(2.5).key).toMatch(/good/);
    expect(rrBadge(3).key).toMatch(/excellent/);
    expect(rrBadge(10).key).toMatch(/excellent/);
});

test('rrBadge: NaN → none', () => {
    expect(rrBadge(NaN).key).toMatch(/none/);
});

// ── dec coercion ──────────────────────────────────────────────────

test('dec coerces string / number / null safely', () => {
    expect(dec('123.45')).toBe(123.45);
    expect(dec(7)).toBe(7);
    expect(dec(null)).toBe(0);
    expect(dec('abc')).toBe(0);
});

// ── demos invariants ──────────────────────────────────────────────

test('demos: every preset is either ok or emits the expected error', () => {
    const expectations = {
        'long-3r':      { ok: true,  r: 3 },
        'long-1r':      { ok: true,  r: 1 },
        'long-5r':      { ok: true,  r: 5 },
        'short-3r':     { ok: true,  r: 3 },
        'options-1ct':  { ok: true,  r: 3 },
        'es-futures':   { ok: true,  r: 3 },
        'broken-long':  { ok: false, err: /long requires/ },
        'broken-short': { ok: false, err: /short requires/ },
        'zero-stop':    { ok: false, err: /stop equals/ },
    };
    for (const [k, exp] of Object.entries(expectations)) {
        const r = localCompute(makeDemoInput(k));
        expect(r.ok).toBe(exp.ok);
        if (exp.ok) expect(r.report.r_multiple).toBeCloseTo(exp.r, 6);
        else expect(r.error).toMatch(exp.err);
    }
});

test('demo es-futures: qty = 1, $500 risk, $1500 reward', () => {
    const r = localCompute(makeDemoInput('es-futures'));
    expect(r.report.qty).toBeCloseTo(1, 9);
    expect(r.report.dollar_risk).toBeCloseTo(500, 9);
    expect(r.report.dollar_reward).toBeCloseTo(1500, 9);
});

// ── DEFAULT_INPUTS / formatters ───────────────────────────────────

test('DEFAULT_INPUTS is the long-3r preset', () => {
    expect(DEFAULT_INPUTS).toEqual(long3R());
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtUSD(1234)).toBe('$1234.00');
    expect(fmtUSDSigned(-50)).toBe('-$50.00');
    expect(fmtNum(0.12345, 2)).toBe('0.12');
    expect(fmtPct(0.25, 0)).toBe('25%');
    expect(fmtR(3.5)).toBe('3.50R');
    expect(fmtFraction(1 / 3)).toBe('33.3%');
    expect(fmtUSD(NaN)).toBe('—');
    expect(fmtR(null)).toBe('—');
});
