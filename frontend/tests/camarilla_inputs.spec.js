// Camarilla pivots helpers: parser, validator, localCompute parity, badges.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS,
    parseInputBlob, inputToBlob, validateInputs, buildBody, localCompute,
    zoneBadge, ruleBadge, widthBadge, nearestLevelInfo,
    makeDemoInput,
    fmtPrice, fmtPriceSigned, fmtPct, fmtInt,
} from '../js/_camarilla_inputs.js';

// ── parser ────────────────────────────────────────────────────────

test('parseInputBlob: 3-token HLC', () => {
    const r = parseInputBlob('110 100 105');
    expect(r.errors).toEqual([]);
    expect(r.session).toEqual({ high: 110, low: 100, close: 105 });
    expect(r.current_price).toBe(null);
});

test('parseInputBlob: 4-token HLC + current_price', () => {
    const r = parseInputBlob('110 100 105 106');
    expect(r.errors).toEqual([]);
    expect(r.current_price).toBe(106);
});

test('parseInputBlob: rejects < 3 tokens', () => {
    expect(parseInputBlob('110 100').errors[0].message).toMatch(/3 tokens/);
});

test('parseInputBlob: rejects non-finite', () => {
    expect(parseInputBlob('110 NaN 105').errors[0].message).toMatch(/finite/);
});

