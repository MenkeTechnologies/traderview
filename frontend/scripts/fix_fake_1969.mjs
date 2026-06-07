// One-shot codemod: every uPlot chart that uses an index-based x array
// (`labels.map((_, i) => i)` and friends) but doesn't opt out of
// uPlot's default time scale renders a `1969-12-31 7:00pm` cursor
// tooltip and (when the chart has discrete labels) prints the same
// label many times across the axis.
//
// This script walks every `js/views/*.js`. For each file that has at
// least one index-based xs assignment, it rewrites every
// `scales: { x: {` (no `time:` set) to `scales: { x: { time: false,`.
// Files that mix index-based and timestamp-based x arrays are left
// untouched — they need a per-chart review (the report list at the
// end tells you which).

import { readdirSync, readFileSync, writeFileSync, statSync } from 'fs';
import { join } from 'path';

const VIEWS_DIR = new URL('../js/views', import.meta.url).pathname;
const INDEX_X = /\.map\(\(_, i\) => i\b/;
// Time-based heuristic: anything that builds x from timestamps. Files
// that match this AND match INDEX_X are flagged "mixed" — left alone.
const TIME_X = /getTime\(\)|Date\.parse|\.timestamp\b|\.ts\b/;
// Match `scales: { x: { ... }`. Capture only the `{` after `x:` so we
// can inject `time: false,` right after it.
const SCALES_X = /scales\s*:\s*\{\s*x\s*:\s*\{/g;

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
const skippedMixed = [];
const skippedNoIndex = [];
const skippedNoScale = [];

for (const file of files) {
    const src = readFileSync(file, 'utf8');
    if (!INDEX_X.test(src)) { skippedNoIndex.push(file); continue; }
    if (TIME_X.test(src)) { skippedMixed.push(file); continue; }
    if (!SCALES_X.test(src)) { skippedNoScale.push(file); continue; }

    let out = '';
    let lastIdx = 0;
    SCALES_X.lastIndex = 0;
    let m;
    let count = 0;
    while ((m = SCALES_X.exec(src)) !== null) {
        const after = src.slice(m.index + m[0].length, m.index + m[0].length + 64);
        // Skip if the chart already opts in/out of time explicitly.
        if (/^\s*time\s*:/.test(after)) continue;
        out += src.slice(lastIdx, m.index + m[0].length);
        // Inject `time: false,` right after the open brace.
        out += ' time: false,';
        lastIdx = m.index + m[0].length;
        count += 1;
    }
    out += src.slice(lastIdx);
    if (count === 0) { skippedNoScale.push(file); continue; }
    writeFileSync(file, out);
    touched.push([file, count]);
}

const rel = (p) => p.slice(VIEWS_DIR.length + 1);
console.log(`Touched ${touched.length} files (${touched.reduce((a, [, n]) => a + n, 0)} chart constructions):`);
for (const [f, n] of touched.slice(0, 12)) console.log(`  ${rel(f)}  (${n}x)`);
if (touched.length > 12) console.log(`  … and ${touched.length - 12} more`);
console.log(`Skipped (mixed time + index x): ${skippedMixed.length}`);
for (const f of skippedMixed.slice(0, 8)) console.log(`  ${rel(f)}`);
console.log(`Skipped (no index xs): ${skippedNoIndex.length}`);
console.log(`Skipped (no scales.x.{ open form): ${skippedNoScale.length}`);
