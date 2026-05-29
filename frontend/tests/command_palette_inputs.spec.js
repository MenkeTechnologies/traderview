// Command-palette pure helpers: catalog builders, fuzzy scoring,
// filterAndRank, highlightLabel, moveSelection.

import { test, expect } from 'vitest';
import {
    buildTileItems, buildFavoriteItems, buildBookmarkItems, buildActionItems,
    categoriesByViewId, tilesByViewId,
    fuzzyScore, filterAndRank, highlightLabel, moveSelection,
    tileLabelKey, tileDescKey, categoryLabelKey,
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

// ── buildActionItems ─────────────────────────────────────────────

const SHORTCUTS = [
    { id: 'command_palette', actionKey: 'tv:open-palette', descKey: 'shortcut.command_palette', scope: 'global' },
    { id: 'toggle_favorite', actionKey: 'tv:toggle-favorite', descKey: 'shortcut.toggle_favorite', scope: 'global' },
    { id: 'no_action',        descKey: 'no.action.key',                scope: 'global' },  // no actionKey → dropped
];
const TR = (k) => ({
    'shortcut.command_palette': 'Open command palette',
    'shortcut.toggle_favorite': 'Toggle favorite for this view',
})[k] || k;
const CHIP = (sc) => sc.id === 'command_palette' ? '⌘K' : '⌘D';

test('buildActionItems: maps each shortcut with actionKey to kind=action', () => {
    const items = buildActionItems(SHORTCUTS, TR, CHIP);
    expect(items.length).toBe(2);   // no_action dropped
    expect(items[0].kind).toBe('action');
    expect(items[0].actionKey).toBe('tv:open-palette');
    expect(items[0].label).toBe('Open command palette');
    expect(items[0].hint).toBe('⌘K');
    expect(items[0].category).toBe('Actions');
    expect(items[0].id).toBe('action:command_palette');
});

test('buildActionItems: drops shortcuts without actionKey', () => {
    const items = buildActionItems(SHORTCUTS, TR, CHIP);
    expect(items.find(it => it.id === 'action:no_action')).toBeUndefined();
});

test('buildActionItems: translate/chip optional → safe defaults', () => {
    const items = buildActionItems(SHORTCUTS);
    expect(items.length).toBe(2);
    expect(items[0].label).toBe('shortcut.command_palette');   // verbatim key
    expect(items[0].hint).toBe('');
});

test('buildActionItems: non-array input safe', () => {
    expect(buildActionItems(null)).toEqual([]);
    expect(buildActionItems(undefined)).toEqual([]);
});

test('buildActionItems: fuzzy matches by translated label', () => {
    const items = buildActionItems(SHORTCUTS, TR, CHIP);
    expect(fuzzyScore('favorite', items[1])).toBeGreaterThan(0);
    expect(fuzzyScore('palette',  items[0])).toBeGreaterThan(0);
    expect(fuzzyScore('nonsense', items[0])).toBe(0);
});

test('buildActionItems: action items rank above raw views (kind tiebreaker)', () => {
    const action = { kind: 'action', label: 'Reload data', icon: '⚡' };
    const view   = { kind: 'view',   label: 'Reload data', icon: '🔄' };
    expect(fuzzyScore('reload', action)).toBeGreaterThan(fuzzyScore('reload', view));
});

// ── i18n key conventions ─────────────────────────────────────────

test('tileLabelKey / tileDescKey / categoryLabelKey: stable key formats', () => {
    expect(tileLabelKey('choppiness')).toBe('tile.choppiness.label');
    expect(tileDescKey('choppiness')).toBe('tile.choppiness.desc');
    expect(categoryLabelKey('research')).toBe('tile.cat.research');
});

test('buildTileItems: when translate hits, uses translated label/hint', () => {
    const cats = categoriesByViewId(CATEGORIES);
    const dict = {
        'tile.choppiness.label': 'Índice de turbulencia',
        'tile.choppiness.desc':  'Oscilador de tendencia vs lateral',
    };
    const tr = (k) => dict[k] || k;
    const items = buildTileItems(TILES, cats, tr);
    const chop = items.find(i => i.viewId === 'choppiness');
    expect(chop.label).toBe('Índice de turbulencia');
    expect(chop.hint).toBe('Oscilador de tendencia vs lateral');
});

test('buildTileItems: missing key falls back to TILES literal', () => {
    const cats = categoriesByViewId(CATEGORIES);
    const tr = (k) => k; // every key is a miss (returns the key, matching i18n.t() semantics)
    const items = buildTileItems(TILES, cats, tr);
    const chop = items.find(i => i.viewId === 'choppiness');
    expect(chop.label).toBe('Choppiness Idx');
    expect(chop.hint).toBe('Trend vs sideways oscillator');
});

test('buildTileItems: no translate arg → behaves like before', () => {
    const cats = categoriesByViewId(CATEGORIES);
    const items = buildTileItems(TILES, cats);
    expect(items[0].label).toBe('Squeeze Alerts');
});

test('categoriesByViewId: translates category labels via tile.cat.<id>', () => {
    const dict = { 'tile.cat.research': 'INVESTIGACIÓN' };
    const tr = (k) => dict[k] || k;
    const map = categoriesByViewId(CATEGORIES, tr);
    expect(map.get('choppiness')).toBe('INVESTIGACIÓN');
    expect(map.get('squeeze-alerts')).toBe('TRADING'); // miss → literal
});

test('buildFavoriteItems: translated label + Favorites category via translate', () => {
    const tr = (k) => k === 'palette.cat.favorites' ? 'Favoritos'
        : k === 'tile.choppiness.label' ? 'Turbulencia'
        : k;
    const byVid = tilesByViewId(TILES);
    const items = buildFavoriteItems(['choppiness'], byVid, tr);
    expect(items[0].label).toBe('Turbulencia');
    expect(items[0].category).toBe('Favoritos');
});

test('buildBookmarkItems: user-named bookmark keeps custom name', () => {
    const tr = (k) => k === 'tile.choppiness.label' ? 'Turbulencia' : k;
    const byVid = tilesByViewId(TILES);
    const items = buildBookmarkItems(
        [{ id: 'b1', name: 'My pinned name', viewId: 'choppiness' }], byVid, tr);
    expect(items[0].label).toBe('My pinned name');
});

test('buildBookmarkItems: nameless bookmark falls back to translated tile label', () => {
    const tr = (k) => k === 'tile.choppiness.label' ? 'Turbulencia' : k;
    const byVid = tilesByViewId(TILES);
    const items = buildBookmarkItems(
        [{ id: 'b1', name: '', viewId: 'choppiness' }], byVid, tr);
    expect(items[0].label).toBe('Turbulencia');
});

test('buildActionItems: Actions category translated via palette.cat.actions', () => {
    const tr = (k) => k === 'palette.cat.actions' ? 'Acciones'
        : k === 'shortcut.reload' ? 'Recargar' : k;
    const shortcuts = [
        { id: 'reload', descKey: 'shortcut.reload', actionKey: 'tv:reload', scope: 'global' },
    ];
    const items = buildActionItems(shortcuts, tr, () => '⌘R');
    expect(items[0].category).toBe('Acciones');
    expect(items[0].label).toBe('Recargar');
});