test('parseInputBlob: non-string returns 1 error', () => {
    expect(parseInputBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts valid', () => {
    expect(validateInputs({ session: { high: 110, low: 100, close: 105 } })).toBe(null);
});

test('validate rejects: missing / non-number / non-finite / high < low / close out / non-positive', () => {
    expect(validateInputs({})).toMatch(/session/);
    expect(validateInputs({ session: { high: '110', low: 100, close: 105 } })).toMatch(/numbers/);
    expect(validateInputs({ session: { high: NaN, low: 100, close: 105 } })).toMatch(/finite/);
    expect(validateInputs({ session: { high: 99, low: 100, close: 100 } })).toMatch(/high < low/);
    expect(validateInputs({ session: { high: 110, low: 100, close: 50 } })).toMatch(/close outside/);
    expect(validateInputs({ session: { high: 110, low: -1, close: 105 } })).toMatch(/positive/);
    expect(validateInputs({ session: { high: 110, low: 100, close: 105 }, current_price: NaN })).toMatch(/current_price/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody passes session through; drops current_price', () => {
    const body = buildBody({ session: { high: 110, low: 100, close: 105 }, current_price: 106 });
    expect(body).toEqual({ session: { high: 110, low: 100, close: 105 } });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: invalid session returns null', () => {
    expect(localCompute({ high: NaN, low: 99, close: 100 })).toBe(null);
    expect(localCompute({ high: 99, low: 101, close: 100 })).toBe(null);
});

test('local: levels symmetric around close + ordered', () => {
    const r = localCompute({ high: 110, low: 100, close: 105 });
    expect(Math.abs((r.h1 - 105) - (105 - r.l1))).toBeLessThan(1e-9);
    expect(Math.abs((r.h4 - 105) - (105 - r.l4))).toBeLessThan(1e-9);
    expect(r.h1).toBeLessThan(r.h2);
    expect(r.h2).toBeLessThan(r.h3);
    expect(r.h3).toBeLessThan(r.h4);
    expect(r.l1).toBeGreaterThan(r.l2);
    expect(r.l2).toBeGreaterThan(r.l3);
    expect(r.l3).toBeGreaterThan(r.l4);
});

test('local: exact formula values (range=10, k=11)', () => {
    const r = localCompute({ high: 110, low: 100, close: 105 });
    expect(Math.abs(r.h4 - (105 + 11/2))).toBeLessThan(1e-9);
    expect(Math.abs(r.h3 - (105 + 11/4))).toBeLessThan(1e-9);
    expect(Math.abs(r.h2 - (105 + 11/6))).toBeLessThan(1e-9);
    expect(Math.abs(r.h1 - (105 + 11/12))).toBeLessThan(1e-9);
    expect(Math.abs(r.l1 - (105 - 11/12))).toBeLessThan(1e-9);
    expect(Math.abs(r.l2 - (105 - 11/6))).toBeLessThan(1e-9);
    expect(Math.abs(r.l3 - (105 - 11/4))).toBeLessThan(1e-9);
    expect(Math.abs(r.l4 - (105 - 11/2))).toBeLessThan(1e-9);
});

test('local: pivot = typical price (H+L+C)/3', () => {
    const r = localCompute({ high: 110, low: 100, close: 108 });
    expect(Math.abs(r.pivot - (110 + 100 + 108) / 3)).toBeLessThan(1e-9);
});

test('local: zero range collapses all levels to close', () => {
    const r = localCompute({ high: 100, low: 100, close: 100 });
    for (const v of [r.h4, r.h3, r.h2, r.h1, r.l1, r.l2, r.l3, r.l4]) {
        expect(Math.abs(v - 100)).toBeLessThan(1e-9);
    }
});

test('local: deterministic', () => {
    const s = { high: 110, low: 100, close: 105 };
    expect(localCompute(s)).toEqual(localCompute(s));
});

// ── badges ────────────────────────────────────────────────────────

test('zoneBadge: tiers across all 9 zones', () => {
    const lv = localCompute({ high: 110, low: 100, close: 105 });
    expect(zoneBadge(lv, lv.h4 + 1).key).toMatch(/above_h4/);
    expect(zoneBadge(lv, lv.h3 + 0.5).key).toMatch(/h3_h4/);
    expect(zoneBadge(lv, lv.h2 + 0.1).key).toMatch(/h2_h3/);
    expect(zoneBadge(lv, lv.h1 + 0.01).key).toMatch(/h1_h2/);
    expect(zoneBadge(lv, 105).key).toMatch(/pivot/);
    expect(zoneBadge(lv, lv.l1 - 0.01).key).toMatch(/l1_l2/);
    expect(zoneBadge(lv, lv.l2 - 0.1).key).toMatch(/l2_l3/);
    expect(zoneBadge(lv, lv.l3 - 0.1).key).toMatch(/l3_l4/);
    expect(zoneBadge(lv, lv.l4 - 1).key).toMatch(/below_l4/);
    expect(zoneBadge(null, 100).key).toMatch(/unknown/);
});

test('ruleBadge: breakout_long / breakdown_short / reversals / no_signal', () => {
    const lv = localCompute({ high: 110, low: 100, close: 105 });
    expect(ruleBadge(lv, lv.h4 + 1).key).toMatch(/breakout_long/);
    expect(ruleBadge(lv, lv.l4 - 1).key).toMatch(/breakdown_short/);
    expect(ruleBadge(lv, lv.h3 + 0.01).key).toMatch(/short_reversal/);
    expect(ruleBadge(lv, lv.l3 - 0.01).key).toMatch(/long_reversal/);
    expect(ruleBadge(lv, 105).key).toMatch(/no_signal/);
    expect(ruleBadge(null, 100).key).toMatch(/unknown/);
});

test('widthBadge: tight / normal / wide / very_wide / unknown', () => {
    // Need to compute via localCompute on contrived close values.
    const tight = localCompute({ high: 100.1, low: 99.9, close: 100 });   // range 0.2
    expect(widthBadge(tight).key).toMatch(/tight/);
    const normal = localCompute({ high: 100.5, low: 99.5, close: 100 });   // range 1 → width 0.011
    expect(widthBadge(normal).key).toMatch(/normal/);
    const wide = localCompute({ high: 110, low: 100, close: 105 });        // range 10
    expect(widthBadge(wide).key).toMatch(/very_wide/);
    expect(widthBadge(null).key).toMatch(/unknown/);
});

// ── nearestLevelInfo ─────────────────────────────────────────────

test('nearestLevelInfo: pivot when at midline', () => {
    const lv = localCompute({ high: 110, low: 100, close: 105 });
    const r = nearestLevelInfo(lv, lv.pivot);
    expect(r.name).toBe('Pivot');
    expect(Math.abs(r.distance)).toBeLessThan(1e-9);
});

test('nearestLevelInfo: H4 when above', () => {
    const lv = localCompute({ high: 110, low: 100, close: 105 });
    const r = nearestLevelInfo(lv, lv.h4 + 0.001);
    expect(r.name).toBe('H4');
});

test('nearestLevelInfo: null on bad input', () => {
    expect(nearestLevelInfo(null, 100).name).toBe(null);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + computes', () => {
    for (const k of ['standard-range','breakout-long','breakdown-short','short-reversal',
                     'long-reversal','tight-range','wide-range','flat-session']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.session);
        expect(r).not.toBe(null);
    }
});

test('demo breakout-long: rule = breakout_long', () => {
    const inp = makeDemoInput('breakout-long');
    const lv = localCompute(inp.session);
    expect(ruleBadge(lv, inp.current_price).key).toMatch(/breakout_long/);
});

test('demo breakdown-short: rule = breakdown_short', () => {
    const inp = makeDemoInput('breakdown-short');
    const lv = localCompute(inp.session);
    expect(ruleBadge(lv, inp.current_price).key).toMatch(/breakdown_short/);
});

test('demo flat-session: all levels collapse to 100', () => {
    const inp = makeDemoInput('flat-session');
    const r = localCompute(inp.session);
    for (const v of [r.h4, r.h3, r.h2, r.h1, r.pivot, r.l1, r.l2, r.l3, r.l4]) {
        expect(Math.abs(v - 100)).toBeLessThan(1e-9);
    }
});

// ── formatters ────────────────────────────────────────────────────

test('inputToBlob round-trips through parseInputBlob (with current_price)', () => {
    const inp = { session: { high: 110, low: 100, close: 105 }, current_price: 106 };
    const back = parseInputBlob(inputToBlob(inp));
    expect(back.errors).toEqual([]);
    expect(back.session).toEqual(inp.session);
    expect(back.current_price).toBe(106);
});

test('inputToBlob: no current_price renders 3 tokens', () => {
    const inp = { session: { high: 110, low: 100, close: 105 }, current_price: null };
    const blob = inputToBlob(inp);
    expect(blob.split(/\s+/).length).toBe(3);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtPrice(100.4567)).toBe('100.4567');
    expect(fmtPriceSigned(1.5)).toBe('+1.5000');
    expect(fmtPriceSigned(-1.5)).toBe('-1.5000');
    expect(fmtPct(0.0125)).toBe('1.25%');
    expect(fmtInt(42.9)).toBe('42');
    expect(fmtPrice(NaN)).toBe('—');
});

test('DEFAULTS sanity', () => {
    expect(DEFAULT_INPUTS.session).toBeDefined();
    expect(DEFAULT_INPUTS.session.high).toBe(110);
});
