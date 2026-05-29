// Setups-by-setup helpers: parser, body shape (synthetic Trade per row
// + UUID-keyed setup map), local stats roll-up parity with Rust
// stats_by_setup, formatters.

import { test, expect } from 'vitest';
import {
    parseSetupTradeBlob, validateInputs, buildBody, makeDeterministicUuid,
    localAnalyze, dec, setupBadge, makeDemoRows,
    fmtUSD, fmtUSDSigned, fmtPct, fmtPF, fmtR,
} from '../js/_setups_by_setup_inputs.js';

const row = (s, p, r) => ({ setup: s, net_pnl: p, risk_amount: r });

// ── parser ────────────────────────────────────────────────────────

test('parser accepts 2- or 3-token rows + comments', () => {
    const r = parseSetupTradeBlob('orb 500 100   # win\n# pure comment\nabcd -150');
    expect(r.errors).toEqual([]);
    expect(r.rows).toEqual([
        row('orb', 500, 100),
        row('abcd', -150, null),
    ]);
});

test('parser accepts "-" setup tag (untagged sentinel)', () => {
    const r = parseSetupTradeBlob('- 100 50');
    expect(r.errors).toEqual([]);
    expect(r.rows[0].setup).toBe('-');
});

test('parser accepts negative net_pnl', () => {
    const r = parseSetupTradeBlob('fade -250');
    expect(r.errors).toEqual([]);
    expect(r.rows[0].net_pnl).toBe(-250);
});

test('parser rejects non-finite net_pnl / risk', () => {
    expect(parseSetupTradeBlob('orb abc').errors[0].message).toMatch(/net_pnl/);
    expect(parseSetupTradeBlob('orb 100 abc').errors[0].message).toMatch(/risk_amount/);
});

test('parser rejects risk_amount ≤ 0 (would divide-by-zero in R calc)', () => {
    expect(parseSetupTradeBlob('orb 100 0').errors[0].message).toMatch(/risk_amount/);
    expect(parseSetupTradeBlob('orb 100 -5').errors[0].message).toMatch(/risk_amount/);
});

test('parser rejects wrong token count (1 or ≥ 4)', () => {
    expect(parseSetupTradeBlob('orb').errors[0].message).toMatch(/2 or 3 tokens/);
    expect(parseSetupTradeBlob('a b c d').errors[0].message).toMatch(/2 or 3 tokens/);
});

test('parser non-string returns 1 error', () => {
    expect(parseSetupTradeBlob(null).errors.length).toBe(1);
});

// ── validateInputs / buildBody ────────────────────────────────────

test('validate rejects empty', () => {
    expect(validateInputs([])).toMatch(/≥ 1 trade/);
});

test('buildBody emits trades[] + trade_setups{}; "-" rows excluded from map', () => {
    const rows = [row('orb', 100, 50), row('-', 999, 50)];
    const body = buildBody(rows);
    expect(body.trades.length).toBe(2);
    expect(Object.keys(body.trade_setups).length).toBe(1);
    const taggedId = body.trades[0].id;
    expect(body.trade_setups[taggedId]).toBe('orb');
    // Untagged trade IS sent but its id is NOT a key in the map.
    expect(body.trade_setups[body.trades[1].id]).toBeUndefined();
});

test('buildBody synthesizes a Trade with status=closed + Decimal-as-string fields', () => {
    const body = buildBody([row('orb', 100, 50)]);
    const t = body.trades[0];
    expect(t.status).toBe('closed');
    expect(t.net_pnl).toBe('100');
    expect(t.gross_pnl).toBe('100');
    expect(t.fees).toBe('0');
    expect(t.risk_amount).toBe('50');
});

test('buildBody preserves null risk_amount when row has none', () => {
    const body = buildBody([row('orb', 100, null)]);
    expect(body.trades[0].risk_amount).toBeNull();
});

test('makeDeterministicUuid produces valid 8-4-4-4-12 hex pattern', () => {
    const id = makeDeterministicUuid(1);
    expect(id).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/);
    expect(makeDeterministicUuid(1)).toBe(makeDeterministicUuid(1));
    expect(makeDeterministicUuid(1)).not.toBe(makeDeterministicUuid(2));
});

// ── localAnalyze parity (one test per Rust test case + extras) ────

test('local: empty rows → empty', () => {
    expect(localAnalyze([])).toEqual([]);
});

test('local: untagged-only rows → empty', () => {
    expect(localAnalyze([row('-', 100, 50), row('-', 200, 50)])).toEqual([]);
});

test('local: single winning setup computes correctly', () => {
    const out = localAnalyze([row('orb', 500, 100)]);
    expect(out.length).toBe(1);
    const s = out[0];
    expect(s.setup).toBe('orb');
    expect(s.trades).toBe(1);
    expect(s.wins).toBe(1);
    expect(s.net_pnl).toBe(500);
    expect(s.avg_win).toBe(500);
    expect(s.profit_factor).toBe(Infinity);
    expect(s.avg_r).toBeCloseTo(5.0, 9);
});

