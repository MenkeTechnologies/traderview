// SDE business valuation — seller's discretionary earnings × multiple, via
// /calc/sde-valuation. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 1 }) + '%');

export async function renderSdeValuation(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.sde.h1.title">// SDE BUSINESS VALUATION</span></h1>
        <p class="muted small" data-i18n="view.sde.hint.intro">
            What an owner-operated business is worth. Buyers value Seller's Discretionary Earnings —
            net income with the owner's compensation and discretionary items added back — times an
            industry multiple (typically ~2–4×). The small-business analog of EV/EBITDA. Updates as
            you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.sde.h2.inputs">The business</h2>
            <form id="sde-form" class="inline-form">
                <label><span data-i18n="view.sde.label.ni">Net income ($)</span>
                    <input type="number" step="0.01" name="net_income_usd" value="100000" required></label>
                <label><span data-i18n="view.sde.label.owner">Owner compensation ($)</span>
                    <input type="number" step="0.01" min="0" name="owner_compensation_usd" value="80000" required></label>
                <label><span data-i18n="view.sde.label.da">Depreciation & amortization ($)</span>
                    <input type="number" step="0.01" min="0" name="depreciation_amortization_usd" value="15000"></label>
                <label><span data-i18n="view.sde.label.interest">Interest ($)</span>
                    <input type="number" step="0.01" min="0" name="interest_usd" value="5000"></label>
                <label><span data-i18n="view.sde.label.addbacks">Discretionary add-backs ($)</span>
                    <input type="number" step="0.01" min="0" name="discretionary_addbacks_usd" value="10000"></label>
                <label><span data-i18n="view.sde.label.multiple">SDE multiple</span>
                    <input type="number" step="0.1" min="0" name="sde_multiple" value="2.5" required></label>
                <label><span data-i18n="view.sde.label.revenue">Revenue ($)</span>
                    <input type="number" step="0.01" min="0" name="revenue_usd" value="600000"></label>
            </form>
        </div>
        <div id="sde-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#sde-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            net_income_usd: Number(fd.get('net_income_usd')) || 0,
            owner_compensation_usd: Number(fd.get('owner_compensation_usd')) || 0,
            depreciation_amortization_usd: Number(fd.get('depreciation_amortization_usd')) || 0,
            interest_usd: Number(fd.get('interest_usd')) || 0,
            discretionary_addbacks_usd: Number(fd.get('discretionary_addbacks_usd')) || 0,
            sde_multiple: Number(fd.get('sde_multiple')) || 0,
            revenue_usd: Number(fd.get('revenue_usd')) || 0,
        };
        try {
            const r = await api.calcSdeValuation(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.sde.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#sde-result');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.sde.h2.result">The valuation</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.sde.card.value">Business value</div>
                    <div class="value pos">${money(r.business_value_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.sde.card.sde">SDE</div>
                    <div class="value">${money(r.sde_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.sde.card.margin">SDE margin</div>
                    <div class="value">${pct(r.sde_margin_pct)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.sde.row.addbacks">Total add-backs</td><td>${money(r.total_addbacks_usd)}</td></tr>
                    <tr><td data-i18n="view.sde.row.sde">SDE</td><td>${money(r.sde_usd)}</td></tr>
                    <tr><td data-i18n="view.sde.row.margin">SDE margin</td><td>${pct(r.sde_margin_pct)}</td></tr>
                    <tr class="emph"><td data-i18n="view.sde.row.value">Business value</td><td>${money(r.business_value_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
