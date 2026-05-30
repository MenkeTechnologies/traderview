// Right-click context-menu helpers (pure, no DOM).
//
// Menu items shape:
//   { id, labelKey, kind?: 'item' | 'separator', shortcut?, scope?, hidden?,
//     actionKey?, navTo?, onClick? }
//
// `actionKey`: a CustomEvent name to dispatch on `window` when the
// item is selected (decouples menu wiring from the action implementation).
// `navTo`: shortcut to set `window.location.hash`.
// `onClick`: in-process callback (used for synchronous things like
// document.execCommand('copy')).

// Built-in global menu items every right-click receives. Per-view code
// can register additional items via the DOM-glue layer (context_menu.js).
export const GLOBAL_ITEMS = [
    { id: 'open_palette', labelKey: 'ctxmenu.open_command_palette',
      actionKey: 'tv:open-palette', section: 'navigate' },
    { id: 'go_home',      labelKey: 'ctxmenu.go_home',
      navTo: 'launcher', section: 'navigate' },
    { id: 'go_back',      labelKey: 'ctxmenu.go_back',
      actionKey: 'tv:nav-back', section: 'navigate' },
    { id: 'reload',       labelKey: 'ctxmenu.reload',
      actionKey: 'tv:reload', section: 'navigate' },
    { kind: 'separator' },
    { id: 'toggle_favorite', labelKey: 'ctxmenu.toggle_favorite',
      actionKey: 'tv:toggle-favorite', section: 'view' },
    { id: 'open_new_tab', labelKey: 'ctxmenu.open_new_tab',
      actionKey: 'tv:open-new-tab', section: 'view' },
    { id: 'copy_view_url', labelKey: 'ctxmenu.copy_view_url',
      actionKey: 'tv:copy-view-url', section: 'clipboard' },
    { id: 'copy_view_id', labelKey: 'ctxmenu.copy_view_id',
      actionKey: 'tv:copy-view-id', section: 'clipboard' },
    { id: 'add_bookmark', labelKey: 'ctxmenu.add_bookmark',
      actionKey: 'tv:add-bookmark', section: 'view' },
    { id: 'manage_favorites', labelKey: 'ctxmenu.manage_favorites',
      navTo: 'favorites', section: 'view' },
    { kind: 'separator' },
    { id: 'toggle_theme', labelKey: 'ctxmenu.toggle_theme',
      actionKey: 'tv:toggle-theme', section: 'appearance' },
    { id: 'toggle_crt',   labelKey: 'ctxmenu.toggle_crt',
      actionKey: 'tv:toggle-crt',   section: 'appearance' },
    { id: 'toggle_neon',  labelKey: 'ctxmenu.toggle_neon',
      actionKey: 'tv:toggle-neon',  section: 'appearance' },
    { kind: 'separator' },
    { id: 'shortcuts',    labelKey: 'ctxmenu.shortcuts',
      navTo: 'keyboard-shortcuts', section: 'help' },
];

// Compute the position the menu should render at, given the trigger
// event's clientX/Y + the menu's measured size + the viewport size.
// Flips to the other side / top when the menu would overflow.
export function positionMenu(eventX, eventY, menuW, menuH, viewportW, viewportH, margin = 8) {
    let x = eventX;
    let y = eventY;
    if (x + menuW + margin > viewportW) x = Math.max(margin, viewportW - menuW - margin);
    if (y + menuH + margin > viewportH) y = Math.max(margin, viewportH - menuH - margin);
    if (x < margin) x = margin;
    if (y < margin) y = margin;
    return { x, y };
}

// Filter + section + de-dup logic. Hidden items dropped; separators
// collapsed when adjacent or at edges; per-section ordering preserved.
export function compileMenu(items) {
    if (!Array.isArray(items)) return [];
    const filtered = items.filter(it => it && !it.hidden);
    // Collapse leading/trailing/duplicate separators.
    const out = [];
    let prevSep = true; // disallow leading separator
    for (const it of filtered) {
        if (it.kind === 'separator') {
            if (prevSep) continue;
            out.push(it);
            prevSep = true;
        } else {
            out.push(it);
            prevSep = false;
        }
    }
    while (out.length > 0 && out[out.length - 1].kind === 'separator') out.pop();
    return out;
}

