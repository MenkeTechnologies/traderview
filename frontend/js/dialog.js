// Modal dialog DOM glue. Public API:
//
//   await tConfirm(messageKey, params?, opts?)
//     → resolves to true if user confirmed, false if cancelled/dismissed.
//
//   await tPrompt(messageKey, params?, opts?)
//     → resolves to the trimmed string the user entered, or null if
//       cancelled (or if opts.required && empty).
//
// Replaces native `confirm()` / `prompt()` — those are blocking, ugly,
// untheme-able, and untranslated. The dialog renders into #tv-dialog-root,
// pulls i18n keys for the buttons + message, and traps Enter/Esc.

import {
    validateOptions, defaultButtons, classFor,
    isConfirmKey, isCancelKey, normalizePromptResult,
} from './_dialog.js';
import { t, applyUiI18n } from './i18n.js';
import { esc } from './util.js';

let _installed = false;

// Close fn of the currently-open dialog, if any. A second showDialog
// while one is open would otherwise clobber root.innerHTML, leaving the
// first dialog's promise unresolved and its capture-phase keydown
// listener orphaned on document — Enter later resolves the stale promise
// and wipes the live dialog.
let _activeClose = null;

export function installDialog() {
    if (_installed) return;
    _installed = true;
    ensureMount();
}

function ensureMount() {
    if (typeof document === 'undefined') return;
    if (document.getElementById('tv-dialog-root')) return;
    const root = document.createElement('div');
    root.id = 'tv-dialog-root';
    document.body.appendChild(root);
}

// Show a confirm dialog. Returns Promise<boolean>.
export function tConfirm(messageKey, params, opts) {
    return showDialog('confirm', messageKey, params || {}, opts || {});
}

// Show a prompt dialog. Returns Promise<string | null>.
export function tPrompt(messageKey, params, opts) {
    return showDialog('prompt', messageKey, params || {}, opts || {});
}

function showDialog(kind, messageKey, params, opts) {
    return new Promise((resolve) => {
        if (typeof document === 'undefined') { resolve(kind === 'confirm' ? false : null); return; }
        const err = validateOptions(opts);
        if (err) { console.warn('dialog options invalid:', err); resolve(kind === 'confirm' ? false : null); return; }
        ensureMount();
        const root = document.getElementById('tv-dialog-root');
        if (!root) { resolve(kind === 'confirm' ? false : null); return; }
        // Dismiss any dialog already open (resolves it as cancelled and
        // detaches its listeners) before rendering this one.
        if (_activeClose) _activeClose(undefined, true);
        const level = opts.level || (kind === 'confirm' ? 'info' : 'info');
        const defaults = defaultButtons(kind, level);
        const confirmKey = opts.confirmKey || defaults.confirmKey;
        const cancelKey  = opts.cancelKey  || defaults.cancelKey;
        const title    = opts.titleKey ? esc(t(opts.titleKey, params)) : '';
        const message  = esc(t(messageKey, params));
        // Raw dynamic context under the message (account lists, max qty,
        // symbol/ATR hints) — content that isn't translatable copy and so
        // doesn't belong inside the i18n message template.
        const detail   = opts.detail ? `<div class="tv-dialog-detail">${esc(opts.detail)}</div>` : '';
        const inputHtml = kind === 'prompt'
            ? `<input id="tv-dialog-input"
                      class="tv-dialog-input"
                      type="text"
                      autocomplete="off"
                      spellcheck="false"
                      placeholder="${esc(opts.placeholder || '')}"
                      value="${esc(opts.defaultValue || '')}">`
            : '';
        root.innerHTML = `
            <div class="tv-dialog-overlay" role="dialog" aria-modal="true">
                <div class="${classFor(level)}">
                    ${title ? `<div class="tv-dialog-title">${title}</div>` : ''}
                    <div class="tv-dialog-message">${message}</div>
                    ${detail}
                    ${inputHtml}
                    <div class="tv-dialog-actions">
                        <button type="button"
                                class="tv-dialog-btn tv-dialog-cancel"
                                data-i18n="${esc(cancelKey)}">${esc(t(cancelKey))}</button>
                        <button type="button"
                                class="tv-dialog-btn tv-dialog-confirm"
                                data-i18n="${esc(confirmKey)}">${esc(t(confirmKey))}</button>
                    </div>
                </div>
            </div>
        `;
        applyUiI18n(root);
        const overlay = root.querySelector('.tv-dialog-overlay');
        const input   = root.querySelector('#tv-dialog-input');
        const confirmBtn = root.querySelector('.tv-dialog-confirm');
        const cancelBtn  = root.querySelector('.tv-dialog-cancel');

        const close = (result, displaced = false) => {
            document.removeEventListener('keydown', onKey, true);
            if (_activeClose === close) _activeClose = null;
            if (!displaced) root.innerHTML = '';
            resolve(displaced ? (kind === 'confirm' ? false : null) : result);
        };
        _activeClose = close;
        const confirmAction = () => {
            if (kind === 'prompt') {
                const raw = input ? input.value : '';
                const norm = normalizePromptResult(raw, opts);
                if (norm === null) {
                    // required-but-empty: shake the input rather than resolve.
                    if (input) {
                        input.classList.add('tv-dialog-input-error');
                        input.focus();
                        setTimeout(() => input.classList.remove('tv-dialog-input-error'), 400);
                    }
                    return;
                }
                close(norm);
            } else {
                close(true);
            }
        };
        const cancelAction = () => close(kind === 'confirm' ? false : null);
        const onKey = (e) => {
            // Trap Tab inside the dialog — without this, focus walks into
            // the page behind the overlay and Enter then confirms blind
            // (a destructive default on level:'danger' confirms).
            if (e.key === 'Tab') {
                const focusables = [input, cancelBtn, confirmBtn].filter(Boolean);
                const first = focusables[0];
                const last = focusables[focusables.length - 1];
                if (!root.contains(document.activeElement)) {
                    e.preventDefault(); first.focus();
                } else if (e.shiftKey && document.activeElement === first) {
                    e.preventDefault(); last.focus();
                } else if (!e.shiftKey && document.activeElement === last) {
                    e.preventDefault(); first.focus();
                }
                return;
            }
            if (isConfirmKey(e) && document.activeElement !== cancelBtn
                && root.contains(document.activeElement)) {
                e.preventDefault(); e.stopPropagation();
                confirmAction();
            } else if (isCancelKey(e)) {
                e.preventDefault(); e.stopPropagation();
                cancelAction();
            }
        };
        document.addEventListener('keydown', onKey, true);
        if (confirmBtn) confirmBtn.addEventListener('click', confirmAction);
        if (cancelBtn)  cancelBtn.addEventListener('click', cancelAction);
        if (overlay) overlay.addEventListener('click', (e) => {
            if (e.target === overlay) cancelAction();
        });
        // Focus the input for prompt, confirm button for confirm.
        requestAnimationFrame(() => {
            if (input) { input.focus(); input.select(); }
            else if (confirmBtn) confirmBtn.focus();
        });
    });
}
