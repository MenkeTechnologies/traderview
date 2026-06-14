// MACRS depreciation — IRS tax depreciation schedule, via /calc/macrs-depreciation.
import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
export async function renderMacrsDepreciation(mount, _s) {
    const tok = currentViewToken(); if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.macrs.h1.title">// MACRS DEPRECIATION</span></h1>
        <p class="muted small" data-i18n="view.macrs.hint.intro">The U.S. Modified Accelerated Cost Recovery System tax depreciation schedule (GDS, half-year convention, IRS Pub. 946). Applies the published per-year percentage table for the recovery period to give the deduction, accumulated depreciation, and remaining basis each year. Not tax advice.</p>
        <div class="lpv-split"><div class="chart-panel"><h2 data-i18n="view.macrs.h2.inputs">Asset</h2>
        <form id="macrs-form" class="inline-form">
            <label><span data-i18n="view.macrs.label.label">Asset</span><input type="text" name="asset_label" value="Equipment"></label>
            <label><span data-i18n="view.macrs.label.basis">Depreciable basis ($)</span><input type="number" step="100" min="0" name="basis_usd" value="10000" required></label>
            <label><span data-i18n="view.macrs.label.period">Recovery period</span><select name="recovery_years">
                <option value="3">3-year</option><option value="5" selected>5-year</option><option value="7">7-year</option>
                <option value="10">10-year</option><option value="15">15-year</option><option value="20">20-year</option></select></label>
            <label><span data-i18n="view.macrs.label.year">Placed in service (year)</span><input type="number" step="1" name="placed_in_service_year" value="2026"></label>
        </form></div><div id="macrs-result" class="chart-panel lpv-preview"></div></div>`;
    applyUiI18n(mount);
    const form = mount.querySelector('#macrs-form');
    const gen = async () => {
        const body = { asset_label: (form.querySelector('[name="asset_label"]').value || '').trim(), basis_usd: Number(form.querySelector('[name="basis_usd"]').value) || 0, recovery_years: Number(form.querySelector('[name="recovery_years"]').value) || 5, placed_in_service_year: Number(form.querySelector('[name="placed_in_service_year"]').value) || 0 };
        try { const d = await api.calcMacrsDepreciation(body); if (!viewIsCurrent(tok)) return; res(mount, d, body.placed_in_service_year); }
        catch (e) { showToast(e.message || t('view.macrs.toast.error'), { level: 'error' }); }
    };
    const live = debounce(gen, 250); form.addEventListener('input', () => live()); form.addEventListener('submit', (e) => { e.preventDefault(); gen(); }); gen();
}
function res(mount, d, startYear) {
    const el = mount.querySelector('#macrs-result');
    if (!d.valid) { el.innerHTML = `<p class="muted" data-i18n="view.macrs.invalid">Recovery period must be 3, 5, 7, 10, 15, or 20 years.</p>`; applyUiI18n(el); return; }
    const rows = d.schedule.map((r) => `<tr><td>${startYear > 0 ? startYear + r.year - 1 : r.year}</td><td>${r.rate_pct}%</td><td>${money(r.depreciation_usd)}</td><td>${money(r.accumulated_usd)}</td><td>${money(r.remaining_basis_usd)}</td></tr>`).join('');
    el.innerHTML = `<div class="lpv-bar"><div class="cards">
        <div class="card pos"><div class="label" data-i18n="view.macrs.card.total">Total depreciation</div><div class="value">${money(d.total_depreciation_usd)}</div></div>
        <div class="card"><div class="label" data-i18n="view.macrs.card.period">Recovery period</div><div class="value">${d.recovery_years}y</div></div>
    </div></div>
    <table class="data-table"><thead><tr><th data-i18n="view.macrs.th.year">Year</th><th data-i18n="view.macrs.th.rate">Rate</th><th data-i18n="view.macrs.th.dep">Depreciation</th><th data-i18n="view.macrs.th.accum">Accumulated</th><th data-i18n="view.macrs.th.basis">Remaining basis</th></tr></thead><tbody>${rows}</tbody></table>`;
    applyUiI18n(el);
}
