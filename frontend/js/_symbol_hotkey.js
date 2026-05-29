// Pure logic for the global "type a ticker anywhere" hotkey.
//
// Lives outside the DOM-wiring file so vitest can exercise the buffer
// rules and the route-decision logic without a browser.
//
// Rules:
//   * Allowed buffer chars: A-Z, 0-9, `.`, `-` (BRK.B, RDS-A).
//   * Lowercase letters are uppercased into the buffer.
//   * Max length: 8 chars (longer tickers exist in some markets but for
//     a quick-jump hotkey 8 is the right ceiling — covers BRKB, NVDA,
//     INTC, RDS-A, BRK.B, etc. and avoids accidentally grabbing
//     paragraphs of typed text).

const MAX_BUFFER_LEN = 8;

/** Returns true if `ch` is a single character that belongs in a ticker.
 *  Accepts upper/lower letters, digits, `.`, `-`. */
export function isTickerChar(ch) {
    if (typeof ch !== 'string' || ch.length !== 1) return false;
    return /^[A-Za-z0-9.\-]$/.test(ch);
}

/** Buffer state. Methods mutate `this` and return `this` for chaining. */
export class SymbolBuffer {
    constructor() { this.value = ''; }
    appendChar(ch) {
        if (!isTickerChar(ch)) return this;
        if (this.value.length >= MAX_BUFFER_LEN) return this;
        this.value += ch.toUpperCase();
        return this;
    }
    backspace() { this.value = this.value.slice(0, -1); return this; }
    reset()     { this.value = ''; return this; }
    isEmpty()   { return this.value.length === 0; }
    /** Is the current value a plausibly-committable ticker? Requires
     *  at least one letter and length 1-8. */
    isValid() {
        if (this.value.length === 0 || this.value.length > MAX_BUFFER_LEN) return false;
        // At least one letter — pure-digit "tickers" aren't a thing on US
        // exchanges and almost always indicate the user is typing
        // something else (a number into a non-input page).
        return /[A-Z]/.test(this.value);
    }
}

// Routes that DO take a symbol as their first path segment. When the
// user is on one of these, the hotkey replaces the symbol in the
// current route. Otherwise we navigate to /research/<SYMBOL> as the
// canonical symbol-detail page.
const SYMBOL_AWARE_ROUTES = new Set([
    'research', 'charts', 'options', 'earnings-iv', 'sentiment',
    'short-interest', 'darkpool', 'replay', 'tape-replay',
]);

/** Given the current URL hash and a committed symbol, return the next
 *  hash to navigate to. Hash is the raw string (with or without
 *  leading `#`). */
export function decideTargetHash(currentHash, symbol) {
    if (!symbol || typeof symbol !== 'string') return null;
    const sym = symbol.toUpperCase();
    const raw = typeof currentHash === 'string'
        ? currentHash.replace(/^#/, '')
        : '';
    const parts = raw.split('/').filter(Boolean);
    const view = parts[0] || '';
    if (SYMBOL_AWARE_ROUTES.has(view)) {
        return `${view}/${sym}`;
    }
    return `research/${sym}`;
}

/** Decide whether a keydown event is "type-a-letter-on-the-page" that
 *  should be captured. Returns one of: 'append' / 'enter' / 'backspace'
 *  / 'escape' / null (ignore). */
export function classifyKey(e) {
    if (!e) return null;
    // If focus is in an editable surface, never capture.
    const tag = (e.target && e.target.tagName || '').toLowerCase();
    if (tag === 'input' || tag === 'textarea' || tag === 'select') return null;
    if (e.target && e.target.isContentEditable) return null;
    // Modifier keys other than Shift mean "this is a shortcut, leave it
    // alone".
    if (e.metaKey || e.ctrlKey || e.altKey) return null;
    // Special keys.
    if (e.key === 'Enter')      return 'enter';
    if (e.key === 'Backspace')  return 'backspace';
    if (e.key === 'Escape')     return 'escape';
    if (e.key === ' ')          return 'escape';   // space = abandon buffer
    // Single ticker char.
    if (isTickerChar(e.key))    return 'append';
    return null;
}
