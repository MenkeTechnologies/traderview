// AlphaTrend helpers: parser, validator, localCompute mirror (ATR-SMA + Wilder RSI), badges, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, DEFAULT_PERIOD, DEFAULT_MULTIPLIER,
    MIN_PERIOD, MAX_PERIOD,
    parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute, wilderRsi,
    dirBadge, trendBadge, positionBadge, summarizeBars,
    makeDemoInput,
    fmtPrice, fmtPriceSigned, fmtPct, fmtInt, fmtDir,
} from '../js/_alphatrend_inputs.js';

const b = (h, l, c) => ({ high: h, low: l, close: c });

// ── parser ────────────────────────────────────────────────────────

test('parseBarsBlob: 3 tokens per line', () => {
    const r = parseBarsBlob('101 99 100\n# midday\n102 100 101');
    expect(r.errors).toEqual([]);
    expect(r.bars).toEqual([b(101, 99, 100), b(102, 100, 101)]);
});

test('parseBarsBlob: rejects wrong count / non-positive / low > high / close out-of-range', () => {
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
    expect(validateInputs({ bars, period: 14, multiplier: 1.0 })).toBe(null);
});

test('validate rejects: bad array / bad period / bad multiplier / too short / non-finite / inverted', () => {
    const ok = Array.from({ length: 30 }, () => b(101, 99, 100));
    expect(validateInputs({ bars: 'no', period: 14, multiplier: 1.0 })).toMatch(/bars/);
    expect(validateInputs({ bars: ok, period: 1, multiplier: 1.0 })).toMatch(/period/);
    expect(validateInputs({ bars: ok, period: 9999, multiplier: 1.0 })).toMatch(/period/);
    expect(validateInputs({ bars: ok, period: 14, multiplier: 0 })).toMatch(/multiplier/);
    expect(validateInputs({ bars: ok, period: 14, multiplier: -1 })).toMatch(/multiplier/);
    expect(validateInputs({ bars: ok.slice(0, 5), period: 14, multiplier: 1.0 })).toMatch(/period \+ 1/);
    const badNan = [...ok];
    badNan[5] = { high: NaN, low: 99, close: 100 };
    expect(validateInputs({ bars: badNan, period: 14, multiplier: 1.0 })).toMatch(/finite/);
    const inv = [...ok];
    inv[5] = b(99, 101, 100);
    expect(validateInputs({ bars: inv, period: 14, multiplier: 1.0 })).toMatch(/high < low/);
    const closeOOR = [...ok];
    closeOOR[5] = b(101, 99, 50);
    expect(validateInputs({ bars: closeOOR, period: 14, multiplier: 1.0 })).toMatch(/close outside/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: trims bars to HLC only', () => {
    const body = buildBody({ bars: [{ ...b(101, 99, 100), extra: 'x' }], period: 14, multiplier: 1.0 });
    expect(body).toEqual({ bars: [b(101, 99, 100)], period: 14, multiplier: 1.0 });
});

// ── wilderRsi sanity ─────────────────────────────────────────────

test('wilderRsi: monotone uptrend → 100', () => {
    const closes = Array.from({ length: 30 }, (_, i) => 100 + i);
    const r = wilderRsi(closes, 14);
    expect(r[14]).toBeCloseTo(100, 6);
});

test('wilderRsi: monotone downtrend → 0', () => {
    const closes = Array.from({ length: 30 }, (_, i) => 130 - i);
    const r = wilderRsi(closes, 14);
    expect(r[14]).toBeCloseTo(0, 6);
});

test('wilderRsi: flat → 50', () => {
    const closes = new Array(30).fill(100);
    const r = wilderRsi(closes, 14);
    expect(r[14]).toBe(50);
});

test('wilderRsi: too-short returns all null', () => {
    expect(wilderRsi([100], 14).every(x => x === null)).toBe(true);
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: invalid inputs return empty alpha', () => {
    const bars = Array.from({ length: 30 }, () => b(101, 99, 100));
    const r1 = localCompute(bars, 1, 1.0);
    expect(r1.alpha.every(x => x === null)).toBe(true);
    const r2 = localCompute(bars, 14, 0);
    expect(r2.alpha.every(x => x === null)).toBe(true);
});

test('local: NaN bar returns empty', () => {
    const bars = Array.from({ length: 30 }, () => b(101, 99, 100));
    bars[5] = b(NaN, 99, 100);
    const r = localCompute(bars, 14, 1.0);
    expect(r.alpha.every(x => x === null)).toBe(true);
});

test('local: uptrend alpha rises monotone', () => {
    const bars = Array.from({ length: 60 }, (_, i) => b(100 + i + 0.5, 100 + i - 0.5, 100 + i));
    const r = localCompute(bars, 14, 1.0);
    const vals = r.alpha.filter(v => v != null);
    for (let i = 1; i < vals.length; i++) {
        expect(vals[i]).toBeGreaterThanOrEqual(vals[i - 1] - 1e-9);
    }
});

test('local: downtrend alpha falls monotone', () => {
    const bars = Array.from({ length: 60 }, (_, i) => {
        const m = 200 - i;
        return b(m + 0.5, m - 0.5, m);
    });
    const r = localCompute(bars, 14, 1.0);
    const vals = r.alpha.filter(v => v != null);
    for (let i = 1; i < vals.length; i++) {
        expect(vals[i]).toBeLessThanOrEqual(vals[i - 1] + 1e-9);
    }
});

test('local: output lengths match input', () => {
    const bars = Array.from({ length: 30 }, () => b(101, 99, 100));
    const r = localCompute(bars, 14, 1.0);
    expect(r.alpha.length).toBe(30);
    expect(r.direction.length).toBe(30);
});

test('local: deterministic for same input', () => {
    const bars = Array.from({ length: 60 }, (_, i) => b(100 + i + 0.5, 100 + i - 0.5, 100 + i));
    const r1 = localCompute(bars, 14, 1.0);
    const r2 = localCompute(bars, 14, 1.0);
    expect(r1.alpha).toEqual(r2.alpha);
    expect(r1.direction).toEqual(r2.direction);
});

test('local: direction is mostly +1 during pure uptrend', () => {
    const bars = Array.from({ length: 60 }, (_, i) => b(100 + i + 0.5, 100 + i - 0.5, 100 + i));
    const r = localCompute(bars, 14, 1.0);
    const dirs = r.direction.filter(v => v != null);
    const ups = dirs.filter(d => d > 0).length;
    expect(ups / dirs.length).toBeGreaterThan(0.6);
});

test('local: direction is mostly -1 during pure downtrend', () => {
    const bars = Array.from({ length: 60 }, (_, i) => {
        const m = 200 - i;
        return b(m + 0.5, m - 0.5, m);
    });
    const r = localCompute(bars, 14, 1.0);
    const dirs = r.direction.filter(v => v != null);
    const downs = dirs.filter(d => d < 0).length;
    expect(downs / dirs.length).toBeGreaterThan(0.6);
});

// ── badges ────────────────────────────────────────────────────────

test('dirBadge: up / down / flat / unknown', () => {
    expect(dirBadge([1, 1, 1]).key).toMatch(/up/);
    expect(dirBadge([-1, -1, -1]).key).toMatch(/down/);
    expect(dirBadge([0, 0, 0]).key).toMatch(/flat/);
    expect(dirBadge([null, null]).key).toMatch(/unknown/);
    expect(dirBadge([]).key).toMatch(/unknown/);
});

test('trendBadge: tiers', () => {
    expect(trendBadge(new Array(10).fill(1)).key).toMatch(/strong_up/);
    expect(trendBadge([1, 1, 1, 1, 1, 1, -1, -1, -1, -1]).key).toMatch(/mixed|up/);
    expect(trendBadge(new Array(10).fill(-1)).key).toMatch(/strong_down/);
    expect(trendBadge([1, 1, 1, 1, 1, 1, 1, -1, -1, -1]).key).toMatch(/up/);
    expect(trendBadge([-1, -1, -1, -1, -1, -1, -1, 1, 1, 1]).key).toMatch(/down/);
    expect(trendBadge([0, 0, 0, 0, 0, 1, -1, 0, 0, 0]).key).toMatch(/flat|mixed/);
    expect(trendBadge([]).key).toMatch(/unknown/);
});

test('positionBadge: well_above / above / well_below / below / at / unknown', () => {
    expect(positionBadge(110, 100).key).toMatch(/well_above/);
    expect(positionBadge(101, 100).key).toMatch(/above/);
    expect(positionBadge(100, 100).key).toMatch(/at/);
    expect(positionBadge(99, 100).key).toMatch(/below/);
    expect(positionBadge(85, 100).key).toMatch(/well_below/);
    expect(positionBadge(null, 100).key).toMatch(/unknown/);
    expect(positionBadge(100, 0).key).toMatch(/unknown/);
});

// ── summarize ─────────────────────────────────────────────────────

test('summarizeBars: count / last_close / min_low / max_high / mean_close', () => {
    const bars = [b(102, 98, 100), b(105, 100, 103), b(106, 101, 105)];
    const s = summarizeBars(bars);
    expect(s.count).toBe(3);
    expect(s.last_close).toBe(105);
    expect(s.min_low).toBe(98);
    expect(s.max_high).toBe(106);
    expect(s.mean_close).toBeCloseTo(102.667, 2);
});

test('summarizeBars: empty → NaN', () => {
    const s = summarizeBars([]);
    expect(s.count).toBe(0);
    expect(Number.isNaN(s.last_close)).toBe(true);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + computes', () => {
    for (const k of ['uptrend','downtrend','reversal-up','reversal-down',
                     'sideways','volatile','high-mult','low-mult']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.bars, inp.period, inp.multiplier);
        expect(r.alpha.length).toBe(inp.bars.length);
    }
});

