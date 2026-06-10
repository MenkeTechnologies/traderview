// Persistent log of every toast the session has emitted (mirrored to
// localStorage so a reload doesn't lose them). Useful for catching the
// last-second error toast you dismissed too quickly.
//
// Live-updating: subscribes to the toast module so new entries appear
// without refresh. Level filter + free-text filter.

import { getToastHistory, subscribeToastHistory, clearToastHistory } from '../toast.js';
import { esc, fmtDateTime } from '../util.js';
import { t } from '../i18n.js';
import { tConfirm } from '../dialog.js';
import { searchMatch, getMatchIndices, highlightWithIndices } from '../fzf.js';

const STATE = {
    levelFilter: 'all',   // 'all' | 'error' | 'warning' | 'success' | 'info'
    textFilter: '',
};

export async function renderToastHistory(mount) {
    mount.innerHTML = `<h1 data-i18n="view.toast_history.h1" class="view-title">// TOAST HISTORY</h1>
        <div class="toast-history-bar">
            <input id="th-filter" type="text" class="form-input" autocomplete="off"
                   placeholder="${esc(t('view.toast_history.filter_placeholder'))}">
            <select id="th-level">
                <option value="all"     data-i18n="view.toast_history.level.all">All</option>
                <option value="error"   data-i18n="view.toast_history.level.error">Errors</option>
                <option value="warning" data-i18n="view.toast_history.level.warning">Warnings</option>
                <option value="success" data-i18n="view.toast_history.level.success">Success</option>
                <option value="info"    data-i18n="view.toast_history.level.info">Info</option>
            </select>
            <span class="th-spacer"></span>
            <span class="muted small" id="th-count"></span>
            <button class="btn btn-secondary danger" id="th-clear"
                    data-i18n="view.toast_history.clear">Clear history</button>
        </div>
        <div id="th-list" class="toast-history-list"></div>`;
    const filterEl = mount.querySelector('#th-filter');
    const levelEl  = mount.querySelector('#th-level');
    const listEl   = mount.querySelector('#th-list');
    const countEl  = mount.querySelector('#th-count');
    levelEl.value = STATE.levelFilter;
    filterEl.value = STATE.textFilter;

    const repaint = () => {
        const all = getToastHistory();
        const q = STATE.textFilter.trim();
        const visible = all
            .filter(e => STATE.levelFilter === 'all' || e.level === STATE.levelFilter)
            .filter(e => !q || searchMatch(q, e.message))
            .slice().reverse();   // newest first
        countEl.textContent = t('view.toast_history.count', { shown: visible.length, total: all.length });
        if (visible.length === 0) {
            listEl.innerHTML = `<p class="muted">${esc(t('view.toast_history.empty'))}</p>`;
            return;
        }
        listEl.innerHTML = visible.map(e => {
            const msg = e.message || '';
            const msgHtml = q ? highlightWithIndices(msg, getMatchIndices(q, msg)) : esc(msg);
            return `
            <div class="th-row th-${esc(e.level || 'info')}">
                <span class="th-ts muted small">${esc(fmtDateTime(new Date(e.ts).toISOString()))}</span>
                <span class="th-level th-level-${esc(e.level)}">${esc((e.level || 'info').toUpperCase())}</span>
                <span class="th-msg">${msgHtml}</span>
                ${e.view ? `<a class="th-view muted small" href="#${esc(e.view)}">${esc(e.view)}</a>` : ''}
            </div>`;
        }).join('');
    };
    filterEl.addEventListener('input', () => { STATE.textFilter = filterEl.value; repaint(); });
    levelEl.addEventListener('change', () => { STATE.levelFilter = levelEl.value; repaint(); });
    mount.querySelector('#th-clear').addEventListener('click', async () => {
        const ok = await tConfirm('view.toast_history.confirm_clear', {}, { level: 'danger' });
        if (!ok) return;
        clearToastHistory();
        repaint();
    });
    // Live updates — re-render whenever a new toast lands or history clears.
    const unsub = subscribeToastHistory(() => repaint());
    mount.__toastHistoryUnsub = unsub;
    repaint();
}
