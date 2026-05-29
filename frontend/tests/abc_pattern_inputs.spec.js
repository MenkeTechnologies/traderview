// ABC pattern detector helpers: parser, validator, localDetect parity, badges, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, DEFAULT_MIN_B, DEFAULT_MAX_B, DEFAULT_MIN_C_EXT,
    MIN_SWINGS, MAX_SWINGS,
    parseSwingsBlob, swingsToBlob, validateInputs, buildBody,
    localDetect, statusBadge, biasMixBadge, strengthBadge, summarizeSwings,
    makeDemoInput,
    fmtPrice, fmtRatio, fmtPct, fmtInt,
} from '../js/_abc_pattern_inputs.js';

const sp = (idx, price, kind) => ({ index: idx, price, kind });

// ── parser ────────────────────────────────────────────────────────

test('parseSwingsBlob: 3 tokens per line (index price kind)', () => {
    const r = parseSwingsBlob('0 150 high\n10 130 low\n20 155 high');
    expect(r.errors).toEqual([]);
    expect(r.swings).toEqual([sp(0, 150, 'high'), sp(10, 130, 'low'), sp(20, 155, 'high')]);
});

test('parseSwingsBlob: h/l abbreviations + comments', () => {
    const r = parseSwingsBlob('# header\n0 150 h\n10 130 l');
    expect(r.errors).toEqual([]);
    expect(r.swings.length).toBe(2);
    expect(r.swings[0].kind).toBe('high');
    expect(r.swings[1].kind).toBe('low');
});

test('parseSwingsBlob: rejects wrong token count / bad kind / bad index / bad price', () => {
    expect(parseSwingsBlob('0 150').errors[0].message).toMatch(/3 tokens/);
    expect(parseSwingsBlob('0 150 sideways').errors[0].message).toMatch(/kind/);
    expect(parseSwingsBlob('-1 150 high').errors[0].message).toMatch(/index/);
    expect(parseSwingsBlob('0 nan high').errors[0].message).toMatch(/price/);
});

