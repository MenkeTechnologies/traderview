// Custom alert rules engine: storage, sanitize/migrate, rule CRUD,
// multi-rule detection, template rendering.

import { test, expect, beforeEach } from 'vitest';
import * as engine from '../js/_alert_rules.js';

function makeStorage() {
    const data = new Map();
    return {
        getItem: k => data.has(k) ? data.get(k) : null,
        setItem: (k, v) => { data.set(k, String(v)); },
    };
}
let storage;
beforeEach(() => { storage = makeStorage(); });

// ── defaults + migrate ──────────────────────────────────────────

test('defaultState ships one default squeeze rule', () => {
    const s = engine.defaultState();
    expect(s.version).toBe(engine.SCHEMA_VERSION);
    expect(s.rules.length).toBe(1);
    expect(s.rules[0].type).toBe('squeeze');
});

test('migrate drops bad rules (missing id / bad type / no name)', () => {
    const m = engine.migrate({
        version: engine.SCHEMA_VERSION,
        rules: [
            { id: 'a', name: 'Good', type: 'squeeze' },
            { id: '', name: 'no-id', type: 'squeeze' },
            { id: 'b', name: '   ', type: 'squeeze' },
            { id: 'c', name: 'bad-type', type: 'martingale' },
            { id: 'd', name: 'Good 2', type: 'price_above', sound: 'bell' },
        ],
    });
    expect(m.rules.length).toBe(2);
    expect(m.rules.map(r => r.id)).toEqual(['a', 'd']);
});

test('migrate restores defaults for missing/invalid sub-fields', () => {
    const m = engine.migrate({
        version: engine.SCHEMA_VERSION,
        rules: [{
            id: 'a', name: 'x', type: 'squeeze',
            sound: 'nonsense',
            cooldown_seconds: -10,
            watchlist: ['aapl', 42],
        }],
    });
    const r = m.rules[0];
    expect(r.sound).toBe('bell');
    expect(r.cooldown_seconds).toBe(60);
    expect(r.watchlist).toEqual(['AAPL']);
});

test('migrate rejects null / wrong-version', () => {
    expect(engine.migrate(null)).toEqual(engine.defaultState());
    expect(engine.migrate({ version: 99 })).toEqual(engine.defaultState());
});

// ── load/save ───────────────────────────────────────────────────

test('save + load round-trip', () => {
    const s = engine.addRule(engine.defaultState(), engine.newRule('price_above', 'AAPL > 150'));
    engine.saveState(s, storage);
    expect(engine.loadState(storage).rules.length).toBe(2);
});

// ── CRUD ────────────────────────────────────────────────────────

test('newRule generates an id + default params per type', () => {
    const r = engine.newRule('volume_spike');
    expect(typeof r.id).toBe('string');
    expect(r.params.volume_mult).toBe(3.0);
    expect(r.params.window_seconds).toBe(300);
});

test('defaultParamsFor unknown type → empty', () => {
    expect(engine.defaultParamsFor('garbage')).toEqual({});
});

test('addRule appends; updateRule patches; removeRule removes; setEnabled flips', () => {
    let s = engine.defaultState();
    const r = engine.newRule('price_above', 'A');
    s = engine.addRule(s, r);
    expect(s.rules.length).toBe(2);
    s = engine.updateRule(s, r.id, { name: 'B', params: { threshold: 200 } });
    expect(s.rules.find(x => x.id === r.id).name).toBe('B');
    expect(s.rules.find(x => x.id === r.id).params.threshold).toBe(200);
    s = engine.setEnabled(s, r.id, false);
    expect(s.rules.find(x => x.id === r.id).enabled).toBe(false);
    s = engine.removeRule(s, r.id);
    expect(s.rules.length).toBe(1);
});

test('addRule regenerates id on collision', () => {
    let s = engine.defaultState();
    const existing = s.rules[0].id;
    s = engine.addRule(s, { ...engine.newRule('squeeze'), id: existing });
    expect(s.rules[1].id).not.toBe(existing);
});

// ── detectEvents ────────────────────────────────────────────────

const tick = (sym, ts, p, v) => ({ symbol: sym, ts, price: p, volume: v });

test('price_above fires on cross + does not refire while still above', () => {
    const rule = {
        ...engine.newRule('price_above'),
        params: { threshold: 100 },
        cooldown_seconds: 0,
    };
    const ticks = [
        tick('AAPL', 0, 95, 0),
        tick('AAPL', 1, 105, 0),    // cross above → fire
        tick('AAPL', 2, 106, 0),    // still above → no fire
        tick('AAPL', 3, 90, 0),     // back below → reset
        tick('AAPL', 4, 105, 0),    // cross above again → fire
    ];
    const events = engine.detectEvents(ticks, {}, { rules: [rule] });
    expect(events.length).toBe(2);
    expect(events[0].ts).toBe(1);
    expect(events[1].ts).toBe(4);
});

test('price_below fires on cross down + reset on cross back up', () => {
    const rule = {
        ...engine.newRule('price_below'),
        params: { threshold: 50 },
        cooldown_seconds: 0,
    };
    const ticks = [
        tick('AAPL', 0, 60, 0),
        tick('AAPL', 1, 40, 0),    // cross down → fire
        tick('AAPL', 2, 60, 0),    // reset
        tick('AAPL', 3, 30, 0),    // cross down → fire
    ];
    const events = engine.detectEvents(ticks, {}, { rules: [rule] });
    expect(events.length).toBe(2);
});

