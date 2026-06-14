// Debt snowball — Dave Ramsey's behavioural strategy: pay extra to the
// SMALLEST balance first regardless of APR, then roll its min into
// the next smallest. The early wins fuel adherence even though
// avalanche is mathematically optimal.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import * as enh from '../calc_enhance.js';

const STATE = {
    debts: [
        { name: 'Credit card A',  balance_usd: 5000,  apr_pct: 22.99, min_payment_usd: 150 },
        { name: 'Credit card B',  balance_usd: 2500,  apr_pct: 19.49, min_payment_usd: 75 },
        { name: 'Auto loan',      balance_usd: 15000, apr_pct: 6.5,   min_payment_usd: 350 },
        { name: 'Student loan',   balance_usd: 25000, apr_pct: 5.0,   min_payment_usd: 270 },
    ],
    extra_payment_usd: 300,
};

export async function renderDebtSnowball(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.debt_snowball.title">// DEBT SNOWBALL</span></h1>
        <p class="muted small" data-i18n-html="view.debt_snowball.intro">
            Dave Ramsey's behavioural strategy: pay minimums on every debt; route
            <strong>all extra</strong> to the <strong>smallest balance</strong> first
            regardless of APR. When a debt clears, roll its min onto the next-smallest.
            The early wins fuel adherence. Pure-math view says avalanche is optimal —
            but Northwestern's 2012 Gal & McShane study shows snowball users finish more
            often. Side-by-side these two views to see the interest-vs-adherence tradeoff.
        </p>
        <div class="chart-panel">
            <div style="display:flex;gap:8px;align-items:end;margin-bottom:12px">
                <label style="flex:0 0 240px">
                    <span class="muted small" data-i18n="view.debt_snowball.field.extra">Extra payment $/mo</span>
                    <input type="number" id="ds-extra" step="50" min="0" value="${STATE.extra_payment_usd}" style="width:100%">
                </label>
                <button class="btn btn-sm primary" id="ds-run" data-shortcut="r" data-i18n="view.debt_snowball.btn.run">⚡ Compute Payoff</button>
            </div>
            <h2>${esc(t('view.debt_snowball.h2.debts'))}</h2>
            <table class="trades" id="ds-table">
                <thead><tr>
                    <th data-i18n="view.debt_snowball.th.name">Debt</th>
                    <th data-i18n="view.debt_snowball.th.balance">Balance $</th>
                    <th data-i18n="view.debt_snowball.th.apr">APR %</th>
                    <th data-i18n="view.debt_snowball.th.min">Min $/mo</th>
                    <th></th>
                </tr></thead>
                <tbody id="ds-body"></tbody>
            </table>
            <button class="btn btn-sm" id="ds-add" data-i18n="view.debt_snowball.btn.add">＋ Add debt</button>
            <div id="ds-result"></div>
        </div>
    `;
    drawRows(mount);
    mount.querySelector('#ds-extra').addEventListener('input', e => {
        STATE.extra_payment_usd = parseFloat(e.target.value) || 0;
    });
    mount.querySelector('#ds-add').addEventListener('click', () => {
        STATE.debts.push({ name: 'New debt', balance_usd: 1000, apr_pct: 10, min_payment_usd: 50 });
        drawRows(mount);
    });
    mount.querySelector('#ds-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

function drawRows(mount) {
    const body = mount.querySelector('#ds-body');
    body.innerHTML = STATE.debts.map((d, i) => `
        <tr>
            <td><input type="text" data-k="name" data-i="${i}" value="${esc(d.name)}" style="width:100%"></td>
            <td><input type="number" step="100" min="0" data-k="balance_usd" data-i="${i}" value="${d.balance_usd}" style="width:100%"></td>
            <td><input type="number" step="0.25" min="0" max="100" data-k="apr_pct" data-i="${i}" value="${d.apr_pct}" style="width:100%"></td>
            <td><input type="number" step="10" min="0" data-k="min_payment_usd" data-i="${i}" value="${d.min_payment_usd}" style="width:100%"></td>
            <td><button class="btn btn-xs" data-del="${i}" data-tip="common.tip.remove_row" data-i18n-aria-label="common.aria.remove" aria-label="Remove">✕</button></td>
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
    const result = mount.querySelector('#ds-result');
    result.innerHTML = `<p class="muted">${esc(t('view.debt_snowball.status.computing'))}</p>`;
    try {
        const r = await api.request('/debt-snowball/compute', { method: 'POST', body: JSON.stringify(STATE) });
        // Interest-paid-per-debt bar chart (payoff order), straight from the result.
        const ordered = (r.debts || []).slice().sort((a, b) => (a.payoff_month || 9999) - (b.payoff_month || 9999));
        const chart = enh.svgBarChart(ordered.map(d => ({ label: d.name, value: d.total_interest_paid_usd })));
        const yearsMo = `${Math.floor(r.total_months / 12)}y ${r.total_months % 12}m`;
        const statusCls = r.all_paid_off ? 'pos' : 'neg';
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.debt_snowball.field.total_months'))}</div>
                    <strong style="font-size:1.4em">${r.total_months}</strong>
                    <div class="muted small">${esc(yearsMo)}</div></div>
                <div><div class="muted small">${esc(t('view.debt_snowball.field.total_principal'))}</div>
                    <strong>$${(r.total_principal_usd / 1000).toFixed(1)}K</strong></div>
                <div><div class="muted small">${esc(t('view.debt_snowball.field.total_interest'))}</div>
                    <strong class="neg">$${(r.total_interest_paid_usd / 1000).toFixed(2)}K</strong></div>
                <div><div class="muted small">${esc(t('view.debt_snowball.field.total_paid'))}</div>
                    <strong>$${(r.total_paid_usd / 1000).toFixed(1)}K</strong></div>
                <div><div class="muted small">${esc(t('view.debt_snowball.field.status'))}</div>
                    <strong class="${statusCls}">${r.all_paid_off
                        ? esc(t('view.debt_snowball.status.all_paid'))
                        : esc(t('view.debt_snowball.status.not_paid'))}</strong></div>
            </div>
            ${chart}
            <h2 style="margin-top:1rem">${esc(t('view.debt_snowball.h2.per_debt'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.debt_snowball.th.name">Debt</th>
                    <th data-i18n="view.debt_snowball.th.balance_h">Balance</th>
                    <th data-i18n="view.debt_snowball.th.apr_h">APR</th>
                    <th data-i18n="view.debt_snowball.th.payoff_month">Payoff month</th>
                    <th data-i18n="view.debt_snowball.th.interest">Interest paid</th>
                </tr></thead>
                <tbody>${(r.debts || []).slice().sort((a, b) => (a.payoff_month || 9999) - (b.payoff_month || 9999)).map(d => `
                    <tr>
                        <td><strong>${esc(d.name)}</strong></td>
                        <td>$${d.starting_balance_usd.toFixed(0)}</td>
                        <td>${d.apr_pct.toFixed(2)}%</td>
                        <td>${d.payoff_month == null ? '∞' : d.payoff_month}</td>
                        <td class="neg">$${d.total_interest_paid_usd.toFixed(0)}</td>
                    </tr>
                `).join('')}</tbody>
            </table>
            <div id="ds-tools" class="ce-toolbar"></div>
        `;
        // Per-debt + totals export (Copy / CSV). No permalink — multi-debt table state.
        enh.mountToolbar(mount.querySelector('#ds-tools'), {
            viewId: 'debt-snowball',
            link: false,
            filename: 'debt-snowball.csv',
            getRows: () => [['debt', 'starting_balance_usd', 'apr_pct', 'payoff_month', 'interest_paid_usd'],
                ...ordered.map(d => [d.name, d.starting_balance_usd, d.apr_pct, d.payoff_month == null ? '' : d.payoff_month, d.total_interest_paid_usd]),
                ['TOTAL', r.total_principal_usd, '', r.total_months, r.total_interest_paid_usd]],
        });
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
