// Main entry. Wires tabs, runs auth bootstrap, dispatches to view modules.

import { api, initApi, ApiError } from './api.js';
import { showAuthScreen, hideAuthScreen } from './auth.js';
import { equityChart } from './charts.js';
import { renderTradesView } from './trades.js';
import { renderImportView } from './import.js';
import { renderJournalView } from './journal.js';
import { renderExpensesView } from './expenses.js';

const state = {
    mode: 'web',
    accountId: null,
};

async function boot() {
    await initApi();
    try {
        const cfg = await api.config();
        state.mode = cfg.mode;
    } catch (_) { /* server may not be reachable yet */ }
    try {
        const me = await api.me();
        document.getElementById('user-strip').textContent =
            me.is_local ? 'local user' : (me.email || me.display_name || '');
        await loadAccounts();
        showView('dashboard');
        hideAuthScreen();
    } catch (e) {
        if (e instanceof ApiError && e.status === 401 && state.mode === 'web') {
            showAuthScreen();
        } else {
            document.getElementById('app').innerHTML =
                `<p class="boot">Failed to connect: ${e.message}</p>`;
        }
    }
}

async function loadAccounts() {
    const accounts = await api.accounts();
    if (accounts.length) state.accountId = accounts[0].id;
}

function bindTabs() {
    document.querySelectorAll('.tab').forEach(btn => {
        btn.addEventListener('click', () => {
            document.querySelectorAll('.tab').forEach(b => b.classList.toggle('active', b === btn));
            showView(btn.dataset.view);
        });
    });
}

async function showView(view) {
    const mount = document.getElementById('app');
    mount.innerHTML = '<div class="boot">loading…</div>';
    try {
        if (view === 'dashboard') await renderDashboard(mount);
        else if (view === 'trades') await renderTradesView(mount, state.accountId);
        else if (view === 'journal') await renderJournalView(mount);
        else if (view === 'import') renderImportView(mount);
        else if (view === 'expenses') await renderExpensesView(mount);
        else if (view === 'accounts') await renderAccounts(mount);
    } catch (e) {
        mount.innerHTML = `<p class="boot">Error: ${e.message}</p>`;
    }
}

async function renderDashboard(mount) {
    if (!state.accountId) {
        mount.innerHTML = '<p class="boot">No account. Go to Import to bring in broker data.</p>';
        return;
    }
    const [summary, equity] = await Promise.all([
        api.summary(state.accountId),
        api.equity(state.accountId),
    ]);
    const pos = n => Number(n) >= 0 ? 'pos' : 'neg';
    mount.innerHTML = `
        <div class="cards">
            <div class="card"><div class="label">Net P&L</div>
                <div class="value ${pos(summary.net_pnl)}">${fmt(summary.net_pnl)}</div></div>
            <div class="card"><div class="label">Trades</div>
                <div class="value">${summary.trade_count}</div></div>
            <div class="card"><div class="label">Win rate</div>
                <div class="value">${(summary.win_rate * 100).toFixed(1)}%</div></div>
            <div class="card"><div class="label">Profit factor</div>
                <div class="value">${Number.isFinite(summary.profit_factor) ? summary.profit_factor.toFixed(2) : '∞'}</div></div>
            <div class="card"><div class="label">Expectancy</div>
                <div class="value ${pos(summary.expectancy)}">${fmt(summary.expectancy)}</div></div>
            <div class="card"><div class="label">Fees</div>
                <div class="value">${fmt(summary.fees)}</div></div>
        </div>
        <div class="chart-panel">
            <h2>Equity Curve</h2>
            <div id="equity-chart"></div>
        </div>`;
    equityChart(document.getElementById('equity-chart'), equity);
}

async function renderAccounts(mount) {
    const accounts = await api.accounts();
    if (!accounts.length) {
        mount.innerHTML = '<p class="boot">No accounts yet. Account creation UI lands in phase 4.</p>';
        return;
    }
    mount.innerHTML = `
        <table class="trades">
            <thead><tr><th>Broker</th><th>Name</th><th>Currency</th><th>Created</th></tr></thead>
            <tbody>${accounts.map(a => `
                <tr>
                    <td>${a.broker}</td>
                    <td>${a.name}</td>
                    <td>${a.base_currency}</td>
                    <td>${a.created_at.slice(0, 10)}</td>
                </tr>`).join('')}
            </tbody>
        </table>`;
}

function fmt(n) {
    const v = Number(n);
    return v.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}

window.addEventListener('tv:authed', () => boot());
document.addEventListener('DOMContentLoaded', () => {
    bindTabs();
    boot();
});
