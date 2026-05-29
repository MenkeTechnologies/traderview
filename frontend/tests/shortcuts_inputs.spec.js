// Shortcut registry: matches(), formatKey(), DEFAULT_SHORTCUTS,
// loadShortcuts (localStorage overrides), firesInEditableContext.

import { test, expect } from 'vitest';
import {
    DEFAULT_SHORTCUTS, LS_KEY, VERSION, matches, formatKey, findMatch,
    loadShortcuts, saveOverrides, firesInEditableContext, isTextEntryTarget,
} from '../js/_shortcuts.js';

const k = (over = {}) => ({ key: 'k', meta: false, ctrl: false, shift: false, alt: false, ...over });
const sc = (id, keys, extras = {}) => ({ id, keys, scope: 'global', actionKey: `tv:${id}`, ...extras });

// ── matches ───────────────────────────────────────────────────────

test('matches: exact key + no modifiers', () => {
    expect(matches({ key: 'a' }, sc('x', k({ key: 'a' })))).toBe(true);
    expect(matches({ key: 'b' }, sc('x', k({ key: 'a' })))).toBe(false);
});

test('matches: case-insensitive on key', () => {
    expect(matches({ key: 'A' }, sc('x', k({ key: 'a' })))).toBe(true);
});

test('matches: meta+ctrl both true ⇒ either modifier satisfies (cross-platform)', () => {
    const cmdK = sc('x', k({ key: 'k', meta: true, ctrl: true }));
    expect(matches({ key: 'k', metaKey: true,  ctrlKey: false }, cmdK)).toBe(true); // Mac
    expect(matches({ key: 'k', metaKey: false, ctrlKey: true  }, cmdK)).toBe(true); // PC
    expect(matches({ key: 'k', metaKey: false, ctrlKey: false }, cmdK)).toBe(false);
});

test('matches: strict meta-only when ctrl=false', () => {
    const macOnly = sc('x', k({ key: 'k', meta: true, ctrl: false }));
    expect(matches({ key: 'k', metaKey: true,  ctrlKey: false }, macOnly)).toBe(true);
    expect(matches({ key: 'k', metaKey: false, ctrlKey: true  }, macOnly)).toBe(false);
});

test('matches: shift / alt strict equality', () => {
    const sk = sc('x', k({ key: 'a', shift: true }));
    expect(matches({ key: 'a', shiftKey: true  }, sk)).toBe(true);
    expect(matches({ key: 'a', shiftKey: false }, sk)).toBe(false);
});

test('matches: rejects malformed events / shortcuts safely', () => {
    expect(matches(null, sc('x', k()))).toBe(false);
    expect(matches({ key: 'a' }, null)).toBe(false);
    expect(matches({ key: 'a' }, sc('x', null))).toBe(false);
});

// ── formatKey ─────────────────────────────────────────────────────

test('formatKey: mac glyphs ⌘⇧⌥⌃ + uppercase', () => {
    expect(formatKey(sc('x', k({ key: 'k', meta: true, ctrl: true })), true)).toBe('⌘K');
    expect(formatKey(sc('x', k({ key: 'a', shift: true, alt: true })), true)).toBe('⇧⌥A');
    expect(formatKey(sc('x', k({ key: '/' })), true)).toBe('/');
});

test('formatKey: PC fallback uses Ctrl/Win/Shift/Alt', () => {
    expect(formatKey(sc('x', k({ key: 'k', meta: true, ctrl: true })), false)).toBe('Ctrl+K');
    expect(formatKey(sc('x', k({ key: 'a', shift: true })), false)).toBe('Shift+A');
});

test('formatKey: space key → ␣', () => {
    expect(formatKey(sc('x', k({ key: ' ' })))).toBe('␣');
});

// ── findMatch + scope ─────────────────────────────────────────────

test('findMatch: returns first matching global shortcut', () => {
    const sc1 = sc('a', k({ key: 'a' }));
    const sc2 = sc('b', k({ key: 'b' }));
    expect(findMatch({ key: 'a' }, [sc1, sc2])).toBe(sc1);
    expect(findMatch({ key: 'b' }, [sc1, sc2])).toBe(sc2);
    expect(findMatch({ key: 'c' }, [sc1, sc2])).toBeNull();
});

