// Regression-prevention test: every shortcut's `descKey` field in
// DEFAULT_SHORTCUTS must resolve to a key in `app_i18n_en.json`.
// Otherwise the keyboard-shortcuts view shows the raw key string
// ("shortcut.foo") instead of the human-readable description.
//
// Sibling tests:
//   no_missing_i18n_keys.spec.js     — DOM `data-i18n*` + `data-tip` keys
//   no_missing_shortcut_ids.spec.js  — DOM `data-shortcut` IDs

import { test, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { DEFAULT_SHORTCUTS } from '../js/_shortcuts.js';

test('every shortcut.descKey resolves to app_i18n_en.json', () => {
    const catalog = JSON.parse(readFileSync(join(__dirname, '../i18n/app_i18n_en.json'), 'utf8'));
    const catalogKeys = new Set(Object.keys(catalog));
    const missing = [];
    for (const sc of DEFAULT_SHORTCUTS) {
        if (!sc.descKey) continue;  // descKey is optional per the schema
        if (!catalogKeys.has(sc.descKey)) {
            missing.push({ id: sc.id, descKey: sc.descKey });
        }
    }
    if (missing.length > 0) {
        const lines = missing.map(m => `  ${m.id} → ${m.descKey}`).join('\n');
        throw new Error(`Found ${missing.length} shortcut descKey(s) not in catalog:\n${lines}`);
    }
    expect(missing).toEqual([]);
});

test('every ctxmenu item labelKey resolves to app_i18n_en.json', async () => {
    // Sibling check: context menu items have labelKey instead of descKey.
    const catalog = JSON.parse(readFileSync(join(__dirname, '../i18n/app_i18n_en.json'), 'utf8'));
    const catalogKeys = new Set(Object.keys(catalog));
    const { GLOBAL_ITEMS, EDITING_ITEMS } = await import('../js/_context_menu.js');
    const missing = [];
    for (const it of [...GLOBAL_ITEMS, ...EDITING_ITEMS]) {
        if (it.kind === 'separator' || !it.labelKey) continue;
        if (!catalogKeys.has(it.labelKey)) {
            missing.push({ id: it.id, labelKey: it.labelKey });
        }
    }
    if (missing.length > 0) {
        const lines = missing.map(m => `  ${m.id} → ${m.labelKey}`).join('\n');
        throw new Error(`Found ${missing.length} ctxmenu labelKey(s) not in catalog:\n${lines}`);
    }
    expect(missing).toEqual([]);
});
