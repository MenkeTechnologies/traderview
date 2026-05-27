import { api } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

export async function renderAccounts(mount, _state, onChange) {
    const tok = currentViewToken();
    const accounts = await api.accounts();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title">// ACCOUNTS</h1>
        <div class="chart-panel">
            <h2>Add account</h2>
            <form id="acct-form" class="inline-form">
                <select name="broker">
                    <option value="webull">Webull</option>
                    <option value="ibkr">Interactive Brokers (Flex)</option>
                    <option value="tdameritrade">TD Ameritrade</option>
                    <option value="schwab">Schwab</option>
                    <option value="tradestation">TradeStation</option>
                    <option value="lightspeed">Lightspeed</option>
                    <option value="das">DAS Trader</option>
                    <option value="tos">ThinkOrSwim</option>
                    <option value="etrade">E*TRADE</option>
                    <option value="fidelity">Fidelity</option>
                    <option value="tradezero">TradeZero</option>
                    <option value="robinhood">Robinhood</option>
                    <option value="manual">Manual / Other</option>
                </select>
                <input name="name" placeholder="account name (e.g. Margin)" required>
                <input name="base_currency" placeholder="USD" value="USD">
                <button class="primary" type="submit">Create</button>
            </form>
        </div>

        <table class="trades">
            <thead><tr><th>Broker</th><th>Name</th><th>Currency</th><th>Created</th><th></th></tr></thead>
            <tbody>${accounts.map(a => `
                <tr><td>${esc(a.broker)}</td><td>${esc(a.name)}</td>
                <td>${esc(a.base_currency)}</td>
                <td>${fmtDateTime(a.created_at)}</td>
                <td><button class="link" data-del="${a.id}">delete</button></td></tr>
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
