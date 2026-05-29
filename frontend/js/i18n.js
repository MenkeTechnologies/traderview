// Lightweight i18n for traderview. Same conceptual model as audio-haxor's
// i18n-ui.js: a key→string map at `window.__appStr`, with data-i18n*
// element annotations applied by `applyUiI18n()`. Adds a `t()` helper
// for programmatic lookup + parameter interpolation.

let _locale = 'en';
let _map = {};
const LS_KEY = 'tv-locale-v1';

// Lookup a key. Returns the key itself on miss (so untranslated strings
// are visible as keys, not blanks). Params: `{name}` placeholder syntax.
export function t(key, params) {
    let s = _map[key];
    if (s == null || s === '') s = key;
    if (params && typeof params === 'object') {
        s = s.replace(/\{(\w+)\}/g, (_, name) =>
            params[name] != null ? String(params[name]) : `{${name}}`);
    }
    return s;
}

// Apply current map to all annotated DOM elements under `root`. Idempotent.
// Mirrors audio-haxor `applyUiI18n` semantics:
//   data-i18n         → textContent
//   data-i18n-placeholder → placeholder
//   data-i18n-title   → title
//   data-i18n-aria-label  → aria-label
export function applyUiI18n(root) {
    const scope = root != null
        ? root
        : (typeof document !== 'undefined' ? document : null);
    if (!scope || typeof scope.querySelectorAll !== 'function') return 0;
    let n = 0;
    scope.querySelectorAll('[data-i18n]').forEach(el => {
        const k = el.dataset.i18n;
        if (!k) return;
        const v = _map[k];
        if (v != null && v !== '') { el.textContent = v; n++; }
    });
    scope.querySelectorAll('[data-i18n-placeholder]').forEach(el => {
        const k = el.dataset.i18nPlaceholder;
        if (!k) return;
        const v = _map[k];
        if (v != null && v !== '') { el.placeholder = decodeNewlines(v); n++; }
    });
    scope.querySelectorAll('[data-i18n-title]').forEach(el => {
        const k = el.dataset.i18nTitle;
        if (!k) return;
        const v = _map[k];
        if (v != null && v !== '') { el.title = v; n++; }
    });
    scope.querySelectorAll('[data-i18n-aria-label]').forEach(el => {
        const k = el.dataset.i18nAriaLabel;
        if (!k) return;
        const v = _map[k];
        if (v != null && v !== '') { el.setAttribute('aria-label', v); n++; }
    });
    return n;
}

// HTML / JSON-attribute values can store newlines as `&#10;`. JS-set
// .placeholder needs a literal '\n'.
export function decodeNewlines(s) {
    if (typeof s !== 'string') return s;
    return s.replace(/&#10;/g, '\n').replace(/&#13;/g, '\r');
}

// Replace the active catalog with a new map. Triggers `tv:i18n-changed`.
export function setMap(map) {
    if (!map || typeof map !== 'object') return;
    _map = { ...map };
    if (typeof window !== 'undefined') {
        window.__appStr = _map;
        window.dispatchEvent(new CustomEvent('tv:i18n-changed', { detail: { locale: _locale } }));
    }
}

// Add/override individual entries without replacing the whole map. Useful
// for letting view modules contribute their own strings post-boot.
export function extendMap(partial) {
    if (!partial || typeof partial !== 'object') return;
    Object.assign(_map, partial);
    if (typeof window !== 'undefined') {
        window.__appStr = _map;
        window.dispatchEvent(new CustomEvent('tv:i18n-changed', { detail: { locale: _locale } }));
    }
}

export function currentLocale() { return _locale; }
export function getMap() { return _map; }

// Fetch a locale JSON from /i18n/<code>.json and apply it. Returns the
// number of keys loaded, or 0 on failure (with the existing map intact).
// File-naming convention follows audio_haxor: `i18n/app_i18n_<code>.json`.
// When the locale file is empty / partial, fall through to the English
// catalog so the UI never shows blanks.
export async function loadLocale(code) {
    if (typeof fetch !== 'function') return 0;
    try {
        const resp = await fetch(`i18n/app_i18n_${encodeURIComponent(code)}.json`);
        if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
        const localized = await resp.json();
        let base = {};
        if (code !== 'en') {
            try {
                const enResp = await fetch('i18n/app_i18n_en.json');
                if (enResp.ok) base = await enResp.json();
            } catch { /* en seed missing; ship localized-only */ }
        }
        const merged = { ...base, ...localized };
        _locale = code;
        if (typeof localStorage !== 'undefined') {
            try { localStorage.setItem(LS_KEY, code); } catch { /* private mode */ }
        }
        setMap(merged);
        applyUiI18n();
        return Object.keys(merged).length;
    } catch {
        return 0;
    }
}

// Boot helper: read persisted locale (defaults to 'en') and load it.
export async function bootI18n(defaultCode = 'en') {
    let code = defaultCode;
    if (typeof localStorage !== 'undefined') {
        const saved = localStorage.getItem(LS_KEY);
        if (saved) code = saved;
    }
    return loadLocale(code);
}
