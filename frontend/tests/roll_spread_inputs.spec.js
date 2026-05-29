// Roll-spread helpers: parser, validator, body shape,
// localCompute Rust-mirror, summarize, badges, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_WINDOW, DEFAULT_INPUTS,
    parsePricesBlob, pricesToBlob, validateInputs, buildBody, localCompute,
    summarize, liquidityBadge, regimeBadge, spreadToBps,
    makeDemoInput,
    fmtUSD, fmtBps, fmtNum, fmtInt, fmtPct,
} from '../js/_roll_spread_inputs.js';

// ── constants ─────────────────────────────────────────────────────

test('DEFAULT_WINDOW = 50 (matches Rust default)', () => {
    expect(DEFAULT_WINDOW).toBe(50);
});

// ── parser ────────────────────────────────────────────────────────

test('parsePricesBlob: comma + whitespace + # comments', () => {
    const r = parsePricesBlob('100.05, 99.95\n# bid/ask\n100.05  99.95');
    expect(r.errors).toEqual([]);
    expect(r.prices).toEqual([100.05, 99.95, 100.05, 99.95]);
});

test('parsePricesBlob: rejects non-finite tokens', () => {
    expect(parsePricesBlob('100, foo, 99').errors[0].message).toMatch(/foo/);
});

test('parsePricesBlob: non-string returns 1 error', () => {
    expect(parsePricesBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts default + window=3 minimum', () => {
    expect(validateInputs({ prices: [100, 100.01, 100, 100.01], window: 3 })).toBe(null);
});

test('validate rejects: bad array / window < 3 / non-integer window', () => {
    expect(validateInputs({ ...DEFAULT_INPUTS, prices: 'no', window: 50 })).toMatch(/prices/);
    expect(validateInputs({ ...DEFAULT_INPUTS, window: 2 })).toMatch(/window/);
    expect(validateInputs({ ...DEFAULT_INPUTS, window: 5.5 })).toMatch(/integer/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: forwards prices + window verbatim', () => {
    const b = buildBody({ prices: [100, 101], window: 10 });
    expect(b).toEqual({ prices: [100, 101], window: 10 });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: empty returns empty', () => {
    expect(localCompute([], 20)).toEqual([]);
});

test('local: window too small → all-null (0, 1, 2)', () => {
    const p = Array(30).fill(100);
    expect(localCompute(p, 0).every(v => v == null)).toBe(true);
    expect(localCompute(p, 1).every(v => v == null)).toBe(true);
    expect(localCompute(p, 2).every(v => v == null)).toBe(true);
});

test('local: window larger than input → all-null', () => {
    expect(localCompute(Array(10).fill(100), 20).every(v => v == null)).toBe(true);
});

test('local: trending market (monotonic) → spread = 0 at the tail', () => {
    const p = Array.from({ length: 200 }, (_, i) => 100 + i);
    const out = localCompute(p, 50);
    expect(out[199]).toBe(0);
});

test('local: flat market → spread = 0 at the tail', () => {
    const out = localCompute(Array(200).fill(100), 50);
    expect(out[199]).toBe(0);
});

test('local: pure alternating bid/ask (deterministic) recovers 2×spread (Rust note matches)', () => {
    // Spread 0.10 → pure alternation: cov = −0.01 → est = 2·√0.01 = 0.20 = 2×spread.
    const bid = 99.95, ask = 100.05;
    const p = [];
    for (let i = 0; i < 500; i++) p.push(i % 2 === 0 ? bid : ask);
    const out = localCompute(p, 100);
    // Per the Rust comment: pure alternation violates Roll's 50/50 assumption.
    // Finite-window mean adjustment leaves a ~1e-5 residual from exactly 0.20.
    expect(out[499]).toBeCloseTo(0.20, 4);
});

test('local: NaN prices skipped — still populated at the tail', () => {
    const p = Array.from({ length: 200 }, (_, i) => 100 + (i % 2) * 0.10);
    p[100] = NaN;
    const out = localCompute(p, 50);
    expect(out[199]).not.toBeNull();
});

test('local: warmup region (0..window-2) is null', () => {
    const out = localCompute(Array.from({ length: 60 }, (_, i) => 100 + (i % 2) * 0.10), 30);
    for (let i = 0; i < 29; i++) expect(out[i]).toBeNull();
});

test('local: output length = input length', () => {
    expect(localCompute(Array(50).fill(100), 30).length).toBe(50);
});

test('local: positive serial covariance returns spread=0 (strictly monotonic Δp matches Rust trending test)', () => {
    // Strictly monotonic price → constant +1 Δp → cov(Δp,Δp_prev) = 0 (zero variance).
    // Note: Roll's estimator clamps to 0 when cov ≥ 0, so the constant-Δp case
    // collapses to spread = 0 (mirrors Rust `trending_market_yields_zero_spread`).
    const p = Array.from({ length: 100 }, (_, i) => 100 + i);
    const out = localCompute(p, 50);
    expect(out[99]).toBe(0);
});

// ── summarize ─────────────────────────────────────────────────────

test('summarize: count/mean/min/max/last/zero_count over non-null portion', () => {
    const s = summarize([null, null, 0.10, 0.20, 0]);
    expect(s.count).toBe(3);
    expect(s.mean).toBeCloseTo(0.10, 9);
    expect(s.min).toBe(0);
    expect(s.max).toBe(0.20);
    expect(s.last).toBe(0);
    expect(s.zero_count).toBe(1);
});

test('summarize: all-null → count=0, NaN aggregates', () => {
    const s = summarize([null, null, null]);
    expect(s.count).toBe(0);
    expect(Number.isNaN(s.mean)).toBe(true);
});

// ── badges ────────────────────────────────────────────────────────

test('liquidityBadge: trending (spread=0), tight (<1 bp), normal (<5 bp), wide (<20 bp), extreme', () => {
    expect(liquidityBadge(0,    100).key).toMatch(/trending/);
    expect(liquidityBadge(0.005, 100).key).toMatch(/tight/);
    expect(liquidityBadge(0.04,  100).key).toMatch(/normal/);
    expect(liquidityBadge(0.10,  100).key).toMatch(/wide/);
    expect(liquidityBadge(1.00,  100).key).toMatch(/extreme/);
    expect(liquidityBadge(null,  100).key).toMatch(/unknown/);
    expect(liquidityBadge(0.05,  NaN).key).toMatch(/unknown/);
});

test('regimeBadge: high zero-rate = directional; low zero-rate = random_walk', () => {
    expect(regimeBadge([0, 0, 0, 0, 0, 0, 0, 0.05, 0.05, 0.05]).key).toMatch(/directional/);
    expect(regimeBadge([0.05, 0.10, 0.07, 0.06, 0.08]).key).toMatch(/random_walk/);
    expect(regimeBadge([0, 0.05, 0.10, 0, 0.08]).key).toMatch(/mixed/);
    expect(regimeBadge([]).key).toMatch(/unknown/);
});

// ── spreadToBps ──────────────────────────────────────────────────

test('spreadToBps: (spread / price) × 10_000', () => {
    expect(spreadToBps(0.10, 100)).toBeCloseTo(10, 9);
    expect(spreadToBps(0.05, 100)).toBeCloseTo(5, 9);
    expect(Number.isNaN(spreadToBps(0.10, 0))).toBe(true);
    expect(Number.isNaN(spreadToBps(NaN, 100))).toBe(true);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + produces a series of correct length', () => {
    for (const k of ['random-bounce','trending','flat','tight-bounce','wide-bounce',
                     'regime-shift','spotty-nan','huge-window']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const out = localCompute(inp.prices, inp.window);
        expect(out.length).toBe(inp.prices.length);
    }
});

test('demo trending: last spread = 0', () => {
    const inp = makeDemoInput('trending');
    const out = localCompute(inp.prices, inp.window);
    expect(out[out.length - 1]).toBe(0);
});

test('demo flat: last spread = 0', () => {
    const inp = makeDemoInput('flat');
    const out = localCompute(inp.prices, inp.window);
    expect(out[out.length - 1]).toBe(0);
});

test('demo random-bounce: last spread is positive and finite', () => {
    const inp = makeDemoInput('random-bounce');
    const out = localCompute(inp.prices, inp.window);
    const last = out[out.length - 1];
    expect(Number.isFinite(last)).toBe(true);
    expect(last).toBeGreaterThan(0);
});

test('demo wide-bounce: last spread > tight-bounce last spread', () => {
    const w = makeDemoInput('wide-bounce');
    const t = makeDemoInput('tight-bounce');
    const ws = localCompute(w.prices, w.window);
    const ts = localCompute(t.prices, t.window);
    expect(ws[ws.length - 1]).toBeGreaterThan(ts[ts.length - 1]);
});

test('demo huge-window: all-null (window > n)', () => {
    const inp = makeDemoInput('huge-window');
    const out = localCompute(inp.prices, inp.window);
    expect(out.every(v => v == null)).toBe(true);
});

test('demo spotty-nan: tail bar is still populated', () => {
    const inp = makeDemoInput('spotty-nan');
    const out = localCompute(inp.prices, inp.window);
    expect(out[out.length - 1]).not.toBeNull();
});

// ── round-trip + formatters ───────────────────────────────────────

test('pricesToBlob round-trips through parsePricesBlob', () => {
    const prices = [100.05, 99.95, 100.05, 99.95];
    const back = parsePricesBlob(pricesToBlob(prices));
    expect(back.errors).toEqual([]);
    expect(back.prices).toEqual(prices);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtUSD(0.1234)).toBe('$0.1234');
    expect(fmtBps(10.5)).toBe('10.50 bps');
    expect(fmtNum(0.123456, 4)).toBe('0.1235');
    expect(fmtInt(7.9)).toBe('7');
    expect(fmtPct(0.05)).toBe('5.00%');
    expect(fmtUSD(NaN)).toBe('—');
    expect(fmtNum(null)).toBe('—');
});
