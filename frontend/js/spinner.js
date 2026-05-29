// Spinner — async-operation loading indicator. Two flavors:
//
//   spinnerHTML(text?)        — string for innerHTML, used in views that
//                               build a section then await data
//   withSpinner(el, promise)  — wraps the element while the promise runs,
//                               restores prior content on success/error
//   spinnerOverlay(el)        — push a translucent overlay on top of an
//                               element so it stays visible during refresh
//
// Visual: 3-arc orbital CSS spinner styled with the active HUD palette.
// All pure DOM; safe to call from any view's render path.

import { t } from './i18n.js';

export function spinnerHTML(text = t('common.loading')) {
    const safe = escapeText(text);
    return `<div class="tv-spinner-wrap">
        <div class="tv-spinner" role="status" aria-label="${safe}"></div>
        <div class="tv-spinner-text">${safe}</div>
    </div>`;
}

/// Replace `el.innerHTML` with a spinner while `promise` runs; restore the
/// original content if the promise rejects, leave-as-is on resolve (caller
/// renders the result). Returns the promise so it composes.
export async function withSpinner(el, promise, text = t('common.loading')) {
    if (!el) return promise;
    const prev = el.innerHTML;
    el.innerHTML = spinnerHTML(text);
    try {
        return await promise;
    } catch (e) {
        el.innerHTML = prev;
        throw e;
    }
}

/// Push a translucent loading overlay onto an element without blowing
/// away its content. The overlay is removed when the returned `dispose()`
/// is called. Use during refresh of a panel that should stay visible.
export function spinnerOverlay(el, text = t('common.refreshing')) {
    if (!el) return () => {};
    const prevPos = el.style.position;
    if (!prevPos || prevPos === 'static') el.style.position = 'relative';
    const overlay = document.createElement('div');
    overlay.className = 'tv-spinner-overlay';
    overlay.innerHTML = spinnerHTML(text);
    el.appendChild(overlay);
    return () => {
        try { overlay.remove(); } catch (_) {}
        el.style.position = prevPos;
    };
}

/// Disable a button + show a tiny inline spinner inside it while a
/// promise runs. Restores label + enabled state on completion.
export async function buttonSpinner(btn, promise, busyLabel = t('common.working')) {
    if (!btn) return promise;
    const prevLabel = btn.innerHTML;
    const prevDisabled = btn.disabled;
    btn.disabled = true;
    btn.innerHTML = `<span class="tv-spinner tv-spinner-inline" aria-hidden="true"></span> ${escapeText(busyLabel)}`;
    try {
        return await promise;
    } finally {
        btn.disabled = prevDisabled;
        btn.innerHTML = prevLabel;
    }
}

function escapeText(s) {
    return String(s == null ? '' : s)
        .replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}
