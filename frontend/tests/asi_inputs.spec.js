// Accumulation Swing Index helpers: parser, validator, Wilder formula mirror, badges.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, DEFAULT_LIMIT_MOVE,
    parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute,
    trendBadge, breakoutBadge, biasBadge, summarizeBars,
    makeDemoInput,
    fmtNum, fmtSigned, fmtPrice, fmtInt,
} from '../js/_asi_inputs.js';

const bar = (o, h, l, c) => ({ open: o, high: h, low: l, close: c });

// ── parser ────────────────────────────────────────────────────────

test('parseBarsBlob: 4 tokens per line (OHLC)', () => {
    const r = parseBarsBlob('100 101 99 100.5\n# midday\n100.5 102 100 101.5');
    expect(r.errors).toEqual([]);
    expect(r.bars).toEqual([bar(100, 101, 99, 100.5), bar(100.5, 102, 100, 101.5)]);
});

test('parseBarsBlob: rejects wrong count / non-positive / low > high / close OOR / open OOR', () => {
    expect(parseBarsBlob('100 101 99').errors[0].message).toMatch(/4 tokens/);
    expect(parseBarsBlob('-1 101 99 100').errors[0].message).toMatch(/OHLC/);
    expect(parseBarsBlob('100 99 101 100').errors[0].message).toMatch(/low > high/);
    expect(parseBarsBlob('100 101 99 50').errors[0].message).toMatch(/close outside/);
    expect(parseBarsBlob('50 101 99 100').errors[0].message).toMatch(/open outside/);
});

