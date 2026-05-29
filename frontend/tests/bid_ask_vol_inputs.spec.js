// Bid/Ask Volume Ratio helpers: parser, validator, localCompute parity, badges.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, DEFAULT_PERIOD, MIN_PERIOD, MAX_PERIOD,
    parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute,
    flowBadge, trendBadge, imbalanceBadge, summarizeBars,
    makeDemoInput,
    fmtNum, fmtRatio, fmtPct, fmtInt,
} from '../js/_bid_ask_vol_inputs.js';

const b = (bv, av) => ({ bid_volume: bv, ask_volume: av });

// ── parser ────────────────────────────────────────────────────────

test('parseBarsBlob: 2 tokens per line', () => {
    const r = parseBarsBlob('1000 1100\n# midday\n1200, 950');
    expect(r.errors).toEqual([]);
    expect(r.bars).toEqual([b(1000, 1100), b(1200, 950)]);
});

test('parseBarsBlob: rejects wrong count / negative / non-finite', () => {
    expect(parseBarsBlob('1000').errors[0].message).toMatch(/2 tokens/);
    expect(parseBarsBlob('-1 100').errors[0].message).toMatch(/≥ 0/);
    expect(parseBarsBlob('NaN 100').errors[0].message).toMatch(/≥ 0/);
});

