// Hawkes intensity helpers: parser, validator, body shape,
// localCompute Rust-mirror, badges, clustering ratio, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_PARAMS, DEFAULT_INPUTS,
    parseTimesBlob, validateInputs, buildBody,
    localCompute, localIntensityAfterEach, makeQueryGrid,
    stabilityBadge, clusteringRatio,
    makeDemoInput, fmtNum, fmtInt, fmtRatio,
} from '../js/_hawkes_intensity_inputs.js';

// ── constants ─────────────────────────────────────────────────────

test('DEFAULT_PARAMS matches standard stable parameterization', () => {
    expect(DEFAULT_PARAMS).toEqual({ baseline_mu: 0.5, excitation_alpha: 0.4, decay_beta: 1.0 });
});

// ── parser ────────────────────────────────────────────────────────

test('parseTimesBlob: one timestamp per line; ignores blanks + # comments', () => {
    const r = parseTimesBlob('1\n# note\n\n2.5\n3');
    expect(r.errors).toEqual([]);
    expect(r.times).toEqual([1, 2.5, 3]);
});

test('parseTimesBlob: rejects non-finite', () => {
    expect(parseTimesBlob('1\nfoo\n2').errors[0].message).toMatch(/timestamp/);
});

test('parseTimesBlob: non-string returns 1 error', () => {
    expect(parseTimesBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts default', () => {
    expect(validateInputs(DEFAULT_INPUTS)).toBe(null);
});

test('validate rejects: non-array / non-finite / unsorted / bad params', () => {
    expect(validateInputs({ ...DEFAULT_INPUTS, event_times: 'no' })).toMatch(/event_times/);
    expect(validateInputs({ ...DEFAULT_INPUTS, query_times: null })).toMatch(/query_times/);
    expect(validateInputs({ ...DEFAULT_INPUTS, event_times: [1, NaN] })).toMatch(/finite/);
    expect(validateInputs({ ...DEFAULT_INPUTS, event_times: [2, 1] })).toMatch(/sorted/);
    expect(validateInputs({ ...DEFAULT_INPUTS, params: null })).toMatch(/params/);
    expect(validateInputs({ ...DEFAULT_INPUTS, params: { baseline_mu: -0.1, excitation_alpha: 0.5, decay_beta: 1 } })).toMatch(/baseline_mu/);
    expect(validateInputs({ ...DEFAULT_INPUTS, params: { baseline_mu: 0.5, excitation_alpha: -0.1, decay_beta: 1 } })).toMatch(/excitation_alpha/);
    expect(validateInputs({ ...DEFAULT_INPUTS, params: { baseline_mu: 0.5, excitation_alpha: 0.5, decay_beta: 0 } })).toMatch(/decay_beta/);
});

test('validate accepts excitation_alpha = 0 (degenerate Poisson)', () => {
    expect(validateInputs({ ...DEFAULT_INPUTS,
        params: { baseline_mu: 0.5, excitation_alpha: 0, decay_beta: 1 } })).toBe(null);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: nested params object preserved', () => {
    const body = buildBody(DEFAULT_INPUTS);
    expect(body.event_times).toEqual(DEFAULT_INPUTS.event_times);
    expect(body.params).toEqual(DEFAULT_PARAMS);
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: invalid params return null', () => {
    expect(localCompute([], [], { baseline_mu: -0.1, excitation_alpha: 0.5, decay_beta: 1 })).toBeNull();
    expect(localCompute([], [], { baseline_mu: 0.1, excitation_alpha: -0.1, decay_beta: 1 })).toBeNull();
    expect(localCompute([], [], { baseline_mu: 0.1, excitation_alpha: 0.5, decay_beta: 0 })).toBeNull();
    expect(localCompute([], [], { baseline_mu: NaN, excitation_alpha: 0.5, decay_beta: 1 })).toBeNull();
});

test('local: unsorted events rejected', () => {
    expect(localCompute([1, 0.5], [2], { baseline_mu: 0.1, excitation_alpha: 0.5, decay_beta: 1 })).toBeNull();
});

test('local: no events yields baseline intensity at every query', () => {
    const r = localCompute([], [1, 2, 3], { baseline_mu: 0.5, excitation_alpha: 0, decay_beta: 1 });
    for (const v of r.intensities) expect(v).toBeCloseTo(0.5, 12);
});

test('local: event burst inflates local intensity', () => {
    const evs = [1, 2, 3, 4, 5];
    const r = localCompute(evs, [0.5, 5.5], { baseline_mu: 0.1, excitation_alpha: 0.5, decay_beta: 1 });
    expect(r.intensities[1]).toBeGreaterThan(r.intensities[0]);
    expect(r.intensities[1]).toBeGreaterThan(0.1);
});

test('local: stable when alpha < beta', () => {
    const r = localCompute([], [1], { baseline_mu: 0.1, excitation_alpha: 0.3, decay_beta: 1 });
    expect(r.is_stable).toBe(true);
    expect(Number.isFinite(r.unconditional_mean_intensity)).toBe(true);
});

test('local: unstable when alpha >= beta (infinite unconditional mean)', () => {
    const r = localCompute([], [1], { baseline_mu: 0.1, excitation_alpha: 1, decay_beta: 1 });
    expect(r.is_stable).toBe(false);
    expect(r.unconditional_mean_intensity).toBe(Infinity);
});

test('local: decay back toward baseline after single event', () => {
    const r = localCompute([0], [10], { baseline_mu: 0.1, excitation_alpha: 0.5, decay_beta: 1 });
    expect(Math.abs(r.intensities[0] - 0.1)).toBeLessThan(0.001);
});

test('local: unconditional mean = μ / (1 - α/β)', () => {
    const r = localCompute([], [1], { baseline_mu: 1, excitation_alpha: 0.5, decay_beta: 1 });
    expect(r.unconditional_mean_intensity).toBeCloseTo(2, 9);
});

test('local: query at event time itself does NOT include that event (strict <)', () => {
    // At t=1 exactly, the event at t=1 has not "occurred before t" — intensity = μ only.
    const r = localCompute([1], [1], { baseline_mu: 0.5, excitation_alpha: 1, decay_beta: 1 });
    expect(r.intensities[0]).toBeCloseTo(0.5, 12);
});

test('local: query before first event = baseline μ', () => {
    const r = localCompute([5], [1, 2, 3, 4], { baseline_mu: 0.3, excitation_alpha: 0.5, decay_beta: 1 });
    for (const v of r.intensities) expect(v).toBeCloseTo(0.3, 12);
});

test('local: intensity_after_each_event includes self-kick', () => {
    const r = localIntensityAfterEach([1], { baseline_mu: 0.1, excitation_alpha: 0.5, decay_beta: 1 });
    expect(r[0]).toBeCloseTo(0.6, 9);
});

test('local: intensity_after each event accumulates prior excitation', () => {
    const r = localIntensityAfterEach([0, 0.5, 1.0],
        { baseline_mu: 0.1, excitation_alpha: 0.5, decay_beta: 1 });
    // First event: μ + α = 0.6. Each subsequent should be ≥ first.
    expect(r[0]).toBeCloseTo(0.6, 9);
    expect(r[1]).toBeGreaterThan(r[0]);
    expect(r[2]).toBeGreaterThan(r[1]);
});

// ── makeQueryGrid ─────────────────────────────────────────────────

test('makeQueryGrid: empty events returns evenly-spaced ints', () => {
    const g = makeQueryGrid([], 5);
    expect(g).toEqual([0, 1, 2, 3, 4]);
});

test('makeQueryGrid: covers [first - pad, last + pad] inclusive', () => {
    const g = makeQueryGrid([1, 10], 11, 0);
    expect(g[0]).toBe(1);
    expect(g[g.length - 1]).toBe(10);
    expect(g.length).toBe(11);
});

// ── stabilityBadge ────────────────────────────────────────────────

test('stabilityBadge: poisson (α=0), weak (<0.5), clustered (<0.9), critical (<1), explosive (≥1)', () => {
    const mk = (a, b) => ({ baseline_mu: 0.1, excitation_alpha: a, decay_beta: b });
    expect(stabilityBadge(mk(0, 1)).key).toMatch(/poisson/);
    expect(stabilityBadge(mk(0.3, 1)).key).toMatch(/weak/);
    expect(stabilityBadge(mk(0.6, 1)).key).toMatch(/clustered/);
    expect(stabilityBadge(mk(0.95, 1)).key).toMatch(/critical/);
    expect(stabilityBadge(mk(1, 1)).key).toMatch(/explosive/);
    expect(stabilityBadge(null).key).toMatch(/unknown/);
});

// ── clusteringRatio ──────────────────────────────────────────────

test('clusteringRatio: max / baseline_mu', () => {
    expect(clusteringRatio([0.5, 1.5, 0.8], 0.5)).toBeCloseTo(3, 9);
});

test('clusteringRatio: empty / bad mu → NaN', () => {
    expect(Number.isNaN(clusteringRatio([], 0.5))).toBe(true);
    expect(Number.isNaN(clusteringRatio([1, 2], 0))).toBe(true);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + produces a non-null report', () => {
    for (const k of ['poisson-baseline','cluster-burst','news-burst','critical',
                     'explosive','no-events','long-decay','fast-decay']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.event_times, inp.query_times, inp.params);
        expect(r).not.toBeNull();
        expect(r.intensities.length).toBe(inp.query_times.length);
    }
});

