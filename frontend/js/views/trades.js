import { api } from '../api.js';
import { esc, fmt, fmtMoney, fmtDateTime, fmtSecs, makeFilter, pnlClass } from '../util.js';
import { go, currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { tConfirm, tPrompt } from '../dialog.js';

let currentFilter = {};

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
            <button data-i18n="view.trades.btn.re_run_fifo" class="primary" id="rollup-btn">Re-run FIFO</button>
            <button data-i18n="view.trades.btn.close_expired_options" class="primary" id="close-exp-btn" style="background:linear-gradient(180deg,var(--magenta),#7f00b5);border-color:var(--magenta)">Close expired options</button>
            <span class="muted" id="sel-count" style="margin-left:14px">${esc(t('view.trades.label.n_selected', { n: 0 }))}</span>
            <select id="bulk-action" style="width:auto;min-width:140px;display:inline-block">
                <option data-i18n="view.trades.opt.bulk_action" value="">— bulk action —</option>
                <option data-i18n="view.trades.opt.delete" value="delete">Delete</option>
                <option data-i18n="view.trades.opt.merge_into_one" value="merge">Merge into one</option>
                <option data-i18n="view.trades.opt.split_re_fifo" value="split">Split (re-FIFO)</option>
                <option data-i18n="view.trades.opt.add_tag" value="add_tag">Add tag…</option>
                <option data-i18n="view.trades.opt.remove_tag" value="remove_tag">Remove tag…</option>
                <option data-i18n="view.trades.opt.set_risk_amount" value="set_risk">Set risk amount…</option>
                <option data-i18n="view.trades.opt.share_publicly" value="share">Share publicly</option>
            </select>
            <button data-i18n="view.trades.btn.apply" class="primary" id="apply-bulk" disabled>Apply</button>
        </div>
        <div id="trades-table"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.trades.h2.pnl_chart">Net P&L by trade (closed only)</h2>
            <div id="trades-chart" style="width:100%;height:240px"></div>
        </div>
    `;
    const { el: fEl } = makeFilter(currentFilter, async (f) => {
        currentFilter = f;
        await refresh();
    });
    const filterMount = mount.querySelector('#filter-mount');
    if (filterMount) filterMount.appendChild(fEl);

    mount.querySelector('#rollup-btn').addEventListener('click', async () => {
        await api.rollupTrades(state.accountId);
        if (!viewIsCurrent(tok)) return;
        await refresh();
    });
    mount.querySelector('#close-exp-btn').addEventListener('click', async () => {
        const n = await api.closeExpiredOptions(state.accountId);
        if (!viewIsCurrent(tok)) return;
        showToast(t('view.trades.alert.closed_expired', {
            n,
            label: t(n === 1 ? 'view.trades.label.trade_singular' : 'view.trades.label.trade_plural'),
        }), { level: 'error' });
        await refresh();
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
            showToast(t('view.trades.alert.bulk_done', { action, affected: r.affected }), { level: 'error' });
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

    async function refresh() {
        const trades = await api.trades(state.accountId, currentFilter);
        if (!viewIsCurrent(tok)) return;
        renderPnlChart(trades);
        const tableEl = mount.querySelector('#trades-table');
        if (!tableEl) return;
        if (!trades.length) { tableEl.innerHTML = '<p data-i18n="view.trades.hint.no_trades_match" class="boot">No trades match.</p>'; return; }
        tableEl.innerHTML = `
            <table class="trades">
                <thead><tr>
                    <th style="width:28px"><input type="checkbox" id="sel-all"></th>
                    <th data-i18n="view.trades.th.symbol">Symbol</th><th data-i18n="view.trades.th.asset">Asset</th><th data-i18n="view.trades.th.side">Side</th><th data-i18n="view.trades.th.status">Status</th>
                    <th data-i18n="view.trades.th.qty">Qty</th><th data-i18n="view.trades.th.entry">Entry</th><th data-i18n="view.trades.th.exit">Exit</th>
                    <th data-i18n="view.trades.th.net_p_l">Net P&L</th><th>R</th>
                    <th data-i18n="view.trades.th.hold">Hold</th><th data-i18n="view.trades.th.opened">Opened</th><th data-i18n="view.trades.th.closed">Closed</th><th></th>
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
                        <td class="${pnlClass(t.net_pnl)}">${t.net_pnl !== null ? fmtMoney(t.net_pnl) : '—'}</td>
                        <td>${t.r_multiple ?? '—'}</td>
                        <td>${fmtSecs(holdSeconds(t))}</td>
                        <td>${fmtDateTime(t.opened_at)}</td>
                        <td>${t.closed_at ? fmtDateTime(t.closed_at) : 'open'}</td>
                        <td><button data-i18n="view.trades.btn.delete" class="link" data-del="${t.id}">delete</button></td>
                    </tr>`).join('')}
                </tbody>
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
        tableEl.querySelectorAll('[data-del]').forEach(b =>
            b.addEventListener('click', async (e) => {
                e.stopPropagation();
                if (!await tConfirm('view.trades.confirm.delete', {}, { level: 'danger' })) return;
                await api.deleteTrade(b.dataset.del);
                if (!viewIsCurrent(tok)) return;
                await refresh();
            }));
    }
    await refresh();
}

function holdSeconds(t) {
    if (!t.closed_at) return null;
    return Math.round((new Date(t.closed_at) - new Date(t.opened_at)) / 1000);
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
        scales: { x: {}, y: { auto: true } },
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
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, ys, zero], el);
}
