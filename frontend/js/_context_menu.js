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

// View IDs (URL slugs) where SYMBOL_ITEMS are auto-registered. Matches
// the views that take `sym()` in the app.js dispatcher.
export const SYMBOL_AWARE_SCOPES = [
    'research', 'replay', 'earnings-iv', 'sentiment', 'options',
    'short-interest', 'darkpool', 'tape-replay', 'charts',
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
