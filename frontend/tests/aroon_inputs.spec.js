// Aroon Indicator helpers: parser, validator, body shape,
// localCompute Rust-mirror, badges, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_PERIOD, DEFAULT_INPUTS,
    parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute,
    regimeBadge, lastCrossover, summarize,
    makeDemoInput,
    fmtNum, fmtPct, fmtOsc, fmtInt, fmtUSD,
} from '../js/_aroon_inputs.js';

const b = (h, l) => ({ high: h, low: l });

// ── constants ─────────────────────────────────────────────────────

test('DEFAULT_PERIOD = 25 (Chande original)', () => {
    expect(DEFAULT_PERIOD).toBe(25);
});

// ── parser ────────────────────────────────────────────────────────

test('parseBarsBlob: 2 tokens per line; comments + blanks ignored', () => {
    const r = parseBarsBlob('101 99\n# mid\n102, 100');
    expect(r.errors).toEqual([]);
    expect(r.bars).toEqual([b(101, 99), b(102, 100)]);
});

test('parseBarsBlob: rejects wrong count / non-finite / high<low', () => {
    expect(parseBarsBlob('101').errors[0].message).toMatch(/2 tokens/);
    expect(parseBarsBlob('foo 99').errors[0].message).toMatch(/non-finite/);
    expect(parseBarsBlob('98 100').errors[0].message).toMatch(/high < low/);
});

