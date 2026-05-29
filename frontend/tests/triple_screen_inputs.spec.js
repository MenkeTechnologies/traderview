// Triple Screen helpers: validator, body shape, local evaluator
// (backend parity), per-stage gate results, verdict badge, demo presets.

import { test, expect } from 'vitest';
import {
    validateInputs, buildBody, localEvaluate,
    stageResults, verdictBadge, makeDemoData, fmtN,
} from '../js/_triple_screen_inputs.js';

const ok = {
    weekly_trend: 'up',
    daily_oscillator_value: 25,
    oversold_threshold: 30,
    overbought_threshold: 70,
    intraday_breakout_up: true,
    intraday_breakout_down: false,
};

// ── validateInputs ────────────────────────────────────────────────

test('validate accepts canonical buy setup', () => {
    expect(validateInputs(ok)).toBe(null);
});

test('validate rejects bad weekly_trend enum', () => {
    expect(validateInputs({ ...ok, weekly_trend: 'sideways' })).toMatch(/weekly_trend/);
});

test('validate rejects non-finite oscillator / thresholds', () => {
    expect(validateInputs({ ...ok, daily_oscillator_value: NaN })).toMatch(/daily_oscillator_value/);
    expect(validateInputs({ ...ok, oversold_threshold: NaN })).toMatch(/oversold_threshold/);
    expect(validateInputs({ ...ok, overbought_threshold: NaN })).toMatch(/overbought_threshold/);
});

test('validate enforces overbought > oversold', () => {
    expect(validateInputs({ ...ok, oversold_threshold: 70, overbought_threshold: 30 }))
        .toMatch(/overbought_threshold/);
});

test('validate requires booleans for the intraday flags', () => {
    expect(validateInputs({ ...ok, intraday_breakout_up: 'yes' })).toMatch(/intraday_breakout_up/);
    expect(validateInputs({ ...ok, intraday_breakout_down: 1 })).toMatch(/intraday_breakout_down/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody passes the 6 input fields through unchanged', () => {
    expect(buildBody(ok)).toEqual(ok);
});

// ── localEvaluate (mirrors backend evaluate) ─────────────────────

test('localEvaluate: all 3 screens aligned UP → BUY', () => {
    expect(localEvaluate(ok)).toBe('buy');
});

test('localEvaluate: all 3 aligned DOWN → SELL', () => {
    expect(localEvaluate({
        ...ok, weekly_trend: 'down',
        daily_oscillator_value: 75,
        intraday_breakout_up: false, intraday_breakout_down: true,
    })).toBe('sell');
});

test('localEvaluate: weekly UP but oscillator not oversold → WAIT', () => {
    expect(localEvaluate({ ...ok, daily_oscillator_value: 50 })).toBe('wait');
});

test('localEvaluate: weekly UP + oversold but no intraday breakout → WAIT', () => {
    expect(localEvaluate({ ...ok, intraday_breakout_up: false })).toBe('wait');
});

test('localEvaluate: weekly NEUTRAL → always WAIT regardless of other screens', () => {
    expect(localEvaluate({ ...ok, weekly_trend: 'neutral' })).toBe('wait');
});

test('localEvaluate: weekly UP blocks SELL signal even when overbought', () => {
    expect(localEvaluate({
        ...ok, daily_oscillator_value: 80,
        intraday_breakout_up: false, intraday_breakout_down: true,
    })).toBe('wait');
});

test('localEvaluate: weekly DOWN blocks BUY signal even when oversold + intraday up', () => {
    expect(localEvaluate({ ...ok, weekly_trend: 'down' })).toBe('wait');
});

// ── stageResults ─────────────────────────────────────────────────

test('stageResults: all 3 stages pass for canonical buy setup', () => {
    const r = stageResults(ok);
    expect(r.longTide.pass).toBe(true);
    expect(r.intermediate.pass).toBe(true);
    expect(r.shortRipple.pass).toBe(true);
});

test('stageResults: longTide fails when weekly_trend = neutral', () => {
    const r = stageResults({ ...ok, weekly_trend: 'neutral' });
    expect(r.longTide.pass).toBe(false);
    expect(r.longTide.detail).toMatch(/NEUTRAL/);
});

test('stageResults: intermediate fails when not oversold in uptrend', () => {
    const r = stageResults({ ...ok, daily_oscillator_value: 50 });
    expect(r.intermediate.pass).toBe(false);
    expect(r.intermediate.detail).toMatch(/no pullback/);
});

test('stageResults: shortRipple fails when no intraday breakout up in uptrend', () => {
    const r = stageResults({ ...ok, intraday_breakout_up: false });
    expect(r.shortRipple.pass).toBe(false);
});

test('stageResults: intermediate detail varies for downtrend (overbought rally check)', () => {
    const r = stageResults({
        ...ok, weekly_trend: 'down', daily_oscillator_value: 75,
        intraday_breakout_up: false, intraday_breakout_down: true,
    });
    expect(r.intermediate.pass).toBe(true);
    expect(r.intermediate.detail).toMatch(/rally entry zone/);
});

// ── verdictBadge ─────────────────────────────────────────────────

test('verdictBadge: buy=pos, sell=neg, wait=neutral, fallthrough', () => {
    expect(verdictBadge('buy').cls).toBe('pos');
    expect(verdictBadge('sell').cls).toBe('neg');
    expect(verdictBadge('wait').cls).toBe('');
    expect(verdictBadge('garbage').label).toBe('garbage');
});

// ── makeDemoData ─────────────────────────────────────────────────

test('all 5 demo presets pass the validator + produce the expected verdict', () => {
    const cases = [
        ['buy', 'buy'],
        ['sell', 'sell'],
        ['wait-no-pullback', 'wait'],
        ['wait-no-breakout', 'wait'],
        ['wait-neutral-tide', 'wait'],
    ];
    for (const [kind, expectedVerdict] of cases) {
        const data = makeDemoData(kind);
        expect(validateInputs(data)).toBe(null);
        expect(localEvaluate(data)).toBe(expectedVerdict);
    }
});

test('unknown demo kind falls back to wait scenario', () => {
    const d = makeDemoData('garbage');
    expect(localEvaluate(d)).toBe('wait');
});

// ── fmtN ─────────────────────────────────────────────────────────

test('fmtN: 1-decimal default + non-finite', () => {
    expect(fmtN(25.456)).toBe('25.5');
    expect(fmtN(NaN)).toBe('—');
});
