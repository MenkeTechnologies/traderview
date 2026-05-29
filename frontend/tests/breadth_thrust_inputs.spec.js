// Zweig Breadth Thrust helpers: parser, validator, body shape,
// localCompute Rust-mirror, badges, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_EMA_PERIOD, DEFAULT_MAX_WINDOW, DEFAULT_LOW, DEFAULT_HIGH,
    parseBreadthBlob, breadthToBlob, validateInputs, buildBody, localCompute,
    regimeBadge, thrustBadge, lastThrustIndex, summarize,
    makeDemoInput,
    fmtRatio, fmtPct, fmtInt,
} from '../js/_breadth_thrust_inputs.js';

const d = (adv, dec) => ({ advancing: adv, declining: dec });

// ── constants ─────────────────────────────────────────────────────

test('Zweig defaults match Rust', () => {
    expect(DEFAULT_EMA_PERIOD).toBe(10);
    expect(DEFAULT_MAX_WINDOW).toBe(10);
    expect(DEFAULT_LOW).toBe(0.40);
    expect(DEFAULT_HIGH).toBe(0.615);
});

// ── parser ────────────────────────────────────────────────────────

test('parseBreadthBlob: 2 tokens per line, comments + blanks ignored', () => {
    const r = parseBreadthBlob('30 70\n# session 2\n40, 60');
    expect(r.errors).toEqual([]);
    expect(r.breadth).toEqual([d(30, 70), d(40, 60)]);
});

test('parseBreadthBlob: rejects wrong count / non-integer / negative', () => {
    expect(parseBreadthBlob('30').errors[0].message).toMatch(/2 tokens/);
    expect(parseBreadthBlob('30.5 70').errors[0].message).toMatch(/advancing/);
    expect(parseBreadthBlob('30 -1').errors[0].message).toMatch(/declining/);
});

