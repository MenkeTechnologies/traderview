// Lease cost with escalations + free rent — total, NPV, and effective monthly
// rent, via /calc/rent-escalation. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const VIEW = 'rent-escalation';
let lastReport = null;
let lastBody = null;

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
            <div id="rentesc-tools" class="ce-toolbar"></div>
            <button type="button" id="rentesc-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
            <div id="rentesc-sens" class="ce-sens"></div>
        </div>
        <div id="rentesc-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#rentesc-form');
    enh.prefillForm(form, enh.readHashInputs());
    const readBody = () => {
        const fd = new FormData(form);
        return {
            base_monthly_rent_usd: Number(fd.get('base_monthly_rent_usd')) || 0,
            annual_escalation_pct: Number(fd.get('annual_escalation_pct')) || 0,
            term_months: Number(fd.get('term_months')) || 0,
            free_months: Number(fd.get('free_months')) || 0,
            discount_rate_pct: Number(fd.get('discount_rate_pct')) || 0,
        };
    };
    const generate = async () => {
        const body = readBody();
        try {
            const r = await api.calcRentEscalation(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || t('view.rentesc.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#rentesc-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'rent-escalation.csv' });
    mount.querySelector('#rentesc-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['effective_monthly_rent_usd', r.effective_monthly_rent_usd],
        ['total_rent_usd', r.total_rent_usd],
        ['npv_usd', r.npv_usd],
        ['concession_value_usd', r.concession_value_usd],
        ['final_monthly_rent_usd', r.final_monthly_rent_usd],
    ];
}

async function renderResult(mount, r, body, tok) {
    const el = mount.querySelector('#rentesc-result');
    // Line chart: effective monthly rent as the annual escalation sweeps 0 -> 10%.
    const xs = enh.linspace(0, 10, 13);
    const pts = await Promise.all(xs.map(async (e) => {
        const rr = await api.calcRentEscalation({ ...body, annual_escalation_pct: e });
        return { x: e, y: rr ? rr.effective_monthly_rent_usd : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'escalation %', ylabel: 'eff rent $/mo' });
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
            ${chart}
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

async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#rentesc-sens');
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    // x: annual escalation 0 -> 10%; y: free months 0 -> 6. Output: effective monthly rent.
    const xVals = enh.linspace(0, 10, 5);
    const yVals = enh.linspace(0, 6, 5);
    const { cells } = await enh.runSensitivity({ base, xKey: 'annual_escalation_pct', yKey: 'free_months', xVals, yVals: yVals.map(Math.round), compute: (b) => api.calcRentEscalation(b), pick: (r) => (r ? r.effective_monthly_rent_usd : null) });
    if (!viewIsCurrent(tok)) return;
    // Lower effective rent is better (tenant), so negate for shading (green = cheaper).
    panel.innerHTML = enh.renderSensitivityTable({ xVals, yVals: yVals.map(Math.round), cells: cells.map((row) => row.map((v) => (v == null ? null : -v))), fmt: (v) => (v == null ? '—' : '$' + Math.round(-v)), xfmt: (v) => v.toFixed(0) + '%', yfmt: (v) => v.toFixed(0) + 'mo', xName: t('view.rentesc.label.esc') || 'Escal', yName: t('view.rentesc.label.free') || 'Free' });
}
