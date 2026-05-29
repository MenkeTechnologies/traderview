// Dollar-bar helpers: parser, validator, body shape,
// localCompute Rust-mirror, badges, summarize, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS,
    parsePrintsBlob, printsToBlob, validateInputs, buildBody, localCompute,
    trendBadge, coverageBadge, summarize,
    makeDemoInput,
    fmtUSD, fmtMove, fmtNum, fmtInt, fmtVol, fmtNotional,
} from '../js/_dollar_bar_inputs.js';

const p = (price, size) => ({ price, size });

// ── parser ────────────────────────────────────────────────────────

test('parsePrintsBlob: 2 tokens per line, blanks + comments ignored', () => {
    const r = parsePrintsBlob('100.05 200\n# midday\n100.06, 250');
    expect(r.errors).toEqual([]);
    expect(r.prints).toEqual([p(100.05, 200), p(100.06, 250)]);
});

test('parsePrintsBlob: rejects wrong count / bad price / bad size', () => {
    expect(parsePrintsBlob('100').errors[0].message).toMatch(/2 tokens/);
    expect(parsePrintsBlob('-1 10').errors[0].message).toMatch(/price/);
    expect(parsePrintsBlob('100 -1').errors[0].message).toMatch(/size/);
});

test('parsePrintsBlob: non-string returns 1 error', () => {
    expect(parsePrintsBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts default-shape', () => {
    expect(validateInputs({ prints: [p(100, 10)], dollars_per_bar: 100_000 })).toBe(null);
});

test('validate rejects: bad array / dollars_per_bar ≤ 0 / NaN', () => {
    expect(validateInputs({ prints: 'no', dollars_per_bar: 100 })).toMatch(/prints/);
    expect(validateInputs({ prints: [p(100, 10)], dollars_per_bar: 0 })).toMatch(/dollars_per_bar/);
    expect(validateInputs({ prints: [p(100, 10)], dollars_per_bar: NaN })).toMatch(/dollars_per_bar/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: strips extras', () => {
    const body = buildBody({ prints: [{ ...p(100, 10), extra: 'x' }], dollars_per_bar: 500 });
    expect(body).toEqual({ prints: [p(100, 10)], dollars_per_bar: 500 });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: empty / dollars_per_bar=0 → empty', () => {
    expect(localCompute([], 100_000)).toEqual([]);
    expect(localCompute(Array(50).fill(p(100, 10)), 0)).toEqual([]);
});

test('local: NaN price → empty', () => {
    expect(localCompute([p(NaN, 10)], 10_000)).toEqual([]);
});

test('local: negative size → empty', () => {
    expect(localCompute([p(100, -1)], 10_000)).toEqual([]);
});

test('local: notional reaches target → at least one bar (Rust spec test)', () => {
    // 10 prints, price ~100, size 100 each → notional 10_000 each. Target 50_000.
    const prints = Array.from({ length: 10 }, (_, i) => p(100 + i * 0.1, 100));
    const bars = localCompute(prints, 50_000);
    expect(bars.length).toBeGreaterThan(0);
    expect(bars[0].notional).toBeGreaterThanOrEqual(50_000);
});

test('local: high/low tracked over the dollar window', () => {
    const prints = [p(100, 100), p(110, 100), p(95, 100), p(102, 100), p(98, 100), p(101, 100)];
    const bars = localCompute(prints, 60_000);
    expect(bars.length).toBe(1);
    expect(bars[0].high).toBeCloseTo(110, 9);
    expect(bars[0].low).toBeCloseTo(95, 9);
});

test('local: trailing partial bar dropped (only 3 prints × 10k notional)', () => {
    const prints = Array.from({ length: 3 }, () => p(100, 100));
    expect(localCompute(prints, 50_000)).toEqual([]);
});

test('local: notional = Σ price × size per bar', () => {
    const prints = [p(100, 200), p(110, 300)];
    const bars = localCompute(prints, 50_000);
    expect(bars.length).toBe(1);
    // 100·200 + 110·300 = 53_000.
    expect(bars[0].notional).toBeCloseTo(53_000, 9);
});

test('local: open=first print of bar, close=trigger print', () => {
    const prints = [p(100, 200), p(105, 200), p(110, 300)];
    const bars = localCompute(prints, 50_000);
    expect(bars.length).toBe(1);
    expect(bars[0].open).toBe(100);
    expect(bars[0].close).toBe(110);
});

test('local: next bar opens at print AFTER the trigger', () => {
    // Engineer to fit 2 bars cleanly: each bar accumulates 30k notional then trips.
    const prints = [p(100, 100), p(100, 200), p(100, 50), p(100, 100), p(100, 200), p(100, 50)];
    const bars = localCompute(prints, 30_000);
    expect(bars.length).toBe(2);
});

test('local: one giant print fills the bar immediately', () => {
    const prints = [p(100, 5000)];
    const bars = localCompute(prints, 100_000);
    expect(bars.length).toBe(1);
    expect(bars[0].notional).toBe(500_000);
    expect(bars[0].tick_count).toBe(1);
});

// ── trendBadge / coverageBadge ────────────────────────────────────

test('trendBadge: last bar direction', () => {
    expect(trendBadge([]).key).toMatch(/flat/);
    expect(trendBadge([{ open: 100, close: 105 }]).key).toMatch(/uptrend/);
    expect(trendBadge([{ open: 100, close: 95 }]).key).toMatch(/downtrend/);
    expect(trendBadge([{ open: 100, close: 100 }]).key).toMatch(/flat/);
});

test('coverageBadge: low / normal / high / full', () => {
    const mk = (nots) => nots.map(n => ({ notional: n }));
    expect(coverageBadge(mk([30_000]),  100_000, 50_000).key).toMatch(/low/);
    expect(coverageBadge(mk([70_000]),  100_000, 50_000).key).toMatch(/normal/);
    expect(coverageBadge(mk([90_000]),  100_000, 50_000).key).toMatch(/high/);
    expect(coverageBadge(mk([100_000]), 100_000, 50_000).key).toMatch(/full/);
    expect(coverageBadge([], 0, 50_000).key).toMatch(/unknown/);
});

// ── summarize ─────────────────────────────────────────────────────

test('summarize: counts/notional/range/last_close', () => {
    const bars = [
        { open: 100, high: 105, low: 99, close: 105, volume: 1000, notional: 100_000, tick_count: 5 },
        { open: 105, high: 106, low: 100, close: 102, volume: 800, notional: 80_000,  tick_count: 6 },
        { open: 102, high: 103, low: 101, close: 102, volume: 600, notional: 60_000,  tick_count: 4 },
    ];
    const s = summarize(bars);
    expect(s.count).toBe(3);
    expect(s.total_notional).toBe(240_000);
    expect(s.total_volume).toBe(2400);
    expect(s.total_ticks).toBe(15);
    expect(s.avg_ticks).toBeCloseTo(5, 9);
    expect(s.avg_range).toBeCloseTo((6 + 6 + 2) / 3, 9);
    expect(s.avg_notional).toBeCloseTo(80_000, 9);
    expect(s.ups).toBe(1);
    expect(s.downs).toBe(1);
    expect(s.doji).toBe(1);
    expect(s.last_close).toBe(102);
});

test('summarize: empty → count 0, NaN aggregates', () => {
    const s = summarize([]);
    expect(s.count).toBe(0);
    expect(Number.isNaN(s.avg_notional)).toBe(true);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + computes without error', () => {
    for (const k of ['mid-cap-uptrend','mid-cap-downtrend','flat-market','penny-stock',
                     'large-cap','partial-trail','spiky-notional','noisy-walk']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const bars = localCompute(inp.prints, inp.dollars_per_bar);
        expect(Array.isArray(bars)).toBe(true);
    }
});

test('demo partial-trail: 0 bars emitted (only 30k notional, 50k target)', () => {
    const inp = makeDemoInput('partial-trail');
    expect(localCompute(inp.prints, inp.dollars_per_bar).length).toBe(0);
});

test('demo flat-market: every bar has identical open/high/low/close', () => {
    const inp = makeDemoInput('flat-market');
    const bars = localCompute(inp.prints, inp.dollars_per_bar);
    expect(bars.length).toBeGreaterThan(0);
    for (const b of bars) {
        expect(b.open).toBe(b.close);
        expect(b.high).toBe(b.low);
    }
});

test('demo penny-stock: bars emitted despite low price (compensated by size)', () => {
    const inp = makeDemoInput('penny-stock');
    const bars = localCompute(inp.prints, inp.dollars_per_bar);
    expect(bars.length).toBeGreaterThan(0);
});

test('demo large-cap: bars emitted with high notional per bar', () => {
    const inp = makeDemoInput('large-cap');
    const bars = localCompute(inp.prints, inp.dollars_per_bar);
    expect(bars.length).toBeGreaterThan(0);
});

// ── round-trip + formatters ──────────────────────────────────────

test('printsToBlob round-trips through parsePrintsBlob', () => {
    const prints = [p(100.05, 200), p(100.06, 250)];
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
    expect(fmtNotional(1_500_000_000)).toBe('$1.50B');
    expect(fmtNotional(1_500_000)).toBe('$1.50M');
    expect(fmtNotional(15_500)).toBe('$15.50k');
    expect(fmtNotional(NaN)).toBe('—');
});
