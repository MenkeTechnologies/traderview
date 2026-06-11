// Regression-prevention test: every topbar nav tab (`data-view` in
// index.html, including the "More" menu items) must have a matching
// entry in launcher.js's TILES array. Otherwise the tab is reachable
// from the nav strip but has no launcher tile AND no Dashboards widget
// (the "add tile" picker sources from TILES), so it silently can't be
// pinned to a board — the "Golden Stars has no widget/tile" bug class.
//
// Sibling: no_orphan_tile_routes.spec.js asserts the forward direction
// (TILES → router). This asserts nav → TILES.

import { test, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

const html = readFileSync(join(__dirname, '../index.html'), 'utf8');
const launcher = readFileSync(join(__dirname, '../js/views/launcher.js'), 'utf8');
const app = readFileSync(join(__dirname, '../js/app.js'), 'utf8');

// `launcher` is the Home tile-grid host, not a tile.
const NON_TILE_VIEWS = new Set(['launcher']);

function parseNavViews() {
    return [...html.matchAll(/data-view="([a-z0-9_-]+)"/g)]
        .map(m => m[1])
        .filter(v => !NON_TILE_VIEWS.has(v));
}

function parseTileIds() {
    const block = launcher.match(/export const TILES = \[([\s\S]+?)\n\];/);
    if (!block) throw new Error('TILES export not found');
    return new Set([...block[1].matchAll(/\[\s*'([a-z0-9_-]+)'/g)].map(m => m[1]));
}

function parseViewRendererKeys() {
    const block = app.match(/export const viewRenderers = \{([\s\S]+?)\n\};/);
    if (!block) throw new Error('viewRenderers export not found');
    return new Set([...block[1].matchAll(/^\s*'([a-z0-9_-]+)'\s*:/gm)].map(m => m[1]));
}

test('every nav tab data-view has a TILES entry', () => {
    const tiles = parseTileIds();
    const missing = [...new Set(parseNavViews())].filter(v => !tiles.has(v));
    if (missing.length > 0) {
        throw new Error(`${missing.length} nav tab(s) without a launcher tile: ${missing.join(', ')}`);
    }
    expect(missing).toEqual([]);
});

test('every nav tab data-view has a viewRenderers entry (addable as a dashboard widget)', () => {
    const renderers = parseViewRendererKeys();
    const missing = [...new Set(parseNavViews())].filter(v => !renderers.has(v));
    if (missing.length > 0) {
        throw new Error(`${missing.length} nav tab(s) not addable as a dashboard widget (missing from viewRenderers): ${missing.join(', ')}`);
    }
    expect(missing).toEqual([]);
});
