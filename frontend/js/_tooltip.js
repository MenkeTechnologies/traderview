// Tooltip pure helpers. Concept: elements declare `data-tip="<i18n-key>"`
// and the glue layer (tooltip.js / applyTooltips) sets the matching
// translated string as the element's native `title` attribute. Native
// title gets the OS hover tooltip for free + screen-reader access.
//
// This file is the pure-JS surface (no DOM) so the matcher is testable.

// Decide whether to assign `data-i18n-title` (handled by applyUiI18n)
// or just set the title directly. We always prefer the data-i18n-title
// path so i18n re-applies on locale change. Returns the attribute pair
// the caller should set.
export function tipAttrsFor(key) {
    if (!key || typeof key !== 'string') return null;
    return { 'data-i18n-title': key };
}

// Walk an items list and return the data-tip selectors to upgrade.
// (Used by the DOM glue + the spec.)
export function tipSelectors() {
    return ['[data-tip]', '[data-tooltip]', '[data-tip-key]'];
}

// Extract the i18n key from any of the supported attribute names.
export function tipKey(el) {
    if (!el || !el.dataset) return null;
    return el.dataset.tip || el.dataset.tooltip || el.dataset.tipKey || null;
}

// Extract the shortcut id declared on the element. Used by the augmenter
// to append a "(⌘K)" chip to existing tooltips.
export function shortcutId(el) {
    if (!el || !el.dataset) return null;
    return el.dataset.shortcut || null;
}

// Compose a final tooltip string from a translated tip + optional chip.
// Adds two spaces of separator so the chip reads as a distinct hint.
export function composeTooltip(translatedTip, shortcutChip) {
    const tip  = (translatedTip || '').trim();
    const chip = (shortcutChip || '').trim();
    if (!tip && !chip) return '';
    if (!tip)  return chip;
    if (!chip) return tip;
    return `${tip}  (${chip})`;
}
