// Regression-prevention test: no JS file under frontend/js/ should
// import a named symbol it doesn't use. Catches refactor leftovers
// where the using-call-site got deleted but the import was kept,
// silently bloating the module graph.
//
// Heuristic: for each `import { a, b, c } from '...'`, count text
// occurrences of each name. If it's <= 1 (only the import line),
// the symbol is unused. False positives are possible when a name
// appears in a comment that mentions it; the scanner skips lines
// starting with `//` or `*` to mitigate.

import { test, expect } from 'vitest';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';

const JS_DIR = join(__dirname, '../js');
const TESTS_DIR = join(__dirname);

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

function stripComments(src) {
    // Strip ONLY block comments. Stripping line comments naively (everything
    // after `//`) would chop content inside template-literal strings like
    // `<h1>// SHARED · ${esc(x)}</h1>` — falsely removing the `esc(x)` usage.
    // Block comments are unambiguous and safe.
    return src.replace(/\/\*[\s\S]*?\*\//g, '');
}

function findUnused(src) {
    // Strip block comments for usage counting; line comments left intact
    // because chopping at `//` would also chop content inside template
    // literals like `<h1>// SHARED · ${esc(x)}</h1>`. Mitigation: when
    // scanning for `import { ... }` matches, skip ones that appear inside
    // a `//` line comment (jsdoc usage examples).
    const stripped = stripComments(src);
    const re = /import\s*\{([^}]+)\}\s*from\s*['"][^'"]+['"]/g;
    const lines = src.split('\n');
    // Pre-compute a per-line "starts-with-line-comment-marker?" flag.
    const lineCommentAt = new Set();
    let offset = 0;
    for (let i = 0; i < lines.length; i++) {
        const trimmed = lines[i].trimStart();
        if (trimmed.startsWith('//') || trimmed.startsWith('*')) {
            // mark the range of byte offsets this line covers
            for (let p = offset; p < offset + lines[i].length; p++) lineCommentAt.add(p);
        }
        offset += lines[i].length + 1;  // +1 for \n
    }
    const unused = [];
    let m;
    while ((m = re.exec(src)) !== null) {
        // Skip if the match starts inside a line comment.
        if (lineCommentAt.has(m.index)) continue;
        const names = m[1].split(',').map(n => n.trim()).filter(Boolean);
        for (const n of names) {
            const usageName = n.split(/\s+as\s+/).pop().trim();
            if (!/^[a-zA-Z_$][\w$]*$/.test(usageName)) continue;
            const wre = new RegExp(`\\b${usageName.replace(/[$]/g, '\\$&')}\\b`, 'g');
            const count = (stripped.match(wre) || []).length;
            if (count <= 1) unused.push(usageName);
        }
    }
    return unused;
}

test('no JS file under frontend/js/ or frontend/tests/ has unused named imports', () => {
    const files = [...walk(JS_DIR), ...walk(TESTS_DIR)];
    const offenders = [];
    for (const f of files) {
        const src = readFileSync(f, 'utf8');
        const unused = findUnused(src);
        for (const u of unused) offenders.push({ file: f, name: u });
    }
    if (offenders.length > 0) {
        const lines = offenders
            .slice(0, 25)
            .map(o => `  ${o.file}: ${o.name}`)
            .join('\n');
        const tail = offenders.length > 25 ? `\n  … and ${offenders.length - 25} more` : '';
        throw new Error(`Found ${offenders.length} unused named import(s):\n${lines}${tail}`);
    }
    expect(offenders).toEqual([]);
});
