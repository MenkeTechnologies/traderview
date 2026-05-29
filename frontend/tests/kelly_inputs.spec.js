// Kelly helpers: static validator + local mirror + note thresholds,
// dynamic validator + local mirror parity, pnls→static derivation,
// pnl blob parser, size badges, demos, formatters.

import { test, expect } from 'vitest';
import {
    validateStaticInputs, buildStaticBody, localComputeStatic, pnlsToStaticInput,
    validateDynamicInputs, buildDynamicBody, localComputeDynamic, parsePnlBlob,
    sizeBadge, makeDemoPnls,
    fmtPct, fmtNum, fmtUSD, fmtUSDSigned,
} from '../js/_kelly_inputs.js';

// ── static validator + body ───────────────────────────────────────

test('validateStatic accepts in-range', () => {
    expect(validateStaticInputs(0.5, 1)).toBe(null);
});

test('validateStatic rejects bad win_rate / payoff', () => {
    expect(validateStaticInputs(NaN, 1)).toMatch(/win_rate/);
    expect(validateStaticInputs(-0.1, 1)).toMatch(/\[0, 1\]/);
    expect(validateStaticInputs(1.1, 1)).toMatch(/\[0, 1\]/);
    expect(validateStaticInputs(0.5, 0)).toMatch(/payoff/);
    expect(validateStaticInputs(0.5, -1)).toMatch(/payoff/);
});

test('buildStaticBody = { win_rate, payoff_ratio }', () => {
    expect(buildStaticBody(0.6, 2)).toEqual({ win_rate: 0.6, payoff_ratio: 2 });
});

// ── localComputeStatic parity (mirrors kelly::compute) ────────────

test('local static: 50/50 even payoff → full = 0', () => {
    const r = localComputeStatic(0.5, 1);
    expect(r.full_kelly).toBeCloseTo(0, 12);
    expect(r.recommended_f).toBe(0);
});

test('local static: 40% wr × 1:1 → negative edge → recommended = 0', () => {
    const r = localComputeStatic(0.4, 1);
    expect(r.full_kelly).toBeLessThan(0);
    expect(r.recommended_f).toBe(0);
    expect(r.note).toMatch(/No edge/);
});

test('local static: 60% wr × 2:1 → f* = 0.4 exactly', () => {
    const r = localComputeStatic(0.6, 2);
    expect(r.full_kelly).toBeCloseTo(0.4,  9);
    expect(r.half_kelly).toBeCloseTo(0.2,  9);
    expect(r.quarter_kelly).toBeCloseTo(0.1, 9);
    expect(r.recommended_f).toBeCloseTo(0.2, 9);
});

test('local static: payoff = 0 → zeroed + payoff-error note', () => {
    const r = localComputeStatic(0.6, 0);
    expect(r.full_kelly).toBe(0);
    expect(r.note).toMatch(/payoff_ratio/);
});

test('local static: extreme edge (90% × 5:1) → > 50% + warning note', () => {
    const r = localComputeStatic(0.9, 5);
    expect(r.full_kelly).toBeGreaterThan(0.5);
    expect(r.note).toMatch(/very large/);
});

test('local static: win_rate > 1 clamps to 1; p=1 + b=1 → f=1', () => {
    expect(localComputeStatic(1.5, 1).full_kelly).toBeCloseTo(1, 12);
});

test('local static: tiny edge (50.1% × 1:1) → dedicated tiny note', () => {
    const r = localComputeStatic(0.501, 1);
    expect(r.full_kelly).toBeGreaterThan(0);
    expect(r.full_kelly).toBeLessThan(0.01);
    expect(r.note).toMatch(/tiny/);
});

// ── pnls → static input derivation ────────────────────────────────

test('pnlsToStaticInput: 3 wins ($200 avg) + 2 losses ($100 avg) → wr=0.6 payoff=2', () => {
    const d = pnlsToStaticInput([200, 200, 200, -100, -100]);
    expect(d.win_rate).toBeCloseTo(0.6, 9);
    expect(d.payoff_ratio).toBeCloseTo(2, 9);
});

test('pnlsToStaticInput: zero-pnl trades count as scratches, not wins/losses', () => {
    const d = pnlsToStaticInput([100, 0, 0, -100]);
    expect(d.wins).toBe(1);
    expect(d.losses).toBe(1);
    expect(d.scratches).toBe(2);
    expect(d.win_rate).toBeCloseTo(0.5, 9);
});

test('pnlsToStaticInput: empty / no-losses / no-wins → payoff=0', () => {
    expect(pnlsToStaticInput([]).payoff_ratio).toBe(0);
    expect(pnlsToStaticInput([100, 100]).payoff_ratio).toBe(0);
    expect(pnlsToStaticInput([-100, -100]).payoff_ratio).toBe(0);
});

// ── dynamic validator + body ──────────────────────────────────────

test('validateDynamic accepts in-range', () => {
    expect(validateDynamicInputs([1, 2, 3], 2)).toBe(null);
});

test('validateDynamic rejects empty / bad window / window > n', () => {
    expect(validateDynamicInputs([], 2)).toMatch(/≥ 1 trade/);
    expect(validateDynamicInputs([1], 0)).toMatch(/window/);
    expect(validateDynamicInputs([1], 1.5)).toMatch(/window/);
    expect(validateDynamicInputs([1, 2], 5)).toMatch(/exceeds/);
});

test('buildDynamicBody = { trade_pnls, window }', () => {
    expect(buildDynamicBody([1, 2], 2)).toEqual({ trade_pnls: [1, 2], window: 2 });
});

// ── localComputeDynamic parity (mirrors dynamic_kelly::compute) ───

