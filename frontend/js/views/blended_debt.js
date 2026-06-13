// Blended debt rate — balance-weighted average APR across debts, total
// monthly interest, and a consolidation comparison, via /calc/blended-debt.
// Editable debt rows; updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const DEFAULT_DEBTS = [
    ['Credit card', 8000, 22.99],
    ['Auto loan', 18000, 6.5],
    ['Student loan', 24000, 5.0],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 });
const esc = (s) => String(s).replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));

function debtRow(name = '', bal = 0, apr = 0) {
    return `<tr class="bd-row">
        <td><input type="text" class="bd-name" value="${esc(name)}" placeholder="${esc(t('view.bld.ph.name'))}"></td>
        <td><input type="number" step="0.01" min="0" class="bd-bal" value="${bal}"></td>
        <td><input type="number" step="0.01" min="0" class="bd-apr" value="${apr}"></td>
        <td><button type="button" class="btn btn-secondary bd-del">✕</button></td>
    </tr>`;
}

export async function renderBlendedDebt(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.bld.h1.title">// BLENDED DEBT RATE</span></h1>
        <p class="muted small" data-i18n="view.bld.hint.intro">
            With several balances at different rates, the rate that matters is the
            balance-weighted average — a big balance at a middling rate costs more than a tiny
            one at a brutal rate. Enter your debts to get the blended APR and total monthly
            interest, then a consolidation rate to see if rolling them into one loan saves
            money. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.bld.h2.inputs">Your debts</h2>
            <table class="data-table" id="bd-debts">
                <thead><tr>
                    <th data-i18n="view.bld.col.name">Debt</th>
                    <th data-i18n="view.bld.col.balance">Balance</th>
                    <th data-i18n="view.bld.col.apr">APR %</th>
                    <th></th>
                </tr></thead>
                <tbody>${DEFAULT_DEBTS.map((d) => debtRow(...d)).join('')}</tbody>
            </table>
            <p>
                <button type="button" class="btn btn-secondary" id="bd-add" data-i18n="view.bld.btn.add">+ Add debt</button>
            </p>
            <form id="bd-form" class="inline-form">
                <label><span data-i18n="view.bld.label.consol">Consolidation APR (%, 0 = skip)</span>
                    <input type="number" step="0.01" min="0" name="consolidation_apr_pct" value="9"></label>
            </form>
        </div>
        <div id="bd-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const body = mount.querySelector('#bd-debts tbody');
    const generate = debounce(async () => {
        const debts = [...mount.querySelectorAll('#bd-debts .bd-row')].map((row) => ({
            name: row.querySelector('.bd-name').value.trim(),
            balance_usd: Number(row.querySelector('.bd-bal').value) || 0,
            apr_pct: Number(row.querySelector('.bd-apr').value) || 0,
        }));
        const consolidation_apr_pct = Number(mount.querySelector('[name="consolidation_apr_pct"]').value) || 0;
        try {
            const r = await api.calcBlendedDebt({ debts, consolidation_apr_pct });
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.bld.toast.error'), { level: 'error' });
        }
    }, 250);

    mount.querySelector('#bd-add').addEventListener('click', () => {
        body.insertAdjacentHTML('beforeend', debtRow());
        applyUiI18n(body.lastElementChild);
        generate();
    });
    body.addEventListener('click', (e) => {
        if (e.target.classList.contains('bd-del')) {
            if (body.querySelectorAll('.bd-row').length > 1) { e.target.closest('tr').remove(); generate(); }
        }
    });
    body.addEventListener('input', generate);
    mount.querySelector('#bd-form').addEventListener('input', generate);
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#bd-result');
    const worth = r.consolidation_worth_it;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.bld.h2.result">The blend</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.bld.card.blended">Blended APR</div>
                    <div class="value">${Number(r.blended_apr_pct).toFixed(2)}%</div></div>
                <div class="card"><div class="label" data-i18n="view.bld.card.balance">Total balance</div>
                    <div class="value">${money(r.total_balance_usd)}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.bld.card.interest">Monthly interest</div>
                    <div class="value neg">${money(r.total_monthly_interest_usd)}</div></div>
                <div class="card ${worth ? 'pos' : ''}"><div class="label" data-i18n="view.bld.card.consol">Consolidation</div>
                    <div class="value ${worth ? 'pos' : 'neg'}">${worth ? t('view.bld.worth') : t('view.bld.notworth')}</div></div>
            </div>
            <table class="data-table">
                <thead><tr><th data-i18n="view.bld.col.line">Line</th><th data-i18n="view.bld.col.amount">Monthly</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.bld.row.current">Current monthly interest</td><td>${money(r.total_monthly_interest_usd)}</td></tr>
                    <tr><td data-i18n="view.bld.row.consol">Consolidated monthly interest</td><td>${money(r.consolidation_monthly_interest_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.bld.row.savings">Monthly savings</td>
                        <td class="${r.monthly_savings_usd >= 0 ? 'pos' : 'neg'}">${money(r.monthly_savings_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
