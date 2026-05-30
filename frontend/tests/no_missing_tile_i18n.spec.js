// Regression-prevention test: every TILES entry in launcher.js must have
// `tile.<id>.label` and `tile.<id>.desc` keys in app_i18n_en.json.
//
// launcher.js falls back to the English literal in TILES when the i18n
// key is missing, so a missing key isn't a runtime crash — but it IS a
// regression: the tile is no longer translatable. Pinning this prevents
// silent loss of i18n coverage as new tiles ship.
//
// Sibling: no_orphan_categories.spec.js (TILES ↔ CATEGORIES structure).

import { test, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

const launcher = readFileSync(join(__dirname, '../js/views/launcher.js'), 'utf8');
const CATALOG_KEYS = new Set(Object.keys(
    JSON.parse(readFileSync(join(__dirname, '../i18n/app_i18n_en.json'), 'utf8'))
));

function parseTileIds() {
    const block = launcher.match(/export const TILES = \[([\s\S]+?)\n\];/);
    if (!block) throw new Error('TILES export not found');
    return [...block[1].matchAll(/\[\s*'([a-z0-9_-]+)'/g)].map(m => m[1]);
}

test('every TILES id has tile.<id>.label in catalog', () => {
    const ids = parseTileIds();
    const missing = ids.filter(id => !CATALOG_KEYS.has(`tile.${id}.label`));
    if (missing.length > 0) {
        const lines = missing.slice(0, 30).map(id => `  tile.${id}.label`).join('\n');
        const tail = missing.length > 30 ? `\n  … and ${missing.length - 30} more` : '';
        throw new Error(`${missing.length} TILES without .label i18n key:\n${lines}${tail}`);
    }
    expect(missing).toEqual([]);
});

test('every TILES id has tile.<id>.desc in catalog', () => {
    const ids = parseTileIds();
    const missing = ids.filter(id => !CATALOG_KEYS.has(`tile.${id}.desc`));
    if (missing.length > 0) {
        const lines = missing.slice(0, 30).map(id => `  tile.${id}.desc`).join('\n');
        const tail = missing.length > 30 ? `\n  … and ${missing.length - 30} more` : '';
        throw new Error(`${missing.length} TILES without .desc i18n key:\n${lines}${tail}`);
    }
    expect(missing).toEqual([]);
});