test('parseBarsBlob: non-string returns 1 error', () => {
    expect(parseBarsBlob(undefined).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts valid', () => {
    const bars = Array.from({ length: 100 }, () => b(100, 100));
    expect(validateInputs({ bars, period: 60 })).toBe(null);
});

test('validate rejects: bad array / bad period / too short / non-finite / negative', () => {
    const ok = Array.from({ length: 100 }, () => b(100, 100));
    expect(validateInputs({ bars: 'no', period: 60 })).toMatch(/bars/);
    expect(validateInputs({ bars: ok, period: 1 })).toMatch(/period/);
    expect(validateInputs({ bars: ok, period: 9999 })).toMatch(/period/);
    expect(validateInputs({ bars: ok.slice(0, 5), period: 60 })).toMatch(/period/);
    const badNan = [...ok];
    badNan[5] = { bid_volume: NaN, ask_volume: 100 };
    expect(validateInputs({ bars: badNan, period: 60 })).toMatch(/finite/);
    const neg = [...ok];
    neg[5] = b(-1, 100);
    expect(validateInputs({ bars: neg, period: 60 })).toMatch(/≥ 0/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody trims bars to bid/ask only', () => {
    const body = buildBody({ bars: [{ ...b(100, 100), extra: 'x' }], period: 60 });
    expect(body).toEqual({ bars: [b(100, 100)], period: 60 });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: invalid inputs return all null', () => {
    const bars = Array.from({ length: 100 }, () => b(100, 100));
    expect(localCompute(bars, 1).every(x => x === null)).toBe(true);
    expect(localCompute(bars.slice(0, 5), 60).every(x => x === null)).toBe(true);
});

test('local: NaN or negative returns all null', () => {
    const bars1 = Array.from({ length: 100 }, () => b(100, 100));
    bars1[5] = b(NaN, 100);
    expect(localCompute(bars1, 60).every(x => x === null)).toBe(true);
    const bars2 = Array.from({ length: 100 }, () => b(100, 100));
    bars2[5] = b(-1, 100);
    expect(localCompute(bars2, 60).every(x => x === null)).toBe(true);
});

test('local: balanced flow yields ratio ≈ 1', () => {
    const bars = Array.from({ length: 100 }, () => b(100, 100));
    const r = localCompute(bars, 60);
    for (const v of r) {
        if (v == null) continue;
        expect(Math.abs(v - 1)).toBeLessThan(1e-9);
    }
});

test('local: buy pressure (50/100) yields ratio = 0.5', () => {
    const bars = Array.from({ length: 100 }, () => b(50, 100));
    const r = localCompute(bars, 60);
    expect(Math.abs(r[99] - 0.5)).toBeLessThan(1e-9);
});

test('local: sell pressure (200/100) yields ratio = 2', () => {
    const bars = Array.from({ length: 100 }, () => b(200, 100));
    const r = localCompute(bars, 60);
    expect(Math.abs(r[99] - 2)).toBeLessThan(1e-9);
});

test('local: zero ask sum → all null', () => {
    const bars = Array.from({ length: 100 }, () => b(100, 0));
    const r = localCompute(bars, 60);
    expect(r.every(x => x === null)).toBe(true);
});

test('local: output length matches input', () => {
    const bars = Array.from({ length: 100 }, () => b(100, 100));
    expect(localCompute(bars, 60).length).toBe(100);
});

test('local: leading nulls until period', () => {
    const bars = Array.from({ length: 100 }, () => b(100, 100));
    const r = localCompute(bars, 60);
    for (let i = 0; i < 59; i++) expect(r[i]).toBe(null);
    expect(r[59]).not.toBe(null);
});

test('local: deterministic', () => {
    const bars = Array.from({ length: 100 }, (_, i) => b(100 + i, 100));
    expect(localCompute(bars, 60)).toEqual(localCompute(bars, 60));
});

test('local: rolling window correctly removes oldest bar', () => {
    // First 60 bars: bid=100, ask=100 → ratio 1.0 at index 59
    // Next 60 bars: bid=200, ask=100 → after window fully shifts, ratio = 2.0
    const bars = [
        ...Array.from({ length: 60 }, () => b(100, 100)),
        ...Array.from({ length: 60 }, () => b(200, 100)),
    ];
    const r = localCompute(bars, 60);
    expect(Math.abs(r[59] - 1.0)).toBeLessThan(1e-9);
    expect(Math.abs(r[119] - 2.0)).toBeLessThan(1e-9);
});

// ── badges ────────────────────────────────────────────────────────

test('flowBadge: 7 tiers', () => {
    expect(flowBadge(4).key).toMatch(/heavy_sell/);
    expect(flowBadge(2).key).toMatch(/sell_pressure/);
    expect(flowBadge(1.3).key).toMatch(/sell_tilt/);
    expect(flowBadge(1.0).key).toMatch(/balanced/);
    expect(flowBadge(0.8).key).toMatch(/buy_tilt/);
    expect(flowBadge(0.5).key).toMatch(/buy_pressure/);
    expect(flowBadge(0.1).key).toMatch(/heavy_buy/);
    expect(flowBadge(null).key).toMatch(/unknown/);
});

test('trendBadge: tiers', () => {
    expect(trendBadge([1, 1, 1, 1, 1, 1, 1, 1, 1, 1]).key).toMatch(/flat/);
    expect(trendBadge([0.5, 0.6, 0.7, 0.8, 0.9, 1, 1.1, 1.2, 1.3, 2.0]).key).toMatch(/rising_sell/);
    // Slight upward slope inside wider noise → tilting_sell (slope/range between 0.1 and 0.5).
    expect(trendBadge([0.95, 1.1, 1.05, 0.95, 1.05, 1, 1, 1, 1, 1.02]).key).toMatch(/tilting_sell/);
    expect(trendBadge([2.0, 1.9, 1.8, 1.7, 1.6, 1.5, 1.4, 1.3, 1.2, 0.5]).key).toMatch(/rising_buy/);
    // Slight downward slope inside wider noise → tilting_buy.
    expect(trendBadge([1.05, 0.9, 0.95, 1.05, 0.95, 1, 1, 1, 1, 0.98]).key).toMatch(/tilting_buy/);
    expect(trendBadge([]).key).toMatch(/unknown/);
});

test('imbalanceBadge: tiers via |log(ratio)|', () => {
    expect(imbalanceBadge(1.0).key).toMatch(/symmetric/);    // log=0
    expect(imbalanceBadge(1.3).key).toMatch(/mild/);          // |log|≈0.26
    expect(imbalanceBadge(2.0).key).toMatch(/strong/);        // |log|≈0.69
    expect(imbalanceBadge(5.0).key).toMatch(/extreme/);       // |log|≈1.6
    expect(imbalanceBadge(0.2).key).toMatch(/extreme/);       // |log(0.2)|≈1.609 → extreme tier (≥ 1.10)
    expect(imbalanceBadge(0).key).toMatch(/unknown/);
    expect(imbalanceBadge(null).key).toMatch(/unknown/);
});

// ── summarize ─────────────────────────────────────────────────────

test('summarizeBars: count / totals / means / lifetime ratio', () => {
    const bars = [b(100, 200), b(150, 250), b(200, 300)];
    const s = summarizeBars(bars);
    expect(s.count).toBe(3);
    expect(s.total_bid).toBe(450);
    expect(s.total_ask).toBe(750);
    expect(s.total_vol).toBe(1200);
    expect(s.mean_bid).toBe(150);
    expect(s.mean_ask).toBe(250);
    expect(s.lifetime_ratio).toBeCloseTo(0.6, 9);
});

test('summarizeBars: empty → NaN', () => {
    const s = summarizeBars([]);
    expect(s.count).toBe(0);
    expect(Number.isNaN(s.total_bid)).toBe(true);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + computes', () => {
    for (const k of ['balanced','buy-pressure','sell-pressure','shifting-buy',
                     'shifting-sell','heavy-buy','heavy-sell','short-period']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.bars, inp.period);
        expect(r.length).toBe(inp.bars.length);
    }
});

test('demo balanced: last ratio ≈ 1', () => {
    const inp = makeDemoInput('balanced');
    const r = localCompute(inp.bars, inp.period);
    expect(Math.abs(r[r.length - 1] - 1)).toBeLessThan(0.1);
});

test('demo buy-pressure: last ratio < 1', () => {
    const inp = makeDemoInput('buy-pressure');
    const r = localCompute(inp.bars, inp.period);
    expect(r[r.length - 1]).toBeLessThan(1);
});

test('demo sell-pressure: last ratio > 1', () => {
    const inp = makeDemoInput('sell-pressure');
    const r = localCompute(inp.bars, inp.period);
    expect(r[r.length - 1]).toBeGreaterThan(1);
});

test('demo shifting-buy: ratio decreases (rising_buy)', () => {
    const inp = makeDemoInput('shifting-buy');
    const r = localCompute(inp.bars, inp.period);
    const first = r.find(v => v != null);
    const last  = r[r.length - 1];
    expect(last).toBeLessThan(first);
});

// ── formatters ────────────────────────────────────────────────────

test('barsToBlob round-trips', () => {
    const bars = [b(1000, 1100), b(1200, 950)];
    const back = parseBarsBlob(barsToBlob(bars));
    expect(back.errors).toEqual([]);
    expect(back.bars).toEqual(bars);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtNum(1500)).toBe('1.50k');
    expect(fmtNum(1_500_000)).toBe('1.50M');
    expect(fmtNum(42.5)).toBe('42.50');
    expect(fmtRatio(1.2345)).toBe('1.2345');
    expect(fmtPct(0.0125)).toBe('1.25%');
    expect(fmtInt(42.9)).toBe('42');
    expect(fmtNum(NaN)).toBe('—');
    expect(fmtRatio(NaN)).toBe('—');
});

test('DEFAULTS sanity', () => {
    expect(DEFAULT_INPUTS.bars).toEqual([]);
    expect(DEFAULT_INPUTS.period).toBe(DEFAULT_PERIOD);
    expect(DEFAULT_PERIOD).toBe(60);
    expect(MIN_PERIOD).toBe(2);
    expect(MAX_PERIOD).toBe(1000);
});
