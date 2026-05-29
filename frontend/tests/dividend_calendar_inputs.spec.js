// Dividend Calendar pure helpers: symbol parser, extractor against
// Yahoo-shaped payloads, date helpers, horizon filter.

import { test, expect } from 'vitest';
import {
    parseSymbolList, extractDividend, daysBetween,
    sortByExDate, filterByHorizon,
    fmtDate, fmtYield, fmtAmount,
} from '../js/_dividend_calendar_inputs.js';

// ── parseSymbolList ────────────────────────────────────────────────

test('parseSymbolList uppercases and dedupes', () => {
    expect(parseSymbolList('aapl\nMsft, AAPL\ngoog')).toEqual(['AAPL', 'MSFT', 'GOOG']);
});

test('parseSymbolList handles whitespace OR comma separators', () => {
    expect(parseSymbolList('KO PG, JNJ\nXOM')).toEqual(['KO', 'PG', 'JNJ', 'XOM']);
});

test('parseSymbolList accepts symbols with `.` and `-`', () => {
    expect(parseSymbolList('BRK.B RDS-A')).toEqual(['BRK.B', 'RDS-A']);
});

test('parseSymbolList strips # comment lines', () => {
    expect(parseSymbolList('# header\n\nAAPL\n# another')).toEqual(['AAPL']);
});

test('parseSymbolList rejects garbage tokens', () => {
    expect(parseSymbolList('AAPL $$$ MSFT@@')).toEqual(['AAPL']);
});

test('parseSymbolList returns [] for non-string input', () => {
    expect(parseSymbolList(null)).toEqual([]);
    expect(parseSymbolList(undefined)).toEqual([]);
    expect(parseSymbolList(42)).toEqual([]);
});

// ── extractDividend ────────────────────────────────────────────────

test('extractDividend returns null for missing payload', () => {
    expect(extractDividend('AAPL', null)).toBe(null);
    expect(extractDividend('AAPL', {})).toBe(null);
});

test('extractDividend pulls Yahoo summaryDetail + calendarEvents fields', () => {
    const payload = {
        summaryDetail: {
            dividendYield: { raw: 0.0250, fmt: '2.50%' },
            dividendRate:  { raw: 2.50,   fmt: '2.50' },
            payoutRatio:   { raw: 0.45,   fmt: '45.00%' },
            lastDividendValue: { raw: 0.625, fmt: '0.625' },
            lastDividendDate:  { raw: 1700000000, fmt: '2023-11-14' },
        },
        calendarEvents: {
            exDividendDate: { raw: 1733000000, fmt: '2024-12-01' },
            dividendDate:   { raw: 1734000000, fmt: '2024-12-13' },
        },
    };
    const d = extractDividend('AAPL', payload);
    expect(d.symbol).toBe('AAPL');
    expect(d.yield).toBeCloseTo(0.0250, 6);
    expect(d.amount).toBeCloseTo(2.50, 6);
    expect(d.payout_ratio).toBeCloseTo(0.45, 6);
    expect(d.ex_date).toBeInstanceOf(Date);
    expect(d.pay_date).toBeInstanceOf(Date);
    expect(d.last_div_amount).toBeCloseTo(0.625, 6);
});

test('extractDividend falls back to summaryDetail.exDividendDate when calendarEvents missing', () => {
    const d = extractDividend('XYZ', {
        summaryDetail: {
            exDividendDate: { raw: 1733000000 },
            dividendYield: { raw: 0.02 },
        },
    });
    expect(d.ex_date).toBeInstanceOf(Date);
});

test('extractDividend returns null when no dividend signal whatsoever', () => {
    // Non-payer like an ETF or growth stock — all Yahoo fields are null.
    expect(extractDividend('GOOGL', {
        summaryDetail: { dividendYield: { raw: null }, dividendRate: { raw: null } },
        calendarEvents: {},
    })).toBe(null);
});

test('extractDividend tolerates partial data', () => {
    // Has a yield, no ex-date, no amount. Should still surface.
    const d = extractDividend('PP', {
        summaryDetail: { dividendYield: { raw: 0.04 } },
    });
    expect(d).not.toBe(null);
    expect(d.yield).toBe(0.04);
    expect(d.ex_date).toBe(null);
    expect(d.amount).toBe(null);
});

