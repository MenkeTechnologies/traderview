// Liquidity pure helpers: trade + ADV parsers, validator, body builder,
// Trade synthesis, tier classifier, demo invariants, formatters.

import { test, expect } from 'vitest';
import {
    parseTradeLines, parseAdvLines, validateInputs, buildBody,
    synthesizeTrade, liquidityTier, makeDemoData,
    fmtN, fmtPct, fmtUSD,
} from '../js/_liquidity_inputs.js';

// ── parseTradeLines ────────────────────────────────────────────────

test('parseTradeLines accepts whitespace + commas, uppercases symbols, skips comments', () => {
    const r = parseTradeLines('# header\naapl 100 75\nmsft, 2000, -150');
    expect(r.errors).toEqual([]);
    expect(r.trades).toEqual([
        { symbol: 'AAPL', qty: 100, net_pnl: 75 },
        { symbol: 'MSFT', qty: 2000, net_pnl: -150 },
    ]);
});

test('parseTradeLines rejects wrong token count', () => {
    const r = parseTradeLines('AAPL 100');
    expect(r.errors.length).toBe(1);
    expect(r.errors[0].message).toMatch(/expected 3 tokens/);
});

test('parseTradeLines rejects non-positive qty + non-finite pnl', () => {
    const r = parseTradeLines('AAPL 0 1\nMSFT 100 abc\nGOOG -10 5');
    expect(r.trades).toEqual([]);
    expect(r.errors.length).toBe(3);
});

test('parseTradeLines accepts negative pnl', () => {
    const r = parseTradeLines('AAPL 100 -500');
    expect(r.trades[0].net_pnl).toBe(-500);
});

test('parseTradeLines rejects malformed symbol', () => {
    const r = parseTradeLines('A!P 100 5');
    expect(r.trades).toEqual([]);
    expect(r.errors[0].message).toMatch(/bad symbol/);
});

// ── parseAdvLines ──────────────────────────────────────────────────

test('parseAdvLines parses + uppercases', () => {
    const r = parseAdvLines('aapl 50000000\nmsft, 1500000');
    expect(r.errors).toEqual([]);
    expect(r.adv).toEqual({ AAPL: 50_000_000, MSFT: 1_500_000 });
});

test('parseAdvLines rejects non-positive ADV', () => {
    const r = parseAdvLines('AAPL 0\nMSFT -100');
    expect(r.adv).toEqual({});
    expect(r.errors.length).toBe(2);
});

test('parseAdvLines rejects wrong token count', () => {
    const r = parseAdvLines('AAPL');
    expect(r.errors[0].message).toMatch(/expected 2 tokens/);
});

// ── validateInputs ────────────────────────────────────────────────

test('validate accepts trades + matching ADV', () => {
    expect(validateInputs(
        [{ symbol: 'AAPL', qty: 100, net_pnl: 1 }],
        { AAPL: 1e6 },
    )).toBe(null);
});

test('validate rejects empty trades or adv', () => {
    expect(validateInputs([], { AAPL: 1 })).toMatch(/at least 1 trade/);
    expect(validateInputs([{ symbol: 'AAPL', qty: 1, net_pnl: 1 }], {})).toMatch(/symbol/);
});

test('validate flags total-miss (no trade symbol has ADV)', () => {
    expect(validateInputs(
        [{ symbol: 'AAPL', qty: 1, net_pnl: 1 }],
        { MSFT: 1e6 },
    )).toMatch(/no trade symbol has ADV/);
});

test('validate passes when at least 1 trade symbol matches', () => {
    expect(validateInputs(
        [{ symbol: 'AAPL', qty: 1, net_pnl: 1 }, { symbol: 'XYZ', qty: 1, net_pnl: 1 }],
        { AAPL: 1e6 },
    )).toBe(null);
});

// ── synthesizeTrade ───────────────────────────────────────────────

test('synthesizeTrade emits deterministic IDs and stringifies Decimal fields', () => {
    const t = synthesizeTrade({ symbol: 'AAPL', qty: 100, net_pnl: 75 }, 5);
    expect(t.id).toBe('00000000-0000-4000-8000-000000000005');
    expect(t.symbol).toBe('AAPL');
    expect(t.qty).toBe('100');
    expect(t.net_pnl).toBe('75');
    expect(t.side).toBe('long');
});

test('synthesizeTrade flips side based on pnl sign', () => {
    expect(synthesizeTrade({ symbol: 'X', qty: 1, net_pnl: -1 }, 0).side).toBe('short');
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody emits backend LiquidityBody shape (Decimal-as-string)', () => {
    const body = buildBody(
        [{ symbol: 'AAPL', qty: 100, net_pnl: 75 }],
        { AAPL: 1e6 },
    );
    expect(body.trades.length).toBe(1);
    expect(body.trades[0].symbol).toBe('AAPL');
    expect(body.adv).toEqual({ AAPL: '1000000' });
});

// ── liquidityTier ─────────────────────────────────────────────────

test('liquidityTier buckets at 0.1% / 1% / 5% / 20%', () => {
    expect(liquidityTier(0.0005).label).toMatch(/invisible/);
    expect(liquidityTier(0.0005).cls).toBe('pos');
    expect(liquidityTier(0.005).label).toMatch(/normal/);
    expect(liquidityTier(0.005).cls).toBe('pos');
    expect(liquidityTier(0.03).label).toMatch(/large/);
    expect(liquidityTier(0.03).cls).toBe('');
    expect(liquidityTier(0.10).label).toMatch(/illiquid/);
    expect(liquidityTier(0.10).cls).toBe('neg');
    expect(liquidityTier(0.50).label).toMatch(/whale/);
    expect(liquidityTier(0.50).cls).toBe('neg');
});

test('liquidityTier returns em-dash on non-finite', () => {
    expect(liquidityTier(NaN).label).toBe('—');
});

// ── makeDemoData ──────────────────────────────────────────────────

test('makeDemoData has 4 symbols and 53 trades', () => {
    const { trades, adv } = makeDemoData();
    expect(trades.length).toBe(53);
    expect(Object.keys(adv).sort()).toEqual(['AAPL', 'ILQD', 'MSFT', 'SMID']);
});

test('makeDemoData spans all liquidity tiers when bucketed against ADV', () => {
    const { trades, adv } = makeDemoData();
    const pcts = trades.map(t => t.qty / adv[t.symbol]).filter(Number.isFinite);
    const hasInvisible = pcts.some(p => p < 0.001);
    const hasNormal    = pcts.some(p => p >= 0.001 && p < 0.01);
    const hasLarge     = pcts.some(p => p >= 0.01 && p < 0.05);
    const hasIlliquid  = pcts.some(p => p >= 0.05);
    expect(hasInvisible && hasNormal && hasLarge && hasIlliquid).toBe(true);
});

// ── formatters ─────────────────────────────────────────────────────

test('fmtN locale-formats integers', () => {
    expect(fmtN(1234567)).toBe('1,234,567');
    expect(fmtN(NaN)).toBe('—');
});

test('fmtPct emits 3-decimal percent', () => {
    expect(fmtPct(0.01234)).toBe('1.234%');
    expect(fmtPct(NaN)).toBe('—');
});

test('fmtUSD signs and 2-decimal', () => {
    expect(fmtUSD(1234.56)).toBe('$1234.56');
    expect(fmtUSD(-99.5)).toBe('-$99.50');
    expect(fmtUSD(NaN)).toBe('—');
});
