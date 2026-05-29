// Tooltip DOM glue. Walks the document and upgrades `data-tip="key"`
// elements to `data-i18n-title="key"` so applyUiI18n() translates them
// to the native `title` attribute (free OS hover tooltip + screen
// reader support).
//
// Re-applies on locale change so titles stay in sync.

import {
    tipSelectors, tipKey, tipAttrsFor, shortcutId, composeTooltip,
    interactiveSelectors, deriveAutoTitle, shouldAutoTitle, resetAutoTitled,
    inferI18nKey, shouldStampCommonI18n,
} from './_tooltip.js';
import { applyUiI18n, t } from './i18n.js';
import { listShortcuts } from './shortcuts.js';
import { formatKey } from './_shortcuts.js';

export function installTooltips() {
    upgrade(document);
    autoApply(document);
    augmentShortcutTitles(document);
    window.addEventListener('tv:i18n-changed', () => {
        // Translated DOM text just landed via applyUiI18n — wipe our
        // cached titles so the next pass picks up the new locale.
        resetAutoTitled(document);
        upgrade(document);
        autoApply(document);
        augmentShortcutTitles(document);
    });
    installMutationObserver();
}

// Watches the whole document for added subtrees and runs upgrade +
// autoApply on each. Covers views that re-render parts of themselves
// after the initial dispatch (filter toggles, sort handlers, modal
// opens, palette overlay, ctx menu) so every dynamically inserted
// interactive element gets a title automatically.
//
// Pending nodes are coalesced into one pass per microtask to avoid
// O(n²) churn when a render produces hundreds of new nodes.
let _observer = null;
const _pending = new Set();
let _flushScheduled = false;

function installMutationObserver() {
    if (_observer || typeof MutationObserver === 'undefined' || typeof document === 'undefined') return;
    _observer = new MutationObserver(records => {
        for (const r of records) {
            for (const node of r.addedNodes) {
                if (node && node.nodeType === 1) _pending.add(node);
            }
        }
        scheduleFlush();
    });
    const target = document.body || document.documentElement;
    if (target) _observer.observe(target, { childList: true, subtree: true });
}

function scheduleFlush() {
    if (_flushScheduled) return;
    _flushScheduled = true;
    const run = () => { _flushScheduled = false; flushPending(); };
    if (typeof queueMicrotask === 'function') queueMicrotask(run);
    else Promise.resolve().then(run);
}

function flushPending() {
    if (_pending.size === 0) return;
    const nodes = Array.from(_pending);
    _pending.clear();
    for (const node of nodes) {
        try {
            upgrade(node);
            autoApplyNode(node);
        } catch { /* defensive: never let a mutation pass crash */ }
    }
}

// Same as autoApply, but also stamps the root node itself if it
// matches the interactive selector — querySelectorAll only walks
// descendants, so a freshly inserted `<button>` would be missed.
function autoApplyNode(node) {
    if (!node || node.nodeType !== 1) return 0;
    const sel = interactiveSelectors();
    let n = 0;
    if (typeof node.matches === 'function' && node.matches(sel) && shouldAutoTitle(node)) {
        const title = deriveAutoTitle(node);
        if (title) {
            node.setAttribute('title', title);
            node.dataset.autoTitle = '1';
            n++;
        }
    }
    n += autoApply(node);
    return n;
}

// Upgrade all `data-tip*` elements under `root` to use data-i18n-title.
// Idempotent — already-upgraded elements get skipped.
export function upgradeTooltips(root) { upgrade(root || document); }

// Walk every interactive element under `root` and stamp a `title`
// attribute derived from its semantics when one isn't already set.
// Ensures every clickable thing has at least hover discoverability.
export function autoApplyTooltips(root) { autoApply(root || document); }

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

function autoApply(root) {
    if (!root || typeof root.querySelectorAll !== 'function') return 0;
    let n = 0;
    root.querySelectorAll(interactiveSelectors()).forEach(el => {
        // 1) Common-verb i18n stamp: buttons with high-frequency labels
        //    like "Save" / "Cancel" get a data-i18n attr so applyUiI18n
        //    re-translates them on locale change.
        stampCommonI18n(el);
        // 2) Auto-title: derive a title from semantics if none set.
        if (!shouldAutoTitle(el)) return;
        const title = deriveAutoTitle(el);
        if (!title) return;
        el.setAttribute('title', title);
        el.dataset.autoTitle = '1';
        n++;
    });
    return n;
}

function stampCommonI18n(el) {
    if (!shouldStampCommonI18n(el)) return false;
    const key = inferI18nKey(el.textContent);
    if (!key) return false;
    el.setAttribute('data-i18n', key);
    return true;
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
