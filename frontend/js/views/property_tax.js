// Property tax — annual/monthly tax from value, assessment ratio, exemptions,
// and mill rate, via /calc/property-tax. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const pct = (n) => Number(n).toLocaleString(undefined, { maximumFractionDigits: 3 }) + '%';

export async function renderPropertyTax(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.proptax.h1.title">// PROPERTY TAX</span></h1>
        <p class="muted small" data-i18n="view.proptax.hint.intro">
            Annual property tax from the home's value, the assessment ratio, any exemptions, and the
            mill rate (dollars of tax per $1,000 of taxable value). The effective rate is the tax
            against market value. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.proptax.h2.inputs">The property</h2>
            <form id="proptax-form" class="inline-form">
                <label><span data-i18n="view.proptax.label.market">Market value ($)</span>
                    <input type="number" step="0.01" min="0" name="market_value_usd" value="400000" required></label>
                <label><span data-i18n="view.proptax.label.ratio">Assessment ratio (%)</span>
                    <input type="number" step="0.1" min="0" max="100" name="assessment_ratio_pct" value="100" required></label>
                <label><span data-i18n="view.proptax.label.exemption">Exemptions ($)</span>
                    <input type="number" step="0.01" min="0" name="exemption_usd" value="25000"></label>
                <label><span data-i18n="view.proptax.label.mill">Mill rate (per $1,000)</span>
                    <input type="number" step="0.01" min="0" name="mill_rate" value="20" required></label>
            </form>
        </div>
        <div id="proptax-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#proptax-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            market_value_usd: Number(fd.get('market_value_usd')) || 0,
            assessment_ratio_pct: Number(fd.get('assessment_ratio_pct')) || 0,
            exemption_usd: Number(fd.get('exemption_usd')) || 0,
            mill_rate: Number(fd.get('mill_rate')) || 0,
        };
        try {
            const r = await api.calcPropertyTax(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.proptax.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#proptax-result');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.proptax.h2.result">The tax</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.proptax.card.annual">Annual tax</div>
                    <div class="value pos">${money(r.annual_tax_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.proptax.card.monthly">Monthly</div>
                    <div class="value">${money(r.monthly_tax_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.proptax.card.effrate">Effective rate</div>
                    <div class="value">${pct(r.effective_rate_pct)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.proptax.row.assessed">Assessed value</td><td>${money(r.assessed_value_usd)}</td></tr>
                    <tr><td data-i18n="view.proptax.row.taxable">Taxable value</td><td>${money(r.taxable_value_usd)}</td></tr>
                    <tr><td data-i18n="view.proptax.row.monthly">Monthly tax</td><td>${money(r.monthly_tax_usd)}</td></tr>
                    <tr><td data-i18n="view.proptax.row.effrate">Effective rate on market</td><td>${pct(r.effective_rate_pct)}</td></tr>
                    <tr class="emph"><td data-i18n="view.proptax.row.annual">Annual tax</td><td>${money(r.annual_tax_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
