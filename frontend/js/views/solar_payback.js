// Solar payback — net cost after credits, payback years, lifetime savings, and
// ROI, via /calc/solar-payback. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const pct = (n) => Number(n).toLocaleString(undefined, { maximumFractionDigits: 1 }) + '%';
const yrs = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 1 }));

export async function renderSolarPayback(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.solar.h1.title">// SOLAR PAYBACK</span></h1>
        <p class="muted small" data-i18n="view.solar.hint.intro">
            When a rooftop solar system pays for itself. Net cost is the install price less the 30%
            federal credit and any incentives; annual savings rise with utility-rate inflation but
            fall as panels degrade. Shows the payback year, lifetime savings, and ROI. Updates as
            you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.solar.h2.inputs">The system</h2>
            <form id="solar-form" class="inline-form">
                <label><span data-i18n="view.solar.label.cost">System cost ($)</span>
                    <input type="number" step="0.01" min="0" name="system_cost_usd" value="30000" required></label>
                <label><span data-i18n="view.solar.label.fed">Federal credit (%)</span>
                    <input type="number" step="0.1" min="0" name="federal_credit_pct" value="30" required></label>
                <label><span data-i18n="view.solar.label.incentives">Other incentives ($)</span>
                    <input type="number" step="0.01" min="0" name="other_incentives_usd" value="0"></label>
                <label><span data-i18n="view.solar.label.savings">Year-1 savings ($)</span>
                    <input type="number" step="0.01" min="0" name="annual_savings_usd" value="2000" required></label>
                <label><span data-i18n="view.solar.label.deg">Annual degradation (%)</span>
                    <input type="number" step="0.1" min="0" name="annual_degradation_pct" value="0.5"></label>
                <label><span data-i18n="view.solar.label.inf">Electricity inflation (%)</span>
                    <input type="number" step="0.1" min="0" name="electricity_inflation_pct" value="3"></label>
                <label><span data-i18n="view.solar.label.horizon">Horizon (years)</span>
                    <input type="number" step="1" min="1" name="horizon_years" value="25"></label>
            </form>
        </div>
        <div id="solar-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#solar-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            system_cost_usd: Number(fd.get('system_cost_usd')) || 0,
            federal_credit_pct: Number(fd.get('federal_credit_pct')) || 0,
            other_incentives_usd: Number(fd.get('other_incentives_usd')) || 0,
            annual_savings_usd: Number(fd.get('annual_savings_usd')) || 0,
            annual_degradation_pct: Number(fd.get('annual_degradation_pct')) || 0,
            electricity_inflation_pct: Number(fd.get('electricity_inflation_pct')) || 0,
            horizon_years: Number(fd.get('horizon_years')) || 0,
        };
        try {
            const r = await api.calcSolarPayback(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.solar.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#solar-result');
    const profitCls = r.net_profit_usd >= 0 ? 'pos' : 'neg';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.solar.h2.result">The return</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.solar.card.payback">Payback (years)</div>
                    <div class="value pos">${yrs(r.payback_years)}</div></div>
                <div class="card"><div class="label" data-i18n="view.solar.card.net">Net cost</div>
                    <div class="value">${money(r.net_cost_usd)}</div></div>
                <div class="card ${profitCls}"><div class="label" data-i18n="view.solar.card.roi">Lifetime ROI</div>
                    <div class="value ${profitCls}">${pct(r.roi_pct)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.solar.row.credit">Federal credit</td><td>${money(r.federal_credit_usd)}</td></tr>
                    <tr><td data-i18n="view.solar.row.net">Net cost</td><td>${money(r.net_cost_usd)}</td></tr>
                    <tr><td data-i18n="view.solar.row.lifetime">Lifetime savings</td><td>${money(r.lifetime_savings_usd)}</td></tr>
                    <tr><td data-i18n="view.solar.row.payback">Payback (years)</td><td>${yrs(r.payback_years)}</td></tr>
                    <tr class="emph ${profitCls}"><td data-i18n="view.solar.row.profit">Net profit</td><td>${money(r.net_profit_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
