// Command-palette pure helpers: catalog builders, fuzzy scoring,
// filterAndRank, highlightLabel, moveSelection.

import { test, expect } from 'vitest';
import {
    buildTileItems, buildFavoriteItems, buildBookmarkItems,
    categoriesByViewId, tilesByViewId,
    fuzzyScore, filterAndRank, highlightLabel, moveSelection,
} from '../js/_command_palette_inputs.js';

const TILES = [
    ['squeeze-alerts',  'Squeeze Alerts',   '🔔', 'Audio bell + TTS when stocks squeeze', 'NEW'],
    ['choppiness',      'Choppiness Idx',   '⛵', 'Trend vs sideways oscillator', null],
    ['stress-test',     'Stress Test',      '🧪', 'Options portfolio stress grid', null],
    ['var-estimator',   'VaR vs Gaussian',  '📊', 'Historical vs parametric VaR', 'NEW'],
];

const CATEGORIES = [
    ['research', 'RESEARCH', ['choppiness', 'stress-test', 'var-estimator']],
    ['trading',  'TRADING',  ['squeeze-alerts']],
];

// ── builders ──────────────────────────────────────────────────────

test('buildTileItems: maps each TILE row to a palette item with kind=view', () => {
    const cats = categoriesByViewId(CATEGORIES);
    const items = buildTileItems(TILES, cats);
    expect(items.length).toBe(4);
    expect(items[0]).toMatchObject({
        id: 'view:squeeze-alerts',
        kind: 'view', viewId: 'squeeze-alerts',
        label: 'Squeeze Alerts', icon: '🔔',
        category: 'TRADING', badge: 'NEW',
    });
});

test('buildTileItems: non-array input safe', () => {
    expect(buildTileItems(null, new Map())).toEqual([]);
});

test('categoriesByViewId: every viewId maps to its category label', () => {
    const m = categoriesByViewId(CATEGORIES);
    expect(m.get('choppiness')).toBe('RESEARCH');
    expect(m.get('squeeze-alerts')).toBe('TRADING');
    expect(m.get('absent')).toBeUndefined();
});

test('tilesByViewId: keyed by [0]', () => {
    const m = tilesByViewId(TILES);
    expect(m.get('choppiness')[1]).toBe('Choppiness Idx');
    expect(m.size).toBe(4);
});

test('buildFavoriteItems: pulls label + hint from tiles map; drops unknown ids', () => {
    const byVid = tilesByViewId(TILES);
    const items = buildFavoriteItems(['var-estimator', 'unknown'], byVid);
    expect(items.length).toBe(2);
    expect(items[0].label).toBe('VaR vs Gaussian');
    expect(items[0].icon).toBe('★');
    expect(items[0].category).toBe('Favorites');
});

test('buildBookmarkItems: respects custom name + pulls viewId hint', () => {
    const byVid = tilesByViewId(TILES);
    const items = buildBookmarkItems([
        { id: 'b1', viewId: 'choppiness', name: 'My Chop Setup' },
        { id: 'b2', viewId: 'choppiness', name: '' }, // empty → tile label
    ], byVid);
    expect(items[0].label).toBe('My Chop Setup');
    expect(items[1].label).toBe('Choppiness Idx');
    expect(items[0].kind).toBe('bookmark');
});

// ── fuzzyScore ────────────────────────────────────────────────────

test('fuzzy: empty query → 1 (neutral pass)', () => {
    expect(fuzzyScore('', { label: 'anything' })).toBe(1);
    expect(fuzzyScore('   ', { label: 'anything' })).toBe(1);
});

test('fuzzy: no match → 0', () => {
    expect(fuzzyScore('xyz', { label: 'foo' })).toBe(0);
});

test('fuzzy: exact match beats partial', () => {
    const exact   = fuzzyScore('squeeze', { label: 'Squeeze Alerts', viewId: 'squeeze-alerts' });
    const partial = fuzzyScore('squeeze', { label: 'Other Squeeze',  viewId: 'other' });
    expect(exact).toBeGreaterThan(partial);
});

