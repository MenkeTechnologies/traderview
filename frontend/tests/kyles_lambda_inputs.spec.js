// Kyle's Lambda helpers: parser, validator, body shape,
// localCompute Rust-mirror, summarize, badges, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_WINDOW, DEFAULT_INPUTS,
    parseFlowBlob, validateInputs, buildBody, localCompute,
    summarize, liquidityBadge, signBadge,
    makeDemoInput, fmtLambda, fmtSci, fmtInt,
} from '../js/_kyles_lambda_inputs.js';

// ── constants ─────────────────────────────────────────────────────

test('DEFAULT_WINDOW = 30 (matches Rust default)', () => {
    expect(DEFAULT_WINDOW).toBe(30);
});

// ── parser ────────────────────────────────────────────────────────

test('parseFlowBlob: 2 tokens per line; ignores blanks + # comments', () => {
    const r = parseFlowBlob('0.01 1000\n# note\n\n-0.02 -500');
    expect(r.errors).toEqual([]);
    expect(r.price_changes).toEqual([0.01, -0.02]);
    expect(r.signed_volumes).toEqual([1000, -500]);
});

test('parseFlowBlob: rejects bad token count + non-finite numbers', () => {
    expect(parseFlowBlob('0.01').errors[0].message).toMatch(/2 tokens/);
    expect(parseFlowBlob('NaN 1000').errors[0].message).toMatch(/price_change/);
    expect(parseFlowBlob('0.01 foo').errors[0].message).toMatch(/signed_volume/);
});

