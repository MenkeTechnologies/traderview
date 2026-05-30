// Regression-prevention test: pin that NO i18n catalog (en or any locale)
// contains duplicate keys. JSON allows them (last-wins) but every dup
// silently wastes bytes AND can silently overwrite a correct value with
// a stale/wrong one (we caught 7 such cases in this codebase earlier
// where `{msg}` placeholders overrode `{err}` placeholders — the rendered
// tooltip then showed `Configure failed: {msg}` literally because the
// callsite passed `{err: e.message}`).
//
// JSON.parse can't surface duplicates, so we re-scan the raw text for
// duplicate top-level keys and fail loudly when any locale file regresses.

import { test, expect } from 'vitest';
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

const I18N_DIR = join(__dirname, '../i18n');

function findDuplicates(jsonText) {
    const dups = new Map();  // key → count
    const re = /^\s*"([a-z][a-z0-9_.]+)"\s*:/gm;
    let m;
    while ((m = re.exec(jsonText)) !== null) {
        const key = m[1];
        dups.set(key, (dups.get(key) || 0) + 1);
    }
    return [...dups.entries()].filter(([_, n]) => n > 1);
}

test('en catalog has zero duplicate keys', () => {
    const text = readFileSync(join(I18N_DIR, 'app_i18n_en.json'), 'utf8');
    const dups = findDuplicates(text);
    if (dups.length > 0) {
        const lines = dups.map(([k, n]) => `  ${k} appears ${n}×`).join('\n');
        throw new Error(`en catalog has ${dups.length} duplicate keys:\n${lines}`);
    }
    expect(dups).toEqual([]);
});

test('every non-en locale catalog has zero duplicate keys', () => {
    const files = readdirSync(I18N_DIR).filter(f => f.startsWith('app_i18n_') && f.endsWith('.json') && f !== 'app_i18n_en.json');
    const all = [];
    for (const f of files) {
        const text = readFileSync(join(I18N_DIR, f), 'utf8');
        const dups = findDuplicates(text);
        for (const [k, n] of dups) all.push({ file: f, key: k, count: n });
    }
    if (all.length > 0) {
        const lines = all.map(d => `  ${d.file}: ${d.key} appears ${d.count}×`).join('\n');
        throw new Error(`Locale catalogs have ${all.length} duplicate keys:\n${lines}`);
    }
    expect(all).toEqual([]);
});
