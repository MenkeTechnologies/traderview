// Rent roll — multi-unit rental income summary, via /calc/rent-roll.
import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
const SEED = [
    { label: 'A', rent: 1500, occ: true, sqft: 800 },
    { label: 'B', rent: 1600, occ: true, sqft: 850 },
    { label: 'C', rent: 1400, occ: false, sqft: 750 },
    { label: 'D', rent: 1800, occ: true, sqft: 1000 },
];
function rowHtml(u) {
    return `<div class="mpb-row rr-row">
        <input type="text" class="rr-label" value="${esc(u.label || '')}">
        <input type="number" step="50" min="0" class="rr-rent" value="${u.rent}">
        <input type="number" step="50" min="0" class="rr-sqft" value="${u.sqft}">
        <label class="rr-occ"><input type="checkbox" class="rr-occ-cb" ${u.occ ? 'checked' : ''}></label>
        <button type="button" class="rr-del" data-i18n="view.rentroll.remove">Remove</button></div>`;
}
export async function renderRentRoll(mount, _s) {
    const tok = currentViewToken(); if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.rentroll.h1.title">// RENT ROLL</span></h1>
        <p class="muted small" data-i18n="view.rentroll.hint.intro">A multi-unit rental income summary. From each unit's rent, occupancy, and square footage it computes scheduled vs actual rent, the vacancy loss, physical occupancy (units) and economic occupancy (rent), the annualized totals, and the average rent per square foot. Not investment advice.</p>
        <div class="lpv-split"><div class="chart-panel"><h2 data-i18n="view.rentroll.h2.inputs">Units</h2>
        <form id="rr-form" class="inline-form">
            <label><span data-i18n="view.rentroll.label.property">Property</span><input type="text" name="property_label" value="Maple Court"></label>
        </form>
        <div class="mpb-head rr-head"><span data-i18n="view.rentroll.col.unit">Unit</span><span data-i18n="view.rentroll.col.rent">Rent ($)</span><span data-i18n="view.rentroll.col.sqft">Sq ft</span><span data-i18n="view.rentroll.col.occ">Occ</span><span></span></div>
        <div id="rr-rows">${SEED.map(rowHtml).join('')}</div>
        <button type="button" id="rr-add" class="secondary" data-i18n="view.rentroll.add">+ Add unit</button>
        </div><div id="rr-result" class="chart-panel lpv-preview"></div></div>`;
    applyUiI18n(mount);
    const form = mount.querySelector('#rr-form'); const rowsEl = mount.querySelector('#rr-rows');
    const gen = async () => {
        const units = [...rowsEl.querySelectorAll('.rr-row')].map((r) => ({ label: (r.querySelector('.rr-label').value || '').trim(), monthly_rent_usd: Number(r.querySelector('.rr-rent').value) || 0, occupied: r.querySelector('.rr-occ-cb').checked, sqft: Number(r.querySelector('.rr-sqft').value) || 0 })).filter((u) => u.label);
        const body = { property_label: (form.querySelector('[name="property_label"]').value || '').trim(), units };
        if (!units.length) { mount.querySelector('#rr-result').innerHTML = ''; return; }
        try { const d = await api.calcRentRoll(body); if (!viewIsCurrent(tok)) return; res(mount, d); }
        catch (e) { showToast(e.message || t('view.rentroll.toast.error'), { level: 'error' }); }
    };
    const live = debounce(gen, 250);
    mount.querySelector('#rr-add').addEventListener('click', () => { rowsEl.insertAdjacentHTML('beforeend', rowHtml({ label: '', rent: 1500, occ: true, sqft: 800 })); applyUiI18n(rowsEl.lastElementChild); gen(); });
    rowsEl.addEventListener('click', (e) => { if (e.target.classList.contains('rr-del')) { e.target.closest('.rr-row').remove(); gen(); } });
    form.addEventListener('input', () => live()); rowsEl.addEventListener('input', () => live()); rowsEl.addEventListener('change', () => gen()); gen();
}
function res(mount, d) {
    const el = mount.querySelector('#rr-result');
    el.innerHTML = `<div class="lpv-bar"><div class="cards">
        <div class="card pos"><div class="label" data-i18n="view.rentroll.card.actual">Actual / month</div><div class="value">${money(d.actual_monthly_usd)}</div></div>
        <div class="card"><div class="label" data-i18n="view.rentroll.card.sched">Scheduled / month</div><div class="value">${money(d.scheduled_monthly_usd)}</div></div>
        <div class="card neg"><div class="label" data-i18n="view.rentroll.card.vac">Vacancy loss</div><div class="value">${money(d.vacancy_loss_monthly_usd)}</div></div>
        <div class="card"><div class="label" data-i18n="view.rentroll.card.phys">Physical occ.</div><div class="value">${pct(d.physical_occupancy_pct)} (${d.occupied_units}/${d.unit_count})</div></div>
        <div class="card"><div class="label" data-i18n="view.rentroll.card.econ">Economic occ.</div><div class="value">${pct(d.economic_occupancy_pct)}</div></div>
        <div class="card"><div class="label" data-i18n="view.rentroll.card.annual">Annual actual</div><div class="value">${money(d.annual_actual_usd)}</div></div>
        <div class="card"><div class="label" data-i18n="view.rentroll.card.psf">Rent / sq ft</div><div class="value">${money(d.avg_rent_per_sqft_usd)}</div></div>
    </div></div>`;
    applyUiI18n(el);
}
