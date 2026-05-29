// Time-in-Force helpers: validator, body shape, local evaluator
// (parity with Rust evaluate), date helpers, badges, demos.

import { test, expect } from 'vitest';
import {
    TIF_KINDS, validateInputs, buildBody, localEvaluate, actionBadge,
    isValidUtcIso, isValidDate, dateOnlyFromIso, cmpDate, wholeDaysBetween,
    makeDemoOrder, localDtToIsoUtc, isoUtcToLocalDt, isoToDate,
} from '../js/_time_in_force_inputs.js';

const goodOrder = (over = {}) => ({
    tif: 'gtc',
    original_qty: 100,
    filled_qty: 0,
    placed_at: '2026-05-01T10:00:00.000Z',
    good_until: null,
    ...over,
});
const NOW = '2026-05-27T10:00:00.000Z';
const SESS = '2026-05-27';

// ── validateInputs ───────────────────────────────────────────────

test('validate accepts well-formed order', () => {
    expect(validateInputs(goodOrder(), NOW, SESS)).toBe(null);
});

test('validate rejects bad tif', () => {
    expect(validateInputs(goodOrder({ tif: 'limit' }), NOW, SESS)).toMatch(/tif/);
});

test('validate rejects non-positive original_qty', () => {
    expect(validateInputs(goodOrder({ original_qty: 0 }), NOW, SESS)).toMatch(/original_qty/);
    expect(validateInputs(goodOrder({ original_qty: -5 }), NOW, SESS)).toMatch(/original_qty/);
});

test('validate rejects negative filled_qty', () => {
    expect(validateInputs(goodOrder({ filled_qty: -1 }), NOW, SESS)).toMatch(/filled_qty/);
});

test('validate rejects filled > original', () => {
    expect(validateInputs(goodOrder({ original_qty: 100, filled_qty: 200 }), NOW, SESS))
        .toMatch(/cannot exceed/);
});

test('validate rejects malformed ISO timestamps', () => {
    expect(validateInputs(goodOrder({ placed_at: 'not a date' }), NOW, SESS)).toMatch(/placed_at/);
    expect(validateInputs(goodOrder(), 'not a date', SESS)).toMatch(/now/);
});

test('validate rejects bad session_open format', () => {
    expect(validateInputs(goodOrder(), NOW, '2026/05/27')).toMatch(/session_open/);
    expect(validateInputs(goodOrder(), NOW, '2026-02-30')).toMatch(/session_open/); // 2/30 invalid
});

test('validate accepts gtd with null good_until (will still cancel at evaluate)', () => {
    expect(validateInputs(goodOrder({ tif: 'gtd' }), NOW, SESS)).toBe(null);
});

test('validate rejects gtd with malformed good_until', () => {
    expect(validateInputs(goodOrder({ tif: 'gtd', good_until: 'yesterday' }), NOW, SESS))
        .toMatch(/good_until/);
});

// ── buildBody ────────────────────────────────────────────────────

test('buildBody emits backend TifBody shape', () => {
    const o = goodOrder();
    expect(buildBody(o, NOW, SESS)).toEqual({
        order: { tif: 'gtc', original_qty: 100, filled_qty: 0,
                 placed_at: o.placed_at, good_until: null },
        now: NOW,
        session_open: SESS,
    });
});

test('buildBody normalizes undefined good_until to null', () => {
    const o = goodOrder();
    delete o.good_until;
    expect(buildBody(o, NOW, SESS).order.good_until).toBe(null);
});

// ── localEvaluate parity (one test per Rust evaluate branch) ─────

test('eval: fully filled → completed (fully filled)', () => {
    const r = localEvaluate(goodOrder({ filled_qty: 100 }), NOW, SESS);
    expect(r).toEqual({ action: 'completed', reason: 'fully filled' });
});

test('eval: DAY in-session → keep', () => {
    const r = localEvaluate(goodOrder({ tif: 'day', placed_at: '2026-05-27T10:00:00.000Z' }),
        '2026-05-27T14:00:00.000Z', '2026-05-27');
    expect(r.action).toBe('keep');
    expect(r.reason).toBe('DAY order still in session');
});

