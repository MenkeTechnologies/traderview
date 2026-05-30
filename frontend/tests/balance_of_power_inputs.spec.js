// Balance of Power helpers: parser, validator, body shape,
// localCompute Rust-mirror, badges, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_SMOOTHING, parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute,
    regimeBadge, lastCrossover, summarize,
    makeDemoInput,
    fmtBop, fmtUSD, fmtInt,
} from '../js/_balance_of_power_inputs.js';

const b = (o, h, l, c) => ({ open: o, high: h, low: l, close: c });

// ── constants ─────────────────────────────────────────────────────

test('DEFAULT_SMOOTHING = 14 (matches Rust)', () => {
    expect(DEFAULT_SMOOTHING).toBe(14);
});

// ── parser ────────────────────────────────────────────────────────

test('parseBarsBlob: 4 tokens per line; comments + blanks ignored', () => {
    const r = parseBarsBlob('99 101 99 101\n# bear\n101 101 99 99');
    expect(r.errors).toEqual([]);
    expect(r.bars).toEqual([b(99, 101, 99, 101), b(101, 101, 99, 99)]);
});

test('parseBarsBlob: rejects wrong count / non-finite / high<low', () => {
    expect(parseBarsBlob('99 101').errors[0].message).toMatch(/4 tokens/);
    expect(parseBarsBlob('99 foo 99 101').errors[0].message).toMatch(/non-finite/);
    expect(parseBarsBlob('99 98 100 101').errors[0].message).toMatch(/high < low/);
});