test('local dynamic: empty / window=0 → empty', () => {
    expect(localComputeDynamic([], 10)).toEqual([]);
    expect(localComputeDynamic([1, 2, 3], 0)).toEqual([]);
});

test('local dynamic: pre-warmup indices have kelly_fraction=null', () => {
    const out = localComputeDynamic([100, -50, 100], 5);
    for (const p of out) expect(p.kelly_fraction).toBeNull();
});

test('local dynamic: winning window → kelly ≈ matches static calc', () => {
    // 6 wins of $200, 4 losses of $100 → wr=0.6, payoff=2, kelly=0.4.
    const trades = [...Array(6).fill(200), ...Array(4).fill(-100)];
    const out = localComputeDynamic(trades, 10);
    const last = out[9];
    expect(last.kelly_fraction).toBeCloseTo(0.4, 9);
    expect(last.half_kelly_fraction).toBeCloseTo(0.2, 9);
});

test('local dynamic: break-even window → kelly ≈ 0', () => {
    const out = localComputeDynamic([100, -100, 100, -100], 4);
    expect(out[3].kelly_fraction).toBeCloseTo(0, 9);
});

test('local dynamic: losing window → kelly < 0, half-Kelly clamps to 0', () => {
    const trades = [...Array(3).fill(100), ...Array(7).fill(-100)];
    const out = localComputeDynamic(trades, 10);
    expect(out[9].kelly_fraction).toBeLessThan(0);
    expect(out[9].half_kelly_fraction).toBe(0);
});

test('local dynamic: no losses in window → payoff null → kelly null', () => {
    const out = localComputeDynamic(Array(10).fill(100), 10);
    expect(out[9].window_payoff_ratio).toBeNull();
    expect(out[9].kelly_fraction).toBeNull();
});

test('local dynamic: pure losers → kelly = -1, half-Kelly = 0', () => {
    const out = localComputeDynamic(Array(10).fill(-100), 10);
    expect(out[9].kelly_fraction).toBe(-1);
    expect(out[9].half_kelly_fraction).toBe(0);
});

test('local dynamic: zero-pnl trades count in window but not win/loss', () => {
    const trades = [...Array(5).fill(0), ...Array(5).fill(100)];
    const out = localComputeDynamic(trades, 10);
    expect(out[9].window_win_rate).toBe(0.5);
    expect(out[9].window_payoff_ratio).toBeNull();
});

test('local dynamic: clamps kelly to [-1, 1]', () => {
    const trades = [...Array(9).fill(100), -1]; // 90% wr × 100:1 payoff
    const out = localComputeDynamic(trades, 10);
    expect(out[9].kelly_fraction).toBeGreaterThanOrEqual(-1);
    expect(out[9].kelly_fraction).toBeLessThanOrEqual(1);
});

test('local dynamic: NaN trades silently filtered from win/loss buckets', () => {
    const trades = [...Array(5).fill(100), ...Array(5).fill(NaN)];
    const out = localComputeDynamic(trades, 10);
    expect(out[9].window_win_rate).toBeCloseTo(0.5, 9);
});

// ── parsePnlBlob ──────────────────────────────────────────────────

test('parsePnlBlob: csv / whitespace / newline; strips comments', () => {
    const r = parsePnlBlob('100,-50\n200\n# comment\n-100 75');
    expect(r.errors).toEqual([]);
    expect(r.pnls).toEqual([100, -50, 200, -100, 75]);
});

test('parsePnlBlob: flags non-finite tokens with index', () => {
    const r = parsePnlBlob('100,abc,200');
    expect(r.errors.length).toBe(1);
    expect(r.pnls).toEqual([100, 200]);
});

test('parsePnlBlob: non-string → 1 error', () => {
    expect(parsePnlBlob(null).errors.length).toBe(1);
});

// ── badges + demos ────────────────────────────────────────────────

test('sizeBadge: thresholds map to no_trade / tiny / moderate / aggressive', () => {
    expect(sizeBadge(-0.1).label).toBe('NO TRADE');
    expect(sizeBadge(0.005).label).toBe('TINY');
    expect(sizeBadge(0.20).label).toBe('MODERATE');
    expect(sizeBadge(0.75).label).toBe('AGGRESSIVE');
    expect(sizeBadge(NaN).label).toBe('NO TRADE');
});

test('demo positive-edge: localComputeDynamic latest kelly > 0', () => {
    const trades = makeDemoPnls('positive-edge');
    const out = localComputeDynamic(trades, 10);
    expect(out[out.length - 1].kelly_fraction).toBeGreaterThan(0);
});

test('demo negative-edge: latest kelly < 0', () => {
    const trades = makeDemoPnls('negative-edge');
    const out = localComputeDynamic(trades, 10);
    expect(out[out.length - 1].kelly_fraction).toBeLessThan(0);
});

test('demo regime-switch: kelly transitions from neg to pos territory', () => {
    const trades = makeDemoPnls('regime-switch');
    const out = localComputeDynamic(trades, 10);
    // After warmup, early indices are inside the losing block → kelly should be ≤ 0.
    // Past the switch, kelly should swing positive (or null if all-winners).
    const after = out[out.length - 1].kelly_fraction;
    expect(after == null || after >= 0).toBe(true);
});

// ── formatters ────────────────────────────────────────────────────

test('fmt helpers + non-finite guards', () => {
    expect(fmtPct(0.05, 2)).toBe('5.00%');
    expect(fmtNum(0.1234, 2)).toBe('0.12');
    expect(fmtUSD(1234)).toBe('$1234');
    expect(fmtUSDSigned(100)).toBe('+$100');
    expect(fmtUSDSigned(-100)).toBe('-$100');
    expect(fmtPct(null)).toBe('—');
    expect(fmtNum(Infinity)).toBe('—');
});
