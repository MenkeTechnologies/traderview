// Squeeze Alerts engine: tick + ADV parsers, settings persistence,
// watchlist gating, detection engine, demo invariants, formatters.

import { test, expect, beforeEach } from 'vitest';
import * as engine from '../js/_squeeze_alerts_inputs.js';

function makeStorage() {
    const data = new Map();
    return {
        getItem: k => data.has(k) ? data.get(k) : null,
        setItem: (k, v) => { data.set(k, String(v)); },
        removeItem: k => { data.delete(k); },
        clear: () => { data.clear(); },
    };
}
let storage;
beforeEach(() => { storage = makeStorage(); });

// ── parseTickBlob ─────────────────────────────────────────────────

test('parseTickBlob accepts 4-token ticks + uppercases symbols', () => {
    const r = engine.parseTickBlob('aapl 1700000000 150 5000\nmsft 1700000030 320 3000');
    expect(r.errors).toEqual([]);
    expect(r.ticks.map(t => t.symbol)).toEqual(['AAPL', 'MSFT']);
});

test('parseTickBlob rejects bad symbol / negative ts / non-positive price / negative volume', () => {
    expect(engine.parseTickBlob('A!P 1 150 5000').errors[0].message).toMatch(/bad symbol/);
    expect(engine.parseTickBlob('AAPL -1 150 5000').errors[0].message).toMatch(/timestamp/);
    expect(engine.parseTickBlob('AAPL 1 0 5000').errors[0].message).toMatch(/price/);
    expect(engine.parseTickBlob('AAPL 1 150 -5').errors[0].message).toMatch(/volume/);
});

test('parseTickBlob rejects wrong token count', () => {
    expect(engine.parseTickBlob('AAPL 1 150').errors[0].message).toMatch(/expected 4 tokens/);
});

// ── parseAdvBlob ─────────────────────────────────────────────────

test('parseAdvBlob parses + uppercases', () => {
    const r = engine.parseAdvBlob('aapl 50000000\nmsft 25000000');
    expect(r.errors).toEqual([]);
    expect(r.adv).toEqual({ AAPL: 50_000_000, MSFT: 25_000_000 });
});

test('parseAdvBlob rejects non-positive adv', () => {
    expect(engine.parseAdvBlob('AAPL 0').errors[0].message).toMatch(/adv/);
});

// ── Settings ──────────────────────────────────────────────────────

test('loadSettings returns defaults when no key', () => {
    expect(engine.loadSettings(storage)).toEqual(engine.DEFAULT_SETTINGS);
});

test('save + load round-trips', () => {
    const next = { ...engine.DEFAULT_SETTINGS, price_threshold_pct: 0.1, bell_enabled: false };
    expect(engine.saveSettings(next, storage)).toBe(true);
    expect(engine.loadSettings(storage)).toEqual(next);
});

test('migrateSettings drops bad fields', () => {
    const dirty = {
        price_threshold_pct: 'not-a-number',
        bell_enabled: 'truthy-string',
        watchlist: ['AAPL', 42, 'TSLA'],
        unknown_field: 'ignored',
    };
    const m = engine.migrateSettings(dirty);
    expect(m.price_threshold_pct).toBe(engine.DEFAULT_SETTINGS.price_threshold_pct);
    expect(m.bell_enabled).toBe(engine.DEFAULT_SETTINGS.bell_enabled);
    expect(m.watchlist).toEqual(['AAPL', 'TSLA']);
});

// ── isWatched ────────────────────────────────────────────────────

test('isWatched: empty watchlist = all symbols qualify', () => {
    expect(engine.isWatched('AAPL', [])).toBe(true);
    expect(engine.isWatched('TSLA', null)).toBe(true);
});

test('isWatched: non-empty list gates by case-insensitive match', () => {
    expect(engine.isWatched('AAPL', ['aapl', 'TSLA'])).toBe(true);
    expect(engine.isWatched('MSFT', ['AAPL', 'TSLA'])).toBe(false);
});

// ── detectSqueezes ───────────────────────────────────────────────

function tick(symbol, ts, price, volume) { return { symbol, ts, price, volume }; }

test('detectSqueezes fires when BOTH price + volume thresholds clear', () => {
    const ticks = [
        tick('AAPL', 0,   100, 10000),
        tick('AAPL', 300, 106, 60000),    // +6% over 5 min, big vol
    ];
    const adv = { AAPL: 1_000_000 };
    const events = engine.detectSqueezes(ticks, adv, {
        price_threshold_pct: 0.05, volume_threshold: 1.0,
        window_seconds: 300, cooldown_seconds: 60,
    });
    expect(events.length).toBe(1);
    expect(events[0].symbol).toBe('AAPL');
    expect(events[0].price_change_pct).toBeCloseTo(0.06, 6);
});

test('detectSqueezes: price OK but volume below threshold = no alert', () => {
    const ticks = [
        tick('AAPL', 0, 100, 100),
        tick('AAPL', 300, 110, 100),    // +10% but tiny volume
    ];
    const adv = { AAPL: 100_000_000 };
    const events = engine.detectSqueezes(ticks, adv, {
        price_threshold_pct: 0.05, volume_threshold: 2.0,
        window_seconds: 300, cooldown_seconds: 60,
    });
    expect(events).toEqual([]);
});

