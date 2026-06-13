// Invoice factoring — advance, fee, reserve, net proceeds, and the
// annualized effective APR of selling a receivable, via
// /calc/invoice-factoring. Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const FIELDS = [
    ['invoice_amount_usd', 'Invoice amount ($)', 10000],
    ['advance_rate_pct', 'Advance rate (%)', 80],
    ['factor_fee_pct', 'Factor fee (% of invoice)', 3],
    ['term_days', 'Collection term (days)', 30],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 });
const pct = (n) => Number(n).toFixed(1) + '%';

export async function renderInvoiceFactoring(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.fact.h1.title">// INVOICE FACTORING</span></h1>
        <p class="muted small" data-i18n="view.fact.hint.intro">
            A factor advances most of an unpaid invoice now, charges a fee, and releases the
            rest (the reserve, net of the fee) when the customer pays. A flat 3% fee looks
            cheap, but it buys cash for only the short collection period — annualized, it's an
            expensive loan. The effective APR = (fee ÷ advance) × (365 ÷ term days). Updates
            as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.fact.h2.inputs">The invoice</h2>
            <form id="fact-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.fact.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
        </div>
        <div id="fact-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#fact-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        try {
            const r = await api.calcInvoiceFactoring(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.fact.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#fact-result');
    // Annualized cost above ~30% is the "expensive" zone worth flagging.
    const aprCls = r.effective_apr_pct >= 30 ? 'neg' : '';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.fact.h2.result">The cost</h2>
            <div class="cards">
                <div class="card ${aprCls}"><div class="label" data-i18n="view.fact.card.apr">Effective APR</div>
                    <div class="value ${aprCls}">${pct(r.effective_apr_pct)}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.fact.card.advance">Cash advanced now</div>
                    <div class="value">${money(r.advance_usd)}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.fact.card.fee">Factor fee</div>
                    <div class="value neg">${money(r.fee_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.fact.card.net">Net proceeds</div>
                    <div class="value">${money(r.net_proceeds_usd)}</div></div>
            </div>
            <table class="data-table">
                <thead><tr><th data-i18n="view.fact.col.line">Line</th><th data-i18n="view.fact.col.amount">Amount</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.fact.row.advance">Advanced upfront</td><td>${money(r.advance_usd)}</td></tr>
                    <tr><td data-i18n="view.fact.row.reserve">Reserve held back</td><td>${money(r.reserve_usd)}</td></tr>
                    <tr><td data-i18n="view.fact.row.released">Reserve released (net of fee)</td><td>${money(r.reserve_released_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.fact.row.net">Total received</td><td>${money(r.net_proceeds_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