// Allow callers to merge per-view custom items on top of the globals.
// `customItems` get inserted at the top under their own separator block.
export function mergeMenu(globalItems, customItems) {
    if (!Array.isArray(customItems) || customItems.length === 0) {
        return [...globalItems];
    }
    return [...customItems, { kind: 'separator' }, ...globalItems];
}

// Symbol-aware quick-nav items. Shown inside any view whose scope is
// listed in SYMBOL_AWARE_SCOPES — typical right-click on a chart view
// gets "Copy SYMBOL", "Charts for SYMBOL", etc. as the top block.
// The label gets interpolated with the current global symbol at render
// time; if no global symbol is set the items are hidden.
export const SYMBOL_ITEMS = [
    { id: 'copy_symbol',             labelKey: 'ctxmenu.copy_symbol',
      actionKey: 'tv:copy-symbol',             section: 'symbol' },
    { id: 'open_charts_for_symbol',  labelKey: 'ctxmenu.open_charts_for_symbol',
      actionKey: 'tv:open-charts-for-symbol',  section: 'symbol' },
    { id: 'open_options_for_symbol', labelKey: 'ctxmenu.open_options_for_symbol',
      actionKey: 'tv:open-options-for-symbol', section: 'symbol' },
    { id: 'open_research_for_symbol', labelKey: 'ctxmenu.open_research_for_symbol',
      actionKey: 'tv:open-research-for-symbol', section: 'symbol' },
    { id: 'open_earnings_for_symbol', labelKey: 'ctxmenu.open_earnings_for_symbol',
      actionKey: 'tv:open-earnings-for-symbol', section: 'symbol' },
    { id: 'open_news_for_symbol',     labelKey: 'ctxmenu.open_news_for_symbol',
      actionKey: 'tv:open-news-for-symbol',     section: 'symbol' },
];

// Watchlist symbol-row context items. Right-click on a <tr data-
// context-scope="watchlist-symbol-row" data-symbol="X" data-wid="Y">
// shows: Set as active / Charts / Research / Options / Remove.
// Handlers read `data-symbol` and `data-wid` from detail.target.
export const WATCHLIST_ROW_ITEMS = [
    { id: 'wl_row_set_active', labelKey: 'ctxmenu.wl_row_set_active',
      actionKey: 'tv:wl-row-set-active', section: 'watchlist' },
    { id: 'wl_row_charts',     labelKey: 'ctxmenu.wl_row_charts',
      actionKey: 'tv:wl-row-charts',     section: 'watchlist' },
    { id: 'wl_row_research',   labelKey: 'ctxmenu.wl_row_research',
      actionKey: 'tv:wl-row-research',   section: 'watchlist' },
    { id: 'wl_row_options',    labelKey: 'ctxmenu.wl_row_options',
      actionKey: 'tv:wl-row-options',    section: 'watchlist' },
    { id: 'wl_row_remove',     labelKey: 'ctxmenu.wl_row_remove',
      actionKey: 'tv:wl-row-remove',     section: 'watchlist' },
];

// Position-row context items. Right-click on a <tr data-context-scope=
// "position-row" data-symbol="X" data-id="Y"> shows: View trade /
// Set active / Charts / Research / Options. Handlers read data-symbol
// or data-id from `CustomEvent.detail.target`.
export const POSITION_ROW_ITEMS = [
    { id: 'pos_row_view_trade', labelKey: 'ctxmenu.pos_row_view_trade',
      actionKey: 'tv:pos-row-view-trade', section: 'position' },
    { id: 'pos_row_set_active', labelKey: 'ctxmenu.pos_row_set_active',
      actionKey: 'tv:pos-row-set-active', section: 'position' },
    { id: 'pos_row_charts',     labelKey: 'ctxmenu.pos_row_charts',
      actionKey: 'tv:pos-row-charts',     section: 'position' },
    { id: 'pos_row_research',   labelKey: 'ctxmenu.pos_row_research',
      actionKey: 'tv:pos-row-research',   section: 'position' },
    { id: 'pos_row_options',    labelKey: 'ctxmenu.pos_row_options',
      actionKey: 'tv:pos-row-options',    section: 'position' },
];

