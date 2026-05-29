// Bollinger Squeeze helpers: parser, validator, body shape,
// localCompute Rust-mirror, badges, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_BB_PERIOD, DEFAULT_N_STDEV, DEFAULT_LOOKBACK, DEFAULT_SLACK,
    parseClosesBlob, closesToBlob, validateInputs, buildBody, localCompute,
    squeezeBadge, summarize, lastSqueezeIndex,
    makeDemoInput,
    fmtWidth, fmtNum, fmtInt, fmtUSD,
} from '../js/_bollinger_squeeze_inputs.js';

// ── constants ─────────────────────────────────────────────────────

test('Bollinger defaults match Rust', () => {
    expect(DEFAULT_BB_PERIOD).toBe(20);
    expect(DEFAULT_N_STDEV).toBe(2.0);
    expect(DEFAULT_LOOKBACK).toBe(125);
    expect(DEFAULT_SLACK).toBe(0.05);
});

// ── parser ────────────────────────────────────────────────────────

test('parseClosesBlob: comma + whitespace + comments', () => {
    const r = parseClosesBlob('100.0, 100.1\n# noise\n99.9  100.05');
    expect(r.errors).toEqual([]);
    expect(r.closes).toEqual([100.0, 100.1, 99.9, 100.05]);
});

test('parseClosesBlob: rejects non-finite', () => {
    expect(parseClosesBlob('100, foo').errors[0].message).toMatch(/foo/);
});