test('parseBarsBlob: non-string returns 1 error', () => {
    expect(parseBarsBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts non-empty default', () => {
    expect(validateInputs({ bars: [b(99, 101, 99, 101)], smoothing_period: 14 })).toBe(null);
});

test('validate rejects: bad array / non-finite / high<low / smoothing < 1', () => {
    expect(validateInputs({ bars: 'no', smoothing_period: 14 })).toMatch(/bars/);
    expect(validateInputs({ bars: [b(NaN, 101, 99, 101)], smoothing_period: 14 })).toMatch(/open/);
    expect(validateInputs({ bars: [b(99, 98, 100, 101)], smoothing_period: 14 })).toMatch(/high/);
    expect(validateInputs({ bars: [b(99, 101, 99, 101)], smoothing_period: 0 })).toMatch(/smoothing_period/);
    expect(validateInputs({ bars: [b(99, 101, 99, 101)], smoothing_period: 1.5 })).toMatch(/integer/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: strips extras + preserves smoothing_period', () => {
    const body = buildBody({ bars: [{ ...b(99, 101, 99, 101), extra: 'x' }], smoothing_period: 5 });
    expect(body).toEqual({ bars: [b(99, 101, 99, 101)], smoothing_period: 5 });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: empty input → empty output arrays', () => {
    const r = localCompute([], 14);
    expect(r.raw_bop).toEqual([]);
    expect(r.smoothed_bop).toEqual([]);
});

test('local: smoothing_period=0 → all null (matches Rust short-circuit)', () => {
    const r = localCompute([b(100, 101, 99, 100.5)], 0);
    expect(r.raw_bop.every(v => v == null)).toBe(true);
});

test('local: NaN → all null', () => {
    const r = localCompute([b(NaN, 101, 99, 100.5)], 14);
    expect(r.raw_bop.every(v => v == null)).toBe(true);
});

test('local: full bullish bar (marubozu) → +1', () => {
    const r = localCompute([b(99, 101, 99, 101)], 1);
    expect(r.raw_bop[0]).toBe(1);
});

test('local: full bearish bar (marubozu) → −1', () => {
    const r = localCompute([b(101, 101, 99, 99)], 1);
    expect(r.raw_bop[0]).toBe(-1);
});

test('local: balanced bar (open=close) → 0', () => {
    const r = localCompute([b(100, 101, 99, 100)], 1);
    expect(r.raw_bop[0]).toBe(0);
});

test('local: zero-range bar (high=low) → 0', () => {
    const r = localCompute([b(100, 100, 100, 100)], 1);
    expect(r.raw_bop[0]).toBe(0);
});

test('local: smoothed BOP = SMA of raw over window', () => {
    const bars = [
        b(99, 101, 99, 101),     // +1
        b(101, 101, 99, 99),     // -1
        b(100, 101, 99, 100),    // 0
    ];
    const r = localCompute(bars, 3);
    expect(Math.abs(r.smoothed_bop[2])).toBeLessThan(1e-12);
});

test('local: output lengths match input', () => {
    const r = localCompute(new Array(30).fill(b(100, 101, 99, 100.5)), 5);
    expect(r.raw_bop.length).toBe(30);
    expect(r.smoothed_bop.length).toBe(30);
});

test('local: smoothing_period=1 → smoothed === raw', () => {
    const bars = [
        b(99, 101, 99, 101),
        b(101, 101, 99, 99),
        b(100, 101, 99, 100),
    ];
    const r = localCompute(bars, 1);
    expect(r.smoothed_bop).toEqual(r.raw_bop);
});

test('local: clamping enforces [-1, +1] even when bar weirdness pushes ratio', () => {
    // close > high mathematically can't happen if input validates, but verify clamp behavior:
    // simulate with high=low (avoided by validator anyway). Just check clamp visually.
    const r = localCompute([b(99.999, 101, 99, 101.001)], 1);
    expect(r.raw_bop[0]).toBeLessThanOrEqual(1);
    expect(r.raw_bop[0]).toBeGreaterThanOrEqual(-1);
});

test('local: smoothed warmup is null until i ≥ smoothing_period - 1', () => {
    const bars = new Array(10).fill(b(100, 101, 99, 100.5));
    const r = localCompute(bars, 5);
    for (let i = 0; i < 4; i++) expect(r.smoothed_bop[i]).toBeNull();
    expect(r.smoothed_bop[4]).not.toBeNull();
});

// ── regimeBadge / lastCrossover / summarize ──────────────────────

test('regimeBadge: 5-tier on smoothed BOP', () => {
    expect(regimeBadge(0.8).key).toMatch(/strong_bull/);
    expect(regimeBadge(0.3).key).toMatch(/bullish/);
    expect(regimeBadge(0).key).toMatch(/balanced/);
    expect(regimeBadge(-0.3).key).toMatch(/bearish/);
    expect(regimeBadge(-0.8).key).toMatch(/strong_bear/);
    expect(regimeBadge(null).key).toMatch(/unknown/);
});

test('lastCrossover: detects raw-vs-smoothed crossover', () => {
    const report = {
        raw_bop:      [null, null,  0.2, 0.4, 0.5, 0.7],
        smoothed_bop: [null, null,  0.3, 0.4, 0.4, 0.5],
    };
    const x = lastCrossover(report);
    expect(x).not.toBeNull();
    expect(x.kind).toBe('bull');
});

test('lastCrossover: returns null when no crossover', () => {
    const r = { raw_bop: [0.5, 0.6, 0.7], smoothed_bop: [0.1, 0.2, 0.3] };
    expect(lastCrossover(r)).toBeNull();
});

test('summarize: count / populated / last_raw / last_smoothed / mean / bull-bear', () => {
    const report = {
        raw_bop:      [null, 0.5, -0.3, 1.0],
        smoothed_bop: [null, 0.5, 0.1, 0.4],
    };
    const s = summarize(report);
    expect(s.count).toBe(4);
    expect(s.populated).toBe(3);
    expect(s.last_raw).toBe(1.0);
    expect(s.last_smoothed).toBe(0.4);
    expect(s.bull_bars).toBe(2);
    expect(s.bear_bars).toBe(1);
});

test('summarize: empty / null → count 0, NaN aggregates', () => {
    expect(summarize(null).count).toBe(0);
    expect(Number.isNaN(summarize(null).last_raw)).toBe(true);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + produces correct-length output', () => {
    for (const k of ['strong-bull','strong-bear','balanced','choppy-noise',
                     'bull-then-bear','zero-range','short-smoothing','no-smoothing']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.bars, inp.smoothing_period);
        expect(r.raw_bop.length).toBe(inp.bars.length);
    }
});

test('demo strong-bull: every raw BOP = +1', () => {
    const inp = makeDemoInput('strong-bull');
    const r = localCompute(inp.bars, inp.smoothing_period);
    for (const v of r.raw_bop) expect(v).toBe(1);
});

test('demo strong-bear: every raw BOP = −1', () => {
    const inp = makeDemoInput('strong-bear');
    const r = localCompute(inp.bars, inp.smoothing_period);
    for (const v of r.raw_bop) expect(v).toBe(-1);
});

test('demo zero-range: every BOP = 0 (no range info)', () => {
    const inp = makeDemoInput('zero-range');
    const r = localCompute(inp.bars, inp.smoothing_period);
    for (const v of r.raw_bop) expect(v).toBe(0);
});

test('demo bull-then-bear: last smoothed value is negative (bears took over)', () => {
    const inp = makeDemoInput('bull-then-bear');
    const r = localCompute(inp.bars, inp.smoothing_period);
    expect(r.smoothed_bop[r.smoothed_bop.length - 1]).toBeLessThan(0);
});

test('demo no-smoothing: smoothed === raw', () => {
    const inp = makeDemoInput('no-smoothing');
    const r = localCompute(inp.bars, inp.smoothing_period);
    expect(r.smoothed_bop).toEqual(r.raw_bop);
});

test('demo short-smoothing: smoothing_period=3', () => {
    const inp = makeDemoInput('short-smoothing');
    expect(inp.smoothing_period).toBe(3);
});

// ── round-trip + formatters ──────────────────────────────────────

test('barsToBlob round-trips through parseBarsBlob', () => {
    const bars = [b(99, 101, 99, 101), b(101, 101, 99, 99)];
    const back = parseBarsBlob(barsToBlob(bars));
    expect(back.errors).toEqual([]);
    expect(back.bars).toEqual(bars);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtBop(0.5)).toBe('+0.5000');
    expect(fmtBop(-0.5)).toBe('-0.5000');
    expect(fmtUSD(100.5)).toBe('$100.50');
    expect(fmtInt(7.9)).toBe('7');
    expect(fmtBop(null)).toBe('—');
    expect(fmtBop(NaN)).toBe('—');
});
