import { api } from '../api.js';
import { esc, fmt, fmtMoney, fmtDateTime, fmtSecs, makeFilter, pnlClass } from '../util.js';
import { go, currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { tConfirm, tPrompt } from '../dialog.js';
import { initDragReorder, resetDragReorder } from '../drag_reorder.js';

let currentFilter = {};
const PAGE_SIZE = 100;
let currentPage = 1;
let lastPageRows = 0;  // rows returned on the last page (to detect last page)
// Column-level sort state. Backend returns trades ordered by opened_at DESC;
// these get applied client-side after fetch so we don't hit the server when
// the user toggles direction or column.
let sortKey = '';      // '' = backend order, else one of trade keys
let sortDir = 'desc';  // 'asc' | 'desc'
// View mode mirrors Tradervue's Table / Charts(large) / Charts(small) toggle.
// Persisted in sessionStorage so the user's preference survives refresh.
const VIEW_MODE_KEY = 'tv-trades-view-mode';
const PNL_MODE_KEY  = 'tv-trades-pnl-mode';
function getViewMode() {
    const v = sessionStorage.getItem(VIEW_MODE_KEY);
    return (v === 'charts-large' || v === 'charts-small') ? v : 'table';
}
function setViewMode(m) { sessionStorage.setItem(VIEW_MODE_KEY, m); }
function getPnlMode() {
    return sessionStorage.getItem(PNL_MODE_KEY) === 'gross' ? 'gross' : 'net';
}
function setPnlMode(m) { sessionStorage.setItem(PNL_MODE_KEY, m); }

export async function renderTradesView(mount, state) {
    const tok = currentViewToken();
    if (!state.accountId) {
        mount.innerHTML = '<p data-i18n="view.trades.hint.no_account" class="boot">No account.</p>';
        return;
    }
    mount.innerHTML = `
        <h1 data-i18n="view.trades.h1.trades" class="view-title">// TRADES</h1>
        <div id="filter-mount"></div>
        <div class="trades-toolbar">
            <a href="#new-trade" class="btn primary" id="new-trade-link"
               data-i18n="view.trades.btn.new_trade"
               data-tip="view.trades.tip.new_trade"
               data-shortcut="trades_new">+ New trade</a>
            <button type="button" class="btn btn-secondary" id="trades-refresh-btn"
                    data-i18n="view.trades.btn.refresh"
                    data-tip="view.trades.tip.refresh"
                    data-shortcut="trades_refresh">⟳ Refresh</button>
            <button data-i18n="view.trades.btn.re_run_fifo" data-tip="view.trades.tip.rollup" class="primary" id="rollup-btn">Re-run FIFO</button>
            <button data-i18n="view.trades.btn.close_expired_options" data-tip="view.trades.tip.close_exp" class="primary btn-magenta-gradient" id="close-exp-btn">Close expired options</button>
            <span class="muted trades-selcount" id="sel-count">${esc(t('view.trades.label.n_selected', { n: 0 }))}</span>
            <select id="bulk-action" data-tip="view.trades.tip.bulk_action" class="trades-bulk-sel">
                <option data-i18n="view.trades.opt.bulk_action" value="">— bulk action —</option>
                <option data-i18n="view.trades.opt.delete" value="delete">Delete</option>
                <option data-i18n="view.trades.opt.merge_into_one" value="merge">Merge into one</option>
                <option data-i18n="view.trades.opt.split_re_fifo" value="split">Split (re-FIFO)</option>
                <option data-i18n="view.trades.opt.add_tag" value="add_tag">Add tag…</option>
                <option data-i18n="view.trades.opt.remove_tag" value="remove_tag">Remove tag…</option>
                <option data-i18n="view.trades.opt.set_risk_amount" value="set_risk">Set risk amount…</option>
                <option data-i18n="view.trades.opt.share_publicly" value="share">Share publicly</option>
            </select>
            <button data-i18n="view.trades.btn.apply" data-tip="view.trades.tip.apply" class="primary" id="apply-bulk" disabled>Apply</button>
            <a href="${api.exportTradesUrl(state.accountId)}" download class="btn btn-secondary btn-compact trades-export-link"
               data-i18n="view.trades.export.csv">Export CSV</a>
            <div class="trades-view-toggle" role="tablist" aria-label="View mode">
                <button type="button" data-mode="table"
                        class="${getViewMode() === 'table' ? 'active' : ''}"
                        data-i18n="view.trades.view.table">Table</button>
                <button type="button" data-mode="charts-large"
                        class="${getViewMode() === 'charts-large' ? 'active' : ''}"
                        data-i18n="view.trades.view.charts_large">Charts (large)</button>
                <button type="button" data-mode="charts-small"
                        class="${getViewMode() === 'charts-small' ? 'active' : ''}"
                        data-i18n="view.trades.view.charts_small">Charts (small)</button>
            </div>
            <div class="trades-pnl-toggle" role="tablist" aria-label="P&amp;L mode">
                <button type="button" data-pnl="gross"
                        class="${getPnlMode() === 'gross' ? 'active' : ''}"
                        data-i18n="view.trades.pnl.gross">Gross</button>
                <button type="button" data-pnl="net"
                        class="${getPnlMode() === 'net' ? 'active' : ''}"
                        data-i18n="view.trades.pnl.net">Net</button>
            </div>
        </div>
        <div id="trades-table"></div>
        <div id="trades-pagination" class="trades-pagination" hidden>
            <button type="button" id="page-prev" data-i18n="view.trades.pagination.prev">← Prev</button>
            <span class="page-info" id="page-info"></span>
            <button type="button" id="page-next" data-i18n="view.trades.pagination.next">Next →</button>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.trades.h2.pnl_chart">Net P&L by trade (closed only)</h2>
            <div id="trades-chart" class="chart-h-240"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.trades.h2.cum_chart">Cumulative equity over closed trades</h2>
            <div id="trades-cum-chart" class="chart-h-220"></div>
            <p data-i18n="view.trades.hint.cum_chart" class="muted small">Running sum of net P&L in close-date order. Orthogonal to per-trade dots: reveals trajectory, drawdowns and recovery across the filtered set. Yellow dashed = breakeven.</p>
        </div>
    `;
    const { el: fEl } = makeFilter(currentFilter, async (f) => {
        currentFilter = f;
        await refresh();
    });
    const filterMount = mount.querySelector('#filter-mount');
    if (filterMount) filterMount.appendChild(fEl);

    const refreshBtn = mount.querySelector('#trades-refresh-btn');
    if (refreshBtn) refreshBtn.addEventListener('click', () =>
        window.dispatchEvent(new HashChangeEvent('hashchange')));
    mount.querySelector('#rollup-btn').addEventListener('click', async () => {
        try {
            await api.rollupTrades(state.accountId);
            if (!viewIsCurrent(tok)) return;
            showToast(t('view.trades.toast.rollup_done'), { level: 'success' });
            await refresh();
        } catch (e) {
            showToast(t('toast.error.api', { err: e.message }), { level: 'error' });
        }
    });
    mount.querySelector('#close-exp-btn').addEventListener('click', async () => {
        try {
            const n = await api.closeExpiredOptions(state.accountId);
            if (!viewIsCurrent(tok)) return;
            showToast(t('view.trades.alert.closed_expired', {
                n,
                label: t(n === 1 ? 'view.trades.label.trade_singular' : 'view.trades.label.trade_plural'),
            }), { level: n > 0 ? 'success' : 'info' });
            await refresh();
        } catch (e) {
            showToast(t('toast.error.api', { err: e.message }), { level: 'error' });
        }
    });

    mount.querySelector('#apply-bulk').addEventListener('click', async () => {
        const actEl = mount.querySelector('#bulk-action');
        const action = actEl ? actEl.value : '';
        if (!action) return;
        const ids = Array.from(mount.querySelectorAll('.trade-row input:checked'))
            .map(c => c.value);
        if (!ids.length) { showToast(t('view.trades.alert.select_first'), { level: 'error' }); return; }
        try {
            const extras = await collectActionExtras(action);
            if (!viewIsCurrent(tok)) return;
            if (extras === null) return; // cancelled
            const r = await api.bulkTrades(ids, action, extras);
            if (!viewIsCurrent(tok)) return;
            showToast(t('view.trades.alert.bulk_done', { action, affected: r.affected }), { level: 'success' });
            await refresh();
        } catch (e) {
            showToast(t('view.trades.alert.error', { msg: e.message }), { level: 'error' });
        }
    });

    async function collectActionExtras(action) {
        if (action === 'add_tag' || action === 'remove_tag') {
            const tags = await api.tags();
            if (!viewIsCurrent(tok)) return null;
            if (!tags.length) { showToast(t('view.trades.alert.no_tags'), { level: 'error' }); return null; }
            const name = await tPrompt(t('view.trades.prompt.tag_name', { names: tags.map(x => x.name).join(', ') }));
            if (!name) return null;
            const tag = tags.find(x => x.name.toLowerCase() === name.toLowerCase());
            if (!tag) { showToast(t('view.trades.alert.no_tag_named', { name }), { level: 'error' }); return null; }
            return { tag_id: tag.id };
        }
        if (action === 'set_risk') {
            const stop = await tPrompt('view.trades.prompt.stop', {});
            const risk = await tPrompt('view.trades.prompt.risk', {});
            const tgt = await tPrompt('view.trades.prompt.target', {});
            return {
                stop_loss: stop ? Number(stop) : null,
                risk_amount: risk ? Number(risk) : null,
                initial_target: tgt ? Number(tgt) : null,
            };
        }
        if (action === 'share') return { is_public: true };
        return {};
    }

    function applyViewMode() {
        const mode = getViewMode();
        const tableEl = mount.querySelector('#trades-table');
        const pagerEl = mount.querySelector('#trades-pagination');
        const pnlPanel = mount.querySelector('#trades-chart')?.closest('.chart-panel');
        const cumPanel = mount.querySelector('#trades-cum-chart')?.closest('.chart-panel');
        const showTable = mode === 'table';
        if (tableEl) tableEl.style.display = showTable ? '' : 'none';
        if (pagerEl) pagerEl.hidden = !showTable;
        // 'charts-large' keeps the chart height; 'charts-small' shrinks them.
        if (pnlPanel) {
            const inner = pnlPanel.querySelector('#trades-chart');
            if (inner) inner.className = mode === 'charts-small' ? 'chart-h-160' : 'chart-h-240';
        }
        if (cumPanel) {
            const inner = cumPanel.querySelector('#trades-cum-chart');
            if (inner) inner.className = mode === 'charts-small' ? 'chart-h-140' : 'chart-h-220';
        }
    }

    mount.querySelectorAll('.trades-view-toggle button[data-mode]').forEach(btn => {
        btn.addEventListener('click', () => {
            setViewMode(btn.dataset.mode);
            mount.querySelectorAll('.trades-view-toggle button').forEach(b =>
                b.classList.toggle('active', b.dataset.mode === btn.dataset.mode));
            applyViewMode();
        });
    });
    mount.querySelectorAll('.trades-pnl-toggle button[data-pnl]').forEach(btn => {
        btn.addEventListener('click', async () => {
            setPnlMode(btn.dataset.pnl);
            mount.querySelectorAll('.trades-pnl-toggle button').forEach(b =>
                b.classList.toggle('active', b.dataset.pnl === btn.dataset.pnl));
            await refresh();
        });
    });

    async function refresh() {
        const offset = (currentPage - 1) * PAGE_SIZE;
        let trades = await api.trades(state.accountId, {
            ...currentFilter, limit: PAGE_SIZE, offset,
        });
        lastPageRows = trades.length;
        if (sortKey) trades = sortTrades(trades, sortKey, sortDir);
        if (!viewIsCurrent(tok)) return;
        renderPager(mount);
        renderPnlChart(trades);
        renderCumChart(trades);
        applyViewMode();
        const tableEl = mount.querySelector('#trades-table');
        if (!tableEl) return;
        if (!trades.length) { tableEl.innerHTML = '<p data-i18n="view.trades.hint.no_trades_match" class="boot">No trades match.</p>'; return; }
        // Click headers to sort; clicking again toggles direction.
        const th = (key, labelKey, label) => {
            const active = sortKey === key;
            const arrow = active ? (sortDir === 'asc' ? ' ▲' : ' ▼') : '';
            return `<th data-sort-key="${esc(key)}" class="sortable${active ? ' active' : ''}"
                       data-i18n="${labelKey}">${esc(label)}${arrow}</th>`;
        };
        tableEl.innerHTML = `
            <table class="trades">
                <thead><tr>
                    <th class="col-checkbox"><input type="checkbox" id="sel-all"></th>
                    ${th('symbol',      'view.trades.th.symbol',  'Symbol')}
                    ${th('asset_class', 'view.trades.th.asset',   'Asset')}
                    ${th('side',        'view.trades.th.side',    'Side')}
                    ${th('status',      'view.trades.th.status',  'Status')}
                    ${th('qty',         'view.trades.th.qty',     'Qty')}
                    ${th('entry_avg',   'view.trades.th.entry',   'Entry')}
                    ${th('exit_avg',    'view.trades.th.exit',    'Exit')}
                    ${th('net_pnl', getPnlMode() === 'gross' ? 'view.trades.th.gross_p_l' : 'view.trades.th.net_p_l', getPnlMode() === 'gross' ? 'Gross P&L' : 'Net P&L')}
                    ${th('r_multiple',  'view.trades.th.r',       'R')}
                    ${th('hold',        'view.trades.th.hold',    'Hold')}
                    ${th('opened_at',   'view.trades.th.opened',  'Opened')}
                    ${th('closed_at',   'view.trades.th.closed',  'Closed')}
                    <th></th>
                </tr></thead>
                <tbody>${trades.map(t => `
                    <tr class="trade-row" data-id="${t.id}" data-context-scope="trade-row">
                        <td><input type="checkbox" value="${t.id}"></td>
                        <td><a href="#trade/${t.id}">${esc(t.symbol)}</a></td>
                        <td>${esc(t.asset_class)}</td>
                        <td>${t.side}</td>
                        <td>${t.status}</td>
                        <td>${fmt(t.qty, 0)}</td>
                        <td>${fmt(t.entry_avg)}</td>
                        <td>${t.exit_avg !== null ? fmt(t.exit_avg) : '—'}</td>
                        <td class="${pnlClass(getPnlMode() === 'gross' ? t.gross_pnl : t.net_pnl)}">${(getPnlMode() === 'gross' ? t.gross_pnl : t.net_pnl) != null ? fmtMoney(getPnlMode() === 'gross' ? t.gross_pnl : t.net_pnl) : '—'}</td>
                        <td>${t.r_multiple ?? '—'}</td>
                        <td>${fmtSecs(holdSeconds(t))}</td>
                        <td>${fmtDateTime(t.opened_at)}</td>
                        <td>${t.closed_at ? fmtDateTime(t.closed_at) : 'open'}</td>
                        <td><button data-i18n="view.trades.btn.delete" class="link" data-del="${t.id}">delete</button></td>
                    </tr>`).join('')}
                </tbody>
                ${tradesFooter(trades)}
            </table>
            <p class="muted">${trades.length} trade${trades.length === 1 ? '' : 's'}</p>
        `;
        const updateSel = () => {
            const n = mount.querySelectorAll('.trade-row input:checked').length;
            const cEl = mount.querySelector('#sel-count');
            const aEl = mount.querySelector('#apply-bulk');
            if (cEl) cEl.textContent = t('view.trades.label.n_selected', { n });
            if (aEl) aEl.disabled = n === 0;
        };
        tableEl.querySelectorAll('.trade-row input').forEach(c =>
            c.addEventListener('change', updateSel));
        const selAll = mount.querySelector('#sel-all');
        if (selAll) selAll.addEventListener('change', (e) => {
            tableEl.querySelectorAll('.trade-row input').forEach(c => c.checked = e.target.checked);
            updateSel();
        });
        tableEl.querySelectorAll('tr[data-id]').forEach(tr => {
            tr.addEventListener('dblclick', () => go('trade', tr.dataset.id));
        });
        tableEl.querySelectorAll('th.sortable').forEach(th => {
            th.addEventListener('click', async () => {
                const key = th.dataset.sortKey;
                if (sortKey === key) {
                    sortDir = sortDir === 'asc' ? 'desc' : 'asc';
                } else {
                    sortKey = key;
                    sortDir = 'desc';
                }
                await refresh();
            });
        });

        // Trello-style column reorder. Persists per-column order to
        // localStorage and reorders matching cells in each tbody row.
        const headRow = tableEl.querySelector('thead tr');
        if (headRow) {
            resetDragReorder(headRow);
            initDragReorder(headRow, 'th', 'trades_column_order', {
                direction: 'horizontal',
                getKey: (el) => el.dataset.sortKey || el.textContent.trim().slice(0, 12),
                onReorder: () => reorderBodyCellsToHead(tableEl),
                toastMessage: t('toast.reordered_columns'),
            });
            // Also reorder body cells once on init so a saved order applies.
            reorderBodyCellsToHead(tableEl);
        }
        tableEl.querySelectorAll('[data-del]').forEach(b =>
            b.addEventListener('click', async (e) => {
                e.stopPropagation();
                if (!await tConfirm('view.trades.confirm.delete', {}, { level: 'danger' })) return;
                await api.deleteTrade(b.dataset.del);
                if (!viewIsCurrent(tok)) return;
                await refresh();
            }));
    }
    const prev = mount.querySelector('#page-prev');
    const next = mount.querySelector('#page-next');
    if (prev) prev.addEventListener('click', async () => {
        if (currentPage > 1) { currentPage--; await refresh(); window.scrollTo({ top: 0 }); }
    });
    if (next) next.addEventListener('click', async () => {
        if (lastPageRows === PAGE_SIZE) { currentPage++; await refresh(); window.scrollTo({ top: 0 }); }
    });
    await refresh();
}

function renderPager(mount) {
    const wrap = mount.querySelector('#trades-pagination');
    const info = mount.querySelector('#page-info');
    const prev = mount.querySelector('#page-prev');
    const next = mount.querySelector('#page-next');
    if (!wrap || !info || !prev || !next) return;
    const showing = currentPage > 1 || lastPageRows === PAGE_SIZE;
    wrap.hidden = !showing;
    info.textContent = t('view.trades.pagination.page_of', {
        page: currentPage,
        total: lastPageRows === PAGE_SIZE ? `${currentPage}+` : currentPage,
    });
    prev.disabled = currentPage <= 1;
    next.disabled = lastPageRows < PAGE_SIZE;
}

function holdSeconds(t) {
    if (!t.closed_at) return null;
    return Math.round((new Date(t.closed_at) - new Date(t.opened_at)) / 1000);
}

// Footer with TOTAL and AVERAGE rows over the rendered trades. Tradervue's
// trades table shows these pinned under the body; we mirror the pattern so
// users can spot the bottom-line P&L without scrolling back to summary
// cards. Columns that don't aggregate naturally (symbol, side, status, etc.)
// stay blank in the footer rows.
function tradesFooter(trades) {
    if (!Array.isArray(trades) || trades.length === 0) return '';
    let totalQty = 0, totalNet = 0, holdSum = 0, holdN = 0, netN = 0;
    for (const tr of trades) {
        if (Number.isFinite(Number(tr.qty))) totalQty += Number(tr.qty);
        if (tr.net_pnl !== null && tr.net_pnl !== undefined) {
            totalNet += Number(tr.net_pnl) || 0;
            netN++;
        }
        const h = holdSeconds(tr);
        if (h !== null) { holdSum += h; holdN++; }
    }
    const avgQty = totalQty / trades.length;
    const avgNet = netN ? totalNet / netN : 0;
    const avgHold = holdN ? holdSum / holdN : null;
    return `
        <tfoot class="trades-foot">
            <tr class="trades-foot-total">
                <td></td>
                <td colspan="4">${esc(t('view.trades.foot.total'))}</td>
                <td>${fmt(totalQty, 0)}</td>
                <td></td><td></td>
                <td class="${pnlClass(totalNet)}">${fmtMoney(totalNet)}</td>
                <td></td><td></td><td></td><td></td><td></td>
            </tr>
            <tr class="trades-foot-avg">
                <td></td>
                <td colspan="4">${esc(t('view.trades.foot.average'))}</td>
                <td>${fmt(avgQty, 0)}</td>
                <td></td><td></td>
                <td class="${pnlClass(avgNet)}">${fmtMoney(avgNet)}</td>
                <td></td>
                <td>${avgHold !== null ? fmtSecs(avgHold) : '—'}</td>
                <td></td><td></td><td></td>
            </tr>
        </tfoot>
    `;
}

/**
 * After the user reorders `<th>` cells via drag, reshuffle each `<tr>` in
 * tbody so cells match the new header order. We build an index map from the
 * pre-reorder DOM order via data-orig-idx on every cell.
 */
function reorderBodyCellsToHead(tableEl) {
    const headRow = tableEl.querySelector('thead tr');
    const tbody = tableEl.querySelector('tbody');
    if (!headRow || !tbody) return;
    // Assign stable indices on first call.
    [...headRow.children].forEach((th, i) => {
        if (th.dataset.origIdx == null) th.dataset.origIdx = String(i);
    });
    const newOrder = [...headRow.children].map(th => Number(th.dataset.origIdx));
    // Stamp the rows' original cell indices once.
    for (const row of tbody.rows) {
        [...row.cells].forEach((td, i) => {
            if (td.dataset.origIdx == null) td.dataset.origIdx = String(i);
        });
        const byIdx = new Map([...row.cells].map(td => [Number(td.dataset.origIdx), td]));
        const frag = document.createDocumentFragment();
        for (const idx of newOrder) {
            const cell = byIdx.get(idx);
            if (cell) frag.appendChild(cell);
        }
        // Append any cells not covered (defensive).
        for (const td of row.cells) frag.appendChild(td);
        row.appendChild(frag);
    }
}

/**
 * Client-side sort for the current page of trades. We don't sort across
 * the entire dataset — pagination already constrained the rows the user
 * sees. 'hold' is a derived field so it doesn't exist on the row.
 */
function sortTrades(rows, key, dir) {
    const mult = dir === 'asc' ? 1 : -1;
    const get = (r) => {
        if (key === 'hold') return holdSeconds(r) ?? -Infinity;
        const v = r[key];
        if (v == null) return null;
        if (key === 'opened_at' || key === 'closed_at') return new Date(v).getTime();
        const n = Number(v);
        return Number.isFinite(n) ? n : String(v).toLowerCase();
    };
    return [...rows].sort((a, b) => {
        const av = get(a), bv = get(b);
        if (av === bv) return 0;
        if (av === null || av === undefined) return 1;   // nulls last
        if (bv === null || bv === undefined) return -1;
        return av < bv ? -1 * mult : 1 * mult;
    });
}

function renderCumChart(trades) {
    const el = document.getElementById('trades-cum-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const closed = (trades || [])
        .filter(tr => tr.closed_at && Number.isFinite(Number(tr.net_pnl)))
        .sort((a, b) => new Date(a.closed_at) - new Date(b.closed_at));
    if (closed.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.trades.empty_cum_chart">${esc(t('view.trades.empty_cum_chart'))}</div>`;
        return;
    }
    let acc = 0;
    const cum = closed.map(tr => (acc += Number(tr.net_pnl)));
    const xs = cum.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false }, y: { auto: true } },
        series: [
            { label: t('view.trades.chart.trade_idx') },
            { label: t('view.trades.chart.cum_pnl'),
              stroke: '#b86bff', width: 1.6, points: { show: false } },
            { label: t('view.trades.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, cum, zero], el);
}

function renderPnlChart(trades) {
    const el = document.getElementById('trades-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const closed = (trades || [])
        .filter(tr => tr.closed_at && Number.isFinite(Number(tr.net_pnl)))
        .sort((a, b) => new Date(a.closed_at) - new Date(b.closed_at));
    if (closed.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.trades.empty_chart">${esc(t('view.trades.empty_chart'))}</div>`;
        return;
    }
    const labels = closed.map(tr => tr.symbol);
    const ys = closed.map(tr => Number(tr.net_pnl));
    const xs = labels.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { time: false }, y: { auto: true } },
        series: [
            { label: t('view.trades.chart.trade_idx') },
            { label: t('view.trades.chart.pnl'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 8, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.trades.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, ys, zero], el);
}