test('findMatch: scoped shortcuts ignored outside their scope', () => {
    const palette = sc('a', k({ key: 'a' }), { scope: 'palette' });
    expect(findMatch({ key: 'a' }, [palette], 'global')).toBeNull();
    expect(findMatch({ key: 'a' }, [palette], 'palette')).toBe(palette);
});

test('findMatch: enabled=false skips shortcut', () => {
    const disabled = { ...sc('a', k({ key: 'a' })), enabled: false };
    expect(findMatch({ key: 'a' }, [disabled])).toBeNull();
});

// ── DEFAULT_SHORTCUTS ─────────────────────────────────────────────

test('defaults: command_palette is Cmd-or-Ctrl + K', () => {
    const cp = DEFAULT_SHORTCUTS.find(s => s.id === 'command_palette');
    expect(cp.keys.key).toBe('k');
    expect(cp.keys.meta && cp.keys.ctrl).toBe(true);
    expect(cp.actionKey).toBe('tv:open-palette');
});

test('defaults: help is "?" no modifiers', () => {
    const h = DEFAULT_SHORTCUTS.find(s => s.id === 'help');
    expect(h.keys).toEqual({ key: '?', meta: false, ctrl: false, shift: false, alt: false });
});

test('defaults: escape is bare Escape', () => {
    const e = DEFAULT_SHORTCUTS.find(s => s.id === 'escape');
    expect(e.keys.key).toBe('Escape');
});

test('defaults: toggle_favorite is Cmd-or-Ctrl + D', () => {
    const sc = DEFAULT_SHORTCUTS.find(s => s.id === 'toggle_favorite');
    expect(sc.keys.key).toBe('d');
    expect(sc.keys.meta && sc.keys.ctrl).toBe(true);
    expect(sc.actionKey).toBe('tv:toggle-favorite');
});

test('defaults: open_new_tab is Cmd-or-Ctrl + N → tv:open-new-tab', () => {
    const sc = DEFAULT_SHORTCUTS.find(s => s.id === 'open_new_tab');
    expect(sc.keys.key).toBe('n');
    expect(sc.actionKey).toBe('tv:open-new-tab');
});

test('defaults: add_bookmark is Cmd-or-Ctrl + B → tv:add-bookmark', () => {
    const sc = DEFAULT_SHORTCUTS.find(s => s.id === 'add_bookmark');
    expect(sc.keys.key).toBe('b');
    expect(sc.keys.meta && sc.keys.ctrl).toBe(true);
    expect(sc.actionKey).toBe('tv:add-bookmark');
});

test('defaults: theme/crt/neon toggles use Shift to avoid common conflicts', () => {
    for (const id of ['toggle_theme', 'toggle_crt', 'toggle_neon']) {
        const sc = DEFAULT_SHORTCUTS.find(s => s.id === id);
        expect(sc).toBeTruthy();
        expect(sc.keys.shift).toBe(true);
        expect(sc.keys.meta && sc.keys.ctrl).toBe(true);
        expect(sc.actionKey).toBe(`tv:${id.replace(/_/g, '-')}`);
    }
});

test('defaults: open_settings is Cmd-or-Ctrl + , → tv:open-settings', () => {
    const sc = DEFAULT_SHORTCUTS.find(s => s.id === 'open_settings');
    expect(sc.keys.key).toBe(',');
    expect(sc.keys.meta && sc.keys.ctrl).toBe(true);
    expect(sc.actionKey).toBe('tv:open-settings');
});

test('defaults: focus_search shortcut bound (consumed by tv:focus-search listener)', () => {
    const sc = DEFAULT_SHORTCUTS.find(s => s.id === 'focus_search');
    expect(sc.keys.key).toBe('/');
    expect(sc.actionKey).toBe('tv:focus-search');
});

test('defaults: go_home is Cmd-or-Ctrl + Shift + H → tv:go-home', () => {
    const sc = DEFAULT_SHORTCUTS.find(s => s.id === 'go_home');
    expect(sc.keys.key).toBe('h');
    expect(sc.keys.shift).toBe(true);
    expect(sc.actionKey).toBe('tv:go-home');
});

test('defaults: every id is unique', () => {
    const ids = DEFAULT_SHORTCUTS.map(s => s.id);
    expect(new Set(ids).size).toBe(ids.length);
});

