// Tests for the shared calculator-enhancement engine (calc_enhance.js). Four+
// calculator views depend on these helpers for charts, CSV export, and the
// two-axis sensitivity grid, so a regression here breaks every enhanced tile.
// Only the DOM-free pure helpers are exercised — chart/export/permalink DOM
// paths are integration concerns, not unit-testable in plain node.
//
// Run: `node --test frontend/tests/calc_enhance.test.mjs`

import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
    linspace,
    toCsv,
    runSensitivity,
    renderSensitivityTable,
    svgLineChart,
    svgBarChart,
} from '../js/calc_enhance.js';

// ─── linspace ────────────────────────────────────────────────────────────

test('linspace returns inclusive evenly spaced range', () => {
    assert.deepEqual(linspace(0, 10, 5), [0, 2.5, 5, 7.5, 10]);
});

test('linspace with one step returns the low endpoint only', () => {
    assert.deepEqual(linspace(7, 99, 1), [7]);
});

test('linspace endpoints are exact (first=lo, last=hi)', () => {
    const r = linspace(100, 400, 7);
    assert.equal(r[0], 100);
    assert.equal(r[r.length - 1], 400);
    assert.equal(r.length, 7);
});

// ─── toCsv (RFC-4180-style quoting) ──────────────────────────────────────

test('toCsv leaves simple cells unquoted', () => {
    assert.equal(toCsv([['metric', 'value'], ['magic_number', 1.5]]), 'metric,value\nmagic_number,1.5');
});

test('toCsv quotes and escapes commas, quotes, and newlines', () => {
    assert.equal(
        toCsv([['a,b', 'x"y', 'l\nm']]),
        '"a,b","x""y","l\nm"',
    );
});

test('toCsv renders null/undefined cells as empty', () => {
    assert.equal(toCsv([[null, undefined, 0]]), ',,0');
});

// ─── runSensitivity (two-axis matrix) ────────────────────────────────────

test('runSensitivity builds a yVals×xVals matrix from compute+pick', async () => {
    const compute = async (b) => ({ product: b.x * b.y, valid: true });
    const { cells } = await runSensitivity({
        base: { x: 0, y: 0, other: 'keep' },
        xKey: 'x', yKey: 'y',
        xVals: [1, 2, 3], yVals: [10, 20],
        compute, pick: (r) => (r.valid ? r.product : null),
    });
    assert.deepEqual(cells, [[10, 20, 30], [20, 40, 60]]);
});

test('runSensitivity records null for failed/invalid cells', async () => {
    const compute = async (b) => (b.x === 2 ? { valid: false } : { v: b.x, valid: true });
    const { cells } = await runSensitivity({
        base: {}, xKey: 'x', yKey: 'y',
        xVals: [1, 2], yVals: [0],
        compute, pick: (r) => (r.valid ? r.v : null),
    });
    assert.deepEqual(cells, [[1, null]]);
});

test('runSensitivity treats a thrown compute as a null cell', async () => {
    const compute = async (b) => { if (b.x === 9) throw new Error('boom'); return { v: b.x, valid: true }; };
    const { cells } = await runSensitivity({
        base: {}, xKey: 'x', yKey: 'y',
        xVals: [9, 5], yVals: [0],
        compute, pick: (r) => r.v,
    });
    assert.deepEqual(cells, [[null, 5]]);
});

// ─── renderSensitivityTable ──────────────────────────────────────────────

test('renderSensitivityTable emits a table with axis headers and formatted cells', () => {
    const html = renderSensitivityTable({
        xVals: [1, 2], yVals: [10],
        cells: [[1.5, 3.0]],
        fmt: (v) => (v == null ? '—' : v.toFixed(2)),
        xfmt: (v) => `x${v}`, yfmt: (v) => `y${v}`,
        xName: 'XX', yName: 'YY',
    });
    assert.match(html, /<table class="ce-sens-table">/);
    assert.match(html, /YY \\ XX/);
    assert.match(html, />x1</);
    assert.match(html, />y10</);
    assert.match(html, />1\.50</);
    assert.match(html, />3\.00</);
});

test('renderSensitivityTable shades populated cells and dashes nulls', () => {
    const html = renderSensitivityTable({
        xVals: [1], yVals: [1, 2],
        cells: [[5], [null]],
        fmt: (v) => (v == null ? '—' : String(v)),
    });
    assert.match(html, /background:rgba/);   // shaded numeric cell
    assert.match(html, />—</);               // null rendered as em dash
});

// ─── SVG charts ──────────────────────────────────────────────────────────

test('svgLineChart returns empty string for fewer than two points', () => {
    assert.equal(svgLineChart([]), '');
    assert.equal(svgLineChart([{ x: 1, y: 2 }]), '');
});

test('svgLineChart draws a polyline for valid points', () => {
    const svg = svgLineChart([{ x: 0, y: 0 }, { x: 1, y: 10 }, { x: 2, y: 5 }], { xlabel: 'spend', ylabel: 'ratio' });
    assert.match(svg, /<svg /);
    assert.match(svg, /<polyline /);
    assert.match(svg, /spend →/);
});

test('svgLineChart filters non-finite points before plotting', () => {
    // Two finite points remain after dropping the NaN → still renders.
    const svg = svgLineChart([{ x: 0, y: 0 }, { x: 1, y: NaN }, { x: 2, y: 4 }]);
    assert.match(svg, /<polyline /);
});

test('svgBarChart returns empty for no data and rects for bars', () => {
    assert.equal(svgBarChart([]), '');
    const svg = svgBarChart([{ label: 'up', value: 5 }, { label: 'down', value: -3 }]);
    assert.match(svg, /<rect /);
});
