import { api } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';
import { tConfirm } from '../dialog.js';

export async function renderAccounts(mount, _state, onChange) {
    const tok = currentViewToken();
    const accounts = await api.accounts();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 data-i18n="view.accounts.h1.accounts" class="view-title">// ACCOUNTS</h1>
        <div class="chart-panel">
            <h2 data-i18n="view.accounts.h2.add_account">Add account</h2>
            <form id="acct-form" class="inline-form">
                <select name="broker">
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
                <input name="name" placeholder="account name (e.g. Margin)" data-i18n-placeholder="view.accounts.placeholder.name" required>
                <input name="base_currency" placeholder="USD" value="USD">
                <button data-i18n="view.accounts.btn.create" class="primary" type="submit">Create</button>
            </form>
        </div>

        <table class="trades">
            <thead><tr><th data-i18n="view.accounts.th.broker">Broker</th><th data-i18n="view.accounts.th.name">Name</th><th data-i18n="view.accounts.th.currency">Currency</th><th data-i18n="view.accounts.th.created">Created</th><th></th></tr></thead>
            <tbody>${accounts.map(a => `
                <tr data-context-scope="account-row" data-id="${esc(a.id)}" data-name="${esc(a.name)}"><td>${esc(a.broker)}</td><td>${esc(a.name)}</td>
                <td>${esc(a.base_currency)}</td>
                <td>${fmtDateTime(a.created_at)}</td>
                <td><button data-i18n="view.accounts.btn.delete" class="link" data-del="${a.id}">delete</button></td></tr>
            `).join('') || `<tr><td colspan="5" class="muted">${esc(t('view.accounts.empty'))}</td></tr>`}
            </tbody>
        </table>

        <div class="chart-panel">
            <h2 data-i18n="view.accounts.h2.broker_chart">Accounts by broker</h2>
            <div id="acct-chart" style="width:100%;height:240px"></div>
        </div>
    `;
    renderBrokerChart(accounts);

    mount.querySelector('#acct-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        await api.createAccount(fd.get('broker'), fd.get('name'), fd.get('base_currency'));
        if (!viewIsCurrent(tok)) return;
        if (onChange) onChange();
        renderAccounts(mount, _state, onChange);
    });
    mount.querySelectorAll('[data-del]').forEach(b =>
        b.addEventListener('click', async () => {
            if (!await tConfirm('view.accounts.confirm.delete', {}, { level: 'danger' })) return;
            await api.deleteAccount(b.dataset.del);
            if (!viewIsCurrent(tok)) return;
            if (onChange) onChange();
            renderAccounts(mount, _state, onChange);
        }));
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