test('defaults: every actionKey starts with tv:', () => {
    for (const s of DEFAULT_SHORTCUTS) expect(s.actionKey.startsWith('tv:')).toBe(true);
});

test('defaults: registers all six tv:edit-* actions (palette-findable)', () => {
    const want = [
        { id: 'edit_cut',        actionKey: 'tv:edit-cut' },
        { id: 'edit_copy',       actionKey: 'tv:edit-copy' },
        { id: 'edit_paste',      actionKey: 'tv:edit-paste' },
        { id: 'edit_select_all', actionKey: 'tv:edit-select-all' },
        { id: 'edit_undo',       actionKey: 'tv:edit-undo' },
        { id: 'edit_redo',       actionKey: 'tv:edit-redo' },
    ];
    const byId = new Map(DEFAULT_SHORTCUTS.map(s => [s.id, s]));
    for (const w of want) {
        const sc = byId.get(w.id);
        expect(sc, `missing shortcut id=${w.id}`).toBeTruthy();
        expect(sc.actionKey).toBe(w.actionKey);
        // No default key binding — palette-only.
        expect(sc.keys.key).toBe(null);
        expect(sc.scope).toBe('global');
        expect(sc.descKey).toBe(`shortcut.${w.id}`);
    }
});

test('defaults: every key:null shortcut still has a descKey for palette label', () => {
    for (const s of DEFAULT_SHORTCUTS) {
        if (s.keys.key !== null) continue;
        expect(s.descKey).toBeTruthy();
        expect(s.descKey.startsWith('shortcut.')).toBe(true);
    }
});

// ── loadShortcuts (overrides via localStorage) ────────────────────

test('loadShortcuts: no overrides → defaults', () => {
    const out = loadShortcuts(() => null);
    expect(out.length).toBe(DEFAULT_SHORTCUTS.length);
    expect(out[0].actionKey).toBe(DEFAULT_SHORTCUTS[0].actionKey);
});

test('loadShortcuts: override replaces keys, preserves actionKey + descKey', () => {
    const fake = JSON.stringify({
        version: VERSION,
        overrides: { command_palette: { key: 'p', meta: true, ctrl: true } },
    });
    const out = loadShortcuts(() => fake);
    const cp = out.find(s => s.id === 'command_palette');
    expect(cp.keys.key).toBe('p');
    expect(cp.actionKey).toBe('tv:open-palette');
});

test('loadShortcuts: malformed JSON falls back to defaults', () => {
    const out = loadShortcuts(() => 'not json');
    expect(out.length).toBe(DEFAULT_SHORTCUTS.length);
});

test('loadShortcuts: wrong version → ignore overrides', () => {
    const fake = JSON.stringify({ version: 999, overrides: { command_palette: { key: 'z' } } });
    const out = loadShortcuts(() => fake);
    expect(out.find(s => s.id === 'command_palette').keys.key).toBe('k');
});

test('saveOverrides: writes versioned envelope', () => {
    let written = null;
    saveOverrides({ command_palette: { key: 'p', meta: true, ctrl: true } },
        (key, val) => { written = { key, val }; });
    expect(written.key).toBe(LS_KEY);
    const parsed = JSON.parse(written.val);
    expect(parsed.version).toBe(VERSION);
    expect(parsed.overrides.command_palette.key).toBe('p');
});

// ── editable-context gating ───────────────────────────────────────

test('isTextEntryTarget: input/textarea/select/contentEditable true', () => {
    expect(isTextEntryTarget({ tagName: 'INPUT' })).toBe(true);
    expect(isTextEntryTarget({ tagName: 'TEXTAREA' })).toBe(true);
    expect(isTextEntryTarget({ tagName: 'SELECT' })).toBe(true);
    expect(isTextEntryTarget({ tagName: 'DIV', isContentEditable: true })).toBe(true);
    expect(isTextEntryTarget({ tagName: 'BUTTON' })).toBe(false);
    expect(isTextEntryTarget(null)).toBe(false);
});

test('firesInEditableContext: Escape + Cmd+K-style true, plain letters false', () => {
    expect(firesInEditableContext(sc('escape', k({ key: 'Escape' })))).toBe(true);
    expect(firesInEditableContext(sc('cp', k({ key: 'k', meta: true, ctrl: true })))).toBe(true);
    expect(firesInEditableContext(sc('plain', k({ key: 'a' })))).toBe(false);
});
