// DOW × Hour heatmap helpers: parser (two formats), validator, body
// (synthesized Trade per row), localBuild Rust-mirror, heat classes,
// demos, formatters.

import { test, expect } from 'vitest';
import {
    DOW_LABELS, parseTradeBlob, validateInputs, buildBody,
    localBuild, emptyCells, dec, dowFromIsoDate, isValidDate,
    maxCellAbs, heatClass, extremeCells, winRate, makeDemoRows,
    makeDeterministicUuid,
    fmtUSD, fmtUSDSigned, fmtPct, fmtHour,
} from '../js/_heatmap_dow_hour_inputs.js';

// ── constants ─────────────────────────────────────────────────────

test('DOW_LABELS = Sun..Sat (matches Rust 0=Sun)', () => {
    expect(DOW_LABELS).toEqual(['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat']);
});

// ── parser (two input formats) ────────────────────────────────────

test('parseTradeBlob: spaced form (date hour pnl)', () => {
    const r = parseTradeBlob('2026-05-25 9 100\n2026-05-26 14 -50');
    expect(r.errors).toEqual([]);
    expect(r.rows).toEqual([
        { date: '2026-05-25', hour: 9,  net_pnl: 100 },
        { date: '2026-05-26', hour: 14, net_pnl: -50 },
    ]);
});

test('parseTradeBlob: iso form (date T hh:mm pnl)', () => {
    const r = parseTradeBlob('2026-05-25T09:30 100\n2026-05-26T14:15 -50');
    expect(r.errors).toEqual([]);
    expect(r.rows).toEqual([
        { date: '2026-05-25', hour: 9,  net_pnl: 100 },
        { date: '2026-05-26', hour: 14, net_pnl: -50 },
    ]);
});

test('parseTradeBlob: comments stripped', () => {
    expect(parseTradeBlob('2026-05-25 9 100  # note\n# pure').rows).toEqual([
        { date: '2026-05-25', hour: 9, net_pnl: 100 },
    ]);
});

test('parseTradeBlob: rejects bad date / bad hour / non-finite pnl / wrong token count', () => {
    expect(parseTradeBlob('2026/05/25 9 100').errors[0].message).toMatch(/date/);
    expect(parseTradeBlob('2026-05-25 25 100').errors[0].message).toMatch(/hour/);
    expect(parseTradeBlob('2026-05-25 9 abc').errors[0].message).toMatch(/finite/);
    expect(parseTradeBlob('2026-05-25 9').errors[0].message).toMatch(/expected/);
});

test('parseTradeBlob: hour=0 and hour=23 boundaries accepted', () => {
    expect(parseTradeBlob('2026-05-25 0 100\n2026-05-25 23 100').errors).toEqual([]);
});

test('parseTradeBlob: non-string returns 1 error', () => {
    expect(parseTradeBlob(null).errors.length).toBe(1);
});

// ── date helpers ──────────────────────────────────────────────────

test('isValidDate: strict + rejects bogus calendar', () => {
    expect(isValidDate('2026-05-25')).toBe(true);
    expect(isValidDate('2026-13-01')).toBe(false);
    expect(isValidDate('2026-02-30')).toBe(false);
});

test('dowFromIsoDate: 0 = Sunday, 1 = Monday, etc.', () => {
    // 2026-05-25 = Monday; 2026-05-31 = Sunday.
    expect(dowFromIsoDate('2026-05-25')).toBe(1);
    expect(dowFromIsoDate('2026-05-31')).toBe(0);
    expect(dowFromIsoDate('2026-05-30')).toBe(6);  // Saturday
});

test('dowFromIsoDate: bad date → -1', () => {
    expect(dowFromIsoDate('bogus')).toBe(-1);
});

// ── validator / buildBody ─────────────────────────────────────────

test('validate accepts arrays', () => {
    expect(validateInputs([])).toBe(null);
});

