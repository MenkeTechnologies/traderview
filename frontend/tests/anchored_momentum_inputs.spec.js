// Anchored Momentum helpers: parser, validator, body shape,
// localCompute Rust-mirror (raw ROC + WMA smoothing), badges, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_SMOOTH, DEFAULT_INPUTS,
    parseClosesBlob, closesToBlob, validateInputs, buildBody, localCompute,
    momentumBadge, summarize,
    makeDemoInput,
    fmtPctSigned, fmtUSD, fmtInt,
} from '../js/_anchored_momentum_inputs.js';

// ── constants ─────────────────────────────────────────────────────

test('DEFAULT_SMOOTH = 5 (matches Rust default)', () => {
    expect(DEFAULT_SMOOTH).toBe(5);
});

// ── parser ────────────────────────────────────────────────────────

test('parseClosesBlob: whitespace + commas + NaN-string tolerated', () => {
    const r = parseClosesBlob('100.0, 100.5\n# halt\nNaN  101.2');
    expect(r.errors).toEqual([]);
    expect(r.closes.length).toBe(4);
    expect(r.closes[0]).toBe(100.0);
    expect(Number.isNaN(r.closes[2])).toBe(true);
});

test('parseClosesBlob: rejects garbage tokens', () => {
    expect(parseClosesBlob('100, foo').errors[0].message).toMatch(/foo/);
});

