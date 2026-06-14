// Standard vs itemized deduction — whichever deduction is larger, via
// /calc/standard-vs-itemized. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 }));
const signed = (n) => (n == null ? '—' : (n >= 0 ? '+$' : '−$') + Math.abs(Number(n)).toLocaleString(undefined, { maximumFractionDigits: 0 }));
const VIEW = 'standard-vs-itemized';
let lastReport = null;
let lastBody = null;

export async function renderStdVsItemized(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.itemize.h1.title">// STANDARD VS ITEMIZED</span></h1>
        <p class="muted small" data-i18n="view.itemize.hint.intro">
            Itemized deductions are SALT (state/local + property tax, capped) plus mortgage interest,
            charitable gifts, and medical costs above 7.5% of AGI. You take whichever is larger — the
            standard deduction or the itemized total. 2026 defaults: $16,100 single / $32,200 MFJ
            standard, $40,400 SALT cap (phasing down above $505k AGI). Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.itemize.h2.inputs">Your deductions</h2>
            <form id="itemize-form" class="inline-form">
                <label><span data-i18n="view.itemize.label.agi">AGI ($)</span>
                    <input type="number" step="1000" min="0" name="agi_usd" value="200000" required></label>
                <label><span data-i18n="view.itemize.label.status">Filing status</span>
                    <select name="filing_status">
                        <option value="single" data-i18n="view.itemize.status.single">Single</option>
                        <option value="married_joint" selected data-i18n="view.itemize.status.mfj">Married filing jointly</option>
                    </select></label>
                <label><span data-i18n="view.itemize.label.salt">State/local income tax ($)</span>
                    <input type="number" step="500" min="0" name="state_local_tax_usd" value="20000"></label>
                <label><span data-i18n="view.itemize.label.property">Property tax ($)</span>
                    <input type="number" step="500" min="0" name="property_tax_usd" value="10000"></label>
                <label><span data-i18n="view.itemize.label.mortgage">Mortgage interest ($)</span>
                    <input type="number" step="500" min="0" name="mortgage_interest_usd" value="18000"></label>
                <label><span data-i18n="view.itemize.label.charitable">Charitable gifts ($)</span>
                    <input type="number" step="500" min="0" name="charitable_usd" value="5000"></label>
                <label><span data-i18n="view.itemize.label.medical">Medical expenses ($)</span>
                    <input type="number" step="500" min="0" name="medical_usd" value="0"></label>
            </form>
            <div id="itemize-tools" class="ce-toolbar"></div>
        </div>
        <div id="itemize-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#itemize-form');
    enh.prefillForm(form, enh.readHashInputs());
    const readBody = () => {
        const fd = new FormData(form);
        return {
            agi_usd: Number(fd.get('agi_usd')) || 0,
            filing_status: fd.get('filing_status'),
            state_local_tax_usd: Number(fd.get('state_local_tax_usd')) || 0,
            property_tax_usd: Number(fd.get('property_tax_usd')) || 0,
            mortgage_interest_usd: Number(fd.get('mortgage_interest_usd')) || 0,
            charitable_usd: Number(fd.get('charitable_usd')) || 0,
            medical_usd: Number(fd.get('medical_usd')) || 0,
        };
    };
    const generate = async () => {
        const body = readBody();
        try {
            const r = await api.calcStdVsItemized(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.itemize.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#itemize-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'standard-vs-itemized.csv' });
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['deduction_taken_usd', r.deduction_taken_usd],
        ['standard_deduction_usd', r.standard_deduction_usd],
        ['itemized_total_usd', r.itemized_total_usd],
        ['itemizing_advantage_usd', r.itemizing_advantage_usd],
        ['salt_deductible_usd', r.salt_deductible_usd],
        ['medical_deductible_usd', r.medical_deductible_usd],
    ];
}

function renderResult(mount, r) {
    const el = mount.querySelector('#itemize-result');
    const cls = r.should_itemize ? 'pos' : '';
    // Standard vs itemized deduction comparison bars (the larger wins).
    const chart = enh.svgBarChart([
        { label: 'Standard', value: r.standard_deduction_usd },
        { label: 'Itemized', value: r.itemized_total_usd },
    ]);
    const verdictKey = r.should_itemize ? 'view.itemize.verdict.itemize' : 'view.itemize.verdict.standard';
    const saltLostRow = r.salt_lost_usd > 0
        ? `<tr class="neg"><td data-i18n="view.itemize.row.salt_lost">SALT lost over cap</td><td>${money(r.salt_lost_usd)}</td></tr>`
        : '';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.itemize.h2.result">The choice</h2>
            <div class="cards">
                <div class="card ${cls}"><div class="label" data-i18n="${verdictKey}">—</div>
                    <div class="value ${cls}">${money(r.deduction_taken_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.itemize.card.standard">Standard</div>
                    <div class="value">${money(r.standard_deduction_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.itemize.card.itemized">Itemized</div>
                    <div class="value">${money(r.itemized_total_usd)}</div></div>
            </div>
            ${chart}
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.itemize.row.salt_paid">SALT paid</td><td>${money(r.salt_paid_usd)}</td></tr>
                    <tr><td data-i18n="view.itemize.row.salt_cap">SALT cap (after phase-out)</td><td>${money(r.salt_cap_usd)}</td></tr>
                    <tr><td data-i18n="view.itemize.row.salt_deductible">SALT deductible</td><td>${money(r.salt_deductible_usd)}</td></tr>
                    ${saltLostRow}
                    <tr><td data-i18n="view.itemize.row.medical_floor">Medical floor (7.5% AGI)</td><td>${money(r.medical_floor_usd)}</td></tr>
                    <tr><td data-i18n="view.itemize.row.medical_deductible">Medical deductible</td><td>${money(r.medical_deductible_usd)}</td></tr>
                    <tr><td data-i18n="view.itemize.row.itemized">Itemized total</td><td>${money(r.itemized_total_usd)}</td></tr>
                    <tr><td data-i18n="view.itemize.row.standard">Standard deduction</td><td>${money(r.standard_deduction_usd)}</td></tr>
                    <tr class="emph ${cls}"><td data-i18n="view.itemize.row.advantage">Itemizing advantage</td><td>${signed(r.itemizing_advantage_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
