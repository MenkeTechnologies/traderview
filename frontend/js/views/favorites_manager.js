// Favorites + bookmarks management view. Lists every saved favorite and
// every bookmark; lets the user rename / remove / clear via buttons or
// right-click. The launcher's star toggle + ctxmenu "Bookmark this view…"
// create entries; this is where they're maintained.

import { go } from '../app.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { TILES } from './launcher.js';
import { tilesByViewId } from '../_command_palette_inputs.js';
import { showToast } from '../toast.js';
import { tConfirm, tPrompt } from '../dialog.js';
import * as favs from '../_favorites_storage.js';

let _filter = '';
let _wired  = false;

export async function renderFavoritesManager(mount, _state) {
    mount.innerHTML = `
        <h1 data-i18n="view.favorites.h1.title" class="view-title">// FAVORITES &amp; BOOKMARKS</h1>

        <div class="chart-panel">
            <input id="fav-filter" type="text" autocomplete="off" spellcheck="false"
                   data-i18n-placeholder="view.favorites.filter_placeholder"
                   data-tip="view.favorites.filter_tip"
                   value="${esc(_filter)}"
                   placeholder="Filter favorites + bookmarks…  (Esc to clear)"
                   style="width:100%">
        </div>

        <div class="chart-panel" data-context-scope="favorites-manager">
            <h2 data-i18n="view.favorites.h2.favorites">Favorites</h2>
            <div id="fav-list"></div>
            <div class="inline-form">
                <button data-i18n="view.favorites.btn.clear_favorites" id="fav-clear"
                        class="btn btn-secondary"
                        data-tip="view.favorites.tip.clear_favorites" type="button">Clear all favorites</button>
            </div>
        </div>

        <div class="chart-panel" data-context-scope="bookmarks-manager">
            <h2 data-i18n="view.favorites.h2.bookmarks">Bookmarks</h2>
            <div id="bm-list"></div>
            <div class="inline-form">
                <button data-i18n="view.favorites.btn.clear_bookmarks" id="bm-clear"
                        class="btn btn-secondary"
                        data-tip="view.favorites.tip.clear_bookmarks" type="button">Clear all bookmarks</button>
            </div>
        </div>

        <p data-i18n="view.favorites.hint" class="muted">Click a row to open. Use the star on launcher tiles to add favorites; right-click any view and pick "Bookmark this view…" (or press <kbd>Cmd+B</kbd>) to add a named bookmark.</p>
    `;

    paint();
    document.getElementById('fav-clear').addEventListener('click', clearFavoritesClick);
    document.getElementById('bm-clear').addEventListener('click', clearBookmarksClick);
    const filter = document.getElementById('fav-filter');
    if (filter) {
        filter.addEventListener('input', (e) => { _filter = e.target.value; paint(); });
        filter.addEventListener('keydown', (e) => {
            if (e.key === 'Escape' && filter.value) {
                e.stopPropagation();
                filter.value = '';
                _filter = '';
                paint();
            }
        });
    }
    if (!_wired) {
        _wired = true;
        // Repaint only while the manager is the active view, so external
        // mutations (Cmd+B from another view, launcher star toggle) flow
        // in next time the user opens this view too.
        window.addEventListener('tv:favorites-changed', () => {
            if ((window.location.hash || '').replace(/^#/, '').split('/')[0] === 'favorites') {
                paint();
            }
        });
    }
}

function matchesFilter(label, viewId, name) {
    const q = (_filter || '').trim().toLowerCase();
    if (!q) return true;
    return (label || '').toLowerCase().includes(q)
        || (viewId || '').toLowerCase().includes(q)
        || (name || '').toLowerCase().includes(q);
}

function paint() {
    const byVid = tilesByViewId(TILES);
    const state = favs.loadState();

    // Favorites list.
    const favList = document.getElementById('fav-list');
    if (favList) {
        const ids = (Array.isArray(state.favorites) ? state.favorites : [])
            .filter(vid => {
                const tile = byVid.get(vid);
                return matchesFilter(tile ? tile[1] : vid, vid, '');
            });
        if (ids.length === 0) {
            const key = _filter ? 'view.favorites.empty_filtered' : 'view.favorites.empty_favorites';
            favList.innerHTML = `<div class="muted" data-i18n="${key}">${esc(t(key))}</div>`;
        } else {
            favList.innerHTML = `
                <table class="lq-table">
                    <thead><tr>
                        <th></th>
                        <th data-i18n="view.favorites.col.label">Label</th>
                        <th data-i18n="view.favorites.col.viewid">View ID</th>
                        <th></th>
                    </tr></thead>
                    <tbody>
                        ${ids.map(vid => {
                            const tile = byVid.get(vid);
                            const icon = tile ? tile[2] : '☆';
                            const label = tile ? tile[1] : vid;
                            return `<tr data-go="${esc(vid)}">
                                <td>${esc(icon)}</td>
                                <td>${esc(label)}</td>
                                <td class="muted"><code>${esc(vid)}</code></td>
                                <td><button class="btn btn-secondary"
                                            data-remove-fav="${esc(vid)}"
                                            data-tip="view.favorites.tip.remove_fav"
                                            data-i18n-aria-label="view.favorites.aria.remove_fav"
                                            type="button">✕</button></td>
                            </tr>`;
                        }).join('')}
                    </tbody>
                </table>
            `;
            favList.querySelectorAll('tr[data-go]').forEach(row => {
                row.addEventListener('click', (e) => {
                    if (e.target instanceof HTMLElement && e.target.closest('button')) return;
                    go(row.dataset.go);
                });
            });
            favList.querySelectorAll('button[data-remove-fav]').forEach(btn => {
                btn.addEventListener('click', (e) => {
                    e.stopPropagation();
                    const vid = btn.dataset.removeFav;
                    favs.saveState(favs.toggleFavorite(favs.loadState(), vid));
                    showToast(t('toast.favorite_removed', { view: vid }), { level: 'success' });
                    notify();
                });
            });
        }
    }

    // Bookmarks list.
    const bmList = document.getElementById('bm-list');
    if (bmList) {
        const bms = (Array.isArray(state.bookmarks) ? state.bookmarks : [])
            .filter(b => {
                const tile = byVid.get(b.viewId);
                return matchesFilter(tile ? tile[1] : b.viewId, b.viewId, b.name);
            });
        if (bms.length === 0) {
            const key = _filter ? 'view.favorites.empty_filtered' : 'view.favorites.empty_bookmarks';
            bmList.innerHTML = `<div class="muted" data-i18n="${key}">${esc(t(key))}</div>`;
        } else {
            bmList.innerHTML = `
                <table class="lq-table">
                    <thead><tr>
                        <th></th>
                        <th data-i18n="view.favorites.col.name">Name</th>
                        <th data-i18n="view.favorites.col.viewid">View ID</th>
                        <th data-i18n="view.favorites.col.created">Created</th>
                        <th></th>
                    </tr></thead>
                    <tbody>
                        ${bms.map(b => {
                            const tile = byVid.get(b.viewId);
                            const icon = tile ? tile[2] : '📌';
                            return `<tr data-go="${esc(b.viewId)}">
                                <td>${esc(icon)}</td>
                                <td>${esc(b.name)}</td>
                                <td class="muted"><code>${esc(b.viewId)}</code></td>
                                <td class="muted">${esc((b.created_at || '').slice(0, 10))}</td>
                                <td>
                                    <button class="btn btn-secondary"
                                            data-rename-bm="${esc(b.id)}"
                                            data-tip="view.favorites.tip.rename_bm"
                                            data-i18n-aria-label="view.favorites.aria.rename_bm"
                                            type="button">✎</button>
                                    <button class="btn btn-secondary"
                                            data-remove-bm="${esc(b.id)}"
                                            data-tip="view.favorites.tip.remove_bm"
                                            data-i18n-aria-label="view.favorites.aria.remove_bm"
                                            type="button">✕</button>
                                </td>
                            </tr>`;
                        }).join('')}
                    </tbody>
                </table>
            `;
            bmList.querySelectorAll('tr[data-go]').forEach(row => {
                row.addEventListener('click', (e) => {
                    if (e.target instanceof HTMLElement && e.target.closest('button')) return;
                    go(row.dataset.go);
                });
            });
            bmList.querySelectorAll('button[data-rename-bm]').forEach(btn => {
                btn.addEventListener('click', async (e) => {
                    e.stopPropagation();
                    const id = btn.dataset.renameBm;
                    const cur = favs.getBookmark(favs.loadState(), id);
                    if (!cur) return;
                    const name = (typeof window.prompt === 'function')
                        ? await tPrompt('prompt.bookmark_rename', { name: cur.name }, { defaultValue: cur.name })
                        : null;
                    if (name == null) return;
                    const trimmed = String(name).trim();
                    if (!trimmed) return;
                    favs.saveState(favs.renameBookmark(favs.loadState(), id, trimmed));
                    showToast(t('toast.bookmark_renamed', { name: trimmed }), { level: 'success' });
                    notify();
                });
            });
            bmList.querySelectorAll('button[data-remove-bm]').forEach(btn => {
                btn.addEventListener('click', (e) => {
                    e.stopPropagation();
                    const id = btn.dataset.removeBm;
                    const cur = favs.getBookmark(favs.loadState(), id);
                    favs.saveState(favs.removeBookmark(favs.loadState(), id));
                    showToast(t('toast.bookmark_removed', { name: cur ? cur.name : id }), { level: 'success' });
                    notify();
                });
            });
        }
    }
}

async function clearFavoritesClick() {
    if (!await tConfirm('confirm.clear_favorites', {}, { level: 'danger' })) return;
    favs.saveState(favs.clearFavorites(favs.loadState()));
    showToast(t('toast.favorites_cleared'), { level: 'success' });
    notify();
}

async function clearBookmarksClick() {
    if (!await tConfirm('confirm.clear_bookmarks', {}, { level: 'danger' })) return;
    const s = favs.loadState();
    favs.saveState({ ...s, bookmarks: [] });
    showToast(t('toast.bookmarks_cleared'), { level: 'success' });
    notify();
}

// Single dispatch site: re-paints this view (via the listener attached in
// renderFavoritesManager) AND notifies any other open surface (launcher
// tile stars, future favorites widgets) at the same time.
function notify() {
    window.dispatchEvent(new CustomEvent('tv:favorites-changed'));
}
