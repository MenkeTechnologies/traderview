import { api } from '../api.js';
import { esc, fmt, fmtMoney, fmtDateTime, fmtSecs, makeFilter, pnlClass } from '../util.js';
import { go } from '../app.js';

let currentFilter = {};

export async function renderTradesView(mount, state) {
    if (!state.accountId) {
        mount.innerHTML = '<p class="boot">No account.</p>';
        return;
    }
    mount.innerHTML = `
        <h1 class="view-title">// TRADES</h1>
        <div id="filter-mount"></div>
        <div class="trades-toolbar">
            <button class="primary" id="rollup-btn">Re-run FIFO</button>
        </div>
        <div id="trades-table"></div>
    `;
    const { el: fEl, collect } = makeFilter(currentFilter, async (f) => {
        currentFilter = f;
        await refresh();
    });
    document.getElementById('filter-mount').appendChild(fEl);

    document.getElementById('rollup-btn').addEventListener('click', async () => {
        await api.rollupTrades(state.accountId);
        await refresh();
    });

    async function refresh() {
        const trades = await api.trades(state.accountId, currentFilter);
        const tableEl = document.getElementById('trades-table');
        if (!trades.length) { tableEl.innerHTML = '<p class="boot">No trades match.</p>'; return; }
        tableEl.innerHTML = `
            <table class="trades">
                <thead><tr>
                    <th>Symbol</th><th>Asset</th><th>Side</th><th>Status</th>
                    <th>Qty</th><th>Entry</th><th>Exit</th>
                    <th>Net P&L</th><th>R</th>
                    <th>Hold</th><th>Opened</th><th>Closed</th>
                </tr></thead>
                <tbody>${trades.map(t => `
                    <tr data-id="${t.id}">
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
                    </tr>`).join('')}
                </tbody>
            </table>
            <p class="muted">${trades.length} trade${trades.length === 1 ? '' : 's'}</p>
        `;
        tableEl.querySelectorAll('tr[data-id]').forEach(tr => {
            tr.addEventListener('dblclick', () => go('trade', tr.dataset.id));
        });
    }
    await refresh();
    void collect; // referenced for symmetry
}

function holdSeconds(t) {
    if (!t.closed_at) return null;
    return Math.round((new Date(t.closed_at) - new Date(t.opened_at)) / 1000);
}
