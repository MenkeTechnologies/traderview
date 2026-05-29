// Context-menu pure helpers: GLOBAL_ITEMS, positionMenu, compileMenu,
// mergeMenu, nextVisibleIdx, findMnemonic.

import { test, expect } from 'vitest';
import {
    GLOBAL_ITEMS, EDITING_ITEMS, positionMenu, compileMenu, mergeMenu,
    mergeMenuWithEditing, nextVisibleIdx, findMnemonic,
} from '../js/_context_menu.js';

// ── GLOBAL_ITEMS ──────────────────────────────────────────────────

test('GLOBAL_ITEMS: every non-separator has a labelKey + (actionKey || navTo)', () => {
    for (const it of GLOBAL_ITEMS) {
        if (it.kind === 'separator') continue;
        expect(typeof it.labelKey).toBe('string');
        expect(it.actionKey || it.navTo).toBeTruthy();
    }
});

test('GLOBAL_ITEMS: includes open_palette + reload + copy_view_url + shortcuts', () => {
    const ids = GLOBAL_ITEMS.filter(it => it.id).map(it => it.id);
    expect(ids).toContain('open_palette');
    expect(ids).toContain('reload');
    expect(ids).toContain('copy_view_url');
    expect(ids).toContain('shortcuts');
});

test('GLOBAL_ITEMS: includes toggle_favorite + open_new_tab + copy_view_id', () => {
    const ids = GLOBAL_ITEMS.filter(it => it.id).map(it => it.id);
    expect(ids).toContain('toggle_favorite');
    expect(ids).toContain('open_new_tab');
    expect(ids).toContain('copy_view_id');
});

test('GLOBAL_ITEMS: includes add_bookmark wired to tv:add-bookmark', () => {
    const item = GLOBAL_ITEMS.find(it => it.id === 'add_bookmark');
    expect(item).toBeTruthy();
    expect(item.actionKey).toBe('tv:add-bookmark');
    expect(item.labelKey).toBe('ctxmenu.add_bookmark');
    expect(item.section).toBe('view');
});

test('GLOBAL_ITEMS: theme / crt / neon toggles present in appearance section', () => {
    for (const id of ['toggle_theme', 'toggle_crt', 'toggle_neon']) {
        const item = GLOBAL_ITEMS.find(it => it.id === id);
        expect(item).toBeTruthy();
        expect(item.section).toBe('appearance');
        expect(item.actionKey).toBe(`tv:${id.replace(/_/g, '-')}`);
    }
});

test('GLOBAL_ITEMS: every id is unique', () => {
    const ids = GLOBAL_ITEMS.filter(it => it.id).map(it => it.id);
    expect(new Set(ids).size).toBe(ids.length);
});

test('GLOBAL_ITEMS: every actionKey starts with tv:', () => {
    for (const it of GLOBAL_ITEMS) {
        if (!it.actionKey) continue;
        expect(it.actionKey.startsWith('tv:')).toBe(true);
    }
});

// ── positionMenu ──────────────────────────────────────────────────

test('positionMenu: no overflow → uses click position', () => {
    expect(positionMenu(100, 100, 200, 200, 1024, 768)).toEqual({ x: 100, y: 100 });
});

test('positionMenu: right overflow → flips to fit viewport', () => {
    const p = positionMenu(900, 100, 200, 200, 1024, 768, 8);
    expect(p.x).toBeLessThanOrEqual(1024 - 200 - 8);
});

test('positionMenu: bottom overflow → flips to fit viewport', () => {
    const p = positionMenu(100, 700, 200, 200, 1024, 768, 8);
    expect(p.y).toBeLessThanOrEqual(768 - 200 - 8);
});

test('positionMenu: clamps to margin when off-screen left/top', () => {
    expect(positionMenu(-50, -50, 200, 200, 1024, 768, 8)).toEqual({ x: 8, y: 8 });
});

// ── compileMenu ───────────────────────────────────────────────────

test('compileMenu: drops hidden items', () => {
    const items = [
        { id: 'a', labelKey: 'a' },
        { id: 'b', labelKey: 'b', hidden: true },
        { id: 'c', labelKey: 'c' },
    ];
    expect(compileMenu(items).map(it => it.id)).toEqual(['a', 'c']);
});

test('compileMenu: collapses leading / trailing / duplicate separators', () => {
    const items = [
        { kind: 'separator' },
        { kind: 'separator' },
        { id: 'a', labelKey: 'a' },
        { kind: 'separator' },
        { kind: 'separator' },
        { id: 'b', labelKey: 'b' },
        { kind: 'separator' },
    ];
    const out = compileMenu(items);
    expect(out.length).toBe(3);
    expect(out[0].id).toBe('a');
    expect(out[1].kind).toBe('separator');
    expect(out[2].id).toBe('b');
});

test('compileMenu: empty / null safe', () => {
    expect(compileMenu([])).toEqual([]);
    expect(compileMenu(null)).toEqual([]);
});

// ── mergeMenu ─────────────────────────────────────────────────────

test('mergeMenu: empty custom → returns shallow copy of globals', () => {
    const g = [{ id: 'a', labelKey: 'a' }];
    const out = mergeMenu(g, []);
    expect(out).toEqual(g);
    out.push({ kind: 'separator' });
    expect(g.length).toBe(1);
});