// ── daysBetween ────────────────────────────────────────────────────

test('daysBetween computes whole-day offsets', () => {
    const a = new Date(2025, 0, 1);
    const b = new Date(2025, 0, 11);
    expect(daysBetween(a, b)).toBe(10);
});

test('daysBetween returns negative for past dates', () => {
    const a = new Date(2025, 0, 11);
    const b = new Date(2025, 0, 1);
    expect(daysBetween(a, b)).toBe(-10);
});

test('daysBetween returns null on non-Date inputs', () => {
    expect(daysBetween(null, new Date())).toBe(null);
    expect(daysBetween(new Date(), 'not a date')).toBe(null);
});

// ── sortByExDate ───────────────────────────────────────────────────

test('sortByExDate puts earliest ex-dates first', () => {
    const rows = [
        { symbol: 'B', ex_date: new Date(2025, 5, 1) },
        { symbol: 'A', ex_date: new Date(2025, 1, 1) },
        { symbol: 'C', ex_date: new Date(2025, 8, 1) },
    ];
    expect(sortByExDate(rows).map(r => r.symbol)).toEqual(['A', 'B', 'C']);
});

test('sortByExDate puts null ex-dates at the end', () => {
    const rows = [
        { symbol: 'NULL', ex_date: null },
        { symbol: 'A',    ex_date: new Date(2025, 1, 1) },
    ];
    expect(sortByExDate(rows).map(r => r.symbol)).toEqual(['A', 'NULL']);
});

test('sortByExDate does not mutate input', () => {
    const rows = [
        { symbol: 'B', ex_date: new Date(2025, 5, 1) },
        { symbol: 'A', ex_date: new Date(2025, 1, 1) },
    ];
    sortByExDate(rows);
    expect(rows[0].symbol).toBe('B');
});

// ── filterByHorizon ────────────────────────────────────────────────

test('filterByHorizon keeps rows within [today, today+horizon]', () => {
    const today = new Date(2025, 0, 1);
    const rows = [
        { symbol: 'PAST',    ex_date: new Date(2024, 11, 31) },
        { symbol: 'TODAY',   ex_date: new Date(2025, 0, 1) },
        { symbol: 'NEAR',    ex_date: new Date(2025, 0, 10) },
        { symbol: 'EDGE',    ex_date: new Date(2025, 0, 30) },
        { symbol: 'FAR',     ex_date: new Date(2025, 2, 1) },
    ];
    const out = filterByHorizon(rows, today, 30).map(r => r.symbol);
    expect(out).toContain('TODAY');
    expect(out).toContain('NEAR');
    expect(out).toContain('EDGE');
    expect(out).not.toContain('PAST');
    expect(out).not.toContain('FAR');
});

test('filterByHorizon drops null ex-dates', () => {
    const today = new Date(2025, 0, 1);
    const rows = [{ symbol: 'NULL', ex_date: null }];
    expect(filterByHorizon(rows, today, 30)).toEqual([]);
});

// ── formatters ─────────────────────────────────────────────────────

test('fmtDate emits YYYY-MM-DD', () => {
    expect(fmtDate(new Date(2025, 0, 15))).toBe('2025-01-15');
});

test('fmtDate returns "—" for null / NaN', () => {
    expect(fmtDate(null)).toBe('—');
    expect(fmtDate(new Date('not a date'))).toBe('—');
});

test('fmtYield formats decimal as 2-decimal percent', () => {
    expect(fmtYield(0.0250)).toBe('2.50%');
});

test('fmtYield returns "—" on non-finite', () => {
    expect(fmtYield(null)).toBe('—');
    expect(fmtYield(NaN)).toBe('—');
});

test('fmtAmount emits $ with 4 decimals', () => {
    expect(fmtAmount(2.5)).toBe('$2.5000');
});

test('fmtAmount returns "—" on non-finite', () => {
    expect(fmtAmount(null)).toBe('—');
});
