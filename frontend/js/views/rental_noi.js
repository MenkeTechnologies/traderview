// Rental NOI — net operating income from rental income statement line items,
// via /calc/rental-noi. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 1 }) + '%');

export async function renderRentalNoi(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.noi.h1.title">// RENTAL NOI</span></h1>
        <p class="muted small" data-i18n="view.noi.hint.intro">
            Net operating income from the rental income statement — rent plus other income, less
            vacancy and operating expenses (management is a percent of effective gross income). NOI
            excludes debt service and capital improvements, and it feeds cap rate, DSCR, and debt
            yield. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.noi.h2.inputs">The property</h2>
            <form id="noi-form" class="inline-form">
                <label><span data-i18n="view.noi.label.rent">Gross rental income / yr ($)</span>
                    <input type="number" step="0.01" min="0" name="gross_rental_income_usd" value="60000" required></label>
                <label><span data-i18n="view.noi.label.other">Other income ($)</span>
                    <input type="number" step="0.01" min="0" name="other_income_usd" value="2000"></label>
                <label><span data-i18n="view.noi.label.vacancy">Vacancy (% of rent)</span>
                    <input type="number" step="0.1" min="0" name="vacancy_pct" value="5"></label>
                <label><span data-i18n="view.noi.label.taxes">Property taxes ($)</span>
                    <input type="number" step="0.01" min="0" name="property_taxes_usd" value="7000"></label>
                <label><span data-i18n="view.noi.label.insurance">Insurance ($)</span>
                    <input type="number" step="0.01" min="0" name="insurance_usd" value="2000"></label>
                <label><span data-i18n="view.noi.label.maintenance">Maintenance ($)</span>
                    <input type="number" step="0.01" min="0" name="maintenance_usd" value="3000"></label>
                <label><span data-i18n="view.noi.label.mgmt">Management (% of EGI)</span>
                    <input type="number" step="0.1" min="0" name="management_pct" value="8"></label>
                <label><span data-i18n="view.noi.label.utilities">Utilities ($)</span>
                    <input type="number" step="0.01" min="0" name="utilities_usd" value="1500"></label>
                <label><span data-i18n="view.noi.label.repairs">Repairs ($)</span>
                    <input type="number" step="0.01" min="0" name="repairs_usd" value="1000"></label>
                <label><span data-i18n="view.noi.label.hoa">HOA ($)</span>
                    <input type="number" step="0.01" min="0" name="hoa_usd" value="0"></label>
                <label><span data-i18n="view.noi.label.other_exp">Other expenses ($)</span>
                    <input type="number" step="0.01" min="0" name="other_expenses_usd" value="500"></label>
            </form>
        </div>
        <div id="noi-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#noi-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            gross_rental_income_usd: Number(fd.get('gross_rental_income_usd')) || 0,
            other_income_usd: Number(fd.get('other_income_usd')) || 0,
            vacancy_pct: Number(fd.get('vacancy_pct')) || 0,
            property_taxes_usd: Number(fd.get('property_taxes_usd')) || 0,
            insurance_usd: Number(fd.get('insurance_usd')) || 0,
            maintenance_usd: Number(fd.get('maintenance_usd')) || 0,
            management_pct: Number(fd.get('management_pct')) || 0,
            utilities_usd: Number(fd.get('utilities_usd')) || 0,
            repairs_usd: Number(fd.get('repairs_usd')) || 0,
            hoa_usd: Number(fd.get('hoa_usd')) || 0,
            other_expenses_usd: Number(fd.get('other_expenses_usd')) || 0,
        };
        try {
            const r = await api.calcRentalNoi(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.noi.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#noi-result');
    const noiCls = r.noi_usd >= 0 ? 'pos' : 'neg';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.noi.h2.result">The income statement</h2>
            <div class="cards">
                <div class="card ${noiCls}"><div class="label" data-i18n="view.noi.card.noi">NOI</div>
                    <div class="value ${noiCls}">${money(r.noi_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.noi.card.egi">Effective gross income</div>
                    <div class="value">${money(r.effective_gross_income_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.noi.card.oer">Expense ratio</div>
                    <div class="value">${pct(r.operating_expense_ratio_pct)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.noi.row.potential">Potential gross income</td><td>${money(r.potential_gross_income_usd)}</td></tr>
                    <tr><td data-i18n="view.noi.row.vacancy">Vacancy loss</td><td>${money(r.vacancy_loss_usd)}</td></tr>
                    <tr><td data-i18n="view.noi.row.egi">Effective gross income</td><td>${money(r.effective_gross_income_usd)}</td></tr>
                    <tr><td data-i18n="view.noi.row.mgmt">Management fee</td><td>${money(r.management_fee_usd)}</td></tr>
                    <tr><td data-i18n="view.noi.row.opex">Total operating expenses</td><td>${money(r.total_operating_expenses_usd)}</td></tr>
                    <tr class="emph ${noiCls}"><td data-i18n="view.noi.row.noi">Net operating income</td><td>${money(r.noi_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
