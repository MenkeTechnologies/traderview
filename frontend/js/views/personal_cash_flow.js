// Personal cash-flow statement. GAAP-style operating / investing /
// financing split applied to a household. Reports section subtotals,
// net change in cash, savings rate, status.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

const STATE = {
    rows: [
        { name: 'Salary (net)',     category: 'operating', direction: 'inflow',  amount_usd: 6000 },
        { name: 'Side hustle',      category: 'operating', direction: 'inflow',  amount_usd: 400 },
        { name: 'Rent / mortgage',  category: 'operating', direction: 'outflow', amount_usd: 1800 },
        { name: 'Groceries',        category: 'operating', direction: 'outflow', amount_usd: 600 },
        { name: 'Utilities',        category: 'operating', direction: 'outflow', amount_usd: 200 },
        { name: 'Insurance',        category: 'operating', direction: 'outflow', amount_usd: 250 },
        { name: 'Dividends',        category: 'investing', direction: 'inflow',  amount_usd: 50 },
        { name: '401k contribution', category: 'investing', direction: 'outflow', amount_usd: 1000 },
        { name: 'IRA contribution', category: 'investing', direction: 'outflow', amount_usd: 500 },
        { name: 'Mortgage principal', category: 'financing', direction: 'outflow', amount_usd: 800 },
        { name: 'Credit card paydown', category: 'financing', direction: 'outflow', amount_usd: 200 },
    ],
};

