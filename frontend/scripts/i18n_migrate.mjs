#!/usr/bin/env node
// Bulk i18n migrator — adds `data-i18n="..."` annotations to safe HTML
// elements in frontend/js/views/*.js + frontend/index.html, and seeds
// the corresponding key→English entries into frontend/i18n/app_i18n_en.json.
//
// Conservative: purely additive. If an element already has `data-i18n*`,
// it's left alone. Elements containing template-literal markers (`${`,
// backticks, < or > in their text) are skipped to avoid trashing
// dynamic content.
//
// Usage:
//   node scripts/i18n_migrate.mjs --dry-run    # preview without writing
//   node scripts/i18n_migrate.mjs              # apply
//
// Re-runnable: skips elements already annotated; reseeds en.json keys
// only if value actually changed.

import { readFile, writeFile, readdir } from 'node:fs/promises';
import { resolve, basename, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, '..');
const VIEWS_DIR = resolve(ROOT, 'js/views');
const INDEX_PATH = resolve(ROOT, 'index.html');
const EN_PATH = resolve(ROOT, 'i18n/app_i18n_en.json');

const args = new Set(process.argv.slice(2));
const DRY = args.has('--dry-run');

// Patterns that match safe HTML elements with plain-text content.
// Each pattern: { re, tag, scopeFn(matchedTag), keyScope }.
// The capture group structure must be: [1]=opening-tag-attrs, [2]=text-content.
// Pattern note: we restrict text to /^[^<>{`]{2,200}$/ to skip dynamic content.
const PATTERNS = [
    { re: /(<h1\b([^>]*?)>)([^<>{`]{2,200})(<\/h1>)/g, tagOpenIdx: 1, textIdx: 3, tagCloseIdx: 4, attrsIdx: 2, scope: 'h1' },
    { re: /(<h2\b([^>]*?)>)([^<>{`]{2,200})(<\/h2>)/g, tagOpenIdx: 1, textIdx: 3, tagCloseIdx: 4, attrsIdx: 2, scope: 'h2' },
    { re: /(<h3\b([^>]*?)>)([^<>{`]{2,200})(<\/h3>)/g, tagOpenIdx: 1, textIdx: 3, tagCloseIdx: 4, attrsIdx: 2, scope: 'h3' },
    { re: /(<button\b([^>]*?)>)([^<>{`]{2,200})(<\/button>)/g, tagOpenIdx: 1, textIdx: 3, tagCloseIdx: 4, attrsIdx: 2, scope: 'btn' },
    { re: /(<p\b([^>]*?)>)([^<>{`]{2,400})(<\/p>)/g, tagOpenIdx: 1, textIdx: 3, tagCloseIdx: 4, attrsIdx: 2, scope: 'hint' },
    { re: /(<option\b([^>]*?)>)([^<>{`]{2,80})(<\/option>)/g, tagOpenIdx: 1, textIdx: 3, tagCloseIdx: 4, attrsIdx: 2, scope: 'opt' },
    { re: /(<th\b([^>]*?)>)([^<>{`]{2,80})(<\/th>)/g, tagOpenIdx: 1, textIdx: 3, tagCloseIdx: 4, attrsIdx: 2, scope: 'th' },
];

async function main() {
    let en = {};
    try { en = JSON.parse(await readFile(EN_PATH, 'utf8')); }
    catch { en = {}; }

    let totalAnnotated = 0;
    let totalKeys = Object.keys(en).length;
    let filesTouched = 0;

    const files = [];
    files.push({ path: INDEX_PATH, viewName: 'index' });
    for (const f of (await readdir(VIEWS_DIR))) {
        if (f.endsWith('.js')) files.push({ path: resolve(VIEWS_DIR, f), viewName: basename(f, '.js') });
    }

    for (const { path, viewName } of files) {
        const src = await readFile(path, 'utf8');
        const { out, annotations } = migrate(src, viewName);
        if (annotations.length === 0) continue;
        filesTouched++;
        totalAnnotated += annotations.length;
        for (const ann of annotations) {
            if (!(ann.key in en)) en[ann.key] = ann.text;
        }
        if (!DRY) await writeFile(path, out, 'utf8');
    }

    const newKeyCount = Object.keys(en).length;
    if (!DRY) await writeFile(EN_PATH, JSON.stringify(en, null, 2) + '\n', 'utf8');

    console.log(`${DRY ? 'DRY-RUN ' : ''}files touched: ${filesTouched}`);
    console.log(`${DRY ? 'DRY-RUN ' : ''}annotations:   ${totalAnnotated}`);
    console.log(`${DRY ? 'DRY-RUN ' : ''}en.json keys:  ${totalKeys} → ${newKeyCount} (+${newKeyCount - totalKeys})`);
}

function migrate(src, viewName) {
    let out = src;
    const annotations = [];
    const seenKeysInFile = new Set();
    for (const pat of PATTERNS) {
        // Reset lastIndex (regex with /g is stateful).
        pat.re.lastIndex = 0;
        out = out.replace(pat.re, (match, tagOpen, attrs, text, tagClose) => {
            // Skip if already annotated.
            if (/data-i18n\b/.test(attrs)) return match;
            const cleanText = text.trim();
            if (!isUserFacing(cleanText)) return match;
            const baseKey = `view.${viewName}.${pat.scope}.${slug(cleanText)}`;
            let key = baseKey;
            let n = 2;
            while (seenKeysInFile.has(key)) key = `${baseKey}_${n++}`;
            seenKeysInFile.add(key);
            annotations.push({ key, text: cleanText });
            // Inject data-i18n right after the opening tag name, before
            // any existing attributes. Preserve original text as fallback.
            const newOpen = tagOpen.replace(/^<(\w+)/, `<$1 data-i18n="${key}"`);
            return `${newOpen}${text}${tagClose}`;
        });
    }
    return { out, annotations };
}

function isUserFacing(s) {
    if (!s) return false;
    if (!/[A-Za-z]/.test(s)) return false;            // need at least one letter
    if (/^[<>{}$\\]/.test(s)) return false;            // template/JSX leftovers
    if (/\${/.test(s)) return false;                   // template-literal expression
    if (/\$\{/.test(s)) return false;
    if (/^(true|false|null|undefined)$/i.test(s)) return false;
    return true;
}

function slug(s) {
    return s.toLowerCase()
        .replace(/&amp;/g, 'and')
        .replace(/&[a-z]+;/g, '_')        // strip HTML entities
        .replace(/[^a-z0-9]+/g, '_')
        .replace(/^_|_$/g, '')
        .slice(0, 50) || 'x';
}

main().catch(err => { console.error(err); process.exit(1); });
