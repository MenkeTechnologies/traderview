// Regression-prevention test: every non-separator ctxmenu item must have
// (a) a labelKey for i18n, and (b) at least one of {actionKey, navTo, onClick}.
// An item with no labelKey renders as raw "ctxmenu.X" or blank; an item
// with no action does nothing when clicked.
//
// Also pins: every navTo target maps to a router case in app.js
// (otherwise the item navigates to a 404 hash).

import { test, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { GLOBAL_ITEMS, EDITING_ITEMS, SYMBOL_ITEMS, ALL_SCOPED_ITEMS, SYMBOL_AWARE_SCOPES } from '../js/_context_menu.js';

const SCOPED_ROW_ITEMS = ALL_SCOPED_ITEMS.flatMap(([, items]) => items);

const app = readFileSync(join(__dirname, '../js/app.js'), 'utf8');
const ROUTER_CASES = new Set(
    [...app.matchAll(/case\s+'([a-z0-9_-]+)':/g)].map(m => m[1])
);

test('every non-separator ctxmenu item has a labelKey', () => {
    const items = [...GLOBAL_ITEMS, ...EDITING_ITEMS, ...SYMBOL_ITEMS, ...SCOPED_ROW_ITEMS].filter(it => it.kind !== 'separator');
    const missing = items.filter(it => !it.labelKey);
    if (missing.length > 0) {
        const lines = missing.map(m => `  ${m.id || JSON.stringify(m)}`).join('\n');
        throw new Error(`${missing.length} ctxmenu item(s) without labelKey:\n${lines}`);
    }
    expect(missing).toEqual([]);
});

test('every non-separator ctxmenu item has at least one action sink', () => {
    const items = [...GLOBAL_ITEMS, ...EDITING_ITEMS, ...SYMBOL_ITEMS, ...SCOPED_ROW_ITEMS].filter(it => it.kind !== 'separator');
    const dead = items.filter(it => !it.actionKey && !it.navTo && !it.onClick);
    if (dead.length > 0) {
        const lines = dead.map(d => `  ${d.id} (no actionKey, navTo, or onClick)`).join('\n');
        throw new Error(`${dead.length} ctxmenu item(s) with no action sink:\n${lines}`);
    }
    expect(dead).toEqual([]);
});

test('every ctxmenu navTo target maps to a router case in app.js', () => {
    const items = [...GLOBAL_ITEMS, ...EDITING_ITEMS, ...SYMBOL_ITEMS, ...SCOPED_ROW_ITEMS].filter(it => it.navTo);
    const missing = items.filter(it => !ROUTER_CASES.has(it.navTo));
    if (missing.length > 0) {
        const lines = missing.map(m => `  ${m.id} → navTo:'${m.navTo}'`).join('\n');
        throw new Error(`${missing.length} ctxmenu item(s) navigate to non-existent route:\n${lines}`);
    }
    expect(missing).toEqual([]);
});

test('every SYMBOL_AWARE_SCOPES entry maps to a router case in app.js', () => {
    const missing = SYMBOL_AWARE_SCOPES.filter(s => !ROUTER_CASES.has(s));
    if (missing.length > 0) {
        throw new Error(`${missing.length} SYMBOL_AWARE_SCOPES not in router:\n  ${missing.join('\n  ')}`);
    }
    expect(missing).toEqual([]);
});