test('parseSwingsBlob: non-string returns 1 error', () => {
    expect(parseSwingsBlob(undefined).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts default', () => {
    expect(validateInputs(DEFAULT_INPUTS)).toBe(null);
});

test('validate rejects shape problems', () => {
    expect(validateInputs({ ...DEFAULT_INPUTS, swings: 'no' })).toMatch(/swings/);
    expect(validateInputs({ ...DEFAULT_INPUTS, swings: [] })).toMatch(/at least 3/);
    const long = Array.from({ length: MAX_SWINGS + 1 }, (_, i) => sp(i, 100, 'high'));
    expect(validateInputs({ ...DEFAULT_INPUTS, swings: long })).toMatch(/too many/);
});

test('validate rejects bad swing fields', () => {
    expect(validateInputs({ ...DEFAULT_INPUTS, swings: [sp(-1, 100, 'high'), sp(1, 100, 'low'), sp(2, 100, 'high')] })).toMatch(/index/);
    expect(validateInputs({ ...DEFAULT_INPUTS, swings: [sp(0, NaN, 'high'), sp(1, 100, 'low'), sp(2, 100, 'high')] })).toMatch(/price/);
    expect(validateInputs({ ...DEFAULT_INPUTS, swings: [sp(0, 100, 'bad'), sp(1, 100, 'low'), sp(2, 100, 'high')] })).toMatch(/kind/);
});

test('validate rejects bad config', () => {
    expect(validateInputs({ ...DEFAULT_INPUTS, min_b_retrace: -0.1 })).toMatch(/min_b_retrace/);
    expect(validateInputs({ ...DEFAULT_INPUTS, max_b_retrace: 1.5 })).toMatch(/max_b_retrace/);
    expect(validateInputs({ ...DEFAULT_INPUTS, min_b_retrace: 0.8, max_b_retrace: 0.2 })).toMatch(/min_b_retrace > max_b_retrace/);
    expect(validateInputs({ ...DEFAULT_INPUTS, min_c_extension: 0 })).toMatch(/min_c_extension/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody shapes swings + config payload', () => {
    const body = buildBody(DEFAULT_INPUTS);
    expect(body.swings.length).toBe(3);
    expect(body.config.min_b_retrace).toBe(DEFAULT_MIN_B);
    expect(body.config.max_b_retrace).toBe(DEFAULT_MAX_B);
    expect(body.config.min_c_extension).toBe(DEFAULT_MIN_C_EXT);
});

// ── localDetect parity (mirrors every Rust #[test]) ──────────────

test('local: empty/short returns no events', () => {
    expect(localDetect([], DEFAULT_INPUTS).events).toEqual([]);
    expect(localDetect([sp(0, 100, 'high')], DEFAULT_INPUTS).events).toEqual([]);
});

test('local: invalid config returns no events', () => {
    const swings = [sp(0, 150, 'high'), sp(10, 130, 'low'), sp(20, 155, 'high')];
    expect(localDetect(swings, { min_b_retrace: 0.8, max_b_retrace: 0.2, min_c_extension: 1 }).events).toEqual([]);
    expect(localDetect(swings, { min_b_retrace: 0.4, max_b_retrace: 0.6, min_c_extension: 0 }).events).toEqual([]);
});

test('local: bearish ABC after top detected', () => {
    const swings = [sp(0, 150, 'high'), sp(10, 130, 'low'), sp(20, 155, 'high')];
    const r = localDetect(swings, DEFAULT_INPUTS);
    expect(r.events.length).toBe(1);
    expect(r.events[0].bias).toBe('bearish');
    expect(Math.abs(r.events[0].ab_length - 20)).toBeLessThan(1e-9);
    expect(Math.abs(r.events[0].bc_length - 25)).toBeLessThan(1e-9);
    expect(Math.abs(r.events[0].c_extension_ratio - 1.25)).toBeLessThan(1e-9);
});

test('local: bullish ABC after bottom detected', () => {
    const swings = [sp(0, 100, 'low'), sp(10, 120, 'high'), sp(20, 95, 'low')];
    const r = localDetect(swings, DEFAULT_INPUTS);
    expect(r.events.length).toBe(1);
    expect(r.events[0].bias).toBe('bullish');
});

test('local: non-alternating kinds dont match', () => {
    const swings = [sp(0, 100, 'high'), sp(10, 120, 'high'), sp(20, 95, 'high')];
    expect(localDetect(swings, DEFAULT_INPUTS).events).toEqual([]);
});

test('local: weak C extension skipped', () => {
    const swings = [sp(0, 150, 'high'), sp(10, 130, 'low'), sp(20, 135, 'high')];
    expect(localDetect(swings, DEFAULT_INPUTS).events).toEqual([]);
});

test('local: zero-length leg skipped', () => {
    const swings = [sp(0, 100, 'high'), sp(10, 100, 'low'), sp(20, 110, 'high')];
    expect(localDetect(swings, DEFAULT_INPUTS).events).toEqual([]);
});

test('local: deterministic', () => {
    const swings = [sp(0, 150, 'high'), sp(10, 130, 'low'), sp(20, 155, 'high')];
    expect(localDetect(swings, DEFAULT_INPUTS)).toEqual(localDetect(swings, DEFAULT_INPUTS));
});

// ── badges ────────────────────────────────────────────────────────

test('statusBadge: bullish / bearish / none / unknown', () => {
    expect(statusBadge(null).key).toMatch(/unknown/);
    expect(statusBadge({ events: [] }).key).toMatch(/none/);
    expect(statusBadge({ events: [{ bias: 'bullish' }] }).key).toMatch(/bullish/);
    expect(statusBadge({ events: [{ bias: 'bearish' }] }).key).toMatch(/bearish/);
});

test('biasMixBadge: tiers', () => {
    expect(biasMixBadge({ events: [] }).key).toMatch(/unknown/);
    expect(biasMixBadge({ events: [{ bias: 'bullish' }, { bias: 'bullish' }] }).key).toMatch(/all_bull/);
    expect(biasMixBadge({ events: [{ bias: 'bearish' }] }).key).toMatch(/all_bear/);
    expect(biasMixBadge({ events: [{ bias: 'bullish' }, { bias: 'bullish' }, { bias: 'bearish' }] }).key).toMatch(/bull_lean/);
    expect(biasMixBadge({ events: [{ bias: 'bearish' }, { bias: 'bearish' }, { bias: 'bullish' }] }).key).toMatch(/bear_lean/);
    expect(biasMixBadge({ events: [{ bias: 'bullish' }, { bias: 'bearish' }] }).key).toMatch(/balanced/);
});

test('strengthBadge: 4 tiers', () => {
    expect(strengthBadge(null).key).toMatch(/unknown/);
    expect(strengthBadge({ c_extension_ratio: 2.5 }).key).toMatch(/very_strong/);
    expect(strengthBadge({ c_extension_ratio: 1.7 }).key).toMatch(/strong/);
    expect(strengthBadge({ c_extension_ratio: 1.2 }).key).toMatch(/standard/);
    expect(strengthBadge({ c_extension_ratio: 0.5 }).key).toMatch(/weak/);
});

// ── summarize ─────────────────────────────────────────────────────

test('summarizeSwings: counts + extrema', () => {
    const s = summarizeSwings([sp(0, 150, 'high'), sp(10, 130, 'low'), sp(20, 155, 'high')]);
    expect(s.count).toBe(3);
    expect(s.highs).toBe(2);
    expect(s.lows).toBe(1);
    expect(s.min_price).toBe(130);
    expect(s.max_price).toBe(155);
    expect(s.span).toBe(25);
});

test('summarizeSwings: empty → NaN', () => {
    const s = summarizeSwings([]);
    expect(s.count).toBe(0);
    expect(Number.isNaN(s.min_price)).toBe(true);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: every preset validates', () => {
    for (const k of ['bearish-classic', 'bullish-classic', 'weak-c', 'non-alternating',
                     'multi-events', 'very-strong', 'zero-leg', 'tight-config']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
    }
});

test('demo bearish-classic: 1 bearish event', () => {
    const inp = makeDemoInput('bearish-classic');
    const r = localDetect(inp.swings, inp);
    expect(r.events.length).toBe(1);
    expect(r.events[0].bias).toBe('bearish');
});

test('demo bullish-classic: 1 bullish event', () => {
    const inp = makeDemoInput('bullish-classic');
    const r = localDetect(inp.swings, inp);
    expect(r.events.length).toBe(1);
    expect(r.events[0].bias).toBe('bullish');
});

test('demo weak-c / non-alternating / zero-leg: no events', () => {
    for (const k of ['weak-c', 'non-alternating', 'zero-leg']) {
        const inp = makeDemoInput(k);
        expect(localDetect(inp.swings, inp).events.length).toBe(0);
    }
});

test('demo very-strong: c_extension_ratio ≥ 2', () => {
    const inp = makeDemoInput('very-strong');
    const r = localDetect(inp.swings, inp);
    expect(r.events.length).toBe(1);
    expect(r.events[0].c_extension_ratio).toBeGreaterThanOrEqual(2);
});

test('demo multi-events: detects multiple ABC windows', () => {
    const inp = makeDemoInput('multi-events');
    const r = localDetect(inp.swings, inp);
    // 3 sliding windows from 5 swings; how many qualify depends on data.
    expect(r.events.length).toBeGreaterThanOrEqual(1);
});

test('demo tight-config: passes its own narrow band', () => {
    const inp = makeDemoInput('tight-config');
    const r = localDetect(inp.swings, inp);
    expect(r.events.length).toBe(1);
});

// ── formatters / roundtrip ────────────────────────────────────────

test('swingsToBlob round-trips', () => {
    const sws = [sp(0, 150, 'high'), sp(10, 130, 'low'), sp(20, 155, 'high')];
    const back = parseSwingsBlob(swingsToBlob(sws));
    expect(back.errors).toEqual([]);
    expect(back.swings).toEqual(sws);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtPrice(150.456)).toBe('150.46');
    expect(fmtRatio(0.7654)).toBe('0.765');
    expect(fmtPct(0.125)).toBe('12.5%');
    expect(fmtInt(42.9)).toBe('42');
    expect(fmtPrice(NaN)).toBe('—');
    expect(fmtRatio(Infinity)).toBe('—');
    expect(fmtPct(NaN)).toBe('—');
    expect(fmtInt(NaN)).toBe('—');
});

test('DEFAULTS sanity', () => {
    expect(MIN_SWINGS).toBe(3);
    expect(DEFAULT_MIN_B).toBe(0.382);
    expect(DEFAULT_MAX_B).toBe(0.618);
    expect(DEFAULT_MIN_C_EXT).toBe(1.0);
});
