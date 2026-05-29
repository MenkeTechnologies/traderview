// Futures-roll helpers: parser, validator, body shape, localSchedule
// Rust-mirror with priority + boundary tests, badges, demos.

import { test, expect } from 'vitest';
import {
    URGENCIES, parsePositionBlob, validateInputs, buildBody, localSchedule,
    isValidDate, daysBetween, dateOffset, todayIso,
    urgencyBadge, urgencyLabelKey, overallBadge,
    makeDemoPositions, fmtDays, fmtContracts,
} from '../js/_futures_roll_inputs.js';

const pos = (sym, ct, exp) => ({ symbol: sym, contracts: ct, expiration: exp });

// ── constants ─────────────────────────────────────────────────────

test('URGENCIES exposes the 4 Rust enum values', () => {
    expect(URGENCIES).toEqual(['now', 'soon', 'comfortable', 'expired']);
});

// ── parser ────────────────────────────────────────────────────────

test('parsePositionBlob: 3 tokens + comments', () => {
    const r = parsePositionBlob('/ES 1 2026-06-19\n# note\n/NQ -2 2026-06-19');
    expect(r.errors).toEqual([]);
    expect(r.positions).toEqual([pos('/ES', 1, '2026-06-19'), pos('/NQ', -2, '2026-06-19')]);
});

test('parsePositionBlob: rejects bad date / non-integer or zero contracts / wrong token count', () => {
    expect(parsePositionBlob('/ES 1 2026/06/19').errors[0].message).toMatch(/expiration/);
    expect(parsePositionBlob('/ES 1.5 2026-06-19').errors[0].message).toMatch(/contracts/);
    expect(parsePositionBlob('/ES 0 2026-06-19').errors[0].message).toMatch(/non-zero/);
    expect(parsePositionBlob('/ES 1').errors[0].message).toMatch(/3 tokens/);
});

test('parsePositionBlob: non-string returns 1 error', () => {
    expect(parsePositionBlob(null).errors.length).toBe(1);
});

// ── date helpers ──────────────────────────────────────────────────

test('isValidDate strict + rejects bogus calendar', () => {
    expect(isValidDate('2026-06-19')).toBe(true);
    expect(isValidDate('2026-02-30')).toBe(false);
    expect(isValidDate('2026/06/19')).toBe(false);
});

test('daysBetween: whole-days; b−a sign', () => {
    expect(daysBetween('2026-05-27', '2026-06-03')).toBe(7);
    expect(daysBetween('2026-06-03', '2026-05-27')).toBe(-7);
});

test('dateOffset: adds days correctly across month boundary', () => {
    expect(dateOffset('2026-01-30', 5)).toBe('2026-02-04');
    expect(dateOffset('2026-03-01', -10)).toBe('2026-02-19');
});

test('todayIso returns valid YYYY-MM-DD', () => {
    expect(isValidDate(todayIso())).toBe(true);
});

// ── validator / buildBody ─────────────────────────────────────────

test('validate accepts good inputs', () => {
    expect(validateInputs([], '2026-05-27', 7)).toBe(null);
});

test('validate rejects bad today / negative window / non-integer window', () => {
    expect(validateInputs([], 'bogus', 7)).toMatch(/today/);
    expect(validateInputs([], '2026-05-27', -1)).toMatch(/roll_window_days/);
    expect(validateInputs([], '2026-05-27', 1.5)).toMatch(/roll_window_days/);
});

test('validate accepts roll_window_days=0 (only flag expired)', () => {
    expect(validateInputs([], '2026-05-27', 0)).toBe(null);
});

test('buildBody preserves all fields', () => {
    expect(buildBody([pos('/ES', 1, '2026-06-19')], '2026-05-27', 7)).toEqual({
        positions: [{ symbol: '/ES', contracts: 1, expiration: '2026-06-19' }],
        today: '2026-05-27',
        roll_window_days: 7,
    });
});

// ── localSchedule parity (one test per Rust property) ────────────

test('local: empty → empty rows + zero counts', () => {
    const r = localSchedule([], '2026-05-27', 7);
    expect(r.rows).toEqual([]);
    expect(r.now_count).toBe(0);
    expect(r.expired_count).toBe(0);
});

test('local: within roll window → now', () => {
    const r = localSchedule([pos('/ES', 1, '2026-06-03')], '2026-05-27', 7);
    expect(r.rows[0].urgency).toBe('now');
    expect(r.now_count).toBe(1);
});

test('local: beyond 2× window → comfortable', () => {
    const r = localSchedule([pos('/ES', 1, '2026-06-26')], '2026-05-27', 7);
    expect(r.rows[0].urgency).toBe('comfortable');
});

test('local: in second window (between 1×–2×) → soon', () => {
    const r = localSchedule([pos('/ES', 1, '2026-06-06')], '2026-05-27', 7);
    expect(r.rows[0].urgency).toBe('soon');
});

test('local: past expiry → expired (negative days)', () => {
    const r = localSchedule([pos('/ES', 1, '2026-05-20')], '2026-05-27', 7);
    expect(r.rows[0].urgency).toBe('expired');
    expect(r.expired_count).toBe(1);
    expect(r.rows[0].days_to_expiry).toBe(-7);
});

