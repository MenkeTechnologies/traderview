// Regression-prevention test: every literal-string key passed to t() or tr()
// in JS source must resolve to a key in app_i18n_en.json.
//
// Sibling tests:
//   no_missing_i18n_keys.spec.js          — DOM `data-i18n*` + `data-tip` keys
//   no_missing_shortcut_ids.spec.js       — DOM `data-shortcut` IDs
//   no_missing_shortcut_desc_keys.spec.js — shortcut/ctxmenu *Key fields
//
// Dynamic template-literal keys like t(`view.X.${name}`) are skipped — those
// can't be statically resolved without executing the code.

import { test, expect } from 'vitest';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';

const JS_DIR = join(__dirname, '../js');
const CATALOG = JSON.parse(readFileSync(join(__dirname, '../i18n/app_i18n_en.json'), 'utf8'));
const CATALOG_KEYS = new Set(Object.keys(CATALOG));

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

function findTCalls(src) {
    // Match t('key'), tr('key'), translateOrFallback(translate, 'key', ...),
    // lookup(translate, 'key', ...) — only static string literals.
    // Ternaries (t(cond ? 'a' : 'b')) are intentionally NOT matched; false
    // negatives are acceptable, false positives are not.
    const out = [];
    const tRe = /\b(?:t|tr)\(\s*(['"])([a-z][a-z0-9_-]*(?:\.[a-z0-9_-]+)+)\1/gi;
    let m;
    while ((m = tRe.exec(src)) !== null) out.push(m[2]);
    // Wrappers: translateOrFallback / lookup — second argument is the key.
    const wrapRe = /\b(?:translateOrFallback|lookup)\(\s*\w+\s*,\s*(['"])([a-z][a-z0-9_-]*(?:\.[a-z0-9_-]+)+)\1/gi;
    while ((m = wrapRe.exec(src)) !== null) out.push(m[2]);
    return out;
}

test('every literal t()/tr() key resolves to app_i18n_en.json', () => {
    const files = walk(JS_DIR);
    const missing = new Map();  // key → [files]
    for (const f of files) {
        const src = readFileSync(f, 'utf8');
        for (const key of findTCalls(src)) {
            if (!CATALOG_KEYS.has(key)) {
                if (!missing.has(key)) missing.set(key, new Set());
                missing.get(key).add(f.replace(/^.*\/frontend\//, ''));
            }
        }
    }
    if (missing.size > 0) {
        const entries = [...missing.entries()].slice(0, 30);
        const lines = entries.map(([k, fs]) => `  ${k}\n    used in: ${[...fs].slice(0, 3).join(', ')}`).join('\n');
        const tail = missing.size > 30 ? `\n  … and ${missing.size - 30} more` : '';
        throw new Error(`Found ${missing.size} t()/tr() key(s) not in catalog:\n${lines}${tail}`);
    }
    expect(missing.size).toBe(0);
});
