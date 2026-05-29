#!/usr/bin/env node
// i18n extractor — scans frontend/js/views/*.js for strings that look
// like user-facing UI text (innerHTML templates, h1/h2 contents, card
// labels, error messages) and prints them as `key: "value"` suggestions
// for app_i18n_en.json. Output is grouped by view file.
//
// Usage:
//   node scripts/i18n_extract.mjs           # list candidates
//   node scripts/i18n_extract.mjs --json    # JSON output for tooling
//   node scripts/i18n_extract.mjs --diff    # only print keys not in app_i18n_en.json

import { readFile, readdir } from 'node:fs/promises';
import { resolve, basename, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, '..');
const VIEWS_DIR = resolve(ROOT, 'js/views');
const EN_PATH = resolve(ROOT, 'i18n/app_i18n_en.json');

// Heuristic patterns that catch most user-facing strings without
// flagging logical comparisons / dataset keys. Kept conservative —
// false-negatives are fine, false-positives waste the user's time.
const PATTERNS = [
    // <h1 class="view-title">// TITLE</h1>
    { re: /<h[12][^>]*>([^<]{3,80})<\/h[12]>/g, scope: 'heading' },
    // <button …>Label</button> (only when label has letters)
    { re: /<button[^>]*>([A-Z][^<>{]{2,60})<\/button>/g, scope: 'button' },
    // Card labels like card('Active traders', ...)
    { re: /card\(\s*['"]([^'"]{3,60})['"]/g, scope: 'card.label' },
    // showErr('static msg')
    { re: /showErr\(\s*['"]([^'"]{5,120})['"]/g, scope: 'error' },
    // Top-level <p class="muted">…</p> hint text
    { re: /<p\s+class="muted"[^>]*>([^<]{10,200})<\/p>/g, scope: 'hint' },
];

const args = new Set(process.argv.slice(2));
const wantJson = args.has('--json');
const wantDiff = args.has('--diff');

async function main() {
    const files = (await readdir(VIEWS_DIR))
        .filter(f => f.endsWith('.js'))
        .map(f => resolve(VIEWS_DIR, f));
    let existing = {};
    try { existing = JSON.parse(await readFile(EN_PATH, 'utf8')); }
    catch { /* fine; first-time bootstrap */ }
    const existingValues = new Set(Object.values(existing));
    const out = {};
    for (const path of files) {
        const text = await readFile(path, 'utf8');
        const view = basename(path, '.js');
        const found = new Set();
        for (const { re, scope } of PATTERNS) {
            re.lastIndex = 0;
            let m;
            while ((m = re.exec(text)) !== null) {
                const value = m[1].trim();
                if (!value || /^[<>{}\\$:]/.test(value)) continue;
                // Skip strings that look like CSS / class / template-interpolation residue.
                if (/^[a-z_-]+$/.test(value) && !value.includes(' ')) continue;
                if (wantDiff && existingValues.has(value)) continue;
                const key = `view.${view}.${scope}.${slug(value)}`;
                found.add(`${key} = ${value}`);
            }
        }
        if (found.size > 0) out[view] = [...found];
    }
    if (wantJson) {
        console.log(JSON.stringify(out, null, 2));
        return;
    }
    let total = 0;
    for (const [view, lines] of Object.entries(out)) {
        console.log(`\n── ${view} ──`);
        for (const line of lines) { console.log(`  ${line}`); total++; }
    }
    console.log(`\n${total} candidate strings across ${Object.keys(out).length} views.`);
}

function slug(s) {
    return s.toLowerCase()
        .replace(/[^a-z0-9]+/g, '_')
        .replace(/^_|_$/g, '')
        .slice(0, 40);
}

main().catch(err => { console.error(err); process.exit(1); });
