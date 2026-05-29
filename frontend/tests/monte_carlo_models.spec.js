// Monte Carlo view pure helpers: model registry, validation, payload
// shaping, normal-density curve. View itself is DOM-bound and tested in
// the browser.

import { test, expect } from 'vitest';
import {
    MODELS, validateValues, defaultValues, normalDensityCurve,
} from '../js/_monte_carlo_models.js';

// ── registry shape ───────────────────────────────────────────────────

test('every model has the required fields', () => {
    for (const [id, m] of Object.entries(MODELS)) {
        expect(typeof m.label).toBe('string');
        expect(typeof m.endpoint).toBe('string');
        expect(Array.isArray(m.fields)).toBe(true);
        expect(m.fields.length).toBeGreaterThan(0);
        expect(typeof m.buildBody).toBe('function');
        expect(typeof m.extractTerminalStats).toBe('function');
        // every field has key, label, default.
        for (const f of m.fields) {
            expect(typeof f.key, `${id}.${f.key} missing key`).toBe('string');
            expect(typeof f.label, `${id}.${f.key} missing label`).toBe('string');
            expect(f.default).toBeDefined();
        }
    }
});

test('endpoint names match the api.js wrapper convention', () => {
    for (const m of Object.values(MODELS)) {
        expect(m.endpoint.startsWith('anly')).toBe(true);
    }
});

// ── defaultValues ────────────────────────────────────────────────────

test('defaultValues returns every field with its default', () => {
    for (const [id, m] of Object.entries(MODELS)) {
        const v = defaultValues(id);
        for (const f of m.fields) {
            expect(v[f.key]).toBe(f.default);
        }
    }
});

test('defaultValues returns a fresh object per call (no shared state)', () => {
    const a = defaultValues('gbm');
    a.s0 = 999;
    const b = defaultValues('gbm');
    expect(b.s0).not.toBe(999);
});

// ── validateValues ───────────────────────────────────────────────────

test('validateValues rejects unknown model id', () => {
    expect(validateValues('nope', {})).toMatch(/unknown model/);
});

test('validateValues rejects missing required field', () => {
    const v = defaultValues('gbm');
    delete v.s0;
    expect(validateValues('gbm', v)).toMatch(/Spot/);
});

test('validateValues rejects NaN / infinite values', () => {
    const v = defaultValues('gbm');
    v.s0 = NaN;
    expect(validateValues('gbm', v)).toMatch(/finite/);
    v.s0 = Infinity;
    expect(validateValues('gbm', v)).toMatch(/finite/);
});

test('validateValues enforces integer fields', () => {
    const v = defaultValues('gbm');
    v.steps = 252.5;
    expect(validateValues('gbm', v)).toMatch(/integer/);
});

test('validateValues enforces min/max bounds', () => {
    const v = defaultValues('kou_jump');
    v.up_prob = 1.5;
    expect(validateValues('kou_jump', v)).toMatch(/≤ 1/);
    v.up_prob = 0.5;
    v.eta_up = 0.5;       // must be > 1
    expect(validateValues('kou_jump', v)).toMatch(/η up/);
});

test('validateValues returns null on good defaults for all models', () => {
    for (const id of Object.keys(MODELS)) {
        expect(validateValues(id, defaultValues(id))).toBe(null);
    }
});

// ── buildBody ────────────────────────────────────────────────────────

test('gbm buildBody includes all canonical GBM fields', () => {
    const body = MODELS.gbm.buildBody(defaultValues('gbm'));
    expect(body).toHaveProperty('s0');
    expect(body).toHaveProperty('mu');
    expect(body).toHaveProperty('sigma');
    expect(body).toHaveProperty('dt');
    expect(body).toHaveProperty('steps');
    expect(body).toHaveProperty('paths');
    expect(body).toHaveProperty('seed');
});

test('merton_jump buildBody includes jump parameters', () => {
    const body = MODELS.merton_jump.buildBody(defaultValues('merton_jump'));
    expect(body).toHaveProperty('jump_lambda');
    expect(body).toHaveProperty('jump_mean');
    expect(body).toHaveProperty('jump_stdev');
});

