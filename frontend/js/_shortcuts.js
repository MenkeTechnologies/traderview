// Keyboard-shortcut registry — pure helpers shared with vitest.
//
// Shape per shortcut:
//   { id, keys: { key: 'k', meta: true, ctrl: false, shift: false, alt: false },
//     descKey, scope: 'global' | 'palette' | 'editor', actionKey }
//
// `actionKey` is a CustomEvent name dispatched on `window` when the
// shortcut fires (e.g. 'tv:open-palette'). The wiring layer
// (frontend/js/shortcuts.js) translates DOM keydown → registry lookup →
// `window.dispatchEvent(new CustomEvent(actionKey))`.

export const LS_KEY = 'tv-shortcuts-v1';
export const VERSION = 1;

export const DEFAULT_SHORTCUTS = [
    { id: 'command_palette', keys: { key: 'k', meta: true,  ctrl: true,  shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.command_palette', actionKey: 'tv:open-palette' },
    { id: 'help',            keys: { key: '?', meta: false, ctrl: false, shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.help',           actionKey: 'tv:open-help' },
    { id: 'escape',          keys: { key: 'Escape', meta: false, ctrl: false, shift: false, alt: false }, scope: 'global', descKey: 'shortcut.escape', actionKey: 'tv:escape' },
    { id: 'focus_search',    keys: { key: '/', meta: false, ctrl: false, shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.focus_search',   actionKey: 'tv:focus-search' },
    { id: 'reload',          keys: { key: 'r', meta: true,  ctrl: true,  shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.reload',         actionKey: 'tv:reload' },
    { id: 'toggle_favorite', keys: { key: 'd', meta: true,  ctrl: true,  shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.toggle_favorite', actionKey: 'tv:toggle-favorite' },
    { id: 'open_new_tab',    keys: { key: 'n', meta: true,  ctrl: true,  shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.open_new_tab',    actionKey: 'tv:open-new-tab' },
    { id: 'add_bookmark',    keys: { key: 'b', meta: true,  ctrl: true,  shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.add_bookmark',    actionKey: 'tv:add-bookmark' },
    { id: 'go_home',         keys: { key: 'h', meta: true,  ctrl: true,  shift: true,  alt: false }, scope: 'global',  descKey: 'shortcut.go_home',         actionKey: 'tv:go-home' },
    // No default keybind for clear_recents — it's destructive enough that a single keypress would be too easy. Surface it only via palette + ctx menu.
    { id: 'clear_recents',   keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.clear_recents',   actionKey: 'tv:clear-recents' },
    { id: 'toggle_theme',    keys: { key: 'l', meta: true,  ctrl: true,  shift: true,  alt: false }, scope: 'global',  descKey: 'shortcut.toggle_theme',    actionKey: 'tv:toggle-theme' },
    { id: 'toggle_crt',      keys: { key: 'c', meta: true,  ctrl: true,  shift: true,  alt: false }, scope: 'global',  descKey: 'shortcut.toggle_crt',      actionKey: 'tv:toggle-crt' },
    { id: 'toggle_neon',     keys: { key: 'g', meta: true,  ctrl: true,  shift: true,  alt: false }, scope: 'global',  descKey: 'shortcut.toggle_neon',     actionKey: 'tv:toggle-neon' },
    { id: 'cycle_locale',    keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.cycle_locale',    actionKey: 'tv:cycle-locale' },
    { id: 'open_settings',   keys: { key: ',',  meta: true,  ctrl: true,  shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.open_settings',   actionKey: 'tv:open-settings' },
    // Context-menu actions surfaced in palette only — no default binding.
    { id: 'nav_back',        keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.nav_back',        actionKey: 'tv:nav-back' },
    { id: 'copy_view_url',   keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.copy_view_url',   actionKey: 'tv:copy-view-url' },
    { id: 'copy_view_id',    keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.copy_view_id',    actionKey: 'tv:copy-view-id' },
    // Text-entry edit actions — palette finds them; activation falls
    // back to document.activeElement when no ctxmenu target was set.
    { id: 'edit_cut',        keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.edit_cut',        actionKey: 'tv:edit-cut' },
    { id: 'edit_copy',       keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.edit_copy',       actionKey: 'tv:edit-copy' },
    { id: 'edit_paste',      keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.edit_paste',      actionKey: 'tv:edit-paste' },
    { id: 'edit_select_all', keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.edit_select_all', actionKey: 'tv:edit-select-all' },
    { id: 'edit_undo',       keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.edit_undo',       actionKey: 'tv:edit-undo' },
    { id: 'edit_redo',       keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.edit_redo',       actionKey: 'tv:edit-redo' },
    // Symbol-aware quick-nav. No default keybinding — surfaced via the
    // command palette and right-click context menu only. Acts on the
    // current global symbol; toasts "no active symbol" if unset.
    { id: 'copy_symbol',              keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global', descKey: 'shortcut.copy_symbol',              actionKey: 'tv:copy-symbol' },
    { id: 'open_charts_for_symbol',   keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global', descKey: 'shortcut.open_charts_for_symbol',   actionKey: 'tv:open-charts-for-symbol' },
    { id: 'open_options_for_symbol',  keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global', descKey: 'shortcut.open_options_for_symbol',  actionKey: 'tv:open-options-for-symbol' },
    { id: 'open_research_for_symbol', keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global', descKey: 'shortcut.open_research_for_symbol', actionKey: 'tv:open-research-for-symbol' },
    { id: 'open_earnings_for_symbol', keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global', descKey: 'shortcut.open_earnings_for_symbol', actionKey: 'tv:open-earnings-for-symbol' },
    { id: 'open_news_for_symbol',     keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global', descKey: 'shortcut.open_news_for_symbol',     actionKey: 'tv:open-news-for-symbol' },
    // View-scoped: fire only when the dispatcher's setScope(view)
    // matches. The trades scope unlocks `n` for new-trade; the
    // dashboard scope unlocks `r` for refresh. Neither triggers in
    // text-entry context (handled by isTextEntryTarget + non-modifier).
    { id: 'trades_new',         keys: { key: 'n', meta: false, ctrl: false, shift: false, alt: false }, scope: 'trades',    descKey: 'shortcut.trades_new',         actionKey: 'tv:trades-new' },
    { id: 'dashboard_refresh',  keys: { key: 'r', meta: false, ctrl: false, shift: false, alt: false }, scope: 'dashboard', descKey: 'shortcut.dashboard_refresh',  actionKey: 'tv:dashboard-refresh' },
    { id: 'journal_focus_body', keys: { key: 'n', meta: false, ctrl: false, shift: false, alt: false }, scope: 'journal',   descKey: 'shortcut.journal_focus_body', actionKey: 'tv:journal-focus-body' },
    { id: 'watchlists_focus_add', keys: { key: 'n', meta: false, ctrl: false, shift: false, alt: false }, scope: 'watchlists',  descKey: 'shortcut.watchlists_focus_add',  actionKey: 'tv:watchlists-focus-add' },
    { id: 'alert_rules_focus_new', keys: { key: 'n', meta: false, ctrl: false, shift: false, alt: false }, scope: 'alert-rules', descKey: 'shortcut.alert_rules_focus_new', actionKey: 'tv:alert-rules-focus-new' },
    { id: 'rebalance_compute',       keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'rebalance',   descKey: 'shortcut.rebalance_compute',       actionKey: 'tv:rebalance-compute' },
    { id: 'rebalance_focus_targets', keys: { key: 't', meta: false, ctrl: false, shift: false, alt: false }, scope: 'rebalance',   descKey: 'shortcut.rebalance_focus_targets', actionKey: 'tv:rebalance-focus-targets' },
    { id: 'strategy_alerts_focus_name',   keys: { key: 'n', meta: false, ctrl: false, shift: false, alt: false }, scope: 'strategy-alerts', descKey: 'shortcut.strategy_alerts_focus_name',   actionKey: 'tv:strategy-alerts-focus-name' },
    { id: 'strategy_alerts_evaluate_now', keys: { key: 'e', meta: false, ctrl: false, shift: false, alt: false }, scope: 'strategy-alerts', descKey: 'shortcut.strategy_alerts_evaluate_now', actionKey: 'tv:strategy-alerts-evaluate-now' },
    { id: 'accounts_focus_name',          keys: { key: 'n', meta: false, ctrl: false, shift: false, alt: false }, scope: 'accounts',          descKey: 'shortcut.accounts_focus_name',          actionKey: 'tv:accounts-focus-name' },
    { id: 'accounts_overview_refresh',    keys: { key: 'r', meta: false, ctrl: false, shift: false, alt: false }, scope: 'accounts-overview', descKey: 'shortcut.accounts_overview_refresh',    actionKey: 'tv:accounts-overview-refresh' },
    { id: 'developer_focus_name',         keys: { key: 'n', meta: false, ctrl: false, shift: false, alt: false }, scope: 'developer',         descKey: 'shortcut.developer_focus_name',         actionKey: 'tv:developer-focus-name' },
    { id: 'developer_generate',           keys: { key: 'g', meta: false, ctrl: false, shift: false, alt: false }, scope: 'developer',         descKey: 'shortcut.developer_generate',           actionKey: 'tv:developer-generate' },
    { id: 'backtest_run',                 keys: { key: 'r', meta: false, ctrl: false, shift: false, alt: false }, scope: 'backtest',          descKey: 'shortcut.backtest_run',                 actionKey: 'tv:backtest-run' },
    { id: 'backtest_presets_focus_name',  keys: { key: 'n', meta: false, ctrl: false, shift: false, alt: false }, scope: 'backtest-presets',  descKey: 'shortcut.backtest_presets_focus_name',  actionKey: 'tv:backtest-presets-focus-name' },
    { id: 'csv_wizard_upload',            keys: { key: 'u', meta: false, ctrl: false, shift: false, alt: false }, scope: 'csv-wizard',        descKey: 'shortcut.csv_wizard_upload',            actionKey: 'tv:csv-wizard-upload' },
    { id: 'boards_focus_name',            keys: { key: 'n', meta: false, ctrl: false, shift: false, alt: false }, scope: 'boards',            descKey: 'shortcut.boards_focus_name',            actionKey: 'tv:boards-focus-name' },
    { id: 'live_refresh',         keys: { key: 'r', meta: false, ctrl: false, shift: false, alt: false }, scope: 'live',       descKey: 'shortcut.live_refresh',         actionKey: 'tv:live-refresh' },
    { id: 'trades_refresh',       keys: { key: 'r', meta: false, ctrl: false, shift: false, alt: false }, scope: 'trades',     descKey: 'shortcut.trades_refresh',       actionKey: 'tv:trades-refresh' },
    { id: 'journal_refresh',      keys: { key: 'r', meta: false, ctrl: false, shift: false, alt: false }, scope: 'journal',    descKey: 'shortcut.journal_refresh',      actionKey: 'tv:journal-refresh' },
    { id: 'watchlists_refresh',   keys: { key: 'r', meta: false, ctrl: false, shift: false, alt: false }, scope: 'watchlists', descKey: 'shortcut.watchlists_refresh',   actionKey: 'tv:watchlists-refresh' },
    { id: 'webull_refresh',       keys: { key: 'r', meta: false, ctrl: false, shift: false, alt: false }, scope: 'webull',     descKey: 'shortcut.webull_refresh',       actionKey: 'tv:webull-refresh' },
    { id: 'charts_refresh',       keys: { key: 'r', meta: false, ctrl: false, shift: false, alt: false }, scope: 'charts',     descKey: 'shortcut.charts_refresh',       actionKey: 'tv:charts-refresh' },
    // Quick-nav globals (Cmd/Ctrl + Option/Alt + letter): zero-conflict
    // with browser defaults; jumps directly to high-traffic views.
    { id: 'nav_trades',      keys: { key: 't', meta: true, ctrl: true, shift: false, alt: true }, scope: 'global', descKey: 'shortcut.nav_trades',      actionKey: 'tv:nav-trades' },
    { id: 'nav_journal',     keys: { key: 'j', meta: true, ctrl: true, shift: false, alt: true }, scope: 'global', descKey: 'shortcut.nav_journal',     actionKey: 'tv:nav-journal' },
    { id: 'nav_dashboard',   keys: { key: 'd', meta: true, ctrl: true, shift: false, alt: true }, scope: 'global', descKey: 'shortcut.nav_dashboard',   actionKey: 'tv:nav-dashboard' },
    { id: 'nav_watchlists',  keys: { key: 'w', meta: true, ctrl: true, shift: false, alt: true }, scope: 'global', descKey: 'shortcut.nav_watchlists',  actionKey: 'tv:nav-watchlists' },
    { id: 'nav_charts',      keys: { key: 'c', meta: true, ctrl: true, shift: false, alt: true }, scope: 'global', descKey: 'shortcut.nav_charts',      actionKey: 'tv:nav-charts' },
    { id: 'nav_live',        keys: { key: 'l', meta: true, ctrl: true, shift: false, alt: true }, scope: 'global', descKey: 'shortcut.nav_live',        actionKey: 'tv:nav-live' },
    { id: 'nav_reports',     keys: { key: 'r', meta: true, ctrl: true, shift: false, alt: true }, scope: 'global', descKey: 'shortcut.nav_reports',     actionKey: 'tv:nav-reports' },
    { id: 'nav_scanner',     keys: { key: 'm', meta: true, ctrl: true, shift: false, alt: true }, scope: 'global', descKey: 'shortcut.nav_scanner',     actionKey: 'tv:nav-scanner' },
];

// Whether a DOM-style keydown event satisfies a shortcut keys spec.
// `meta` OR `ctrl` matches both Mac and PC users — set both `true` in
// the keys spec to "fire if either Cmd or Ctrl is down". Set `meta: true,
// ctrl: false` to be strict Mac-only.
export function matches(e, sc) {
    if (!e || !sc || !sc.keys) return false;
    const k = sc.keys;
    if (k.key == null) return false;
    if (typeof e.key !== 'string') return false;
    if (e.key.toLowerCase() !== String(k.key).toLowerCase()) return false;
    // Modifier match policy: meta/ctrl are OR'd if both required keys are
    // true (cross-platform); otherwise strict equality.
    if (k.meta && k.ctrl) {
        if (!(e.metaKey || e.ctrlKey)) return false;
    } else {
        if ((k.meta  || false) !== !!e.metaKey) return false;
        if ((k.ctrl  || false) !== !!e.ctrlKey) return false;
    }
    if ((k.shift || false) !== !!e.shiftKey) return false;
    if ((k.alt   || false) !== !!e.altKey)   return false;
    return true;
}

// Human-readable key chip. macOS-style glyphs when available.
export function formatKey(sc, isMac = true) {
    if (!sc || !sc.keys) return '';
    const k = sc.keys;
    const parts = [];
    if (k.meta && k.ctrl) parts.push(isMac ? '⌘' : 'Ctrl');
    else {
        if (k.ctrl)  parts.push(isMac ? '⌃' : 'Ctrl');
        if (k.meta)  parts.push(isMac ? '⌘' : 'Win');
    }
    if (k.shift) parts.push(isMac ? '⇧' : 'Shift');
    if (k.alt)   parts.push(isMac ? '⌥' : 'Alt');
    parts.push(prettyKey(k.key));
    return parts.join(isMac ? '' : '+');
}

function prettyKey(k) {
    if (!k) return '';
    if (k === ' ' || k.toLowerCase() === 'space') return '␣';
    if (k.length === 1) return k.toUpperCase();
    return k;
}

// Find first registered shortcut whose keys match the event, scoped by
// `currentScope` ('global' shortcuts always match; others only match
// when current scope === sc.scope).
export function findMatch(event, shortcuts, currentScope = 'global') {
    if (!Array.isArray(shortcuts)) return null;
    for (const sc of shortcuts) {
        if (!sc.enabled && sc.enabled !== undefined) continue;
        if (sc.scope !== 'global' && sc.scope !== currentScope) continue;
        if (matches(event, sc)) return sc;
    }
    return null;
}

// localStorage-backed: load user overrides on top of defaults. Each
// override is keyed by shortcut id and replaces `keys` (not the whole
// entry — descKey + actionKey stay from defaults so user can't break
// the registry by deleting them).
export function loadShortcuts(getItem) {
    const get = getItem || ((typeof localStorage !== 'undefined') ? (k => localStorage.getItem(k)) : () => null);
    let saved = {};
    try {
        const raw = get(LS_KEY);
        if (raw) {
            const obj = JSON.parse(raw);
            if (obj && obj.version === VERSION && obj.overrides && typeof obj.overrides === 'object') {
                saved = obj.overrides;
            }
        }
    } catch { /* malformed → ignore */ }
    return DEFAULT_SHORTCUTS.map(sc =>
        saved[sc.id] ? { ...sc, keys: saved[sc.id] } : { ...sc });
}

export function saveOverrides(overrides, setItem) {
    const set = setItem || ((typeof localStorage !== 'undefined')
        ? ((k, v) => localStorage.setItem(k, v))
        : () => {});
    try { set(LS_KEY, JSON.stringify({ version: VERSION, overrides })); }
    catch { /* private mode */ }
}

// Should the shortcut fire even when the user is typing? Most shortcuts
// should NOT — leave room for the user to type 'k' in a textbox. But a
// few (Escape, Cmd+K, Cmd+/) DO want to fire from inside text fields
// because they are how you EXIT the field.
export function firesInEditableContext(sc) {
    if (!sc) return false;
    if (sc.id === 'escape') return true;
    // Cmd+K-like (meta && ctrl-tolerant) always fires.
    return !!(sc.keys && sc.keys.meta && sc.keys.ctrl);
}

// True if the event target is a text-entry element (input/textarea/select/
// contentEditable). Used together with `firesInEditableContext` to gate
// non-modifier shortcuts.
export function isTextEntryTarget(t) {
    if (!t) return false;
    const tag = (t.tagName || '').toLowerCase();
    if (tag === 'input' || tag === 'textarea' || tag === 'select') return true;
    if (t.isContentEditable) return true;
    return false;
}
