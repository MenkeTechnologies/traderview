// Option-payoff view pure helpers — presets, leg validation, body
// shaping, default spot-range. The view itself touches the DOM and is
// not unit-testable without jsdom; everything that CAN be tested in
// isolation lives in _option_strategy_presets.js.
//
// Bugs this guards against:
//   * Preset payloads with missing fields (NaN strike/qty → backend rejects).
//   * Validator passing zero-qty legs (would emit a no-op leg into payload).
//   * defaultSpotRange producing min ≥ max on edge cases (chart crashes).
//   * Iron-condor / iron-butterfly leg counts changing inadvertently.

import { test, expect } from 'vitest';
import {
    PRESETS, validateLeg, validateLegs,
    buildPayoffBody, buildPricerBody, defaultSpotRange,
} from '../js/_option_strategy_presets.js';

test('every preset returns at least one leg', () => {
    for (const id of Object.keys(PRESETS)) {
        const legs = PRESETS[id](100);
        expect(legs.length).toBeGreaterThanOrEqual(1);
    }
});

test('iron_condor has exactly 4 legs', () => {
    expect(PRESETS.iron_condor(100).length).toBe(4);
});

test('iron_butterfly has exactly 4 legs', () => {
    expect(PRESETS.iron_butterfly(100).length).toBe(4);
});

test('long_straddle has matching call and put strikes', () => {
    const legs = PRESETS.long_straddle(100);
    expect(legs[0].strike).toBe(legs[1].strike);
    expect(legs[0].kind).toBe('call');
    expect(legs[1].kind).toBe('put');
});

test('bull_call_spread is long lower-strike, short higher-strike', () => {
    const legs = PRESETS.bull_call_spread(100);
    expect(legs[0].qty).toBe(1);
    expect(legs[1].qty).toBe(-1);
    expect(legs[0].strike).toBeLessThan(legs[1].strike);
});

test('covered_call legs: long stock + short call', () => {
    const legs = PRESETS.covered_call(100);
    expect(legs[0].kind).toBe('underlying');
    expect(legs[0].qty).toBe(1);
    expect(legs[1].kind).toBe('call');
    expect(legs[1].qty).toBe(-1);
});

test('all preset legs pass validateLeg', () => {
    for (const id of Object.keys(PRESETS)) {
        for (const leg of PRESETS[id](100)) {
            expect(validateLeg(leg)).toBe(null);
        }
    }
});

test('validateLeg rejects bad kind', () => {
    expect(validateLeg({ kind: 'forward', strike: 100, premium: 5, qty: 1 }))
        .toMatch(/bad kind/);
});

test('validateLeg rejects non-finite strike / premium / qty', () => {
    expect(validateLeg({ kind: 'call', strike: NaN, premium: 5, qty: 1 }))
        .toMatch(/strike/);
    expect(validateLeg({ kind: 'call', strike: 100, premium: Infinity, qty: 1 }))
        .toMatch(/premium/);
    expect(validateLeg({ kind: 'call', strike: 100, premium: 5, qty: NaN }))
        .toMatch(/qty/);
});

test('validateLeg rejects zero qty', () => {
    expect(validateLeg({ kind: 'call', strike: 100, premium: 5, qty: 0 }))
        .toMatch(/qty/);
});

test('validateLeg rejects non-positive strike', () => {
    expect(validateLeg({ kind: 'call', strike: 0, premium: 5, qty: 1 }))
        .toMatch(/strike/);
    expect(validateLeg({ kind: 'put', strike: -1, premium: 5, qty: 1 }))
        .toMatch(/strike/);
});

test('validateLegs rejects empty list', () => {
    expect(validateLegs([])).toMatch(/at least one leg/);
});

test('validateLegs reports the index of the first bad leg', () => {
    const r = validateLegs([
        { kind: 'call', strike: 100, premium: 5, qty: 1 },
        { kind: 'call', strike: 100, premium: 5, qty: 0 },
    ]);
    expect(r).toMatch(/leg 2/);
});

test('validateLegs returns null when all legs are valid', () => {
    expect(validateLegs(PRESETS.iron_condor(100))).toBe(null);
});

test('buildPayoffBody includes all required backend fields', () => {
    const legs = PRESETS.long_call(100);
    const body = buildPayoffBody(legs, 50, 150, 121);
    expect(body.spot_min).toBe(50);
    expect(body.spot_max).toBe(150);
    expect(body.steps).toBe(121);
    expect(body.legs.length).toBe(1);
    expect(body.legs[0]).toEqual({ kind: 'call', strike: 100, premium: 3, qty: 1 });
});

test('buildPricerBody includes IV, rate, dividend, t_to_expiry', () => {
    const legs = PRESETS.long_straddle(100);
    const body = buildPricerBody(legs, 105, 0.5, 0.05, 0.02, 0.25);
    expect(body.spot).toBe(105);
    expect(body.t_to_expiry).toBe(0.5);
    expect(body.rate).toBe(0.05);
    expect(body.div_yield).toBe(0.02);
    expect(body.sigma).toBe(0.25);
    expect(body.legs.length).toBe(2);
});

test('defaultSpotRange spans ±50% around spot', () => {
    expect(defaultSpotRange(100)).toEqual({ min: 50, max: 150 });
});

test('defaultSpotRange handles zero / negative / NaN spot', () => {
    expect(defaultSpotRange(0).max > defaultSpotRange(0).min).toBe(true);
    expect(defaultSpotRange(-1).max > defaultSpotRange(-1).min).toBe(true);
    expect(defaultSpotRange(NaN).max > defaultSpotRange(NaN).min).toBe(true);
});
