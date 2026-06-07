// Global symbol autocomplete.
//
// Installs a hidden `<datalist id="tv-symbol-datalist">` populated
// from `/api/symbols/list` (~11k US listings, seeded from Finnhub on
// first call), then attaches `list="tv-symbol-datalist"` to every
// `<input>` that looks like a symbol field — `name="symbol"`,
// `name="symbols"`, or a data-symbol-input opt-in. Native `<datalist>`
// match is case-insensitive in every modern browser, so typing "aapl"
// surfaces `AAPL`.
//
// New inputs added later (modal mounts, view re-renders) get wired
// automatically via the same MutationObserver pattern as
// `_no_spellcheck.js`.

import { api } from './api.js';

const DATALIST_ID = 'tv-symbol-datalist';
// Inputs the autocomplete attaches to. Anything matching
// `[data-no-symbol-list]` opts out (e.g. comma-separated symbol
// inputs that don't want native datalist behavior).
const SELECTOR = [
    'input[name="symbol"]',
    'input[name="symbols"]',
    'input[name="sym"]',
    'input[name="ticker"]',
    'input[data-symbol-input]',
].join(',');

const HANDLED = new WeakSet();
let _ready = false;
let _readyPromise = null;

async function ensureDatalist() {
    if (_ready) return;
    if (_readyPromise) return _readyPromise;
    _readyPromise = (async () => {
        let list = document.getElementById(DATALIST_ID);
        if (!list) {
            list = document.createElement('datalist');
            list.id = DATALIST_ID;
            document.body.appendChild(list);
        }
        try {
            const rows = await api.symbolsList(true);
            list.innerHTML = (Array.isArray(rows) ? rows : [])
                .map((r) => {
                    const sym = String(r.symbol || '').toUpperCase();
                    // description (Finnhub) is richer; fall back to the
                    // legacy `name` column (migration 0007) so rows that
                    // existed before the catalog seed still display.
                    const label = r.description || r.name || '';
                    const tail = label ? ` — ${label}` : '';
                    return `<option value="${escAttr(sym)}">${escAttr(sym + tail)}</option>`;
                })
                .join('');
            _ready = true;
        } catch (e) {
            console.warn('symbol catalog fetch failed', e?.message || e);
        }
    })();
    return _readyPromise;
}

function wire(el) {
    if (!el || HANDLED.has(el)) return;
    if (el.hasAttribute('data-no-symbol-list')) return;
    HANDLED.add(el);
    el.setAttribute('list', DATALIST_ID);
    // Browsers ignore datalist when `autocomplete="off"` is set — flip
    // it back on for our symbol inputs so the suggestion panel appears.
    el.setAttribute('autocomplete', 'on');
    // Auto-uppercase as the user types. CSS `text-transform: uppercase`
    // only affects display; .value stays as typed and gets read by JS
    // as lowercase. This rewrites the live value so every code path
    // (form submit, fetch URL, search params) sees the canonical form.
    // Caret position is preserved.
    el.addEventListener('input', () => {
        const up = el.value.toUpperCase();
        if (el.value === up) return;
        const start = el.selectionStart;
        const end = el.selectionEnd;
        el.value = up;
        try { el.setSelectionRange(start, end); } catch {}
    });
}

function sweep(root = document) {
    if (!root || !root.querySelectorAll) return;
    root.querySelectorAll(SELECTOR).forEach(wire);
}

export function installSymbolAutocomplete() {
    if (typeof document === 'undefined') return;
    if (typeof window !== 'undefined' && window.__tvSymAutocompleteInstalled) return;
    if (typeof window !== 'undefined') window.__tvSymAutocompleteInstalled = true;

    // Kick off the datalist fetch async — every input mounted before
    // the fetch resolves still gets `list=` wired immediately, the
    // browser just shows an empty completion menu until the fetch
    // lands. Once the fetch lands the same datalist is shared by
    // every input via id.
    ensureDatalist();
    sweep(document);
    const obs = new MutationObserver((mutations) => {
        for (const m of mutations) {
            for (const node of m.addedNodes) {
                if (node.nodeType !== 1) continue;
                if (node.matches?.(SELECTOR)) wire(node);
                else sweep(node);
            }
        }
    });
    obs.observe(document.body, { childList: true, subtree: true });
}

function escAttr(s) {
    return String(s).replace(/[&<>"]/g, (c) => ({
        '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;',
    }[c]));
}
