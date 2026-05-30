// Regression-prevention test: every shortcut.actionKey + ctxmenu actionKey
// must have a matching `window.addEventListener('tv:xxx', ...)` handler
// somewhere in JS source. An unwired actionKey = silent no-op when the
// user invokes the shortcut or right-click item.

import { test, expect } from 'vitest';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { DEFAULT_SHORTCUTS } from '../js/_shortcuts.js';
import { GLOBAL_ITEMS, EDITING_ITEMS, SYMBOL_ITEMS, TRADE_ROW_ITEMS } from '../js/_context_menu.js';

function walk(dir) {
    const out = [];
    for (const name of readdirSync(dir)) {
        const p = join(dir, name);
        const s = statSync(p);
        if (s.isDirectory()) out.push(...walk(p));
        else if (name.endsWith('.js')) out.push(p);
    }
    return out;
}

function collectHandlers() {
    const handlers = new Set();
    const files = walk(join(__dirname, '../js'));
    for (const f of files) {
        const src = readFileSync(f, 'utf8');
        for (const m of src.matchAll(/addEventListener\(\s*['"](tv:[a-z0-9-]+)['"]/g)) {
            handlers.add(m[1]);
        }
    }
    return handlers;
}

test('every DEFAULT_SHORTCUTS actionKey has a window event handler', () => {
    const handlers = collectHandlers();
    const missing = [];
    for (const sc of DEFAULT_SHORTCUTS) {
        if (!sc.actionKey) continue;
        if (!handlers.has(sc.actionKey)) {
            missing.push({ id: sc.id, actionKey: sc.actionKey });
        }
    }
    if (missing.length > 0) {
        const lines = missing.map(m => `  ${m.id} → ${m.actionKey}`).join('\n');
        throw new Error(`${missing.length} shortcut actionKey(s) without a handler:\n${lines}`);
    }
    expect(missing).toEqual([]);
});

test('every ctxmenu item actionKey has a window event handler', () => {
    const handlers = collectHandlers();
    const missing = [];
    for (const it of [...GLOBAL_ITEMS, ...EDITING_ITEMS, ...SYMBOL_ITEMS, ...TRADE_ROW_ITEMS]) {
        if (it.kind === 'separator' || !it.actionKey) continue;
        if (!handlers.has(it.actionKey)) {
            missing.push({ id: it.id, actionKey: it.actionKey });
        }
    }
    if (missing.length > 0) {
        const lines = missing.map(m => `  ${m.id} → ${m.actionKey}`).join('\n');
        throw new Error(`${missing.length} ctxmenu actionKey(s) without a handler:\n${lines}`);
    }
    expect(missing).toEqual([]);
});
