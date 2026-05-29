// Goal Tracker helpers: parser, validator, body shape, local evaluator
// (backend parity), pace badge, demo invariants, formatters.

import { test, expect } from 'vitest';
import {
    parseEquity, validateInputs, buildBody, localEvaluate, paceBadge,
    makeDemoData, todayIso, fmtUSD, fmtPct,
} from '../js/_goal_tracker_inputs.js';

const baseline = {
    period_start_equity: 100_000,
    target_pct_return: 0.30,
    max_dd_pct: 0.10,
    period_start: '2026-01-01',
    period_end: '2026-12-31',
    today: '2026-06-30',
    equity: [100_000, 110_000, 115_000],
};

// ── validateInputs ────────────────────────────────────────────────

test('validate accepts canonical', () => {
    expect(validateInputs(baseline)).toBe(null);
});

test('validate rejects non-positive period_start_equity', () => {
    expect(validateInputs({ ...baseline, period_start_equity: 0 })).toMatch(/period_start_equity/);
});

test('validate enforces max_dd_pct in [0, 1]', () => {
    expect(validateInputs({ ...baseline, max_dd_pct: -0.01 })).toMatch(/max_dd_pct/);
    expect(validateInputs({ ...baseline, max_dd_pct: 1.5 })).toMatch(/max_dd_pct/);
});

test('validate rejects malformed dates', () => {
    expect(validateInputs({ ...baseline, period_start: '2026/01/01' })).toMatch(/period_start/);
    expect(validateInputs({ ...baseline, period_end: 'tomorrow' })).toMatch(/period_end/);
    expect(validateInputs({ ...baseline, today: '06-30-2026' })).toMatch(/today/);
});

test('validate enforces period_end > period_start', () => {
    expect(validateInputs({ ...baseline, period_end: '2025-12-31' }))
        .toMatch(/period_end must be after period_start/);
});

test('validate rejects empty / non-positive equity', () => {
    expect(validateInputs({ ...baseline, equity: [] })).toMatch(/at least 1 equity/);
    expect(validateInputs({ ...baseline, equity: [100, 0, 110] })).toMatch(/all equity values/);
});

// ── buildBody ────────────────────────────────────────────────────

test('buildBody emits backend GoalTrackerBody shape', () => {
    const body = buildBody(baseline);
    expect(body.goals.period_start_equity).toBe(100_000);
    expect(body.goals.period_start).toBe('2026-01-01');
    expect(body.equity_history).toEqual(baseline.equity);
    expect(body.today).toBe('2026-06-30');
});

// ── localEvaluate (backend parity) ──────────────────────────────

test('localEvaluate empty equity returns zeros', () => {
    const r = localEvaluate({ ...baseline, equity: [] });
    expect(r.current_equity).toBe(0);
    expect(r.current_pct_return).toBe(0);
});

test('localEvaluate on-pace at mid-year with 15% return on 30% target', () => {
    const r = localEvaluate(baseline);
    expect(r.current_pct_return).toBeCloseTo(0.15, 8);
    expect(r.on_pace).toBe('on_pace');
});

test('localEvaluate ahead-of-pace when well above proportional target', () => {
    const r = localEvaluate({ ...baseline, equity: [100_000, 130_000] });
    expect(r.on_pace).toBe('ahead_of_pace');
    expect(r.pct_of_target).toBeGreaterThan(0.9);
});

test('localEvaluate behind-of-pace when well below proportional target', () => {
    const r = localEvaluate({ ...baseline, equity: [100_000, 102_000] });
    expect(r.on_pace).toBe('behind_pace');
});

test('localEvaluate kill-switch breach at DD > max_dd_pct', () => {
    // Peak 120k, current 100k → DD = 16.7% > 10%.
    const r = localEvaluate({ ...baseline, equity: [100_000, 120_000, 100_000] });
    expect(r.kill_switch_breached).toBe(true);
    expect(r.current_dd_pct).toBeGreaterThan(0.10);
});

test('localEvaluate kill-switch NOT breached when DD ≤ limit', () => {
    const r = localEvaluate({ ...baseline, equity: [100_000, 110_000, 105_000] });
    expect(r.kill_switch_breached).toBe(false);
});

test('localEvaluate out-of-period when today is before period_start', () => {
    const r = localEvaluate({ ...baseline, today: '2025-12-31' });
    expect(r.on_pace).toBe('out_of_period');
});

test('localEvaluate out-of-period when today is after period_end', () => {
    const r = localEvaluate({ ...baseline, today: '2027-01-01' });
    expect(r.on_pace).toBe('out_of_period');
});

test('localEvaluate annualized pace extrapolates from partial year', () => {
    // Mid-year (180 days), 15% gain → annualized ≈ 30%.
    const r = localEvaluate(baseline);
    expect(r.annualized_pace).toBeGreaterThan(0.28);
    expect(r.annualized_pace).toBeLessThan(0.32);
});

// ── paceBadge ────────────────────────────────────────────────────

test('paceBadge: 4 enums with hints', () => {
    expect(paceBadge('ahead_of_pace').cls).toBe('pos');
    expect(paceBadge('on_pace').cls).toBe('pos');
    expect(paceBadge('behind_pace').cls).toBe('neg');
    expect(paceBadge('out_of_period').cls).toBe('');
    expect(paceBadge('garbage').label).toBe('garbage');
});

// ── makeDemoData ─────────────────────────────────────────────────

test('5 demo presets land in their expected on_pace state', () => {
    const cases = [
        ['ahead', 'ahead_of_pace'],
        ['on-pace', 'on_pace'],
        ['behind', 'behind_pace'],
        ['out-of-period', 'out_of_period'],
        // kill-switch produces a breached flag but the on_pace state still
        // reflects period progress; we test the flag separately below.
    ];
    for (const [kind, expected] of cases) {
        const d = makeDemoData(kind);
        expect(validateInputs(d)).toBe(null);
        expect(localEvaluate(d).on_pace).toBe(expected);
    }
});

test('kill-switch preset has kill_switch_breached=true', () => {
    const r = localEvaluate(makeDemoData('kill-switch'));
    expect(r.kill_switch_breached).toBe(true);
});

// ── todayIso ─────────────────────────────────────────────────────

test('todayIso returns YYYY-MM-DD', () => {
    const s = todayIso();
    expect(/^\d{4}-\d{2}-\d{2}$/.test(s)).toBe(true);
});

// ── formatters ───────────────────────────────────────────────────

test('fmtUSD / fmtPct', () => {
    expect(fmtUSD(115_000)).toBe('$115000');
    expect(fmtUSD(-500)).toBe('-$500');
    expect(fmtPct(0.15)).toBe('+15.00%');
    expect(fmtPct(-0.05, 1)).toBe('-5.0%');
    expect(fmtPct(NaN)).toBe('—');
});
