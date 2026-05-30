// Regression-prevention test: every CATEGORIES entry in launcher.js must
// have a `view.launcher.category.<cat>` key in app_i18n_en.json.
//
// launcher.js falls back to the hardcoded English category label when the
// key is missing — no runtime crash, but the category header silently loses
// translatability. Pinning this catches the next new-category-added-without-
// i18n regression.

import { test, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

const launcher = readFileSync(join(__dirname, '../js/views/launcher.js'), 'utf8');
const CATALOG_KEYS = new Set(Object.keys(
    JSON.parse(readFileSync(join(__dirname, '../i18n/app_i18n_en.json'), 'utf8'))
));

function parseCategoryIds() {
    // CATEGORIES shape: `['cat_id', 'label', [tile_id, ...]]` — match the
    // outer structure so we don't accidentally pick up tile_ids inside the
    // nested array (a naive `[\s*'(id)'` regex would).
    const block = launcher.match(/export const CATEGORIES = \[([\s\S]+?)\n\];/);
    if (!block) throw new Error('CATEGORIES export not found');
    const re = /\[\s*'([a-z0-9_-]+)'\s*,\s*'[^']*'\s*,\s*\[/g;
    return [...block[1].matchAll(re)].map(m => m[1]);
}

test('every CATEGORIES id has view.launcher.category.<id> in catalog', () => {
    const ids = parseCategoryIds();
    const missing = ids.filter(id => !CATALOG_KEYS.has(`view.launcher.category.${id}`));
    if (missing.length > 0) {
        const lines = missing.map(id => `  view.launcher.category.${id}`).join('\n');
        throw new Error(`${missing.length} categories without i18n key:\n${lines}`);
    }
    expect(missing).toEqual([]);
});
