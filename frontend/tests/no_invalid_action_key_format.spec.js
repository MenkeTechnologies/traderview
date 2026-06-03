// Regression-prevention test: every actionKey in DEFAULT_SHORTCUTS + ctxmenu
// items must follow the `tv:<lowercase-kebab>` naming convention.
// Inconsistent naming (camelCase, snake_case, missing prefix) is invisible
// to TypeScript-less JS but causes addEventListener mismatches at dispatch.

import { test, expect } from 'vitest';
import { DEFAULT_SHORTCUTS } from '../js/_shortcuts.js';
import { GLOBAL_ITEMS, EDITING_ITEMS } from '../js/_context_menu.js';

const VALID = /^tv:[a-z][a-z0-9]*(-[a-z0-9]+)*$/;

test('every DEFAULT_SHORTCUTS actionKey matches tv:<lowercase-kebab>', () => {
    const bad = [];
    for (const sc of DEFAULT_SHORTCUTS) {
        if (!sc.actionKey) continue;
        if (!VALID.test(sc.actionKey)) bad.push({ id: sc.id, actionKey: sc.actionKey });
    }
    if (bad.length > 0) {
        const lines = bad.map(b => `  ${b.id} → ${b.actionKey}`).join('\n');
        throw new Error(`${bad.length} shortcut actionKey(s) violate tv:<kebab> format:\n${lines}`);
    }
    expect(bad).toEqual([]);
});

test('every ctxmenu actionKey matches tv:<lowercase-kebab>', () => {
    const bad = [];
    for (const it of [...GLOBAL_ITEMS, ...EDITING_ITEMS]) {
        if (it.kind === 'separator' || !it.actionKey) continue;
        if (!VALID.test(it.actionKey)) bad.push({ id: it.id, actionKey: it.actionKey });
    }
    if (bad.length > 0) {
        const lines = bad.map(b => `  ${b.id} → ${b.actionKey}`).join('\n');
        throw new Error(`${bad.length} ctxmenu actionKey(s) violate tv:<kebab> format:\n${lines}`);
    }
    expect(bad).toEqual([]);
});

test('every i18n key matches namespace.section.slug format', () => {
    // Catalog keys should be dot-separated lowercase tokens. Catches typos
    // like `view.foo Bar.baz` or `View.foo.baz`.
    const fs = require('node:fs');
    const path = require('node:path');
    const cat = JSON.parse(fs.readFileSync(path.join(__dirname, '../i18n/app_i18n_en.json'), 'utf8'));
    // Tokens are letters + digits + underscore + hyphen. Both `by-symbol`
    // and `by_symbol` are accepted (current corpus uses both for routing
    // path segments vs internal scope respectively). Uppercase letters are
    // allowed so the catalog can carry IRC section codes verbatim
    // (s951A, s179d, etc.) and camelCase data references that already
    // ship in the corpus — typos still get caught by the no_missing_*
    // tests that resolve each callsite key against this catalog.
    const KEY_RE = /^[a-z][a-zA-Z0-9_-]*(\.[a-zA-Z0-9_-]+)+$/;
    const bad = Object.keys(cat).filter(k => !KEY_RE.test(k));
    if (bad.length > 0) {
        const sample = bad.slice(0, 20).join('\n  ');
        const tail = bad.length > 20 ? `\n  … and ${bad.length - 20} more` : '';
        throw new Error(`${bad.length} catalog key(s) violate namespace.section.slug format:\n  ${sample}${tail}`);
    }
    expect(bad).toEqual([]);
});
