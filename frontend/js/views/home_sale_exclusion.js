// Home sale exclusion (§121) — capital gain on a primary-home sale net of the
// $250k/$500k exclusion, with depreciation recapture, via
// /calc/home-sale-exclusion. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });

export async function renderHomeSaleExclusion(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.homesale.h1.title">// HOME SALE EXCLUSION</span></h1>
        <p class="muted small" data-i18n="view.homesale.hint.intro">
            Capital gain on selling your primary home, net of the §121 exclusion — up to $250,000
            single, $500,000 married-joint, if you meet the ownership/use test. Depreciation taken
            (home office or former rental) lowers basis and that portion is taxed at up to 25% and
            can't be excluded. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.homesale.h2.inputs">The sale</h2>
            <form id="homesale-form" class="inline-form">
                <label><span data-i18n="view.homesale.label.sale">Sale price ($)</span>
                    <input type="number" step="0.01" min="0" name="sale_price_usd" value="900000" required></label>
                <label><span data-i18n="view.homesale.label.selling">Selling costs ($)</span>
                    <input type="number" step="0.01" min="0" name="selling_costs_usd" value="30000"></label>
                <label><span data-i18n="view.homesale.label.purchase">Purchase price ($)</span>
                    <input type="number" step="0.01" min="0" name="purchase_price_usd" value="300000" required></label>
                <label><span data-i18n="view.homesale.label.improvements">Improvements ($)</span>
                    <input type="number" step="0.01" min="0" name="improvements_usd" value="50000"></label>
                <label><span data-i18n="view.homesale.label.status">Filing status</span>
                    <select name="filing_status">
                        <option value="single" data-i18n="view.homesale.status.single">Single</option>
                        <option value="married_joint" data-i18n="view.homesale.status.mfj">Married filing jointly</option>
                    </select></label>
                <label><span data-i18n="view.homesale.label.dep">Depreciation taken ($)</span>
                    <input type="number" step="0.01" min="0" name="depreciation_taken_usd" value="0"></label>
                <label><span data-i18n="view.homesale.label.ltcg">LTCG rate (%)</span>
                    <input type="number" step="0.1" min="0" name="ltcg_rate_pct" value="15"></label>
            </form>
        </div>
        <div id="homesale-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#homesale-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            sale_price_usd: Number(fd.get('sale_price_usd')) || 0,
            selling_costs_usd: Number(fd.get('selling_costs_usd')) || 0,
            purchase_price_usd: Number(fd.get('purchase_price_usd')) || 0,
            improvements_usd: Number(fd.get('improvements_usd')) || 0,
            filing_status: fd.get('filing_status'),
            depreciation_taken_usd: Number(fd.get('depreciation_taken_usd')) || 0,
            ltcg_rate_pct: Number(fd.get('ltcg_rate_pct')) || 0,
            recapture_rate_pct: 25.0,
        };
        try {
            const r = await api.calcHomeSaleExclusion(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.homesale.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#homesale-result');
    const taxCls = r.tax_usd > 0 ? 'neg' : 'pos';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.homesale.h2.result">The tax</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.homesale.card.gain">Total gain</div>
                    <div class="value">${money(r.total_gain_usd)}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.homesale.card.excluded">Excluded</div>
                    <div class="value pos">${money(r.excluded_gain_usd)}</div></div>
                <div class="card ${taxCls}"><div class="label" data-i18n="view.homesale.card.tax">Tax</div>
                    <div class="value ${taxCls}">${money(r.tax_usd)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.homesale.row.realized">Amount realized</td><td>${money(r.amount_realized_usd)}</td></tr>
                    <tr><td data-i18n="view.homesale.row.basis">Adjusted basis</td><td>${money(r.adjusted_basis_usd)}</td></tr>
                    <tr><td data-i18n="view.homesale.row.gain">Total gain</td><td>${money(r.total_gain_usd)}</td></tr>
                    <tr><td data-i18n="view.homesale.row.limit">Exclusion limit</td><td>${money(r.exclusion_limit_usd)}</td></tr>
                    <tr><td data-i18n="view.homesale.row.excluded">Excluded gain</td><td>${money(r.excluded_gain_usd)}</td></tr>
                    <tr><td data-i18n="view.homesale.row.recapture">Depreciation recapture</td><td>${money(r.recapture_gain_usd)}</td></tr>
                    <tr><td data-i18n="view.homesale.row.ltcg">Taxable LTCG</td><td>${money(r.taxable_ltcg_usd)}</td></tr>
                    <tr class="${taxCls}"><td data-i18n="view.homesale.row.tax">Tax</td><td>${money(r.tax_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.homesale.row.aftertax">After-tax gain</td><td>${money(r.after_tax_gain_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
