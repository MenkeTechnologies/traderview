// Regression-prevention test: every `showToast(..., { level: '...' })` call
// site must use one of the LEVELS exported by _toast.js. _toast.js validates
// at runtime and returns a validation error, but the result is a silent
// fallback to 'info' — the developer sees nothing.
//
// This spec walks all JS source for `level: '...'` literals inside showToast
// argument objects and verifies the string is in LEVELS.

import { test, expect } from 'vitest';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { LEVELS } from '../js/_toast.js';

const VALID = new Set(LEVELS);

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

test('every showToast level literal is in LEVELS', () => {
    const files = walk(join(__dirname, '../js'));
    const violations = [];
    // Match `showToast(..., { ... level: 'xxx' ... })` — find showToast
    // call, then look ahead for `level: '...'` in the next ~200 chars.
    const callRe = /showToast\([^)]{0,400}\blevel:\s*['"]([a-z]+)['"]/g;
    for (const f of files) {
        const src = readFileSync(f, 'utf8');
        let m;
        while ((m = callRe.exec(src)) !== null) {
            if (!VALID.has(m[1])) {
                violations.push({ file: f.replace(/^.*\/frontend\//, ''), level: m[1] });
            }
        }
    }
    if (violations.length > 0) {
        const lines = violations.map(v => `  ${v.file}: level='${v.level}'`).join('\n');
        throw new Error(`${violations.length} showToast call(s) with invalid level (valid: ${LEVELS.join(', ')}):\n${lines}`);
    }
    expect(violations).toEqual([]);
});
