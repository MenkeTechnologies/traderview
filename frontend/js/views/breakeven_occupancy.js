// Break-even occupancy — occupancy needed to cover opex + debt service, with
// the cash flow at an expected occupancy, via /calc/breakeven-occupancy. Live.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 1 }) + '%');

export async function renderBreakevenOccupancy(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.beocc.h1.title">// BREAK-EVEN OCCUPANCY</span></h1>
        <p class="muted small" data-i18n="view.beocc.hint.intro">
            The occupancy at which a rental's income exactly covers operating expenses plus debt
            service — the lender's cushion metric. The lower it is, the more vacancy the property can
            absorb. Also shows the cash flow at your expected occupancy. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.beocc.h2.inputs">The property</h2>
            <form id="beocc-form" class="inline-form">
                <label><span data-i18n="view.beocc.label.gpr">Gross potential rent / yr ($)</span>
                    <input type="number" step="0.01" min="0" name="gross_potential_rent_usd" value="100000" required></label>
                <label><span data-i18n="view.beocc.label.opex">Operating expenses ($)</span>
                    <input type="number" step="0.01" min="0" name="operating_expenses_usd" value="35000" required></label>
                <label><span data-i18n="view.beocc.label.ds">Annual debt service ($)</span>
                    <input type="number" step="0.01" min="0" name="annual_debt_service_usd" value="45000" required></label>
                <label><span data-i18n="view.beocc.label.expected">Expected occupancy (%)</span>
                    <input type="number" step="0.1" min="0" max="100" name="expected_occupancy_pct" value="95"></label>
            </form>
        </div>
        <div id="beocc-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#beocc-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            gross_potential_rent_usd: Number(fd.get('gross_potential_rent_usd')) || 0,
            operating_expenses_usd: Number(fd.get('operating_expenses_usd')) || 0,
            annual_debt_service_usd: Number(fd.get('annual_debt_service_usd')) || 0,
            expected_occupancy_pct: Number(fd.get('expected_occupancy_pct')) || 0,
        };
        try {
            const r = await api.calcBreakevenOccupancy(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.beocc.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#beocc-result');
    const cushionCls = (r.vacancy_cushion_pct ?? 0) >= 0 ? 'pos' : 'neg';
    const cfCls = r.cash_flow_at_expected_usd >= 0 ? 'pos' : 'neg';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.beocc.h2.result">The cushion</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.beocc.card.breakeven">Break-even occupancy</div>
                    <div class="value">${pct(r.breakeven_occupancy_pct)}</div></div>
                <div class="card ${cushionCls}"><div class="label" data-i18n="view.beocc.card.cushion">Vacancy cushion</div>
                    <div class="value ${cushionCls}">${pct(r.vacancy_cushion_pct)}</div></div>
                <div class="card ${cfCls}"><div class="label" data-i18n="view.beocc.card.cashflow">Cash flow at expected</div>
                    <div class="value ${cfCls}">${money(r.cash_flow_at_expected_usd)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.beocc.row.breakeven">Break-even occupancy</td><td>${pct(r.breakeven_occupancy_pct)}</td></tr>
                    <tr><td data-i18n="view.beocc.row.cushion">Vacancy cushion</td><td>${pct(r.vacancy_cushion_pct)}</td></tr>
                    <tr><td data-i18n="view.beocc.row.egi">EGI at expected occupancy</td><td>${money(r.effective_gross_income_usd)}</td></tr>
                    <tr class="emph ${cfCls}"><td data-i18n="view.beocc.row.cashflow">Cash flow at expected</td><td>${money(r.cash_flow_at_expected_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
