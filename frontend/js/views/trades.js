import { api } from '../api.js';
import { esc, fmt, fmtMoney, fmtDateTime, fmtSecs, makeFilter, pnlClass } from '../util.js';
import { go, currentViewToken, viewIsCurrent } from '../app.js';

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
            <button data-i18n="view.trades.btn.re_run_fifo" class="primary" id="rollup-btn">Re-run FIFO</button>
            <button data-i18n="view.trades.btn.close_expired_options" class="primary" id="close-exp-btn" style="background:linear-gradient(180deg,var(--magenta),#7f00b5);border-color:var(--magenta)">Close expired options</button>
            <span class="muted" id="sel-count" style="margin-left:14px">0 selected</span>
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
        alert(`Closed ${n} expired option trade${n === 1 ? '' : 's'}.`);
        await refresh();
    });

    mount.querySelector('#apply-bulk').addEventListener('click', async () => {
        const actEl = mount.querySelector('#bulk-action');
        const action = actEl ? actEl.value : '';
        if (!action) return;
        const ids = Array.from(mount.querySelectorAll('.trade-row input:checked'))
            .map(c => c.value);
        if (!ids.length) { alert('Select trades first.'); return; }
        try {
            const extras = await collectActionExtras(action);
            if (!viewIsCurrent(tok)) return;
            if (extras === null) return; // cancelled
            const r = await api.bulkTrades(ids, action, extras);
            if (!viewIsCurrent(tok)) return;
            alert(`${action} → affected ${r.affected}`);
            await refresh();
        } catch (e) {
            alert('Error: ' + e.message);
        }
    });

    async function collectActionExtras(action) {
        if (action === 'add_tag' || action === 'remove_tag') {
            const tags = await api.tags();
            if (!viewIsCurrent(tok)) return null;
            if (!tags.length) { alert('Create a tag first (Tags tab).'); return null; }
            const name = prompt(`Tag name (${tags.map(t => t.name).join(', ')})`);
            if (!name) return null;
            const tag = tags.find(t => t.name.toLowerCase() === name.toLowerCase());
            if (!tag) { alert(`No tag named "${name}".`); return null; }
            return { tag_id: tag.id };
        }
        if (action === 'set_risk') {
            const stop = prompt('Stop-loss price (blank = none):');
            const risk = prompt('Risk amount $ (blank = none):');
            const tgt = prompt('Initial target price (blank = none):');
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
                    <tr class="trade-row" data-id="${t.id}">
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
            if (cEl) cEl.textContent = `${n} selected`;
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
                if (!confirm('Delete this trade?')) return;
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