test('eval: DAY rolled into next session → cancel', () => {
    const r = localEvaluate(goodOrder({ tif: 'day', placed_at: '2026-05-27T10:00:00.000Z' }),
        '2026-05-28T14:00:00.000Z', '2026-05-28');
    expect(r.action).toBe('cancel');
    expect(r.reason).toMatch(/DAY order rolled/);
});

test('eval: GTC within 90 days → keep with reason age', () => {
    const r = localEvaluate(goodOrder({ tif: 'gtc', placed_at: '2026-05-27T10:00:00.000Z' }),
        '2026-06-27T10:00:00.000Z', '2026-06-27');
    expect(r.action).toBe('keep');
    expect(r.reason).toBe('GTC order, age 31 days');
});

test('eval: GTC past 90 days → cancel', () => {
    const r = localEvaluate(goodOrder({ tif: 'gtc', placed_at: '2026-01-01T10:00:00.000Z' }),
        '2026-05-27T10:00:00.000Z', '2026-05-27');
    expect(r.action).toBe('cancel');
    expect(r.reason).toMatch(/exceeded 90-day/);
});

test('eval: IOC with remaining qty → cancel with qty count', () => {
    const r = localEvaluate(goodOrder({ tif: 'ioc', filled_qty: 50 }), NOW, SESS);
    expect(r.action).toBe('cancel');
    expect(r.reason).toMatch(/IOC: cancel 50 unfilled qty/);
});

test('eval: FOK no fill → cancel', () => {
    const r = localEvaluate(goodOrder({ tif: 'fok' }), NOW, SESS);
    expect(r.action).toBe('cancel');
    expect(r.reason).toMatch(/no fill available/);
});

test('eval: FOK partial fill → cancel (partial not allowed)', () => {
    const r = localEvaluate(goodOrder({ tif: 'fok', filled_qty: 50 }), NOW, SESS);
    expect(r.action).toBe('cancel');
    expect(r.reason).toMatch(/partial fill not allowed/);
});

test('eval: GTD within date → keep', () => {
    const r = localEvaluate(goodOrder({ tif: 'gtd', good_until: '2026-06-30' }), NOW, '2026-06-01');
    expect(r.action).toBe('keep');
    expect(r.reason).toBe('GTD valid until 2026-06-30');
});

test('eval: GTD past date → cancel', () => {
    const r = localEvaluate(goodOrder({ tif: 'gtd', good_until: '2026-06-01' }),
        '2026-07-01T10:00:00.000Z', '2026-07-01');
    expect(r.action).toBe('cancel');
    expect(r.reason).toBe('GTD order past good_until date 2026-06-01');
});

test('eval: GTD missing date → cancel', () => {
    const r = localEvaluate(goodOrder({ tif: 'gtd', good_until: null }), NOW, SESS);
    expect(r.action).toBe('cancel');
    expect(r.reason).toBe('GTD missing good_until date');
});

// Boundary: GTC exactly at 90 days = keep (>=, not >).
test('eval: GTC at exactly 90 days → keep', () => {
    const placed = '2026-02-26T10:00:00.000Z';
    const now    = '2026-05-27T10:00:00.000Z'; // 90 days later
    expect(wholeDaysBetween(placed, now)).toBe(90);
    const r = localEvaluate(goodOrder({ tif: 'gtc', placed_at: placed }), now, '2026-05-27');
    expect(r.action).toBe('keep');
});

// ── date helpers ─────────────────────────────────────────────────

test('isValidUtcIso accepts standard ISO; rejects garbage', () => {
    expect(isValidUtcIso('2026-05-27T10:00:00.000Z')).toBe(true);
    expect(isValidUtcIso('2026-05-27T10:00:00Z')).toBe(true);
    expect(isValidUtcIso('garbage')).toBe(false);
    expect(isValidUtcIso(null)).toBe(false);
});

