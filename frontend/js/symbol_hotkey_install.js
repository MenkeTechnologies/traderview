// DOM wiring for the global "type a ticker anywhere" hotkey.
//
// Behavior:
//   * Anywhere outside an input/textarea/contenteditable, the user
//     types letters/digits and the buffer accumulates.
//   * A floating HUD overlay appears in the bottom-center showing the
//     current buffer + hint text.
//   * ENTER commits immediately; idle of 1.5s also commits.
//   * BACKSPACE removes the last char; ESC or SPACE abandons.
//   * Commit: navigates to the current view with the new symbol if the
//     view is symbol-aware (research/charts/options/etc.), or jumps
//     to research/<SYMBOL> otherwise.
//   * Also dispatches a `tv-symbol-changed` CustomEvent so any view
//     that wants to react in-place can subscribe without a route
//     change.

import {
    SymbolBuffer, classifyKey, decideTargetHash,
} from './_symbol_hotkey.js';
import { setGlobalSymbol, getGlobalSymbol, onGlobalSymbolChanged } from './_global_symbol.js';

const IDLE_COMMIT_MS = 1500;

let buffer = new SymbolBuffer();
let idleTimer = null;
let hud = null;

export function installSymbolHotkey() {
    document.addEventListener('keydown', onKeydown);
    // Some events come from focus that lives in the iframe-less WebView;
    // keypress on the document is fine for our purposes.
    wireTopbarIndicator();
}

/** Show + keep-in-sync the topbar "active ticker" pill so the user
 *  always has a visible cue of what symbol the app is centered on. */
function wireTopbarIndicator() {
    const el = document.getElementById('globalTicker');
    if (!el) return;
    const sync = (sym) => {
        if (sym && typeof sym === 'string') {
            el.textContent = sym;
            el.style.display = '';
            el.setAttribute('href', `#research/${encodeURIComponent(sym)}`);
        } else {
            el.style.display = 'none';
        }
    };
    sync(getGlobalSymbol());
    onGlobalSymbolChanged(sync);
}

function onKeydown(e) {
    const kind = classifyKey(e);
    if (kind === null) return;

    // Once we're handling, defang the default to avoid scrolling on space etc.
    if (kind === 'append' || kind === 'backspace' || kind === 'escape' || kind === 'enter') {
        // Only prevent default when there's actually buffered state — a
        // fresh ENTER on an empty buffer should NOT swallow the default
        // (e.g. opening focused buttons).
        if (kind !== 'enter' || !buffer.isEmpty()) {
            e.preventDefault();
        }
    }

    if (kind === 'escape') { abandon(); return; }
    if (kind === 'backspace') {
        if (buffer.isEmpty()) return;
        buffer.backspace();
        renderHud();
        resetIdle();
        return;
    }
    if (kind === 'append') {
        buffer.appendChar(e.key);
        renderHud();
        resetIdle();
        return;
    }
    if (kind === 'enter') {
        commit();
        return;
    }
}

function ensureHud() {
    if (hud) return hud;
    hud = document.createElement('div');
    hud.className = 'tv-symbol-hud';
    hud.style.display = 'none';
    hud.innerHTML = `
        <span class="tv-symbol-hud-prompt" data-i18n="symbol_hud.prompt">Symbol</span>
        <span class="tv-symbol-hud-value" id="tv-symbol-hud-value"></span>
        <span class="tv-symbol-hud-hint" data-i18n="symbol_hud.hint">enter to jump · esc to cancel</span>
    `;
    document.body.appendChild(hud);
    // Translate inline strings if the i18n catalog is ready.
    try {
        void import('./i18n.js').then(m => m.applyUiI18n(hud));
    } catch (_) { /* i18n not boot-ready yet */ }
    return hud;
}

function renderHud() {
    ensureHud();
    if (buffer.isEmpty()) {
        hud.style.display = 'none';
        return;
    }
    hud.style.display = '';
    const valEl = document.getElementById('tv-symbol-hud-value');
    if (valEl) valEl.textContent = buffer.value;
    // Pulse green when isValid() to signal "ENTER will work".
    hud.classList.toggle('tv-symbol-hud-valid', buffer.isValid());
}

function resetIdle() {
    if (idleTimer) clearTimeout(idleTimer);
    idleTimer = setTimeout(() => {
        commit();
    }, IDLE_COMMIT_MS);
}

function abandon() {
    if (idleTimer) { clearTimeout(idleTimer); idleTimer = null; }
    buffer.reset();
    renderHud();
}

function commit() {
    if (idleTimer) { clearTimeout(idleTimer); idleTimer = null; }
    if (!buffer.isValid()) {
        abandon();
        return;
    }
    const sym = buffer.value;
    buffer.reset();
    renderHud();
    // The store dispatches `tv-symbol-changed` itself, so listeners are
    // notified regardless of whether the hash also changes below.
    setGlobalSymbol(sym);
    // Sync the URL so the route reflects the active symbol — preserves
    // back/forward navigation and makes the URL shareable.
    const target = decideTargetHash(window.location.hash, sym);
    if (target && target !== window.location.hash.replace(/^#/, '')) {
        window.location.hash = target;
    }
}
