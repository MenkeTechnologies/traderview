// Commission-optimizer helpers: parser, validator, body shape (Decimals
// as strings), feeForTier + localEvaluate parity (mirror of Rust evaluate
// with sort-cheapest-first + best_alternative + annual projection),
// defaultTiers, demo invariants.

import { test, expect } from 'vitest';
import {
    parseExecutionBlob, validateInputs, buildBody, dec,
    feeForTier, localEvaluate, defaultTiers, makeDemoExecutions,
    savingsBadge, fmtUSD, fmtUSDSigned, fmtPct, fmtN, fmtInt,
} from '../js/_commission_optimizer_inputs.js';

const ex = (qty, notional, fee) => ({ qty, notional, actual_fee: fee });
const tier = (over = {}) => ({
    name: 'test', per_trade_flat: 0, per_share: 0, per_dollar: 0,
    min_per_trade: 0, max_per_trade: 0, ...over,
});

// ── parser ────────────────────────────────────────────────────────

test('parser accepts 3 tokens + comments', () => {
    const r = parseExecutionBlob('100 5000 1.00\n# note\n200,8000,1.00');
    expect(r.errors).toEqual([]);
    expect(r.executions).toEqual([ex(100, 5000, 1), ex(200, 8000, 1)]);
});

test('parser rejects wrong token count / non-finite / non-positive qty / non-positive notional / negative fee', () => {
    expect(parseExecutionBlob('100 5000').errors[0].message).toMatch(/3 tokens/);
    expect(parseExecutionBlob('abc 5000 1').errors[0].message).toMatch(/finite/);
    expect(parseExecutionBlob('0 5000 1').errors[0].message).toMatch(/qty/);
    expect(parseExecutionBlob('100 0 1').errors[0].message).toMatch(/notional/);
    expect(parseExecutionBlob('100 5000 -1').errors[0].message).toMatch(/actual_fee/);
});

test('parser accepts fee=0 (commission-free brokers)', () => {
    expect(parseExecutionBlob('100 5000 0').errors).toEqual([]);
});

test('parser non-string returns 1 error', () => {
    expect(parseExecutionBlob(null).errors.length).toBe(1);
});

// ── validator / buildBody ─────────────────────────────────────────

test('validate accepts good inputs', () => {
    expect(validateInputs([ex(1, 1, 0)], [tier()])).toBe(null);
});

test('validate rejects empty / bad tier', () => {
    expect(validateInputs([], [tier()])).toMatch(/≥ 1 execution/);
    expect(validateInputs([ex(1, 1, 0)], [])).toMatch(/≥ 1 tier/);
    expect(validateInputs([ex(1, 1, 0)], [{ name: '' }])).toMatch(/name required/);
    expect(validateInputs([ex(1, 1, 0)], [tier({ per_share: -1 })])).toMatch(/per_share/);
    expect(validateInputs([ex(1, 1, 0)], [tier({ min_per_trade: NaN })])).toMatch(/min_per_trade/);
});

test('buildBody emits Decimal-as-string contract on execs + tiers', () => {
    const body = buildBody([ex(100, 5000, 1)], [tier({ per_share: 0.0035 })]);
    expect(body.executions[0]).toEqual({
        qty: '100', notional: '5000', actual_fee: '1',
    });
    expect(body.tiers[0].per_share).toBe('0.0035');
    expect(body.tiers[0].min_per_trade).toBe('0');
});

// ── feeForTier mirror of Rust Tier::fee_for ───────────────────────

test('feeForTier: pure per-share', () => {
    expect(feeForTier(tier({ per_share: 0.0035 }), ex(100, 5000, 0))).toBeCloseTo(0.35, 10);
});

test('feeForTier: min_per_trade floor', () => {
    // Raw = 10 sh × 0.0035 = 0.035 → floor 0.35.
    expect(feeForTier(tier({ per_share: 0.0035, min_per_trade: 0.35 }), ex(10, 500, 0))).toBe(0.35);
});

test('feeForTier: max_per_trade ceiling', () => {
    // Raw = 10000 × 0.0035 = 35 → cap 5.
    expect(feeForTier(tier({ per_share: 0.0035, max_per_trade: 5 }), ex(10000, 500000, 0))).toBe(5);
});

test('feeForTier: pure flat', () => {
    expect(feeForTier(tier({ per_trade_flat: 1 }), ex(50, 100, 0))).toBe(1);
});

test('feeForTier: pure per_dollar', () => {
    // 0.000119 × 10000 = 1.19.
    expect(feeForTier(tier({ per_dollar: 0.000119 }), ex(100, 10000, 0))).toBeCloseTo(1.19, 9);
});

test('feeForTier: composite flat + per_share + per_dollar', () => {
    const t_ = tier({ per_trade_flat: 0.50, per_share: 0.01, per_dollar: 0.0001 });
    // 0.50 + 100 × 0.01 + 5000 × 0.0001 = 0.50 + 1.00 + 0.50 = 2.00.
    expect(feeForTier(t_, ex(100, 5000, 0))).toBeCloseTo(2.00, 10);
});

test('feeForTier: min/max=0 means "no floor/ceiling" not "fee=0"', () => {
    expect(feeForTier(tier({ per_share: 0.0035, min_per_trade: 0, max_per_trade: 0 }),
                     ex(100, 5000, 0))).toBeCloseTo(0.35, 10);
});

// ── localEvaluate parity (one test per Rust property) ─────────────

test('local: empty execs → default zeroed report + null best', () => {
    const out = localEvaluate([], [tier()]);
    expect(out.trade_count).toBe(0);
    expect(out.best_alternative).toBeNull();
});

