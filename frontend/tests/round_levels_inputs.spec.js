// Round-number levels helpers: classify, validator, body shape,
// localDetect Rust-mirror, badges, demos, window guard.

import { test, expect } from 'vitest';
import {
    WEIGHTS, MAX_INTEGER_SCAN, DEFAULT_INPUTS,
    validateInputs, buildBody,
    classify, weightRank, localDetect,
    weightBadge, pinningBadge,
    makeDemoInput, fmtUSD, fmtUSDSigned, fmtAtrs, fmtPct, weightLabelKey,
} from '../js/_round_levels_inputs.js';

// ── constants ─────────────────────────────────────────────────────

test('WEIGHTS = snake_case Rust enum strings', () => {
    expect(WEIGHTS).toEqual(['major', 'medium', 'minor']);
});

test('MAX_INTEGER_SCAN = 100_000 (matches Rust guard)', () => {
    expect(MAX_INTEGER_SCAN).toBe(100_000);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts default', () => {
    expect(validateInputs(DEFAULT_INPUTS)).toBe(null);
});

test('validate accepts atr=null (Option<f64>=None)', () => {
    expect(validateInputs({ ...DEFAULT_INPUTS, atr: null })).toBe(null);
});

test('validate rejects: non-finite price / non-positive price / negative atr / non-positive window / bad enum', () => {
    expect(validateInputs({ ...DEFAULT_INPUTS, current_price: NaN })).toMatch(/current_price/);
    expect(validateInputs({ ...DEFAULT_INPUTS, current_price: 0 })).toMatch(/current_price/);
    expect(validateInputs({ ...DEFAULT_INPUTS, atr: -1 })).toMatch(/atr/);
    expect(validateInputs({ ...DEFAULT_INPUTS, config: { window: 0, min_weight: 'minor' } })).toMatch(/window/);
    expect(validateInputs({ ...DEFAULT_INPUTS, config: { window: 10, min_weight: 'huge' } })).toMatch(/min_weight/);
});

// ── classify (mirrors Rust #[test] classify_recognizes_each_weight_tier) ──

test('classify: each weight tier (largest-divisor wins)', () => {
    expect(classify(1000)).toBe('major');
    expect(classify(500)).toBe('major');
    expect(classify(100)).toBe('major');
    expect(classify(50)).toBe('medium');
    expect(classify(25)).toBe('medium');
    expect(classify(10)).toBe('minor');
    expect(classify(5)).toBe('minor');
    expect(classify(7)).toBe('minor');     // mod 1
    expect(classify(100.5)).toBe(null);    // non-integer → none
    expect(classify(NaN)).toBe(null);
});

test('weightRank: major > medium > minor > unknown', () => {
    expect(weightRank('major')).toBe(3);
    expect(weightRank('medium')).toBe(2);
    expect(weightRank('minor')).toBe(1);
    expect(weightRank('garbage')).toBe(0);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: flat-passes Numbers (no Decimal in this route) + nested config', () => {
    const body = buildBody({
        current_price: 100.5, atr: 2,
        config: { window: 50, min_weight: 'medium' },
    });
    expect(body).toEqual({
        current_price: 100.5, atr: 2,
        config: { window: 50, min_weight: 'medium' },
    });
});

test('buildBody: preserves atr=null', () => {
    expect(buildBody({ current_price: 100, atr: null,
                       config: { window: 10, min_weight: 'minor' } }).atr).toBe(null);
});

// ── localDetect parity (mirrors Rust #[test]s + boundaries) ───────

test('local: near_50 emits 45 and 50 within window', () => {
    const r = localDetect(48, null, { window: 5, min_weight: 'minor' });
    const has45 = r.levels.some(l => l.price === 45);
    const has50 = r.levels.some(l => l.price === 50);
    expect(has45 && has50).toBe(true);
    expect(r.nearest_above).not.toBeNull();
    expect(r.nearest_below).not.toBeNull();
});

test('local: min_weight=major filters out minor/medium', () => {
    const r = localDetect(125, null, { window: 100, min_weight: 'major' });
    for (const l of r.levels) expect(l.weight).toBe('major');
    expect(r.levels.some(l => l.price === 100)).toBe(true);
    expect(r.levels.some(l => l.price === 200)).toBe(true);
});

test('local: atr_scaling populates distance_atrs (100 from 101 with ATR=1 → -1.0)', () => {
    const r = localDetect(101, 1, { window: 5, min_weight: 'major' });
    const l100 = r.levels.find(l => l.price === 100);
    expect(l100).toBeDefined();
    expect(l100.distance_atrs).toBeCloseTo(-1, 9);
});

test('local: atr=null → distance_atrs=null on every level', () => {
    const r = localDetect(101, null, { window: 5, min_weight: 'major' });
    for (const l of r.levels) expect(l.distance_atrs).toBeNull();
});

test('local: atr=0 → distance_atrs=null (Rust filters >0)', () => {
    const r = localDetect(101, 0, { window: 5, min_weight: 'major' });
    for (const l of r.levels) expect(l.distance_atrs).toBeNull();
});

test('local: invalid inputs → empty', () => {
    expect(localDetect(-1, null, { window: 50, min_weight: 'minor' }).levels).toEqual([]);
    expect(localDetect(NaN, null, { window: 50, min_weight: 'minor' }).levels).toEqual([]);
    expect(localDetect(100, null, { window: 0, min_weight: 'minor' }).levels).toEqual([]);
});

test('local: enormous window > 100k integers short-circuits to empty (memory guard)', () => {
    const r = localDetect(50_000, null, { window: 100_001, min_weight: 'minor' });
    expect(r.levels).toEqual([]);
});

