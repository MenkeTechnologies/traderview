// Intraday Heatmap pure helpers: timestamp normalizer, trade parser,
// validator, body shape, grid layout, heat-color picker, demo
// invariants, formatters.

import { test, expect } from 'vitest';
import {
    parseTradeBlob, normalizeTimestamp, validateInputs, buildBody,
    gridify, heatStyleClass, makeDemoTrades,
    fmtUSD, fmtPct,
} from '../js/_intraday_heatmap_inputs.js';

// ── normalizeTimestamp ────────────────────────────────────────────

test('normalizeTimestamp accepts HH:MM and anchors to epoch', () => {
    expect(normalizeTimestamp('09:30')).toBe('2024-01-01T09:30:00Z');
    expect(normalizeTimestamp('14:00:15')).toBe('2024-01-01T14:00:15Z');
});

test('normalizeTimestamp rejects out-of-range HH:MM', () => {
    expect(normalizeTimestamp('24:00')).toBe(null);
    expect(normalizeTimestamp('09:60')).toBe(null);
    expect(normalizeTimestamp('09:30:99')).toBe(null);
});

test('normalizeTimestamp accepts full ISO 8601 and round-trips it', () => {
    const iso = normalizeTimestamp('2024-03-15T10:30:00Z');
    expect(iso).toBe('2024-03-15T10:30:00.000Z');
});

test('normalizeTimestamp returns null on garbage / non-string', () => {
    expect(normalizeTimestamp('not-a-date')).toBe(null);
    expect(normalizeTimestamp(null)).toBe(null);
    expect(normalizeTimestamp(42)).toBe(null);
    expect(normalizeTimestamp('')).toBe(null);
});

// ── parseTradeBlob ────────────────────────────────────────────────

test('parseTradeBlob accepts mixed HH:MM and ISO 8601 timestamps', () => {
    const r = parseTradeBlob('09:30 125.50\n2024-03-15T14:00:00Z, -42');
    expect(r.errors).toEqual([]);
    expect(r.trades.length).toBe(2);
    expect(r.trades[0].when).toBe('2024-01-01T09:30:00Z');
    expect(r.trades[0].pnl).toBe(125.50);
    expect(r.trades[1].pnl).toBe(-42);
});

test('parseTradeBlob skips # comments + blanks', () => {
    const r = parseTradeBlob('# header\n\n09:30 100\n# tail');
    expect(r.errors).toEqual([]);
    expect(r.trades.length).toBe(1);
});

test('parseTradeBlob rejects wrong token count', () => {
    const r = parseTradeBlob('09:30');
    expect(r.errors[0].message).toMatch(/expected 2 tokens/);
});

test('parseTradeBlob rejects bad timestamp', () => {
    const r = parseTradeBlob('25:00 100');
    expect(r.errors[0].message).toMatch(/bad timestamp/);
});

test('parseTradeBlob rejects non-finite pnl', () => {
    const r = parseTradeBlob('09:30 abc');
    expect(r.errors[0].message).toMatch(/pnl must be finite/);
});

test('parseTradeBlob accepts negative pnl', () => {
    const r = parseTradeBlob('09:30 -500');
    expect(r.trades[0].pnl).toBe(-500);
});

test('parseTradeBlob returns error on non-string input', () => {
    const r = parseTradeBlob(undefined);
    expect(r.trades).toEqual([]);
    expect(r.errors.length).toBe(1);
});

// ── validateInputs / buildBody ────────────────────────────────────

test('validate accepts ≥1 trade, rejects empty', () => {
    expect(validateInputs([{ when: '2024-01-01T09:30:00Z', pnl: 1 }])).toBe(null);
    expect(validateInputs([])).toMatch(/at least 1 trade/);
});

test('buildBody emits backend IntradayHeatmapBody shape', () => {
    const t = [{ when: '2024-01-01T09:30:00Z', pnl: 1 }];
    expect(buildBody(t)).toEqual({ trades: t });
});

// ── gridify ───────────────────────────────────────────────────────