test('parseBarsBlob: non-string returns 1 error', () => {
    expect(parseBarsBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts non-empty default', () => {
    expect(validateInputs({ bars: [b(101, 99)], period: 25 })).toBe(null);
});

test('validate rejects: bad array / NaN / high<low / period < 2', () => {
    expect(validateInputs({ bars: 'no', period: 25 })).toMatch(/bars/);
    expect(validateInputs({ bars: [b(NaN, 99)], period: 25 })).toMatch(/high/);
    expect(validateInputs({ bars: [b(98, 100)], period: 25 })).toMatch(/high/);
    expect(validateInputs({ bars: [b(101, 99)], period: 1 })).toMatch(/period/);
    expect(validateInputs({ bars: [b(101, 99)], period: 1.5 })).toMatch(/integer/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: strips extras + preserves period', () => {
    const body = buildBody({ bars: [{ ...b(101, 99), extra: 'x' }], period: 14 });
    expect(body).toEqual({ bars: [b(101, 99)], period: 14 });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: empty → empty arrays', () => {
    const r = localCompute([], 25);
    expect(r.aroon_up).toEqual([]);
    expect(r.aroon_down).toEqual([]);
    expect(r.aroon_oscillator).toEqual([]);
});

test('local: period < 2 → all-null', () => {
    const bars = Array.from({ length: 30 }, () => b(101, 99));
    const r = localCompute(bars, 1);
    expect(r.aroon_up.every(v => v == null)).toBe(true);
});

test('local: too-short series → all-null', () => {
    const bars = Array.from({ length: 5 }, () => b(101, 99));
    const r = localCompute(bars, 25);
    expect(r.aroon_up.every(v => v == null)).toBe(true);
});

test('local: strict uptrend → AroonUp=100, AroonDown=0 at tail', () => {
    const bars = Array.from({ length: 30 }, (_, i) => b(100 + i, 99 + i));
    const r = localCompute(bars, 25);
    expect(r.aroon_up[29]).toBeCloseTo(100, 9);
    expect(r.aroon_down[29]).toBeCloseTo(0, 9);
});

test('local: strict downtrend → AroonDown=100, AroonUp=0', () => {
    const bars = Array.from({ length: 30 }, (_, i) => b(100 - i, 99 - i));
    const r = localCompute(bars, 25);
    expect(r.aroon_down[29]).toBeCloseTo(100, 9);
    expect(r.aroon_up[29]).toBeCloseTo(0, 9);
});

test('local: oscillator = Up − Down', () => {
    const bars = Array.from({ length: 30 }, (_, i) => b(100 + (i % 5), 99 + (i % 5)));
    const r = localCompute(bars, 25);
    for (let i = 25; i < 30; i++) {
        expect(r.aroon_oscillator[i]).toBeCloseTo(r.aroon_up[i] - r.aroon_down[i], 9);
    }
});

test('local: flat market → ties resolve to oldest → Up=Down=0 at tail', () => {
    const bars = Array.from({ length: 30 }, () => b(101, 99));
    const r = localCompute(bars, 25);
    expect(r.aroon_up[29]).toBe(0);
    expect(r.aroon_down[29]).toBe(0);
});

test('local: output length = input length; warmup at index period-1 is null', () => {
    const bars = Array.from({ length: 50 }, (_, i) => b(101 + i * 0.1, 99 + i * 0.1));
    const r = localCompute(bars, 25);
    expect(r.aroon_up.length).toBe(50);
    expect(r.aroon_up[24]).toBeNull();
    expect(r.aroon_up[25]).not.toBeNull();
});

test('local: NaN in window → null output for that bar', () => {
    const bars = Array.from({ length: 30 }, (_, i) => b(100 + i, 99 + i));
    bars[10] = b(NaN, 0);
    const r = localCompute(bars, 25);
    // Bar 25 window covers [0..25] which includes bar 10 NaN → null.
    expect(r.aroon_up[25]).toBeNull();
});

test('local: Up/Down values bounded in [0, 100]', () => {
    const bars = Array.from({ length: 50 }, (_, i) => b(100 + Math.sin(i * 0.3), 99 + Math.sin(i * 0.3)));
    const r = localCompute(bars, 25);
    for (let i = 25; i < r.aroon_up.length; i++) {
        expect(r.aroon_up[i]).toBeGreaterThanOrEqual(0);
        expect(r.aroon_up[i]).toBeLessThanOrEqual(100);
        expect(r.aroon_down[i]).toBeGreaterThanOrEqual(0);
        expect(r.aroon_down[i]).toBeLessThanOrEqual(100);
    }
});

test('local: Oscillator bounded in [−100, +100]', () => {
    const bars = Array.from({ length: 50 }, (_, i) => b(100 + i * 0.1, 99 + i * 0.1));
    const r = localCompute(bars, 25);
    for (let i = 25; i < r.aroon_oscillator.length; i++) {
        expect(r.aroon_oscillator[i]).toBeGreaterThanOrEqual(-100);
        expect(r.aroon_oscillator[i]).toBeLessThanOrEqual(100);
    }
});

// ── regimeBadge / lastCrossover / summarize ──────────────────────

test('regimeBadge: 5-tier on oscillator', () => {
    expect(regimeBadge(90).key).toMatch(/strong_up/);
    expect(regimeBadge(50).key).toMatch(/up/);
    expect(regimeBadge(0).key).toMatch(/consolidate/);
    expect(regimeBadge(-50).key).toMatch(/down/);
    expect(regimeBadge(-90).key).toMatch(/strong_down/);
    expect(regimeBadge(NaN).key).toMatch(/unknown/);
});

test('lastCrossover: detects Up/Down crossover index + kind', () => {
    const report = {
        aroon_up:   [null, null, 50, 60, 70, 80],
        aroon_down: [null, null, 60, 60, 60, 60],
    };
    // bar 3: prev = 50-60 = -10, cur = 60-60 = 0  → 0 > 0 is false; need cur > 0.
    // bar 4: prev = 60-60 = 0, cur = 70-60 = 10  → 0 ≤ 0 ✓ and 10 > 0 ✓ → bull cross @ idx 4.
    const x = lastCrossover(report);
    expect(x).not.toBeNull();
    expect(x.kind).toBe('bull');
});

test('lastCrossover: returns null when no crossover', () => {
    const report = { aroon_up: [50, 60, 70], aroon_down: [10, 10, 10] };
    expect(lastCrossover(report)).toBeNull();
});

test('summarize: count / populated / last_up / last_down / last_osc', () => {
    const r = { aroon_up:   [null, null, 50, 60, 100],
                aroon_down: [null, null, 60, 30, 20],
                aroon_oscillator: [null, null, -10, 30, 80] };
    const s = summarize(r);
    expect(s.count).toBe(5);
    expect(s.populated).toBe(3);
    expect(s.last_up).toBe(100);
    expect(s.last_down).toBe(20);
    expect(s.last_osc).toBe(80);
});

test('summarize: empty → count 0, NaN extrema', () => {
    expect(summarize(null).count).toBe(0);
    expect(Number.isNaN(summarize(null).last_up)).toBe(true);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + produces an aroon report of correct length', () => {
    for (const k of ['strong-uptrend','strong-downtrend','flat','consolidation',
                     'bull-cross','bear-cross','noisy','short-period']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.bars, inp.period);
        expect(r.aroon_up.length).toBe(inp.bars.length);
    }
});

test('demo strong-uptrend: AroonUp=100, AroonDown=0 at tail', () => {
    const inp = makeDemoInput('strong-uptrend');
    const r = localCompute(inp.bars, inp.period);
    expect(r.aroon_up[r.aroon_up.length - 1]).toBe(100);
    expect(r.aroon_down[r.aroon_down.length - 1]).toBe(0);
});

test('demo strong-downtrend: AroonDown=100, AroonUp=0 at tail', () => {
    const inp = makeDemoInput('strong-downtrend');
    const r = localCompute(inp.bars, inp.period);
    expect(r.aroon_down[r.aroon_down.length - 1]).toBe(100);
    expect(r.aroon_up[r.aroon_up.length - 1]).toBe(0);
});

test('demo flat: AroonUp = AroonDown = 0 at tail', () => {
    const inp = makeDemoInput('flat');
    const r = localCompute(inp.bars, inp.period);
    expect(r.aroon_up[r.aroon_up.length - 1]).toBe(0);
    expect(r.aroon_down[r.aroon_down.length - 1]).toBe(0);
});

test('demo bull-cross: contains a bullish crossover', () => {
    const inp = makeDemoInput('bull-cross');
    const r = localCompute(inp.bars, inp.period);
    const x = lastCrossover(r);
    expect(x).not.toBeNull();
    expect(x.kind).toBe('bull');
});

test('demo bear-cross: contains a bearish crossover', () => {
    const inp = makeDemoInput('bear-cross');
    const r = localCompute(inp.bars, inp.period);
    const x = lastCrossover(r);
    expect(x).not.toBeNull();
    expect(x.kind).toBe('bear');
});

test('demo short-period: works with period=10', () => {
    const inp = makeDemoInput('short-period');
    expect(inp.period).toBe(10);
    const r = localCompute(inp.bars, inp.period);
    expect(r.aroon_up[r.aroon_up.length - 1]).not.toBeNull();
});

// ── round-trip + formatters ──────────────────────────────────────

test('barsToBlob round-trips through parseBarsBlob', () => {
    const bars = [b(101, 99), b(102, 100)];
    const back = parseBarsBlob(barsToBlob(bars));
    expect(back.errors).toEqual([]);
    expect(back.bars).toEqual(bars);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtNum(1.234, 2)).toBe('1.23');
    expect(fmtPct(80)).toBe('80.0');
    expect(fmtOsc(50)).toBe('+50.0');
    expect(fmtOsc(-50)).toBe('-50.0');
    expect(fmtInt(7.9)).toBe('7');
    expect(fmtUSD(100.5)).toBe('$100.50');
    expect(fmtPct(null)).toBe('—');
    expect(fmtOsc(NaN)).toBe('—');
});
