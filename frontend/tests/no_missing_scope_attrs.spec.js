// Regression-prevention test: every `data-context-scope="symbol-row"`
// element in view files MUST also carry a `data-symbol="..."` attribute
// on the same tag — otherwise the symbol-row handlers fire with a null
// symbol and the user sees a confusing "no active symbol" toast.
//
// The disclosures.js view emits the scope conditionally
// (`d.symbol ? data-context-scope='symbol-row' data-symbol='X' : ''`) so
// either both are present in the literal markup, or neither — never
// scope-without-symbol. This test pins that invariant.

import { test, expect } from 'vitest';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { ALL_SCOPED_ITEMS } from '../js/_context_menu.js';

const VIEWS_DIR = join(__dirname, '../js/views');

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

// Scope → set of `data-X` attrs that every emitting tag MUST also carry,
// because the registered handlers in context_menu.js read them via
// dataFromTarget(). A missing attr → handler short-circuits with a
// confusing toast, so pin it at test-time.
const SCOPE_REQUIRED_ATTRS = {
    'symbol-row':            ['data-symbol'],
    'trade-row':             ['data-id'],
    'watchlist-symbol-row':  ['data-symbol', 'data-wid'],
    'position-row':          ['data-symbol', 'data-id'],
    'alert-rule-row':        ['data-rule-id'],
    'strategy-alert-row':    ['data-id'],
    'webhook-row':           ['data-id'],
    'tag-chip':              ['data-id', 'data-name'],
    'api-token-row':         ['data-id', 'data-prefix'],
    'journal-entry':         ['data-id'],
    'hotkey-row':            ['data-id', 'data-combo'],
    'custom-indicator-row':  ['data-id', 'data-definition'],
    'account-row':           ['data-id'],
    'plan-row':              ['data-id', 'data-symbol'],
    'share-row':             ['data-id', 'data-slug'],
    'backtest-preset-row':   ['data-id', 'data-slug'],
    'board-row':             ['data-id'],
    'dashboard-sidebar-item': ['data-id'],
    // Budget-view category row carries data-code (the expense_categories.code).
    // The handlers click the row's existing button/input by selector keyed
    // on data-code, so this is the only required attr.
    'budget-row':            ['data-code'],
    // Tax wizard root pane — actions target the autopopulate / PDF
    // controls by ID, no per-row data needed.
    'tax-wiz-section':       [],
    // Categorize merchant group — data-i is the group index in STATE.groups.
    'categorize-group':      ['data-i'],
};

test('SCOPE_REQUIRED_ATTRS covers every scope in ALL_SCOPED_ITEMS', () => {
    // If a new scope is added to the registry but its required attrs
    // aren't declared here, this test fires so the developer documents
    // the contract (or explicitly opts out with an empty array).
    const registered = ALL_SCOPED_ITEMS.map(([scope]) => scope);
    const declared = new Set(Object.keys(SCOPE_REQUIRED_ATTRS));
    const undocumented = registered.filter(s => !declared.has(s));
    if (undocumented.length > 0) {
        throw new Error(
            `${undocumented.length} scope(s) in ALL_SCOPED_ITEMS missing from ` +
            `SCOPE_REQUIRED_ATTRS — declare required data-* attrs (or [] if ` +
            `none) for: ${undocumented.join(', ')}`,
        );
    }
    expect(undocumented).toEqual([]);
});

test('every row-scope tag carries the data-* attrs its handler reads', () => {
    const offenders = [];
    for (const f of walk(VIEWS_DIR)) {
        const src = readFileSync(f, 'utf8');
        for (const [scope, required] of Object.entries(SCOPE_REQUIRED_ATTRS)) {
            // Match opening tags declaring this scope; tag may span lines.
            const re = new RegExp(
                `<[a-z][^>]*?\\bdata-context-scope=["']${scope}["'][^>]*?>`,
                'gi',
            );
            for (const m of src.matchAll(re)) {
                const tag = m[0];
                const missing = required.filter(a => !new RegExp(`\\b${a}\\s*=`).test(tag));
                if (missing.length > 0) {
                    offenders.push({
                        file: f.split('/views/')[1],
                        scope,
                        missing,
                        snippet: tag.length > 120 ? tag.slice(0, 120) + '…' : tag,
                    });
                }
            }
        }
    }
    if (offenders.length > 0) {
        const lines = offenders.map(o =>
            `  ${o.file} (scope=${o.scope}) missing ${o.missing.join(', ')}:\n    ${o.snippet}`
        ).join('\n');
        throw new Error(
            `${offenders.length} row-scope tag(s) missing required data-*:\n${lines}`,
        );
    }
    expect(offenders).toEqual([]);
});
