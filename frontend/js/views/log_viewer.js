// In-app log viewer — tails the backend log file via the Tauri
// `read_log_tail` command. Level filter (INFO/WARN/ERROR/DEBUG/TRACE),
// free-text filter, auto-refresh toggle, copy-to-clipboard. Pairs with
// the existing Toast History tile — toast history is what the USER
// saw, this view is what the BACKEND wrote.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';

const STATE = {
    refreshTimer: null,
    autoRefresh: true,
    levelFilter: 'all',
    textFilter: '',
    raw: '',
    path: '',
};

const POLL_MS = 2500;

export async function renderLogViewer(mount) {
    mount.innerHTML = `<h1 data-i18n="view.log_viewer.h1" class="view-title">// LOG VIEWER</h1>
        <div class="log-viewer-bar">
            <input id="lv-filter" type="text" class="form-input" autocomplete="off"
                   placeholder="${esc(t('view.log_viewer.filter_placeholder'))}">
            <select id="lv-level">
                <option value="all"   data-i18n="view.log_viewer.level.all">All</option>
                <option value="ERROR" data-i18n="view.log_viewer.level.error">Errors</option>
                <option value="WARN"  data-i18n="view.log_viewer.level.warn">Warnings</option>
                <option value="INFO"  data-i18n="view.log_viewer.level.info">Info</option>
                <option value="DEBUG" data-i18n="view.log_viewer.level.debug">Debug</option>
                <option value="TRACE" data-i18n="view.log_viewer.level.trace">Trace</option>
            </select>
            <label class="lv-auto">
                <input type="checkbox" id="lv-auto" ${STATE.autoRefresh ? 'checked' : ''}>
                <span data-i18n="view.log_viewer.auto_refresh">Auto-refresh</span>
            </label>
            <span class="th-spacer"></span>
            <button class="btn btn-secondary" id="lv-copy" data-i18n="view.log_viewer.copy">Copy path</button>
            <button class="btn btn-secondary" id="lv-refresh" data-i18n="view.log_viewer.refresh">Refresh</button>
        </div>
        <div class="log-viewer-pathbar" id="lv-pathbar"></div>
        <div id="lv-out" class="log-viewer-out"></div>`;

    const filterEl = mount.querySelector('#lv-filter');
    const levelEl = mount.querySelector('#lv-level');
    const autoEl = mount.querySelector('#lv-auto');
    const outEl = mount.querySelector('#lv-out');
    const pathbarEl = mount.querySelector('#lv-pathbar');

    filterEl.value = STATE.textFilter;
    levelEl.value = STATE.levelFilter;

    const isTauri = typeof window !== 'undefined' && window.__TAURI__;
    if (!isTauri) {
        outEl.innerHTML = `<p class="muted">${esc(t('view.log_viewer.web_unsupported'))}</p>`;
        return;
    }
    try {
        STATE.path = await window.__TAURI__.core.invoke('get_log_path');
        pathbarEl.innerHTML = `<code class="muted small">${esc(STATE.path)}</code>`;
    } catch { /* path is optional */ }

    let firstRender = true;
    const render = () => {
        // Capture scroll state BEFORE the innerHTML wipe — once we
        // re-set the DOM, scrollTop resets to 0 on browsers that
        // don't preserve it across reflows.
        const wasNearBottom = !firstRender &&
            (outEl.scrollHeight - outEl.scrollTop - outEl.clientHeight < 64);
        const q = STATE.textFilter.trim().toLowerCase();
        const lvl = STATE.levelFilter;
        const lines = STATE.raw.split('\n');
        const visible = lines.filter(line => {
            if (!line) return false;
            if (lvl !== 'all' && !line.includes(` ${lvl} `) && !line.includes(`${lvl} `)) return false;
            if (q && !line.toLowerCase().includes(q)) return false;
            return true;
        });
        outEl.innerHTML = visible.map(line => {
            const cls = line.includes(' ERROR ') ? 'lv-error'
                : line.includes(' WARN ')  ? 'lv-warn'
                : line.includes(' INFO ')  ? 'lv-info'
                : line.includes(' DEBUG ') ? 'lv-debug'
                : line.includes(' TRACE ') ? 'lv-trace'
                : '';
            return `<div class="lv-line ${cls}">${esc(line)}</div>`;
        }).join('');
        // First render: always snap to bottom so the user lands on the
        // freshest entries without having to scroll. Subsequent renders:
        // pin to bottom only if the user was already there (tail mode);
        // if they scrolled up to read something, don't yank them back.
        if (firstRender || wasNearBottom) {
            // rAF so the new innerHTML has been laid out and scrollHeight
            // reflects the post-update content size, not pre-update.
            requestAnimationFrame(() => {
                outEl.scrollTop = outEl.scrollHeight;
            });
            firstRender = false;
        }
    };

    const refresh = async () => {
        try {
            STATE.raw = await window.__TAURI__.core.invoke('read_log_tail', { maxBytes: 128 * 1024 });
            render();
        } catch (e) {
            outEl.innerHTML = `<p class="muted">${esc(t('view.log_viewer.read_failed', { err: e.message || String(e) }))}</p>`;
        }
    };

    filterEl.addEventListener('input', () => { STATE.textFilter = filterEl.value; render(); });
    levelEl.addEventListener('change', () => { STATE.levelFilter = levelEl.value; render(); });
    autoEl.addEventListener('change', () => {
        STATE.autoRefresh = autoEl.checked;
        if (STATE.refreshTimer) { clearInterval(STATE.refreshTimer); STATE.refreshTimer = null; }
        if (STATE.autoRefresh) STATE.refreshTimer = setInterval(refresh, POLL_MS);
    });
    mount.querySelector('#lv-refresh').addEventListener('click', refresh);
    mount.querySelector('#lv-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(STATE.path);
            showToast(t('view.log_viewer.toast_copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.log_viewer.toast_copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });

    await refresh();
    if (STATE.autoRefresh) STATE.refreshTimer = setInterval(refresh, POLL_MS);

    // Tear down on view exit — caller stashes this on the mount so the
    // route dispatcher can call it when navigating away.
    mount.__logViewerCleanup = () => {
        if (STATE.refreshTimer) clearInterval(STATE.refreshTimer);
        STATE.refreshTimer = null;
    };
}
