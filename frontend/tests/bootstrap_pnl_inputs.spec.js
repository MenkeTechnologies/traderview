// Bootstrap P&L helpers: parser, validator, body shape,
// localBootstrap Rust-mirror with LCG (BigInt), badges, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_RESAMPLES, DEFAULT_SEED, MIN_TRADES, MIN_RESAMPLES,
    parseTradesBlob, tradesToBlob, validateInputs, buildBody, localBootstrap,
    probBadge, ciBadge, summarizeTrades,
    makeDemoInput,
    fmtUSD, fmtUSDSigned, fmtPct, fmtInt,
} from '../js/_bootstrap_pnl_inputs.js';

// ── constants ─────────────────────────────────────────────────────

test('Constants match Rust defaults', () => {
    expect(DEFAULT_RESAMPLES).toBe(5000);
    expect(DEFAULT_SEED).toBe(0n);
    expect(MIN_TRADES).toBe(5);
    expect(MIN_RESAMPLES).toBe(100);
});

// ── parser ────────────────────────────────────────────────────────

test('parseTradesBlob: handles $-prefix, ()-wrapped negatives, comments', () => {
    const r = parseTradesBlob('$50, -$30\n# day 2\n($25)  20');
    expect(r.errors).toEqual([]);
    expect(r.trade_pnls).toEqual([50, -30, -25, 20]);
});

test('parseTradesBlob: rejects garbage', () => {
    expect(parseTradesBlob('50, foo').errors[0].message).toMatch(/foo/);
});

