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

// CSS selector covering every interactive element type used in the
// codebase. Used by autoApplyTooltips to ensure every clickable thing
// has at least a `title` attribute for hover discovery.
export function interactiveSelectors() {
    return [
        'button',
        'a[href]',
        '[role="button"]',
        '[role="link"]',
        '[role="menuitem"]',
        '[role="menuitemcheckbox"]',
        '[role="menuitemradio"]',
        '[role="tab"]',
        '[role="option"]',
        '[role="switch"]',
        '[role="checkbox"]',
        'summary',
        'select',
        'input[type="button"]',
        'input[type="submit"]',
        'input[type="reset"]',
        'input[type="checkbox"]',
        'input[type="radio"]',
        'input[type="file"]',
        'input[type="color"]',
        'input[type="search"]',
        'input[type="email"]',
        'input[type="tel"]',
        'input[type="url"]',
        'input[type="date"]',
        'input[type="time"]',
        'input[type="datetime-local"]',
        'input[type="month"]',
        'input[type="week"]',
        'input[type="range"]',
        'label[for]',
        '[onclick]',
        '[tabindex]:not([tabindex="-1"])',
    ].join(',');
}

// Squash whitespace + cap length. Native title tooltips look ugly past
// ~80 chars and the browser truncates with no ellipsis anyway.
export function normalizeTitle(s, max = 80) {
    if (s == null) return '';
    const t = String(s).replace(/\s+/g, ' ').trim();
    if (t.length <= max) return t;
    return t.slice(0, max - 1).trimEnd() + '…';
}

// Derive a title from an element when no explicit `data-tip` /
// `data-i18n-title` was set. Priority: existing title → aria-label →
// aria-labelledby target text → placeholder (inputs) → text content.
// Returns '' when nothing meaningful is derivable.
export function deriveAutoTitle(el, getById) {
    if (!el || el.nodeType !== 1) return '';
    const get = typeof getById === 'function' ? getById : (id => {
        if (typeof document === 'undefined') return null;
        return document.getElementById(id);
    });
    const existing = el.getAttribute && el.getAttribute('title');
    if (existing && existing.trim()) return normalizeTitle(existing);
    const aria = el.getAttribute && el.getAttribute('aria-label');
    if (aria && aria.trim()) return normalizeTitle(aria);
    const labelledby = el.getAttribute && el.getAttribute('aria-labelledby');
    if (labelledby) {
        for (const id of labelledby.split(/\s+/)) {
            const t = get(id);
            if (t && t.textContent && t.textContent.trim()) {
                return normalizeTitle(t.textContent);
            }
        }
    }
    const ph = el.getAttribute && el.getAttribute('placeholder');
    if (ph && ph.trim()) return normalizeTitle(ph);
    if (el.textContent && el.textContent.trim()) {
        return normalizeTitle(el.textContent);
    }
    return '';
}

// Predicate: should the auto-titler stamp this element?
//   - Must not already have a title
//   - Must not opt-out via `data-no-tip`
//   - Must not already declare a `data-tip` / `data-i18n-title`
//     (those go through the i18n-driven path)
//   - Must not already be marked auto-titled (idempotency)
export function shouldAutoTitle(el) {
    if (!el || el.nodeType !== 1) return false;
    const ds = el.dataset || {};
    if (ds.noTip === '1' || ds.noTip === 'true') return false;
    if (ds.autoTitle === '1') return false;
    if (ds.tooltipUpgraded === '1') return false;
    if (ds.tip || ds.tooltip || ds.tipKey) return false;
    if (el.hasAttribute && (el.hasAttribute('data-i18n-title') || el.hasAttribute('title'))) {
        return false;
    }
    return true;
}
