// Amihud illiquidity helpers: parser, validator, body shape,
// localCompute Rust-mirror, badges, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_PERIOD, parsePairsBlob, pairsToBlob, validateInputs, buildBody, localCompute,
    summarize, liquidityBadge, trendBadge,
    makeDemoInput,
    fmtAmihud, fmtPct, fmtDV, fmtInt,
} from '../js/_amihud_inputs.js';

// ── constants ─────────────────────────────────────────────────────

test('DEFAULT_PERIOD = 21 (matches Rust)', () => {
    expect(DEFAULT_PERIOD).toBe(21);
});

// ── parser ────────────────────────────────────────────────────────

test('parsePairsBlob: 2 tokens per line, pct-suffix returns ok', () => {
    const r = parsePairsBlob('0.012 100000000\n1.5% 90000000');
    expect(r.errors).toEqual([]);
    expect(r.returns).toEqual([0.012, 0.015]);
    expect(r.dollar_volumes).toEqual([100_000_000, 90_000_000]);
});

test('parsePairsBlob: rejects wrong token count', () => {
    expect(parsePairsBlob('0.01').errors[0].message).toMatch(/2 tokens/);
});

test('parsePairsBlob: non-string returns 1 error', () => {
    expect(parsePairsBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts equal-length series + valid period', () => {
    expect(validateInputs({ returns: [0.01], dollar_volumes: [1e6], period: 21 })).toBe(null);
});

test('validate rejects: bad array / length mismatch / bad period', () => {
    expect(validateInputs({ returns: 'no', dollar_volumes: [1], period: 21 })).toMatch(/returns/);
    expect(validateInputs({ returns: [0.01], dollar_volumes: [], period: 21 })).toMatch(/same length/);
    expect(validateInputs({ returns: [0.01], dollar_volumes: [1e6], period: 0 })).toMatch(/period/);
    expect(validateInputs({ returns: [0.01], dollar_volumes: [1e6], period: 1.5 })).toMatch(/integer/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: forwards returns + dv + period verbatim', () => {
    const b = buildBody({ returns: [0.01], dollar_volumes: [1e6], period: 14 });
    expect(b).toEqual({ returns: [0.01], dollar_volumes: [1e6], period: 14 });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: empty returns empty', () => {
    expect(localCompute([], [], 14)).toEqual([]);
});

test('local: length mismatch → all null', () => {
    const r = new Array(30).fill(0.01);
    const v = new Array(15).fill(1e6);
    expect(localCompute(r, v, 14).every(x => x == null)).toBe(true);
});

test('local: period=0 → all null', () => {
    const r = new Array(30).fill(0.01);
    const v = new Array(30).fill(1e6);
    expect(localCompute(r, v, 0).every(x => x == null)).toBe(true);
});

test('local: high volume → low Amihud (|0.01|/100M × 1M = 0.0001)', () => {
    const r = new Array(30).fill(0.01);
    const v = new Array(30).fill(100_000_000);
    const out = localCompute(r, v, 14);
    expect(out[29]).toBeCloseTo(0.0001, 9);
});

test('local: low volume → high Amihud (|0.01|/10k × 1M = 1.0)', () => {
    const r = new Array(30).fill(0.01);
    const v = new Array(30).fill(10_000);
    const out = localCompute(r, v, 14);
    expect(out[29]).toBeCloseTo(1.0, 9);
});

test('local: zero volume bar skipped safely; other bars still populate', () => {
    const r = new Array(30).fill(0.01);
    const v = new Array(30).fill(1e6);
    v[5] = 0;
    const out = localCompute(r, v, 14);
    expect(out[19]).not.toBeNull();
});

test('local: NaN tolerated without panic + tail still populated', () => {
    const r = new Array(30).fill(0.01);
    const v = new Array(30).fill(1e6);
    r[10] = NaN;
    v[15] = NaN;
    const out = localCompute(r, v, 14);
    expect(out[29]).not.toBeNull();
});

test('local: all-zero volume window → null', () => {
    const r = new Array(30).fill(0.01);
    const v = new Array(30).fill(0);
    expect(localCompute(r, v, 14)[29]).toBeNull();
});

test('local: huge period → all null without panic', () => {
    const r = new Array(5).fill(0.01);
    const v = new Array(5).fill(1e6);
    expect(localCompute(r, v, 1000).every(x => x == null)).toBe(true);
});

test('local: warmup region (i < period-1) is null', () => {
    const r = new Array(30).fill(0.01);
    const v = new Array(30).fill(1e6);
    const out = localCompute(r, v, 14);
    for (let i = 0; i < 13; i++) expect(out[i]).toBeNull();
    expect(out[13]).not.toBeNull();
});

test('local: output length = input length', () => {
    expect(localCompute(new Array(50).fill(0.01), new Array(50).fill(1e6), 14).length).toBe(50);
});

// ── liquidityBadge / trendBadge / summarize ──────────────────────

test('liquidityBadge: 5-tier (deep/liquid/normal/thin/illiquid)', () => {
    expect(liquidityBadge(0.0001).key).toMatch(/deep/);
    expect(liquidityBadge(0.005).key).toMatch(/liquid/);
    expect(liquidityBadge(0.05).key).toMatch(/normal/);
    expect(liquidityBadge(0.5).key).toMatch(/thin/);
    expect(liquidityBadge(5).key).toMatch(/illiquid/);
    expect(liquidityBadge(NaN).key).toMatch(/unknown/);
});

test('trendBadge: improving / stable / deteriorating / crashing using half-half split', () => {
    // ratio = meanL/meanE controls band: <0.5 fast / <0.9 improving / <1.1 stable
    // / <2.0 deteriorating / ≥2.0 crashing.
    const improving = [...Array(20).fill(1.0), ...Array(20).fill(0.7)];   // ratio 0.7
    const deteriorating = [...Array(20).fill(0.5), ...Array(20).fill(0.75)]; // ratio 1.5
    const crashing = [...Array(20).fill(0.1), ...Array(20).fill(1.0)];    // ratio 10
    const stable = new Array(40).fill(0.5);
    expect(trendBadge(improving).key).toMatch(/improving/);
    expect(trendBadge(deteriorating).key).toMatch(/deteriorating/);
    expect(trendBadge(crashing).key).toMatch(/crashing/);
    expect(trendBadge(stable).key).toMatch(/stable/);
    expect(trendBadge([]).key).toMatch(/unknown/);
});

test('summarize: count / populated / mean / min / max / last', () => {
    const s = summarize([null, null, 0.1, 0.2, 0.5]);
    expect(s.count).toBe(5);
    expect(s.populated).toBe(3);
    expect(s.mean).toBeCloseTo(0.2667, 4);
    expect(s.min).toBe(0.1);
    expect(s.max).toBe(0.5);
    expect(s.last).toBe(0.5);
});

test('summarize: empty / all-null → count 0, NaN aggregates', () => {
    const s = summarize([]);
    expect(s.count).toBe(0);
    expect(Number.isNaN(s.mean)).toBe(true);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + produces input-length series', () => {
    for (const k of ['large-cap','mid-cap','small-cap','penny-illiquid',
                     'liquidity-shock','recovery','spotty-volume','short-period']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const out = localCompute(inp.returns, inp.dollar_volumes, inp.period);
        expect(out.length).toBe(inp.returns.length);
    }
});

test('demo large-cap: last Amihud < 0.001 (deep liquidity)', () => {
    const inp = makeDemoInput('large-cap');
    const out = localCompute(inp.returns, inp.dollar_volumes, inp.period);
    const last = out[out.length - 1];
    expect(last).toBeLessThan(0.001);
});

test('demo penny-illiquid: last Amihud > 0.1 (thin)', () => {
    const inp = makeDemoInput('penny-illiquid');
    const out = localCompute(inp.returns, inp.dollar_volumes, inp.period);
    const last = out[out.length - 1];
    expect(last).toBeGreaterThan(0.1);
});

test('demo liquidity-shock: later half mean > earlier half mean', () => {
    const inp = makeDemoInput('liquidity-shock');
    const out = localCompute(inp.returns, inp.dollar_volumes, inp.period);
    const valid = out.filter(v => v != null);
    const half = Math.floor(valid.length / 2);
    const earlier = valid.slice(0, half);
    const later = valid.slice(half);
    const meanE = earlier.reduce((s, v) => s + v, 0) / earlier.length;
    const meanL = later.reduce((s, v) => s + v, 0) / later.length;
    expect(meanL).toBeGreaterThan(meanE);
});

test('demo recovery: later half mean < earlier half mean', () => {
    const inp = makeDemoInput('recovery');
    const out = localCompute(inp.returns, inp.dollar_volumes, inp.period);
    const valid = out.filter(v => v != null);
    const half = Math.floor(valid.length / 2);
    const earlier = valid.slice(0, half);
    const later = valid.slice(half);
    const meanE = earlier.reduce((s, v) => s + v, 0) / earlier.length;
    const meanL = later.reduce((s, v) => s + v, 0) / later.length;
    expect(meanL).toBeLessThan(meanE);
});

test('demo spotty-volume: tail still populated (NaN + zero skipped)', () => {
    const inp = makeDemoInput('spotty-volume');
    const out = localCompute(inp.returns, inp.dollar_volumes, inp.period);
    expect(out[out.length - 1]).not.toBeNull();
});

test('demo short-period: works with period=5', () => {
    const inp = makeDemoInput('short-period');
    expect(inp.period).toBe(5);
    const out = localCompute(inp.returns, inp.dollar_volumes, inp.period);
    expect(out[out.length - 1]).not.toBeNull();
});

// ── round-trip + formatters ──────────────────────────────────────

test('pairsToBlob round-trips through parsePairsBlob', () => {
    const r = [0.012, -0.005];
    const v = [100_000_000, 90_000_000];
    const back = parsePairsBlob(pairsToBlob(r, v));
    expect(back.errors).toEqual([]);
    expect(back.returns).toEqual(r);
    expect(back.dollar_volumes).toEqual(v);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtAmihud(0.05)).toBe('0.0500');
    expect(fmtAmihud(1e-5)).toMatch(/e/i);
    expect(fmtPct(0.012)).toBe('1.2000%');
    expect(fmtDV(1_500_000_000)).toBe('$1.50B');
    expect(fmtDV(1_500_000)).toBe('$1.50M');
    expect(fmtDV(15_500)).toBe('$15.50k');
    expect(fmtInt(7.9)).toBe('7');
    expect(fmtAmihud(null)).toBe('—');
    expect(fmtAmihud(NaN)).toBe('—');
});
