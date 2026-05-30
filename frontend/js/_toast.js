// Toast helpers (pure, no DOM) — registry of pending toasts + classifier.
// Mirrors the audio-haxor pattern: showToast(message, duration, type, extraClass).
//
// LEVELS = info | success | warning | error. CSS classes use `toast-<level>`.

import { t } from './i18n.js';

export const DEFAULT_DURATION_MS = 2500;
export const LEVELS = ['info', 'success', 'warning', 'error'];

// Validate caller-supplied options up front so the DOM glue stays simple.
export function validateOptions(opts) {
    const o = opts || {};
    if (o.duration != null && (!Number.isFinite(o.duration) || o.duration < 0))
        return t('toast.validate.duration_non_neg');
    if (o.level != null && !LEVELS.includes(o.level))
        return t('toast.validate.level_invalid', { levels: LEVELS.join(', ') });
    return null;
}

// Normalize options + apply defaults.
export function normalizeOptions(opts) {
    const o = opts || {};
    return {
        duration: Number.isFinite(o.duration) ? o.duration : DEFAULT_DURATION_MS,
        level:    o.level || 'info',
        extraClass: o.extraClass || '',
    };
}

// Glyph prefix for a level. Helps colorblind users distinguish levels
// without relying on the border color. Uses textContent (no ::before)
// to stay safe under the Tauri release-WebKit pseudo-element rule.
export function iconFor(level) {
    switch (level) {
        case 'success': return '✓';
        case 'warning': return '⚠';
        case 'error':   return '✕';
        case 'info':    return 'ⓘ';
        default:        return 'ⓘ';
    }
}

// Compute the CSS class string for a toast given the level + extra class.
export function classFor(level, extraClass) {
    let cls = 'tv-toast';
    if (level && level !== 'info') cls += ` tv-toast-${level}`;
    if (extraClass) cls += ` ${extraClass}`;
    return cls;
}

// Compute the animation property string. Audio-haxor uses two animations:
// toast-in immediately, toast-out triggered (duration - 300)ms later so
// the fade-out finishes exactly at `duration`.
export function animationFor(duration) {
    const out = Math.max(0, (duration - 300) / 1000);
    return `tv-toast-in 0.3s ease-out, tv-toast-out 0.3s ease-in ${out}s forwards`;
}

// Coalesce key: identical (level, message) toasts fired in quick succession
// should not stack — return the dedupe key.
export function coalesceKey(message, level) {
    return `${level || 'info'}|${message || ''}`;
}

// Decide whether an API failure should be surfaced as a toast. Server errors
// (5xx) and network failures (status 0) are always toasted because the user
// can't fix them and per-view UI often hides them. 4xx is intentionally
// silent — those are validation errors that views usually surface inline.
// /client-errors is suppressed to avoid recursion (it's the reporter sink).
export function shouldToastApiError(detail) {
    if (!detail || typeof detail !== 'object') return false;
    if (detail.path === '/client-errors') return false;
    const status = detail.status | 0;
    if (status === 0) return true;
    return status >= 500 && status <= 599;
}

// Build the toast payload for an API failure. Returns null when the failure
// shouldn't be toasted at all. Callers translate the keys themselves.
export function apiErrorToastDetail(detail) {
    if (!shouldToastApiError(detail)) return null;
    const d = detail || {};
    const status = d.status | 0;
    const labelKey = status === 0 ? 'toast.network_down' : null;
    return {
        messageKey: 'toast.api_failed',
        params: {
            method: d.method || '?',
            path:   d.path   || '?',
            labelKey,                            // present when network down
            httpLabel: status !== 0 ? `HTTP ${status}` : null,
        },
        level: 'error',
        duration: 6000,
    };
}
