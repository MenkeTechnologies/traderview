// Tax-equivalent yield — muni vs taxable bond on an after-tax basis, via
// /calc/tax-equivalent-yield. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 3 }) + '%');

const VERDICT = { muni: 'view.tey.verdict.muni', taxable: 'view.tey.verdict.taxable', equal: 'view.tey.verdict.equal' };

export async function renderTaxEquivalentYield(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.tey.h1.title">// TAX-EQUIVALENT YIELD</span></h1>
        <p class="muted small" data-i18n="view.tey.hint.intro">
            Compares a tax-free municipal bond to a taxable bond after tax. Muni interest is free of
            federal tax (and the 3.8% NIIT), and state tax too when issued in your state. The
            tax-equivalent yield is what a taxable bond would have to yield to match the muni after
            tax. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.tey.h2.inputs">The bonds</h2>
            <form id="tey-form" class="inline-form">
                <label><span data-i18n="view.tey.label.muni">Muni yield (%)</span>
                    <input type="number" step="0.01" min="0" name="muni_yield_pct" value="3.5" required></label>
                <label><span data-i18n="view.tey.label.taxable">Taxable yield to compare (%)</span>
                    <input type="number" step="0.01" min="0" name="taxable_yield_pct" value="5"></label>
                <label><span data-i18n="view.tey.label.federal">Federal rate (%)</span>
                    <input type="number" step="0.1" min="0" name="federal_rate_pct" value="32" required></label>
                <label><span data-i18n="view.tey.label.state">State rate (%)</span>
                    <input type="number" step="0.1" min="0" name="state_rate_pct" value="0"></label>
                <label><span data-i18n="view.tey.label.instate">Muni issued in your state</span>
                    <input type="checkbox" name="in_state"></label>
                <label><span data-i18n="view.tey.label.niit">NIIT (3.8%) applies</span>
                    <input type="checkbox" name="niit_applies"></label>
            </form>
        </div>
        <div id="tey-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#tey-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            muni_yield_pct: Number(fd.get('muni_yield_pct')) || 0,
            taxable_yield_pct: Number(fd.get('taxable_yield_pct')) || 0,
            federal_rate_pct: Number(fd.get('federal_rate_pct')) || 0,
            state_rate_pct: Number(fd.get('state_rate_pct')) || 0,
            in_state: form.querySelector('[name=in_state]').checked,
            niit_applies: form.querySelector('[name=niit_applies]').checked,
        };
        try {
            const r = await api.calcTaxEquivalentYield(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.tey.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#tey-result');
    const winCls = r.verdict === 'muni' ? 'pos' : (r.verdict === 'taxable' ? 'neg' : '');
    const verdictRow = r.verdict
        ? `<tr class="${winCls}"><td data-i18n="view.tey.row.verdict">Better after tax</td><td data-i18n="${VERDICT[r.verdict]}">—</td></tr>`
        : '';
    const taxableRow = r.taxable_after_tax_pct == null ? '' :
        `<tr><td data-i18n="view.tey.row.taxableat">Taxable after-tax</td><td>${pct(r.taxable_after_tax_pct)}</td></tr>`;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.tey.h2.result">After tax</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.tey.card.tey">Tax-equivalent yield</div>
                    <div class="value pos">${pct(r.tax_equivalent_yield_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.tey.card.muniat">Muni after-tax</div>
                    <div class="value">${pct(r.muni_after_tax_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.tey.card.rate">Combined tax rate</div>
                    <div class="value">${pct(r.combined_taxable_rate_pct)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.tey.row.muniat">Muni after-tax</td><td>${pct(r.muni_after_tax_pct)}</td></tr>
                    ${taxableRow}
                    <tr><td data-i18n="view.tey.row.rate">Combined taxable rate</td><td>${pct(r.combined_taxable_rate_pct)}</td></tr>
                    ${verdictRow}
                    <tr class="emph"><td data-i18n="view.tey.row.tey">Tax-equivalent yield</td><td>${pct(r.tax_equivalent_yield_pct)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
