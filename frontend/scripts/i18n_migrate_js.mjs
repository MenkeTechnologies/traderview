#!/usr/bin/env node
// Second-pass i18n migrator — wraps JS-source string literals in t()
// calls for the patterns the HTML-only migrator can't reach:
//
//   card('Label', value, cls)     → card(t('view.<name>.card.<slug>'), value, cls)
//   showErr('static msg')         → showErr(t('view.<name>.err.<slug>'))
//
// Strict on what counts as a static label: must be a single-quoted or
// double-quoted literal with no template-expr / backslash / paren
// inside. Anything with ${} or backticks or runtime-flavored content
// is skipped.
//
// Adds `import { t } from '../i18n.js'` (or `./i18n.js` for top-level
// modules) if the file makes any t() substitution and doesn't already
// import it.
//
// Re-runnable; the regex won't re-match already-wrapped t() calls.

import { readFile, writeFile, readdir } from 'node:fs/promises';
import { resolve, basename, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, '..');
const VIEWS_DIR = resolve(ROOT, 'js/views');
const EN_PATH = resolve(ROOT, 'i18n/app_i18n_en.json');

const args = new Set(process.argv.slice(2));
const DRY = args.has('--dry-run');

const PATTERNS = [
    // card('Label', ...)
    { re: /card\(\s*(['"])([^'"\\$`]{2,80})\1/g, scope: 'card', quoteIdx: 1, textIdx: 2 },
    // showErr('msg')
    { re: /showErr\(\s*(['"])([^'"\\$`]{5,150})\1\s*\)/g, scope: 'err', quoteIdx: 1, textIdx: 2 },
];

async function main() {
    let en = {};
    try { en = JSON.parse(await readFile(EN_PATH, 'utf8')); }
    catch { en = {}; }

    let touched = 0, subs = 0;
    const startKeys = Object.keys(en).length;

    const files = (await readdir(VIEWS_DIR))
        .filter(f => f.endsWith('.js'))
        .map(f => resolve(VIEWS_DIR, f));

    for (const path of files) {
        const viewName = basename(path, '.js');
        const src = await readFile(path, 'utf8');
        const { out, used, count } = migrate(src, viewName, en);
        if (count === 0) continue;
        touched++; subs += count;
        // Ensure `t` is imported.
        const final = used ? ensureImport(out) : out;
        if (!DRY) await writeFile(path, final, 'utf8');
    }
    if (!DRY) await writeFile(EN_PATH, JSON.stringify(en, null, 2) + '\n', 'utf8');
    const endKeys = Object.keys(en).length;
    console.log(`${DRY ? 'DRY ' : ''}files touched: ${touched}`);
    console.log(`${DRY ? 'DRY ' : ''}substitutions:  ${subs}`);
    console.log(`${DRY ? 'DRY ' : ''}en.json keys:   ${startKeys} → ${endKeys} (+${endKeys - startKeys})`);
}

function migrate(src, viewName, en) {
    let out = src;
    let used = false;
    let count = 0;
    const seen = new Set();
    for (const pat of PATTERNS) {
        pat.re.lastIndex = 0;
        out = out.replace(pat.re, (match, _q, text) => {
            const cleanText = text.trim();
            if (!isUserFacing(cleanText)) return match;
            const baseKey = `view.${viewName}.${pat.scope}.${slug(cleanText)}`;
            let key = baseKey;
            let n = 2;
            while (seen.has(key)) key = `${baseKey}_${n++}`;
            seen.add(key);
            if (!(key in en)) en[key] = cleanText;
            count++; used = true;
            return match.replace(`${_q}${text}${_q}`, `t('${key}')`);
        });
    }
    return { out, used, count };
}

function ensureImport(src) {
    if (/from\s+['"]\.\.\/i18n\.js['"]/.test(src)) return src;
    if (/from\s+['"]\.\/i18n\.js['"]/.test(src)) return src;
    // Insert after the last existing import (or at top if none).
    const lastImport = src.match(/^(?:import[^;]+;\s*\n)+/m);
    if (lastImport) {
        const insertAt = lastImport.index + lastImport[0].length;
        return src.slice(0, insertAt) + `import { t } from '../i18n.js';\n` + src.slice(insertAt);
    }
    return `import { t } from '../i18n.js';\n` + src;
}

function isUserFacing(s) {
    if (!s) return false;
    if (!/[A-Za-z]/.test(s)) return false;
    if (s.startsWith('$') || s.startsWith('<')) return false;
    return true;
}

function slug(s) {
    return s.toLowerCase()
        .replace(/&amp;/g, 'and')
        .replace(/&[a-z]+;/g, '_')
        .replace(/[^a-z0-9]+/g, '_')
        .replace(/^_|_$/g, '')
        .slice(0, 50) || 'x';
}

main().catch(err => { console.error(err); process.exit(1); });
