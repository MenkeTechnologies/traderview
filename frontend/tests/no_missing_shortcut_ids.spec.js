// Regression-prevention test: every `data-shortcut="id"` annotation in
// view files + index.html must map to a registered shortcut in
// `_shortcuts.js` DEFAULT_SHORTCUTS. Otherwise `tooltip.js`'s
// `augmentShortcutTitles` silently skips the chip (the button shows
// the tip but no keyboard chip), which looks like the shortcut works
// when it doesn't.

import { test, expect } from 'vitest';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { DEFAULT_SHORTCUTS } from '../js/_shortcuts.js';

const FRONTEND_DIR = join(__dirname, '..');

function walk(dir) {
    const out = [];
    for (const name of readdirSync(dir)) {
        const p = join(dir, name);
        const s = statSync(p);
        if (s.isDirectory()) out.push(...walk(p));
        else if (name.endsWith('.js') || name.endsWith('.html')) out.push(p);
    }
    return out;
}

test('every literal data-shortcut="id" attribute maps to a registered shortcut', () => {
    const registered = new Set(DEFAULT_SHORTCUTS.map(s => s.id));
    const re = /data-shortcut\s*=\s*["']([a-z][a-z0-9_]+)["']/g;
    const sources = [join(FRONTEND_DIR, 'index.html'), ...walk(join(FRONTEND_DIR, 'js'))];

    const missing = new Map();  // id → list of files
    for (const file of sources) {
        const text = readFileSync(file, 'utf8');
        for (const line of text.split('\n')) {
            const trimmed = line.trimStart();
            if (trimmed.startsWith('//') || trimmed.startsWith('*')) continue;
            let m;
            while ((m = re.exec(line)) !== null) {
                const id = m[1];
                if (!registered.has(id)) {
                    if (!missing.has(id)) missing.set(id, []);
                    missing.get(id).push(file);
                }
            }
            re.lastIndex = 0;
        }
    }
    if (missing.size > 0) {
        const lines = [...missing.entries()]
            .map(([id, files]) => `  ${id} (${files.length} ref${files.length > 1 ? 's' : ''}): ${files[0]}`)
            .join('\n');
        throw new Error(`Found ${missing.size} data-shortcut id(s) not in DEFAULT_SHORTCUTS:\n${lines}`);
    }
    expect(missing.size).toBe(0);
});
