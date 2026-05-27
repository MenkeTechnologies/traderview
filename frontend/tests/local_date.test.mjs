// localToday must use LOCAL TZ — not UTC. The bug this guards against:
//   * `new Date().toISOString().slice(0, 10)` is UTC, so during ET/PT
//     evening review the journal hotkey lands on tomorrow's empty page.
//
// Run: `node --test frontend/tests/local_date.test.mjs`

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { localToday } from '../js/local_date.js';

test('localToday formats YYYY-MM-DD with zero-padding', () => {
    const d = new Date(2026, 0, 5);  // January 5, 2026 LOCAL
    assert.equal(localToday(d), '2026-01-05');
});

test('localToday uses 1-based month (not 0-based getMonth)', () => {
    const dec = new Date(2026, 11, 31);  // December 31
    assert.equal(localToday(dec), '2026-12-31',
        'getMonth() returns 11 for December — must be +1');
});

test('localToday uses local date, not UTC', () => {
    // A moment that is 2026-05-27 23:30 in PT (UTC-7) but already
    // 2026-05-28 in UTC. The local-date helper must return the local day.
    //
    // We can't force the system TZ inside a test, so we sanity-check the
    // contract: localToday(d) == year-month-day of THE PASSED DATE's local
    // components. If the helper accidentally went through toISOString(),
    // this would diverge whenever local and UTC days differ.
    const d = new Date(2026, 4, 27, 23, 30);  // May 27, 23:30 LOCAL
    const expected = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
    assert.equal(localToday(d), expected);
});

test('localToday defaults to "now" when called with no arg', () => {
    const out = localToday();
    assert.match(out, /^\d{4}-\d{2}-\d{2}$/);
});

test('localToday pads single-digit days and months', () => {
    const d = new Date(2026, 0, 1);  // Jan 1
    assert.equal(localToday(d), '2026-01-01');
});