test('parseClosesBlob: non-string returns 1 error', () => {
    expect(parseClosesBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts default + NaN inside closes', () => {
    expect(validateInputs({ closes: [100, NaN, 102], anchor: 0, smooth_period: 1 })).toBe(null);
});

test('validate rejects: bad array / non-integer anchor / negative / bad smooth / anchor ≥ length', () => {
    expect(validateInputs({ closes: 'no', anchor: 0, smooth_period: 5 })).toMatch(/closes/);
    expect(validateInputs({ closes: [100], anchor: 1.5, smooth_period: 5 })).toMatch(/anchor/);
    expect(validateInputs({ closes: [100], anchor: -1, smooth_period: 5 })).toMatch(/anchor/);
    expect(validateInputs({ closes: [100], anchor: 0, smooth_period: 0 })).toMatch(/smooth_period/);
    expect(validateInputs({ closes: [100, 200], anchor: 5, smooth_period: 1 })).toMatch(/anchor/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: forwards closes + anchor + smooth verbatim', () => {
    expect(buildBody({ closes: [1, 2, 3], anchor: 1, smooth_period: 2 }))
        .toEqual({ closes: [1, 2, 3], anchor: 1, smooth_period: 2 });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: empty → empty', () => {
    expect(localCompute([], 0, 5)).toEqual([]);
});

test('local: anchor out of range → all null', () => {
    expect(localCompute(new Array(10).fill(100), 20, 5).every(v => v == null)).toBe(true);
});

test('local: smooth_period=0 → all null', () => {
    expect(localCompute(new Array(10).fill(100), 0, 0).every(v => v == null)).toBe(true);
});

test('local: zero / NaN anchor close → all null', () => {
    const c = new Array(10).fill(100);
    c[0] = 0;
    expect(localCompute(c, 0, 1).every(v => v == null)).toBe(true);
    c[0] = NaN;
    expect(localCompute(c, 0, 1).every(v => v == null)).toBe(true);
});

test('local: smooth_period=1 returns raw ROC (% from anchor)', () => {
    const out = localCompute([100, 105, 110, 115, 120], 0, 1);
    expect(out[0]).toBeCloseTo(0, 9);
    expect(out[1]).toBeCloseTo(0.05, 9);
    expect(out[4]).toBeCloseTo(0.20, 9);
});

test('local: smooth_period larger than available bars → all null', () => {
    const c = new Array(5).fill(100);
    expect(localCompute(c, 3, 5).every(v => v == null)).toBe(true);
});

test('local: flat series after anchor → 0 smoothed', () => {
    const out = localCompute(new Array(20).fill(100), 5, 3);
    for (const v of out) {
        if (v != null) expect(v).toBe(0);
    }
});

test('local: rising series → positive smoothed last bar', () => {
    const c = Array.from({ length: 20 }, (_, i) => 100 + i);
    const out = localCompute(c, 0, 5);
    expect(out[19]).toBeGreaterThan(0);
});

test('local: falling series → negative smoothed last bar', () => {
    const c = Array.from({ length: 20 }, (_, i) => 100 - i * 0.5);
    const out = localCompute(c, 0, 5);
    expect(out[19]).toBeLessThan(0);
});

test('local: NaN at index 10 blocks 3-bar windows touching it; others populate', () => {
    const c = new Array(20).fill(100);
    c[10] = NaN;
    const out = localCompute(c, 0, 3);
    for (let i = 10; i <= 12; i++) expect(out[i]).toBeNull();
    expect(out[5]).not.toBeNull();
    expect(out[15]).not.toBeNull();
});

test('local: WMA weights = 1..N (sum = N(N+1)/2) verified on rising series', () => {
    // closes 100, 101, 102, 103 (anchor=0, smooth=3 → first WMA at i=2)
    // raw values: 0, 0.01, 0.02, ~0.03
    // WMA at i=2: (0·1 + 0.01·2 + 0.02·3) / (1+2+3) = (0 + 0.02 + 0.06)/6 = 0.0133...
    const c = [100, 101, 102, 103];
    const out = localCompute(c, 0, 3);
    expect(out[2]).toBeCloseTo(0.0133333, 5);
});

test('local: anchor index has raw=0 and (when smooth=1) smoothed=0', () => {
    const out = localCompute([100, 105, 110], 0, 1);
    expect(out[0]).toBe(0);
});

test('local: anchor=mid-series; pre-anchor slots remain null', () => {
    const c = new Array(20).fill(100);
    for (let i = 10; i < 20; i++) c[i] = 100 + (i - 10);
    const out = localCompute(c, 10, 3);
    for (let i = 0; i < 10; i++) expect(out[i]).toBeNull();
    expect(out[12]).not.toBeNull();
});

// ── momentumBadge / summarize ────────────────────────────────────

test('momentumBadge: 5-tier on smoothed momentum', () => {
    expect(momentumBadge(0.30).key).toMatch(/strong_up/);
    expect(momentumBadge(0.10).key).toMatch(/up/);
    expect(momentumBadge(0).key).toMatch(/flat/);
    expect(momentumBadge(-0.10).key).toMatch(/down/);
    expect(momentumBadge(-0.30).key).toMatch(/strong_down/);
    expect(momentumBadge(null).key).toMatch(/unknown/);
});

test('summarize: count / populated / last / mean / min / max', () => {
    const s = summarize([null, null, 0.05, 0.10, 0.15]);
    expect(s.count).toBe(5);
    expect(s.populated).toBe(3);
    expect(s.last).toBe(0.15);
    expect(s.mean).toBeCloseTo(0.10, 9);
    expect(s.min).toBe(0.05);
    expect(s.max).toBe(0.15);
});

test('summarize: empty → count 0, NaN aggregates', () => {
    const s = summarize([]);
    expect(s.count).toBe(0);
    expect(Number.isNaN(s.last)).toBe(true);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + computes input-length output', () => {
    for (const k of ['post-earnings-rally','post-news-crash','flat-after-anchor',
                     'pre-anchor-clipped','raw-only','long-smoothing','with-nan-gap',
                     'fomc-volatile']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.closes, inp.anchor, inp.smooth_period);
        expect(r.length).toBe(inp.closes.length);
    }
});

test('demo post-earnings-rally: last smoothed momentum > +5%', () => {
    const inp = makeDemoInput('post-earnings-rally');
    const r = localCompute(inp.closes, inp.anchor, inp.smooth_period);
    expect(r[r.length - 1]).toBeGreaterThan(0.05);
});

test('demo post-news-crash: last smoothed momentum < −5%', () => {
    const inp = makeDemoInput('post-news-crash');
    const r = localCompute(inp.closes, inp.anchor, inp.smooth_period);
    expect(r[r.length - 1]).toBeLessThan(-0.05);
});

test('demo flat-after-anchor: every non-null bar is 0', () => {
    const inp = makeDemoInput('flat-after-anchor');
    const r = localCompute(inp.closes, inp.anchor, inp.smooth_period);
    for (const v of r) if (v != null) expect(v).toBe(0);
});

test('demo pre-anchor-clipped: bars before anchor are null', () => {
    const inp = makeDemoInput('pre-anchor-clipped');
    const r = localCompute(inp.closes, inp.anchor, inp.smooth_period);
    for (let i = 0; i < inp.anchor; i++) expect(r[i]).toBeNull();
});

test('demo raw-only: smooth_period=1 → output matches raw at anchor index = 0', () => {
    const inp = makeDemoInput('raw-only');
    const r = localCompute(inp.closes, inp.anchor, inp.smooth_period);
    expect(r[0]).toBe(0);
    expect(r[r.length - 1]).toBeGreaterThan(0);
});

test('demo with-nan-gap: NaN at index 10 blocks ≥1 surrounding bar', () => {
    const inp = makeDemoInput('with-nan-gap');
    const r = localCompute(inp.closes, inp.anchor, inp.smooth_period);
    let anyBlocked = false;
    for (let i = 10; i <= 10 + inp.smooth_period - 1; i++) {
        if (r[i] === null) anyBlocked = true;
    }
    expect(anyBlocked).toBe(true);
});

// ── round-trip + formatters ──────────────────────────────────────

test('closesToBlob round-trips through parseClosesBlob (including NaN)', () => {
    const closes = [100, 101, NaN, 102.5];
    const back = parseClosesBlob(closesToBlob(closes));
    expect(back.errors).toEqual([]);
    expect(back.closes.length).toBe(4);
    expect(back.closes[0]).toBe(100);
    expect(Number.isNaN(back.closes[2])).toBe(true);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtPctSigned(0.05)).toBe('+5.00%');
    expect(fmtPctSigned(-0.05)).toBe('-5.00%');
    expect(fmtUSD(100.5)).toBe('$100.50');
    expect(fmtInt(7.9)).toBe('7');
    expect(fmtPctSigned(null)).toBe('—');
    expect(fmtPctSigned(NaN)).toBe('—');
});
