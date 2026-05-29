// ATR Trailing Stop helpers: parser, validator, localCompute mirror (Wilder ATR + ratchet), badges, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, DEFAULT_PERIOD, DEFAULT_MULTIPLIER,
    MIN_PERIOD, MAX_PERIOD,
    parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute,
    longBadge, shortBadge, regimeBadge, summarizeBars,
    makeDemoInput,
    fmtPrice, fmtPriceSigned, fmtPct, fmtInt,
} from '../js/_atr_trail_stop_inputs.js';

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
    expect(validateInputs({ bars, period: 14, multiplier: 3.0 })).toBe(null);
});

test('validate rejects: bad array / bad period / bad multiplier / too short / non-finite / inverted', () => {
    const ok = Array.from({ length: 30 }, () => b(101, 99, 100));
    expect(validateInputs({ bars: 'no', period: 14, multiplier: 3 })).toMatch(/bars/);
    expect(validateInputs({ bars: ok, period: 1, multiplier: 3 })).toMatch(/period/);
    expect(validateInputs({ bars: ok, period: 9999, multiplier: 3 })).toMatch(/period/);
    expect(validateInputs({ bars: ok, period: 14, multiplier: 0 })).toMatch(/multiplier/);
    expect(validateInputs({ bars: ok, period: 14, multiplier: -1 })).toMatch(/multiplier/);
    expect(validateInputs({ bars: ok.slice(0, 5), period: 14, multiplier: 3 })).toMatch(/period \+ 1/);
    const badNan = [...ok];
    badNan[5] = { high: NaN, low: 99, close: 100 };
    expect(validateInputs({ bars: badNan, period: 14, multiplier: 3 })).toMatch(/finite/);
    const inv = [...ok];
    inv[5] = b(99, 101, 100);
    expect(validateInputs({ bars: inv, period: 14, multiplier: 3 })).toMatch(/high < low/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody trims bars to HLC', () => {
    const body = buildBody({ bars: [{ ...b(101, 99, 100), x: 1 }], period: 14, multiplier: 3 });
    expect(body).toEqual({ bars: [b(101, 99, 100)], period: 14, multiplier: 3 });
});

// ── localCompute parity (mirrors Rust #[test]) ───────────────────

test('local: invalid inputs return empty', () => {
    const bars = Array.from({ length: 30 }, () => b(101, 99, 100));
    expect(localCompute(bars, 1, 3).long_stop.every(x => x === null)).toBe(true);
    expect(localCompute(bars, 14, 0).long_stop.every(x => x === null)).toBe(true);
});

test('local: NaN bar returns empty', () => {
    const bars = Array.from({ length: 30 }, () => b(101, 99, 100));
    bars[5] = b(NaN, 99, 100);
    expect(localCompute(bars, 14, 3).long_stop.every(x => x === null)).toBe(true);
});

test('local: flat market — stops at constant offset around close', () => {
    const bars = Array.from({ length: 50 }, () => b(101, 99, 100));
    const r = localCompute(bars, 14, 3);
    const last = 49;
    expect(Math.abs(r.long_stop[last] - 94)).toBeLessThan(0.1);
    expect(Math.abs(r.short_stop[last] - 106)).toBeLessThan(0.1);
});

test('local: long stop ratchets up in uptrend', () => {
    const bars = Array.from({ length: 50 }, (_, i) => {
        const m = 100 + i;
        return b(m + 0.5, m - 0.5, m);
    });
    const r = localCompute(bars, 14, 3);
    const vals = r.long_stop.filter(v => v != null);
    for (let i = 1; i < vals.length; i++) {
        expect(vals[i]).toBeGreaterThanOrEqual(vals[i - 1] - 1e-9);
    }
});

test('local: short stop ratchets down in downtrend', () => {
    const bars = Array.from({ length: 50 }, (_, i) => {
        const m = 200 - i;
        return b(m + 0.5, m - 0.5, m);
    });
    const r = localCompute(bars, 14, 3);
    const vals = r.short_stop.filter(v => v != null);
    for (let i = 1; i < vals.length; i++) {
        expect(vals[i]).toBeLessThanOrEqual(vals[i - 1] + 1e-9);
    }
});

test('local: output lengths match input', () => {
    const bars = Array.from({ length: 30 }, () => b(101, 99, 100));
    const r = localCompute(bars, 14, 3);
    expect(r.long_stop.length).toBe(30);
    expect(r.short_stop.length).toBe(30);
});

test('local: deterministic', () => {
    const bars = Array.from({ length: 30 }, (_, i) => b(100 + i + 1, 100 + i - 1, 100 + i));
    const r1 = localCompute(bars, 14, 3);
    const r2 = localCompute(bars, 14, 3);
    expect(r1.long_stop).toEqual(r2.long_stop);
    expect(r1.short_stop).toEqual(r2.short_stop);
});

test('local: long_stop < close < short_stop everywhere populated', () => {
    const bars = Array.from({ length: 50 }, (_, i) => {
        const m = 100 + Math.sin(i * 0.3) * 5;
        return b(m + 1.5, m - 1.5, m);
    });
    const r = localCompute(bars, 14, 3);
    for (let i = 0; i < 50; i++) {
        if (r.long_stop[i] != null) {
            expect(r.long_stop[i]).toBeLessThan(r.short_stop[i]);
        }
    }
});

test('local: higher multiplier → wider stops', () => {
    const bars = Array.from({ length: 50 }, (_, i) => {
        const m = 100 + Math.sin(i * 0.3) * 3;
        return b(m + 1, m - 1, m);
    });
    const r1 = localCompute(bars, 14, 1);
    const r3 = localCompute(bars, 14, 3);
    const i = 49;
    // 3x mult must produce wider distance from close (long_stop further below + short_stop further above).
    expect(r3.short_stop[i] - r3.long_stop[i])
        .toBeGreaterThan(r1.short_stop[i] - r1.long_stop[i]);
});

// ── badges ────────────────────────────────────────────────────────

test('longBadge: tiers', () => {
    expect(longBadge(110, 100).key).toMatch(/safe/);          // 10% margin
    expect(longBadge(103, 100).key).toMatch(/holding/);       // 3% margin
    expect(longBadge(101, 100).key).toMatch(/tight/);         // 1% margin
    expect(longBadge(100, 100).key).toMatch(/triggered/);     // at stop
    expect(longBadge(95,  100).key).toMatch(/triggered/);     // below stop
    expect(longBadge(null, 100).key).toMatch(/unknown/);
});

test('shortBadge: tiers', () => {
    expect(shortBadge(90,  100).key).toMatch(/safe/);
    expect(shortBadge(97,  100).key).toMatch(/holding/);
    expect(shortBadge(99,  100).key).toMatch(/tight/);
    expect(shortBadge(100, 100).key).toMatch(/triggered/);
    expect(shortBadge(105, 100).key).toMatch(/triggered/);
    expect(shortBadge(null, 100).key).toMatch(/unknown/);
});

test('regimeBadge: long_bias / short_bias / balanced / both_triggered / long_only / short_only', () => {
    // (close, long_stop, short_stop); long margin = (close-long_stop)/long_stop, short = (short_stop-close)/short_stop.
    // Long bias: longMargin > shortMargin * 1.5
    expect(regimeBadge(110, 80, 120).key).toMatch(/long_bias/);   // L=0.375, S=0.083
    // Short bias: shortMargin > longMargin * 1.5
    expect(regimeBadge(110, 105, 130).key).toMatch(/short_bias/); // L=0.048, S=0.154
    // Balanced: roughly equal margins
    expect(regimeBadge(100, 95, 105).key).toMatch(/balanced/);    // L=0.053, S=0.048
    // close ≤ long_stop AND close < short_stop → short_only (long triggered, short holds)
    expect(regimeBadge(100, 105, 110).key).toMatch(/short_only/);
    // close > long_stop AND close ≥ short_stop → long_only (short triggered, long holds)
    expect(regimeBadge(105, 100, 95).key).toMatch(/long_only/);
    // both triggered
    expect(regimeBadge(100, 105, 95).key).toMatch(/both_triggered/);
    expect(regimeBadge(null, 100, 110).key).toMatch(/unknown/);
});

// ── summarize ─────────────────────────────────────────────────────

test('summarizeBars: count / last_close / extrema / mean', () => {
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

test('demos: each preset validates + computes', () => {
    for (const k of ['uptrend','downtrend','sideways','long-trigger',
                     'short-trigger','tight-mult','wide-mult','flat']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.bars, inp.period, inp.multiplier);
        expect(r.long_stop.length).toBe(inp.bars.length);
        expect(r.short_stop.length).toBe(inp.bars.length);
    }
});

test('demo uptrend: long_stop monotone non-decreasing', () => {
    const inp = makeDemoInput('uptrend');
    const r = localCompute(inp.bars, inp.period, inp.multiplier);
    const vals = r.long_stop.filter(v => v != null);
    for (let i = 1; i < vals.length; i++) {
        expect(vals[i]).toBeGreaterThanOrEqual(vals[i - 1] - 1e-9);
    }
});

test('demo wide-mult has wider stops than tight-mult', () => {
    const tight = makeDemoInput('tight-mult');
    const wide  = makeDemoInput('wide-mult');
    const rT = localCompute(tight.bars, tight.period, tight.multiplier);
    const rW = localCompute(wide.bars,  wide.period,  wide.multiplier);
    const i = 59;
    expect(rW.short_stop[i] - rW.long_stop[i])
        .toBeGreaterThan(rT.short_stop[i] - rT.long_stop[i]);
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
    expect(DEFAULT_PERIOD).toBe(14);
    expect(DEFAULT_MULTIPLIER).toBe(3.0);
    expect(MIN_PERIOD).toBe(2);
    expect(MAX_PERIOD).toBe(500);
});
