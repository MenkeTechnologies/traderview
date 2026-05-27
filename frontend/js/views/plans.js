import { api } from '../api.js';
import { esc, fmt, fmtDateTime } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

export async function renderPlans(mount, state) {
    const tok = currentViewToken();
    if (!state.accountId) { mount.innerHTML = '<p class="boot">No account.</p>'; return; }
    const plans = await api.plans();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title">// PRE-TRADE PLANS</h1>
        <div class="chart-panel">
            <h2>New plan</h2>
            <form id="plan-form" class="inline-form">
                <input name="symbol" placeholder="symbol" required>
                <select name="asset_class">
                    <option value="stock">stock</option>
                    <option value="option">option</option>
                    <option value="future">future</option>
                    <option value="forex">forex</option>
                </select>
                <select name="side"><option value="long">long</option><option value="short">short</option></select>
                <input name="intended_qty" type="number" step="any" placeholder="qty" required>
                <input name="intended_entry" type="number" step="any" placeholder="entry" required>
                <input name="stop_loss" type="number" step="any" placeholder="stop">
                <input name="initial_target" type="number" step="any" placeholder="target">
                <input name="setup_notes" placeholder="setup notes">
                <button class="primary" type="submit">Create</button>
            </form>
        </div>

        <table class="trades">
            <thead><tr>
                <th>Created</th><th>Symbol</th><th>Side</th><th>Qty</th>
                <th>Entry</th><th>Stop</th><th>Target</th><th>R:R</th><th>Setup</th><th></th>
            </tr></thead>
            <tbody>${plans.map(p => {
                const risk = p.stop_loss ? Math.abs(Number(p.intended_entry) - Number(p.stop_loss)) : null;
                const reward = p.initial_target ? Math.abs(Number(p.initial_target) - Number(p.intended_entry)) : null;
                const rr = risk && reward ? (reward / risk).toFixed(2) : '—';
                return `<tr>
                    <td>${fmtDateTime(p.created_at)}</td>
                    <td>${esc(p.symbol)}</td>
                    <td>${p.side}</td>
                    <td>${fmt(p.intended_qty, 0)}</td>
                    <td>${fmt(p.intended_entry)}</td>
                    <td>${p.stop_loss !== null ? fmt(p.stop_loss) : '—'}</td>
                    <td>${p.initial_target !== null ? fmt(p.initial_target) : '—'}</td>
                    <td>${rr}</td>
                    <td>${esc(p.setup_notes)}</td>
                    <td><button class="link" data-del="${p.id}">abandon</button></td>
                </tr>`;
            }).join('') || '<tr><td colspan="10" class="muted">No pending plans.</td></tr>'}
            </tbody>
        </table>
    `;
    mount.querySelector('#plan-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const body = {
            account_id: state.accountId,
            symbol: fd.get('symbol'),
            asset_class: fd.get('asset_class'),
            side: fd.get('side'),
            intended_qty: Number(fd.get('intended_qty')),
            intended_entry: Number(fd.get('intended_entry')),
            stop_loss: fd.get('stop_loss') ? Number(fd.get('stop_loss')) : null,
            initial_target: fd.get('initial_target') ? Number(fd.get('initial_target')) : null,
            setup_notes: fd.get('setup_notes') || '',
        };
        await api.createPlan(body);
        if (!viewIsCurrent(tok)) return;
        renderPlans(mount, state);
    });
    mount.querySelectorAll('[data-del]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.abandonPlan(b.dataset.del);
            if (!viewIsCurrent(tok)) return;
            renderPlans(mount, state);
        }));
}
