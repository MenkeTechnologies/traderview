// Capital-gains tax — long-term 0/15/20 bracket stacking or short-term
// ordinary, via /calc/capital-gains-tax. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const pct = (n) => Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%';

export async function renderCapitalGainsTax(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.capgains.h1.title">// CAPITAL GAINS TAX</span></h1>
        <p class="muted small" data-i18n="view.capgains.hint.intro">
            Federal tax on a sale. Long-term gains use the preferential 0% / 15% / 20% brackets,
            stacked on top of your ordinary taxable income so the gain is taxed in slices as it
            crosses each 2026 threshold. Short-term gains are ordinary income at the rate you enter.
            Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.capgains.h2.inputs">The sale</h2>
            <form id="capgains-form" class="inline-form">
                <label><span data-i18n="view.capgains.label.proceeds">Proceeds ($)</span>
                    <input type="number" step="0.01" name="proceeds_usd" value="50000" required></label>
                <label><span data-i18n="view.capgains.label.basis">Cost basis ($)</span>
                    <input type="number" step="0.01" name="cost_basis_usd" value="30000" required></label>
                <label><span data-i18n="view.capgains.label.term">Holding term</span>
                    <select name="term">
                        <option value="long" data-i18n="view.capgains.term.long">Long-term (> 1 year)</option>
                        <option value="short" data-i18n="view.capgains.term.short">Short-term (≤ 1 year)</option>
                    </select></label>
                <label><span data-i18n="view.capgains.label.status">Filing status</span>
                    <select name="filing_status">
                        <option value="single" data-i18n="view.capgains.status.single">Single</option>
                        <option value="married_joint" data-i18n="view.capgains.status.mfj">Married filing jointly</option>
                    </select></label>
                <label><span data-i18n="view.capgains.label.income">Ordinary taxable income ($)</span>
                    <input type="number" step="0.01" min="0" name="ordinary_taxable_income_usd" value="40000"></label>
                <label><span data-i18n="view.capgains.label.ordrate">Ordinary rate for short-term (%)</span>
                    <input type="number" step="0.1" min="0" name="ordinary_rate_pct" value="24"></label>
            </form>
        </div>
        <div id="capgains-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#capgains-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            proceeds_usd: Number(fd.get('proceeds_usd')) || 0,
            cost_basis_usd: Number(fd.get('cost_basis_usd')) || 0,
            term: fd.get('term'),
            filing_status: fd.get('filing_status'),
            ordinary_taxable_income_usd: Number(fd.get('ordinary_taxable_income_usd')) || 0,
            ordinary_rate_pct: Number(fd.get('ordinary_rate_pct')) || 0,
        };
        try {
            const r = await api.calcCapitalGainsTax(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.capgains.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#capgains-result');
    const gainClass = r.gain_usd >= 0 ? 'pos' : 'neg';
    const slices = r.is_long_term
        ? `
            <tr><td data-i18n="view.capgains.row.at0">Taxed at 0%</td><td>${money(r.taxed_at_0_usd)}</td></tr>
            <tr><td data-i18n="view.capgains.row.at15">Taxed at 15%</td><td>${money(r.taxed_at_15_usd)}</td></tr>
            <tr><td data-i18n="view.capgains.row.at20">Taxed at 20%</td><td>${money(r.taxed_at_20_usd)}</td></tr>`
        : `<tr><td data-i18n="view.capgains.row.ordinary">Taxed at ordinary rate</td><td>${money(r.taxed_at_ordinary_usd)}</td></tr>`;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.capgains.h2.result">The tax</h2>
            <div class="cards">
                <div class="card ${gainClass}"><div class="label" data-i18n="view.capgains.card.gain">Gain</div>
                    <div class="value ${gainClass}">${money(r.gain_usd)}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.capgains.card.tax">Tax</div>
                    <div class="value neg">${money(r.tax_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.capgains.card.effrate">Effective rate</div>
                    <div class="value">${pct(r.effective_rate_pct)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    ${slices}
                    <tr><td data-i18n="view.capgains.row.tax">Tax owed</td><td>${money(r.tax_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.capgains.row.aftertax">After-tax gain</td><td>${money(r.after_tax_gain_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
