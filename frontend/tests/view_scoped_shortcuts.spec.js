// Pins the view-scoped shortcut wiring: each non-global entry in the
// DEFAULT_SHORTCUTS registry must reference a router case (so the
// scope name is meaningful) AND only fire under findMatch when the
// dispatcher's setScope(view) matches.

import { test, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { DEFAULT_SHORTCUTS, findMatch } from '../js/_shortcuts.js';

const APP = readFileSync(join(__dirname, '../js/app.js'), 'utf8');
const ROUTER_CASES = new Set(
    [...APP.matchAll(/case\s+'([a-z0-9_-]+)':/g)].map(m => m[1])
);

// Scopes that aren't view-IDs but are still legitimate (palette and
// editor scopes are dispatcher-controlled, not view-router controlled).
const NON_VIEW_SCOPES = new Set(['global', 'palette', 'editor']);

test('every view-scoped shortcut targets a real router case', () => {
    const viewScoped = DEFAULT_SHORTCUTS.filter(sc => !NON_VIEW_SCOPES.has(sc.scope));
    const orphans = viewScoped.filter(sc => !ROUTER_CASES.has(sc.scope));
    if (orphans.length > 0) {
        const lines = orphans.map(o => `  ${o.id} (scope='${o.scope}')`).join('\n');
        throw new Error(`${orphans.length} view-scoped shortcut(s) reference unknown router case:\n${lines}`);
    }
    expect(orphans).toEqual([]);
});

test('trades_new fires under scope=trades, not under global', () => {
    const sc = DEFAULT_SHORTCUTS.find(s => s.id === 'trades_new');
    expect(sc).toBeDefined();
    const ev = { key: 'n', metaKey: false, ctrlKey: false, shiftKey: false, altKey: false };
    expect(findMatch(ev, DEFAULT_SHORTCUTS, 'trades')).toEqual(sc);
    // Under the global default scope, plain `n` must NOT fire trades_new.
    const hit = findMatch(ev, DEFAULT_SHORTCUTS, 'global');
    expect(hit && hit.id).not.toBe('trades_new');
});

test('dashboard_refresh fires under scope=dashboard, not under scope=trades', () => {
    const sc = DEFAULT_SHORTCUTS.find(s => s.id === 'dashboard_refresh');
    expect(sc).toBeDefined();
    const ev = { key: 'r', metaKey: false, ctrlKey: false, shiftKey: false, altKey: false };
    expect(findMatch(ev, DEFAULT_SHORTCUTS, 'dashboard')).toEqual(sc);
    const hit = findMatch(ev, DEFAULT_SHORTCUTS, 'trades');
    expect(hit && hit.id).not.toBe('dashboard_refresh');
});

test('view-scoped shortcuts are not shadowed by a global with the same key', () => {
    // For each view-scoped entry, scan globals for a colliding (modifier-free) binding.
    const viewScoped = DEFAULT_SHORTCUTS.filter(sc => !NON_VIEW_SCOPES.has(sc.scope));
    const collisions = [];
    for (const sc of viewScoped) {
        if (!sc.keys || sc.keys.key == null) continue;
        const collide = DEFAULT_SHORTCUTS.find(g =>
            g.scope === 'global' &&
            g.keys && g.keys.key &&
            g.keys.key.toLowerCase() === sc.keys.key.toLowerCase() &&
            !!g.keys.meta === !!sc.keys.meta &&
            !!g.keys.ctrl === !!sc.keys.ctrl &&
            !!g.keys.shift === !!sc.keys.shift &&
            !!g.keys.alt === !!sc.keys.alt);
        if (collide) collisions.push({ sc: sc.id, global: collide.id });
    }
    if (collisions.length > 0) {
        const lines = collisions.map(c => `  ${c.sc} ↔ global ${c.global}`).join('\n');
        throw new Error(`${collisions.length} view-scoped shortcut(s) shadowed by global:\n${lines}`);
    }
    expect(collisions).toEqual([]);
});
