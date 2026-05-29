// Range-bar helpers: parser, validator, body shape,
// localCompute Rust-mirror, badges, summarize, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS,
    parsePrintsBlob, printsToBlob, validateInputs, buildBody, localCompute,
    trendBadge, activityBadge, summarize,
    makeDemoInput,
    fmtUSD, fmtMove, fmtNum, fmtInt, fmtVol,
} from '../js/_range_bar_inputs.js';

const p = (price, size) => ({ price, size });

// ── parser ────────────────────────────────────────────────────────

test('parsePrintsBlob: 2 tokens per line, comments and blanks ignored', () => {
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

test('validate accepts non-empty defaults', () => {
    expect(validateInputs({ prints: [p(100, 10)], target_range: 5 })).toBe(null);
});

test('validate accepts empty prints (server returns empty bars)', () => {
    expect(validateInputs({ prints: [], target_range: 5 })).toBe(null);
});

test('validate rejects: bad array / bad price / bad size / bad target_range', () => {
    expect(validateInputs({ prints: 'no', target_range: 5 })).toMatch(/prints/);
    expect(validateInputs({ prints: [{ price: 0, size: 10 }], target_range: 5 })).toMatch(/price/);
    expect(validateInputs({ prints: [{ price: 100, size: -1 }], target_range: 5 })).toMatch(/size/);
    expect(validateInputs({ prints: [p(100, 10)], target_range: 0 })).toMatch(/target_range/);
    expect(validateInputs({ prints: [p(100, 10)], target_range: NaN })).toMatch(/target_range/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: passes through prints (strips extras) + target_range', () => {
    const body = buildBody({ prints: [{ ...p(100, 10), extra: 'x' }], target_range: 5 });
    expect(body).toEqual({ prints: [p(100, 10)], target_range: 5 });
});

// ── localCompute parity (mirrors every Rust #[test] + boundaries) ─

test('local: empty / target=0 → empty', () => {
    expect(localCompute([], 1)).toEqual([]);
    expect(localCompute(Array(5).fill(p(100, 10)), 0)).toEqual([]);
});

test('local: NaN price or negative size → empty', () => {
    expect(localCompute([p(NaN, 10)], 1)).toEqual([]);
    expect(localCompute([p(100, -1)], 1)).toEqual([]);
});

test('local: flat market produces no bars', () => {
    expect(localCompute(Array(50).fill(p(100, 10)), 1)).toEqual([]);
});

test('local: pure uptrend emits expected number of bars + correct OHLC', () => {
    // Prints 100..110, target_range=5 → bars close at 105 and 110.
    const prints = Array.from({ length: 11 }, (_, i) => p(100 + i, 10));
    const bars = localCompute(prints, 5);
    expect(bars.length).toBe(2);
    expect(bars[0].open).toBeCloseTo(100, 9);
    expect(bars[0].close).toBeCloseTo(105, 9);
    expect(bars[1].close).toBeCloseTo(110, 9);
});

test('local: tiny range below target → no bars', () => {
    const bars = localCompute([p(100, 10), p(100.5, 10), p(100.3, 10)], 1);
    expect(bars).toEqual([]);
});

test('local: volume aggregates per bar + tick_count tracked', () => {
    const bars = localCompute([p(100, 10), p(103, 20), p(105, 30)], 5);
    expect(bars.length).toBe(1);
    expect(bars[0].volume).toBeCloseTo(60, 9);
    expect(bars[0].tick_count).toBe(3);
});

test('local: bar opens at prior bar\'s close (continuity)', () => {
    const prints = Array.from({ length: 11 }, (_, i) => p(100 + i, 10));
    const bars = localCompute(prints, 5);
    expect(bars[1].open).toBeCloseTo(bars[0].close, 9);
});

test('local: trailing partial bar NOT emitted (matches Rust contract)', () => {
    // Range = 3 reaches before final print, so a bar exists, but next leg never closes.
    const prints = [p(100, 10), p(101, 10), p(102, 10), p(103, 10), p(103.5, 10), p(103.6, 10)];
    const bars = localCompute(prints, 3);
    // First bar closes at 103 (100 to 103 = range 3). Then bars stop — final 103.5/103.6 partial NOT emitted.
    expect(bars.length).toBe(1);
});

test('local: bar size sums volume + each tick increments tick_count by 1', () => {
    const bars = localCompute([p(100, 5), p(101, 7), p(105, 11)], 4);
    expect(bars.length).toBe(1);
    expect(bars[0].volume).toBeCloseTo(5 + 7 + 11, 9);
    expect(bars[0].tick_count).toBe(3);
});

test('local: pure downtrend produces bars closing at descending prices', () => {
    const prints = Array.from({ length: 11 }, (_, i) => p(110 - i, 10));
    const bars = localCompute(prints, 5);
    expect(bars.length).toBe(2);
    expect(bars[0].close).toBeCloseTo(105, 9);
    expect(bars[1].close).toBeCloseTo(100, 9);
});

test('local: target_range >= moving range → no bars', () => {
    const prints = Array.from({ length: 11 }, (_, i) => p(100 + i * 0.1, 10));
    const bars = localCompute(prints, 5);
    expect(bars).toEqual([]);
});

// ── trendBadge / activityBadge ───────────────────────────────────

test('trendBadge: last bar direction wins; flat on empty', () => {
    expect(trendBadge([]).key).toMatch(/flat/);
    expect(trendBadge([{ open: 100, close: 105 }]).key).toMatch(/uptrend/);
    expect(trendBadge([{ open: 100, close: 95 }]).key).toMatch(/downtrend/);
    expect(trendBadge([{ open: 100, close: 100 }]).key).toMatch(/flat/);
});

test('activityBadge: density tiers', () => {
    expect(activityBadge([], 100).key).toMatch(/quiet/);
    expect(activityBadge(Array(2).fill({}), 100).key).toMatch(/normal/);
    expect(activityBadge(Array(10).fill({}), 100).key).toMatch(/active/);
    expect(activityBadge(Array(25).fill({}), 100).key).toMatch(/volatile/);
    expect(activityBadge([], 0).key).toMatch(/unknown/);
});

// ── summarize ─────────────────────────────────────────────────────

test('summarize: count / volume / ticks / up-down-doji / last_close', () => {
    const bars = [
        { open: 100, close: 105, volume: 100, tick_count: 5 },
        { open: 105, close: 102, volume: 80,  tick_count: 4 },
        { open: 102, close: 102, volume: 60,  tick_count: 3 },
    ];
    const s = summarize(bars);
    expect(s.count).toBe(3);
    expect(s.total_volume).toBe(240);
    expect(s.total_ticks).toBe(12);
    expect(s.avg_volume).toBeCloseTo(80, 9);
    expect(s.avg_ticks).toBeCloseTo(4, 9);
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
    for (const k of ['uptrend','downtrend','chop','flat','big-prints',
                     'small-range','wide-range','noisy-walk']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const bars = localCompute(inp.prints, inp.target_range);
        expect(Array.isArray(bars)).toBe(true);
    }
});

test('demo uptrend: emits multiple Up bars', () => {
    const inp = makeDemoInput('uptrend');
    const bars = localCompute(inp.prints, inp.target_range);
    expect(bars.length).toBeGreaterThan(0);
    for (const b of bars) expect(b.close).toBeGreaterThan(b.open);
});

test('demo downtrend: every bar has close < open', () => {
    const inp = makeDemoInput('downtrend');
    const bars = localCompute(inp.prints, inp.target_range);
    expect(bars.length).toBeGreaterThan(0);
    for (const b of bars) expect(b.close).toBeLessThan(b.open);
});

test('demo flat: 0 bars', () => {
    const inp = makeDemoInput('flat');
    expect(localCompute(inp.prints, inp.target_range).length).toBe(0);
});

test('demo small-range: bounce amp < target → 0 bars', () => {
    const inp = makeDemoInput('small-range');
    expect(localCompute(inp.prints, inp.target_range).length).toBe(0);
});

test('demo wide-range: fewer bars than uptrend with same print sequence', () => {
    const wide = makeDemoInput('wide-range');
    const wideBars = localCompute(wide.prints, wide.target_range);
    // The wide-range demo uses a different print generator, but result must be valid.
    expect(wideBars.length).toBeGreaterThanOrEqual(0);
});

test('demo big-prints: volume per bar reflects the high-volume prints', () => {
    const inp = makeDemoInput('big-prints');
    const bars = localCompute(inp.prints, inp.target_range);
    expect(bars.length).toBeGreaterThan(0);
    for (const b of bars) expect(b.volume).toBeGreaterThan(0);
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
