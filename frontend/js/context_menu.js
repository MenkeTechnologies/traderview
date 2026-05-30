// Right-click context-menu DOM glue.
//
// API:
//   installContextMenu()                 — listen for `contextmenu` events globally
//   registerContextItems(scope, items)   — view registers extra items shown only inside `scope`
//   tv:context-menu CustomEvent          — external code can request the menu programmatically
//
// The menu reads `data-context-scope` on the nearest ancestor; only
// scopes matching a registered set get their custom items merged in.

import {
    GLOBAL_ITEMS, EDITING_ITEMS, positionMenu, compileMenu, mergeMenu,
    mergeMenuWithEditing, nextVisibleIdx,
} from './_context_menu.js';
import { t, applyUiI18n } from './i18n.js';
import { esc } from './util.js';
import { showToast } from './toast.js';
import { loadState, saveState, toggleFavorite, isFavorite, addBookmark } from './_favorites_storage.js';
import { getGlobalSymbol, setGlobalSymbol } from './_global_symbol.js';
import { api } from './api.js';
import { tConfirm } from './dialog.js';
import * as alertEngine from './_alert_rules.js';

let _installed = false;
let _open = false;
let _items = [];
let _selected = -1;
let _editingTarget = null;          // the input/textarea/CE the menu was opened on
let _scopeTarget = null;            // the element carrying the matched data-context-scope
const _customByScope = new Map();   // scope-string → items[]