test('parseBarsBlob: non-string returns 1 error', () => {
    expect(parseBarsBlob(undefined).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts valid', () => {
    expect(validateInputs({ bars: [bar(100, 101, 99, 100.5)], limit_move: 10 })).toBe(null);
});

test('validate rejects: bad array / empty / bad limit_move / non-finite / inverted / OHL violations', () => {
    expect(validateInputs({ bars: 'no', limit_move: 10 })).toMatch(/bars/);
    expect(validateInputs({ bars: [], limit_move: 10 })).toMatch(/empty/);
    expect(validateInputs({ bars: [bar(100, 101, 99, 100)], limit_move: 0 })).toMatch(/limit_move/);
    expect(validateInputs({ bars: [bar(100, 101, 99, 100)], limit_move: -1 })).toMatch(/limit_move/);
    expect(validateInputs({ bars: [bar(NaN, 101, 99, 100)], limit_move: 10 })).toMatch(/finite/);
    expect(validateInputs({ bars: [bar(100, 99, 101, 100)], limit_move: 10 })).toMatch(/high < low/);
    expect(validateInputs({ bars: [bar(100, 101, 99, 50)],  limit_move: 10 })).toMatch(/close outside/);
    expect(validateInputs({ bars: [bar(50,  101, 99, 100)], limit_move: 10 })).toMatch(/open outside/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody trims bars to OHLC', () => {
    const body = buildBody({ bars: [{ ...bar(100, 101, 99, 100.5), x: 'y' }], limit_move: 10 });
    expect(body).toEqual({ bars: [bar(100, 101, 99, 100.5)], limit_move: 10 });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: empty returns empty', () => {
    expect(localCompute([], 10)).toEqual([]);
});

test('local: invalid limit_move returns all null', () => {
    const bars = Array.from({ length: 5 }, () => bar(100, 101, 99, 100.5));
    expect(localCompute(bars, 0).every(x => x === null)).toBe(true);
    expect(localCompute(bars, NaN).every(x => x === null)).toBe(true);
    expect(localCompute(bars, -1).every(x => x === null)).toBe(true);
});

test('local: NaN bar returns all null', () => {
    const bars = [bar(100, 101, 99, 100.5), bar(NaN, 102, 100, 101.5)];
    expect(localCompute(bars, 10).every(x => x === null)).toBe(true);
});

test('local: first bar is 0', () => {
    const bars = [bar(100, 101, 99, 100.5)];
    expect(Math.abs(localCompute(bars, 10)[0])).toBeLessThan(1e-9);
});

test('local: uptrending bars yield positive last ASI', () => {
    const bars = Array.from({ length: 30 }, (_, i) => {
        const p = 100 + i;
        return bar(p, p + 0.5, p - 0.5, p + 0.4);
    });
    const r = localCompute(bars, 10);
    expect(r[29]).toBeGreaterThan(0);
});

test('local: downtrending bars yield negative last ASI', () => {
    const bars = Array.from({ length: 30 }, (_, i) => {
        const p = 200 - i;
        return bar(p, p + 0.5, p - 0.5, p - 0.4);
    });
    const r = localCompute(bars, 10);
    expect(r[29]).toBeLessThan(0);
});

test('local: flat market yields ASI = 0 throughout', () => {
    const bars = Array.from({ length: 30 }, () => bar(100, 101, 99, 100));
    const r = localCompute(bars, 10);
    for (const v of r) expect(Math.abs(v)).toBeLessThan(1e-9);
});

test('local: output length matches input', () => {
    const bars = Array.from({ length: 30 }, () => bar(100, 101, 99, 100.5));
    expect(localCompute(bars, 10).length).toBe(30);
});

test('local: deterministic', () => {
    const bars = Array.from({ length: 20 }, (_, i) => bar(100 + i * 0.1, 101 + i * 0.1, 99 + i * 0.1, 100.5 + i * 0.1));
    expect(localCompute(bars, 10)).toEqual(localCompute(bars, 10));
});

test('local: tighter limit_move → larger ASI magnitude (same series)', () => {
    const bars = Array.from({ length: 20 }, (_, i) => {
        const p = 100 + i;
        return bar(p, p + 0.5, p - 0.5, p + 0.4);
    });
    const tight = localCompute(bars, 1);
    const loose = localCompute(bars, 10);
    expect(Math.abs(tight[19])).toBeGreaterThan(Math.abs(loose[19]));
});

test('local: r == 0 case carries ASI forward unchanged', () => {
    // Construct a bar where high == low == prev.close AND prev.close == prev.open
    // → a = 0, b = 0, c = 0, d = 0, R = 0 → fall through.
    const bars = [
        bar(100, 100, 100, 100),  // doji
        bar(100, 100, 100, 100),  // doji
    ];
    const r = localCompute(bars, 10);
    expect(r[1]).toBe(0);   // r=0 path keeps ASI at prior value (0)
});

// ── badges ────────────────────────────────────────────────────────

test('trendBadge: tiers', () => {
    expect(trendBadge([0, 0, 0, 0, 0, 0, 0, 0, 0, 0]).key).toMatch(/flat/);
    expect(trendBadge([0, 1, 2, 3, 4, 5, 6, 7, 8, 100]).key).toMatch(/strong_up/);
    expect(trendBadge([10, 10.5, 10.7, 10.8, 10.9, 11, 11.2, 11.3, 11.5, 11.7]).key).toMatch(/up/);
    expect(trendBadge([100, 90, 80, 70, 60, 50, 40, 30, 20, 0]).key).toMatch(/strong_down/);
    expect(trendBadge([10, 9.9, 9.8, 9.7, 9.6, 9.5, 9.4, 9.3, 9.2, 9.1]).key).toMatch(/down/);
    expect(trendBadge([]).key).toMatch(/unknown/);
});

test('breakoutBadge: up / down / none / unknown', () => {
    expect(breakoutBadge([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]).key).toMatch(/up/);
    expect(breakoutBadge([10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0]).key).toMatch(/down/);
    expect(breakoutBadge([10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 5]).key).toMatch(/none/);   // 5 within prior window [10,9,...,9]
    expect(breakoutBadge([5]).key).toMatch(/unknown/);
    expect(breakoutBadge([]).key).toMatch(/unknown/);
});

test('biasBadge: bullish / bearish / neutral / unknown', () => {
    expect(biasBadge(100).key).toMatch(/bullish/);
    expect(biasBadge(-100).key).toMatch(/bearish/);
    expect(biasBadge(0).key).toMatch(/neutral/);
    expect(biasBadge(null).key).toMatch(/unknown/);
});

// ── summarize ─────────────────────────────────────────────────────

test('summarizeBars: counts up/down bars, extrema, mean', () => {
    const bars = [
        bar(100, 102, 99, 101),    // up
        bar(101, 103, 100, 100),   // down
        bar(100, 105, 99, 104),    // up
    ];
    const s = summarizeBars(bars);
    expect(s.count).toBe(3);
    expect(s.last_close).toBe(104);
    expect(s.up_bars).toBe(2);
    expect(s.down_bars).toBe(1);
    expect(s.max_high).toBe(105);
    expect(s.min_low).toBe(99);
});

test('summarizeBars: empty → NaN', () => {
    const s = summarizeBars([]);
    expect(s.count).toBe(0);
    expect(Number.isNaN(s.last_close)).toBe(true);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + computes', () => {
    for (const k of ['uptrend','downtrend','sideways','reversal-up',
                     'reversal-down','wide-bars','tight-limit','flat-doji']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.bars, inp.limit_move);
        expect(r.length).toBe(inp.bars.length);
        expect(r[0]).toBe(0);  // first bar always 0
    }
});

test('demo uptrend: last ASI > 0', () => {
    const inp = makeDemoInput('uptrend');
    const r = localCompute(inp.bars, inp.limit_move);
    expect(r[r.length - 1]).toBeGreaterThan(0);
});

test('demo downtrend: last ASI < 0', () => {
    const inp = makeDemoInput('downtrend');
    const r = localCompute(inp.bars, inp.limit_move);
    expect(r[r.length - 1]).toBeLessThan(0);
});

test('demo flat-doji: every ASI value is 0 (numerator = 0 ⇒ SI = 0)', () => {
    const inp = makeDemoInput('flat-doji');
    const r = localCompute(inp.bars, inp.limit_move);
    for (const v of r) expect(v).toBe(0);
});

test('demo tight-limit has larger ASI magnitude than uptrend (same series, smaller limit)', () => {
    const tight = makeDemoInput('tight-limit');
    const norm  = makeDemoInput('uptrend');
    const rT = localCompute(tight.bars, tight.limit_move);
    const rN = localCompute(norm.bars,  norm.limit_move);
    expect(Math.abs(rT[rT.length - 1])).toBeGreaterThan(Math.abs(rN[rN.length - 1]));
});

// ── formatters ────────────────────────────────────────────────────

test('barsToBlob round-trips', () => {
    const bars = [bar(100, 101, 99, 100.5), bar(100.5, 102, 100, 101.5)];
    const back = parseBarsBlob(barsToBlob(bars));
    expect(back.errors).toEqual([]);
    expect(back.bars).toEqual(bars);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtNum(1500)).toBe('1.50k');
    expect(fmtNum(1_500_000)).toBe('1.50M');
    expect(fmtNum(42.5)).toBe('42.50');
    expect(fmtSigned(1.5)).toBe('+1.50');
    expect(fmtSigned(-1.5)).toBe('-1.50');
    expect(fmtPrice(100.456)).toBe('100.46');
    expect(fmtInt(42.9)).toBe('42');
    expect(fmtNum(NaN)).toBe('—');
    expect(fmtPrice(NaN)).toBe('—');
});

test('DEFAULTS sanity', () => {
    expect(DEFAULT_INPUTS.bars).toEqual([]);
    expect(DEFAULT_INPUTS.limit_move).toBe(DEFAULT_LIMIT_MOVE);
    expect(DEFAULT_LIMIT_MOVE).toBe(10);
});
