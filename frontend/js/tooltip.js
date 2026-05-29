// Tooltip DOM glue. Walks the document and upgrades `data-tip="key"`
// elements to `data-i18n-title="key"` so applyUiI18n() translates them
// to the native `title` attribute (free OS hover tooltip + screen
// reader support).
//
// Re-applies on locale change so titles stay in sync.

import { tipSelectors, tipKey, tipAttrsFor, shortcutId, composeTooltip } from './_tooltip.js';
import { applyUiI18n, t } from './i18n.js';
import { listShortcuts } from './shortcuts.js';
import { formatKey } from './_shortcuts.js';

export function installTooltips() {
    upgrade(document);
    augmentShortcutTitles(document);
    window.addEventListener('tv:i18n-changed', () => {
        upgrade(document);
        augmentShortcutTitles(document);
    });
}

// Upgrade all `data-tip*` elements under `root` to use data-i18n-title.
// Idempotent — already-upgraded elements get skipped.
export function upgradeTooltips(root) { upgrade(root || document); }

function upgrade(root) {
    if (!root || typeof root.querySelectorAll !== 'function') return 0;
    let n = 0;
    for (const sel of tipSelectors()) {
        root.querySelectorAll(sel).forEach(el => {
            const key = tipKey(el);
            if (!key) return;
            const attrs = tipAttrsFor(key);
            if (!attrs) return;
            // Skip if already done so we don't re-trigger applyUiI18n.
            if (el.dataset.tooltipUpgraded === '1') return;
            for (const [k, v] of Object.entries(attrs)) {
                if (!el.hasAttribute(k)) el.setAttribute(k, v);
            }
            el.dataset.tooltipUpgraded = '1';
            n++;
        });
    }
    if (n > 0) applyUiI18n(root);
    return n;
}

// Append "(⌘K)"-style shortcut chip to the title of every element that
// declares both data-tip (i18n key) and data-shortcut (shortcut id).
// Runs AFTER applyUiI18n so the translated tip is already in place.
function augmentShortcutTitles(root) {
    if (!root || typeof root.querySelectorAll !== 'function') return 0;
    const all = listShortcuts();
    const byId = new Map(all.map(sc => [sc.id, sc]));
    const isMac = typeof navigator !== 'undefined' && /Mac|iPhone|iPad/.test(navigator.platform);
    let n = 0;
    root.querySelectorAll('[data-shortcut]').forEach(el => {
        const id = shortcutId(el);
        if (!id) return;
        const sc = byId.get(id);
        if (!sc || !sc.keys || sc.keys.key == null) return;
        const chip = formatKey(sc, isMac);
        if (!chip) return;
        const key = tipKey(el);
        const tip = key ? t(key) : (el.getAttribute('title') || '');
        el.setAttribute('title', composeTooltip(tip, chip));
        n++;
    });
    return n;
}
