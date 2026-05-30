// Regression-prevention test: every view ID listed in CATEGORIES must
// have a matching tile in TILES. Otherwise the launcher category header
// shows up but the row of tiles under it is empty (or includes ghost
// references that error when clicked).
//
// And vice versa: every TILE id should be reachable from at least one
// CATEGORY — orphan tiles never appear in the launcher even though
// they exist as routes.

import { test, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

// Parse TILES + CATEGORIES from launcher.js source rather than importing,
// because launcher.js's transitive imports pull in app.js + dashboards.js
// which require a real DOM. Source-text parsing keeps this test pure.
const src = readFileSync(join(__dirname, '../js/views/launcher.js'), 'utf8');

function parseTileIds() {
    // Match TILES array entries: `['id', 'label', 'glyph', 'desc', 'badge']`
    const tilesBlock = src.match(/export const TILES = \[([\s\S]+?)\n\];/);
    if (!tilesBlock) throw new Error('TILES export not found');
    const out = [];
    const re = /\[\s*'([a-z0-9_-]+)'/g;
    let m;
    while ((m = re.exec(tilesBlock[1])) !== null) out.push(m[1]);
    return out;
}

function parseCategories() {
    // Match CATEGORIES array: `[ 'catId', 'label', ['viewId1', 'viewId2', ...] ]`
    const catsBlock = src.match(/export const CATEGORIES = \[([\s\S]+?)\n\];/);
    if (!catsBlock) throw new Error('CATEGORIES export not found');
    const out = [];
    const re = /\[\s*'([a-z0-9_-]+)'\s*,\s*'[^']*'\s*,\s*\[([^\]]+)\]\s*\]/g;
    let m;
    while ((m = re.exec(catsBlock[1])) !== null) {
        const catId = m[1];
        const ids = [...m[2].matchAll(/'([a-z0-9_-]+)'/g)].map(x => x[1]);
        out.push([catId, ids]);
    }
    return out;
}

const TILE_IDS_LIST = parseTileIds();
const tileIds = new Set(TILE_IDS_LIST);
const CATS = parseCategories();
const cats = new Set();
for (const [, viewIds] of CATS) for (const v of viewIds) cats.add(v);

test('every CATEGORIES viewId has a matching TILES entry', () => {
    const missing = [...cats].filter(v => !tileIds.has(v));
    if (missing.length > 0) {
        throw new Error(`Categories reference ${missing.length} unknown viewId(s): ${missing.join(', ')}`);
    }
    expect(missing).toEqual([]);
});

test('every TILES entry appears in at least one CATEGORY', () => {
    const orphans = [...tileIds].filter(t => !cats.has(t));
    if (orphans.length > 0) {
        throw new Error(`${orphans.length} TILES are orphaned from categories (will not show in launcher): ${orphans.join(', ')}`);
    }
    expect(orphans).toEqual([]);
});

test('no duplicate viewIds across CATEGORIES (would render twice)', () => {
    const seen = new Map();  // viewId → categories that contain it
    for (const [catId, viewIds] of CATS) {
        for (const v of viewIds) {
            if (!seen.has(v)) seen.set(v, []);
            seen.get(v).push(catId);
        }
    }
    const dups = [...seen.entries()].filter(([_, c]) => c.length > 1);
    if (dups.length > 0) {
        const lines = dups.map(([v, c]) => `  ${v} → ${c.join(', ')}`).join('\n');
        throw new Error(`${dups.length} viewIds appear in multiple categories:\n${lines}`);
    }
    expect(dups).toEqual([]);
});

test('no duplicate TILE ids (would create palette confusion)', () => {
    const counts = new Map();
    for (const id of TILE_IDS_LIST) counts.set(id, (counts.get(id) || 0) + 1);
    const dups = [...counts.entries()].filter(([, n]) => n > 1);
    if (dups.length > 0) {
        throw new Error(`${dups.length} duplicate TILE ids: ${dups.map(([id, n]) => `${id} (${n}×)`).join(', ')}`);
    }
    expect(dups).toEqual([]);
});