export function installContextMenu() {
    if (_installed) return;
    _installed = true;
    ensureMount();
    document.addEventListener('contextmenu', onContextMenu, { capture: true });
    document.addEventListener('click', onDocClick, { capture: true });
    document.addEventListener('keydown', onKey, { capture: true });
    window.addEventListener('tv:context-menu', (e) => {
        const d = e && e.detail || {};
        _editingTarget = isTextEntry(d.editingTarget) ? d.editingTarget : null;
        openAt(d.x || 0, d.y || 0, d.scope || null, !!_editingTarget);
    });
    window.addEventListener('tv:copy-view-url', () => {
        const url = window.location.href;
        if (navigator.clipboard && navigator.clipboard.writeText) {
            void navigator.clipboard.writeText(url).then(
                () => showToast(t('toast.copied', { what: t('toast.what.url') }), { level: 'success' }),
                () => showToast(t('toast.error.api', { err: t('toast.err.clipboard_denied') }), { level: 'error' }),
            );
        }
    });
    window.addEventListener('tv:nav-back', () => {
        if (typeof window.history?.back === 'function') window.history.back();
    });
    window.addEventListener('tv:reload', () => {
        // Force re-dispatch of the current view.
        window.dispatchEvent(new HashChangeEvent('hashchange'));
    });
    window.addEventListener('tv:open-new-tab', () => {
        window.open(window.location.href, '_blank', 'noopener,noreferrer');
    });
    window.addEventListener('tv:copy-view-id', () => {
        const vid = currentViewId();
        if (!vid) {
            showToast(t('toast.error.api', { err: t('toast.err.no_view') }), { level: 'error' });
            return;
        }
        if (navigator.clipboard && navigator.clipboard.writeText) {
            void navigator.clipboard.writeText(vid).then(
                () => showToast(t('toast.copied', { what: vid }), { level: 'success' }),
                () => showToast(t('toast.error.api', { err: t('toast.err.clipboard_denied') }), { level: 'error' }),
            );
        }
    });
    window.addEventListener('tv:add-bookmark', () => {
        const vid = currentViewId();
        if (!vid) {
            showToast(t('toast.error.api', { err: t('toast.err.no_view') }), { level: 'error' });
            return;
        }
        const name = (typeof window.prompt === 'function')
            ? window.prompt(t('prompt.bookmark_name', { view: vid }), vid)
            : vid;
        if (name == null) return;     // user cancelled
        const trimmed = String(name).trim();
        if (!trimmed) return;
        const state = loadState();
        const next = addBookmark(state, trimmed, vid);
        saveState(next);
        showToast(t('toast.bookmark_added', { name: trimmed }), { level: 'success' });
        window.dispatchEvent(new CustomEvent('tv:favorites-changed'));
    });
    window.addEventListener('tv:edit-cut',        () => execEdit('cut'));
    window.addEventListener('tv:edit-copy',       () => execEdit('copy'));
    window.addEventListener('tv:edit-paste',      () => void execPaste());
    window.addEventListener('tv:edit-select-all', () => execEdit('selectAll'));
    window.addEventListener('tv:edit-undo',       () => execEdit('undo'));
    window.addEventListener('tv:edit-redo',       () => execEdit('redo'));
    window.addEventListener('tv:copy-symbol', () => {
        const sym = (getGlobalSymbol() || '').toUpperCase();
        if (!sym) {
            showToast(t('toast.error.api', { err: t('toast.err.no_symbol') }), { level: 'error' });
            return;
        }
        if (navigator.clipboard && navigator.clipboard.writeText) {
            void navigator.clipboard.writeText(sym).then(
                () => showToast(t('toast.copied', { what: sym }), { level: 'success' }),
                () => showToast(t('toast.error.api', { err: t('toast.err.clipboard_denied') }), { level: 'error' }),
            );
        }
    });
    const navForSymbol = (viewId) => () => {
        const sym = (getGlobalSymbol() || '').toUpperCase();
        if (!sym) {
            showToast(t('toast.error.api', { err: t('toast.err.no_symbol') }), { level: 'error' });
            return;
        }
        window.location.hash = `${viewId}/${sym}`;
    };
    window.addEventListener('tv:open-charts-for-symbol',   navForSymbol('charts'));
    window.addEventListener('tv:open-options-for-symbol',  navForSymbol('options'));
    window.addEventListener('tv:open-research-for-symbol', navForSymbol('research'));
    window.addEventListener('tv:open-earnings-for-symbol', navForSymbol('earnings-iv'));
    // News view doesn't accept a hash-path symbol — it's filtered via
    // its own form. Navigate to the view; the user picks the symbol.
    window.addEventListener('tv:open-news-for-symbol',     () => { window.location.hash = 'news'; });
    // Trade-row actions — read data-id from the right-clicked <tr>.
    const tradeIdFrom = (detail) => {
        const t = detail && detail.target;
        if (!t || !t.dataset) return null;
        return t.dataset.id || null;
    };
    window.addEventListener('tv:trade-view-detail', (e) => {
        const id = tradeIdFrom(e.detail);
        if (!id) {
            showToast(t('toast.error.api', { err: t('toast.err.no_trade') }), { level: 'error' });
            return;
        }
        window.location.hash = `trade/${id}`;
    });
    window.addEventListener('tv:trade-copy-id', (e) => {
        const id = tradeIdFrom(e.detail);
        if (!id) {
            showToast(t('toast.error.api', { err: t('toast.err.no_trade') }), { level: 'error' });
            return;
        }
        if (navigator.clipboard && navigator.clipboard.writeText) {
            void navigator.clipboard.writeText(id).then(
                () => showToast(t('toast.copied', { what: id }), { level: 'success' }),
                () => showToast(t('toast.error.api', { err: t('toast.err.clipboard_denied') }), { level: 'error' }),
            );
        }
    });
    // Watchlist-row actions — read data-symbol (and data-wid for remove)
    // from the right-clicked <tr>.
    const wlSymbolFrom = (detail) => {
        const t = detail && detail.target;
        if (!t || !t.dataset) return null;
        return (t.dataset.symbol || '').toUpperCase() || null;
    };
    const wlWidFrom = (detail) => {
        const t = detail && detail.target;
        if (!t || !t.dataset) return null;
        return t.dataset.wid || null;
    };
    const wlNavTo = (viewId) => (e) => {
        const sym = wlSymbolFrom(e.detail);
        if (!sym) {
            showToast(t('toast.error.api', { err: t('toast.err.no_symbol') }), { level: 'error' });
            return;
        }
        setGlobalSymbol(sym);
        window.location.hash = `${viewId}/${sym}`;
    };
    window.addEventListener('tv:wl-row-set-active', (e) => {
        const sym = wlSymbolFrom(e.detail);
        if (!sym) {
            showToast(t('toast.error.api', { err: t('toast.err.no_symbol') }), { level: 'error' });
            return;
        }
        setGlobalSymbol(sym);
        showToast(t('toast.symbol_set_active', { sym }), { level: 'success' });
    });
    window.addEventListener('tv:wl-row-charts',   wlNavTo('charts'));
    window.addEventListener('tv:wl-row-research', wlNavTo('research'));
    window.addEventListener('tv:wl-row-options',  wlNavTo('options'));
    window.addEventListener('tv:wl-row-remove', (e) => {
        const sym = wlSymbolFrom(e.detail);
        const wid = wlWidFrom(e.detail);
        if (!sym || !wid) {
            showToast(t('toast.error.api', { err: t('toast.err.no_symbol') }), { level: 'error' });
            return;
        }
        void (async () => {
            if (!await tConfirm('ctxmenu.wl_row_remove_confirm', { sym }, { level: 'danger' })) return;
            try {
                await api.removeWatchlistSym(wid, sym);
                showToast(t('toast.wl_symbol_removed', { sym }), { level: 'success' });
                window.dispatchEvent(new HashChangeEvent('hashchange'));
            } catch (err) {
                showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
            }
        })();
    });
    // Position-row actions — read data-symbol (and data-id for trade
    // detail nav). Same shape as wlNavTo / wlSymbolFrom; reuses them.
    window.addEventListener('tv:pos-row-view-trade', (e) => {
        const target = e.detail && e.detail.target;
        const id = target && target.dataset && target.dataset.id;
        if (!id) {
            showToast(t('toast.error.api', { err: t('toast.err.no_trade') }), { level: 'error' });
            return;
        }
        window.location.hash = `trade/${id}`;
    });
    window.addEventListener('tv:pos-row-set-active', (e) => {
        const sym = wlSymbolFrom(e.detail);
        if (!sym) {
            showToast(t('toast.error.api', { err: t('toast.err.no_symbol') }), { level: 'error' });
            return;
        }
        setGlobalSymbol(sym);
        showToast(t('toast.symbol_set_active', { sym }), { level: 'success' });
    });
    window.addEventListener('tv:pos-row-charts',   wlNavTo('charts'));
    window.addEventListener('tv:pos-row-research', wlNavTo('research'));
    window.addEventListener('tv:pos-row-options',  wlNavTo('options'));
    // Alert-rule-row actions — read data-rule-id from the right-clicked
    // <div>. Mutates engine state in localStorage, then re-dispatches
    // hashchange so the view repaints.
    const ruleIdFrom = (detail) => {
        const t = detail && detail.target;
        if (!t || !t.dataset) return null;
        return t.dataset.ruleId || null;
    };
    window.addEventListener('tv:ar-row-toggle', (e) => {
        const id = ruleIdFrom(e.detail);
        if (!id) return;
        let s = alertEngine.loadState();
        const rule = (s.rules || []).find(r => r.id === id);
        if (!rule) return;
        s = alertEngine.setEnabled(s, id, !rule.enabled);
        alertEngine.saveState(s);
        showToast(
            t(rule.enabled ? 'toast.ar_disabled' : 'toast.ar_enabled', { name: rule.name }),
            { level: 'success' });
        window.dispatchEvent(new HashChangeEvent('hashchange'));
    });
    window.addEventListener('tv:ar-row-duplicate', (e) => {
        const id = ruleIdFrom(e.detail);
        if (!id) return;
        let s = alertEngine.loadState();
        const rule = (s.rules || []).find(r => r.id === id);
        if (!rule) return;
        const clone = alertEngine.newRule(rule.type, `${rule.name} (copy)`);
        // Preserve all the user's tuned fields, keep the new id + name.
        const dup = { ...rule, id: clone.id, name: clone.name };
        s = alertEngine.addRule(s, dup);
        alertEngine.saveState(s);
        showToast(t('toast.ar_duplicated', { name: dup.name }), { level: 'success' });
        window.dispatchEvent(new HashChangeEvent('hashchange'));
    });
    window.addEventListener('tv:ar-row-delete', (e) => {
        const id = ruleIdFrom(e.detail);
        if (!id) return;
        void (async () => {
            const s0 = alertEngine.loadState();
            const rule = (s0.rules || []).find(r => r.id === id);
            const name = rule ? rule.name : id;
            if (!await tConfirm('ctxmenu.ar_row_delete_confirm', { name }, { level: 'danger' })) return;
            const s = alertEngine.removeRule(s0, id);
            alertEngine.saveState(s);
            showToast(t('toast.ar_deleted', { name }), { level: 'success' });
            window.dispatchEvent(new HashChangeEvent('hashchange'));
        })();
    });
    // Board-row actions — read data-id / data-name.
    window.addEventListener('tv:board-row-open', (e) => {
        const tgt = e.detail && e.detail.target;
        const id = tgt && tgt.dataset && tgt.dataset.id;
        if (!id) return;
        window.location.hash = `boards/${id}`;
    });
    window.addEventListener('tv:board-row-copy-id', (e) => {
        const tgt = e.detail && e.detail.target;
        const id = tgt && tgt.dataset && tgt.dataset.id;
        if (!id) return;
        if (navigator.clipboard && navigator.clipboard.writeText) {
            void navigator.clipboard.writeText(id).then(
                () => showToast(t('toast.copied', { what: id }), { level: 'success' }),
                () => showToast(t('toast.error.api', { err: t('toast.err.clipboard_denied') }), { level: 'error' }),
            );
        }
    });
    window.addEventListener('tv:board-row-delete', (e) => {
        const tgt = e.detail && e.detail.target;
        const id = tgt && tgt.dataset && tgt.dataset.id;
        const name = (tgt && tgt.dataset && tgt.dataset.name) || id;
        if (!id) return;
        void (async () => {
            if (!await tConfirm('ctxmenu.board_row_delete_confirm', { name }, { level: 'danger' })) return;
            try {
                await api.deleteDashboard(id);
                showToast(t('toast.board_deleted', { name }), { level: 'success' });
                window.dispatchEvent(new HashChangeEvent('hashchange'));
            } catch (err) {
                showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
            }
        })();
    });
    // Backtest-preset-row actions — read data-id / data-slug / data-name / data-mine.
    window.addEventListener('tv:bp-row-copy-slug', (e) => {
        const tgt = e.detail && e.detail.target;
        const slug = tgt && tgt.dataset && tgt.dataset.slug;
        if (!slug) return;
        if (navigator.clipboard && navigator.clipboard.writeText) {
            void navigator.clipboard.writeText(slug).then(
                () => showToast(t('toast.copied', { what: slug }), { level: 'success' }),
                () => showToast(t('toast.error.api', { err: t('toast.err.clipboard_denied') }), { level: 'error' }),
            );
        }
    });
    window.addEventListener('tv:bp-row-open', (e) => {
        const tgt = e.detail && e.detail.target;
        const slug = tgt && tgt.dataset && tgt.dataset.slug;
        if (!slug) return;
        window.location.hash = `backtest-presets/${slug}`;
    });
    window.addEventListener('tv:bp-row-fork', (e) => {
        const tgt = e.detail && e.detail.target;
        const slug = tgt && tgt.dataset && tgt.dataset.slug;
        const isMine = tgt && tgt.dataset && tgt.dataset.mine === 'true';
        if (!slug) return;
        if (isMine) {
            showToast(t('toast.bp_already_mine'), { level: 'warning' });
            return;
        }
        void (async () => {
            try {
                const forked = await api.forkBacktestPreset(slug);
                showToast(t('toast.bp_forked', { name: forked && forked.name ? forked.name : slug }), { level: 'success' });
                window.dispatchEvent(new HashChangeEvent('hashchange'));
            } catch (err) {
                showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
            }
        })();
    });
    window.addEventListener('tv:bp-row-delete', (e) => {
        const tgt = e.detail && e.detail.target;
        const id = tgt && tgt.dataset && tgt.dataset.id;
        const name = (tgt && tgt.dataset && tgt.dataset.name) || id;
        const isMine = tgt && tgt.dataset && tgt.dataset.mine === 'true';
        if (!id) return;
        if (!isMine) {
            showToast(t('toast.bp_not_mine'), { level: 'warning' });
            return;
        }
        void (async () => {
            if (!await tConfirm('ctxmenu.bp_row_delete_confirm', { name }, { level: 'danger' })) return;
            try {
                await api.deleteBacktestPreset(id);
                showToast(t('toast.bp_deleted', { name }), { level: 'success' });
                window.dispatchEvent(new HashChangeEvent('hashchange'));
            } catch (err) {
                showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
            }
        })();
    });
    // Share-row actions — read data-id / data-slug / data-mine.
    window.addEventListener('tv:share-row-copy-url', (e) => {
        const tgt = e.detail && e.detail.target;
        const slug = tgt && tgt.dataset && tgt.dataset.slug;
        if (!slug) return;
        const url = `${window.location.origin}${window.location.pathname}#shared/${slug}`;
        if (navigator.clipboard && navigator.clipboard.writeText) {
            void navigator.clipboard.writeText(url).then(
                () => showToast(t('toast.copied', { what: t('toast.what.url') }), { level: 'success' }),
                () => showToast(t('toast.error.api', { err: t('toast.err.clipboard_denied') }), { level: 'error' }),
            );
        }
    });
    window.addEventListener('tv:share-row-open', (e) => {
        const tgt = e.detail && e.detail.target;
        const slug = tgt && tgt.dataset && tgt.dataset.slug;
        if (!slug) return;
        window.location.hash = `shared/${slug}`;
    });
    window.addEventListener('tv:share-row-delete', (e) => {
        const tgt = e.detail && e.detail.target;
        const id = tgt && tgt.dataset && tgt.dataset.id;
        const isMine = tgt && tgt.dataset && tgt.dataset.mine === 'true';
        if (!id) return;
        if (!isMine) {
            showToast(t('toast.share_not_mine'), { level: 'warning' });
            return;
        }
        void (async () => {
            if (!await tConfirm('ctxmenu.share_row_delete_confirm', {}, { level: 'danger' })) return;
            try {
                await api.deleteShare(id);
                showToast(t('toast.share_deleted'), { level: 'success' });
                window.dispatchEvent(new HashChangeEvent('hashchange'));
            } catch (err) {
                showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
            }
        })();
    });
    // Plan-row actions — read data-id / data-symbol.
    window.addEventListener('tv:plan-row-copy-symbol', (e) => {
        const tgt = e.detail && e.detail.target;
        const sym = tgt && tgt.dataset && tgt.dataset.symbol;
        if (!sym) return;
        if (navigator.clipboard && navigator.clipboard.writeText) {
            void navigator.clipboard.writeText(sym).then(
                () => showToast(t('toast.copied', { what: sym }), { level: 'success' }),
                () => showToast(t('toast.error.api', { err: t('toast.err.clipboard_denied') }), { level: 'error' }),
            );
        }
    });
    window.addEventListener('tv:plan-row-abandon', (e) => {
        const tgt = e.detail && e.detail.target;
        const id = tgt && tgt.dataset && tgt.dataset.id;
        const sym = (tgt && tgt.dataset && tgt.dataset.symbol) || '';
        if (!id) return;
        void (async () => {
            if (!await tConfirm('ctxmenu.plan_row_abandon_confirm', { sym }, { level: 'danger' })) return;
            try {
                await api.abandonPlan(id);
                showToast(t('toast.plan_abandoned', { sym }), { level: 'success' });
                window.dispatchEvent(new HashChangeEvent('hashchange'));
            } catch (err) {
                showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
            }
        })();
    });
    // Account-row actions — read data-id / data-name.
    window.addEventListener('tv:acct-row-copy-id', (e) => {
        const tgt = e.detail && e.detail.target;
        const id = tgt && tgt.dataset && tgt.dataset.id;
        if (!id) return;
        if (navigator.clipboard && navigator.clipboard.writeText) {
            void navigator.clipboard.writeText(id).then(
                () => showToast(t('toast.copied', { what: id }), { level: 'success' }),
                () => showToast(t('toast.error.api', { err: t('toast.err.clipboard_denied') }), { level: 'error' }),
            );
        }
    });
    window.addEventListener('tv:acct-row-delete', (e) => {
        const tgt = e.detail && e.detail.target;
        const id = tgt && tgt.dataset && tgt.dataset.id;
        const name = (tgt && tgt.dataset && tgt.dataset.name) || id;
        if (!id) return;
        void (async () => {
            if (!await tConfirm('ctxmenu.acct_row_delete_confirm', { name }, { level: 'danger' })) return;
            try {
                await api.deleteAccount(id);
                showToast(t('toast.acct_deleted', { name }), { level: 'success' });
                window.dispatchEvent(new HashChangeEvent('hashchange'));
            } catch (err) {
                showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
            }
        })();
    });
    // Custom-indicator-row actions — read data-id / data-name / data-definition.
    window.addEventListener('tv:ci-row-copy-def', (e) => {
        const tgt = e.detail && e.detail.target;
        const def = tgt && tgt.dataset && tgt.dataset.definition;
        const name = (tgt && tgt.dataset && tgt.dataset.name) || '';
        if (!def) return;
        if (navigator.clipboard && navigator.clipboard.writeText) {
            void navigator.clipboard.writeText(def).then(
                () => showToast(t('toast.copied', { what: name || 'definition' }), { level: 'success' }),
                () => showToast(t('toast.error.api', { err: t('toast.err.clipboard_denied') }), { level: 'error' }),
            );
        }
    });
    window.addEventListener('tv:ci-row-delete', (e) => {
        const tgt = e.detail && e.detail.target;
        const id = tgt && tgt.dataset && tgt.dataset.id;
        const name = (tgt && tgt.dataset && tgt.dataset.name) || id;
        if (!id) return;
        void (async () => {
            if (!await tConfirm('ctxmenu.ci_row_delete_confirm', { name }, { level: 'danger' })) return;
            try {
                await api.deleteCustomIndicator(id);
                showToast(t('toast.ci_deleted', { name }), { level: 'success' });
                window.dispatchEvent(new HashChangeEvent('hashchange'));
            } catch (err) {
                showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
            }
        })();
    });
    // Hotkey-row actions — read data-id / data-combo.
    window.addEventListener('tv:hk-row-copy-combo', (e) => {
        const tgt = e.detail && e.detail.target;
        const combo = tgt && tgt.dataset && tgt.dataset.combo;
        if (!combo) return;
        if (navigator.clipboard && navigator.clipboard.writeText) {
            void navigator.clipboard.writeText(combo).then(
                () => showToast(t('toast.copied', { what: combo }), { level: 'success' }),
                () => showToast(t('toast.error.api', { err: t('toast.err.clipboard_denied') }), { level: 'error' }),
            );
        }
    });
    window.addEventListener('tv:hk-row-delete', (e) => {
        const tgt = e.detail && e.detail.target;
        const id = tgt && tgt.dataset && tgt.dataset.id;
        if (!id) return;
        void (async () => {
            if (!await tConfirm('ctxmenu.hk_row_delete_confirm', {}, { level: 'danger' })) return;
            try {
                await api.deleteHotkey(id);
                showToast(t('toast.hk_deleted'), { level: 'success' });
                window.dispatchEvent(new HashChangeEvent('hashchange'));
            } catch (err) {
                showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
            }
        })();
    });
    // Journal-entry actions — read data-id (and data-trade-id for nav).
    window.addEventListener('tv:je-view-trade', (e) => {
        const tgt = e.detail && e.detail.target;
        const tradeId = tgt && tgt.dataset && tgt.dataset.tradeId;
        if (!tradeId) {
            showToast(t('toast.je_no_trade_linked'), { level: 'warning' });
            return;
        }
        window.location.hash = `trade/${tradeId}`;
    });
    window.addEventListener('tv:je-delete', (e) => {
        const tgt = e.detail && e.detail.target;
        const id = tgt && tgt.dataset && tgt.dataset.id;
        if (!id) return;
        void (async () => {
            if (!await tConfirm('ctxmenu.je_delete_confirm', {}, { level: 'danger' })) return;
            try {
                await api.deleteJournal(id);
                showToast(t('toast.je_deleted'), { level: 'success' });
                window.dispatchEvent(new HashChangeEvent('hashchange'));
            } catch (err) {
                showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
            }
        })();
    });
    // API-token-row actions — read data-id / data-prefix / data-revoked.
    window.addEventListener('tv:tok-row-copy-prefix', (e) => {
        const tgt = e.detail && e.detail.target;
        const prefix = tgt && tgt.dataset && tgt.dataset.prefix;
        if (!prefix) return;
        if (navigator.clipboard && navigator.clipboard.writeText) {
            void navigator.clipboard.writeText(prefix).then(
                () => showToast(t('toast.copied', { what: prefix }), { level: 'success' }),
                () => showToast(t('toast.error.api', { err: t('toast.err.clipboard_denied') }), { level: 'error' }),
            );
        }
    });
    window.addEventListener('tv:tok-row-revoke', (e) => {
        const tgt = e.detail && e.detail.target;
        const id = tgt && tgt.dataset && tgt.dataset.id;
        const alreadyRevoked = tgt && tgt.dataset && tgt.dataset.revoked === 'true';
        if (!id) return;
        if (alreadyRevoked) {
            showToast(t('toast.tok_already_revoked'), { level: 'warning' });
            return;
        }
        void (async () => {
            if (!await tConfirm('view.api_tokens.confirm.revoke', {}, { level: 'danger' })) return;
            try {
                await api.revokeApiToken(id);
                showToast(t('toast.tok_revoked'), { level: 'success' });
                window.dispatchEvent(new HashChangeEvent('hashchange'));
            } catch (err) {
                showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
            }
        })();
    });
    // Tag-chip actions — read data-id + data-name from the span.
    window.addEventListener('tv:tag-chip-copy', (e) => {
        const tgt = e.detail && e.detail.target;
        const name = tgt && tgt.dataset && tgt.dataset.name;
        if (!name) return;
        if (navigator.clipboard && navigator.clipboard.writeText) {
            void navigator.clipboard.writeText(name).then(
                () => showToast(t('toast.copied', { what: name }), { level: 'success' }),
                () => showToast(t('toast.error.api', { err: t('toast.err.clipboard_denied') }), { level: 'error' }),
            );
        }
    });
    window.addEventListener('tv:tag-chip-delete', (e) => {
        const tgt = e.detail && e.detail.target;
        const id = tgt && tgt.dataset && tgt.dataset.id;
        const name = (tgt && tgt.dataset && tgt.dataset.name) || id;
        if (!id) return;
        void (async () => {
            if (!await tConfirm('ctxmenu.tag_chip_delete_confirm', { name }, { level: 'danger' })) return;
            try {
                await api.deleteTag(id);
                showToast(t('toast.tag_deleted', { name }), { level: 'success' });
                window.dispatchEvent(new HashChangeEvent('hashchange'));
            } catch (err) {
                showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
            }
        })();
    });
    // Webhook-row actions — read data-id (and data-enabled for toggle) from tr.
    const idFromTr = (detail) => {
        const t = detail && detail.target;
        return (t && t.dataset && t.dataset.id) || null;
    };
    window.addEventListener('tv:wh-row-test', (e) => {
        const id = idFromTr(e.detail);
        if (!id) return;
        void (async () => {
            try {
                await api.testWebhook(id);
                showToast(t('toast.wh_test_fired'), { level: 'success' });
                window.dispatchEvent(new HashChangeEvent('hashchange'));
            } catch (err) {
                showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
            }
        })();
    });
    window.addEventListener('tv:wh-row-toggle', (e) => {
        const id = idFromTr(e.detail);
        const tgt = e.detail && e.detail.target;
        const wasEnabled = tgt && tgt.dataset && tgt.dataset.enabled === 'true';
        if (!id) return;
        void (async () => {
            try {
                await api.toggleWebhook(id, !wasEnabled);
                showToast(t(wasEnabled ? 'toast.wh_disabled' : 'toast.wh_enabled'), { level: 'success' });
                window.dispatchEvent(new HashChangeEvent('hashchange'));
            } catch (err) {
                showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
            }
        })();
    });
    window.addEventListener('tv:wh-row-delete', (e) => {
        const id = idFromTr(e.detail);
        if (!id) return;
        void (async () => {
            if (!await tConfirm('ctxmenu.wh_row_delete_confirm', {}, { level: 'danger' })) return;
            try {
                await api.deleteWebhook(id);
                showToast(t('toast.wh_deleted'), { level: 'success' });
                window.dispatchEvent(new HashChangeEvent('hashchange'));
            } catch (err) {
                showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
            }
        })();
    });
    // Strategy-alert-row actions — read data-id from the tr, call API.
    window.addEventListener('tv:sa-row-toggle', (e) => {
        const target = e.detail && e.detail.target;
        const id = target && target.dataset && target.dataset.id;
        if (!id) return;
        void (async () => {
            try {
                const rules = await api.listStrategyAlerts();
                const row = rules.find(x => x.id === id);
                if (!row) return;
                await api.updateStrategyAlert(id, { enabled: !row.enabled });
                showToast(
                    t(row.enabled ? 'toast.sa_disabled' : 'toast.sa_enabled', { name: row.name }),
                    { level: 'success' });
                window.dispatchEvent(new HashChangeEvent('hashchange'));
            } catch (err) {
                showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
            }
        })();
    });
    window.addEventListener('tv:sa-row-delete', (e) => {
        const target = e.detail && e.detail.target;
        const id = target && target.dataset && target.dataset.id;
        if (!id) return;
        void (async () => {
            if (!await tConfirm('ctxmenu.sa_row_delete_confirm', {}, { level: 'danger' })) return;
            try {
                await api.deleteStrategyAlert(id);
                showToast(t('toast.sa_deleted', { id }), { level: 'success' });
                window.dispatchEvent(new HashChangeEvent('hashchange'));
            } catch (err) {
                showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
            }
        })();
    });
    window.addEventListener('tv:trade-delete', (e) => {
        const id = tradeIdFrom(e.detail);
        if (!id) {
            showToast(t('toast.error.api', { err: t('toast.err.no_trade') }), { level: 'error' });
            return;
        }
        void (async () => {
            if (!await tConfirm('view.trades.confirm.delete', {}, { level: 'danger' })) return;
            try {
                await api.deleteTrade(id);
                showToast(t('toast.trade_deleted', { id }), { level: 'success' });
                window.dispatchEvent(new HashChangeEvent('hashchange'));
            } catch (err) {
                showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
            }
        })();
    });
    window.addEventListener('tv:toggle-favorite', () => {
        const vid = currentViewId();
        if (!vid) {
            showToast(t('toast.error.api', { err: t('toast.err.no_view') }), { level: 'error' });
            return;
        }
        const state = loadState();
        const next = toggleFavorite(state, vid);
        saveState(next);
        const nowFav = isFavorite(next, vid);
        showToast(
            t(nowFav ? 'toast.favorite_added' : 'toast.favorite_removed', { view: vid }),
            { level: 'success' });
        window.dispatchEvent(new CustomEvent('tv:favorites-changed'));
    });
}