test('local: mixed setup aggregates wins / losses / avgs / PF / win-rate', () => {
    const out = localAnalyze([
        row('abcd', 200, null), row('abcd', 300, null), row('abcd', -100, null),
    ]);
    const s = out[0];
    expect(s.trades).toBe(3);
    expect(s.wins).toBe(2);
    expect(s.losses).toBe(1);
    expect(s.net_pnl).toBe(400);
    expect(s.avg_win).toBe(250);
    expect(s.avg_loss).toBe(-100);
    expect(s.win_rate).toBeCloseTo(2 / 3, 9);
    expect(s.profit_factor).toBeCloseTo(5.0, 9); // 500 / 100
});

test('local: results sorted by net_pnl DESC', () => {
    const out = localAnalyze([
        row('big',   1000, null),
        row('meh',    100, null),
        row('loser', -500, null),
    ]);
    expect(out.map(s => s.setup)).toEqual(['big', 'meh', 'loser']);
});

test('local: scratch trades counted separately (not win/loss)', () => {
    const out = localAnalyze([row('be', 0, null), row('be', 0, null)]);
    expect(out[0].scratches).toBe(2);
    expect(out[0].wins).toBe(0);
    expect(out[0].losses).toBe(0);
});

test('local: largest_win / largest_loss track extremes', () => {
    const out = localAnalyze([
        row('orb', 200, null), row('orb', 800, null),
        row('orb', -50, null), row('orb', -500, null),
    ]);
    expect(out[0].largest_win).toBe(800);
    expect(out[0].largest_loss).toBe(-500);
});

test('local: avg_r averages per-trade R (skips rows without risk)', () => {
    const out = localAnalyze([
        row('orb', 200, 100),  // +2R
        row('orb', 400, 100),  // +4R
        row('orb', 300, null), // no R contribution
    ]);
    expect(out[0].avg_r).toBeCloseTo(3.0, 9);
});

test('local: zero wins + zero losses → profit_factor = 0', () => {
    const out = localAnalyze([row('be', 0, null), row('be', 0, null)]);
    expect(out[0].profit_factor).toBe(0);
});

test('local: untagged "-" rows excluded from buckets', () => {
    const out = localAnalyze([row('orb', 100, null), row('-', 999, null)]);
    expect(out.length).toBe(1);
    expect(out[0].setup).toBe('orb');
    expect(out[0].net_pnl).toBe(100);
});

test('local: expectancy == avg_pnl', () => {
    const out = localAnalyze([row('orb', 200, null), row('orb', -100, null)]);
    expect(out[0].expectancy).toBeCloseTo(out[0].avg_pnl, 12);
});

// ── decoder + badge ───────────────────────────────────────────────

test('dec: string / number / null safely → number', () => {
    expect(dec('1234.5')).toBe(1234.5);
    expect(dec(7)).toBe(7);
    expect(dec(null)).toBe(0);
    expect(dec('abc')).toBe(0);
});

test('setupBadge: pos / neg / scratch by avg_pnl', () => {
    expect(setupBadge({ avg_pnl: 100 }).cls).toBe('pos');
    expect(setupBadge({ avg_pnl: -50 }).cls).toBe('neg');
    expect(setupBadge({ avg_pnl: 0  }).cls).toBe('');
    expect(setupBadge(null).label).toBe('—');
});

// ── demos invariants ──────────────────────────────────────────────

test('demo single-winner: PF infinite (no losses)', () => {
    const out = localAnalyze(makeDemoRows('single-winner'));
    expect(out.length).toBe(1);
    expect(out[0].profit_factor).toBe(Infinity);
    expect(out[0].wins).toBe(3);
});

test('demo single-loser: avg_pnl negative', () => {
    const out = localAnalyze(makeDemoRows('single-loser'));
    expect(out[0].avg_pnl).toBeLessThan(0);
});

test('demo mixed: 3 setups, ranked by net_pnl DESC', () => {
    const out = localAnalyze(makeDemoRows('mixed'));
    expect(out.length).toBe(3);
    for (let i = 1; i < out.length; i++) {
        expect(out[i].net_pnl).toBeLessThanOrEqual(out[i - 1].net_pnl);
    }
});

test('demo with-untagged: only 2 setups (untagged "-" rows excluded)', () => {
    const out = localAnalyze(makeDemoRows('with-untagged'));
    expect(out.length).toBe(2);
    expect(out.find(s => s.setup === '-')).toBeUndefined();
});

test('demo all-scratches: 0 wins, 0 losses, all 3 scratch', () => {
    const out = localAnalyze(makeDemoRows('all-scratches'));
    expect(out[0].wins).toBe(0);
    expect(out[0].losses).toBe(0);
    expect(out[0].scratches).toBe(3);
});

// ── formatters ────────────────────────────────────────────────────

test('fmt helpers: USD/signed/pct/PF/R + non-finite + infinity', () => {
    expect(fmtUSD(1234.5)).toBe('$1234.50');
    expect(fmtUSD(-100)).toBe('-$100.00');
    expect(fmtUSDSigned(100)).toBe('+$100.00');
    expect(fmtUSDSigned(-100)).toBe('-$100.00');
    expect(fmtPct(0.66667)).toBe('66.7%');
    expect(fmtPF(2.5)).toBe('2.50');
    expect(fmtPF(Infinity)).toBe('∞');
    expect(fmtPF(NaN)).toBe('—');
    expect(fmtR(3.0)).toBe('+3.00R');
    expect(fmtR(-1.0)).toBe('-1.00R');
});
