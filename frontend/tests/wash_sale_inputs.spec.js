// Wash-sale helpers: parsers, dates, validator, body shape,
// localDetectHits Rust-mirror parity, badges, demos.

import { test, expect } from 'vitest';
import {
    parseClosingBlob, parseOpeningBlob, validateInputs, buildBody, dec,
    isValidDate, daysBetween, hashStr, makeDeterministicUuid,
    localDetectHits, localTotalDisallowed, washBadge, totalRealizedLoss,
    makeDemoClosings, makeDemoOpenings,
    fmtUSD, fmtUSDSigned, fmtDays, fmtPct, fmtNum, shortUuid,
} from '../js/_wash_sale_inputs.js';

const close = (sym, date, pnl, qty) => ({
    trade_id: makeDeterministicUuid(hashStr(`${sym}|${date}`)),
    symbol: sym, closed_at: date, net_pnl: pnl, qty,
});
const open = (sym, date, qty) => ({
    execution_id: makeDeterministicUuid(hashStr(`o|${sym}|${date}`) + 1),
    symbol: sym, executed_at: date, qty,
});

// ── parsers ───────────────────────────────────────────────────────

test('parseClosingBlob: 4 tokens; upcased symbol; signed pnl', () => {
    const r = parseClosingBlob('aapl 2026-06-01 -500 100\n# note\nTSLA 2026-06-15 -300 50');
    expect(r.errors).toEqual([]);
    expect(r.rows.length).toBe(2);
    expect(r.rows[0].symbol).toBe('AAPL');
    expect(r.rows[0].net_pnl).toBe(-500);
    expect(r.rows[0].qty).toBe(100);
});

test('parseClosingBlob: bad date rejected', () => {
    expect(parseClosingBlob('AAPL 2026/06/01 -500 100').errors[0].message).toMatch(/closed_at/);
});

test('parseClosingBlob: non-positive qty rejected', () => {
    expect(parseClosingBlob('AAPL 2026-06-01 -500 0').errors[0].message).toMatch(/qty/);
});

test('parseClosingBlob: accepts positive pnl (winning trade, won\'t flag)', () => {
    expect(parseClosingBlob('AAPL 2026-06-01 500 100').errors).toEqual([]);
});

test('parseOpeningBlob: 3 tokens; upcased symbol', () => {
    const r = parseOpeningBlob('AAPL 2026-06-15 100');
    expect(r.errors).toEqual([]);
    expect(r.rows[0]).toMatchObject({ symbol: 'AAPL', executed_at: '2026-06-15', qty: 100 });
});

test('parseOpeningBlob: non-positive qty rejected', () => {
    expect(parseOpeningBlob('AAPL 2026-06-15 0').errors[0].message).toMatch(/qty/);
});

test('parsers: non-string returns 1 error', () => {
    expect(parseClosingBlob(null).errors.length).toBe(1);
    expect(parseOpeningBlob(null).errors.length).toBe(1);
});

test('parseClosingBlob: assigns deterministic UUID per row', () => {
    const a = parseClosingBlob('AAPL 2026-06-01 -500 100');
    const b = parseClosingBlob('AAPL 2026-06-01 -500 100');
    expect(a.rows[0].trade_id).toBe(b.rows[0].trade_id);
});

// ── date helpers ──────────────────────────────────────────────────

test('isValidDate strict YYYY-MM-DD', () => {
    expect(isValidDate('2026-06-01')).toBe(true);
    expect(isValidDate('2026-13-01')).toBe(false);
    expect(isValidDate('2026-02-30')).toBe(false);
});

test('daysBetween: whole days, signed', () => {
    expect(daysBetween('2026-06-01', '2026-06-15')).toBe(14);
    expect(daysBetween('2026-06-15', '2026-06-01')).toBe(-14);
});

test('daysBetween: invalid date → Infinity (never matches window)', () => {
    expect(daysBetween('bogus', '2026-06-01')).toBe(Infinity);
});

// ── UUID helpers ──────────────────────────────────────────────────