test('parseClosesBlob: non-string returns 1 error', () => {
    expect(parseClosesBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts non-empty default', () => {
    const closes = new Array(125).fill(100);
    expect(validateInputs({ closes, bb_period: 20, n_stdev: 2, lookback: 125, slack: 0.05 })).toBe(null);
});

test('validate rejects: bad array / period < 2 / lookback < period / n_stdev ≤ 0 / negative slack / short series', () => {
    const closes = new Array(125).fill(100);
    expect(validateInputs({ closes: 'no', bb_period: 20, n_stdev: 2, lookback: 125, slack: 0.05 })).toMatch(/closes/);
    expect(validateInputs({ closes, bb_period: 1, n_stdev: 2, lookback: 125, slack: 0.05 })).toMatch(/bb_period/);
    expect(validateInputs({ closes, bb_period: 20, n_stdev: 0, lookback: 125, slack: 0.05 })).toMatch(/n_stdev/);
    expect(validateInputs({ closes, bb_period: 20, n_stdev: 2, lookback: 10, slack: 0.05 })).toMatch(/lookback/);
    expect(validateInputs({ closes, bb_period: 20, n_stdev: 2, lookback: 125, slack: -0.1 })).toMatch(/slack/);
    expect(validateInputs({ closes: [1, 2, 3], bb_period: 20, n_stdev: 2, lookback: 125, slack: 0.05 })).toMatch(/observations/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: forwards all 5 fields verbatim', () => {
    const body = buildBody({ closes: [100], bb_period: 20, n_stdev: 2, lookback: 125, slack: 0.05 });
    expect(body).toEqual({ closes: [100], bb_period: 20, n_stdev: 2, lookback: 125, slack: 0.05 });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: invalid inputs return all-null', () => {
    const c = new Array(200).fill(100);
    const r = localCompute(c, 1, 2.0, 125, 0.05);
    expect(r.width_pct.every(v => v == null)).toBe(true);
    const r2 = localCompute(c, 20, 0.0, 125, 0.05);
    expect(r2.width_pct.every(v => v == null)).toBe(true);
    const r3 = localCompute(c, 20, 2.0, 10, 0.05);
    expect(r3.width_pct.every(v => v == null)).toBe(true);
});

test('local: NaN in closes → all-null', () => {
    const c = new Array(200).fill(100);
    c[5] = NaN;
    const r = localCompute(c, 20, 2.0, 125, 0.05);
    expect(r.width_pct.every(v => v == null)).toBe(true);
});

test('local: flat market → perpetual squeeze (width=0)', () => {
    const c = new Array(200).fill(100);
    const r = localCompute(c, 20, 2.0, 125, 0.05);
    // squeeze_on requires full lookback window of populated widths.
    // First populated squeeze: i = bb_period + lookback - 2 = 143.
    for (let i = 143; i < 200; i++) {
        expect(r.squeeze_on[i]).toBe(true);
        expect(r.width_pct[i]).toBeCloseTo(0, 9);
    }
});

test('local: post-volatility-surge squeeze should turn OFF for some bars', () => {
    let state = 42n;
    const c = new Array(130).fill(100);
    for (let i = 0; i < 70; i++) {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        const u = Number(state >> 32n) / 0xFFFFFFFF;
        c.push(100 + (u - 0.5) * 50);
    }
    const r = localCompute(c, 20, 2.0, 125, 0.05);
    let anyOff = false;
    for (let i = 150; i < c.length; i++) if (r.squeeze_on[i] === false) anyOff = true;
    expect(anyOff).toBe(true);
});

test('local: output lengths match input', () => {
    const r = localCompute(new Array(200).fill(100), 20, 2.0, 125, 0.05);
    expect(r.width_pct.length).toBe(200);
    expect(r.squeeze_on.length).toBe(200);
});

test('local: warmup region for width is null (i < bb_period - 1)', () => {
    const c = new Array(200).fill(100);
    const r = localCompute(c, 20, 2.0, 125, 0.05);
    for (let i = 0; i < 19; i++) expect(r.width_pct[i]).toBeNull();
    expect(r.width_pct[19]).not.toBeNull();
});

test('local: warmup region for squeeze covers (bb_period + lookback − 2) bars total', () => {
    const c = new Array(200).fill(100);
    const r = localCompute(c, 20, 2.0, 125, 0.05);
    // Window at i=124 still includes null widths from the bb_period warmup at indices 0..18.
    // First fully-populated squeeze appears at i = 20 + 125 − 2 = 143.
    for (let i = 0; i < 143; i++) expect(r.squeeze_on[i]).toBeNull();
    expect(r.squeeze_on[143]).not.toBeNull();
});

test('local: width formula = 2·n_stdev·σ/|mean|·100', () => {
    // Engineer closes with σ that we know exactly.
    const c = [];
    for (let i = 0; i < 200; i++) c.push(100 + (i % 2 === 0 ? 0.5 : -0.5));
    const r = localCompute(c, 20, 2.0, 125, 0.05);
    // sample mean ≈ 100; population stdev of ±0.5 alternation = 0.5
    // width ≈ 2 · 2 · 0.5 / 100 · 100 = 2.0%
    expect(r.width_pct[199]).toBeCloseTo(2.0, 6);
});

test('local: slack=0 ⇒ strict equality (squeeze_on only if current = window min)', () => {
    // Flat 145 bars (widths 0 for 19..144) + 1 jitter bar. Squeeze window for i=145
    // is [21..145] — all widths fully populated; min=0; current>0 → squeeze OFF.
    const c = new Array(145).fill(100);
    c.push(100.5);
    const r = localCompute(c, 20, 2.0, 125, 0);
    expect(r.squeeze_on[145]).toBe(false);
});

// ── squeezeBadge / summarize / lastSqueezeIndex ──────────────────

test('squeezeBadge: ON tight tier (width<1 → coiled)', () => {
    expect(squeezeBadge([true], [0.5]).key).toMatch(/coiled/);
    expect(squeezeBadge([true], [2]).key).toMatch(/tight/);
    expect(squeezeBadge([true], [10]).key).toMatch(/squeeze/);
});

test('squeezeBadge: OFF expansion tier (width>20 → expansion); normal otherwise', () => {
    expect(squeezeBadge([false], [25]).key).toMatch(/expansion/);
    expect(squeezeBadge([false], [10]).key).toMatch(/normal/);
});

test('squeezeBadge: unknown when array is empty / all-null', () => {
    expect(squeezeBadge([], []).key).toMatch(/unknown/);
    expect(squeezeBadge([null, null], [null, null]).key).toMatch(/unknown/);
});

test('summarize: count / populated / squeeze_count / last_width / last_state', () => {
    const report = {
        width_pct:  [null, null, 2.5, 1.2, 0.8],
        squeeze_on: [null, null, false, true, true],
    };
    const s = summarize(report);
    expect(s.count).toBe(5);
    expect(s.populated).toBe(3);
    expect(s.squeeze_count).toBe(2);
    expect(s.last_width).toBe(0.8);
    expect(s.last_state).toBe(true);
});

test('summarize: empty / null → count 0, NaN extrema', () => {
    expect(summarize(null).count).toBe(0);
    expect(Number.isNaN(summarize(null).last_width)).toBe(true);
});

test('lastSqueezeIndex: returns last ON index', () => {
    expect(lastSqueezeIndex([false, true, false, true, false])).toBe(3);
    expect(lastSqueezeIndex([false, false])).toBeNull();
    expect(lastSqueezeIndex(null)).toBeNull();
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + computes input-length output', () => {
    for (const k of ['flat-perpetual','expansion-after-quiet','coiling','noisy-walk',
                     'short-lookback','tight-slack','loose-slack','wide-bands']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.closes, inp.bb_period, inp.n_stdev, inp.lookback, inp.slack);
        expect(r.width_pct.length).toBe(inp.closes.length);
    }
});

test('demo flat-perpetual: every bar past warmup is squeeze ON', () => {
    const inp = makeDemoInput('flat-perpetual');
    const r = localCompute(inp.closes, inp.bb_period, inp.n_stdev, inp.lookback, inp.slack);
    const firstActive = inp.bb_period + inp.lookback - 2;
    for (let i = firstActive; i < inp.closes.length; i++) {
        expect(r.squeeze_on[i]).toBe(true);
    }
});

test('demo coiling: at least one squeeze ON in the quiet tail', () => {
    const inp = makeDemoInput('coiling');
    const r = localCompute(inp.closes, inp.bb_period, inp.n_stdev, inp.lookback, inp.slack);
    let anyOn = false;
    for (let i = 150; i < inp.closes.length; i++) if (r.squeeze_on[i] === true) anyOn = true;
    expect(anyOn).toBe(true);
});

test('demo expansion-after-quiet: at least one squeeze OFF in noisy tail', () => {
    const inp = makeDemoInput('expansion-after-quiet');
    const r = localCompute(inp.closes, inp.bb_period, inp.n_stdev, inp.lookback, inp.slack);
    let anyOff = false;
    for (let i = 160; i < inp.closes.length; i++) if (r.squeeze_on[i] === false) anyOff = true;
    expect(anyOff).toBe(true);
});

test('demo short-lookback: faster squeeze detection (works with lookback=40)', () => {
    const inp = makeDemoInput('short-lookback');
    expect(inp.lookback).toBe(40);
    const r = localCompute(inp.closes, inp.bb_period, inp.n_stdev, inp.lookback, inp.slack);
    expect(r.squeeze_on[r.squeeze_on.length - 1]).not.toBeNull();
});

// ── round-trip + formatters ──────────────────────────────────────

test('closesToBlob round-trips through parseClosesBlob', () => {
    const c = [100.0, 100.1, 99.9, 100.05];
    const back = parseClosesBlob(closesToBlob(c));
    expect(back.errors).toEqual([]);
    expect(back.closes).toEqual(c);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtWidth(0.5)).toBe('0.5000%');
    expect(fmtNum(1.234, 1)).toBe('1.2');
    expect(fmtInt(7.9)).toBe('7');
    expect(fmtUSD(100.5)).toBe('$100.50');
    expect(fmtWidth(null)).toBe('—');
});
