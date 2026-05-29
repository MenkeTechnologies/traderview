// Buying-power helpers: validator, body shape (Decimal-as-string),
// localCompute Rust-mirror including match-order corner cases,
// badges, demos, formatters.

import { test, expect } from 'vitest';
import {
    ACCOUNT_TYPES, PDT_MIN_EQUITY, DEFAULT_INPUTS,
    validateInputs, buildBody, localCompute, dec,
    leverageBadge, pdtStatusKey, makeDemoInput,
    fmtUSD, fmtNum, fmtX, fmtPct,
} from '../js/_buying_power_inputs.js';

const inp = (over = {}) => ({ ...DEFAULT_INPUTS, ...over });

// ── constants + validator ─────────────────────────────────────────

test('ACCOUNT_TYPES exposes the three Rust variants', () => {
    expect(ACCOUNT_TYPES).toEqual(['cash', 'reg_t', 'portfolio_margin']);
});

test('PDT_MIN_EQUITY matches Rust constant ($25k)', () => {
    expect(PDT_MIN_EQUITY).toBe(25_000);
});

test('validate accepts good inputs', () => {
    expect(validateInputs(inp())).toBe(null);
});

test('validate rejects bad account_type', () => {
    expect(validateInputs(inp({ account_type: 'pdt' }))).toMatch(/account_type/);
});

test('validate rejects non-finite / negative numerics', () => {
    expect(validateInputs(inp({ equity: NaN }))).toMatch(/equity/);
    expect(validateInputs(inp({ equity: -1 }))).toMatch(/equity/);
    expect(validateInputs(inp({ share_price: NaN }))).toMatch(/share_price/);
    expect(validateInputs(inp({ share_price: -1 }))).toMatch(/share_price/);
});

test('validate rejects non-boolean is_pdt / is_day_trade', () => {
    expect(validateInputs(inp({ is_pdt: 'yes' }))).toMatch(/is_pdt/);
    expect(validateInputs(inp({ is_day_trade: 1 }))).toMatch(/is_day_trade/);
});

// ── buildBody Decimal-as-string ───────────────────────────────────

test('buildBody stringifies Decimal fields, passes bools through', () => {
    const body = buildBody(inp({ equity: 10_000, share_price: 50 }));
    expect(body.equity).toBe('10000');
    expect(body.share_price).toBe('50');
    expect(body.is_pdt).toBe(false);
    expect(body.is_day_trade).toBe(false);
    expect(body.account_type).toBe('reg_t');
});

// ── localCompute parity (one test per Rust match arm) ────────────

test('local: cash 1× notional matches equity', () => {
    const r = localCompute(inp({ account_type: 'cash', equity: 10_000, share_price: 50 }));
    expect(r.leverage).toBe(1.0);
    expect(r.max_notional).toBe(10_000);
    expect(r.max_shares).toBe(200);
    expect(r.note_key).toMatch(/note\.cash$/);
});

test('local: Reg-T overnight = 2×', () => {
    const r = localCompute(inp({ account_type: 'reg_t', equity: 10_000, share_price: 50 }));
    expect(r.leverage).toBe(2.0);
    expect(r.max_notional).toBe(20_000);
    expect(r.note_key).toMatch(/note\.regt$/);
});

test('local: PDT day-trade ≥ $25k = 4×', () => {
    const r = localCompute(inp({ account_type: 'reg_t', equity: 30_000,
                                  is_pdt: true, is_day_trade: true, share_price: 50 }));
    expect(r.leverage).toBe(4.0);
    expect(r.max_notional).toBe(120_000);
    expect(r.note_key).toMatch(/note\.pdt$/);
});

test('local: PDT below $25k falls back to 2× Reg-T', () => {
    const r = localCompute(inp({ account_type: 'reg_t', equity: 20_000,
                                  is_pdt: true, is_day_trade: true, share_price: 50 }));
    expect(r.leverage).toBe(2.0);
});

test('local: PDT overnight loses 4× multiplier', () => {
    const r = localCompute(inp({ account_type: 'reg_t', equity: 30_000,
                                  is_pdt: true, is_day_trade: false, share_price: 50 }));
    expect(r.leverage).toBe(2.0);
});

test('local: sub-$5 stock in Reg-T forces 1× + 100% initial', () => {
    const r = localCompute(inp({ account_type: 'reg_t', equity: 10_000, share_price: 3 }));
    expect(r.leverage).toBe(1.0);
    expect(r.initial_requirement_pct).toBe(1.00);
    expect(r.note_key).toMatch(/note\.sub5$/);
});

