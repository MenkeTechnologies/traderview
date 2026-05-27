// Pure-helper tests extracted from launcher / alert_engine / hotkey_engine.
// These guard the filtering, alert-rule, and hotkey-combo logic that every
// keyboard interaction and screener pass goes through.
//
// Run: `node --test frontend/tests/pure.test.mjs`

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { matchesQuery, matchesAlert, buildCombo } from '../js/_pure.js';

// ─── matchesQuery (launcher tile filter) ─────────────────────────────────

const HALT_TILE   = ['halts',       'Halts',         '⏸',  'Nasdaq halt RSS + TTS voice alerts',     'LIVE'];
const SCANNER_TILE = ['live-scanner','Live Scanner',  '⚡',  'Finnhub WS · 6-panel real-time scanner', 'LIVE'];
const SHORT_TILE = ['short-interest','Short Interest','🩳', 'SI %, days-to-cover, CTB', null];

test('matchesQuery empty query matches every tile', () => {
    assert.equal(matchesQuery(HALT_TILE, ''), true);
    assert.equal(matchesQuery(HALT_TILE, null), true);
    assert.equal(matchesQuery(HALT_TILE, undefined), true);
});

test('matchesQuery matches against label (case-insensitive)', () => {
    assert.equal(matchesQuery(HALT_TILE, 'halt'),  true);
    assert.equal(matchesQuery(HALT_TILE, 'HALT'),  true);
    assert.equal(matchesQuery(HALT_TILE, 'HALTS'), true);
});

test('matchesQuery matches against view id', () => {
    // id is "live-scanner"; user types "scanner" — should hit even if label changes.
    assert.equal(matchesQuery(SCANNER_TILE, 'live-'), true);
});

test('matchesQuery matches against description', () => {
    assert.equal(matchesQuery(HALT_TILE, 'rss'), true);
    assert.equal(matchesQuery(HALT_TILE, 'voice'), true);
    assert.equal(matchesQuery(SHORT_TILE, 'days-to-cover'), true);
});

test('matchesQuery returns false when no field contains the query', () => {
    assert.equal(matchesQuery(HALT_TILE, 'futures'), false);
});

test('matchesQuery handles tile with null description', () => {
    const tile = ['x', 'X', '?', null, null];
    assert.equal(matchesQuery(tile, 'x'), true);   // matched on label
    assert.equal(matchesQuery(tile, 'nonexistent'), false);
});

// ─── matchesAlert (alert rule evaluation) ────────────────────────────────

const quote = (over) => Object.assign({
    price: 100, change_pct: 0, volume: 1_000_000,
    day_high: 102, day_low: 98,
}, over);

test('price_above fires when price >= threshold', () => {
    assert.equal(matchesAlert({ trigger: 'price_above', threshold: 99  }, quote()), true);
    assert.equal(matchesAlert({ trigger: 'price_above', threshold: 100 }, quote()), true,
        'inclusive at equality');
    assert.equal(matchesAlert({ trigger: 'price_above', threshold: 101 }, quote()), false);
});

test('price_below fires when price <= threshold', () => {
    assert.equal(matchesAlert({ trigger: 'price_below', threshold: 101 }, quote()), true);
    assert.equal(matchesAlert({ trigger: 'price_below', threshold: 100 }, quote()), true);
    assert.equal(matchesAlert({ trigger: 'price_below', threshold: 99  }, quote()), false);
});

test('pct_up uses positive threshold', () => {
    assert.equal(matchesAlert({ trigger: 'pct_up', threshold: 5 }, quote({ change_pct: 5 })), true);
    assert.equal(matchesAlert({ trigger: 'pct_up', threshold: 5 }, quote({ change_pct: 4.99 })), false);
});

test('pct_down treats threshold as magnitude (compares to -thr)', () => {
    // threshold 5 → fires when change_pct <= -5.
    assert.equal(matchesAlert({ trigger: 'pct_down', threshold: 5 }, quote({ change_pct: -5   })), true);
    assert.equal(matchesAlert({ trigger: 'pct_down', threshold: 5 }, quote({ change_pct: -6   })), true);
    assert.equal(matchesAlert({ trigger: 'pct_down', threshold: 5 }, quote({ change_pct: -4.99 })), false);
});

test('new_high_of_day requires day_high to be set', () => {
    assert.equal(matchesAlert({ trigger: 'new_high_of_day' }, quote({ price: 102, day_high: 102 })), true);
    assert.equal(matchesAlert({ trigger: 'new_high_of_day' }, quote({ price: 101, day_high: 102 })), false);
    assert.equal(matchesAlert({ trigger: 'new_high_of_day' }, { price: 102, day_high: null }),
        false, 'no day_high → no fire (not a crash)');
});

test('new_low_of_day requires day_low to be set', () => {
    assert.equal(matchesAlert({ trigger: 'new_low_of_day' }, quote({ price: 98, day_low: 98 })), true);
    assert.equal(matchesAlert({ trigger: 'new_low_of_day' }, { price: 98, day_low: null }), false);
});

test('volume_surge requires positive threshold (zero would fire on every quote)', () => {
    assert.equal(matchesAlert({ trigger: 'volume_surge', threshold: 0 }, quote({ volume: 1e9 })), false,
        'threshold 0 must NOT fire — would alert on literally every quote');
    assert.equal(matchesAlert({ trigger: 'volume_surge', threshold: 500_000 }, quote()), true);
    assert.equal(matchesAlert({ trigger: 'volume_surge', threshold: 5_000_000 }, quote()), false);
});

test('unknown trigger returns false (no crash)', () => {
    assert.equal(matchesAlert({ trigger: 'rsi_overbought', threshold: 70 }, quote()), false);
    assert.equal(matchesAlert({ trigger: '', threshold: 0 }, quote()), false);
});

// ─── buildCombo (hotkey event → combo string) ────────────────────────────

const ev = (over) => Object.assign({
    ctrlKey: false, altKey: false, shiftKey: false, metaKey: false, key: '',
}, over);

test('buildCombo single modifier + key', () => {
    assert.equal(buildCombo(ev({ ctrlKey: true, key: 'k' })), 'ctrl+k');
    assert.equal(buildCombo(ev({ metaKey: true, key: 'K' })), 'meta+k',
        'key must be lowercased');
});

test('buildCombo all four modifiers ordered ctrl/alt/shift/meta', () => {
    const c = buildCombo(ev({
        ctrlKey: true, altKey: true, shiftKey: true, metaKey: true, key: 'a',
    }));
    assert.equal(c, 'ctrl+alt+shift+meta+a');
});

test('buildCombo returns null for bare-modifier presses', () => {
    // User holding Shift to type a capital letter generates a Shift keydown
    // first — must not register as a "shift+shift" hotkey.
    for (const k of ['Control', 'Shift', 'Alt', 'Meta']) {
        assert.equal(buildCombo(ev({ key: k })), null,
            `bare ${k} press must not produce a combo`);
    }
});

test('buildCombo returns null for empty key', () => {
    assert.equal(buildCombo(ev({ ctrlKey: true, key: '' })), null);
});

test('buildCombo preserves named keys (Enter, Escape, ArrowUp)', () => {
    assert.equal(buildCombo(ev({ ctrlKey: true, key: 'Enter' })),   'ctrl+enter');
    assert.equal(buildCombo(ev({ key: 'Escape' })),                 'escape');
    assert.equal(buildCombo(ev({ shiftKey: true, key: 'ArrowUp' })),'shift+arrowup');
});

test('buildCombo no modifiers returns just the key', () => {
    assert.equal(buildCombo(ev({ key: '?' })), '?');
});
