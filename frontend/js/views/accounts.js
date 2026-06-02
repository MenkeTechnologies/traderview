import { api } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';
import { tConfirm } from '../dialog.js';
import { showToast } from '../toast.js';

export async function renderAccounts(mount, _state, onChange) {
    const tok = currentViewToken();
    const accounts = await api.accounts();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 data-i18n="view.accounts.h1.accounts" class="view-title">// ACCOUNTS</h1>
        <div class="chart-panel">
            <h2 data-i18n="view.accounts.h2.add_account">Add account</h2>
            <form id="acct-form" class="inline-form">
                <select name="broker" data-tip="view.accounts.tip.broker">
                    <option data-i18n="view.accounts.opt.webull" value="webull">Webull</option>
                    <option data-i18n="view.accounts.opt.interactive_brokers_flex" value="ibkr">Interactive Brokers (Flex)</option>
                    <option data-i18n="view.accounts.opt.td_ameritrade" value="tdameritrade">TD Ameritrade</option>
                    <option data-i18n="view.accounts.opt.schwab" value="schwab">Schwab</option>
                    <option data-i18n="view.accounts.opt.tradestation" value="tradestation">TradeStation</option>
                    <option data-i18n="view.accounts.opt.lightspeed" value="lightspeed">Lightspeed</option>
                    <option data-i18n="view.accounts.opt.das_trader" value="das">DAS Trader</option>
                    <option data-i18n="view.accounts.opt.thinkorswim" value="tos">ThinkOrSwim</option>
                    <option data-i18n="view.accounts.opt.e_trade" value="etrade">E*TRADE</option>
                    <option data-i18n="view.accounts.opt.fidelity" value="fidelity">Fidelity</option>
                    <option data-i18n="view.accounts.opt.tradezero" value="tradezero">TradeZero</option>
                    <option data-i18n="view.accounts.opt.robinhood" value="robinhood">Robinhood</option>
                    <option data-i18n="view.accounts.opt.manual_other" value="manual">Manual / Other</option>
                </select>
                <input name="name" placeholder="account name (e.g. Margin)" data-i18n-placeholder="view.accounts.placeholder.name"
                       data-tip="view.accounts.tip.name" data-shortcut="accounts_focus_name" required>
                <input name="base_currency" placeholder="USD" value="USD" data-tip="view.accounts.tip.base_currency">
                <button data-i18n="view.accounts.btn.create" data-tip="view.accounts.tip.create" class="primary" type="submit">Create</button>
            </form>
        </div>

        <table class="trades">
            <thead><tr><th data-i18n="view.accounts.th.broker">Broker</th><th data-i18n="view.accounts.th.name">Name</th><th data-i18n="view.accounts.th.currency">Currency</th><th data-i18n="view.accounts.th.created">Created</th><th></th></tr></thead>
            <tbody>${accounts.map(a => `
                <tr data-context-scope="account-row" data-id="${esc(a.id)}" data-name="${esc(a.name)}"><td>${esc(a.broker)}</td><td>${esc(a.name)}</td>
                <td>${esc(a.base_currency)}</td>
                <td>${fmtDateTime(a.created_at)}</td>
                <td>
                    <button class="link" data-rebuild="${a.id}" data-i18n="view.accounts.btn.rebuild" style="margin-right:8px">rebuild trades</button>
                    <button data-i18n="view.accounts.btn.delete" class="link" data-del="${a.id}">delete</button>
                </td></tr>
            `).join('') || `<tr><td colspan="5" class="muted">${esc(t('view.accounts.empty'))}</td></tr>`}
            </tbody>
        </table>

        <div class="chart-panel">
            <h2 data-i18n="view.accounts.h2.broker_chart">Accounts by broker</h2>
            <div id="acct-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.accounts.h2.currency_chart">Accounts by currency</h2>
            <div id="acct-ccy-chart" style="width:100%;height:200px"></div>
        </div>
    `;
    renderBrokerChart(accounts);
    renderCurrencyChart(accounts);

    mount.querySelector('#acct-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const name = String(fd.get('name') || '').trim();
        try {
            await api.createAccount(fd.get('broker'), name, fd.get('base_currency'));
            if (!viewIsCurrent(tok)) return;
            showToast(t('view.accounts.toast.created', { name }), { level: 'success' });
            if (onChange) onChange();
            renderAccounts(mount, _state, onChange);
        } catch (err) {
            showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
        }
    });
    mount.querySelectorAll('[data-del]').forEach(b =>
        b.addEventListener('click', async () => {
            if (!await tConfirm('view.accounts.confirm.delete', {}, { level: 'danger' })) return;
            try {
                await api.deleteAccount(b.dataset.del);
                if (!viewIsCurrent(tok)) return;
                const tr = b.closest('tr');
                const name = tr?.dataset?.name || '';
                showToast(t('view.accounts.toast.deleted', { name }), { level: 'success' });
                if (onChange) onChange();
                renderAccounts(mount, _state, onChange);
            } catch (err) {
                showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
            }
        }));
    mount.querySelectorAll('[data-rebuild]').forEach(b =>
        b.addEventListener('click', async () => {
            try {
                b.disabled = true;
                const r = await api.rebuildTrades(b.dataset.rebuild);
                if (!viewIsCurrent(tok)) return;
                showToast(t('view.accounts.toast.rebuilt', { n: r.trades_rolled }), { level: 'success' });
            } catch (err) {
                showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
            } finally {
                b.disabled = false;
            }
        }));
}

function renderCurrencyChart(accounts) {
    const el = document.getElementById('acct-ccy-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    if (!accounts || !accounts.length) {
        el.innerHTML = `<div class="muted" data-i18n="view.accounts.empty_ccy_chart">${esc(t('view.accounts.empty_ccy_chart'))}</div>`;
        return;
    }
    const counts = new Map();
    for (const a of accounts) {
        const key = (a.base_currency || '?').toUpperCase();
        counts.set(key, (counts.get(key) || 0) + 1);
    }
    const pairs = Array.from(counts.entries()).sort((a, b) => b[1] - a[1]);
    const labels = pairs.map(([k]) => k);
    const ys = pairs.map(([, n]) => n);
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 180,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.accounts.chart.ccy_idx') },
            { label: t('view.accounts.chart.count'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 12, fill: '#7af0a8', stroke: '#7af0a8' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function renderBrokerChart(accounts) {
    const el = document.getElementById('acct-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    if (!accounts || !accounts.length) {
        el.innerHTML = `<div class="muted" data-i18n="view.accounts.empty_chart">${esc(t('view.accounts.empty_chart'))}</div>`;
        return;
    }
    const counts = new Map();
    for (const a of accounts) {
        const key = a.broker || '?';
        counts.set(key, (counts.get(key) || 0) + 1);
    }
    const pairs = Array.from(counts.entries()).sort((a, b) => b[1] - a[1]);
    const labels = pairs.map(([k]) => k);
    const ys = pairs.map(([, n]) => n);
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.accounts.chart.broker_idx') },
            { label: t('view.accounts.chart.count'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 12, fill: '#00e5ff', stroke: '#00e5ff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}