test('gridify produces 24×4 grid + tracks max-abs PnL', () => {
    const buckets = [
        { hour: 9,  minute: 30, total_pnl: 100, trade_count: 1, avg_pnl: 100, win_count: 1, win_rate: 1, label: '09:30' },
        { hour: 11, minute: 45, total_pnl: -250, trade_count: 2, avg_pnl: -125, win_count: 0, win_rate: 0, label: '11:45' },
    ];
    const { grid, maxAbs } = gridify(buckets);
    expect(grid.length).toBe(24);
    expect(grid[0].length).toBe(4);
    expect(grid[9][2]).toEqual(buckets[0]);
    expect(grid[11][3]).toEqual(buckets[1]);
    expect(grid[10][0]).toBe(null);
    expect(maxAbs).toBe(250);
});

test('gridify ignores buckets with bad hour or non-quarter minute', () => {
    const { grid, maxAbs } = gridify([
        { hour: 25, minute: 0, total_pnl: 100, trade_count: 1 },
        { hour: 10, minute: 13, total_pnl: 50, trade_count: 1 },
    ]);
    expect(grid.flat().every(c => c === null)).toBe(true);
    expect(maxAbs).toBe(0);
});

test('gridify handles null/missing input', () => {
    const { grid, maxAbs } = gridify(null);
    expect(grid.length).toBe(24);
    expect(maxAbs).toBe(0);
});

// ── heatStyleClass ────────────────────────────────────────────────

test('heatStyleClass tiers by |pnl|/maxAbs (boundary cuts at 0.25/0.50/0.75)', () => {
    expect(heatStyleClass(40, 200)).toBe('heat-pos-1');     // 20%  < 25
    expect(heatStyleClass(70, 200)).toBe('heat-pos-2');     // 35%  < 50
    expect(heatStyleClass(130, 200)).toBe('heat-pos-3');    // 65%  < 75
    expect(heatStyleClass(180, 200)).toBe('heat-pos-4');    // 90%  ≥ 75
});

test('heatStyleClass uses neg tiers for negative pnl', () => {
    expect(heatStyleClass(-40, 200)).toBe('heat-neg-1');
    expect(heatStyleClass(-180, 200)).toBe('heat-neg-4');
});

test('heatStyleClass returns empty for zero/NaN/zero-max-abs', () => {
    expect(heatStyleClass(0, 100)).toBe('heat-empty');
    expect(heatStyleClass(NaN, 100)).toBe('heat-empty');
    expect(heatStyleClass(50, 0)).toBe('heat-empty');
});

// ── makeDemoTrades ────────────────────────────────────────────────

test('makeDemoTrades deterministic + exactly 200 trades', () => {
    const a = makeDemoTrades(42);
    const b = makeDemoTrades(42);
    expect(a).toEqual(b);
    expect(a.length).toBe(200);
});

test('makeDemoTrades has 09:30 window net positive and 11:30 window net negative', () => {
    const t = makeDemoTrades(1);
    const momo = t.filter(x => {
        const m = /T(\d{2}):(\d{2})/.exec(x.when);
        if (!m) return false;
        const h = parseInt(m[1], 10);
        const mn = parseInt(m[2], 10);
        return h === 9 && mn >= 30;
    });
    const chop = t.filter(x => {
        const m = /T(\d{2}):(\d{2})/.exec(x.when);
        if (!m) return false;
        const h = parseInt(m[1], 10);
        const mn = parseInt(m[2], 10);
        return h === 11 && mn >= 30;
    });
    const momoSum = momo.reduce((a, x) => a + x.pnl, 0);
    const chopSum = chop.reduce((a, x) => a + x.pnl, 0);
    expect(momoSum).toBeGreaterThan(0);
    expect(chopSum).toBeLessThan(0);
});

// ── formatters ─────────────────────────────────────────────────────

test('fmtUSD + fmtPct', () => {
    expect(fmtUSD(125.5)).toBe('$125.50');
    expect(fmtUSD(-42)).toBe('-$42.00');
    expect(fmtUSD(NaN)).toBe('—');
    expect(fmtPct(0.55)).toBe('55%');
    expect(fmtPct(NaN)).toBe('—');
});
