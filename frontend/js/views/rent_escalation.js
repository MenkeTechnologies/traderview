// Lease cost with escalations + free rent — total, NPV, and effective monthly
// rent, via /calc/rent-escalation. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });

export async function renderRentEscalation(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.rentesc.h1.title">// LEASE COST</span></h1>
        <p class="muted small" data-i18n="view.rentesc.hint.intro">
            The true cost of a lease with annual rent bumps and free-rent concessions. Rent steps up
            each anniversary; the first months can be waived. The effective monthly rent spreads the
            total over the full term — what you compare offers on. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.rentesc.h2.inputs">The lease</h2>
            <form id="rentesc-form" class="inline-form">
                <label><span data-i18n="view.rentesc.label.base">Base monthly rent ($)</span>
                    <input type="number" step="0.01" min="0" name="base_monthly_rent_usd" value="2000" required></label>
                <label><span data-i18n="view.rentesc.label.esc">Annual escalation (%)</span>
                    <input type="number" step="0.1" min="0" name="annual_escalation_pct" value="3" required></label>
                <label><span data-i18n="view.rentesc.label.term">Term (months)</span>
                    <input type="number" step="1" min="1" name="term_months" value="36" required></label>
                <label><span data-i18n="view.rentesc.label.free">Free months</span>
                    <input type="number" step="1" min="0" name="free_months" value="2"></label>
                <label><span data-i18n="view.rentesc.label.disc">Discount rate (%)</span>
                    <input type="number" step="0.01" min="0" name="discount_rate_pct" value="6"></label>
            </form>
        </div>
        <div id="rentesc-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#rentesc-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            base_monthly_rent_usd: Number(fd.get('base_monthly_rent_usd')) || 0,
            annual_escalation_pct: Number(fd.get('annual_escalation_pct')) || 0,
            term_months: Number(fd.get('term_months')) || 0,
            free_months: Number(fd.get('free_months')) || 0,
            discount_rate_pct: Number(fd.get('discount_rate_pct')) || 0,
        };
        try {
            const r = await api.calcRentEscalation(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.rentesc.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#rentesc-result');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.rentesc.h2.result">The cost</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.rentesc.card.effective">Effective rent / mo</div>
                    <div class="value pos">${money(r.effective_monthly_rent_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.rentesc.card.total">Total rent</div>
                    <div class="value">${money(r.total_rent_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.rentesc.card.npv">NPV</div>
                    <div class="value">${money(r.npv_usd)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.rentesc.row.total">Total rent</td><td>${money(r.total_rent_usd)}</td></tr>
                    <tr><td data-i18n="view.rentesc.row.npv">NPV of rent</td><td>${money(r.npv_usd)}</td></tr>
                    <tr><td data-i18n="view.rentesc.row.concession">Concession value</td><td>${money(r.concession_value_usd)}</td></tr>
                    <tr><td data-i18n="view.rentesc.row.final">Final month rent</td><td>${money(r.final_monthly_rent_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.rentesc.row.effective">Effective monthly rent</td><td>${money(r.effective_monthly_rent_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
