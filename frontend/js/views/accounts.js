import { api } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

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
                <input name="name" placeholder="account name (e.g. Margin)" required>
                <input name="base_currency" placeholder="USD" value="USD">
                <button data-i18n="view.accounts.btn.create" class="primary" type="submit">Create</button>
            </form>
        </div>

        <table class="trades">
            <thead><tr><th data-i18n="view.accounts.th.broker">Broker</th><th data-i18n="view.accounts.th.name">Name</th><th data-i18n="view.accounts.th.currency">Currency</th><th data-i18n="view.accounts.th.created">Created</th><th></th></tr></thead>
            <tbody>${accounts.map(a => `
                <tr><td>${esc(a.broker)}</td><td>${esc(a.name)}</td>
                <td>${esc(a.base_currency)}</td>
                <td>${fmtDateTime(a.created_at)}</td>
                <td><button data-i18n="view.accounts.btn.delete" class="link" data-del="${a.id}">delete</button></td></tr>
            `).join('') || '<tr><td colspan="5" class="muted">No accounts.</td></tr>'}
            </tbody>
        </table>
    `;

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
            if (!confirm('Delete this account and all its trades?')) return;
            await api.deleteAccount(b.dataset.del);
            if (!viewIsCurrent(tok)) return;
            if (onChange) onChange();
            renderAccounts(mount, _state, onChange);
        }));
}