test('local: initial_requirement drops to 50% at price > $5', () => {
    expect(localCompute(inp({ account_type: 'cash', share_price: 5.01 })).initial_requirement_pct).toBe(0.50);
    // EXACTLY $5 still triggers sub-$5 (Rust uses strict <).
    expect(localCompute(inp({ account_type: 'cash', share_price: 5 })).initial_requirement_pct).toBe(0.50);
    expect(localCompute(inp({ account_type: 'cash', share_price: 4.99 })).initial_requirement_pct).toBe(1.00);
});

test('local: share_price=0 → max_shares=0 (no divide-by-zero)', () => {
    const r = localCompute(inp({ account_type: 'cash', equity: 10_000, share_price: 0 }));
    expect(r.max_shares).toBe(0);
});

test('local: portfolio margin overnight = 3×', () => {
    const r = localCompute(inp({ account_type: 'portfolio_margin', equity: 100_000, share_price: 100 }));
    expect(r.leverage).toBe(3.0);
    expect(r.max_notional).toBe(300_000);
    expect(r.note_key).toMatch(/note\.pm$/);
});

test('local: portfolio margin + PDT day-trade = 6×', () => {
    const r = localCompute(inp({ account_type: 'portfolio_margin', equity: 100_000,
                                  is_pdt: true, is_day_trade: true, share_price: 100 }));
    expect(r.leverage).toBe(6.0);
    expect(r.note_key).toMatch(/note\.pm_pdt$/);
});

// ── corner case: PDT match arm fires BEFORE sub-$5 check ──────────

test('local: PDT day-trade + sub-$5 → 4× (PDT arm matches first in Rust)', () => {
    const r = localCompute(inp({ account_type: 'reg_t', equity: 30_000,
                                  is_pdt: true, is_day_trade: true, share_price: 3 }));
    expect(r.leverage).toBe(4.0);
    expect(r.note_key).toMatch(/note\.pdt$/);
    // But initial requirement stays at 100% (sub-$5 rule separate).
    expect(r.initial_requirement_pct).toBe(1.00);
});

// ── leverageBadge tiers ───────────────────────────────────────────

test('leverageBadge: 1×=none, 2×=moderate, 4×=high, 6×=extreme', () => {
    expect(leverageBadge(1.0).key).toMatch(/none/);
    expect(leverageBadge(2.0).key).toMatch(/moderate/);
    expect(leverageBadge(3.0).key).toMatch(/moderate/);
    expect(leverageBadge(4.0).key).toMatch(/high/);
    expect(leverageBadge(6.0).key).toMatch(/extreme/);
    expect(leverageBadge(NaN).key).toMatch(/unknown/);
});

// ── pdtStatusKey ──────────────────────────────────────────────────

test('pdtStatusKey: not_flagged / below_25k / overnight / active', () => {
    expect(pdtStatusKey(inp({ is_pdt: false }))).toMatch(/not_flagged/);
    expect(pdtStatusKey(inp({ is_pdt: true, equity: 20_000, is_day_trade: true }))).toMatch(/below_25k/);
    expect(pdtStatusKey(inp({ is_pdt: true, equity: 30_000, is_day_trade: false }))).toMatch(/overnight/);
    expect(pdtStatusKey(inp({ is_pdt: true, equity: 30_000, is_day_trade: true }))).toMatch(/active/);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset classifies into expected leverage', () => {
    const expectations = {
        'cash':              1.0,
        'reg-t-overnight':   2.0,
        'pdt-day-trade':     4.0,
        'pdt-below-25k':     2.0,
        'pdt-overnight':     2.0,
        'sub-5':             1.0,
        'pdt-sub-5':         4.0,
        'portfolio-margin':  3.0,
        'pm-pdt-day':        6.0,
        'zero-price':        1.0,
    };
    for (const [k, lev] of Object.entries(expectations)) {
        expect(localCompute(makeDemoInput(k)).leverage).toBe(lev);
    }
});

test('demo zero-price: max_shares = 0', () => {
    const r = localCompute(makeDemoInput('zero-price'));
    expect(r.max_shares).toBe(0);
});

// ── dec / formatters ──────────────────────────────────────────────

test('dec coerces strings and guards null/garbage', () => {
    expect(dec('100.5')).toBe(100.5);
    expect(dec(null)).toBe(0);
    expect(dec('abc')).toBe(0);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtUSD(1234)).toBe('$1234.00');
    expect(fmtNum(0.1234, 2)).toBe('0.12');
    expect(fmtX(4)).toBe('4.0×');
    expect(fmtPct(0.5)).toBe('50%');
    expect(fmtUSD(NaN)).toBe('—');
    expect(fmtX(null)).toBe('—');
});
