// Debt avalanche payoff planner. Pay minimums on every debt, route
// all extra to the HIGHEST-APR debt — mathematically optimal for
// total interest minimisation. When a debt clears, roll its minimum
// onto the next-highest-APR debt.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

const STATE = {
    debts: [
        { name: 'Credit card A',  balance_usd: 5000,  apr_pct: 22.99, min_payment_usd: 150 },
        { name: 'Credit card B',  balance_usd: 2500,  apr_pct: 19.49, min_payment_usd: 75 },
        { name: 'Auto loan',      balance_usd: 15000, apr_pct: 6.5,   min_payment_usd: 350 },
        { name: 'Student loan',   balance_usd: 25000, apr_pct: 5.0,   min_payment_usd: 270 },
    ],
    extra_payment_usd: 300,
};

export async function renderDebtAvalanche(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.debt_avalanche.title">// DEBT AVALANCHE</span></h1>
        <p class="muted small" data-i18n-html="view.debt_avalanche.intro">
            Pay minimums on every debt; route <strong>all extra</strong> payment to the
            <strong>highest-APR</strong> debt — mathematically optimal for total interest
            minimisation. When a debt clears, roll its minimum onto the next-highest-APR
            debt (snowball effect applied to avalanche ordering). Reports payoff month
            and interest paid per debt, plus aggregate total months and total interest.
        </p>
        <div class="chart-panel">
            <div style="display:flex;gap:8px;align-items:end;margin-bottom:12px">
                <label style="flex:0 0 240px">
                    <span class="muted small" data-i18n="view.debt_avalanche.field.extra">Extra payment $/mo</span>
                    <input type="number" id="da-extra" step="50" min="0" value="${STATE.extra_payment_usd}" style="width:100%">
                </label>
                <button class="btn btn-sm primary" id="da-run" data-shortcut="r" data-i18n="view.debt_avalanche.btn.run">⚡ Compute Payoff</button>
            </div>
            <h2>${esc(t('view.debt_avalanche.h2.debts'))}</h2>
            <table class="trades" id="da-table">
                <thead><tr>
                    <th data-i18n="view.debt_avalanche.th.name">Debt</th>
                    <th data-i18n="view.debt_avalanche.th.balance">Balance $</th>
                    <th data-i18n="view.debt_avalanche.th.apr">APR %</th>
                    <th data-i18n="view.debt_avalanche.th.min">Min $/mo</th>
                    <th></th>
                </tr></thead>
                <tbody id="da-body"></tbody>
            </table>
            <button class="btn btn-sm" id="da-add" data-i18n="view.debt_avalanche.btn.add">＋ Add debt</button>
            <div id="da-result"></div>
        </div>
    `;
    drawRows(mount);
    mount.querySelector('#da-extra').addEventListener('input', e => {
        STATE.extra_payment_usd = parseFloat(e.target.value) || 0;
    });
    mount.querySelector('#da-add').addEventListener('click', () => {
        STATE.debts.push({ name: 'New debt', balance_usd: 1000, apr_pct: 10, min_payment_usd: 50 });
        drawRows(mount);
    });
    mount.querySelector('#da-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

function drawRows(mount) {
    const body = mount.querySelector('#da-body');
    body.innerHTML = STATE.debts.map((d, i) => `
        <tr>
            <td><input type="text" data-k="name" data-i="${i}" value="${esc(d.name)}" style="width:100%"></td>
            <td><input type="number" step="100" min="0" data-k="balance_usd" data-i="${i}" value="${d.balance_usd}" style="width:100%"></td>
            <td><input type="number" step="0.25" min="0" max="100" data-k="apr_pct" data-i="${i}" value="${d.apr_pct}" style="width:100%"></td>
            <td><input type="number" step="10" min="0" data-k="min_payment_usd" data-i="${i}" value="${d.min_payment_usd}" style="width:100%"></td>
            <td><button class="btn btn-xs" data-del="${i}">✕</button></td>
        </tr>
    `).join('');
    body.querySelectorAll('input').forEach(inp => {
        inp.addEventListener('input', () => {
            const i = parseInt(inp.dataset.i, 10);
            const k = inp.dataset.k;
            STATE.debts[i][k] = k === 'name' ? inp.value : (parseFloat(inp.value) || 0);
        });
    });
    body.querySelectorAll('button[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            STATE.debts.splice(parseInt(btn.dataset.del, 10), 1);
            drawRows(mount);
        });
    });
}

async function runCompute(mount) {
    const result = mount.querySelector('#da-result');
    result.innerHTML = `<p class="muted">${esc(t('view.debt_avalanche.status.computing'))}</p>`;
    try {
        const r = await api('/debt-avalanche/compute', { method: 'POST', body: JSON.stringify(STATE) });
        const yearsMo = `${Math.floor(r.total_months / 12)}y ${r.total_months % 12}m`;
        const statusCls = r.all_paid_off ? 'pos' : 'neg';
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.debt_avalanche.field.total_months'))}</div>
                    <strong style="font-size:1.4em">${r.total_months}</strong>
                    <div class="muted small">${esc(yearsMo)}</div></div>
                <div><div class="muted small">${esc(t('view.debt_avalanche.field.total_principal'))}</div>
                    <strong>$${(r.total_principal_usd / 1000).toFixed(1)}K</strong></div>
                <div><div class="muted small">${esc(t('view.debt_avalanche.field.total_interest'))}</div>
                    <strong class="neg">$${(r.total_interest_paid_usd / 1000).toFixed(2)}K</strong></div>
                <div><div class="muted small">${esc(t('view.debt_avalanche.field.total_paid'))}</div>
                    <strong>$${(r.total_paid_usd / 1000).toFixed(1)}K</strong></div>
                <div><div class="muted small">${esc(t('view.debt_avalanche.field.status'))}</div>
                    <strong class="${statusCls}">${r.all_paid_off
                        ? esc(t('view.debt_avalanche.status.all_paid'))
                        : esc(t('view.debt_avalanche.status.not_paid'))}</strong></div>
            </div>
            <h2 style="margin-top:1rem">${esc(t('view.debt_avalanche.h2.per_debt'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.debt_avalanche.th.name">Debt</th>
                    <th data-i18n="view.debt_avalanche.th.balance_h">Balance</th>
                    <th data-i18n="view.debt_avalanche.th.apr_h">APR</th>
                    <th data-i18n="view.debt_avalanche.th.payoff_month">Payoff month</th>
                    <th data-i18n="view.debt_avalanche.th.interest">Interest paid</th>
                </tr></thead>
                <tbody>${(r.debts || []).sort((a, b) => (a.payoff_month || 9999) - (b.payoff_month || 9999)).map(d => `
                    <tr>
                        <td><strong>${esc(d.name)}</strong></td>
                        <td>$${d.starting_balance_usd.toFixed(0)}</td>
                        <td>${d.apr_pct.toFixed(2)}%</td>
                        <td>${d.payoff_month == null ? '∞' : d.payoff_month}</td>
                        <td class="neg">$${d.total_interest_paid_usd.toFixed(0)}</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