// Alert-rule-row context items. Right-click on a <div data-context-
// scope="alert-rule-row" data-rule-id="X"> shows: Toggle enabled /
// Duplicate / Delete. Handlers read data-rule-id from detail.target
// and mutate the engine state in localStorage.
export const ALERT_RULE_ROW_ITEMS = [
    { id: 'ar_row_toggle',    labelKey: 'ctxmenu.ar_row_toggle',
      actionKey: 'tv:ar-row-toggle',    section: 'alert' },
    { id: 'ar_row_duplicate', labelKey: 'ctxmenu.ar_row_duplicate',
      actionKey: 'tv:ar-row-duplicate', section: 'alert' },
    { id: 'ar_row_delete',    labelKey: 'ctxmenu.ar_row_delete',
      actionKey: 'tv:ar-row-delete',    section: 'alert' },
];

// Dashboard-sidebar item context items. Right-click on a <li data-
// context-scope="dashboard-sidebar-item" data-id="X" data-name="Y">
// shows: Pick / Rename… / Duplicate / Delete… . Handlers reach into
// the dashboards-storage module directly.
export const DASHBOARD_SIDEBAR_ITEMS = [
    { id: 'db_side_pick',      labelKey: 'ctxmenu.db_side_pick',
      actionKey: 'tv:db-side-pick',      section: 'dashboard' },
    { id: 'db_side_rename',    labelKey: 'ctxmenu.db_side_rename',
      actionKey: 'tv:db-side-rename',    section: 'dashboard' },
    { id: 'db_side_duplicate', labelKey: 'ctxmenu.db_side_duplicate',
      actionKey: 'tv:db-side-duplicate', section: 'dashboard' },
    { id: 'db_side_delete',    labelKey: 'ctxmenu.db_side_delete',
      actionKey: 'tv:db-side-delete',    section: 'dashboard' },
];

// Board-row context items. Right-click on a <tr data-context-scope=
// "board-row" data-id="X" data-name="Y"> shows: Open board / Copy
// ID / Delete board.
export const BOARD_ROW_ITEMS = [
    { id: 'board_row_open',    labelKey: 'ctxmenu.board_row_open',
      actionKey: 'tv:board-row-open',    section: 'board' },
    { id: 'board_row_copy_id', labelKey: 'ctxmenu.board_row_copy_id',
      actionKey: 'tv:board-row-copy-id', section: 'board' },
    { id: 'board_row_delete',  labelKey: 'ctxmenu.board_row_delete',
      actionKey: 'tv:board-row-delete',  section: 'board' },
];

// Backtest-preset-row context items. Right-click on a <tr data-
// context-scope="backtest-preset-row" data-id="X" data-slug="Y"
// data-name="Z" data-mine="bool"> shows: Copy slug / Open preset /
// Fork (toasts if mine) / Delete (toasts if not mine).
export const BACKTEST_PRESET_ROW_ITEMS = [
    { id: 'bp_row_copy_slug', labelKey: 'ctxmenu.bp_row_copy_slug',
      actionKey: 'tv:bp-row-copy-slug', section: 'preset' },
    { id: 'bp_row_open',      labelKey: 'ctxmenu.bp_row_open',
      actionKey: 'tv:bp-row-open',      section: 'preset' },
    { id: 'bp_row_fork',      labelKey: 'ctxmenu.bp_row_fork',
      actionKey: 'tv:bp-row-fork',      section: 'preset' },
    { id: 'bp_row_delete',    labelKey: 'ctxmenu.bp_row_delete',
      actionKey: 'tv:bp-row-delete',    section: 'preset' },
];

