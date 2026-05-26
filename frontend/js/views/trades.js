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
            <button class="primary" id="close-exp-btn" style="background:linear-gradient(180deg,var(--magenta),#7f00b5);border-color:var(--magenta)">Close expired options</button>
            <span class="muted" id="sel-count" style="margin-left:14px">0 selected</span>
            <select id="bulk-action" style="width:auto;min-width:140px;display:inline-block">
                <option value="">— bulk action —</option>
                <option value="delete">Delete</option>
                <option value="merge">Merge into one</option>
                <option value="split">Split (re-FIFO)</option>
                <option value="add_tag">Add tag…</option>
                <option value="remove_tag">Remove tag…</option>
                <option value="set_risk">Set risk amount…</option>
                <option value="share">Share publicly</option>
            </select>
            <button class="primary" id="apply-bulk" disabled>Apply</button>
        </div>
        <div id="trades-table"></div>
    `;
    const { el: fEl } = makeFilter(currentFilter, async (f) => {
        currentFilter = f;
        await refresh();
    });
    document.getElementById('filter-mount').appendChild(fEl);

    document.getElementById('rollup-btn').addEventListener('click', async () => {
        await api.rollupTrades(state.accountId);
        await refresh();
    });
    document.getElementById('close-exp-btn').addEventListener('click', async () => {
        const n = await api.closeExpiredOptions(state.accountId);
        alert(`Closed ${n} expired option trade${n === 1 ? '' : 's'}.`);
        await refresh();
    });

    document.getElementById('apply-bulk').addEventListener('click', async () => {
        const action = document.getElementById('bulk-action').value;
        if (!action) return;
        const ids = Array.from(document.querySelectorAll('.trade-row input:checked'))
            .map(c => c.value);
        if (!ids.length) { alert('Select trades first.'); return; }
        try {
            const extras = await collectActionExtras(action);
            if (extras === null) return; // cancelled
            const r = await api.bulkTrades(ids, action, extras);
            alert(`${action} → affected ${r.affected}`);
            await refresh();
        } catch (e) {
            alert('Error: ' + e.message);
        }
    });

    async function collectActionExtras(action) {
        if (action === 'add_tag' || action === 'remove_tag') {
            const tags = await api.tags();
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
        const tableEl = document.getElementById('trades-table');
        if (!trades.length) { tableEl.innerHTML = '<p class="boot">No trades match.</p>'; return; }
        tableEl.innerHTML = `
            <table class="trades">
                <thead><tr>
                    <th style="width:28px"><input type="checkbox" id="sel-all"></th>
                    <th>Symbol</th><th>Asset</th><th>Side</th><th>Status</th>
                    <th>Qty</th><th>Entry</th><th>Exit</th>
                    <th>Net P&L</th><th>R</th>
                    <th>Hold</th><th>Opened</th><th>Closed</th><th></th>
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
                        <td><button class="link" data-del="${t.id}">delete</button></td>
                    </tr>`).join('')}
                </tbody>
            </table>
            <p class="muted">${trades.length} trade${trades.length === 1 ? '' : 's'}</p>
        `;
        const updateSel = () => {
            const n = document.querySelectorAll('.trade-row input:checked').length;
            document.getElementById('sel-count').textContent = `${n} selected`;
            document.getElementById('apply-bulk').disabled = n === 0;
        };
        tableEl.querySelectorAll('.trade-row input').forEach(c =>
            c.addEventListener('change', updateSel));
        document.getElementById('sel-all').addEventListener('change', (e) => {
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
                await refresh();
            }));
    }
    await refresh();
}

function holdSeconds(t) {
    if (!t.closed_at) return null;
    return Math.round((new Date(t.closed_at) - new Date(t.opened_at)) / 1000);
}
