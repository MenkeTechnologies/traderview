// Absorption Detector helpers: parser, validator, localCompute parity, badges, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, DEFAULT_PERIOD, DEFAULT_THRESHOLD, DEFAULT_VOL_MULTIPLIER,
    MIN_PERIOD, MAX_PERIOD,
    parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute,
    lastSignalBadge, biasBadge, intensityBadge, summarizeBars,
    makeDemoInput,
    fmtPrice, fmtInt, fmtPct, fmtRatio,
} from '../js/_absorption_inputs.js';

const b = (h, l, c, v) => ({ high: h, low: l, close: c, volume: v });

// ── parser ────────────────────────────────────────────────────────

test('parseBarsBlob: 4 tokens per line', () => {
    const r = parseBarsBlob('101 99 100 1000\n100.5 99.5 100 800');
    expect(r.errors).toEqual([]);
    expect(r.bars).toEqual([b(101, 99, 100, 1000), b(100.5, 99.5, 100, 800)]);
});

test('parseBarsBlob: rejects wrong count / OHL violations / non-positive volume', () => {
    expect(parseBarsBlob('101 99 100').errors[0].message).toMatch(/4 tokens/);
    expect(parseBarsBlob('99 101 100 500').errors[0].message).toMatch(/high < low/);
    expect(parseBarsBlob('101 99 200 500').errors[0].message).toMatch(/close outside/);
    expect(parseBarsBlob('101 99 100 0').errors[0].message).toMatch(/volume must be/);
});

test('parseBarsBlob: comments + blank lines', () => {
    const r = parseBarsBlob('# header\n101 99 100 1000\n\n');
    expect(r.errors).toEqual([]);
    expect(r.bars.length).toBe(1);
});