// Share-row context items. Right-click on a <tr data-context-scope=
// "share-row" data-id="X" data-slug="Y" data-mine="bool"> shows:
// Copy share URL / Open shared trade / Delete. Delete toasts a
// warning when data-mine="false" (public shares aren't yours).
export const SHARE_ROW_ITEMS = [
    { id: 'share_row_copy_url', labelKey: 'ctxmenu.share_row_copy_url',
      actionKey: 'tv:share-row-copy-url', section: 'share' },
    { id: 'share_row_open',     labelKey: 'ctxmenu.share_row_open',
      actionKey: 'tv:share-row-open',     section: 'share' },
    { id: 'share_row_delete',   labelKey: 'ctxmenu.share_row_delete',
      actionKey: 'tv:share-row-delete',   section: 'share' },
];

// Plan-row context items. Right-click on a <tr data-context-scope=
// "plan-row" data-id="X" data-symbol="Y"> shows: Copy symbol /
// Abandon plan. Abandon uses tConfirm + api.abandonPlan.
export const PLAN_ROW_ITEMS = [
    { id: 'plan_row_copy_symbol', labelKey: 'ctxmenu.plan_row_copy_symbol',
      actionKey: 'tv:plan-row-copy-symbol', section: 'plan' },
    { id: 'plan_row_abandon',     labelKey: 'ctxmenu.plan_row_abandon',
      actionKey: 'tv:plan-row-abandon',     section: 'plan' },
];

// Account-row context items. Right-click on a <tr data-context-scope=
// "account-row" data-id="X" data-name="Y"> shows: Copy ID / Delete.
export const ACCOUNT_ROW_ITEMS = [
    { id: 'acct_row_copy_id', labelKey: 'ctxmenu.acct_row_copy_id',
      actionKey: 'tv:acct-row-copy-id', section: 'account' },
    { id: 'acct_row_delete',  labelKey: 'ctxmenu.acct_row_delete',
      actionKey: 'tv:acct-row-delete',  section: 'account' },
];

// Custom-indicator-row context items. Right-click on a <tr data-
// context-scope="custom-indicator-row" data-id="X" data-name="Y"
// data-definition="<json>"> shows: Copy JSON definition / Delete.
export const CUSTOM_INDICATOR_ROW_ITEMS = [
    { id: 'ci_row_copy_def', labelKey: 'ctxmenu.ci_row_copy_def',
      actionKey: 'tv:ci-row-copy-def', section: 'indicator' },
    { id: 'ci_row_delete',   labelKey: 'ctxmenu.ci_row_delete',
      actionKey: 'tv:ci-row-delete',   section: 'indicator' },
];

// Hotkey-row context items. Right-click on a <tr data-context-scope=
// "hotkey-row" data-id="X" data-combo="ctrl+shift+J"> shows: Copy
// combo / Delete. Delete uses tConfirm to replace the silent inline
// `delete` button.
export const HOTKEY_ROW_ITEMS = [
    { id: 'hk_row_copy_combo', labelKey: 'ctxmenu.hk_row_copy_combo',
      actionKey: 'tv:hk-row-copy-combo', section: 'hotkey' },
    { id: 'hk_row_delete',     labelKey: 'ctxmenu.hk_row_delete',
      actionKey: 'tv:hk-row-delete',     section: 'hotkey' },
];

// Journal-entry context items. Right-click on a <div data-context-
// scope="journal-entry" data-id="X" data-trade-id="Y"> shows:
// View linked trade (if trade-id set) / Delete entry. The view-trade
// handler toasts when no trade is linked; delete now goes through
// tConfirm (replaces the silent inline `delete` button behavior).
export const JOURNAL_ENTRY_ITEMS = [
    { id: 'je_view_trade', labelKey: 'ctxmenu.je_view_trade',
      actionKey: 'tv:je-view-trade', section: 'journal' },
    { id: 'je_delete',     labelKey: 'ctxmenu.je_delete',
      actionKey: 'tv:je-delete',     section: 'journal' },
];

