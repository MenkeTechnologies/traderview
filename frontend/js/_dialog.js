// Modal dialog pure helpers. Concept: non-blocking confirm/prompt that
// returns a Promise — replaces the native `confirm()` / `prompt()` calls
// scattered through view code with something that's themed, i18n-aware,
// keyboard-navigable, and not OS-modal.
//
// This file is the pure-JS surface (no DOM) so it stays testable
// without jsdom. The DOM glue lives in dialog.js.

// Validate the options blob a caller passes to showConfirm / showPrompt.
// Returns an error string on bad input, null on success.
export function validateOptions(opts) {
    if (opts == null) return null;
    if (typeof opts !== 'object') return 'opts must be an object';
    if (opts.level != null && !['info', 'warning', 'danger'].includes(opts.level)) {
        return `level must be info|warning|danger, got ${JSON.stringify(opts.level)}`;
    }
    if (opts.confirmKey != null && typeof opts.confirmKey !== 'string') {
        return 'confirmKey must be a string i18n key';
    }
    if (opts.cancelKey != null && typeof opts.cancelKey !== 'string') {
        return 'cancelKey must be a string i18n key';
    }
    if (opts.defaultValue != null && typeof opts.defaultValue !== 'string') {
        return 'defaultValue must be a string';
    }
    if (opts.placeholder != null && typeof opts.placeholder !== 'string') {
        return 'placeholder must be a string';
    }
    return null;
}

// Default i18n key set per dialog kind. Callers can override via opts.
export function defaultButtons(kind, level) {
    if (kind === 'prompt') {
        return { confirmKey: 'dialog.btn.ok', cancelKey: 'dialog.btn.cancel' };
    }
    // confirm: "danger" → "Delete" verb is more honest than "OK"
    if (level === 'danger') {
        return { confirmKey: 'dialog.btn.delete', cancelKey: 'dialog.btn.cancel' };
    }
    return { confirmKey: 'dialog.btn.confirm', cancelKey: 'dialog.btn.cancel' };
}

// Class name applied to the dialog card based on level. Drives styling
// (cyan for info, amber for warning, red for danger).
export function classFor(level) {
    if (level === 'warning') return 'tv-dialog-card tv-dialog-warning';
    if (level === 'danger')  return 'tv-dialog-card tv-dialog-danger';
    return 'tv-dialog-card tv-dialog-info';
}

// True if the event resolves the dialog's "confirm" action. Centralized
// so the spec can pin behavior across Enter / Cmd+Enter / mod+Return.
export function isConfirmKey(e) {
    if (!e || typeof e.key !== 'string') return false;
    return e.key === 'Enter';
}

// True if the event resolves the dialog's "cancel" action.
export function isCancelKey(e) {
    if (!e || typeof e.key !== 'string') return false;
    return e.key === 'Escape';
}

// Normalize the result of a prompt: trim, optionally apply caller's
// trim/coerce hook. Returns null when the trimmed result is empty AND
// the caller required a non-empty value (opts.required === true).
export function normalizePromptResult(raw, opts) {
    const s = (raw == null ? '' : String(raw)).trim();
    if (opts && opts.required && !s) return null;
    return s;
}
