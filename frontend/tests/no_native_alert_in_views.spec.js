// Regression-prevention test: no view file may call native `alert(...)`.
//
// Native `alert()` is blocking, ugly, and locale-disabled by some
// browsers. Every user-facing notice goes through `showToast()` (which
// composes with the i18n catalog and the toast region in #tv-toast-root).
//
// A previous batch swept 63 call sites across 24 views; pinning the
// invariant here prevents regression when adding new views or copying
// patterns from older snippets.

import { test, expect } from 'vitest';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';

const VIEWS_DIR = join(__dirname, '../js/views');

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

test('no view file calls native alert()', () => {
    const offenders = [];
    for (const f of walk(VIEWS_DIR)) {
        const src = readFileSync(f, 'utf8');
        // Match `alert(` or `window.alert(`, but not `tShowAlert`, etc.
        // Use a word boundary on the left so `mAlertConfig` etc. don't trip.
        const matches = [...src.matchAll(/(?<![A-Za-z0-9_$.])(?:window\.)?alert\s*\(/g)];
        if (matches.length > 0) {
            offenders.push({ file: f.split('/views/')[1], count: matches.length });
        }
    }
    if (offenders.length > 0) {
        const lines = offenders.map(o => `  ${o.file}: ${o.count} call(s)`).join('\n');
        throw new Error(
            `${offenders.length} view file(s) call native alert() — use showToast() instead:\n${lines}`,
        );
    }
    expect(offenders).toEqual([]);
});

test('no view file calls native confirm() / prompt()', () => {
    const offenders = [];
    for (const f of walk(VIEWS_DIR)) {
        const src = readFileSync(f, 'utf8');
        // Match bare `confirm(` / `prompt(` / `window.confirm(` / `window.prompt(`,
        // but allow `tConfirm` / `tPrompt` / property accesses like `obj.confirm`.
        // The negative lookbehind blocks letter/digit/underscore/$/. before the keyword.
        const matches = [
            ...src.matchAll(/(?<![A-Za-z0-9_$.])(?:window\.)?confirm\s*\(/g),
            ...src.matchAll(/(?<![A-Za-z0-9_$.])(?:window\.)?prompt\s*\(/g),
        ];
        if (matches.length > 0) {
            offenders.push({ file: f.split('/views/')[1], count: matches.length });
        }
    }
    if (offenders.length > 0) {
        const lines = offenders.map(o => `  ${o.file}: ${o.count} call(s)`).join('\n');
        throw new Error(
            `${offenders.length} view file(s) call native confirm()/prompt() — use tConfirm/tPrompt from dialog.js instead:\n${lines}`,
        );
    }
    expect(offenders).toEqual([]);
});
