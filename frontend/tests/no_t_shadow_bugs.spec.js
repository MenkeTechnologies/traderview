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

// Detect both `const t = ...` shadows AND function params named `t`.
// For each shadow declaration, scan forward until brace depth drops
// below the depth at the shadow — that's when the shadow scope closes.
function findBugs(path) {
    const src = readFileSync(path, 'utf8');
    if (!/^import\s*\{[^}]*\bt\b[^}]*\}\s*from\s*['"](?:\.\.?\/)*i18n\.js['"]/m.test(src)) return [];
    if (/\bt as tr\b/.test(src)) return [];

    const lines = src.split('\n');
    const bugs = [];

    // Pre-compute cumulative brace depth at the start of each line.
    const startDepth = new Array(lines.length + 1).fill(0);
    for (let i = 0; i < lines.length; i++) {
        const opens = (lines[i].match(/{/g) || []).length;
        const closes = (lines[i].match(/}/g) || []).length;
        startDepth[i + 1] = startDepth[i] + opens - closes;
    }

    function scanForward(shadowLineIdx, kind) {
        // The shadow's scope = the innermost open block. Find the depth
        // at the END of the shadow line (after its own braces). The
        // shadow remains in scope until depth drops to (this END depth - 1).
        const shadowEndDepth = startDepth[shadowLineIdx + 1];
        // The shadow's enclosing scope is one less than shadowEndDepth
        // when the shadow itself opens a new block (function decl, arrow).
        // For a plain const/let we use shadowEndDepth as the "stay >=" bar.
        const minDepth = kind === 'param' ? shadowEndDepth : shadowEndDepth;
        for (let k = shadowLineIdx + 1; k < Math.min(lines.length, shadowLineIdx + 120); k++) {
            // Depth at the END of line k.
            const endDepth = startDepth[k + 1];
            if (endDepth < minDepth) break;
            const l = lines[k];
            // t('...') NOT preceded by an identifier character.
            const isCall = /[^a-zA-Z_$0-9.]t\(['"]/.test(l) || /^t\(['"]/.test(l.trimStart());
            if (isCall) {
                bugs.push({
                    file: path,
                    shadowLine: shadowLineIdx + 1,
                    callLine: k + 1,
                    snippet: l.trim().slice(0, 100),
                });
                break;  // one bug per shadow
            }
        }
    }

    // 1. `const|let t = ...` shadows.
    for (let i = 0; i < lines.length; i++) {
        if (/(?:const|let)\s+t\s*=/.test(lines[i])) scanForward(i, 'const');
    }
    // 2. Function PARAMS named `t`. Match `function name(... t ...) {`
    //    and `(... t ...) => {` — ONLY block-bodied (skip expression
    //    arrows whose scope is one expression and can't shadow line-far
    //    i18n calls).
    for (let i = 0; i < lines.length; i++) {
        const l = lines[i];
        const fnDecl = l.match(/function\s+\w+\s*\(([^)]*)\)\s*\{/);
        const arrow = l.match(/(?:^|[,=({])\s*\(([^)]*)\)\s*=>\s*\{/);
        const params = fnDecl ? fnDecl[1] : (arrow ? arrow[1] : null);
        if (!params) continue;
        const names = params.split(',').map(p => p.split('=')[0].trim().split(':')[0].trim());
        if (!names.includes('t')) continue;
        scanForward(i, 'param');
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
