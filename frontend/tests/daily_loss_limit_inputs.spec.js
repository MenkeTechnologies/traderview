// Daily Loss Limit helpers: validator, body shape (Decimal-as-string),
// local binding-limit + evaluator, state badge, demo, formatters.

import { test, expect } from 'vitest';
import {
    validateInputs, buildBody, localBindingLimit, localEvaluate,
    stateBadge, decToNum, makeDemoData,
    fmtUSD, fmtUSDSigned, fmtPct,
} from '../js/_daily_loss_limit_inputs.js';

const baseline = {
    today_pnl: -1500,
    max_daily_loss_dollars: 2000,
    max_daily_loss_pct: 0.02,
    account_equity: 100_000,
    warning_threshold: 0.50,
    cut_size_threshold: 0.75,
    kill_threshold: 1.00,
};

// ── validateInputs ────────────────────────────────────────────────

test('validate accepts canonical', () => {
    expect(validateInputs(baseline)).toBe(null);
});

test('validate rejects non-finite today_pnl', () => {
    expect(validateInputs({ ...baseline, today_pnl: NaN })).toMatch(/today_pnl/);
});

test('validate rejects negative max_daily_loss_dollars', () => {
    expect(validateInputs({ ...baseline, max_daily_loss_dollars: -1 })).toMatch(/max_daily_loss_dollars/);
});

test('validate enforces 0 ≤ pct ≤ 1 (decimal)', () => {
    expect(validateInputs({ ...baseline, max_daily_loss_pct: -0.01 })).toMatch(/max_daily_loss_pct/);
    expect(validateInputs({ ...baseline, max_daily_loss_pct: 1.5 })).toMatch(/max_daily_loss_pct/);
});

test('validate requires positive account_equity', () => {
    expect(validateInputs({ ...baseline, account_equity: 0 })).toMatch(/account_equity/);
});

test('validate enforces threshold ordering: warning < cut_size ≤ kill', () => {
    expect(validateInputs({ ...baseline, warning_threshold: 0.80, cut_size_threshold: 0.60 }))
        .toMatch(/thresholds must satisfy/);
    expect(validateInputs({ ...baseline, cut_size_threshold: 1.5, kill_threshold: 1.0 }))
        .toMatch(/thresholds must satisfy/);
});

// ── buildBody (Decimal-as-string) ────────────────────────────────

test('buildBody stringifies all Decimal scalars', () => {
    const body = buildBody(baseline);
    expect(body.today_pnl).toBe('-1500');
    expect(body.config.max_daily_loss_dollars).toBe('2000');
    expect(body.config.max_daily_loss_pct).toBe('0.02');
    expect(body.config.account_equity).toBe('100000');
});

// ── localBindingLimit ────────────────────────────────────────────

test('localBindingLimit: when $ cap < pct cap, $ binds', () => {
    expect(localBindingLimit({ ...baseline, max_daily_loss_dollars: 500 })).toBe(500);
});

test('localBindingLimit: when pct cap < $ cap, pct binds', () => {
    expect(localBindingLimit({ ...baseline, max_daily_loss_dollars: 5000 })).toBe(2000);
});

test('localBindingLimit: $ = 0 → pct-only (falls through to pct)', () => {
    expect(localBindingLimit({ ...baseline, max_daily_loss_dollars: 0 })).toBe(2000);
});

// ── localEvaluate (state matrix) ─────────────────────────────────

test('localEvaluate: profit → OK', () => {
    expect(localEvaluate({ ...baseline, today_pnl: 500 }).state).toBe('ok');
});

test('localEvaluate: zero loss → OK', () => {
    expect(localEvaluate({ ...baseline, today_pnl: 0 }).state).toBe('ok');
});

test('localEvaluate: 50% threshold edge → warning', () => {
    expect(localEvaluate({ ...baseline, today_pnl: -1000 }).state).toBe('warning');
});

test('localEvaluate: 75% threshold edge → cut_size', () => {
    expect(localEvaluate({ ...baseline, today_pnl: -1500 }).state).toBe('cut_size');
});

test('localEvaluate: 100% threshold edge → kill_switch', () => {
    expect(localEvaluate({ ...baseline, today_pnl: -2000 }).state).toBe('kill_switch');
});

test('localEvaluate: over-limit stays kill_switch', () => {
    expect(localEvaluate({ ...baseline, today_pnl: -3000 }).state).toBe('kill_switch');
});

test('localEvaluate: pct math correct (partial loss)', () => {
    const r = localEvaluate({ ...baseline, today_pnl: -500 });
    expect(r.pct).toBeCloseTo(0.25, 8);
    expect(r.loss).toBe(500);
    expect(r.limit).toBe(2000);
});

// ── stateBadge ───────────────────────────────────────────────────

test('stateBadge: ok=pos, warning=neutral, cut_size/kill_switch=neg', () => {
    expect(stateBadge('ok').cls).toBe('pos');
    expect(stateBadge('warning').cls).toBe('');
    expect(stateBadge('cut_size').cls).toBe('neg');
    expect(stateBadge('kill_switch').cls).toBe('neg');
});

test('stateBadge unknown fallthrough', () => {
    expect(stateBadge('garbage').label).toBe('garbage');
    expect(stateBadge(null).label).toBe('—');
});

// ── decToNum ─────────────────────────────────────────────────────

test('decToNum: string / number / garbage', () => {
    expect(decToNum('100.5')).toBe(100.5);
    expect(decToNum(100.5)).toBe(100.5);
    expect(decToNum(null)).toBeNaN();
    expect(decToNum('xyz')).toBeNaN();
});

// ── makeDemoData ─────────────────────────────────────────────────

test('all 5 demo presets pass validator + produce expected state', () => {
    const cases = [
        ['ok', 'ok'],
        ['warning', 'warning'],
        ['cut-size', 'cut_size'],
        ['kill', 'kill_switch'],
        ['tight', 'cut_size'],     // pct-binds at 0.5% × $100k = $500; -$400 = 80%
    ];
    for (const [kind, expected] of cases) {
        const d = makeDemoData(kind);
        expect(validateInputs(d)).toBe(null);
        expect(localEvaluate(d).state).toBe(expected);
    }
});

test('unknown demo kind falls back to OK (today_pnl = 0)', () => {
    expect(localEvaluate(makeDemoData('garbage')).state).toBe('ok');
});

// ── formatters ───────────────────────────────────────────────────

test('formatters', () => {
    expect(fmtUSD(1234.5)).toBe('$1234.50');
    expect(fmtUSD(-100)).toBe('-$100.00');
    expect(fmtUSDSigned(1500)).toBe('+$1500.00');
    expect(fmtUSDSigned(-2000)).toBe('-$2000.00');
    expect(fmtPct(0.234)).toBe('23.4%');
    expect(fmtPct(NaN)).toBe('—');
});
