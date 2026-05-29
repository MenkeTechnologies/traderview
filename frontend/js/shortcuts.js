// DOM glue for the keyboard-shortcut registry. Listens on document
// `keydown`, dispatches `actionKey` CustomEvents on `window` when a
// registered shortcut fires. The actual side effects (open palette,
// open help, navigate) are wired by listeners elsewhere — this module
// owns the input layer only.

import {
    loadShortcuts, findMatch, isTextEntryTarget, firesInEditableContext,
} from './_shortcuts.js';

let _shortcuts = [];
let _scope = 'global';
let _installed = false;

export function installShortcuts() {
    if (_installed) return;
    _installed = true;
    _shortcuts = loadShortcuts();
    document.addEventListener('keydown', onKeydown, { capture: true });
}

function onKeydown(e) {
    const sc = findMatch(e, _shortcuts, _scope);
    if (!sc) return;
    if (isTextEntryTarget(e.target) && !firesInEditableContext(sc)) return;
    e.preventDefault();
    e.stopPropagation();
    window.dispatchEvent(new CustomEvent(sc.actionKey, { detail: { shortcut: sc } }));
}

export function setScope(scope) { _scope = scope || 'global'; }
export function currentScope() { return _scope; }
export function listShortcuts() { return _shortcuts.map(sc => ({ ...sc })); }

// Allow runtime additions (e.g. a view registers its own context-scoped
// shortcut). Caller-supplied shortcut must include id, keys, scope,
// actionKey. Returns true if appended (id was unique).
export function registerShortcut(sc) {
    if (!sc || !sc.id || _shortcuts.some(x => x.id === sc.id)) return false;
    _shortcuts.push({ ...sc });
    return true;
}

// Refresh the registry from localStorage (e.g. after the user rebinds
// a shortcut in Preferences). Idempotent.
export function reloadShortcuts() {
    _shortcuts = loadShortcuts();
}