test('pct_change fires when window pct exceeds threshold', () => {
    const rule = {
        ...engine.newRule('pct_change'),
        params: { pct_threshold: 0.10, window_seconds: 100 },
        cooldown_seconds: 100000,
    };
    const ticks = [
        tick('AAPL', 0, 100, 0),
        tick('AAPL', 50, 105, 0),    // +5% — not enough
        tick('AAPL', 90, 112, 0),    // +12% — fires
    ];
    const events = engine.detectEvents(ticks, {}, { rules: [rule] });
    expect(events.length).toBe(1);
    expect(events[0].ts).toBe(90);
});

test('volume_spike requires window cumulative × ADV cross', () => {
    const rule = {
        ...engine.newRule('volume_spike'),
        params: { volume_mult: 2.0, window_seconds: 300 },
        cooldown_seconds: 100000,
    };
    const ticks = [
        tick('AAPL', 0,   100, 10000),
        tick('AAPL', 100, 100, 200000),
        tick('AAPL', 200, 100, 200000),
        tick('AAPL', 300, 100, 200000),
    ];
    const adv = { AAPL: 1_000_000 };
    const events = engine.detectEvents(ticks, adv, { rules: [rule] });
    expect(events.length).toBeGreaterThan(0);
    expect(events[0].symbol).toBe('AAPL');
});

test('squeeze fires only when BOTH price AND volume thresholds clear', () => {
    const rule = {
        ...engine.newRule('squeeze'),
        params: { price_threshold_pct: 0.05, volume_threshold: 2.0, window_seconds: 300 },
        cooldown_seconds: 100000,
    };
    const adv = { AAPL: 1_000_000 };
    // Big price move but tiny volume → no fire
    expect(engine.detectEvents([
        tick('AAPL', 0, 100, 100),
        tick('AAPL', 300, 110, 100),
    ], adv, { rules: [rule] })).toEqual([]);
    // Both thresholds clear → fires
    expect(engine.detectEvents([
        tick('AAPL', 0, 100, 10000),
        tick('AAPL', 300, 110, 100000),
    ], adv, { rules: [rule] }).length).toBe(1);
});

test('watchlist gating: rule with watchlist only fires for listed symbols', () => {
    const rule = {
        ...engine.newRule('price_above'),
        params: { threshold: 100 },
        watchlist: ['AAPL'],
        cooldown_seconds: 0,
    };
    const events = engine.detectEvents([
        tick('AAPL', 0, 95, 0), tick('AAPL', 1, 105, 0),
        tick('TSLA', 2, 95, 0), tick('TSLA', 3, 105, 0),
    ], {}, { rules: [rule] });
    expect(events.length).toBe(1);
    expect(events[0].symbol).toBe('AAPL');
});

test('cooldown suppresses repeat events on the same (rule, symbol)', () => {
    const rule = {
        ...engine.newRule('pct_change'),
        // Window large enough that the post-cooldown bar still has a
        // reference tick; cooldown shorter than the gap to the third event.
        params: { pct_threshold: 0.05, window_seconds: 500 },
        cooldown_seconds: 100,
    };
    const ticks = [
        tick('AAPL', 0, 100, 0),
        tick('AAPL', 50, 110, 0),    // +10% vs ts=0 → fire (last_fired=50)
        tick('AAPL', 80, 112, 0),    // ts−last=30 < cooldown 100 → suppressed
        tick('AAPL', 200, 120, 0),   // ts−last=150 ≥ cooldown, window still has ts=0 → fire
    ];
    const events = engine.detectEvents(ticks, {}, { rules: [rule] });
    expect(events.length).toBe(2);
});

test('disabled rule produces no events', () => {
    const rule = {
        ...engine.newRule('price_above'),
        params: { threshold: 100 },
        enabled: false,
    };
    expect(engine.detectEvents([
        tick('AAPL', 0, 95, 0), tick('AAPL', 1, 105, 0),
    ], {}, { rules: [rule] })).toEqual([]);
});

// ── renderTemplate ──────────────────────────────────────────────

test('renderTemplate substitutes placeholders + formats numerics', () => {
    expect(engine.renderTemplate('{symbol} up {change_pct}% on {volume_mult}× vol', {
        symbol: 'AAPL', change_pct: 0.052, volume_mult: 3.456,
    })).toBe('AAPL up 5.2% on 3.5× vol');
});

test('renderTemplate price + threshold get 2-decimal', () => {
    expect(engine.renderTemplate('{symbol} crossed {threshold}', { symbol: 'X', threshold: 150 }))
        .toBe('X crossed 150.00');
});

test('renderTemplate returns null for empty / non-string', () => {
    expect(engine.renderTemplate('', {})).toBe(null);
    expect(engine.renderTemplate(null, {})).toBe(null);
});

// ── fallbackMessage ─────────────────────────────────────────────

test('fallbackMessage provides per-type human strings', () => {
    expect(engine.fallbackMessage({ type: 'price_above' },
        { symbol: 'X' }, { threshold: 100 })).toMatch(/X crossed above 100/);
    expect(engine.fallbackMessage({ type: 'volume_spike' },
        { symbol: 'X' }, { volume_mult: 3.5 })).toMatch(/X volume spike, 3.5 times/);
});