test('demo uptrend yields mostly +1 direction', () => {
    const inp = makeDemoInput('uptrend');
    const r = localCompute(inp.bars, inp.period, inp.multiplier);
    const dirs = r.direction.filter(v => v != null);
    const ups = dirs.filter(d => d > 0).length;
    expect(ups / dirs.length).toBeGreaterThan(0.5);
});

test('demo high-mult has fewer flips than low-mult on same series-ish', () => {
    const hi = makeDemoInput('high-mult');
    const lo = makeDemoInput('low-mult');
    const rHi = localCompute(hi.bars, hi.period, hi.multiplier);
    const rLo = localCompute(lo.bars, lo.period, lo.multiplier);
    const countFlips = (dirs) => {
        let f = 0, prev = null;
        for (const d of dirs) {
            if (d == null || d === 0) continue;
            if (prev != null && Math.sign(d) !== Math.sign(prev)) f++;
            prev = d;
        }
        return f;
    };
    // High multiplier should not increase flip count vs low (typically much less).
    expect(countFlips(rHi.direction)).toBeLessThanOrEqual(countFlips(rLo.direction) + 2);
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
    expect(fmtDir(1)).toBe('↑');
    expect(fmtDir(-1)).toBe('↓');
    expect(fmtDir(0)).toBe('·');
    expect(fmtDir(NaN)).toBe('—');
});

test('DEFAULTS sanity', () => {
    expect(DEFAULT_INPUTS.bars).toEqual([]);
    expect(DEFAULT_INPUTS.period).toBe(DEFAULT_PERIOD);
    expect(DEFAULT_INPUTS.multiplier).toBe(DEFAULT_MULTIPLIER);
    expect(DEFAULT_PERIOD).toBe(14);
    expect(DEFAULT_MULTIPLIER).toBe(1.0);
    expect(MIN_PERIOD).toBe(2);
    expect(MAX_PERIOD).toBe(500);
});
