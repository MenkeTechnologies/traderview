// Toast notification DOM glue. Concept ported from audio-haxor.
//
// Public API:
//   showToast(message, opts?)   — message is a STRING already translated
//                                 (caller passes t('key', {param}) if needed)
//   tShowToast(key, params?, opts?) — convenience that calls t() for you
//
// Also listens for `tv:toast` CustomEvent with detail = { message?, key?, params?, ...opts }.

import { normalizeOptions, validateOptions, classFor, animationFor, coalesceKey,
         apiErrorToastDetail, iconFor, DEFAULT_DURATION_MS }
    from './_toast.js';
import { t } from './i18n.js';

let _installed = false;
const _liveByKey = new Map();  // dedupe key → element

export function installToasts() {
    if (_installed) return;
    _installed = true;
    ensureMount();
    window.addEventListener('tv:toast', onToastEvent);
    window.addEventListener('tv:api-error', onApiError);
}

function onApiError(e) {
    const payload = apiErrorToastDetail(e && e.detail);
    if (!payload) return;
    const label = payload.params.labelKey
        ? t(payload.params.labelKey)
        : payload.params.httpLabel;
    showToast(
        t(payload.messageKey, {
            method: payload.params.method,
            path:   payload.params.path,
            label,
        }),
        { level: payload.level, duration: payload.duration });
}

function ensureMount() {
    if (document.getElementById('tv-toast-root')) return;
    const root = document.createElement('div');
    root.id = 'tv-toast-root';
    root.setAttribute('role', 'region');
    root.setAttribute('aria-live', 'polite');
    root.setAttribute('aria-label', 'Notifications');
    document.body.appendChild(root);
}

function onToastEvent(e) {
    const d = e && e.detail || {};
    const msg = d.message != null ? d.message : (d.key ? t(d.key, d.params) : '');
    if (!msg) return;
    showToast(msg, { duration: d.duration, level: d.level, extraClass: d.extraClass });
}

export function showToast(message, opts) {
    if (typeof window === 'undefined' || typeof document === 'undefined') return null;
    const err = validateOptions(opts);
    if (err) { console.warn('toast options invalid:', err); return null; }
    const norm = normalizeOptions(opts);
    ensureMount();
    const root = document.getElementById('tv-toast-root');
    if (!root) return null;
    const key = coalesceKey(message, norm.level);
    // Coalesce: if same toast already up, just reset its timer.
    if (_liveByKey.has(key)) {
        const existing = _liveByKey.get(key);
        if (existing._timeout) clearTimeout(existing._timeout);
        existing._timeout = setTimeout(() => dismiss(existing, key), norm.duration);
        return existing;
    }
    const el = document.createElement('div');
    el.className = classFor(norm.level, norm.extraClass);
    el.textContent = `${iconFor(norm.level)} ${message}`;
    el.style.animation = animationFor(norm.duration);
    el.addEventListener('click', () => dismiss(el, key));
    root.appendChild(el);
    _liveByKey.set(key, el);
    el._timeout = setTimeout(() => dismiss(el, key), norm.duration);
    return el;
}

function dismiss(el, key) {
    if (!el) return;
    if (_liveByKey.get(key) === el) _liveByKey.delete(key);
    if (el._timeout) { clearTimeout(el._timeout); el._timeout = null; }
    if (el.parentNode) el.parentNode.removeChild(el);
}

// Convenience: call t() internally — most call-sites want this so they
// don't repeat the i18n import.
export function tShowToast(key, params, opts) {
    return showToast(t(key, params || {}), opts);
}

// Re-export shorthand for default duration.
export { DEFAULT_DURATION_MS };