test('isValidDate strict YYYY-MM-DD; rejects malformed', () => {
    expect(isValidDate('2026-05-27')).toBe(true);
    expect(isValidDate('2026-13-01')).toBe(false);
    expect(isValidDate('2026-02-30')).toBe(false);
    expect(isValidDate('2026/05/27')).toBe(false);
});

test('dateOnlyFromIso extracts UTC date component', () => {
    expect(dateOnlyFromIso('2026-05-27T10:00:00.000Z')).toBe('2026-05-27');
    expect(dateOnlyFromIso('2026-05-27T23:59:59.999Z')).toBe('2026-05-27');
});

test('cmpDate string-orders YYYY-MM-DD correctly', () => {
    expect(cmpDate('2026-05-27', '2026-05-28')).toBe(-1);
    expect(cmpDate('2026-05-28', '2026-05-27')).toBe(1);
    expect(cmpDate('2026-05-27', '2026-05-27')).toBe(0);
});

test('wholeDaysBetween truncates toward zero (chrono num_days semantics)', () => {
    // 23h59m is still 0 days; 24h is 1.
    expect(wholeDaysBetween('2026-05-27T00:00:00Z', '2026-05-27T23:59:59Z')).toBe(0);
    expect(wholeDaysBetween('2026-05-27T00:00:00Z', '2026-05-28T00:00:00Z')).toBe(1);
    // Negative direction truncates toward zero too.
    expect(wholeDaysBetween('2026-05-28T00:00:00Z', '2026-05-27T00:00:00Z')).toBe(-1);
});

// ── badges + demos ───────────────────────────────────────────────

test('actionBadge maps each known action', () => {
    expect(actionBadge('keep').cls).toBe('pos');
    expect(actionBadge('cancel').cls).toBe('neg');
    expect(actionBadge('completed').cls).toBe('pos');
    expect(actionBadge('unknown').label).toBe('UNKNOWN');
});

test('TIF_KINDS includes all 5 enum values', () => {
    expect(TIF_KINDS).toEqual(['day', 'gtc', 'ioc', 'fok', 'gtd']);
});

test('demos: every preset self-classifies to its named action', () => {
    const anchor = new Date('2026-05-27T10:00:00.000Z');
    const matrix = [
        ['day-keep',      'keep'],
        ['day-cancel',    'cancel'],
        ['gtc-keep',      'keep'],
        ['gtc-cancel',    'cancel'],
        ['ioc-cancel',    'cancel'],
        ['fok-no-fill',   'cancel'],
        ['fok-partial',   'cancel'],
        ['fok-completed', 'completed'],
        ['completed',     'completed'],
    ];
    for (const [kind, expected] of matrix) {
        const d = makeDemoOrder(kind, anchor);
        expect(localEvaluate(d.order, d.now, d.session_open).action).toBe(expected);
    }
});

test('demos: gtd presets honor good_until_in_order side-band', () => {
    const anchor = new Date('2026-05-27T10:00:00.000Z');
    const cases = [
        ['gtd-keep',    'keep'],
        ['gtd-cancel',  'cancel'],
        ['gtd-missing', 'cancel'],
    ];
    for (const [kind, expected] of cases) {
        const d = makeDemoOrder(kind, anchor);
        d.order.good_until = d.good_until_in_order;
        expect(localEvaluate(d.order, d.now, d.session_open).action).toBe(expected);
    }
});

// ── datetime-local round trip ────────────────────────────────────

test('localDtToIsoUtc + isoUtcToLocalDt round trip preserves wall-clock minutes', () => {
    const local = '2026-05-27T14:30';
    const iso = localDtToIsoUtc(local);
    expect(isValidUtcIso(iso)).toBe(true);
    expect(isoUtcToLocalDt(iso)).toBe(local);
});

test('isoToDate strips time to UTC date', () => {
    expect(isoToDate('2026-05-27T15:00:00.000Z')).toBe('2026-05-27');
});