test('parseBarsBlob: non-string returns 1 error', () => {
    expect(parseBarsBlob(undefined).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts valid', () => {
    const bars = Array.from({ length: 25 }, () => b(101, 99, 100, 1000));
    expect(validateInputs({ ...DEFAULT_INPUTS, bars })).toBe(null);
});

test('validate rejects bad shape / period / threshold / vol / bars', () => {
    const ok = Array.from({ length: 25 }, () => b(101, 99, 100, 1000));
    expect(validateInputs({ bars: 'no', period: 20, threshold: 0.5, vol_multiplier: 1.5 })).toMatch(/bars/);
    expect(validateInputs({ bars: ok, period: 1, threshold: 0.5, vol_multiplier: 1.5 })).toMatch(/period/);
    expect(validateInputs({ bars: ok, period: 9999, threshold: 0.5, vol_multiplier: 1.5 })).toMatch(/period/);
    expect(validateInputs({ bars: ok.slice(0, 5), period: 20, threshold: 0.5, vol_multiplier: 1.5 })).toMatch(/period/);
    expect(validateInputs({ bars: ok, period: 20, threshold: 0, vol_multiplier: 1.5 })).toMatch(/threshold/);
    expect(validateInputs({ bars: ok, period: 20, threshold: 0.5, vol_multiplier: 0 })).toMatch(/vol_multiplier/);
    const bad = [...ok];
    bad[5] = b(101, 99, 100, 0);
    expect(validateInputs({ bars: bad, period: 20, threshold: 0.5, vol_multiplier: 1.5 })).toMatch(/volume/);
    const nanb = [...ok];
    nanb[5] = b(NaN, 99, 100, 1000);
    expect(validateInputs({ bars: nanb, period: 20, threshold: 0.5, vol_multiplier: 1.5 })).toMatch(/finite/);
    const inv = [...ok];
    inv[5] = b(99, 101, 100, 500);
    expect(validateInputs({ bars: inv, period: 20, threshold: 0.5, vol_multiplier: 1.5 })).toMatch(/high < low/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody shapes payload', () => {
    const body = buildBody({ bars: [{ high: 101, low: 99, close: 100, volume: 1000, extra: 'x' }],
                             period: 20, threshold: 0.5, vol_multiplier: 1.5 });
    expect(body).toEqual({ bars: [b(101, 99, 100, 1000)], period: 20, threshold: 0.5, vol_multiplier: 1.5 });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: invalid period / threshold returns empty signals', () => {
    const bars = Array.from({ length: 30 }, () => b(101, 99, 100, 1000));
    expect(localCompute(bars, 1, 0.5, 1.5).bullish.some(Boolean)).toBe(false);
    expect(localCompute(bars, 20, 0, 1.5).bullish.some(Boolean)).toBe(false);
});

test('local: NaN or zero volume returns empty', () => {
    const bars = Array.from({ length: 30 }, () => b(101, 99, 100, 1000));
    bars[5] = b(NaN, 99, 100, 1000);
    expect(localCompute(bars, 20, 0.5, 1.5).bullish.some(Boolean)).toBe(false);
    const bars2 = Array.from({ length: 30 }, () => b(101, 99, 100, 1000));
    bars2[5] = b(101, 99, 100, 0);
    expect(localCompute(bars2, 20, 0.5, 1.5).bullish.some(Boolean)).toBe(false);
});

test('local: flat market no signal', () => {
    const bars = Array.from({ length: 30 }, () => b(101, 99, 100, 1000));
    const r = localCompute(bars, 20, 0.5, 1.5);
    expect(r.bullish.some(Boolean)).toBe(false);
    expect(r.bearish.some(Boolean)).toBe(false);
});

test('local: bullish absorption detected', () => {
    const bars = Array.from({ length: 25 }, () => b(101, 99, 100, 1000));
    bars.push(b(100.9, 99.9, 100.9, 10000));
    const r = localCompute(bars, 20, 0.5, 1.5);
    expect(r.bullish[25]).toBe(true);
    expect(r.bearish[25]).toBe(false);
});

test('local: bearish absorption detected', () => {
    const bars = Array.from({ length: 25 }, () => b(101, 99, 100, 1000));
    bars.push(b(100.1, 99.1, 99.1, 10000));
    const r = localCompute(bars, 20, 0.5, 1.5);
    expect(r.bearish[25]).toBe(true);
    expect(r.bullish[25]).toBe(false);
});

test('local: normal volume → no absorption', () => {
    const bars = Array.from({ length: 25 }, () => b(101, 99, 100, 1000));
    bars.push(b(100.9, 99.9, 100.9, 1000));
    const r = localCompute(bars, 20, 0.5, 1.5);
    expect(r.bullish[25]).toBe(false);
});

test('local: output length matches input', () => {
    const bars = Array.from({ length: 30 }, () => b(101, 99, 100, 1000));
    const r = localCompute(bars, 20, 0.5, 1.5);
    expect(r.bullish.length).toBe(30);
    expect(r.bearish.length).toBe(30);
});

test('local: leading false until period', () => {
    const bars = Array.from({ length: 25 }, () => b(101, 99, 100, 1000));
    bars.push(b(100.9, 99.9, 100.9, 10000));
    const r = localCompute(bars, 20, 0.5, 1.5);
    for (let i = 0; i < 20; i++) {
        expect(r.bullish[i]).toBe(false);
        expect(r.bearish[i]).toBe(false);
    }
});

test('local: deterministic', () => {
    const bars = Array.from({ length: 30 }, () => b(101, 99, 100, 1000));
    expect(localCompute(bars, 20, 0.5, 1.5)).toEqual(localCompute(bars, 20, 0.5, 1.5));
});

// ── badges ────────────────────────────────────────────────────────

test('lastSignalBadge: bullish / bearish / none / unknown', () => {
    expect(lastSignalBadge(null).key).toMatch(/unknown/);
    expect(lastSignalBadge({ bullish: [false, false], bearish: [false, false] }).key).toMatch(/none/);
    expect(lastSignalBadge({ bullish: [false, true],  bearish: [false, false] }).key).toMatch(/bullish/);
    expect(lastSignalBadge({ bullish: [false, false], bearish: [false, true]  }).key).toMatch(/bearish/);
});

test('lastSignalBadge: barsAgo populated', () => {
    const r = lastSignalBadge({ bullish: [false, true, false, false], bearish: [false, false, false, false] });
    expect(r.barsAgo).toBe(2);
});

test('biasBadge: tiers', () => {
    expect(biasBadge({ bullish: [false], bearish: [false] }).key).toMatch(/flat/);
    expect(biasBadge({ bullish: [true, true], bearish: [false, false] }).key).toMatch(/all_bull/);
    expect(biasBadge({ bullish: [false, false], bearish: [true, true] }).key).toMatch(/all_bear/);
    expect(biasBadge({ bullish: [true, true, true], bearish: [false, true, false] }).key).toMatch(/bull_lean/);
    expect(biasBadge({ bullish: [false, true, false], bearish: [true, true, true] }).key).toMatch(/bear_lean/);
    expect(biasBadge({ bullish: [true], bearish: [true] }).key).toMatch(/balanced/);
});

test('intensityBadge: 5 tiers', () => {
    const mk = (n, sig) => {
        const bullish = new Array(n).fill(false);
        for (let i = 0; i < sig; i++) bullish[i] = true;
        return { bullish, bearish: new Array(n).fill(false) };
    };
    expect(intensityBadge(mk(100, 0)).key).toMatch(/silent/);
    expect(intensityBadge(mk(100, 1)).key).toMatch(/rare/);
    expect(intensityBadge(mk(100, 3)).key).toMatch(/normal/);
    expect(intensityBadge(mk(100, 7)).key).toMatch(/busy/);
    expect(intensityBadge(mk(100, 15)).key).toMatch(/flooded/);
});

// ── summarize ─────────────────────────────────────────────────────

test('summarizeBars: count / extrema / volume stats', () => {
    const bars = [b(101, 99, 100, 1000), b(102, 99, 101, 2000), b(103, 100, 102, 500)];
    const s = summarizeBars(bars);
    expect(s.count).toBe(3);
    expect(s.last_close).toBe(102);
    expect(s.min_low).toBe(99);
    expect(s.max_high).toBe(103);
    expect(s.vol_min).toBe(500);
    expect(s.vol_max).toBe(2000);
    expect(s.vol_avg).toBeCloseTo(3500 / 3, 6);
});

test('summarizeBars: empty', () => {
    const s = summarizeBars([]);
    expect(s.count).toBe(0);
    expect(Number.isNaN(s.last_close)).toBe(true);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: every preset validates', () => {
    for (const k of ['flat', 'bullish-absorb', 'bearish-absorb', 'normal-volume',
                     'multi-absorb', 'noisy', 'short-period', 'tight-thresh']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
    }
});

test('demo flat: zero signals', () => {
    const inp = makeDemoInput('flat');
    const r = localCompute(inp.bars, inp.period, inp.threshold, inp.vol_multiplier);
    expect(r.bullish.some(Boolean)).toBe(false);
    expect(r.bearish.some(Boolean)).toBe(false);
});

test('demo bullish-absorb: last bar bullish', () => {
    const inp = makeDemoInput('bullish-absorb');
    const r = localCompute(inp.bars, inp.period, inp.threshold, inp.vol_multiplier);
    expect(r.bullish[r.bullish.length - 1]).toBe(true);
});

test('demo bearish-absorb: last bar bearish', () => {
    const inp = makeDemoInput('bearish-absorb');
    const r = localCompute(inp.bars, inp.period, inp.threshold, inp.vol_multiplier);
    expect(r.bearish[r.bearish.length - 1]).toBe(true);
});

test('demo normal-volume: zero signals', () => {
    const inp = makeDemoInput('normal-volume');
    const r = localCompute(inp.bars, inp.period, inp.threshold, inp.vol_multiplier);
    expect(r.bullish.some(Boolean)).toBe(false);
    expect(r.bearish.some(Boolean)).toBe(false);
});

test('demo multi-absorb: at least one bull and one bear', () => {
    const inp = makeDemoInput('multi-absorb');
    const r = localCompute(inp.bars, inp.period, inp.threshold, inp.vol_multiplier);
    expect(r.bullish.some(Boolean)).toBe(true);
    expect(r.bearish.some(Boolean)).toBe(true);
});

test('demo tight-thresh: harder to trigger but still detects', () => {
    const inp = makeDemoInput('tight-thresh');
    const r = localCompute(inp.bars, inp.period, inp.threshold, inp.vol_multiplier);
    expect(r.bullish.some(Boolean) || r.bearish.some(Boolean)).toBe(true);
});

// ── formatters / roundtrip ────────────────────────────────────────

test('barsToBlob round-trips', () => {
    const bars = [b(101, 99, 100, 1000), b(102, 99, 101, 2000)];
    const back = parseBarsBlob(barsToBlob(bars));
    expect(back.errors).toEqual([]);
    expect(back.bars).toEqual(bars);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtPrice(100.456)).toBe('100.46');
    expect(fmtInt(42.9)).toBe('42');
    expect(fmtPct(0.125)).toBe('12.5%');
    expect(fmtRatio(0.7654)).toBe('0.765');
    expect(fmtPrice(NaN)).toBe('—');
    expect(fmtInt(Infinity)).toBe('—');
    expect(fmtPct(NaN)).toBe('—');
    expect(fmtRatio(NaN)).toBe('—');
});

test('DEFAULTS sanity', () => {
    expect(DEFAULT_PERIOD).toBe(20);
    expect(DEFAULT_THRESHOLD).toBe(0.5);
    expect(DEFAULT_VOL_MULTIPLIER).toBe(1.5);
    expect(MIN_PERIOD).toBe(2);
    expect(MAX_PERIOD).toBe(500);
});
