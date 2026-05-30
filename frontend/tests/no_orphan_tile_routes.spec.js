// Regression-prevention test: every TILES viewId in launcher.js must have
// a matching `case 'viewId':` clause in app.js's hash router. An orphan
// tile = user clicks it, hash changes, but the router falls through to
// the default branch and the tile silently no-ops or 404s.
//
// Sibling: no_orphan_categories.spec.js asserts TILES ↔ CATEGORIES.
// This one asserts TILES ↔ router.

import { test, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

const launcher = readFileSync(join(__dirname, '../js/views/launcher.js'), 'utf8');
const app = readFileSync(join(__dirname, '../js/app.js'), 'utf8');

function parseTileIds() {
    const block = launcher.match(/export const TILES = \[([\s\S]+?)\n\];/);
    if (!block) throw new Error('TILES export not found');
    return [...block[1].matchAll(/\[\s*'([a-z0-9_-]+)'/g)].map(m => m[1]);
}

function parseRouterCases() {
    // Match all `case 'id':` inside the app.js router switch.
    // The router lives inside `mountView` — naïve full-file scan is fine
    // because no other top-level switch uses lowercase-kebab string cases.
    return [...app.matchAll(/case\s+'([a-z0-9_-]+)':/g)].map(m => m[1]);
}

test('every TILES viewId has a router case in app.js', () => {
    const tiles = parseTileIds();
    const cases = new Set(parseRouterCases());
    const missing = tiles.filter(t => !cases.has(t));
    if (missing.length > 0) {
        const lines = missing.slice(0, 20).map(t => `  ${t}`).join('\n');
        const tail = missing.length > 20 ? `\n  … and ${missing.length - 20} more` : '';
        throw new Error(`${missing.length} TILES without router case (would 404 on click):\n${lines}${tail}`);
    }
    expect(missing).toEqual([]);
});