test('validate rejects non-array', () => {
    expect(validateInputs(null)).toMatch(/rows/);
});

test('buildBody: synthesizes valid Trade per row (Decimal-as-string)', () => {
    const body = buildBody([{ date: '2026-05-25', hour: 9, net_pnl: 100 }]);
    expect(body.trades.length).toBe(1);
    const tr = body.trades[0];
    expect(tr.status).toBe('closed');
    expect(tr.opened_at).toBe('2026-05-25T09:00:00Z');
    expect(tr.net_pnl).toBe('100');
});

test('makeDeterministicUuid: stable 8-4-4-4-12 hex shape', () => {
    expect(makeDeterministicUuid(1)).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/);
});

// ── emptyCells ────────────────────────────────────────────────────

test('emptyCells: 7×24 grid of zero cells', () => {
    const grid = emptyCells();
    expect(grid.length).toBe(7);
    expect(grid[0].length).toBe(24);
    for (const row of grid) for (const c of row) {
        expect(c).toEqual({ trades: 0, wins: 0, net_pnl: 0 });
    }
});

// ── localBuild parity (one test per Rust property) ───────────────

test('local: empty → zero-filled grid', () => {
    const h = localBuild([]);
    expect(h.total_trades).toBe(0);
    expect(h.cells.length).toBe(7);
    expect(h.cells.every(row => row.every(c => c.trades === 0))).toBe(true);
});

test('local: single trade lands in correct (dow, hour) cell', () => {
    // 2026-05-26 is Tuesday (weekday 2 from Sunday).
    const h = localBuild([{ date: '2026-05-26', hour: 10, net_pnl: 100 }]);
    expect(h.cells[2][10].trades).toBe(1);
    expect(h.cells[2][10].wins).toBe(1);
    expect(h.cells[2][10].net_pnl).toBe(100);
    expect(h.cells[2][11].trades).toBe(0);
    expect(h.cells[1][10].trades).toBe(0);
});

test('local: multiple trades in same cell aggregate', () => {
    const h = localBuild([
        { date: '2026-05-26', hour: 10, net_pnl: 100 },
        { date: '2026-05-26', hour: 10, net_pnl: -50 },
        { date: '2026-05-26', hour: 10, net_pnl: 200 },
    ]);
    expect(h.cells[2][10].trades).toBe(3);
    expect(h.cells[2][10].wins).toBe(2);
    expect(h.cells[2][10].net_pnl).toBe(250);
    expect(winRate(h.cells[2][10])).toBeCloseTo(2 / 3, 9);
});

test('local: different days don\'t bleed', () => {
    const h = localBuild([
        { date: '2026-05-26', hour: 10, net_pnl: 100 },  // Tue
        { date: '2026-05-27', hour: 10, net_pnl: -50 },  // Wed
    ]);
    expect(h.cells[2][10].net_pnl).toBe(100);
    expect(h.cells[3][10].net_pnl).toBe(-50);
    expect(h.total_pnl).toBe(50);
});

test('local: weekend trades land at dow 0 (Sun) + 6 (Sat)', () => {
    const h = localBuild([
        { date: '2026-05-30', hour: 10, net_pnl: 100 },  // Sat
        { date: '2026-05-31', hour: 10, net_pnl: 100 },  // Sun
    ]);
    expect(h.cells[6][10].trades).toBe(1);
    expect(h.cells[0][10].trades).toBe(1);
});

test('local: net_pnl=0 counted in trades but NOT wins', () => {
    const h = localBuild([{ date: '2026-05-26', hour: 10, net_pnl: 0 }]);
    expect(h.cells[2][10].trades).toBe(1);
    expect(h.cells[2][10].wins).toBe(0);
});

// ── heat classes ──────────────────────────────────────────────────