test('kou_jump buildBody uses up_prob / eta_up / eta_down', () => {
    const body = MODELS.kou_jump.buildBody(defaultValues('kou_jump'));
    expect(body).toHaveProperty('up_prob');
    expect(body).toHaveProperty('eta_up');
    expect(body).toHaveProperty('eta_down');
});

test('fbm buildBody uses hurst / sigma0 / levels / seed', () => {
    const body = MODELS.fbm.buildBody(defaultValues('fbm'));
    expect(body).toHaveProperty('hurst');
    expect(body).toHaveProperty('sigma0');
    expect(body).toHaveProperty('levels');
    expect(body).toHaveProperty('seed');
});

// ── extractTerminalStats ─────────────────────────────────────────────

test('gbm extractTerminalStats maps mean_terminal → mean', () => {
    const s = MODELS.gbm.extractTerminalStats({
        mean_terminal: 105, stdev_terminal: 20,
        min_terminal: 50, max_terminal: 200, paths_run: 1000,
    });
    expect(s.mean).toBe(105);
    expect(s.stdev).toBe(20);
    expect(s.min).toBe(50);
    expect(s.max).toBe(200);
    expect(s.paths_run).toBe(1000);
});

test('merton_jump extracts skew + total jump count', () => {
    const s = MODELS.merton_jump.extractTerminalStats({
        mean_terminal: 100, stdev_terminal: 20,
        mean_log_return: 0.0, skew_log_return: -0.5,
        jump_count_total: 42, paths_run: 1000,
    });
    expect(s.skew).toBe(-0.5);
    expect(s.extra['Jump count (total)']).toBe(42);
});

test('kou_jump separates up/down jump counts', () => {
    const s = MODELS.kou_jump.extractTerminalStats({
        mean_terminal: 100, stdev_terminal: 20,
        mean_log_return: 0.0, skew_log_return: -0.5,
        up_jumps: 12, down_jumps: 30, paths_run: 1000,
    });
    expect(s.extra['Up jumps']).toBe(12);
    expect(s.extra['Down jumps']).toBe(30);
});

test('fbm extractTerminalStats derives mean/stdev/min/max from the path', () => {
    const path = [0, 1, 2, 3, 4];
    const s = MODELS.fbm.extractTerminalStats(path);
    expect(s.mean).toBe(2);
    expect(s.min).toBe(0);
    expect(s.max).toBe(4);
    expect(s.path).toBe(path);
});

test('fbm extractTerminalStats returns null on empty / non-array', () => {
    expect(MODELS.fbm.extractTerminalStats([])).toBe(null);
    expect(MODELS.fbm.extractTerminalStats(null)).toBe(null);
    expect(MODELS.fbm.extractTerminalStats('not a path')).toBe(null);
});

// ── normalDensityCurve ───────────────────────────────────────────────

test('normalDensityCurve emits the requested number of points', () => {
    const { xs, ys } = normalDensityCurve(100, 10, 51);
    expect(xs.length).toBe(51);
    expect(ys.length).toBe(51);
});

test('normalDensityCurve spans approximately ±4σ around mean', () => {
    const { xs } = normalDensityCurve(100, 10, 9);
    expect(xs[0]).toBeCloseTo(60, 6);
    expect(xs[xs.length - 1]).toBeCloseTo(140, 6);
});

test('normalDensityCurve peaks at the mean', () => {
    const { xs, ys } = normalDensityCurve(0, 1, 101);
    const peakIdx = ys.indexOf(Math.max(...ys));
    expect(Math.abs(xs[peakIdx])).toBeLessThan(0.1);
});

test('normalDensityCurve area integrates to ≈ 1', () => {
    const { xs, ys } = normalDensityCurve(0, 1, 1001);
    const dx = xs[1] - xs[0];
    let area = 0;
    for (let i = 0; i < ys.length - 1; i++) area += 0.5 * (ys[i] + ys[i + 1]) * dx;
    expect(area).toBeCloseTo(1, 2);
});

test('normalDensityCurve returns empty arrays for invalid stdev', () => {
    expect(normalDensityCurve(100, 0).xs).toEqual([]);
    expect(normalDensityCurve(100, NaN).xs).toEqual([]);
    expect(normalDensityCurve(100, -1).xs).toEqual([]);
});
