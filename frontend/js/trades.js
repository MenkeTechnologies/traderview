import { api } from './api.js';

export async function renderTradesView(mount, accountId) {
    if (!accountId) {
        mount.innerHTML = '<p class="boot">No account yet. Import a broker file or create one.</p>';
        return;
    }
    const trades = await api.trades(accountId, 200, 0);
    if (!trades.length) {
        mount.innerHTML = '<p class="boot">No trades. Import broker data via the Import tab.</p>';
        return;
    }
    const rows = trades.map(t => {
        const pnl = t.net_pnl != null ? Number(t.net_pnl) : null;
        const cls = pnl == null ? '' : (pnl >= 0 ? 'pnl-pos' : 'pnl-neg');
        const pnlText = pnl == null ? '—' : pnl.toFixed(2);
        return `<tr>
            <td>${t.opened_at.slice(0, 16).replace('T', ' ')}</td>
            <td>${t.symbol}</td>
            <td>${t.side}</td>
            <td>${t.status}</td>
            <td>${Number(t.qty)}</td>
            <td>${Number(t.entry_avg).toFixed(4)}</td>
            <td>${t.exit_avg ? Number(t.exit_avg).toFixed(4) : '—'}</td>
            <td class="${cls}">${pnlText}</td>
        </tr>`;
    }).join('');
    mount.innerHTML = `
        <table class="trades">
            <thead>
                <tr>
                    <th>Opened</th><th>Symbol</th><th>Side</th><th>Status</th>
                    <th>Qty</th><th>Entry</th><th>Exit</th><th>Net P&L</th>
                </tr>
            </thead>
            <tbody>${rows}</tbody>
        </table>`;
}