test('fuzzy: prefix match beats mid-string', () => {
    const prefix = fuzzyScore('chop', { label: 'Choppiness Idx', viewId: 'choppiness' });
    const mid    = fuzzyScore('chop', { label: 'Pork Chop',      viewId: 'pork-chop' });
    expect(prefix).toBeGreaterThan(mid);
});

test('fuzzy: subsequence match (skipped chars allowed)', () => {
    expect(fuzzyScore('vrg', { label: 'VaR vs Gaussian' })).toBeGreaterThan(0);
});

test('fuzzy: hits viewId / hint / category too', () => {
    expect(fuzzyScore('research', { label: 'X', category: 'RESEARCH' })).toBeGreaterThan(0);
    expect(fuzzyScore('stocks',   { label: 'X', hint: 'when stocks squeeze' })).toBeGreaterThan(0);
});

test('fuzzy: favorite + bookmark get small ranking bonus', () => {
    const view = fuzzyScore('chop', { label: 'Chop', viewId: 'chop', kind: 'view' });
    const fav  = fuzzyScore('chop', { label: 'Chop', viewId: 'chop', kind: 'favorite' });
    expect(fav).toBeGreaterThan(view);
});

// ── filterAndRank ─────────────────────────────────────────────────

test('filterAndRank: empty query returns all items with stable order', () => {
    const cats = categoriesByViewId(CATEGORIES);
    const items = buildTileItems(TILES, cats);
    const out = filterAndRank(items, '', 10);
    expect(out.length).toBe(4);
    expect(out.map(it => it.viewId)).toEqual(['squeeze-alerts', 'choppiness', 'stress-test', 'var-estimator']);
});

test('filterAndRank: filters and sorts by score desc', () => {
    const cats = categoriesByViewId(CATEGORIES);
    const items = buildTileItems(TILES, cats);
    const out = filterAndRank(items, 'sq', 10);
    expect(out.length).toBe(1);
    expect(out[0].viewId).toBe('squeeze-alerts');
});

test('filterAndRank: respects max cap', () => {
    const items = Array.from({ length: 50 }, (_, i) => ({
        id: `view:x${i}`, kind: 'view', viewId: `x${i}`, label: `Match ${i}`,
    }));
    const out = filterAndRank(items, 'match', 10);
    expect(out.length).toBe(10);
});

test('filterAndRank: non-array input → empty', () => {
    expect(filterAndRank(null, 'q', 5)).toEqual([]);
});

// ── highlightLabel ────────────────────────────────────────────────

test('highlightLabel: marks matched character runs', () => {
    const segs = highlightLabel('Squeeze', 'sqz');
    const concat = segs.map(s => s.ch).join('');
    expect(concat).toBe('Squeeze');
    const matched = segs.filter(s => s.hit).map(s => s.ch).join('');
    expect(matched.toLowerCase()).toContain('s');
    expect(matched.toLowerCase()).toContain('q');
    expect(matched.toLowerCase()).toContain('z');
});

test('highlightLabel: empty query → single non-hit segment', () => {
    expect(highlightLabel('foo', '')).toEqual([{ ch: 'foo', hit: false }]);
});

test('highlightLabel: empty label → empty array', () => {
    expect(highlightLabel('', 'q')).toEqual([]);
});

// ── moveSelection ─────────────────────────────────────────────────

test('moveSelection: wraps at boundaries', () => {
    expect(moveSelection(0,  -1, 3)).toBe(2);
    expect(moveSelection(2,   1, 3)).toBe(0);
    expect(moveSelection(1,   1, 3)).toBe(2);
});

test('moveSelection: total=0 returns 0 safely', () => {
    expect(moveSelection(5, 1, 0)).toBe(0);
});
