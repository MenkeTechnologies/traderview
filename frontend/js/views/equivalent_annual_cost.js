// Equivalent annual cost (EAC) — level annual cost over an asset's life, via
// /calc/equivalent-annual-cost. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }));
const num = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 4 }));

export async function renderEquivalentAnnualCost(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.eac.h1.title">// EQUIVALENT ANNUAL COST</span></h1>
        <p class="muted small" data-i18n="view.eac.hint.intro">
            EAC is the level yearly cost with the same present value as buying and running an asset over
            its life — the right way to compare assets with different lifespans. Annualize each option
            and pick the lower. It splits into the annual operating cost plus the capital recovery cost.
            Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.eac.h2.inputs">The asset</h2>
            <form id="eac-form" class="inline-form">
                <label><span data-i18n="view.eac.label.initial">Initial cost ($)</span>
                    <input type="number" step="1000" min="0" name="initial_cost_usd" value="100000" required></label>
                <label><span data-i18n="view.eac.label.op">Annual operating cost ($)</span>
                    <input type="number" step="500" min="0" name="annual_operating_cost_usd" value="10000"></label>
                <label><span data-i18n="view.eac.label.salvage">Salvage value ($)</span>
                    <input type="number" step="500" min="0" name="salvage_value_usd" value="20000"></label>
                <label><span data-i18n="view.eac.label.rate">Discount rate (%)</span>
                    <input type="number" step="0.1" min="0" name="discount_rate_pct" value="8" required></label>
                <label><span data-i18n="view.eac.label.years">Life (years)</span>
                    <input type="number" step="1" min="1" name="years" value="5" required></label>
            </form>
        </div>
        <div id="eac-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#eac-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            initial_cost_usd: Number(fd.get('initial_cost_usd')) || 0,
            annual_operating_cost_usd: Number(fd.get('annual_operating_cost_usd')) || 0,
            salvage_value_usd: Number(fd.get('salvage_value_usd')) || 0,
            discount_rate_pct: Number(fd.get('discount_rate_pct')) || 0,
            years: Number(fd.get('years')) || 0,
        };
        try {
            const r = await api.calcEquivalentAnnualCost(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.eac.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#eac-result');
    const eacCell = r.equivalent_annual_cost_usd != null ? money(r.equivalent_annual_cost_usd) : t('view.eac.undefined');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.eac.h2.result">The annual cost</h2>
            <div class="cards">
                <div class="card neg"><div class="label" data-i18n="view.eac.card.eac">Equivalent annual cost</div>
                    <div class="value neg">${eacCell}</div></div>
                <div class="card"><div class="label" data-i18n="view.eac.card.caprecovery">Capital recovery</div>
                    <div class="value">${money(r.capital_recovery_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.eac.card.pvcosts">PV of costs</div>
                    <div class="value">${money(r.pv_of_costs_usd)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.eac.row.af">Annuity factor</td><td>${num(r.annuity_factor)}</td></tr>
                    <tr><td data-i18n="view.eac.row.salvagepv">Salvage present value</td><td>${money(r.salvage_pv_usd)}</td></tr>
                    <tr><td data-i18n="view.eac.row.pvcosts">PV of all costs</td><td>${money(r.pv_of_costs_usd)}</td></tr>
                    <tr><td data-i18n="view.eac.row.caprecovery">Capital recovery (annualized)</td><td>${money(r.capital_recovery_usd)}</td></tr>
                    <tr class="emph neg"><td data-i18n="view.eac.row.eac">Equivalent annual cost</td><td>${eacCell}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