test('local: window exactly at 100k boundary still emits (NOT >, not ≥)', () => {
    // hi - lo ≤ 100_000 keeps it. Pick price 50_001 + window 50_000:
    // lo = 1, hi = 100_001 → diff = 100_000 → allowed.
    const r = localDetect(50_001, null, { window: 50_000, min_weight: 'major' });
    expect(r.levels.length).toBeGreaterThan(0);
});

test('local: nearest_above/below pick the closest by signed distance', () => {
    const r = localDetect(102.3, null, { window: 20, min_weight: 'minor' });
    expect(r.nearest_above.price).toBe(103);
    expect(r.nearest_below.price).toBe(102);
});

test('local: distance = price - current_price (signed)', () => {
    const r = localDetect(100, null, { window: 5, min_weight: 'minor' });
    for (const l of r.levels) {
        expect(l.distance).toBeCloseTo(l.price - 100, 9);
    }
});

test('local: lo clamped to 0 — current_price < window does not produce negative prices', () => {
    const r = localDetect(3, null, { window: 10, min_weight: 'minor' });
    for (const l of r.levels) expect(l.price).toBeGreaterThanOrEqual(0);
});

test('local: levels are in ascending price order', () => {
    const r = localDetect(102, null, { window: 20, min_weight: 'minor' });
    for (let i = 1; i < r.levels.length; i++) {
        expect(r.levels[i].price).toBeGreaterThan(r.levels[i - 1].price);
    }
});

test('local: $1000 prices register as major even though % 1 == 0', () => {
    const r = localDetect(1000, null, { window: 1, min_weight: 'major' });
    expect(r.levels.some(l => l.price === 1000 && l.weight === 'major')).toBe(true);
});

// ── badges ────────────────────────────────────────────────────────

test('weightBadge: major = neg cls, medium/minor empty cls, unknown safe', () => {
    expect(weightBadge('major').cls).toBe('neg');
    expect(weightBadge('medium').cls).toBe('');
    expect(weightBadge('minor').cls).toBe('');
    expect(weightBadge('garbage').key).toMatch(/unknown/);
});

test('pinningBadge: pinned < 0.1% < adjacent < 0.5% < near < 2% < clear', () => {
    const cp = 100;
    const mk = (d) => ({ distance: d });
    // distance = 0.05 → pct 0.0005 → pinned
    expect(pinningBadge(mk(0.05), null, cp).key).toMatch(/pinned/);
    // distance = 0.3 → pct 0.003 → adjacent
    expect(pinningBadge(mk(0.3), null, cp).key).toMatch(/adjacent/);
    // distance = 1 → pct 0.01 → near
    expect(pinningBadge(mk(1), null, cp).key).toMatch(/near/);
    // distance = 5 → pct 0.05 → clear
    expect(pinningBadge(mk(5), null, cp).key).toMatch(/clear/);
    // no levels either side → no_levels
    expect(pinningBadge(null, null, cp).key).toMatch(/no_levels/);
    // bad price → unknown
    expect(pinningBadge(null, null, NaN).key).toMatch(/unknown/);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + produces a finite levels array', () => {
    for (const k of ['aapl-near-180','spy-near-500','tsla-near-250','btc-near-100k',
                     'penny-near-3','pinned-at-100','major-only','no-atr']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localDetect(inp.current_price, inp.atr, inp.config);
        expect(Array.isArray(r.levels)).toBe(true);
    }
});

test('demo pinned-at-100: nearest_above or nearest_below is $100', () => {
    const inp = makeDemoInput('pinned-at-100');
    const r = localDetect(inp.current_price, inp.atr, inp.config);
    const all = [r.nearest_above, r.nearest_below].filter(Boolean);
    expect(all.some(l => l.price === 100)).toBe(true);
});

test('demo major-only: every emitted level is major-weight', () => {
    const inp = makeDemoInput('major-only');
    const r = localDetect(inp.current_price, inp.atr, inp.config);
    for (const l of r.levels) expect(l.weight).toBe('major');
});

test('demo no-atr: distance_atrs is null on every level', () => {
    const inp = makeDemoInput('no-atr');
    const r = localDetect(inp.current_price, inp.atr, inp.config);
    for (const l of r.levels) expect(l.distance_atrs).toBeNull();
});

test('demo btc-near-100k: $100,000 is in the levels list (major-only filter)', () => {
    // current_price=99850, window=5000 → both $99,900 and $100,000 are major (÷100, ÷1000).
    const inp = makeDemoInput('btc-near-100k');
    const r = localDetect(inp.current_price, inp.atr, inp.config);
    const hundredK = r.levels.find(l => l.price === 100_000);
    expect(hundredK).toBeDefined();
    expect(hundredK.weight).toBe('major');
    for (const l of r.levels) expect(l.weight).toBe('major');
});

// ── label key + formatters ────────────────────────────────────────

test('weightLabelKey: returns view.round_levels.weight.<w>', () => {
    expect(weightLabelKey('major')).toBe('view.round_levels.weight.major');
    expect(weightLabelKey()).toBe('view.round_levels.weight.unknown');
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtUSD(100)).toBe('$100.00');
    expect(fmtUSD(-50)).toBe('-$50.00');
    expect(fmtUSDSigned(2.5)).toBe('+$2.50');
    expect(fmtUSDSigned(-2.5)).toBe('-$2.50');
    expect(fmtAtrs(1.2345)).toBe('+1.23 ATR');
    expect(fmtAtrs(-0.5)).toBe('-0.50 ATR');
    expect(fmtAtrs(null)).toBe('—');
    expect(fmtPct(0.05)).toBe('5.00%');
    expect(fmtUSD(NaN)).toBe('—');
});