test('parseFlowBlob: non-string returns 1 error', () => {
    expect(parseFlowBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts default', () => {
    expect(validateInputs({ ...DEFAULT_INPUTS, window: 30 })).toBe(null);
});

test('validate rejects: bad arrays / window < 2 / non-integer window / mismatched lengths', () => {
    expect(validateInputs({ ...DEFAULT_INPUTS, price_changes: 'no' })).toMatch(/price_changes/);
    expect(validateInputs({ ...DEFAULT_INPUTS, signed_volumes: null })).toMatch(/signed_volumes/);
    expect(validateInputs({ ...DEFAULT_INPUTS, window: 1 })).toMatch(/window/);
    expect(validateInputs({ ...DEFAULT_INPUTS, window: NaN })).toMatch(/window/);
    expect(validateInputs({ ...DEFAULT_INPUTS, window: 5.5 })).toMatch(/integer/);
    expect(validateInputs({ price_changes: [1, 2], signed_volumes: [1], window: 2 })).toMatch(/same length/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: forwards arrays + window verbatim', () => {
    const body = buildBody({ price_changes: [0.01], signed_volumes: [100], window: 30 });
    expect(body).toEqual({ price_changes: [0.01], signed_volumes: [100], window: 30 });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: empty arrays return empty', () => {
    expect(localCompute([], [], 20)).toEqual([]);
});

test('local: length mismatch returns all nulls', () => {
    const out = localCompute(Array(30).fill(0.01), Array(15).fill(1000), 20);
    expect(out.every(v => v == null)).toBe(true);
});

test('local: window < 2 returns all nulls', () => {
    const p = Array(30).fill(0.01), v = Array(30).fill(1000);
    expect(localCompute(p, v, 0).every(x => x == null)).toBe(true);
    expect(localCompute(p, v, 1).every(x => x == null)).toBe(true);
});

test('local: perfect linear Δp = 0.5·v recovers λ = 0.5', () => {
    const v = Array.from({ length: 30 }, (_, i) => i + 1);
    const p = v.map(x => 0.5 * x);
    const out = localCompute(p, v, 20);
    expect(out[29]).toBeCloseTo(0.5, 9);
});

test('local: zero signed-flow window → null (sxx=0 guard)', () => {
    const out = localCompute(Array(30).fill(0.01), Array(30).fill(0), 20);
    expect(out[29]).toBeNull();
});

test('local: NaN pairs skipped — still populated from remaining pairs', () => {
    const p = Array(30).fill(0.01), v = Array(30).fill(1000);
    p[15] = NaN; v[16] = NaN;
    const out = localCompute(p, v, 20);
    expect(out[29]).not.toBeNull();
});

test('local: negative-slope Δp = -0.3·v recovers λ = -0.3', () => {
    const v = Array.from({ length: 30 }, (_, i) => i - 15);
    const p = v.map(x => -0.3 * x);
    const out = localCompute(p, v, 20);
    expect(out[29]).toBeCloseTo(-0.3, 9);
});

test('local: noisy y = 0.4x + sin yields λ near 0.4', () => {
    const v = Array.from({ length: 50 }, (_, i) => i + 1);
    const p = v.map((x, i) => 0.4 * x + Math.sin(i * 0.7) * 0.5);
    const out = localCompute(p, v, 30);
    expect(out[49]).toBeCloseTo(0.4, 1);
});

test('local: warmup region (0..window-2) is null; first populated at window-1', () => {
    const v = Array.from({ length: 30 }, (_, i) => i + 1);
    const p = v.map(x => 0.5 * x);
    const out = localCompute(p, v, 20);
    for (let i = 0; i < 19; i++) expect(out[i]).toBeNull();
    expect(out[19]).not.toBeNull();
});

test('local: output length = input length (no truncation)', () => {
    expect(localCompute(Array(50).fill(0.01), Array(50).fill(1000), 30).length).toBe(50);
});

test('local: valid < 2 in window → null (NaN-saturated window)', () => {
    const p = Array(30).fill(NaN), v = Array(30).fill(NaN);
    p[29] = 0.01; v[29] = 1000;
    const out = localCompute(p, v, 20);
    expect(out[29]).toBeNull();
});

// ── summarize ─────────────────────────────────────────────────────

test('summarize: count/mean/min/max/last over non-null portion', () => {
    const s = summarize([null, null, 0.5, 1.5, 2.5]);
    expect(s.count).toBe(3);
    expect(s.mean).toBeCloseTo(1.5, 9);
    expect(s.min).toBe(0.5);
    expect(s.max).toBe(2.5);
    expect(s.last).toBe(2.5);
});

test('summarize: all-null series returns count=0 + NaN aggregates', () => {
    const s = summarize([null, null, null]);
    expect(s.count).toBe(0);
    expect(Number.isNaN(s.mean)).toBe(true);
});

// ── badges ────────────────────────────────────────────────────────

test('liquidityBadge: deep / normal / thin / illiquid by |λ|', () => {
    expect(liquidityBadge(1e-6).key).toMatch(/deep/);
    expect(liquidityBadge(5e-5).key).toMatch(/normal/);
    expect(liquidityBadge(5e-4).key).toMatch(/thin/);
    expect(liquidityBadge(5e-3).key).toMatch(/illiquid/);
    expect(liquidityBadge(null).key).toMatch(/unknown/);
    expect(liquidityBadge(NaN).key).toMatch(/unknown/);
});

test('liquidityBadge: negative λ uses |λ| for tier', () => {
    expect(liquidityBadge(-5e-3).key).toMatch(/illiquid/);
});

test('signBadge: positive = momentum, negative = reversion, zero = flat', () => {
    expect(signBadge(0.001).key).toMatch(/momentum/);
    expect(signBadge(-0.001).key).toMatch(/reversion/);
    expect(signBadge(0).key).toMatch(/flat/);
    expect(signBadge(null).key).toMatch(/unknown/);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + computes a non-empty series', () => {
    for (const k of ['deep-mm','normal-mid-cap','thin-small-cap','illiquid-penny',
                     'reversion','regime-shift','zero-flow','nan-spotty']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const out = localCompute(inp.price_changes, inp.signed_volumes, inp.window);
        expect(out.length).toBe(inp.price_changes.length);
    }
});

test('demo deep-mm: last λ < normal threshold (1e-4)', () => {
    const inp = makeDemoInput('deep-mm');
    const out = localCompute(inp.price_changes, inp.signed_volumes, inp.window);
    expect(Math.abs(out[out.length - 1])).toBeLessThan(1e-4);
});

test('demo thin-small-cap: last λ in thin tier (≥ 1e-4 and < 1e-3)', () => {
    const inp = makeDemoInput('thin-small-cap');
    const out = localCompute(inp.price_changes, inp.signed_volumes, inp.window);
    const last = Math.abs(out[out.length - 1]);
    expect(last).toBeGreaterThanOrEqual(1e-4);
    expect(last).toBeLessThan(1e-3);
});

test('demo reversion: last λ sign is negative', () => {
    const inp = makeDemoInput('reversion');
    const out = localCompute(inp.price_changes, inp.signed_volumes, inp.window);
    expect(out[out.length - 1]).toBeLessThan(0);
});

test('demo zero-flow: every populated slot is null (sxx=0)', () => {
    const inp = makeDemoInput('zero-flow');
    const out = localCompute(inp.price_changes, inp.signed_volumes, inp.window);
    expect(out.every(v => v == null)).toBe(true);
});

test('demo regime-shift: λ near bar-99 < λ near bar-199 (shift to thinner book)', () => {
    const inp = makeDemoInput('regime-shift');
    const out = localCompute(inp.price_changes, inp.signed_volumes, inp.window);
    expect(Math.abs(out[99])).toBeLessThan(Math.abs(out[199]));
});

test('demo nan-spotty: λ still populated at the tail', () => {
    const inp = makeDemoInput('nan-spotty');
    const out = localCompute(inp.price_changes, inp.signed_volumes, inp.window);
    expect(out[out.length - 1]).not.toBeNull();
});

// ── formatters ────────────────────────────────────────────────────

test('fmtLambda: scientific notation for small λ, fixed for large', () => {
    expect(fmtLambda(0.005)).toBe('0.005000');
    expect(fmtLambda(1e-5)).toMatch(/e/i);
    expect(fmtLambda(null)).toBe('—');
});

test('fmtSci: same scheme; non-finite → —', () => {
    expect(fmtSci(NaN)).toBe('—');
    expect(fmtSci(0)).toBe('0.000000');
    expect(fmtSci(1e-5)).toMatch(/e/i);
});

test('fmtInt: integer part; non-finite → —', () => {
    expect(fmtInt(42.7)).toBe('42');
    expect(fmtInt(NaN)).toBe('—');
});
