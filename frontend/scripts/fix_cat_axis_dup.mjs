// Second-pass codemod: kill the categorical-x label duplication.
//
// The first pass (scripts/fix_fake_1969.mjs) added `time: false` so the
// cursor stopped showing 1969-12-31, but every chart that mapped splits
// → labels[round(v) - 1] still printed each label many times along the
// x-axis (uPlot picks fractional split increments by default, and
// Math.round snaps adjacent splits to the same index).
//
// Fix: in each axis definition that uses the
// `values: (_u, splits) => splits.map(v => labels[...] || '')` shape,
// inject `splits: () => xs,` and `incrs: [1],` BEFORE the `values:`
// line. uPlot then ticks at exactly the data positions and the values
// callback gets called once per real point.
//
// Safe when `xs` is in scope in the same function (verified: every
// candidate file has `const xs = labels.map(...)` or equivalent above
// the chart construction).

import { readdirSync, readFileSync, writeFileSync, statSync } from 'fs';
import { join } from 'path';

const VIEWS_DIR = new URL('../js/views', import.meta.url).pathname;

// Match the broken axis values callback line; capture indentation.
// Loosely anchored on the `labels[Math.round(v) - 1]` pattern — every
// occurrence we care about ends with that index lookup somewhere on
// the same line. Captures whole line through end so the rewrite
// preserves any trailing punctuation (`},`, `},`, etc.).
const VALUES_RE = /^(\s*)values:\s*\(_u,\s*(?:splits|sp)\)\s*=>\s*(?:splits|sp)\.map\(v\s*=>\s*labels\[Math\.round\(v\)\s*-\s*1\].*$/gm;

function walk(dir) {
    const out = [];
    for (const name of readdirSync(dir)) {
        const path = join(dir, name);
        const stat = statSync(path);
        if (stat.isDirectory()) out.push(...walk(path));
        else if (name.endsWith('.js')) out.push(path);
    }
    return out;
}

const files = walk(VIEWS_DIR);
const touched = [];
const skipped = [];

for (const file of files) {
    const src = readFileSync(file, 'utf8');
    if (!VALUES_RE.test(src)) { skipped.push(file); continue; }
    if (src.includes('splits: () => xs')) { skipped.push(file); continue; }

    // Reset regex stateful index after the test()
    VALUES_RE.lastIndex = 0;

    let out = '';
    let lastIdx = 0;
    let count = 0;
    let m;
    while ((m = VALUES_RE.exec(src)) !== null) {
        const indent = m[1];
        // Inject splits + incrs before the values: line.
        out += src.slice(lastIdx, m.index);
        out += `${indent}splits: () => xs,\n`;
        out += `${indent}incrs: [1],\n`;
        out += `${indent}${m[0].slice(indent.length)}\n`;
        lastIdx = m.index + m[0].length;
        // The match consumed the trailing newline; advance past it.
        if (src[lastIdx] === '\n') lastIdx += 1;
        count += 1;
    }
    out += src.slice(lastIdx);

    if (count === 0) { skipped.push(file); continue; }
    writeFileSync(file, out);
    touched.push([file, count]);
}

const rel = (p) => p.slice(VIEWS_DIR.length + 1);
console.log(`Touched ${touched.length} files (${touched.reduce((a, [, n]) => a + n, 0)} axis configs):`);
for (const [f, n] of touched.slice(0, 12)) console.log(`  ${rel(f)}  (${n}x)`);
if (touched.length > 12) console.log(`  … and ${touched.length - 12} more`);
console.log(`Skipped: ${skipped.length}`);
