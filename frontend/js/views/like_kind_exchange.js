// § 1031 like-kind exchange — boot, recognized vs deferred gain, tax now,
// and the replacement property's carryover basis, via
// /calc/like-kind-exchange. Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const FIELDS = [
    ['relinquished_sale_price_usd', 'Relinquished sale price ($)', 500000],
    ['relinquished_adjusted_basis_usd', 'Relinquished adjusted basis ($)', 300000],
    ['relinquished_mortgage_usd', 'Old mortgage relieved ($)', 200000],
    ['replacement_purchase_price_usd', 'Replacement price ($)', 600000],
    ['replacement_mortgage_usd', 'New mortgage ($)', 300000],
    ['cash_received_usd', 'Cash boot received ($)', 0],
    ['selling_costs_usd', 'Exchange/selling costs ($)', 0],
    ['capital_gains_tax_pct', 'Capital-gains rate (%)', 20],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });

export async function renderLikeKindExchange(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.lke.h1.title">// § 1031 LIKE-KIND EXCHANGE</span></h1>
        <p class="muted small" data-i18n="view.lke.hint.intro">
            Swapping one investment property for another defers the gain — but only to the
            extent you fully reinvest. Boot (non-like-kind value you keep) is taxable now: cash
            received plus net debt relief (old mortgage − new mortgage, if you trade down).
            Recognized gain = the lesser of realized gain or total boot; the rest defers into
            the replacement property's carryover basis (its cost − deferred gain). A loss isn't
            recognized. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.lke.h2.inputs">The exchange</h2>
            <form id="lke-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.lke.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
        </div>
        <div id="lke-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#lke-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        try {
            const r = await api.calcLikeKindExchange(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.lke.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#lke-result');
    const deferredCls = r.fully_deferred ? 'pos' : '';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.lke.h2.result">The result</h2>
            <div class="cards">
                <div class="card ${deferredCls}"><div class="label" data-i18n="view.lke.card.deferred">Gain deferred</div>
                    <div class="value ${deferredCls}">${money(r.deferred_gain_usd)}</div></div>
                <div class="card ${r.recognized_gain_usd > 0 ? 'neg' : ''}"><div class="label" data-i18n="view.lke.card.recognized">Gain recognized now</div>
                    <div class="value">${money(r.recognized_gain_usd)}</div></div>
                <div class="card ${r.tax_now_usd > 0 ? 'neg' : ''}"><div class="label" data-i18n="view.lke.card.tax">Tax now</div>
                    <div class="value">${money(r.tax_now_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.lke.card.basis">Replacement basis</div>
                    <div class="value">${money(r.replacement_basis_usd)}</div></div>
            </div>
            <table class="data-table">
                <thead><tr><th data-i18n="view.lke.col.line">Line</th><th data-i18n="view.lke.col.amount">Amount</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.lke.row.realized">Realized gain</td><td>${money(r.realized_gain_usd)}</td></tr>
                    <tr><td data-i18n="view.lke.row.cash_boot">Cash boot</td><td>${money(r.cash_boot_usd)}</td></tr>
                    <tr><td data-i18n="view.lke.row.mortgage_boot">Mortgage boot (net debt relief)</td><td>${money(r.mortgage_boot_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.lke.row.total_boot">Total boot</td><td>${money(r.total_boot_usd)}</td></tr>
                </tbody>
            </table>
            ${r.fully_deferred ? `<p class="muted small pos" data-i18n="view.lke.note.full">Fully deferred — no boot, no tax due now; the entire gain rolls into the replacement basis.</p>` : ''}
        </div>
    `;
    applyUiI18n(el);
}
