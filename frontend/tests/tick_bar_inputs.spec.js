// Tick-bar helpers: parser, validator, body shape,
// localCompute Rust-mirror, badges, summarize, demos.

import { test, expect } from 'vitest';
import {
    parsePrintsBlob, printsToBlob, validateInputs, buildBody, localCompute,
    trendBadge, coverageBadge, summarize,
    makeDemoInput,
    fmtUSD, fmtMove, fmtNum, fmtInt, fmtVol,
} from '../js/_tick_bar_inputs.js';

const p = (price, size) => ({ price, size });

// ── parser ────────────────────────────────────────────────────────

test('parsePrintsBlob: 2 tokens per line, blanks + comments ignored', () => {
    const r = parsePrintsBlob('100.05 10\n# midday\n100.06, 25');
    expect(r.errors).toEqual([]);
    expect(r.prints).toEqual([p(100.05, 10), p(100.06, 25)]);
});

test('parsePrintsBlob: rejects wrong count + non-positive price + negative size', () => {
    expect(parsePrintsBlob('100').errors[0].message).toMatch(/2 tokens/);
    expect(parsePrintsBlob('-1 10').errors[0].message).toMatch(/price/);
    expect(parsePrintsBlob('100 -1').errors[0].message).toMatch(/size/);
});