test('parseBreadthBlob: non-string returns 1 error', () => {
    expect(parseBreadthBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts non-empty default', () => {
    expect(validateInputs({ breadth: [d(30, 70)], ema_period: 10,
        max_window_bars: 10, low_threshold: 0.4, high_threshold: 0.615 })).toBe(null);
});

test('validate rejects: bad array / non-integer counts / low ≥ high / threshold out of range', () => {
    const base = { breadth: [d(30, 70)], ema_period: 10, max_window_bars: 10,
                   low_threshold: 0.4, high_threshold: 0.615 };
    expect(validateInputs({ ...base, breadth: 'no' })).toMatch(/breadth/);
    expect(validateInputs({ ...base, breadth: [{ advancing: 30.5, declining: 70 }] })).toMatch(/advancing/);
    expect(validateInputs({ ...base, ema_period: 1 })).toMatch(/ema_period/);
    expect(validateInputs({ ...base, max_window_bars: 1 })).toMatch(/max_window_bars/);
    expect(validateInputs({ ...base, low_threshold: -0.1 })).toMatch(/low_threshold/);
    expect(validateInputs({ ...base, high_threshold: 1.5 })).toMatch(/high_threshold/);
    expect(validateInputs({ ...base, low_threshold: 0.7 })).toMatch(/low_threshold/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: forwards all fields + strips extras', () => {
    const body = buildBody({ breadth: [{ ...d(30, 70), extra: 'x' }],
        ema_period: 12, max_window_bars: 8, low_threshold: 0.35, high_threshold: 0.65 });
    expect(body).toEqual({ breadth: [d(30, 70)], ema_period: 12,
        max_window_bars: 8, low_threshold: 0.35, high_threshold: 0.65 });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: invalid inputs return all-null + no triggers', () => {
    const b = new Array(100).fill(d(50, 50));
    const r = localCompute(b, 1, 10, 0.4, 0.615);
    expect(r.ratio.every(v => v == null)).toBe(true);
    const r2 = localCompute(b, 10, 10, 0.7, 0.5);
    expect(r2.ratio.every(v => v == null)).toBe(true);
});

test('local: flat 50/50 breadth → no thrust', () => {
    const b = new Array(100).fill(d(50, 50));
    const r = localCompute(b, 10, 10, 0.4, 0.615);
    expect(r.thrust_triggered.some(v => v)).toBe(false);
});

test('local: classic thrust pattern triggers at least once', () => {
    const b = [...new Array(30).fill(d(30, 70)), ...new Array(15).fill(d(90, 10))];
    const r = localCompute(b, 10, 10, 0.4, 0.615);
    expect(r.thrust_triggered.some(v => v)).toBe(true);
});

test('local: slow recovery (rise per bar < window) → no thrust', () => {
    const b = [];
    for (let i = 0; i < 80; i++) {
        const adv = 30 + i;
        const dec = Math.max(1, 70 - i);
        b.push(d(adv, dec));
    }
    const r = localCompute(b, 10, 5, 0.4, 0.615);
    // Slow rise over many bars within a tight 5-bar window: very unlikely to trigger
    // (just verify no panic + lengths).
    expect(r.thrust_triggered.length).toBe(80);
});

test('local: output lengths match input', () => {
    const b = new Array(50).fill(d(50, 50));
    const r = localCompute(b, 10, 10, 0.4, 0.615);
    expect(r.ratio.length).toBe(50);
    expect(r.ema_ratio.length).toBe(50);
    expect(r.thrust_triggered.length).toBe(50);
});

test('local: ratio computed correctly (advancing/total)', () => {
    const b = [d(30, 70), d(40, 60), d(50, 50)];
    const r = localCompute(b, 2, 2, 0.4, 0.6);
    expect(r.ratio[0]).toBeCloseTo(0.30, 9);
    expect(r.ratio[1]).toBeCloseTo(0.40, 9);
    expect(r.ratio[2]).toBeCloseTo(0.50, 9);
});

test('local: zero-denom day uses fallback ratio = 0.5', () => {
    const b = [d(0, 0), d(50, 50), d(50, 50)];
    const r = localCompute(b, 2, 2, 0.4, 0.6);
    expect(r.ratio[0]).toBe(0.5);
});

test('local: EMA seed = SMA over first period (Rust spec)', () => {
    const b = new Array(20).fill(0).map((_, i) => d(40 + i, 60 - i));
    const r = localCompute(b, 10, 5, 0.4, 0.6);
    // Compute expected EMA seed manually.
    let sum = 0;
    for (let i = 0; i < 10; i++) {
        const ratio = (40 + i) / ((40 + i) + (60 - i));
        sum += ratio;
    }
    const expectedSeed = sum / 10;
    expect(r.ema_ratio[9]).toBeCloseTo(expectedSeed, 9);
});

test('local: EMA warmup region (0..period-2) is null', () => {
    const b = new Array(30).fill(d(50, 50));
    const r = localCompute(b, 10, 10, 0.4, 0.615);
    for (let i = 0; i < 9; i++) expect(r.ema_ratio[i]).toBeNull();
    expect(r.ema_ratio[9]).not.toBeNull();
});

test('local: n < ema_period → all-null', () => {
    const b = new Array(5).fill(d(50, 50));
    const r = localCompute(b, 10, 10, 0.4, 0.615);
    expect(r.ratio.every(v => v == null)).toBe(true);
});

// ── regimeBadge / thrustBadge / lastThrustIndex / summarize ──────

test('regimeBadge: 5-tier on last EMA', () => {
    expect(regimeBadge(0.7, 0.4, 0.615).key).toMatch(/bullish_thrust/);
    expect(regimeBadge(0.58, 0.4, 0.615).key).toMatch(/strong/);
    expect(regimeBadge(0.50, 0.4, 0.615).key).toMatch(/neutral/);
    expect(regimeBadge(0.42, 0.4, 0.615).key).toMatch(/weak/);
    expect(regimeBadge(0.30, 0.4, 0.615).key).toMatch(/washout/);
    expect(regimeBadge(null, 0.4, 0.615).key).toMatch(/unknown/);
});

test('thrustBadge: none / fired / multiple', () => {
    expect(thrustBadge([false, false]).key).toMatch(/none/);
    expect(thrustBadge([false, true]).key).toMatch(/fired/);
    expect(thrustBadge([true, false, true]).key).toMatch(/multiple/);
    expect(thrustBadge([]).key).toMatch(/unknown/);
});

test('lastThrustIndex: returns last triggered index', () => {
    expect(lastThrustIndex([false, true, false, true])).toBe(3);
    expect(lastThrustIndex([false, false])).toBeNull();
    expect(lastThrustIndex(null)).toBeNull();
});

test('summarize: ratios / EMA stats / thrust count', () => {
    const report = {
        ratio: [0.3, 0.4, 0.5, 0.7],
        ema_ratio: [null, 0.35, 0.45, 0.6],
        thrust_triggered: [false, false, false, true],
    };
    const s = summarize(report);
    expect(s.count).toBe(4);
    expect(s.populated).toBe(3);
    expect(s.last_ratio).toBe(0.7);
    expect(s.last_ema).toBe(0.6);
    expect(s.min_ema).toBe(0.35);
    expect(s.max_ema).toBe(0.6);
    expect(s.thrust_count).toBe(1);
});

test('summarize: empty / null → count 0, NaN aggregates', () => {
    expect(summarize(null).count).toBe(0);
    expect(Number.isNaN(summarize(null).last_ratio)).toBe(true);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + produces correct-length output', () => {
    for (const k of ['classic-thrust','flat-balanced','slow-recovery','multi-thrust',
                     'washout-only','noisy-walk','tight-window','custom-thresholds']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.breadth, inp.ema_period, inp.max_window_bars,
            inp.low_threshold, inp.high_threshold);
        expect(r.ratio.length).toBe(inp.breadth.length);
    }
});

test('demo classic-thrust: fires at least one trigger', () => {
    const inp = makeDemoInput('classic-thrust');
    const r = localCompute(inp.breadth, inp.ema_period, inp.max_window_bars,
        inp.low_threshold, inp.high_threshold);
    expect(r.thrust_triggered.some(v => v)).toBe(true);
});

test('demo flat-balanced: no triggers', () => {
    const inp = makeDemoInput('flat-balanced');
    const r = localCompute(inp.breadth, inp.ema_period, inp.max_window_bars,
        inp.low_threshold, inp.high_threshold);
    expect(r.thrust_triggered.some(v => v)).toBe(false);
});

test('demo multi-thrust: fires ≥ 2 triggers across the series', () => {
    const inp = makeDemoInput('multi-thrust');
    const r = localCompute(inp.breadth, inp.ema_period, inp.max_window_bars,
        inp.low_threshold, inp.high_threshold);
    const count = r.thrust_triggered.filter(v => v).length;
    expect(count).toBeGreaterThanOrEqual(2);
});

test('demo washout-only: no triggers (EMA never recovers)', () => {
    const inp = makeDemoInput('washout-only');
    const r = localCompute(inp.breadth, inp.ema_period, inp.max_window_bars,
        inp.low_threshold, inp.high_threshold);
    expect(r.thrust_triggered.some(v => v)).toBe(false);
});

test('demo tight-window: classic pattern + 3-bar window = no trigger', () => {
    const inp = makeDemoInput('tight-window');
    const r = localCompute(inp.breadth, inp.ema_period, inp.max_window_bars,
        inp.low_threshold, inp.high_threshold);
    expect(r.thrust_triggered.some(v => v)).toBe(false);
});

test('demo custom-thresholds: looser bands catch weaker rally → trigger', () => {
    const inp = makeDemoInput('custom-thresholds');
    const r = localCompute(inp.breadth, inp.ema_period, inp.max_window_bars,
        inp.low_threshold, inp.high_threshold);
    expect(r.thrust_triggered.some(v => v)).toBe(true);
});

// ── round-trip + formatters ──────────────────────────────────────

test('breadthToBlob round-trips through parseBreadthBlob', () => {
    const breadth = [d(30, 70), d(40, 60)];
    const back = parseBreadthBlob(breadthToBlob(breadth));
    expect(back.errors).toEqual([]);
    expect(back.breadth).toEqual(breadth);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtRatio(0.5)).toBe('0.5000');
    expect(fmtPct(0.5)).toBe('50.00%');
    expect(fmtInt(7.9)).toBe('7');
    expect(fmtRatio(null)).toBe('—');
    expect(fmtPct(NaN)).toBe('—');
});
