// Tenant turnover cost — lost rent + make-ready + leasing + concession, via /calc/tenant-turnover.
import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
export async function renderTenantTurnover(mount, _s) {
    const tok = currentViewToken(); if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.turn.h1.title">// TENANT TURNOVER COST</span></h1>
        <p class="muted small" data-i18n="view.turn.hint.intro">The all-in cost when a unit turns over: rent lost while vacant, make-ready (cleaning, paint, repairs), leasing/marketing, and any move-in concession. It totals these and shows the result as a percentage of annual rent and as months of rent — why minimizing turnover matters. Not advice.</p>
        <div class="lpv-split"><div class="chart-panel"><h2 data-i18n="view.turn.h2.inputs">Turnover</h2>
        <form id="turn-form" class="inline-form">
            <label><span data-i18n="view.turn.label.property">Property</span><input type="text" name="property_label" value="Unit 4B"></label>
            <label><span data-i18n="view.turn.label.rent">Monthly rent ($)</span><input type="number" step="50" min="0" name="monthly_rent_usd" value="1500" required></label>
            <label><span data-i18n="view.turn.label.vac">Vacancy (days)</span><input type="number" step="1" min="0" name="vacancy_days" value="30"></label>
            <label><span data-i18n="view.turn.label.ready">Make-ready ($)</span><input type="number" step="50" min="0" name="make_ready_usd" value="800"></label>
            <label><span data-i18n="view.turn.label.leasing">Leasing / marketing ($)</span><input type="number" step="50" min="0" name="leasing_cost_usd" value="750"></label>
            <label><span data-i18n="view.turn.label.concession">Move-in concession ($)</span><input type="number" step="50" min="0" name="concession_usd" value="500"></label>
        </form></div><div id="turn-result" class="chart-panel lpv-preview"></div></div>`;
    applyUiI18n(mount);
    const form = mount.querySelector('#turn-form'); const n = (k) => Number(form.querySelector(`[name="${k}"]`).value) || 0;
    const gen = async () => {
        const body = { property_label: (form.querySelector('[name="property_label"]').value || '').trim(), monthly_rent_usd: n('monthly_rent_usd'), vacancy_days: n('vacancy_days'), make_ready_usd: n('make_ready_usd'), leasing_cost_usd: n('leasing_cost_usd'), concession_usd: n('concession_usd') };
        try { const d = await api.calcTenantTurnover(body); if (!viewIsCurrent(tok)) return; res(mount, d); }
        catch (e) { showToast(e.message || t('view.turn.toast.error'), { level: 'error' }); }
    };
    const live = debounce(gen, 250); form.addEventListener('input', () => live()); form.addEventListener('submit', (e) => { e.preventDefault(); gen(); }); gen();
}
function res(mount, d) {
    const el = mount.querySelector('#turn-result');
    el.innerHTML = `<div class="lpv-bar"><div class="cards">
        <div class="card neg"><div class="label" data-i18n="view.turn.card.total">Total turnover cost</div><div class="value">${money(d.total_turnover_cost_usd)}</div></div>
        <div class="card"><div class="label" data-i18n="view.turn.card.pct">% of annual rent</div><div class="value">${pct(d.pct_of_annual_rent)}</div></div>
        <div class="card"><div class="label" data-i18n="view.turn.card.months">Months of rent</div><div class="value">${d.months_of_rent}</div></div>
        <div class="card"><div class="label" data-i18n="view.turn.card.lost">Lost rent (vacancy)</div><div class="value">${money(d.lost_rent_usd)}</div></div>
        <div class="card"><div class="label" data-i18n="view.turn.card.daily">Daily rent</div><div class="value">${money(d.daily_rent_usd)}</div></div>
    </div></div>`;
    applyUiI18n(el);
}