test('local: rows sorted by days_to_expiry ASC (most urgent first)', () => {
    const r = localSchedule([
        pos('/A', 1, '2026-07-01'),  // comfortable
        pos('/B', 1, '2026-06-01'),  // sooner
        pos('/C', 1, '2026-05-28'),  // now
    ], '2026-05-27', 7);
    expect(r.rows.map(x => x.symbol)).toEqual(['/C', '/B', '/A']);
});

test('local: counts track per-urgency (expired + now)', () => {
    const r = localSchedule([
        pos('/A', 1, '2026-05-20'),  // expired
        pos('/B', 1, '2026-06-01'),  // now
        pos('/C', 1, '2026-05-28'),  // now
        pos('/D', 1, '2026-07-01'),  // comfortable
    ], '2026-05-27', 7);
    expect(r.expired_count).toBe(1);
    expect(r.now_count).toBe(2);
});

test('local: larger window makes more positions urgent', () => {
    const small = localSchedule([pos('/ES', 1, '2026-06-10')], '2026-05-27', 7);
    const large = localSchedule([pos('/ES', 1, '2026-06-10')], '2026-05-27', 21);
    expect(small.rows[0].urgency).toBe('soon');
    expect(large.rows[0].urgency).toBe('now');
});

// ── boundary semantics (Rust uses <, <=) ──────────────────────────

test('boundary: exactly = window → now (≤)', () => {
    const r = localSchedule([pos('/ES', 1, '2026-06-03')], '2026-05-27', 7);  // exactly 7d
    expect(r.rows[0].urgency).toBe('now');
});

test('boundary: exactly = 2×window → soon (≤)', () => {
    const r = localSchedule([pos('/ES', 1, '2026-06-10')], '2026-05-27', 7);  // exactly 14d
    expect(r.rows[0].urgency).toBe('soon');
});

test('boundary: 1 past 2×window → comfortable', () => {
    const r = localSchedule([pos('/ES', 1, '2026-06-11')], '2026-05-27', 7);  // 15d
    expect(r.rows[0].urgency).toBe('comfortable');
});

test('boundary: today=expiry → now (days=0)', () => {
    const r = localSchedule([pos('/ES', 1, '2026-05-27')], '2026-05-27', 7);
    expect(r.rows[0].days_to_expiry).toBe(0);
    expect(r.rows[0].urgency).toBe('now');
});

// ── badges ────────────────────────────────────────────────────────

test('urgencyBadge: now/expired = neg, soon = empty, comfortable = pos', () => {
    expect(urgencyBadge('now').cls).toBe('neg');
    expect(urgencyBadge('expired').cls).toBe('neg');
    expect(urgencyBadge('soon').cls).toBe('');
    expect(urgencyBadge('comfortable').cls).toBe('pos');
    expect(urgencyBadge('bogus').key).toMatch(/unknown/);
});

test('overallBadge: expired wins, then now, then clean, empty handled', () => {
    expect(overallBadge({ rows: [], now_count: 0, expired_count: 0 }).key).toMatch(/empty/);
    expect(overallBadge({ rows: [1], now_count: 0, expired_count: 1 }).key).toMatch(/emergency/);
    expect(overallBadge({ rows: [1], now_count: 1, expired_count: 0 }).key).toMatch(/action/);
    expect(overallBadge({ rows: [1, 2], now_count: 0, expired_count: 0 }).key).toMatch(/clean/);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset returns ≥ 3 valid positions', () => {
    const t = '2026-05-27';
    for (const k of ['mixed', 'all-now', 'all-soon', 'comfortable', 'emergency']) {
        const positions = makeDemoPositions(k, t);
        expect(positions.length).toBeGreaterThanOrEqual(3);
        for (const p of positions) {
            expect(isValidDate(p.expiration)).toBe(true);
            expect(Number.isInteger(p.contracts)).toBe(true);
            expect(p.contracts).not.toBe(0);
        }
    }
});

test('demo all-now: every position urgency = now', () => {
    const t = '2026-05-27';
    const r = localSchedule(makeDemoPositions('all-now', t), t, 7);
    expect(r.rows.every(row => row.urgency === 'now')).toBe(true);
});

test('demo all-soon: every position urgency = soon (with window 7)', () => {
    const t = '2026-05-27';
    const r = localSchedule(makeDemoPositions('all-soon', t), t, 7);
    expect(r.rows.every(row => row.urgency === 'soon')).toBe(true);
});

test('demo emergency: at least 1 expired + emergency verdict', () => {
    const t = '2026-05-27';
    const r = localSchedule(makeDemoPositions('emergency', t), t, 7);
    expect(r.expired_count).toBeGreaterThan(0);
    expect(overallBadge(r).key).toMatch(/emergency/);
});

// ── formatters ────────────────────────────────────────────────────

test('fmt helpers + non-finite guards', () => {
    expect(fmtDays(7)).toBe('+7d');
    expect(fmtDays(-3)).toBe('-3d');
    expect(fmtDays(0)).toBe('+0d');
    expect(fmtContracts(2)).toBe('+2');
    expect(fmtContracts(-1)).toBe('-1');
    expect(fmtDays(NaN)).toBe('—');
});

test('urgencyLabelKey returns view.futures_roll.urgency.<u>', () => {
    expect(urgencyLabelKey('now')).toBe('view.futures_roll.urgency.now');
    expect(urgencyLabelKey()).toBe('view.futures_roll.urgency.unknown');
});
