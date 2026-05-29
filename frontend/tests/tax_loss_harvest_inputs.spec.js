// Tax-loss-harvest helpers: parsers, validator, body shape, localSuggest
// Rust-mirror (sort-desc / wash-sale window / $3k cap / MTM skip),
// demos, badges, formatters.

import { test, expect } from 'vitest';
import {
    parseLoserBlob, parseRecentBuyBlob, validateInputs, buildBody,
    localSuggest, dec, harvestBadge, todayIso,
    isValidDate, daysBetween,
    makeDemoLosers, makeDemoRecentBuys,
    fmtUSD, fmtUSDSigned, fmtNum, fmtBool,
} from '../js/_tax_loss_harvest_inputs.js';

const loser = (s, q, c, p) => ({ symbol: s, qty: q, avg_cost: c, current_price: p });
const buy = (s, d) => ({ symbol: s, executed_at: d });

// ── parsers ───────────────────────────────────────────────────────

test('parseLoserBlob: 4 tokens, upcased, ignores comments', () => {
    const r = parseLoserBlob('aapl 100 150 140\n# note\nTSLA 10 300 250');
    expect(r.errors).toEqual([]);
    expect(r.losers).toEqual([loser('AAPL', 100, 150, 140), loser('TSLA', 10, 300, 250)]);
});

test('parseLoserBlob: rejects bad token count / non-positive qty / negative price', () => {
    expect(parseLoserBlob('AAPL 100 150').errors[0].message).toMatch(/4 tokens/);
    expect(parseLoserBlob('AAPL 0 150 140').errors[0].message).toMatch(/qty/);
    expect(parseLoserBlob('AAPL 100 150 -1').errors[0].message).toMatch(/current_price/);
});

test('parseLoserBlob: accepts price=0 (worthless / delisted)', () => {
    expect(parseLoserBlob('AAPL 100 150 0').errors).toEqual([]);
});

test('parseRecentBuyBlob: 2 tokens + valid date', () => {
    const r = parseRecentBuyBlob('aapl 2026-05-15');
    expect(r.errors).toEqual([]);
    expect(r.buys).toEqual([buy('AAPL', '2026-05-15')]);
});

test('parseRecentBuyBlob: rejects bad date format', () => {
    expect(parseRecentBuyBlob('AAPL 2026/05/15').errors[0].message).toMatch(/YYYY-MM-DD/);
    expect(parseRecentBuyBlob('AAPL 2026-02-30').errors[0].message).toMatch(/YYYY-MM-DD/);
});

test('parsers non-string returns 1 error', () => {
    expect(parseLoserBlob(null).errors.length).toBe(1);
    expect(parseRecentBuyBlob(null).errors.length).toBe(1);
});

// ── date helpers ──────────────────────────────────────────────────

test('isValidDate: strict YYYY-MM-DD, rejects invalid days/months', () => {
    expect(isValidDate('2026-05-29')).toBe(true);
    expect(isValidDate('2026-13-01')).toBe(false);
    expect(isValidDate('2026-02-30')).toBe(false);
    expect(isValidDate('2026/05/29')).toBe(false);
});

test('daysBetween: whole days, signed (b - a)', () => {
    expect(daysBetween('2026-05-29', '2026-05-30')).toBe(1);
    expect(daysBetween('2026-05-30', '2026-05-29')).toBe(-1);
    expect(daysBetween('2026-05-29', '2026-05-29')).toBe(0);
    expect(daysBetween('2026-01-01', '2026-12-31')).toBe(364);
});

test('daysBetween: invalid date → Infinity (no false-match)', () => {
    expect(daysBetween('bogus', '2026-05-29')).toBe(Infinity);
});

// ── validator / buildBody ─────────────────────────────────────────

test('validate accepts well-formed inputs', () => {
    expect(validateInputs([loser('A', 1, 10, 9)], [], '2026-12-15', 0, false)).toBe(null);
});