test('local: aggregates totals from execs', () => {
    const out = localEvaluate([ex(100, 5000, 1), ex(200, 10000, 2)], [tier()]);
    expect(out.trade_count).toBe(2);
    expect(out.total_shares).toBe(300);
    expect(out.total_notional).toBe(15000);
    expect(out.actual_total_fee).toBe(3);
});

test('local: tiers sorted by total_fee ascending', () => {
    const execs = [ex(100, 5000, 1)];
    const out = localEvaluate(execs, defaultTiers());
    expect(out.tiers[0].tier).toBe('Webull (zero-commission)');
    expect(out.tiers[out.tiers.length - 1].tier).toBe('Lightspeed Active');
});

test('local: best_alternative only set when STRICTLY cheaper than actual', () => {
    // Actual = 0, Webull = 0 → tie → no recommendation.
    const out = localEvaluate([ex(100, 5000, 0)], defaultTiers());
    expect(out.best_alternative).toBeNull();
    expect(out.projected_annual_savings).toBe(0);
});

test('local: zero-commission tier beats $1-fee actual', () => {
    // 100 trades × $1 fee = $100/mo. Webull tier saves $100/mo = $1200/yr.
    const execs = Array.from({ length: 100 }, () => ex(100, 5000, 1));
    const out = localEvaluate(execs, defaultTiers());
    expect(out.actual_total_fee).toBe(100);
    expect(out.best_alternative).toBe('Webull (zero-commission)');
    expect(out.projected_annual_savings).toBe(1200);
});

test('local: fee_pct_of_notional computed correctly', () => {
    // $1 fee on $1000 notional = 0.1%.
    const out = localEvaluate([ex(10, 1000, 1)],
        [tier({ name: 't', per_trade_flat: 1 })]);
    expect(out.tiers[0].fee_pct_of_notional).toBeCloseTo(0.1, 9);
});

test('local: fee_per_trade = total / trade_count', () => {
    const execs = [ex(10, 1000, 1), ex(10, 1000, 1)];
    const out = localEvaluate(execs, [tier({ per_trade_flat: 2 })]);
    expect(out.tiers[0].fee_per_trade).toBe(2);
});

test('local: fee_per_share = total / total_shares (0 when no shares)', () => {
    const out = localEvaluate([ex(100, 5000, 0)], [tier({ per_trade_flat: 5 })]);
    expect(out.tiers[0].fee_per_share).toBe(5 / 100);
});

test('local: delta_vs_actual signed correctly', () => {
    const out = localEvaluate([ex(100, 5000, 5)],
        [tier({ name: 'cheap', per_trade_flat: 1 })]);
    expect(out.tiers[0].delta_vs_actual).toBe(-4);  // 1 vs 5 → savings
});

// ── defaultTiers ──────────────────────────────────────────────────

test('defaultTiers: returns IBKR / Lightspeed / Webull', () => {
    const t = defaultTiers();
    expect(t.map(x => x.name)).toEqual([
        'IBKR Pro tiered', 'Lightspeed Active', 'Webull (zero-commission)',
    ]);
});

test('defaultTiers: IBKR per_share = 0.0035 with $0.35 floor', () => {
    const ibkr = defaultTiers()[0];
    expect(ibkr.per_share).toBe(0.0035);
    expect(ibkr.min_per_trade).toBe(0.35);
});

test('defaultTiers: Webull is entirely zero', () => {
    const webull = defaultTiers()[2];
    expect(webull.per_share).toBe(0);
    expect(webull.per_trade_flat).toBe(0);
    expect(webull.per_dollar).toBe(0);
    expect(webull.min_per_trade).toBe(0);
});

// ── dec coercion ──────────────────────────────────────────────────

test('dec: string / number / null safely → number', () => {
    expect(dec('100.50')).toBe(100.5);
    expect(dec(7)).toBe(7);
    expect(dec(null)).toBe(0);
    expect(dec('abc')).toBe(0);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset returns ≥ 5 valid executions', () => {
    for (const k of ['active-retail', 'scalper-heavy', 'options-light', 'webull-zero', 'big-blocks']) {
        const e = makeDemoExecutions(k);
        expect(e.length).toBeGreaterThanOrEqual(5);
        for (const x of e) {
            expect(x.qty).toBeGreaterThan(0);
            expect(x.notional).toBeGreaterThan(0);
            expect(x.actual_fee).toBeGreaterThanOrEqual(0);
        }
    }
});

test('demo webull-zero: all fees = 0', () => {
    for (const e of makeDemoExecutions('webull-zero')) {
        expect(e.actual_fee).toBe(0);
    }
});

test('demo active-retail: cheapest tier (Webull) strictly beats actual', () => {
    const out = localEvaluate(makeDemoExecutions('active-retail'), defaultTiers());
    expect(out.best_alternative).toBe('Webull (zero-commission)');
    expect(out.projected_annual_savings).toBeGreaterThan(0);
});

// ── savingsBadge ──────────────────────────────────────────────────

test('savingsBadge: thresholds 0 / <100 / <1000 / ≥1000', () => {
    expect(savingsBadge(0).key).toMatch(/optimal/);
    expect(savingsBadge(50).key).toMatch(/marginal/);
    expect(savingsBadge(500).key).toMatch(/meaningful/);
    expect(savingsBadge(2000).key).toMatch(/significant/);
    expect(savingsBadge(NaN).key).toMatch(/unknown/);
});

// ── formatters ────────────────────────────────────────────────────

test('fmt helpers + non-finite guards', () => {
    expect(fmtUSD(1234.5)).toBe('$1234.50');
    expect(fmtUSDSigned(-100)).toBe('-$100.00');
    expect(fmtPct(0.123, 3)).toBe('0.123%');
    expect(fmtN(3.14159, 2)).toBe('3.14');
    expect(fmtInt(7.9)).toBe('7');
    expect(fmtUSD(NaN)).toBe('—');
});
