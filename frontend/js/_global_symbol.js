// Global active-ticker store.
//
// Single source of truth for "what symbol is the user looking at right
// now." Set by the global type-anywhere hotkey, read by every symbol-
// aware view as the fallback when the URL doesn't carry an explicit
// symbol.
//
// Persistence: written to localStorage so the symbol survives page
// reloads. Subscribe via `onGlobalSymbolChanged` for in-place reactive
// updates without a route change.

const LS_KEY = 'tv-global-symbol';

let current = '';
let initialized = false;

/** Lazy-load from localStorage on first access. Wrapped in try/catch
 *  because localStorage throws in private-browsing / cross-origin
 *  contexts. */
function ensureInit() {
    if (initialized) return;
    initialized = true;
    try {
        const stored = (typeof localStorage !== 'undefined')
            ? localStorage.getItem(LS_KEY) : null;
        if (typeof stored === 'string' && stored.length > 0) {
            current = stored;
        }
    } catch (_) { /* ignore — start empty */ }
}

/** Read the current global symbol. Returns '' when never set. */
export function getGlobalSymbol() {
    ensureInit();
    return current;
}

/** Update the global symbol. Returns true if the value actually
 *  changed (so callers can skip work). Silently no-ops if `sym` isn't
 *  a non-empty string. */
export function setGlobalSymbol(sym) {
    if (typeof sym !== 'string' || sym.length === 0) return false;
    ensureInit();
    const upper = sym.toUpperCase();
    if (upper === current) return false;
    current = upper;
    try {
        if (typeof localStorage !== 'undefined') localStorage.setItem(LS_KEY, current);
    } catch (_) { /* ignore */ }
    // Broadcast to subscribers. Wrap in try/catch — a buggy listener
    // shouldn't take down the rest.
    try {
        if (typeof window !== 'undefined' && typeof CustomEvent === 'function') {
            window.dispatchEvent(new CustomEvent('tv-symbol-changed', {
                detail: { symbol: current },
            }));
        }
    } catch (_) { /* ignore */ }
    return true;
}

/** Subscribe to symbol changes. Returns an unsubscribe function. */
export function onGlobalSymbolChanged(handler) {
    if (typeof handler !== 'function') return () => {};
    const wrapped = (e) => handler(e?.detail?.symbol ?? '');
    if (typeof window !== 'undefined') {
        window.addEventListener('tv-symbol-changed', wrapped);
    }
    return () => {
        if (typeof window !== 'undefined') {
            window.removeEventListener('tv-symbol-changed', wrapped);
        }
    };
}

/** Reset to empty. Test-only helper; not used in production code. */
export function _resetForTests() {
    current = '';
    initialized = false;
    try {
        if (typeof localStorage !== 'undefined') localStorage.removeItem(LS_KEY);
    } catch (_) { /* ignore */ }
}