test('validate rejects bad date / non-finite ytd / non-boolean mtm', () => {
    expect(validateInputs([], [], 'bogus', 0, false)).toMatch(/today/);
    expect(validateInputs([], [], '2026-12-15', NaN, false)).toMatch(/realized_loss_ytd/);
    expect(validateInputs([], [], '2026-12-15', 0, 'maybe')).toMatch(/mtm_elected/);
});

test('buildBody: Decimal-as-string contract on all numeric loser fields', () => {
    const body = buildBody([loser('AAPL', 100, 150, 140)], [], '2026-12-15', 2500, true);
    expect(body.losers[0]).toEqual({
        symbol: 'AAPL', qty: '100', avg_cost: '150', current_price: '140',
    });
    expect(body.realized_loss_ytd).toBe('2500');
    expect(body.mtm_elected).toBe(true);
});

// ── localSuggest parity (one test per Rust property) ──────────────

test('local: winners (price ≥ cost) excluded from candidates', () => {
    const r = localSuggest([loser('AAPL', 100, 150, 160)], [], '2026-12-15', 0, false);
    expect(r.candidates).toEqual([]);
});

test('local: unrealized_loss positive when current < cost', () => {
    const r = localSuggest([loser('X', 100, 50, 40)], [], '2026-12-15', 0, false);
    expect(r.candidates[0].unrealized_loss).toBeCloseTo(1000, 9);
});

test('local: wash-sale risk flags ≤ 30-day buy', () => {
    const r = localSuggest(
        [loser('AAPL', 100, 150, 140)],
        [buy('AAPL', '2026-12-05')],
        '2026-12-15', 0, false);
    expect(r.candidates[0].wash_sale_risk).toBe(true);
    expect(r.candidates[0].note_key).toBe('view.tax_loss_harvest.note.wash_sale');
});

test('local: buy outside 30-day window does not flag', () => {
    const r = localSuggest(
        [loser('AAPL', 100, 150, 140)],
        [buy('AAPL', '2026-01-01')],
        '2026-12-15', 0, false);
    expect(r.candidates[0].wash_sale_risk).toBe(false);
});

test('local: $3k cap flagged when not MTM (running > 3000)', () => {
    const r = localSuggest(
        [loser('AAPL', 100, 150, 140)],   // $1k loss
        [], '2026-12-15', 2500, false);   // YTD already 2500
    expect(r.candidates[0].exceeds_3k_cap).toBe(true);
    expect(r.candidates[0].note_key).toBe('view.tax_loss_harvest.note.exceeds_3k');
});

test('local: MTM election skips $3k cap warning', () => {
    const r = localSuggest(
        [loser('AAPL', 100, 150, 140)],
        [], '2026-12-15', 10_000, true);
    expect(r.candidates[0].exceeds_3k_cap).toBe(false);
});

test('local: candidates sorted by loss size DESC', () => {
    const r = localSuggest([
        loser('TINY', 10,   50, 48),   // $20
        loser('BIG',  1000, 50, 30),   // $20000
        loser('MID',  100,  50, 40),   // $1000
    ], [], '2026-12-15', 0, false);
    expect(r.candidates.map(c => c.symbol)).toEqual(['BIG', 'MID', 'TINY']);
});

test('local: total_available_loss + safe_harvest_loss compute correctly', () => {
    const r = localSuggest([
        loser('AAPL', 100, 150, 140),   // $1k, wash
        loser('TSLA', 10,  300, 250),   // $500, safe
    ], [buy('AAPL', '2026-12-01')], '2026-12-15', 0, false);
    expect(r.total_available_loss).toBeCloseTo(1500, 9);
    expect(r.safe_harvest_loss).toBeCloseTo(500, 9);
});

test('local: empty inputs → empty report', () => {
    const r = localSuggest([], [], '2026-12-15', 0, false);
    expect(r.candidates).toEqual([]);
    expect(r.total_available_loss).toBe(0);
    expect(r.safe_harvest_loss).toBe(0);
});

// ── 30-day wash-sale boundary ─────────────────────────────────────

