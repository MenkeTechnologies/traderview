// ATR Channel helpers: parser, validator, localCompute mirror (EMA/SMA + Wilder ATR), badges, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, DEFAULT_PERIOD, DEFAULT_MULTIPLIER, DEFAULT_USE_EMA,
    MIN_PERIOD, MAX_PERIOD,
    parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute,
    sma, ema, positionBadge, trendBadge, widthBadge, summarizeBars,
    makeDemoInput,
    fmtPrice, fmtPriceSigned, fmtPct, fmtInt,
} from '../js/_atr_channel_inputs.js';

const b = (h, l, c) => ({ high: h, low: l, close: c });

// ── parser ────────────────────────────────────────────────────────

test('parseBarsBlob: 3 tokens per line', () => {
    const r = parseBarsBlob('101 99 100\n# midday\n102 100 101');
    expect(r.errors).toEqual([]);
    expect(r.bars).toEqual([b(101, 99, 100), b(102, 100, 101)]);
});

test('parseBarsBlob: rejects wrong count / non-positive / low > high / close OOR', () => {
    expect(parseBarsBlob('100 99').errors[0].message).toMatch(/3 tokens/);
    expect(parseBarsBlob('-1 99 100').errors[0].message).toMatch(/HLC/);
    expect(parseBarsBlob('99 100 99').errors[0].message).toMatch(/low > high/);
    expect(parseBarsBlob('101 99 50').errors[0].message).toMatch(/close outside/);
});