test('parseTradesBlob: non-string returns 1 error', () => {
    expect(parseTradesBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts default-shape input', () => {
    const trades = new Array(10).fill(1);
    expect(validateInputs({ trade_pnls: trades, n_resamples: 1000, seed: 42n })).toBe(null);
});

test('validate rejects: bad array / too short / bad resamples / NaN / bad seed', () => {
    expect(validateInputs({ trade_pnls: 'no', n_resamples: 1000, seed: 42n })).toMatch(/trade_pnls/);
    expect(validateInputs({ trade_pnls: [1, 2, 3, 4], n_resamples: 1000, seed: 42n })).toMatch(/at least 5/);
    expect(validateInputs({ trade_pnls: [1, NaN, 3, 4, 5], n_resamples: 1000, seed: 42n })).toMatch(/finite/);
    expect(validateInputs({ trade_pnls: new Array(10).fill(1), n_resamples: 50, seed: 42n })).toMatch(/n_resamples/);
    expect(validateInputs({ trade_pnls: new Array(10).fill(1), n_resamples: 1.5, seed: 42n })).toMatch(/integer/);
    expect(validateInputs({ trade_pnls: new Array(10).fill(1), n_resamples: 1000, seed: 'no' })).toMatch(/seed/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: forwards arrays + converts bigint seed to Number', () => {
    const body = buildBody({ trade_pnls: [1, 2, 3], n_resamples: 100, seed: 42n });
    expect(body.trade_pnls).toEqual([1, 2, 3]);
    expect(body.n_resamples).toBe(100);
    expect(body.seed).toBe(42);
});

// ── localBootstrap parity (mirrors every Rust #[test]) ───────────

test('local: too short / too few resamples → null', () => {
    expect(localBootstrap(new Array(4).fill(1), 1000, 42n)).toBeNull();
    expect(localBootstrap(new Array(10).fill(1), 50, 42n)).toBeNull();
});

test('local: NaN → null', () => {
    const t = new Array(20).fill(1);
    t[5] = NaN;
    expect(localBootstrap(t, 1000, 42n)).toBeNull();
});

test('local: deterministic for fixed seed', () => {
    const trades = [10, -5, 15, -10, 20, -8, 12, 25];
    const r1 = localBootstrap(trades, 500, 42n);
    const r2 = localBootstrap(trades, 500, 42n);
    expect(r1.mean_total_pnl).toBe(r2.mean_total_pnl);
});

test('local: mean resampled ≈ n × input mean (≤ 15% relative error)', () => {
    const trades = [10, -5, 15, -10, 20, -8, 12, 25];
    const input_mean = trades.reduce((s, v) => s + v, 0) / trades.length;
    const expected = input_mean * trades.length;
    const r = localBootstrap(trades, 5000, 42n);
    const rel = Math.abs(r.mean_total_pnl - expected) / Math.abs(expected);
    expect(rel).toBeLessThan(0.15);
});

test('local: quantiles ordered correctly', () => {
    const trades = [10, -5, 15, -10, 20, -8];
    const r = localBootstrap(trades, 2000, 7n);
    expect(r.pnl_2_5th_percentile).toBeLessThanOrEqual(r.pnl_5th_percentile);
    expect(r.pnl_5th_percentile).toBeLessThanOrEqual(r.median_total_pnl);
    expect(r.median_total_pnl).toBeLessThanOrEqual(r.pnl_95th_percentile);
    expect(r.pnl_95th_percentile).toBeLessThanOrEqual(r.pnl_97_5th_percentile);
});

test('local: all-positive trades → probability_positive = 1', () => {
    const r = localBootstrap([10, 5, 20, 8, 15], 1000, 42n);
    expect(r.probability_positive).toBeCloseTo(1, 12);
});

test('local: all-negative trades → probability_positive = 0', () => {
    const r = localBootstrap([-10, -5, -20, -8, -15], 1000, 42n);
    expect(r.probability_positive).toBeCloseTo(0, 12);
});

test('local: n_resamples + n_trades reported', () => {
    const r = localBootstrap(new Array(25).fill(1), 500, 42n);
    expect(r.n_resamples).toBe(500);
    expect(r.n_trades).toBe(25);
});

// ── badges / summarizeTrades ─────────────────────────────────────

test('probBadge: 6-tier on probability', () => {
    expect(probBadge(0.99).key).toMatch(/almost_certain/);
    expect(probBadge(0.85).key).toMatch(/profitable/);
    expect(probBadge(0.60).key).toMatch(/edge/);
    expect(probBadge(0.50).key).toMatch(/coin_flip/);
    expect(probBadge(0.30).key).toMatch(/unfavorable/);
    expect(probBadge(0.10).key).toMatch(/disaster/);
    expect(probBadge(null).key).toMatch(/unknown/);
});

test('ciBadge: tight / moderate / wide / extreme by width/|mean|', () => {
    const mk = (lo, hi, mean) => ({
        pnl_97_5th_percentile: hi, pnl_2_5th_percentile: lo, mean_total_pnl: mean,
    });
    expect(ciBadge(mk(800, 1200, 1000)).key).toMatch(/tight/);
    expect(ciBadge(mk(500, 1500, 1000)).key).toMatch(/moderate/);
    // width/|mean| in [3, 10) → wide. Using width=6000, mean=1000 → ratio 6.
    expect(ciBadge(mk(-1500, 4500, 1000)).key).toMatch(/wide/);
    // width/|mean| ≥ 10 → extreme. width=12000, mean=1000 → ratio 12.
    expect(ciBadge(mk(-4000, 8000, 1000)).key).toMatch(/extreme/);
    expect(ciBadge(null).key).toMatch(/unknown/);
});

test('summarizeTrades: count/sum/mean/win-loss/win_rate/max', () => {
    const s = summarizeTrades([10, -5, 20, -8, 15]);
    expect(s.count).toBe(5);
    expect(s.sum).toBe(32);
    expect(s.mean).toBeCloseTo(6.4, 9);
    expect(s.wins).toBe(3);
    expect(s.losses).toBe(2);
    expect(s.win_rate).toBeCloseTo(0.6, 9);
    expect(s.max_win).toBe(20);
    expect(s.max_loss).toBe(-8);
});

test('summarizeTrades: empty → count 0, NaN aggregates', () => {
    const s = summarizeTrades([]);
    expect(s.count).toBe(0);
    expect(Number.isNaN(s.mean)).toBe(true);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + bootstraps to non-null report', () => {
    for (const k of ['winning-strategy','losing-strategy','high-variance','low-variance',
                     'all-winners','all-losers','lumpy-tail','few-trades']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localBootstrap(inp.trade_pnls, inp.n_resamples, inp.seed);
        expect(r).not.toBeNull();
    }
});

test('demo winning-strategy: probability_positive ≥ 0.95', () => {
    const inp = makeDemoInput('winning-strategy');
    const r = localBootstrap(inp.trade_pnls, inp.n_resamples, inp.seed);
    expect(r.probability_positive).toBeGreaterThanOrEqual(0.95);
});

test('demo losing-strategy: probability_positive ≤ 0.25', () => {
    const inp = makeDemoInput('losing-strategy');
    const r = localBootstrap(inp.trade_pnls, inp.n_resamples, inp.seed);
    expect(r.probability_positive).toBeLessThanOrEqual(0.25);
});

test('demo all-winners: prob_pos = 1', () => {
    const inp = makeDemoInput('all-winners');
    const r = localBootstrap(inp.trade_pnls, inp.n_resamples, inp.seed);
    expect(r.probability_positive).toBeCloseTo(1, 12);
});

test('demo all-losers: prob_pos = 0', () => {
    const inp = makeDemoInput('all-losers');
    const r = localBootstrap(inp.trade_pnls, inp.n_resamples, inp.seed);
    expect(r.probability_positive).toBeCloseTo(0, 12);
});

test('demo lumpy-tail: 95% CI spans both signs (catastrophic tail)', () => {
    const inp = makeDemoInput('lumpy-tail');
    const r = localBootstrap(inp.trade_pnls, inp.n_resamples, inp.seed);
    expect(r.pnl_2_5th_percentile).toBeLessThan(r.pnl_97_5th_percentile);
});

test('demo high-variance: 95% CI wider than low-variance', () => {
    const hi = makeDemoInput('high-variance');
    const lo = makeDemoInput('low-variance');
    const rh = localBootstrap(hi.trade_pnls, hi.n_resamples, hi.seed);
    const rl = localBootstrap(lo.trade_pnls, lo.n_resamples, lo.seed);
    const wh = rh.pnl_97_5th_percentile - rh.pnl_2_5th_percentile;
    const wl = rl.pnl_97_5th_percentile - rl.pnl_2_5th_percentile;
    expect(wh).toBeGreaterThan(wl);
});

// ── round-trip + formatters ──────────────────────────────────────

test('tradesToBlob round-trips through parseTradesBlob', () => {
    const trades = [10, -5, 20, -8, 15];
    const back = parseTradesBlob(tradesToBlob(trades));
    expect(back.errors).toEqual([]);
    expect(back.trade_pnls).toEqual(trades);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtUSD(1234)).toBe('$1234.00');
    expect(fmtUSDSigned(50)).toBe('+$50.00');
    expect(fmtUSDSigned(-50)).toBe('-$50.00');
    expect(fmtPct(0.75)).toBe('75.00%');
    expect(fmtInt(7.9)).toBe('7');
    expect(fmtUSD(null)).toBe('—');
    expect(fmtUSDSigned(NaN)).toBe('—');
});
