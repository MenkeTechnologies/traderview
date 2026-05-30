// Regression-prevention test: no two BOUND shortcuts in DEFAULT_SHORTCUTS
// share the same (key + modifiers + scope) signature. Two bindings on the
// same combo fight at dispatch time — only one will fire, the other is dead.
//
// Unbound shortcuts (keys.key === null) are skipped — those are intentionally
// command-palette-only until the user customizes them.

import { test, expect } from 'vitest';
import { DEFAULT_SHORTCUTS } from '../js/_shortcuts.js';

function sig(sc) {
    const k = sc.keys;
    const mods = [
        k.meta  ? 'M' : '',
        k.ctrl  ? 'C' : '',
        k.shift ? 'S' : '',
        k.alt   ? 'A' : '',
    ].join('');
    return `${mods}-${k.key}-${sc.scope}`;
}

test('no two BOUND shortcuts share the same key+modifiers+scope signature', () => {
    const buckets = new Map();
    for (const sc of DEFAULT_SHORTCUTS) {
        if (sc.keys.key === null || sc.keys.key === undefined) continue;
        const s = sig(sc);
        if (!buckets.has(s)) buckets.set(s, []);
        buckets.get(s).push(sc.id);
    }
    const dups = [...buckets.entries()].filter(([, ids]) => ids.length > 1);
    if (dups.length > 0) {
        const lines = dups.map(([s, ids]) => `  ${s} → ${ids.join(', ')}`).join('\n');
        throw new Error(`${dups.length} shortcut signature collision(s):\n${lines}`);
    }
    expect(dups).toEqual([]);
});

test('every shortcut has a unique id', () => {
    const counts = new Map();
    for (const sc of DEFAULT_SHORTCUTS) {
        counts.set(sc.id, (counts.get(sc.id) || 0) + 1);
    }
    const dups = [...counts.entries()].filter(([, n]) => n > 1);
    if (dups.length > 0) {
        const lines = dups.map(([id, n]) => `  ${id} (${n}×)`).join('\n');
        throw new Error(`${dups.length} duplicate shortcut id(s):\n${lines}`);
    }
    expect(dups).toEqual([]);
});

test('every shortcut has a unique actionKey', () => {
    const counts = new Map();
    for (const sc of DEFAULT_SHORTCUTS) {
        if (!sc.actionKey) continue;
        counts.set(sc.actionKey, (counts.get(sc.actionKey) || 0) + 1);
    }
    const dups = [...counts.entries()].filter(([, n]) => n > 1);
    if (dups.length > 0) {
        const lines = dups.map(([k, n]) => `  ${k} (${n}×)`).join('\n');
        throw new Error(`${dups.length} duplicate actionKey(s):\n${lines}`);
    }
    expect(dups).toEqual([]);
});