test('mergeMenu: custom items get inserted at top with separator between', () => {
    const g = [{ id: 'g1', labelKey: 'g1' }];
    const c = [{ id: 'c1', labelKey: 'c1' }, { id: 'c2', labelKey: 'c2' }];
    const out = mergeMenu(g, c);
    expect(out.length).toBe(4);
    expect(out[0].id).toBe('c1');
    expect(out[1].id).toBe('c2');
    expect(out[2].kind).toBe('separator');
    expect(out[3].id).toBe('g1');
});

// ── EDITING_ITEMS / mergeMenuWithEditing ──────────────────────────

test('EDITING_ITEMS: contains cut/copy/paste/select-all/undo/redo', () => {
    const ids = EDITING_ITEMS.filter(it => it.kind !== 'separator').map(it => it.id);
    for (const want of ['edit_undo', 'edit_redo', 'edit_cut',
        'edit_copy', 'edit_paste', 'edit_select_all']) {
        expect(ids.includes(want)).toBe(true);
    }
});

test('EDITING_ITEMS: every non-separator has a labelKey + actionKey starting with tv:edit-', () => {
    for (const it of EDITING_ITEMS) {
        if (it.kind === 'separator') continue;
        expect(typeof it.labelKey).toBe('string');
        expect(it.actionKey.startsWith('tv:edit-')).toBe(true);
    }
});

test('mergeMenuWithEditing: editing items get prepended above custom + globals', () => {
    const g = [{ id: 'g1', labelKey: 'g1' }];
    const c = [{ id: 'c1', labelKey: 'c1' }];
    const e = [{ id: 'e1', labelKey: 'e1' }, { id: 'e2', labelKey: 'e2' }];
    const out = mergeMenuWithEditing(g, c, e);
    // editing block (2) + sep + custom (1) + sep + globals (1) = 6
    expect(out.length).toBe(6);
    expect(out[0].id).toBe('e1');
    expect(out[1].id).toBe('e2');
    expect(out[2].kind).toBe('separator');
    expect(out[3].id).toBe('c1');
    expect(out[4].kind).toBe('separator');
    expect(out[5].id).toBe('g1');
});

test('mergeMenuWithEditing: empty editing → behaves like mergeMenu', () => {
    const g = [{ id: 'g1', labelKey: 'g1' }];
    const c = [{ id: 'c1', labelKey: 'c1' }];
    expect(mergeMenuWithEditing(g, c, [])).toEqual(mergeMenu(g, c));
    expect(mergeMenuWithEditing(g, c, null)).toEqual(mergeMenu(g, c));
});

test('mergeMenuWithEditing: empty custom + editing → editing then globals', () => {
    const g = [{ id: 'g1', labelKey: 'g1' }];
    const e = [{ id: 'e1', labelKey: 'e1' }];
    const out = mergeMenuWithEditing(g, [], e);
    expect(out.length).toBe(3);
    expect(out[0].id).toBe('e1');
    expect(out[1].kind).toBe('separator');
    expect(out[2].id).toBe('g1');
});

// ── nextVisibleIdx ────────────────────────────────────────────────

test('nextVisibleIdx: wraps + skips separators', () => {
    const items = [
        { id: 'a', labelKey: 'a' },
        { kind: 'separator' },
        { id: 'b', labelKey: 'b' },
        { kind: 'separator' },
        { id: 'c', labelKey: 'c' },
    ];
    // From idx 0 (a) → +1 → b (idx 2).
    expect(nextVisibleIdx(items, 0,  1)).toBe(2);
    // From idx 2 → +1 → c (idx 4).
    expect(nextVisibleIdx(items, 2,  1)).toBe(4);
    // From idx 4 → +1 → wraps to a (idx 0).
    expect(nextVisibleIdx(items, 4,  1)).toBe(0);
    // From idx 0 → -1 → wraps to c (idx 4).
    expect(nextVisibleIdx(items, 0, -1)).toBe(4);
});

test('nextVisibleIdx: -1 (no current) → first visible going forward', () => {
    const items = [{ id: 'a', labelKey: 'a' }];
    expect(nextVisibleIdx(items, -1, 1)).toBe(0);
});

test('nextVisibleIdx: all-separator menu safely returns 0', () => {
    expect(nextVisibleIdx([{ kind: 'separator' }], 0, 1)).toBe(0);
    expect(nextVisibleIdx([], 0, 1)).toBe(0);
});

// ── findMnemonic ──────────────────────────────────────────────────

test('findMnemonic: first letter of last labelKey segment matches', () => {
    const items = [
        { id: 'a', labelKey: 'ctxmenu.reload' },
        { id: 'b', labelKey: 'ctxmenu.copy' },
    ];
    expect(findMnemonic(items, 'r').id).toBe('a');
    expect(findMnemonic(items, 'c').id).toBe('b');
});

test('findMnemonic: no match → null', () => {
    const items = [{ id: 'a', labelKey: 'ctxmenu.reload' }];
    expect(findMnemonic(items, 'z')).toBeNull();
});

test('findMnemonic: skips separators', () => {
    const items = [{ kind: 'separator' }, { id: 'a', labelKey: 'ctxmenu.reload' }];
    expect(findMnemonic(items, 'r').id).toBe('a');
});

test('findMnemonic: case-insensitive', () => {
    const items = [{ id: 'a', labelKey: 'ctxmenu.Reload' }];
    expect(findMnemonic(items, 'R').id).toBe('a');
});