test('detectSqueezes: volume OK but price below threshold = no alert', () => {
    const ticks = [
        tick('AAPL', 0, 100, 100000),
        tick('AAPL', 300, 102, 100000),    // +2% on big vol — not enough price
    ];
    const adv = { AAPL: 1_000_000 };
    const events = engine.detectSqueezes(ticks, adv, {
        price_threshold_pct: 0.05, volume_threshold: 1.0,
        window_seconds: 300, cooldown_seconds: 60,
    });
    expect(events).toEqual([]);
});

test('detectSqueezes: cooldown suppresses repeat alerts within window', () => {
    const ticks = [
        tick('AAPL', 0, 100, 100000),
        tick('AAPL', 300, 110, 100000),    // first alert
        tick('AAPL', 320, 112, 100000),    // would be alert but within cooldown
        tick('AAPL', 400, 115, 100000),    // post-cooldown alert
    ];
    const adv = { AAPL: 1_000_000 };
    const events = engine.detectSqueezes(ticks, adv, {
        price_threshold_pct: 0.05, volume_threshold: 1.0,
        window_seconds: 600, cooldown_seconds: 60,
    });
    expect(events.length).toBe(2);
    expect(events[0].ts).toBe(300);
    expect(events[1].ts).toBe(400);
});

test('detectSqueezes: severity flips to critical at 2× thresholds', () => {
    const ticks = [
        tick('SMID', 0, 8, 100),
        tick('SMID', 300, 9, 80000),    // +12.5% with massive vol
    ];
    const adv = { SMID: 250_000 };
    const events = engine.detectSqueezes(ticks, adv, {
        price_threshold_pct: 0.05, volume_threshold: 2.0,
        window_seconds: 300, cooldown_seconds: 60,
    });
    expect(events[0].severity).toBe('critical');
});

test('detectSqueezes: watchlist gate filters out non-watched symbols', () => {
    const ticks = [
        tick('AAPL', 0, 100, 100000),
        tick('AAPL', 300, 110, 100000),
        tick('TSLA', 0, 200, 100000),
        tick('TSLA', 300, 220, 100000),
    ];
    const adv = { AAPL: 1_000_000, TSLA: 1_000_000 };
    const events = engine.detectSqueezes(ticks, adv, {
        price_threshold_pct: 0.05, volume_threshold: 1.0,
        window_seconds: 600, cooldown_seconds: 60,
        watchlist: ['AAPL'],
    });
    expect(events.length).toBe(1);
    expect(events[0].symbol).toBe('AAPL');
});

test('detectSqueezes: missing ADV for a symbol silently skips it', () => {
    const ticks = [
        tick('UNKNOWN', 0, 100, 100000),
        tick('UNKNOWN', 300, 110, 100000),
    ];
    const events = engine.detectSqueezes(ticks, {}, {
        price_threshold_pct: 0.05, volume_threshold: 1.0,
        window_seconds: 300, cooldown_seconds: 60,
    });
    expect(events).toEqual([]);
});

test('detectSqueezes: empty + non-array inputs return empty', () => {
    expect(engine.detectSqueezes([], {}, {})).toEqual([]);
    expect(engine.detectSqueezes(null, {}, {})).toEqual([]);
});

// ── Demo invariants ──────────────────────────────────────────────

test('makeDemoData: 3 symbols + ADV present for all + AAPL & SMID squeeze, MSFT does not', () => {
    const { ticks, adv } = engine.makeDemoData();
    expect(Object.keys(adv).sort()).toEqual(['AAPL', 'MSFT', 'SMID']);
    const events = engine.detectSqueezes(ticks, adv, engine.DEFAULT_SETTINGS);
    const symbols = new Set(events.map(e => e.symbol));
    expect(symbols.has('AAPL')).toBe(true);
    expect(symbols.has('SMID')).toBe(true);
    expect(symbols.has('MSFT')).toBe(false);
});

test('makeDemoData: SMID alert is critical (2× thresholds)', () => {
    const { ticks, adv } = engine.makeDemoData();
    const events = engine.detectSqueezes(ticks, adv, engine.DEFAULT_SETTINGS);
    const smid = events.find(e => e.symbol === 'SMID');
    expect(smid).toBeTruthy();
    expect(smid.severity).toBe('critical');
});

// ── Formatters ───────────────────────────────────────────────────

test('fmtPct signs positive + 2-decimal', () => {
    expect(engine.fmtPct(0.052)).toBe('+5.20%');
    expect(engine.fmtPct(-0.05)).toBe('-5.00%');
    expect(engine.fmtPct(NaN)).toBe('—');
});

test('fmtMult suffixes ×', () => {
    expect(engine.fmtMult(3.5)).toBe('3.5×');
    expect(engine.fmtMult(NaN)).toBe('—');
});

test('fmtTime ISO HH:MM:SS', () => {
    expect(engine.fmtTime(0)).toBe('00:00:00');
    expect(engine.fmtTime(NaN)).toBe('—');
});