test('parseBarsBlob: non-string returns 1 error', () => {
    expect(parseBarsBlob(undefined).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts valid', () => {
    const bars = Array.from({ length: 30 }, () => b(101, 99, 100));
    expect(validateInputs({ bars, period: 20, multiplier: 2.0, use_ema: true })).toBe(null);
});

test('validate rejects: bad array / bad period / bad mult / non-bool use_ema / too short / non-finite / inverted', () => {
    const ok = Array.from({ length: 30 }, () => b(101, 99, 100));
    expect(validateInputs({ bars: 'no', period: 20, multiplier: 2, use_ema: true })).toMatch(/bars/);
    expect(validateInputs({ bars: ok, period: 1, multiplier: 2, use_ema: true })).toMatch(/period/);
    expect(validateInputs({ bars: ok, period: 9999, multiplier: 2, use_ema: true })).toMatch(/period/);
    expect(validateInputs({ bars: ok, period: 20, multiplier: 0, use_ema: true })).toMatch(/multiplier/);
    expect(validateInputs({ bars: ok, period: 20, multiplier: -1, use_ema: true })).toMatch(/multiplier/);
    expect(validateInputs({ bars: ok, period: 20, multiplier: 2, use_ema: 'true' })).toMatch(/use_ema/);
    expect(validateInputs({ bars: ok.slice(0, 5), period: 20, multiplier: 2, use_ema: true })).toMatch(/period \+ 1/);
    const badNan = [...ok];
    badNan[5] = { high: NaN, low: 99, close: 100 };
    expect(validateInputs({ bars: badNan, period: 20, multiplier: 2, use_ema: true })).toMatch(/finite/);
    const inv = [...ok];
    inv[5] = b(99, 101, 100);
    expect(validateInputs({ bars: inv, period: 20, multiplier: 2, use_ema: true })).toMatch(/high < low/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody passes through (trimming bars)', () => {
    const body = buildBody({ bars: [{ ...b(101, 99, 100), x: 1 }], period: 20, multiplier: 2, use_ema: false });
    expect(body).toEqual({ bars: [b(101, 99, 100)], period: 20, multiplier: 2, use_ema: false });
});

// ── sma / ema sanity ─────────────────────────────────────────────

test('sma: flat series → flat output equal to value', () => {
    const out = sma(new Array(20).fill(100), 5);
    for (let i = 4; i < 20; i++) expect(out[i]).toBe(100);
});

test('sma: leading nulls until period', () => {
    const out = sma([1, 2, 3, 4, 5], 3);
    expect(out[0]).toBe(null);
    expect(out[1]).toBe(null);
    expect(out[2]).toBeCloseTo(2, 6);
});

test('ema: flat series → flat output equal to value', () => {
    const out = ema(new Array(30).fill(100), 5);
    for (let i = 4; i < 30; i++) expect(out[i]).toBeCloseTo(100, 9);
});

test('ema: faster than sma on step up', () => {
    const closes = [...new Array(20).fill(100), ...new Array(10).fill(200)];
    const e = ema(closes, 10);
    const s = sma(closes, 10);
    expect(e[25]).toBeGreaterThan(s[25]);
});

// ── localCompute parity (mirrors Rust #[test]) ───────────────────

test('local: invalid inputs return empty middle', () => {
    const bars = Array.from({ length: 30 }, () => b(101, 99, 100));
    expect(localCompute(bars, 1, 2, true).middle.every(x => x === null)).toBe(true);
    expect(localCompute(bars, 20, 0, true).middle.every(x => x === null)).toBe(true);
});

test('local: NaN bar returns empty', () => {
    const bars = Array.from({ length: 30 }, () => b(101, 99, 100));
    bars[5] = b(NaN, 99, 100);
    expect(localCompute(bars, 20, 2, true).middle.every(x => x === null)).toBe(true);
});

test('local: flat market — bands at constant offset from middle', () => {
    const bars = Array.from({ length: 50 }, () => b(101, 99, 100));
    const r = localCompute(bars, 20, 2, true);
    const last = 49;
    expect(r.middle[last]).toBeCloseTo(100, 9);
    expect(Math.abs(r.upper[last] - 104)).toBeLessThan(0.1);
    expect(Math.abs(r.lower[last] - 96)).toBeLessThan(0.1);
});

test('local: upper > lower always', () => {
    const bars = Array.from({ length: 50 }, (_, i) => {
        const m = 100 + Math.sin(i * 0.3) * 5;
        return b(m + 1.5, m - 1.5, m);
    });
    const r = localCompute(bars, 20, 2, true);
    for (let i = 0; i < 50; i++) {
        if (r.upper[i] != null && r.lower[i] != null) {
            expect(r.upper[i]).toBeGreaterThan(r.lower[i]);
        }
    }
});

test('local: EMA vs SMA differ on step change', () => {
    const closes = [...new Array(30).fill(100), ...new Array(5).fill(200)];
    const bars = closes.map(c => b(c + 1, c - 1, c));
    const rEma = localCompute(bars, 20, 2, true);
    const rSma = localCompute(bars, 20, 2, false);
    expect(rEma.middle[34]).toBeGreaterThan(rSma.middle[34]);
});

test('local: output lengths match input', () => {
    const bars = Array.from({ length: 30 }, () => b(101, 99, 100));
    const r = localCompute(bars, 20, 2, true);
    expect(r.middle.length).toBe(30);
    expect(r.upper.length).toBe(30);
    expect(r.lower.length).toBe(30);
});

test('local: deterministic', () => {
    const bars = Array.from({ length: 30 }, (_, i) => b(100 + i + 1, 100 + i - 1, 100 + i));
    const r1 = localCompute(bars, 20, 2, true);
    const r2 = localCompute(bars, 20, 2, true);
    expect(r1.middle).toEqual(r2.middle);
    expect(r1.upper).toEqual(r2.upper);
    expect(r1.lower).toEqual(r2.lower);
});

test('local: higher multiplier → wider bands', () => {
    const bars = Array.from({ length: 50 }, (_, i) => {
        const m = 100 + Math.sin(i * 0.3) * 3;
        return b(m + 1, m - 1, m);
    });
    const r1 = localCompute(bars, 20, 1.0, true);
    const r3 = localCompute(bars, 20, 3.0, true);
    const i = 49;
    expect(r3.upper[i] - r3.lower[i]).toBeGreaterThan(r1.upper[i] - r1.lower[i]);
});

// ── badges ────────────────────────────────────────────────────────

test('positionBadge: tiers', () => {
    expect(positionBadge(105, 104, 96, 100).key).toMatch(/above_upper/);
    expect(positionBadge(101, 104, 96, 100).key).toMatch(/upper_half/);
    expect(positionBadge(100, 104, 96, 100).key).toMatch(/at_mid/);
    expect(positionBadge(99,  104, 96, 100).key).toMatch(/lower_half/);
    expect(positionBadge(95,  104, 96, 100).key).toMatch(/below_lower/);
    expect(positionBadge(null, 104, 96, 100).key).toMatch(/unknown/);
});

test('trendBadge: tiers', () => {
    expect(trendBadge([100, 100, 100, 100, 100]).key).toMatch(/flat/);
    expect(trendBadge([100, 101, 102, 103, 110]).key).toMatch(/up_strong/);
    expect(trendBadge([100, 100.5, 100.7, 100.8, 100.9]).key).toMatch(/up/);
    expect(trendBadge([110, 109, 108, 107, 100]).key).toMatch(/down_strong/);
    expect(trendBadge([]).key).toMatch(/unknown/);
});

test('widthBadge: tiers', () => {
    expect(widthBadge(110, 90, 100).key).toMatch(/very_wide/);   // 20%
    expect(widthBadge(105, 95, 100).key).toMatch(/wide/);        // 10%
    expect(widthBadge(102, 98, 100).key).toMatch(/wide|normal/); // 4%
    expect(widthBadge(101, 99, 100).key).toMatch(/narrow/);      // 2%
    expect(widthBadge(100.4, 99.7, 100).key).toMatch(/very_narrow|narrow/);
    expect(widthBadge(101, 99, 0).key).toMatch(/unknown/);
});

// ── summarize ─────────────────────────────────────────────────────

test('summarizeBars: count / last_close / min_low / max_high / mean_close', () => {
    const bars = [b(102, 98, 100), b(105, 100, 103), b(106, 101, 105)];
    const s = summarizeBars(bars);
    expect(s.count).toBe(3);
    expect(s.last_close).toBe(105);
    expect(s.min_low).toBe(98);
    expect(s.max_high).toBe(106);
});

test('summarizeBars: empty → NaN', () => {
    const s = summarizeBars([]);
    expect(s.count).toBe(0);
    expect(Number.isNaN(s.last_close)).toBe(true);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + computes cleanly', () => {
    for (const k of ['uptrend','downtrend','volatile-side','tight-side',
                     'breakout','breakdown','sma','wide-bands']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.bars, inp.period, inp.multiplier, inp.use_ema);
        expect(r.middle.length).toBe(inp.bars.length);
        expect(r.upper.length).toBe(inp.bars.length);
        expect(r.lower.length).toBe(inp.bars.length);
    }
});

test('demo wide-bands has wider envelope than uptrend', () => {
    const wide = makeDemoInput('wide-bands');
    const norm = makeDemoInput('uptrend');
    const rW = localCompute(wide.bars, wide.period, wide.multiplier, wide.use_ema);
    const rN = localCompute(norm.bars, norm.period, norm.multiplier, norm.use_ema);
    expect(rW.upper[rW.upper.length - 1] - rW.lower[rW.lower.length - 1])
        .toBeGreaterThan(rN.upper[rN.upper.length - 1] - rN.lower[rN.lower.length - 1]);
});

test('demo sma uses SMA midline', () => {
    const inp = makeDemoInput('sma');
    expect(inp.use_ema).toBe(false);
});

// ── formatters ────────────────────────────────────────────────────

test('barsToBlob round-trips', () => {
    const bars = [b(101, 99, 100), b(102, 100, 101)];
    const back = parseBarsBlob(barsToBlob(bars));
    expect(back.errors).toEqual([]);
    expect(back.bars).toEqual(bars);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtPrice(100.456)).toBe('100.46');
    expect(fmtPriceSigned(1.5)).toBe('+1.50');
    expect(fmtPriceSigned(-1.5)).toBe('-1.50');
    expect(fmtPct(0.0125)).toBe('1.25%');
    expect(fmtInt(42.9)).toBe('42');
    expect(fmtPrice(NaN)).toBe('—');
});

test('DEFAULTS sanity', () => {
    expect(DEFAULT_INPUTS.bars).toEqual([]);
    expect(DEFAULT_INPUTS.period).toBe(DEFAULT_PERIOD);
    expect(DEFAULT_INPUTS.multiplier).toBe(DEFAULT_MULTIPLIER);
    expect(DEFAULT_INPUTS.use_ema).toBe(DEFAULT_USE_EMA);
    expect(DEFAULT_PERIOD).toBe(20);
    expect(DEFAULT_MULTIPLIER).toBe(2);
    expect(DEFAULT_USE_EMA).toBe(true);
    expect(MIN_PERIOD).toBe(2);
    expect(MAX_PERIOD).toBe(500);
});