export async function renderPersonalCashFlow(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.personal_cash_flow.title">// PERSONAL CASH FLOW STATEMENT</span></h1>
        <p class="muted small" data-i18n-html="view.personal_cash_flow.intro">
            GAAP-style cash-flow statement applied to a household — splits every transaction into
            <strong>operating</strong> (salary, recurring spend, mortgage interest, taxes),
            <strong>investing</strong> (401k / IRA / brokerage contributions, dividends, sales),
            <strong>financing</strong> (new debt, principal paydowns, gifts). Reports per-section
            subtotals, net change in cash, savings rate (operating net / operating inflows),
            and status.
        </p>
        <div class="chart-panel">
            <h2>${esc(t('view.personal_cash_flow.h2.rows'))}</h2>
            <table class="trades" id="pcf-rows-table">
                <thead><tr>
                    <th data-i18n="view.personal_cash_flow.th.name">Name</th>
                    <th data-i18n="view.personal_cash_flow.th.category">Category</th>
                    <th data-i18n="view.personal_cash_flow.th.direction">Direction</th>
                    <th data-i18n="view.personal_cash_flow.th.amount">Amount $</th>
                    <th></th>
                </tr></thead>
                <tbody id="pcf-rows-body"></tbody>
            </table>
            <button class="btn btn-sm" id="pcf-add" data-i18n="view.personal_cash_flow.btn.add">＋ Add row</button>
            <div style="margin-top:1rem">
                <button class="btn btn-sm primary" id="pcf-run" data-shortcut="r" data-i18n="view.personal_cash_flow.btn.run">⚡ Compute Cash Flow</button>
            </div>
            <div id="pcf-result"></div>
        </div>
    `;
    drawRows(mount);
    mount.querySelector('#pcf-add').addEventListener('click', () => {
        STATE.rows.push({ name: 'New row', category: 'operating', direction: 'outflow', amount_usd: 0 });
        drawRows(mount);
    });
    mount.querySelector('#pcf-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

function drawRows(mount) {
    const body = mount.querySelector('#pcf-rows-body');
    body.innerHTML = STATE.rows.map((r, i) => `
        <tr>
            <td><input type="text" data-k="name" data-i="${i}" value="${esc(r.name)}" style="width:100%"></td>
            <td>
                <select data-k="category" data-i="${i}">
                    <option value="operating" ${r.category === 'operating' ? 'selected' : ''}>Operating</option>
                    <option value="investing" ${r.category === 'investing' ? 'selected' : ''}>Investing</option>
                    <option value="financing" ${r.category === 'financing' ? 'selected' : ''}>Financing</option>
                </select>
            </td>
            <td>
                <select data-k="direction" data-i="${i}">
                    <option value="inflow" ${r.direction === 'inflow' ? 'selected' : ''}>Inflow</option>
                    <option value="outflow" ${r.direction === 'outflow' ? 'selected' : ''}>Outflow</option>
                </select>
            </td>
            <td><input type="number" step="50" min="0" data-k="amount_usd" data-i="${i}" value="${r.amount_usd}" style="width:100%"></td>
            <td><button class="btn btn-xs" data-del="${i}" data-tip="common.tip.remove_row" data-i18n-aria-label="common.aria.remove" aria-label="Remove">✕</button></td>
        </tr>
    `).join('');
    body.querySelectorAll('input,select').forEach(inp => {
        const ev = inp.tagName === 'SELECT' ? 'change' : 'input';
        inp.addEventListener(ev, () => {
            const i = parseInt(inp.dataset.i, 10);
            const k = inp.dataset.k;
            STATE.rows[i][k] = k === 'amount_usd' ? (parseFloat(inp.value) || 0) : inp.value;
        });
    });
    body.querySelectorAll('button[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            STATE.rows.splice(parseInt(btn.dataset.del, 10), 1);
            drawRows(mount);
        });
    });
}

async function runCompute(mount) {
    const result = mount.querySelector('#pcf-result');
    result.innerHTML = `<p class="muted">${esc(t('view.personal_cash_flow.status.computing'))}</p>`;
    try {
        const r = await api.request('/personal-cash-flow/compute', { method: 'POST', body: JSON.stringify(STATE) });
        const netCls = r.net_change_in_cash_usd > 0 ? 'pos' : r.net_change_in_cash_usd < 0 ? 'neg' : '';
        const statusCls = r.status === 'surplus' ? 'pos' : r.status === 'deficit' ? 'neg' : '';
        const section = (lbl, s) => `
            <tr>
                <td><strong>${esc(lbl)}</strong></td>
                <td class="pos">$${(s.inflows_usd / 1000).toFixed(2)}K</td>
                <td class="neg">$${(s.outflows_usd / 1000).toFixed(2)}K</td>
                <td class="${s.net_usd > 0 ? 'pos' : s.net_usd < 0 ? 'neg' : ''}"><strong>$${(s.net_usd / 1000).toFixed(2)}K</strong></td>
            </tr>
        `;
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.personal_cash_flow.field.net_change'))}</div>
                    <strong class="${netCls}" style="font-size:1.4em">$${(r.net_change_in_cash_usd / 1000).toFixed(2)}K</strong></div>
                <div><div class="muted small">${esc(t('view.personal_cash_flow.field.total_inflows'))}</div>
                    <strong class="pos">$${(r.total_inflows_usd / 1000).toFixed(2)}K</strong></div>
                <div><div class="muted small">${esc(t('view.personal_cash_flow.field.total_outflows'))}</div>
                    <strong class="neg">$${(r.total_outflows_usd / 1000).toFixed(2)}K</strong></div>
                <div><div class="muted small">${esc(t('view.personal_cash_flow.field.savings_rate'))}</div>
                    <strong>${r.savings_rate_pct == null ? '—' : r.savings_rate_pct.toFixed(1) + '%'}</strong></div>
                <div><div class="muted small">${esc(t('view.personal_cash_flow.field.status'))}</div>
                    <strong class="${statusCls}" style="text-transform:uppercase">${esc(t('view.personal_cash_flow.status.' + r.status) || r.status)}</strong></div>
            </div>
            <h2 style="margin-top:1rem">${esc(t('view.personal_cash_flow.h2.sections'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.personal_cash_flow.th.section">Section</th>
                    <th data-i18n="view.personal_cash_flow.th.inflows">Inflows</th>
                    <th data-i18n="view.personal_cash_flow.th.outflows">Outflows</th>
                    <th data-i18n="view.personal_cash_flow.th.net">Net</th>
                </tr></thead>
                <tbody>
                    ${section(t('view.personal_cash_flow.row.operating'), r.operating)}
                    ${section(t('view.personal_cash_flow.row.investing'), r.investing)}
                    ${section(t('view.personal_cash_flow.row.financing'), r.financing)}
                </tbody>
            </table>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