// API-token-row context items. Right-click on a <tr data-context-
// scope="api-token-row" data-id="X" data-prefix="..." data-revoked=
// "bool"> shows: Copy prefix / Revoke. Revoke handler no-ops with a
// toast when already revoked.
export const API_TOKEN_ROW_ITEMS = [
    { id: 'tok_row_copy_prefix', labelKey: 'ctxmenu.tok_row_copy_prefix',
      actionKey: 'tv:tok-row-copy-prefix', section: 'token' },
    { id: 'tok_row_revoke',      labelKey: 'ctxmenu.tok_row_revoke',
      actionKey: 'tv:tok-row-revoke',      section: 'token' },
];

// Tag-chip context items. Right-click on a <span data-context-scope=
// "tag-chip" data-id="X" data-name="..."> shows: Copy name / Delete.
export const TAG_CHIP_ITEMS = [
    { id: 'tag_chip_copy',   labelKey: 'ctxmenu.tag_chip_copy',
      actionKey: 'tv:tag-chip-copy',   section: 'tag' },
    { id: 'tag_chip_delete', labelKey: 'ctxmenu.tag_chip_delete',
      actionKey: 'tv:tag-chip-delete', section: 'tag' },
];

// Webhook-row context items. Right-click on a <tr data-context-
// scope="webhook-row" data-id="X" data-enabled="bool"> shows:
// Test / Toggle enabled / Delete.
export const WEBHOOK_ROW_ITEMS = [
    { id: 'wh_row_test',   labelKey: 'ctxmenu.wh_row_test',
      actionKey: 'tv:wh-row-test',   section: 'webhook' },
    { id: 'wh_row_toggle', labelKey: 'ctxmenu.wh_row_toggle',
      actionKey: 'tv:wh-row-toggle', section: 'webhook' },
    { id: 'wh_row_delete', labelKey: 'ctxmenu.wh_row_delete',
      actionKey: 'tv:wh-row-delete', section: 'webhook' },
];

// Strategy-alert-row context items. Right-click on a <tr data-
// context-scope="strategy-alert-row" data-id="X"> shows: Toggle
// enabled / Delete. Handlers call api.update/deleteStrategyAlert.
export const STRATEGY_ALERT_ROW_ITEMS = [
    { id: 'sa_row_toggle', labelKey: 'ctxmenu.sa_row_toggle',
      actionKey: 'tv:sa-row-toggle', section: 'alert' },
    { id: 'sa_row_delete', labelKey: 'ctxmenu.sa_row_delete',
      actionKey: 'tv:sa-row-delete', section: 'alert' },
];

// Trade-row context items. Right-click on a <tr data-context-scope=
// "trade-row" data-id="X"> shows: View detail / Copy ID / Delete.
// Handlers read `data-id` from `CustomEvent.detail.target`.
export const TRADE_ROW_ITEMS = [
    { id: 'trade_view_detail', labelKey: 'ctxmenu.trade_view_detail',
      actionKey: 'tv:trade-view-detail', section: 'trade' },
    { id: 'trade_copy_id',     labelKey: 'ctxmenu.trade_copy_id',
      actionKey: 'tv:trade-copy-id',     section: 'trade' },
    { id: 'trade_delete',      labelKey: 'ctxmenu.trade_delete',
      actionKey: 'tv:trade-delete',      section: 'trade' },
];

// View IDs (URL slugs) where SYMBOL_ITEMS are auto-registered. First
// group is views that take `sym()` directly in the app.js dispatcher.
// Second group is symbol-centric views that read the global symbol
// internally (news/dashboard/etc) — same UX value from the menu.
export const SYMBOL_AWARE_SCOPES = [
    // direct sym() consumers
    'research', 'replay', 'earnings-iv', 'sentiment', 'options',
    'short-interest', 'darkpool', 'tape-replay', 'charts',
    // global-symbol consumers
    'dashboard', 'news', 'top-signals', 'premarket', 'webull',
    'vol-smile', 'vol-surface', 'option-payoff', 'compare',
    'live', 'pair-trade-calc',
];

