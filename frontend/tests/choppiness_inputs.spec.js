// Choppiness Index helpers: parser re-export, validator, body shape,
// local Rust-mirror, true-range, regime classification, buckets +
// regime-switch detection, demos, formatters.

import { test, expect } from 'vitest';
import {
    parseBarBlob, validateInputs, buildBody, localCompute, trueRangeAt,
    regimeBadge, regimeBuckets, lastRegimeSwitch, makeDemoBars,
    fmtN, fmtPct,
} from '../js/_choppiness_inputs.js';

const b = (h, l, c) => ({ high: h, low: l, close: c });

// ── parser re-export still works ──────────────────────────────────

test('parser (re-exported) accepts HLC triples', () => {
    const r = parseBarBlob('100 99 99.5\n101 99 100');
    expect(r.errors).toEqual([]);
    expect(r.bars.length).toBe(2);
});

// ── validateInputs ────────────────────────────────────────────────

test('validate accepts good inputs', () => {
    expect(validateInputs(Array(15).fill(b(100, 99, 99.5)), 14)).toBe(null);
});

test('validate rejects empty bars / bad period / too few bars', () => {
    expect(validateInputs([], 14)).toMatch(/≥ 1 bar/);
    expect(validateInputs([b(1, 1, 1)], 1)).toMatch(/period must be/);
    expect(validateInputs([b(1, 1, 1)], 14)).toMatch(/period \+ 1/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody emits backend ChoppinessBody shape', () => {
    const bars = [b(100, 99, 99.5)];
    expect(buildBody(bars, 14)).toEqual({ bars, period: 14 });
});

// ── trueRangeAt ───────────────────────────────────────────────────

test('trueRangeAt: bar 0 = high - low (no prev close)', () => {
    expect(trueRangeAt([b(100, 99, 99.5)], 0)).toBe(1);
});

test('trueRangeAt: bar i picks max of H-L, |H-pc|, |L-pc|', () => {
    // Prev close = 100. Today H=105, L=99 → ranges: H-L=6, H-pc=5, |L-pc|=1.
    const bars = [b(101, 99, 100), b(105, 99, 102)];
    expect(trueRangeAt(bars, 1)).toBe(6);
    // Gap up scenario: prev close = 100, today H=110 L=108 → H-L=2, H-pc=10, |L-pc|=8 → max=10.
    const gap = [b(101, 99, 100), b(110, 108, 109)];
    expect(trueRangeAt(gap, 1)).toBe(10);
});

// ── localCompute parity (one test per Rust test) ──────────────────

test('local: too few bars returns all-null series + warmup note', () => {
    const r = localCompute([b(100, 99, 99.5)], 14);
    expect(r.latest).toBeNull();
    expect(r.note).toMatch(/need/);
    expect(r.series).toEqual([null]);
});

test('local: period < 2 returns null series', () => {
    const bars = Array(5).fill(b(100, 99, 99.5));
    expect(localCompute(bars, 1).latest).toBeNull();
});

test('local: empty bars returns empty series + mixed regime', () => {
    const r = localCompute([], 14);
    expect(r.series).toEqual([]);
    expect(r.regime).toBe('mixed');
});

test('local: strong uptrend yields LOW CI (< 50, often Trending)', () => {
    const bars = Array.from({ length: 30 }, (_, i) => {
        const p = 100 + i;
        return b(p + 0.5, p - 0.5, p + 0.3);
    });
    const r = localCompute(bars, 14);
    expect(r.latest).toBeLessThan(50);
    expect(['trending', 'mixed']).toContain(r.regime);
});

test('local: flat oscillation yields HIGH CI (> 50, often Choppy)', () => {
    const bars = Array.from({ length: 30 }, (_, i) => {
        const p = i % 2 === 0 ? 100.5 : 99.5;
        return b(p + 0.1, p - 0.1, p);
    });
    const r = localCompute(bars, 14);
    expect(r.latest).toBeGreaterThan(50);
    expect(['choppy', 'mixed']).toContain(r.regime);
});

test('local: zero-range window returns None (degenerate guard)', () => {
    const bars = Array(20).fill(b(100, 100, 100));
    const r = localCompute(bars, 14);
    expect(r.latest).toBeNull();
});

test('local: series length always matches bars length (input-aligned invariant)', () => {
    const bars = Array(30).fill(0).map((_, i) => {
        const p = 100 + i * 0.1;
        return b(p + 0.3, p - 0.3, p);
    });
    expect(localCompute(bars, 14).series.length).toBe(30);
    expect(localCompute([b(100, 99, 99.5)], 14).series.length).toBe(1);
});

test('local: warmup bars (i < period) are null', () => {
    const bars = Array.from({ length: 30 }, (_, i) => {
        const p = 100 + i * 0.1;
        return b(p + 0.3, p - 0.3, p);
    });
    const r = localCompute(bars, 14);
    for (let i = 0; i < 14; i++) expect(r.series[i]).toBeNull();
    expect(r.series[14]).not.toBeNull();
});

// ── regimeBadge ───────────────────────────────────────────────────

test('regimeBadge maps each regime to label + class', () => {
    expect(regimeBadge('trending').cls).toBe('pos');
    expect(regimeBadge('mixed').cls).toBe('');
    expect(regimeBadge('choppy').cls).toBe('neg');
    expect(regimeBadge('unknown').label).toBe('UNKNOWN');
});

// ── regimeBuckets ─────────────────────────────────────────────────

test('regimeBuckets: null → warmup, > 61.8 → choppy, < 38.2 → trending', () => {
    const buckets = regimeBuckets([null, null, 30, 50, 70, 65, 39, 38, 0, 100]);
    expect(buckets.warmup).toBe(2);
    expect(buckets.trending).toBe(3);  // 30, 38, 0
    expect(buckets.mixed).toBe(2);      // 50, 39
    expect(buckets.choppy).toBe(3);    // 70, 65, 100
});

test('regimeBuckets: 38.2 and 61.8 boundaries → mixed', () => {
    const buckets = regimeBuckets([38.2, 61.8]);
    expect(buckets.mixed).toBe(2);
    expect(buckets.trending).toBe(0);
    expect(buckets.choppy).toBe(0);
});

// ── lastRegimeSwitch ──────────────────────────────────────────────

test('lastRegimeSwitch: detects most recent switch in series', () => {
    // bars: trending (CI ~30) for 5, then choppy (CI ~70) for 3 = switch at bar 5.
    const series = [30, 30, 30, 30, 30, 70, 70, 70];
    const evt = lastRegimeSwitch(series);
    expect(evt).toEqual({ switchedAt: 5, fromRegime: 'trending', toRegime: 'choppy' });
});

test('lastRegimeSwitch: no switch → null', () => {
    expect(lastRegimeSwitch([30, 30, 30])).toBeNull();
    expect(lastRegimeSwitch([])).toBeNull();
});

test('lastRegimeSwitch: skips null warmup bars', () => {
    const series = [null, null, 30, 30, 70, 70];
    const evt = lastRegimeSwitch(series);
    // First choppy bar (the switch boundary) is idx 4; last-trending was idx 3.
    expect(evt.switchedAt).toBe(4);
    expect(evt.fromRegime).toBe('trending');
    expect(evt.toRegime).toBe('choppy');
});

// ── demos invariants ──────────────────────────────────────────────

test('demos: each preset emits 60 valid HLC bars', () => {
    for (const k of ['trending-up', 'trending-down', 'choppy', 'mixed', 'trend-then-chop']) {
        const bars = makeDemoBars(k);
        expect(bars.length).toBe(60);
        for (const bar of bars) {
            expect(bar.high).toBeGreaterThanOrEqual(bar.low);
            expect(bar.close).toBeGreaterThanOrEqual(bar.low - 1e-9);
            expect(bar.close).toBeLessThanOrEqual(bar.high + 1e-9);
        }
    }
});

test('demo trending-up: latest CI < 50 (low value = trending)', () => {
    const r = localCompute(makeDemoBars('trending-up'), 14);
    expect(r.latest).toBeLessThan(50);
});

test('demo trending-down: same low CI for downtrend', () => {
    const r = localCompute(makeDemoBars('trending-down'), 14);
    expect(r.latest).toBeLessThan(50);
});

test('demo choppy: latest CI is in choppy or mixed band (≥ 38.2)', () => {
    const r = localCompute(makeDemoBars('choppy'), 14);
    expect(r.latest).toBeGreaterThanOrEqual(38.2);
});

test('demo trend-then-chop: regime switch detected in series', () => {
    const r = localCompute(makeDemoBars('trend-then-chop'), 14);
    const evt = lastRegimeSwitch(r.series);
    expect(evt).not.toBeNull();
});

// ── formatters ────────────────────────────────────────────────────

test('fmtN / fmtPct + non-finite guards', () => {
    expect(fmtN(42.5, 1)).toBe('42.5');
    expect(fmtN(0.5, 0)).toBe('1');
    expect(fmtN(null)).toBe('—');
    expect(fmtN(Infinity)).toBe('—');
    expect(fmtPct(0.42)).toBe('42.0%');
    expect(fmtPct(NaN)).toBe('—');
});