test('local: wash-sale check at exactly 30 days → flagged (≤ 30)', () => {
    // 30 days before 2026-12-15 = 2026-11-15.
    const r = localSuggest(
        [loser('AAPL', 100, 150, 140)],
        [buy('AAPL', '2026-11-15')],
        '2026-12-15', 0, false);
    expect(r.candidates[0].wash_sale_risk).toBe(true);
});

test('local: wash-sale check at 31 days → NOT flagged', () => {
    const r = localSuggest(
        [loser('AAPL', 100, 150, 140)],
        [buy('AAPL', '2026-11-14')],
        '2026-12-15', 0, false);
    expect(r.candidates[0].wash_sale_risk).toBe(false);
});

test('local: wash-sale also flags buys AFTER today within 30 days (future-dated forward window)', () => {
    // Rust uses .abs() on the day distance — both ±30 trigger.
    const r = localSuggest(
        [loser('AAPL', 100, 150, 140)],
        [buy('AAPL', '2026-12-30')],
        '2026-12-15', 0, false);
    expect(r.candidates[0].wash_sale_risk).toBe(true);
});

// ── harvestBadge ──────────────────────────────────────────────────

test('harvestBadge: ≤ 0 → no_harvest, < 500 → marginal, < 3000 → useful, ≥ 3000 → significant', () => {
    expect(harvestBadge(0).key).toMatch(/no_harvest/);
    expect(harvestBadge(200).key).toMatch(/marginal/);
    expect(harvestBadge(2000).key).toMatch(/useful/);
    expect(harvestBadge(5000).key).toMatch(/significant/);
    expect(harvestBadge(NaN).key).toMatch(/unknown/);
});

// ── dec coercion ──────────────────────────────────────────────────

test('dec coerces string / number / null safely', () => {
    expect(dec('123.45')).toBe(123.45);
    expect(dec(7)).toBe(7);
    expect(dec(null)).toBe(0);
    expect(dec('abc')).toBe(0);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: mixed has 3 losers with no wash, total ≈ $2500', () => {
    const losers = makeDemoLosers('mixed');
    const buys = makeDemoRecentBuys('mixed', '2026-12-15');
    const r = localSuggest(losers, buys, '2026-12-15', 0, false);
    expect(r.candidates.length).toBe(3);
    expect(r.total_available_loss).toBeCloseTo(2500, 9);
    expect(r.candidates.every(c => !c.wash_sale_risk)).toBe(true);
});

test('demos: wash-sale → flagged', () => {
    const losers = makeDemoLosers('wash-sale');
    const buys = makeDemoRecentBuys('wash-sale', '2026-12-15');
    const r = localSuggest(losers, buys, '2026-12-15', 0, false);
    expect(r.candidates[0].wash_sale_risk).toBe(true);
});

test('demos: exceeds-3k with $2500 YTD → cap flagged', () => {
    const losers = makeDemoLosers('exceeds-3k');
    const r = localSuggest(losers, [], '2026-12-15', 2500, false);
    expect(r.candidates[0].exceeds_3k_cap).toBe(true);
});

test('demos: winners-only → 0 candidates', () => {
    const r = localSuggest(makeDemoLosers('winners-only'), [], '2026-12-15', 0, false);
    expect(r.candidates).toEqual([]);
});

test('demos: big-three sorts BIG → MID → TINY by loss size', () => {
    const r = localSuggest(makeDemoLosers('big-three'), [], '2026-12-15', 0, false);
    expect(r.candidates.map(c => c.symbol)).toEqual(['BIG', 'MID', 'TINY']);
});

// ── todayIso ──────────────────────────────────────────────────────

test('todayIso returns YYYY-MM-DD', () => {
    expect(isValidDate(todayIso())).toBe(true);
});

// ── formatters ────────────────────────────────────────────────────

test('fmt helpers + non-finite guards', () => {
    expect(fmtUSD(1234)).toBe('$1234.00');
    expect(fmtUSDSigned(-100)).toBe('-$100.00');
    expect(fmtNum(0.12345, 2)).toBe('0.12');
    expect(fmtBool(true)).toBe('✓');
    expect(fmtBool(false)).toBe('·');
    expect(fmtUSD(NaN)).toBe('—');
});
