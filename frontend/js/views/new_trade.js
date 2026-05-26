// "New Trade" — one or more manual executions, posted to /executions.
// Each insert triggers a server-side rollup so trades auto-form via FIFO.
import { api } from '../api.js';
import { go } from '../app.js';
import { fmt } from '../util.js';

export async function renderNewTrade(mount, state) {
    if (!state.accountId) {
        mount.innerHTML = '<p class="boot">Create an account first (Accounts tab).</p>';
        return;
    }
    mount.innerHTML = `
        <h1 class="view-title">// NEW TRADE</h1>
        <p class="muted small">Add executions one at a time. The server FIFO-folds each one into the matching open trade (or starts a new one).</p>

        <div class="chart-panel">
            <h2>Add execution</h2>
            <form id="ex-form" class="inline-form">
                <input name="symbol" placeholder="symbol" required>
                <select name="side">
                    <option value="buy">buy</option>
                    <option value="sell">sell</option>
                    <option value="short">short</option>
                    <option value="cover">cover</option>
                </select>
                <input name="qty" type="number" step="any" placeholder="qty" required>
                <input name="price" type="number" step="any" placeholder="price" required>
                <input name="fee" type="number" step="any" placeholder="fee" value="0">
                <input name="executed_at" type="datetime-local" required>
                <select name="asset_class">
                    <option value="stock">stock</option>
                    <option value="option">option</option>
                    <option value="future">future</option>
                    <option value="forex">forex</option>
                </select>
                <select name="option_type" style="display:none">
                    <option value="">—</option>
                    <option value="call">call</option>
                    <option value="put">put</option>
                </select>
                <input name="strike"     type="number" step="any" placeholder="strike" style="display:none">
                <input name="expiration" type="date"   placeholder="exp" style="display:none">
                <input name="multiplier" type="number" step="any" placeholder="multiplier" style="display:none">
                <button class="primary" type="submit">Add</button>
            </form>
        </div>

        <div class="chart-panel">
            <h2>Recent executions on this account</h2>
            <div id="recent-execs"></div>
        </div>
    `;

    // Default executed_at to now (local time).
    const now = new Date();
    const pad = (n) => String(n).padStart(2, '0');
    document.querySelector('[name=executed_at]').value =
        `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}T` +
        `${pad(now.getHours())}:${pad(now.getMinutes())}`;

    const form = document.getElementById('ex-form');
    const assetSelect = form.querySelector('[name=asset_class]');
    const optionFields = ['option_type', 'strike', 'expiration', 'multiplier'];
    const syncOption = () => {
        const isOpt = assetSelect.value === 'option' || assetSelect.value === 'future';
        optionFields.forEach(n => {
            form.querySelector(`[name=${n}]`).style.display = isOpt ? '' : 'none';
        });
    };
    assetSelect.addEventListener('change', syncOption);
    syncOption();

    form.addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const body = {
            account_id: state.accountId,
            symbol: fd.get('symbol').trim().toUpperCase(),
            side: fd.get('side'),
            qty: Number(fd.get('qty')),
            price: Number(fd.get('price')),
            fee: Number(fd.get('fee') || 0),
            executed_at: new Date(fd.get('executed_at')).toISOString(),
            asset_class: fd.get('asset_class'),
        };
        if (body.asset_class === 'option') {
            body.option_type = fd.get('option_type') || null;
            body.strike      = fd.get('strike')      ? Number(fd.get('strike'))      : null;
            body.expiration  = fd.get('expiration')  || null;
            body.multiplier  = fd.get('multiplier')  ? Number(fd.get('multiplier'))  : 100;
        }
        try {
            await api.createExecution(body);
            await refresh();
            e.target.reset();
            // restore the executed_at default
            document.querySelector('[name=executed_at]').value =
                `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}T` +
                `${pad(now.getHours())}:${pad(now.getMinutes())}`;
            syncOption();
        } catch (err) { alert('Error: ' + err.message); }
    });

    async function refresh() {
        const execs = (await api.executions(state.accountId)).slice(-20).reverse();
        document.getElementById('recent-execs').innerHTML = execs.length ? `
            <table class="trades">
                <thead><tr><th>Time</th><th>Symbol</th><th>Side</th>
                    <th>Qty</th><th>Price</th><th>Fee</th></tr></thead>
                <tbody>${execs.map(e => `
                    <tr><td>${new Date(e.executed_at).toLocaleString(undefined, { hour12: false })}</td>
                    <td>${e.symbol}</td><td>${e.side}</td>
                    <td>${fmt(e.qty, 0)}</td><td>${fmt(e.price)}</td><td>${fmt(e.fee)}</td></tr>
                `).join('')}</tbody>
            </table>
            <button class="primary" id="open-trades">View trades</button>
        ` : '<p class="muted">No executions yet on this account.</p>';
        const btn = document.getElementById('open-trades');
        if (btn) btn.addEventListener('click', () => go('trades'));
    }
    refresh();
}
