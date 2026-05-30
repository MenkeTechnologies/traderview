// Regression-prevention test: scan all helper + view files for the bug
// pattern "module imports bare `t` from i18n, then a function scope
// shadows `t` AND calls `t('...')` after the shadow."
//
// The shadowing makes the i18n call fall through to the shadowed local
// (which is usually a DOM element / number / object), throwing
// `TypeError: t is not a function` at runtime — only when the catch/error
// path triggers, so unit tests rarely catch it.
//
// Heuristic: for each file importing bare `t`, find lines that shadow
// (`const|let t = ...`) inside a function, then look for a `t('...')`
// call within the next 60 lines AT THE SAME OR DEEPER brace depth.
//
// This won't catch every shadow pattern (e.g. param-named `t` inside
// a function that nests another function calling `t(...)`), but it
// catches the most-common foot-gun introduced by automated i18n
// migrations.

import { test, expect } from 'vitest';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';

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

function findBugs(path) {
    const src = readFileSync(path, 'utf8');
    // Must import bare t (not aliased as tr).
    if (!/^import\s*\{[^}]*\bt\b[^}]*\}\s*from\s*['"](?:\.\.?\/)*i18n\.js['"]/m.test(src)) return [];
    if (/\bt as tr\b/.test(src)) return [];

    const lines = src.split('\n');
    const bugs = [];
    // For each shadow line, check the next N lines for a t('...') call.
    // We stop scanning when brace depth at the call goes BELOW the depth
    // at the shadow declaration (= shadow scope closed).
    for (let i = 0; i < lines.length; i++) {
        const line = lines[i];
        if (!/(?:const|let)\s+t\s*=/.test(line)) continue;
        // Compute brace-depth delta from start of file to start of this line.
        let depthAtShadow = 0;
        for (let j = 0; j < i; j++) {
            depthAtShadow += (lines[j].match(/{/g) || []).length;
            depthAtShadow -= (lines[j].match(/}/g) || []).length;
        }
        // Scan forward up to 80 lines for a t('...') call at depth >= depthAtShadow.
        let depth = depthAtShadow + ((line.match(/{/g) || []).length) - ((line.match(/}/g) || []).length);
        for (let k = i + 1; k < Math.min(lines.length, i + 80); k++) {
            const l = lines[k];
            depth += (l.match(/{/g) || []).length;
            depth -= (l.match(/}/g) || []).length;
            if (depth < depthAtShadow) break;  // shadow scope closed
            // Look for t('...') NOT preceded by an identifier character.
            const m = l.match(/[^a-zA-Z_$0-9.]t\(['"]/);
            // Also accept t( at line start.
            const isCall = m || /^t\(['"]/.test(l.trimStart());
            if (isCall) {
                bugs.push({ file: path, shadowLine: i + 1, callLine: k + 1, snippet: l.trim().slice(0, 100) });
                break;  // one bug per shadow is enough
            }
        }
    }
    return bugs;
}

test('no `const t = …` shadow + `t(...)` call bugs in any frontend module', () => {
    const root = join(__dirname, '../js');
    const files = walk(root);
    const allBugs = [];
    for (const f of files) {
        allBugs.push(...findBugs(f));
    }
    if (allBugs.length > 0) {
        const msg = allBugs
            .map(b => `  ${b.file}:${b.callLine} (shadow at L${b.shadowLine}): ${b.snippet}`)
            .join('\n');
        throw new Error(`Found ${allBugs.length} shadow+call bug(s):\n${msg}`);
    }
    expect(allBugs).toEqual([]);
});