test('hashStr: deterministic, non-zero for non-empty input', () => {
    expect(hashStr('abc')).toBe(hashStr('abc'));
    expect(hashStr('abc')).not.toBe(hashStr('abd'));
});

test('makeDeterministicUuid: matches 8-4-4-4-12 hex pattern', () => {
    expect(makeDeterministicUuid(1)).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/);
});

// ── validator / buildBody ─────────────────────────────────────────

test('validate accepts arrays', () => {
    expect(validateInputs([], [])).toBe(null);
});

test('validate rejects non-array', () => {
    expect(validateInputs(null, [])).toMatch(/closings/);
    expect(validateInputs([], null)).toMatch(/openings/);
});

test('buildBody: Decimal fields stringified on wire', () => {
    const body = buildBody([close('AAPL', '2026-06-01', -500, 100)],
                           [open('AAPL', '2026-06-15', 100)]);
    expect(body.closings[0].net_pnl).toBe('-500');
    expect(body.closings[0].qty).toBe('100');
    expect(body.openings[0].qty).toBe('100');
});

// ── localDetectHits parity (one test per Rust property) ───────────

test('local: winning trade never flags', () => {
    const hits = localDetectHits([close('AAPL', '2026-06-01', 500, 100)],
                                  [open('AAPL', '2026-06-15', 100)]);
    expect(hits).toEqual([]);
});

test('local: replacement buy inside window flags with correct fields', () => {
    const c = close('AAPL', '2026-06-01', -500, 100);
    const o = open('AAPL', '2026-06-15', 100);
    const hits = localDetectHits([c], [o]);
    expect(hits.length).toBe(1);
    expect(hits[0].days_offset).toBe(14);
    expect(hits[0].loss_amount).toBe(500);
    expect(hits[0].disallowed_loss_estimate).toBe(500);
    expect(hits[0].symbol).toBe('AAPL');
});

test('local: replacement buy outside +30 window does not flag', () => {
    const hits = localDetectHits(
        [close('AAPL', '2026-06-01', -500, 100)],
        [open('AAPL', '2026-07-05', 100)]);  // +34 days
    expect(hits).toEqual([]);
});

test('local: replacement buy BEFORE loss within ±30 days also flags', () => {
    const hits = localDetectHits(
        [close('AAPL', '2026-06-30', -500, 100)],
        [open('AAPL', '2026-06-10', 100)]);  // -20 days
    expect(hits.length).toBe(1);
    expect(hits[0].days_offset).toBe(-20);
});

test('local: different symbol does not flag', () => {
    const hits = localDetectHits(
        [close('AAPL', '2026-06-01', -500, 100)],
        [open('TSLA', '2026-06-15', 100)]);
    expect(hits).toEqual([]);
});

test('local: partial replacement → proportional disallowed', () => {
    // Sold 100, bought back 30 → 30% disallowed.
    const hits = localDetectHits(
        [close('AAPL', '2026-06-01', -500, 100)],
        [open('AAPL', '2026-06-05', 30)]);
    expect(hits[0].disallowed_loss_estimate).toBeCloseTo(150, 9);
});

test('local: replacement > close qty caps at full loss', () => {
    const hits = localDetectHits(
        [close('AAPL', '2026-06-01', -500, 100)],
        [open('AAPL', '2026-06-05', 500)]);
    expect(hits[0].disallowed_loss_estimate).toBe(500);
});

test('local: boundary at EXACTLY 30 days = INSIDE (≤)', () => {
    const hits = localDetectHits(
        [close('AAPL', '2026-06-01', -500, 100)],
        [open('AAPL', '2026-07-01', 100)]);  // +30 days
    expect(hits.length).toBe(1);
});

test('local: 31 days = outside', () => {
    const hits = localDetectHits(
        [close('AAPL', '2026-06-01', -500, 100)],
        [open('AAPL', '2026-07-02', 100)]);
    expect(hits.length).toBe(0);
});

test('local: two replacement buys → two hits, total disallowed sums', () => {
    const hits = localDetectHits(
        [close('AAPL', '2026-06-01', -500, 100)],
        [open('AAPL', '2026-06-05', 100), open('AAPL', '2026-06-20', 100)]);
    expect(hits.length).toBe(2);
    expect(localTotalDisallowed(hits)).toBe(1000);
});

