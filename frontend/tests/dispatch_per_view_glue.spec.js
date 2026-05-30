// Regression test: the app.js dispatcher must set per-view glue —
// mount data-context-scope, shortcut scope, and document.title — every
// time a view is rendered. Pure source-scan; jsdom isn't loaded here.
//
// These three lines together give every view a stable identity in the
// global UI infrastructure: right-click context, keyboard-shortcut
// scope, and OS-level browser-tab discoverability.

import { test, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

const APP = readFileSync(join(__dirname, '../js/app.js'), 'utf8');

test('dispatcher sets mount data-context-scope to current view', () => {
    expect(APP).toMatch(/mount\.setAttribute\(\s*['"]data-context-scope['"]\s*,\s*view\s*\)/);
});

test('dispatcher sets shortcut scope to current view', () => {
    expect(APP).toMatch(/setScope\(\s*view\s*\)/);
});

test('dispatcher sets document.title from view label', () => {
    expect(APP).toMatch(/document\.title\s*=\s*`TraderView/);
    // Must derive the label via t() on `tile.${view}.label`.
    expect(APP).toMatch(/tile\.\$\{view\}\.label/);
});

test('setScope is imported from shortcuts.js', () => {
    expect(APP).toMatch(/import\s+\{[^}]*\bsetScope\b[^}]*\}\s+from\s+['"]\.\/shortcuts(?:\.js)?['"]/);
});