test('demo poisson-baseline: every intensity = μ (α=0)', () => {
    const inp = makeDemoInput('poisson-baseline');
    const r = localCompute(inp.event_times, inp.query_times, inp.params);
    for (const v of r.intensities) expect(v).toBeCloseTo(inp.params.baseline_mu, 12);
});

test('demo cluster-burst: peak intensity > 2 × baseline', () => {
    const inp = makeDemoInput('cluster-burst');
    const r = localCompute(inp.event_times, inp.query_times, inp.params);
    expect(Math.max(...r.intensities)).toBeGreaterThan(2 * inp.params.baseline_mu);
});

test('demo explosive: is_stable=false, unconditional=Infinity', () => {
    const inp = makeDemoInput('explosive');
    const r = localCompute(inp.event_times, inp.query_times, inp.params);
    expect(r.is_stable).toBe(false);
    expect(r.unconditional_mean_intensity).toBe(Infinity);
});

test('demo no-events: every intensity = μ', () => {
    const inp = makeDemoInput('no-events');
    const r = localCompute(inp.event_times, inp.query_times, inp.params);
    for (const v of r.intensities) expect(v).toBe(inp.params.baseline_mu);
});

test('demo critical: branching ratio is in [0.9, 1) → critical badge', () => {
    const inp = makeDemoInput('critical');
    expect(stabilityBadge(inp.params).key).toMatch(/critical/);
});

// ── formatters ────────────────────────────────────────────────────

test('fmt helpers + non-finite guards', () => {
    expect(fmtNum(1.23456)).toBe('1.2346');
    expect(fmtNum(NaN)).toBe('—');
    expect(fmtNum(Infinity)).toBe('∞');
    expect(fmtInt(42.9)).toBe('42');
    expect(fmtInt(NaN)).toBe('—');
    // Avoid 1.2345 — IEEE 754 banker's rounding makes toFixed(3) → '1.234'.
    expect(fmtRatio(1.2346)).toBe('1.235');
    expect(fmtRatio(0.5)).toBe('0.500');
    expect(fmtRatio(NaN)).toBe('—');
});