test('local: empty inputs returns empty', () => {
    expect(localDetectHits([], [])).toEqual([]);
    expect(localTotalDisallowed([])).toBe(0);
});

test('local: closing with qty=0 → 0 qty_ratio → 0 disallowed', () => {
    const hits = localDetectHits(
        [{ ...close('AAPL', '2026-06-01', -500, 100), qty: 0 }],
        [open('AAPL', '2026-06-05', 100)]);
    expect(hits[0].disallowed_loss_estimate).toBe(0);
});

// ── totalRealizedLoss / washBadge ─────────────────────────────────

test('totalRealizedLoss: sums absolute losses only', () => {
    expect(totalRealizedLoss([
        close('A', '2026-06-01', -500, 100),
        close('B', '2026-06-15', 300, 50),
        close('C', '2026-06-20', -200, 25),
    ])).toBe(700);
});

test('washBadge: clean (0) / minor (< 25%) / material (< 75%) / severe (≥ 75%)', () => {
    expect(washBadge(0,   1000).key).toMatch(/clean/);
    expect(washBadge(100, 1000).key).toMatch(/minor/);
    expect(washBadge(400, 1000).key).toMatch(/material/);
    expect(washBadge(800, 1000).key).toMatch(/severe/);
    expect(washBadge(NaN, 1000).key).toMatch(/unknown/);
});

// ── demos ─────────────────────────────────────────────────────────

test('demo classic-trap: 1 hit at +14 days, full loss disallowed', () => {
    const hits = localDetectHits(makeDemoClosings('classic-trap'),
                                  makeDemoOpenings('classic-trap'));
    expect(hits.length).toBe(1);
    expect(hits[0].days_offset).toBe(14);
    expect(hits[0].disallowed_loss_estimate).toBe(500);
});

test('demo winning: never flags', () => {
    expect(localDetectHits(makeDemoClosings('winning-trade-no-flag'),
                            makeDemoOpenings('winning-trade-no-flag'))).toEqual([]);
});

test('demo outside-window: 0 hits', () => {
    expect(localDetectHits(makeDemoClosings('outside-window'),
                            makeDemoOpenings('outside-window'))).toEqual([]);
});

test('demo partial-replacement: 30% disallowed', () => {
    const hits = localDetectHits(makeDemoClosings('partial-replacement'),
                                  makeDemoOpenings('partial-replacement'));
    expect(hits[0].disallowed_loss_estimate).toBeCloseTo(150, 9);
});

test('demo multi-hit: 2 hits, total 1000 disallowed', () => {
    const hits = localDetectHits(makeDemoClosings('multi-hit'),
                                  makeDemoOpenings('multi-hit'));
    expect(hits.length).toBe(2);
    expect(localTotalDisallowed(hits)).toBe(1000);
});

test('demo mixed: only AAPL flagged (TSLA outside window, NVDA winner)', () => {
    const hits = localDetectHits(makeDemoClosings('mixed'),
                                  makeDemoOpenings('mixed'));
    expect(hits.every(h => h.symbol === 'AAPL')).toBe(true);
    expect(hits.length).toBeGreaterThanOrEqual(1);
});

// ── dec / formatters / shortUuid ──────────────────────────────────

test('dec safe coercions', () => {
    expect(dec('123.45')).toBe(123.45);
    expect(dec(null)).toBe(0);
    expect(dec('abc')).toBe(0);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtUSD(1234)).toBe('$1234.00');
    expect(fmtUSDSigned(-100)).toBe('-$100.00');
    expect(fmtDays(14)).toBe('+14d');
    expect(fmtDays(-14)).toBe('-14d');
    expect(fmtPct(0.30)).toBe('30.0%');
    expect(fmtNum(0.1234, 2)).toBe('0.12');
    expect(fmtUSD(NaN)).toBe('—');
});

test('shortUuid takes first 8 chars', () => {
    expect(shortUuid('00000001-0000-0000-0000-000000000000')).toBe('00000001');
    expect(shortUuid(null)).toBe('—');
});
