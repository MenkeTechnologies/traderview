// Momentum Crash Protection helpers: parser, validator, body shape,
// localManage Rust-mirror, summarize, cumReturn, maxDrawdown, badges, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS,
    parseReturnsBlob, returnsToBlob, validateInputs, buildBody, localManage,
    summarize, cumReturn, maxDrawdown,
    leverageBadge, crashBadge,
    makeDemoInput,
    fmtPct, fmtPctSigned, fmtLev, fmtNum, fmtInt,
} from '../js/_momentum_crash_inputs.js';

// ── parser ────────────────────────────────────────────────────────

test('parseReturnsBlob: handles raw decimals + pct-suffix + commas + comments', () => {
    const r = parseReturnsBlob('0.01, -0.02\n# midday\n1.5%, -0.5%');
    expect(r.errors).toEqual([]);
    expect(r.returns).toEqual([0.01, -0.02, 0.015, -0.005]);
});

test('parseReturnsBlob: rejects non-finite token', () => {
    expect(parseReturnsBlob('0.01, foo').errors[0].message).toMatch(/foo/);
});

test('parseReturnsBlob: non-string → 1 error', () => {
    expect(parseReturnsBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts a series long enough for max(vol_lb, crash_lb)+1', () => {
    const inp = { ...DEFAULT_INPUTS, momentum_returns: Array(70).fill(0.001) };
    expect(validateInputs(inp)).toBe(null);
});

test('validate rejects: bad array / NaN return / short series / bad scalar params', () => {
    expect(validateInputs({ ...DEFAULT_INPUTS, momentum_returns: 'no' })).toMatch(/array/);
    expect(validateInputs({ ...DEFAULT_INPUTS, momentum_returns: [NaN, 0.001] })).toMatch(/finite/);
    expect(validateInputs({ ...DEFAULT_INPUTS, momentum_returns: Array(10).fill(0.001) })).toMatch(/need/);
    expect(validateInputs({ ...DEFAULT_INPUTS, vol_lookback: 3, momentum_returns: Array(100).fill(0.001) })).toMatch(/vol_lookback/);
    expect(validateInputs({ ...DEFAULT_INPUTS, target_annualized_vol: 0, momentum_returns: Array(100).fill(0.001) })).toMatch(/target/);
    expect(validateInputs({ ...DEFAULT_INPUTS, periods_per_year: 0, momentum_returns: Array(100).fill(0.001) })).toMatch(/periods/);
    expect(validateInputs({ ...DEFAULT_INPUTS, max_leverage: 0, momentum_returns: Array(100).fill(0.001) })).toMatch(/max_leverage/);
    expect(validateInputs({ ...DEFAULT_INPUTS, crash_filter_lookback: 0, momentum_returns: Array(100).fill(0.001) })).toMatch(/crash_filter_lookback/);
    expect(validateInputs({ ...DEFAULT_INPUTS, crash_filter_threshold_pct: NaN, momentum_returns: Array(100).fill(0.001) })).toMatch(/crash_filter_threshold_pct/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: forwards all 7 fields verbatim', () => {
    const body = buildBody({
        momentum_returns: [0.01],
        vol_lookback: 60, target_annualized_vol: 0.15, periods_per_year: 252,
        max_leverage: 4, crash_filter_lookback: 22, crash_filter_threshold_pct: -0.2,
    });
    expect(body.vol_lookback).toBe(60);
    expect(body.crash_filter_threshold_pct).toBe(-0.2);
});

// ── localManage parity (mirrors every Rust #[test]) ──────────────

test('local: invalid params → null (matches Rust)', () => {
    const r = Array(100).fill(0.01);
    expect(localManage(r, 4,  0.15, 252, 4, 22, -0.05)).toBeNull();    // vol_lookback < 5
    expect(localManage(r, 60, 0.0,  252, 4, 22, -0.05)).toBeNull();    // target = 0
    expect(localManage(r, 60, 0.15, 0,   4, 22, -0.05)).toBeNull();    // periods = 0
    expect(localManage(r, 60, 0.15, 252, 0, 22, -0.05)).toBeNull();    // max_lev = 0
    expect(localManage(r, 60, 0.15, 252, 4, 22, NaN)).toBeNull();      // NaN threshold
});

test('local: NaN in returns → null', () => {
    const r = Array(100).fill(0.01);
    r[5] = NaN;
    expect(localManage(r, 60, 0.15, 252, 4, 22, -0.05)).toBeNull();
});

test('local: crash filter zeros leverage after sustained drawdown', () => {
    // 50 calm + 22 of −1% = −20% cumulative.
    const r = [...Array(50).fill(0.001), ...Array(22).fill(-0.01), ...Array(30).fill(0.001)];
    const result = localManage(r, 60, 0.15, 252, 4, 22, -0.10);
    expect(result.crash_filter_active[72]).toBe(true);
    expect(result.leverages[72]).toBe(0);
});

test('local: normal regime yields nonzero mean leverage', () => {
    // Deterministic LCG, mirrors Rust test.
    let state = BigInt(42);
    const r = [];
    for (let i = 0; i < 200; i++) {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        const u = Number(state >> 32n) / 0xFFFFFFFF;
        r.push(0.0005 + (u - 0.5) * 0.01);
    }
    const result = localManage(r, 60, 0.15, 252, 4, 22, -0.20);
    expect(result.mean_leverage).toBeGreaterThan(0);
});

test('local: calm regime is capped at max_leverage', () => {
    const r = Array(100).fill(0.0001);
    const result = localManage(r, 60, 0.15, 252, 4, 22, -0.20);
    let mx = -Infinity;
    for (const l of result.leverages) if (l != null && l > mx) mx = l;
    expect(mx).toBeLessThanOrEqual(4);
});

test('local: managed_returns / leverages / crash array lengths match input', () => {
    const r = Array(100).fill(0.01);
    const result = localManage(r, 60, 0.15, 252, 4, 22, -0.20);
    expect(result.managed_returns.length).toBe(100);
    expect(result.leverages.length).toBe(100);
    expect(result.crash_filter_active.length).toBe(100);
});

test('local: warmup region (i < lookback) → null entries', () => {
    const r = Array(100).fill(0.001);
    const result = localManage(r, 60, 0.15, 252, 4, 22, -0.20);
    for (let i = 0; i < 60; i++) {
        expect(result.leverages[i]).toBeNull();
        expect(result.managed_returns[i]).toBeNull();
        expect(result.crash_filter_active[i]).toBeNull();
    }
});

test('local: managed_return[i] = leverage[i] × momentum_return[i] when populated', () => {
    const r = Array(100).fill(0.001);
    const result = localManage(r, 60, 0.15, 252, 4, 22, -0.20);
    for (let i = 60; i < 100; i++) {
        if (result.leverages[i] != null) {
            expect(result.managed_returns[i]).toBeCloseTo(result.leverages[i] * 0.001, 9);
        }
    }
});

test('local: n_observations = input length', () => {
    const r = Array(100).fill(0.01);
    const result = localManage(r, 60, 0.15, 252, 4, 22, -0.20);
    expect(result.n_observations).toBe(100);
});

// ── summarize ─────────────────────────────────────────────────────

test('summarize: populated / crash_bars / crash_frac / max_lev / total_managed', () => {
    const fake = {
        leverages: [null, null, 1, 2, 0],
        managed_returns: [null, null, 0.01, 0.02, 0],
        crash_filter_active: [null, null, false, false, true],
        n_observations: 5,
    };
    const s = summarize(fake);
    expect(s.populated).toBe(3);
    expect(s.crash_bars).toBe(1);
    expect(s.max_lev).toBe(2);
    // (1+0.01)*(1+0.02)*(1+0) - 1 = 0.0302
    expect(s.total_managed).toBeCloseTo(0.0302, 9);
});

test('summarize: null report → safe zeros', () => {
    const s = summarize(null);
    expect(s.populated).toBe(0);
});

// ── cumReturn + maxDrawdown ──────────────────────────────────────

test('cumReturn: compound product − 1', () => {
    expect(cumReturn([0.1, -0.05])).toBeCloseTo(1.1 * 0.95 - 1, 9);
    expect(cumReturn([])).toBeNaN();
});

test('maxDrawdown: peak-to-trough fraction', () => {
    // +50%, then −33.33% (back to start) → peak 1.5, trough 1.0 → DD = −0.333
    expect(maxDrawdown([0.5, -1 / 3])).toBeCloseTo(-1 / 3, 6);
    expect(maxDrawdown([])).toBeNaN();
});

test('maxDrawdown: monotonic up → 0', () => {
    expect(maxDrawdown([0.01, 0.01, 0.01])).toBe(0);
});

// ── badges ────────────────────────────────────────────────────────

test('leverageBadge: off / defensive / balanced / aggressive / maxed', () => {
    expect(leverageBadge(0,   4).key).toMatch(/off/);
    expect(leverageBadge(0.3, 4).key).toMatch(/defensive/);
    expect(leverageBadge(1.5, 4).key).toMatch(/balanced/);
    expect(leverageBadge(3.0, 4).key).toMatch(/aggressive/);
    expect(leverageBadge(3.8, 4).key).toMatch(/maxed/);
});

test('crashBadge: none / brief / frequent / dominant', () => {
    expect(crashBadge(0).key).toMatch(/none/);
    expect(crashBadge(0.02).key).toMatch(/brief/);
    expect(crashBadge(0.10).key).toMatch(/frequent/);
    expect(crashBadge(0.50).key).toMatch(/dominant/);
    expect(crashBadge(NaN).key).toMatch(/unknown/);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + produces non-null report', () => {
    for (const k of ['normal-regime','low-vol','high-vol','crash-event',
                     'persistent-crash','mixed-regime','short-lookback','tight-target']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localManage(
            inp.momentum_returns, inp.vol_lookback, inp.target_annualized_vol,
            inp.periods_per_year, inp.max_leverage,
            inp.crash_filter_lookback, inp.crash_filter_threshold_pct,
        );
        expect(r).not.toBeNull();
        expect(r.n_observations).toBe(inp.momentum_returns.length);
    }
});

test('demo crash-event: at least one bar with crash filter active', () => {
    const inp = makeDemoInput('crash-event');
    const r = localManage(
        inp.momentum_returns, inp.vol_lookback, inp.target_annualized_vol,
        inp.periods_per_year, inp.max_leverage,
        inp.crash_filter_lookback, inp.crash_filter_threshold_pct,
    );
    expect(r.crash_filter_active.some(c => c === true)).toBe(true);
});

test('demo persistent-crash: leverages drop to 0 mid-series', () => {
    const inp = makeDemoInput('persistent-crash');
    const r = localManage(
        inp.momentum_returns, inp.vol_lookback, inp.target_annualized_vol,
        inp.periods_per_year, inp.max_leverage,
        inp.crash_filter_lookback, inp.crash_filter_threshold_pct,
    );
    expect(r.leverages.some(l => l === 0)).toBe(true);
});

test('demo low-vol: max leverage hits the cap', () => {
    const inp = makeDemoInput('low-vol');
    const r = localManage(
        inp.momentum_returns, inp.vol_lookback, inp.target_annualized_vol,
        inp.periods_per_year, inp.max_leverage,
        inp.crash_filter_lookback, inp.crash_filter_threshold_pct,
    );
    let mx = 0;
    for (const l of r.leverages) if (l != null && l > mx) mx = l;
    expect(mx).toBeCloseTo(inp.max_leverage, 9);
});

// ── round-trip + formatters ───────────────────────────────────────

test('returnsToBlob round-trips through parseReturnsBlob', () => {
    const rs = [0.01, -0.02, 0.005];
    const back = parseReturnsBlob(returnsToBlob(rs));
    expect(back.errors).toEqual([]);
    expect(back.returns).toEqual(rs);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtPct(0.05)).toBe('5.00%');
    expect(fmtPctSigned(0.05)).toBe('+5.00%');
    expect(fmtPctSigned(-0.02)).toBe('-2.00%');
    expect(fmtLev(2.5)).toBe('2.50x');
    expect(fmtNum(1.23456)).toBe('1.234560');
    expect(fmtInt(7.9)).toBe('7');
    expect(fmtPct(NaN)).toBe('—');
    expect(fmtLev(null)).toBe('—');
});