test('parsePrintsBlob: non-string returns 1 error', () => {
    expect(parsePrintsBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts default', () => {
    expect(validateInputs({ prints: [p(100, 10)], ticks_per_bar: 10 })).toBe(null);
});

test('validate rejects: bad array / non-integer N / N < 1', () => {
    expect(validateInputs({ prints: 'no', ticks_per_bar: 10 })).toMatch(/prints/);
    expect(validateInputs({ prints: [], ticks_per_bar: 1.5 })).toMatch(/integer/);
    expect(validateInputs({ prints: [], ticks_per_bar: 0 })).toMatch(/≥ 1/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: strips extras', () => {
    const body = buildBody({ prints: [{ ...p(100, 10), extra: 'x' }], ticks_per_bar: 5 });
    expect(body).toEqual({ prints: [p(100, 10)], ticks_per_bar: 5 });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: empty / N = 0 → empty', () => {
    expect(localCompute([], 10)).toEqual([]);
    expect(localCompute(Array(50).fill(p(100, 10)), 0)).toEqual([]);
});

test('local: NaN price → empty', () => {
    expect(localCompute([p(NaN, 10)], 10)).toEqual([]);
});

test('local: negative size → empty', () => {
    expect(localCompute([p(100, -1)], 10)).toEqual([]);
});

test('local: emits one bar per N ticks (3 bars from 30 prints, N=10)', () => {
    const prints = Array.from({ length: 30 }, (_, i) => p(100 + i * 0.1, 1));
    const bars = localCompute(prints, 10);
    expect(bars.length).toBe(3);
    for (const b of bars) {
        expect(b.tick_count).toBe(10);
        expect(b.volume).toBeCloseTo(10, 9);
    }
});

test('local: open=first print, close=last print of bar', () => {
    const bars = localCompute([p(100, 1), p(102, 1), p(101, 1)], 3);
    expect(bars.length).toBe(1);
    expect(bars[0].open).toBeCloseTo(100, 9);
    expect(bars[0].close).toBeCloseTo(101, 9);
});

test('local: high/low tracked across the bar', () => {
    const bars = localCompute([p(100, 1), p(110, 1), p(95, 1), p(102, 1)], 4);
    expect(bars[0].high).toBeCloseTo(110, 9);
    expect(bars[0].low).toBeCloseTo(95, 9);
});

test('local: trailing partial bar dropped (23 prints, N=10 → 2 bars)', () => {
    const prints = Array.from({ length: 23 }, (_, i) => p(100 + i, 1));
    expect(localCompute(prints, 10).length).toBe(2);
});

test('local: N = 1 emits one bar per print', () => {
    const prints = [p(100, 1), p(101, 1), p(102, 1)];
    const bars = localCompute(prints, 1);
    expect(bars.length).toBe(3);
    expect(bars[0].open).toBe(100);
    expect(bars[0].close).toBe(100);
    expect(bars[0].tick_count).toBe(1);
});

test('local: volume sums per bar; open == close == high == low when size 1 and prices identical', () => {
    const bars = localCompute([p(100, 5), p(100, 5), p(100, 5)], 3);
    expect(bars.length).toBe(1);
    expect(bars[0].volume).toBeCloseTo(15, 9);
    expect(bars[0].open).toBe(100);
    expect(bars[0].close).toBe(100);
    expect(bars[0].high).toBe(100);
    expect(bars[0].low).toBe(100);
});

test('local: bar boundary resets open/high/low to next print', () => {
    // 10 prints rising 100 → 109, N=5 → 2 bars; bar2 opens at print 5 = 105.
    const prints = Array.from({ length: 10 }, (_, i) => p(100 + i, 1));
    const bars = localCompute(prints, 5);
    expect(bars.length).toBe(2);
    expect(bars[1].open).toBeCloseTo(105, 9);
});

// ── trendBadge / coverageBadge ────────────────────────────────────

test('trendBadge: last bar direction', () => {
    expect(trendBadge([]).key).toMatch(/flat/);
    expect(trendBadge([{ open: 100, close: 105 }]).key).toMatch(/uptrend/);
    expect(trendBadge([{ open: 100, close: 95 }]).key).toMatch(/downtrend/);
    expect(trendBadge([{ open: 100, close: 100 }]).key).toMatch(/flat/);
});

test('coverageBadge: low / normal / high / full', () => {
    // 5 bars × 10 ticks = 50 covered.
    expect(coverageBadge(Array(2).fill({}), 100, 10).key).toMatch(/low/);     // 20/100 = 0.2
    expect(coverageBadge(Array(7).fill({}), 100, 10).key).toMatch(/normal/);  // 70/100 = 0.7
    expect(coverageBadge(Array(9).fill({}), 100, 10).key).toMatch(/high/);    // 90/100 = 0.9
    expect(coverageBadge(Array(10).fill({}), 100, 10).key).toMatch(/full/);   // 100/100 = 1.0
    expect(coverageBadge([], 0, 10).key).toMatch(/unknown/);
});

// ── summarize ─────────────────────────────────────────────────────

test('summarize: counts/volume/range/last_close', () => {
    const bars = [
        { open: 100, high: 105, low: 99, close: 105, volume: 100, tick_count: 5 },
        { open: 105, high: 106, low: 100, close: 102, volume: 80, tick_count: 4 },
        { open: 102, high: 103, low: 101, close: 102, volume: 60, tick_count: 3 },
    ];
    const s = summarize(bars);
    expect(s.count).toBe(3);
    expect(s.total_volume).toBe(240);
    expect(s.total_ticks).toBe(12);
    expect(s.avg_volume).toBeCloseTo(80, 9);
    expect(s.avg_range).toBeCloseTo((6 + 6 + 2) / 3, 9);
    expect(s.ups).toBe(1);
    expect(s.downs).toBe(1);
    expect(s.doji).toBe(1);
    expect(s.last_close).toBe(102);
});

test('summarize: empty → count 0, NaN aggregates', () => {
    const s = summarize([]);
    expect(s.count).toBe(0);
    expect(Number.isNaN(s.avg_volume)).toBe(true);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + computes without error', () => {
    for (const k of ['uptrend','downtrend','flat','noisy','small-bars',
                     'large-bars','partial','one-tick']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const bars = localCompute(inp.prints, inp.ticks_per_bar);
        expect(Array.isArray(bars)).toBe(true);
    }
});

test('demo uptrend: every bar has close > open', () => {
    const inp = makeDemoInput('uptrend');
    const bars = localCompute(inp.prints, inp.ticks_per_bar);
    expect(bars.length).toBeGreaterThan(0);
    for (const b of bars) expect(b.close).toBeGreaterThan(b.open);
});

test('demo downtrend: every bar has close < open', () => {
    const inp = makeDemoInput('downtrend');
    const bars = localCompute(inp.prints, inp.ticks_per_bar);
    expect(bars.length).toBeGreaterThan(0);
    for (const b of bars) expect(b.close).toBeLessThan(b.open);
});

test('demo flat: every bar is a doji', () => {
    const inp = makeDemoInput('flat');
    const bars = localCompute(inp.prints, inp.ticks_per_bar);
    expect(bars.length).toBeGreaterThan(0);
    for (const b of bars) expect(b.close).toBe(b.open);
});

test('demo partial: 23 prints + N=10 → exactly 2 full bars (3 prints dropped)', () => {
    const inp = makeDemoInput('partial');
    const bars = localCompute(inp.prints, inp.ticks_per_bar);
    expect(bars.length).toBe(2);
});

test('demo one-tick: every print becomes a bar', () => {
    const inp = makeDemoInput('one-tick');
    const bars = localCompute(inp.prints, inp.ticks_per_bar);
    expect(bars.length).toBe(inp.prints.length);
    for (const b of bars) expect(b.tick_count).toBe(1);
});

test('demo large-bars: fewer bars than small-bars for same generator', () => {
    const small = makeDemoInput('small-bars');
    const large = makeDemoInput('large-bars');
    const sB = localCompute(small.prints, small.ticks_per_bar);
    const lB = localCompute(large.prints, large.ticks_per_bar);
    expect(sB.length).toBeGreaterThan(lB.length);
});

// ── round-trip + formatters ──────────────────────────────────────

test('printsToBlob round-trips through parsePrintsBlob', () => {
    const prints = [p(100.05, 10), p(100.06, 25)];
    const back = parsePrintsBlob(printsToBlob(prints));
    expect(back.errors).toEqual([]);
    expect(back.prints).toEqual(prints);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtUSD(100.5)).toBe('$100.50');
    expect(fmtMove(2.5)).toBe('+$2.50');
    expect(fmtMove(-2.5)).toBe('-$2.50');
    expect(fmtNum(1.234, 1)).toBe('1.2');
    expect(fmtInt(42.9)).toBe('42');
    expect(fmtVol(1_500_000)).toBe('1.50M');
    expect(fmtVol(15_500)).toBe('15.50k');
    expect(fmtVol(42)).toBe('42');
    expect(fmtUSD(NaN)).toBe('—');
});