// Native edit actions surfaced when the right-click target is a text-
// entry element (input/textarea/contentEditable). These get prepended
// above the standard global menu so the user can still paste, copy,
// select-all etc. — actions a custom menu would otherwise hide.
export const EDITING_ITEMS = [
    { id: 'edit_undo',       labelKey: 'ctxmenu.undo',
      actionKey: 'tv:edit-undo',       section: 'edit' },
    { id: 'edit_redo',       labelKey: 'ctxmenu.redo',
      actionKey: 'tv:edit-redo',       section: 'edit' },
    { kind: 'separator' },
    { id: 'edit_cut',        labelKey: 'ctxmenu.cut',
      actionKey: 'tv:edit-cut',        section: 'edit' },
    { id: 'edit_copy',       labelKey: 'ctxmenu.copy',
      actionKey: 'tv:edit-copy',       section: 'edit' },
    { id: 'edit_paste',      labelKey: 'ctxmenu.paste',
      actionKey: 'tv:edit-paste',      section: 'edit' },
    { id: 'edit_select_all', labelKey: 'ctxmenu.select_all',
      actionKey: 'tv:edit-select-all', section: 'edit' },
];

// Aggregated registry of every scope-specific item set. Each entry is
// `[scope, items]` so callers can iterate registrations or audit-tests
// can validate the whole catalog without naming every constant. Order
// matches the audit's iteration order; add new sets at the bottom.
export const ALL_SCOPED_ITEMS = [
    ['watchlist-symbol-row',   WATCHLIST_ROW_ITEMS],
    ['position-row',           POSITION_ROW_ITEMS],
    ['alert-rule-row',         ALERT_RULE_ROW_ITEMS],
    ['hotkey-row',             HOTKEY_ROW_ITEMS],
    ['custom-indicator-row',   CUSTOM_INDICATOR_ROW_ITEMS],
    ['account-row',            ACCOUNT_ROW_ITEMS],
    ['plan-row',               PLAN_ROW_ITEMS],
    ['share-row',              SHARE_ROW_ITEMS],
    ['backtest-preset-row',    BACKTEST_PRESET_ROW_ITEMS],
    ['board-row',              BOARD_ROW_ITEMS],
    ['dashboard-sidebar-item', DASHBOARD_SIDEBAR_ITEMS],
    ['journal-entry',          JOURNAL_ENTRY_ITEMS],
    ['api-token-row',          API_TOKEN_ROW_ITEMS],
    ['tag-chip',               TAG_CHIP_ITEMS],
    ['webhook-row',            WEBHOOK_ROW_ITEMS],
    ['strategy-alert-row',     STRATEGY_ALERT_ROW_ITEMS],
    ['trade-row',              TRADE_ROW_ITEMS],
];

// Same as `mergeMenu`, but additionally prepends `editingItems` (followed
// by a separator) above the custom + global blocks. Used for right-clicks
// on text-entry targets.
export function mergeMenuWithEditing(globalItems, customItems, editingItems) {
    const merged = mergeMenu(globalItems, customItems);
    if (!Array.isArray(editingItems) || editingItems.length === 0) return merged;
    return [...editingItems, { kind: 'separator' }, ...merged];
}

// For the up/down keyboard navigation through visible items.
export function nextVisibleIdx(items, currentIdx, delta) {
    if (!Array.isArray(items) || items.length === 0) return 0;
    const visibleIdxs = [];
    for (let i = 0; i < items.length; i++) {
        if (items[i].kind !== 'separator') visibleIdxs.push(i);
    }
    if (visibleIdxs.length === 0) return 0;
    const curPos = visibleIdxs.indexOf(currentIdx);
    let nextPos = curPos + delta;
    if (nextPos < 0) nextPos = visibleIdxs.length - 1;
    if (nextPos >= visibleIdxs.length) nextPos = 0;
    return visibleIdxs[nextPos];
}

// Find which item a keystroke should activate when used as a "mnemonic"
// (first character of the labelKey's last segment). Returns the item or null.
export function findMnemonic(items, char) {
    if (!char || char.length !== 1) return null;
    const c = char.toLowerCase();
    for (const it of items) {
        if (it.kind === 'separator') continue;
        const last = (it.labelKey || '').split('.').pop() || '';
        if (last[0] && last[0].toLowerCase() === c) return it;
    }
    return null;
}
