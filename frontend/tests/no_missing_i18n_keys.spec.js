// Regression-prevention test: every static `data-i18n*` annotation in
// view files + index.html must resolve to a key in `app_i18n_en.json`.
//
// Catches the bug class where a refactor renames a catalog key but
// forgets to update one of its DOM references — the missing key falls
// through to literal-key rendering at runtime ("view.foo.bar" appears
// in the UI instead of "Foo bar"). Pre-runtime catches save a real
// support-ticket spiral.
//
// Limits: only matches LITERAL keys (the static `data-i18n="view.x.y"`
// form). Template-literal keys like `data-i18n="${labelKey}"` are
// caller-resolved at render time and skipped here — covered indirectly
// by the dynamic dispatch tests in command_palette / launcher / etc.

import { test, expect } from 'vitest';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';

const FRONTEND_DIR = join(__dirname, '..');
const I18N_PATH = join(FRONTEND_DIR, 'i18n/app_i18n_en.json');

function walk(dir) {
    const out = [];
    for (const name of readdirSync(dir)) {
        const p = join(dir, name);
        const s = statSync(p);
        if (s.isDirectory()) out.push(...walk(p));
        else if (name.endsWith('.js') || name.endsWith('.html')) out.push(p);
    }
    return out;
}

function collectKeys(text) {
    // Match data-i18n / data-i18n-html / data-i18n-title / data-i18n-placeholder
    //       / data-i18n-aria-label
    // when the value is a LITERAL string starting with a lowercase letter
    // (catalog keys never start with uppercase or include ${...}).
    // Skip lines that start with `//` (JS line comment) or `*` (JSDoc) to
    // avoid false positives from docstrings that mention `data-i18n="key"`
    // as documentation rather than as real markup.
    const re = /data-i18n(?:-html|-title|-placeholder|-aria-label)?\s*=\s*["']([a-z][a-z0-9_.]+)["']/g;
    const keys = new Set();
    for (const line of text.split('\n')) {
        const trimmed = line.trimStart();
        if (trimmed.startsWith('//') || trimmed.startsWith('*')) continue;
        let m;
        while ((m = re.exec(line)) !== null) keys.add(m[1]);
        re.lastIndex = 0;
    }
    return keys;
}

test('every literal data-i18n* key under frontend/ resolves to app_i18n_en.json', () => {
    const catalog = JSON.parse(readFileSync(I18N_PATH, 'utf8'));
    const catalogKeys = new Set(Object.keys(catalog));

    // Scan ALL .js + .html under frontend/ EXCEPT node_modules + tests +
    // i18n/ + scripts/.
    const sources = [];
    sources.push(join(FRONTEND_DIR, 'index.html'));
    sources.push(...walk(join(FRONTEND_DIR, 'js')));

    const missing = new Map();  // key → list of files that reference it
    for (const file of sources) {
        const text = readFileSync(file, 'utf8');
        const keys = collectKeys(text);
        for (const k of keys) {
            if (!catalogKeys.has(k)) {
                if (!missing.has(k)) missing.set(k, []);
                missing.get(k).push(file);
            }
        }
    }
    if (missing.size > 0) {
        const lines = [...missing.entries()]
            .slice(0, 20)
            .map(([k, files]) => `  ${k} (${files.length} ref${files.length > 1 ? 's' : ''}): ${files[0]}`)
            .join('\n');
        const tail = missing.size > 20 ? `\n  … and ${missing.size - 20} more` : '';
        throw new Error(`Found ${missing.size} data-i18n key(s) not in app_i18n_en.json:\n${lines}${tail}`);
    }
    expect(missing.size).toBe(0);
});