function currentViewId() {
    const h = (window.location.hash || '').replace(/^#/, '').split('?')[0];
    return h || null;
}

export function registerContextItems(scope, items) {
    if (!scope || !Array.isArray(items)) return;
    _customByScope.set(scope, items);
}

function ensureMount() {
    if (document.getElementById('tv-ctxmenu-root')) return;
    const root = document.createElement('div');
    root.id = 'tv-ctxmenu-root';
    document.body.appendChild(root);
}

function onContextMenu(e) {
    // Hold Shift to bypass and get native browser menu (escape hatch).
    if (e.shiftKey) return;
    e.preventDefault();
    const { scope, el } = nearestScopeAndElement(e.target);
    const editing = isTextEntry(e.target) ? e.target : null;
    _editingTarget = editing;
    _scopeTarget = el;
    openAt(e.clientX, e.clientY, scope, !!editing);
}

function isTextEntry(el) {
    if (!el) return false;
    const tag = (el.tagName || '').toLowerCase();
    if (tag === 'textarea') return true;
    if (tag === 'input') {
        const type = (el.getAttribute('type') || 'text').toLowerCase();
        return !['button', 'submit', 'reset', 'checkbox', 'radio', 'file', 'image', 'range', 'color'].includes(type);
    }
    return !!el.isContentEditable;
}

// Run a synchronous edit command (cut/copy/selectAll/undo/redo). Falls
// back silently when the browser refuses (e.g. CSP / cross-origin).
// Target priority: ctx-menu target → document.activeElement (palette
// path) → no-op.
function execEdit(cmd) {
    const tgt = resolveEditTarget();
    if (!tgt) return;
    if (typeof tgt.focus === 'function') tgt.focus();
    try {
        if (typeof document.execCommand === 'function') document.execCommand(cmd);
    } catch (_) { /* ignored */ }
}

// Paste needs the async clipboard API in modern browsers; execCommand
// `paste` is widely blocked outside browser extensions.
async function execPaste() {
    const tgt = resolveEditTarget();
    if (!tgt) return;
    try {
        const txt = await navigator.clipboard.readText();
        if (typeof tgt.setRangeText === 'function' && typeof tgt.selectionStart === 'number') {
            const s = tgt.selectionStart, e = tgt.selectionEnd;
            tgt.setRangeText(txt, s, e, 'end');
            tgt.dispatchEvent(new Event('input', { bubbles: true }));
        } else if (tgt.isContentEditable) {
            tgt.focus();
            document.execCommand('insertText', false, txt);
        }
    } catch (_) { /* clipboard denied */ }
}

// Edit commands can fire from two sources: a right-click on an input
// (ctxmenu sets _editingTarget) or the command palette (no target —
// fall back to whatever has focus). Returns null when neither path
// yields a text-entry element.
function resolveEditTarget() {
    if (_editingTarget && isTextEntry(_editingTarget)) return _editingTarget;
    const ae = (typeof document !== 'undefined') ? document.activeElement : null;
    if (ae && isTextEntry(ae)) return ae;
    return null;
}

function nearestScope(el) {
    return nearestScopeAndElement(el).scope;
}

// Like nearestScope but also returns the element carrying the matched
// attribute. Handlers reading row-level data (data-id, data-symbol) use
// the element via CustomEvent detail.target.
function nearestScopeAndElement(el) {
    let cur = el;
    while (cur && cur.nodeType === 1) {
        const s = cur.getAttribute && cur.getAttribute('data-context-scope');
        if (s) return { scope: s, el: cur };
        cur = cur.parentNode;
    }
    return { scope: null, el: null };
}

function openAt(x, y, scope, editing = false) {
    const custom = scope ? (_customByScope.get(scope) || []) : [];
    const merged = editing
        ? mergeMenuWithEditing(GLOBAL_ITEMS, custom, EDITING_ITEMS)
        : mergeMenu(GLOBAL_ITEMS, custom);
    _items = compileMenu(merged);
    _selected = -1;
    _open = true;
    paint(x, y);
}

function paint(x, y) {
    const root = document.getElementById('tv-ctxmenu-root');
    if (!root) return;
    root.innerHTML = `
        <div class="tv-ctxmenu" role="menu" id="tv-ctxmenu">
            ${_items.map((it, i) => renderItem(it, i)).join('')}
        </div>
    `;
    applyUiI18n(root);
    const menu = root.querySelector('.tv-ctxmenu');
    if (!menu) return;
    const w = menu.offsetWidth  || 220;
    const h = menu.offsetHeight || 200;
    const pos = positionMenu(x, y, w, h,
        window.innerWidth || 1024, window.innerHeight || 768, 8);
    menu.style.left = pos.x + 'px';
    menu.style.top  = pos.y + 'px';
    menu.addEventListener('click', onItemClick);
    menu.addEventListener('mousemove', onHover);
}

function renderItem(it, idx) {
    if (it.kind === 'separator') return `<div class="tv-ctxmenu-sep"></div>`;
    return `<div class="tv-ctxmenu-item" role="menuitem"
                 data-idx="${idx}"
                 data-i18n="${esc(it.labelKey)}">${esc(t(it.labelKey))}</div>`;
}

function onItemClick(e) {
    const row = e.target.closest('.tv-ctxmenu-item');
    if (!row) return;
    const idx = parseInt(row.dataset.idx, 10);
    activate(_items[idx]);
}

function onHover(e) {
    const row = e.target.closest('.tv-ctxmenu-item');
    if (!row) return;
    _selected = parseInt(row.dataset.idx, 10);
    repaintSelection();
}

function repaintSelection() {
    document.querySelectorAll('.tv-ctxmenu-item').forEach((el, i) => {
        const idx = parseInt(el.dataset.idx, 10);
        el.classList.toggle('active', idx === _selected);
    });
}

function onDocClick(e) {
    if (!_open) return;
    const menu = document.getElementById('tv-ctxmenu');
    if (menu && menu.contains(e.target)) return;
    close();
}

function onKey(e) {
    if (!_open) return;
    if (e.key === 'Escape') { e.preventDefault(); close(); return; }
    if (e.key === 'ArrowDown') {
        e.preventDefault();
        _selected = nextVisibleIdx(_items, _selected, 1);
        repaintSelection();
    } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        _selected = nextVisibleIdx(_items, _selected, -1);
        repaintSelection();
    } else if (e.key === 'Enter') {
        e.preventDefault();
        if (_selected >= 0 && _selected < _items.length) activate(_items[_selected]);
    }
}

function activate(item) {
    if (!item) return;
    const scopeTarget = _scopeTarget;
    close();
    if (item.actionKey) {
        window.dispatchEvent(new CustomEvent(item.actionKey, {
            detail: { item, target: scopeTarget },
        }));
    }
    if (item.navTo) {
        window.location.hash = item.navTo;
    }
    if (typeof item.onClick === 'function') {
        try { item.onClick(item); }
        catch (e) { console.error('ctxmenu onClick failed', e); }
    }
}

function close() {
    _open = false;
    _scopeTarget = null;
    const root = document.getElementById('tv-ctxmenu-root');
    if (root) root.innerHTML = '';
}