test('heatClass: 4-tier scaling by intensity (pos + neg)', () => {
    expect(heatClass(20, 100)).toBe('heat-pos-1');
    expect(heatClass(40, 100)).toBe('heat-pos-2');
    expect(heatClass(60, 100)).toBe('heat-pos-3');
    expect(heatClass(90, 100)).toBe('heat-pos-4');
    expect(heatClass(-40, 100)).toBe('heat-neg-2');
});

test('heatClass: 0 / NaN / maxAbs=0 → empty', () => {
    expect(heatClass(0, 100)).toBe('heat-empty');
    expect(heatClass(NaN, 100)).toBe('heat-empty');
    expect(heatClass(50, 0)).toBe('heat-empty');
});

// ── maxCellAbs + extremeCells ─────────────────────────────────────

test('maxCellAbs: returns largest |net_pnl| across all cells', () => {
    const h = localBuild([
        { date: '2026-05-26', hour: 9,  net_pnl: 100 },
        { date: '2026-05-27', hour: 14, net_pnl: -250 },
    ]);
    expect(maxCellAbs(h)).toBe(250);
});

test('extremeCells: picks best + worst by net_pnl across grid', () => {
    const h = localBuild([
        { date: '2026-05-25', hour: 9,  net_pnl: 100 },  // Mon
        { date: '2026-05-26', hour: 11, net_pnl: -250 }, // Tue
        { date: '2026-05-27', hour: 14, net_pnl: 500 },  // Wed
    ]);
    const { best, worst } = extremeCells(h);
    expect(best).toEqual({ dow: 3, hour: 14, net_pnl: 500, trades: 1 });
    expect(worst).toEqual({ dow: 2, hour: 11, net_pnl: -250, trades: 1 });
});

test('extremeCells: empty grid → null pair', () => {
    expect(extremeCells({ cells: emptyCells() })).toEqual({ best: null, worst: null });
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset emits ≥ 3 valid rows', () => {
    for (const k of ['mixed', 'monday-disaster', 'sweet-spot', 'weekend-crypto', 'all-week']) {
        const rows = makeDemoRows(k);
        expect(rows.length).toBeGreaterThanOrEqual(3);
        for (const r of rows) {
            expect(isValidDate(r.date)).toBe(true);
            expect(Number.isInteger(r.hour)).toBe(true);
            expect(Number.isFinite(r.net_pnl)).toBe(true);
        }
    }
});

test('demo monday-disaster: all trades land on Monday and are losers', () => {
    const h = localBuild(makeDemoRows('monday-disaster'));
    expect(h.cells[1][9].trades).toBeGreaterThan(0);  // Monday = 1
    expect(h.cells[1][9].wins).toBe(0);
});

test('demo sweet-spot: Tue + Wed 10am cells all wins', () => {
    const h = localBuild(makeDemoRows('sweet-spot'));
    expect(h.cells[2][10].trades).toBeGreaterThan(0);
    expect(h.cells[3][10].trades).toBeGreaterThan(0);
    expect(h.cells[2][10].wins).toBe(h.cells[2][10].trades);
});

test('demo weekend-crypto: lands in Sat (6) + Sun (0) cells', () => {
    const h = localBuild(makeDemoRows('weekend-crypto'));
    expect(h.cells[6][12].trades).toBeGreaterThan(0);  // Sat 12pm
    expect(h.cells[0][14].trades).toBeGreaterThan(0);  // Sun 2pm
});

// ── dec / formatters ──────────────────────────────────────────────

test('dec: safe coercion', () => {
    expect(dec('123.45')).toBe(123.45);
    expect(dec(null)).toBe(0);
    expect(dec('abc')).toBe(0);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtUSD(1234)).toBe('$1234');
    expect(fmtUSDSigned(-100)).toBe('-$100');
    expect(fmtPct(0.5)).toBe('50%');
    expect(fmtHour(9)).toBe('09:00');
    expect(fmtHour(23)).toBe('23:00');
    expect(fmtUSD(NaN)).toBe('—');
    expect(fmtHour(1.5)).toBe('—');
});
