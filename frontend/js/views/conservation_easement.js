// Conservation Easement § 170(h) Deduction Tracker.
// Charitable deduction for donating qualified real-property interest to qualified
// org for conservation. 50% AGI limit (100% for "qualified farmers/ranchers").
// 15-year carryforward of unused deduction. Heavy IRS scrutiny on syndicated deals.
// Listed Transactions (Notice 2017-10): syndicated deals with 2.5×+ ratio require
// Form 8886 disclosure + risk audit penalties.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const AGI_LIMIT_INDIVIDUAL = 0.50;
const AGI_LIMIT_FARMER = 1.00;
const CARRYFORWARD_YEARS = 15;
const SYNDICATED_RATIO_THRESHOLD = 2.5;

let state = {
    agi: 500_000,
    is_farmer: false,
    appraised_value: 1_000_000,
    cost_basis: 200_000,
    is_syndicated: false,
    promoter_offering_ratio: 4.0,
    your_marginal_rate: 0.37,
    state_rate: 0.05,
    investment_in_syndication: 100_000,
};

export async function renderConservationEasement(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ce.h1.title">// CONSERVATION EASEMENT § 170(h)</span></h1>
        <p class="muted small" data-i18n="view.ce.hint.intro">
            <strong>50% AGI</strong> charitable deduction (100% for qualified farmers).
            15-year carryforward. Donate a permanent restriction on real property to
            qualified org (land trust). <strong>Syndicated deals with 2.5×+ deduction
            ratio = Listed Transaction</strong> (Notice 2017-10) — Form 8886 required +
            high audit risk + 40% accuracy-related penalty if disallowed.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.ce.h2.inputs">Inputs</h2>
            <form id="ce-form" class="inline-form">
                <label><span data-i18n="view.ce.label.agi">AGI ($)</span>
                    <input type="number" step="0.01" name="agi" value="${state.agi}"></label>
                <label><span data-i18n="view.ce.label.is_farmer">Qualified farmer/rancher?</span>
                    <input type="checkbox" name="is_farmer" ${state.is_farmer ? 'checked' : ''}></label>
                <label><span data-i18n="view.ce.label.appraised_value">Appraised easement value ($)</span>
                    <input type="number" step="0.01" name="appraised_value" value="${state.appraised_value}"></label>
                <label><span data-i18n="view.ce.label.cost_basis">Cost basis in property ($)</span>
                    <input type="number" step="0.01" name="cost_basis" value="${state.cost_basis}"></label>
                <label><span data-i18n="view.ce.label.is_syndicated">Syndicated deal?</span>
                    <input type="checkbox" name="is_syndicated" ${state.is_syndicated ? 'checked' : ''}></label>
                <label><span data-i18n="view.ce.label.promoter_ratio">Promoter offering ratio (deduction / investment)</span>
                    <input type="number" step="0.1" name="promoter_offering_ratio" value="${state.promoter_offering_ratio}"></label>
                <label><span data-i18n="view.ce.label.investment">Your investment ($)</span>
                    <input type="number" step="0.01" name="investment_in_syndication" value="${state.investment_in_syndication}"></label>
                <label><span data-i18n="view.ce.label.your_marginal_rate">Marginal federal %</span>
                    <input type="number" step="0.5" name="your_marginal_rate" value="${(state.your_marginal_rate * 100).toFixed(1)}"></label>
                <label><span data-i18n="view.ce.label.state_rate">State %</span>
                    <input type="number" step="0.5" name="state_rate" value="${(state.state_rate * 100).toFixed(1)}"></label>
                <button class="primary" type="submit" data-i18n="view.ce.btn.compute">Compute</button>
            </form>
        </div>
        <div id="ce-output"></div>
    `;
    document.getElementById('ce-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.agi = Number(fd.get('agi')) || 0;
        state.is_farmer = !!fd.get('is_farmer');
        state.appraised_value = Number(fd.get('appraised_value')) || 0;
        state.cost_basis = Number(fd.get('cost_basis')) || 0;
        state.is_syndicated = !!fd.get('is_syndicated');
        state.promoter_offering_ratio = Number(fd.get('promoter_offering_ratio')) || 0;
        state.investment_in_syndication = Number(fd.get('investment_in_syndication')) || 0;
        state.your_marginal_rate = (Number(fd.get('your_marginal_rate')) || 37) / 100;
        state.state_rate = (Number(fd.get('state_rate')) || 0) / 100;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('ce-output');
    if (!el) return;
    const agiCap = state.is_farmer ? AGI_LIMIT_FARMER : AGI_LIMIT_INDIVIDUAL;
    const deductionCap = state.agi * agiCap;
    const yearDeduction = Math.min(state.appraised_value, deductionCap);
    const carryforward = Math.max(0, state.appraised_value - yearDeduction);
    const taxSavings = state.appraised_value * (state.your_marginal_rate + state.state_rate);
    const netCost = state.is_syndicated ? state.investment_in_syndication - taxSavings : -taxSavings;
    const isListedTransaction = state.is_syndicated && state.promoter_offering_ratio >= SYNDICATED_RATIO_THRESHOLD;
    el.innerHTML = `
        <div class="chart-panel ${isListedTransaction ? 'neg' : ''}">
            <h2 data-i18n="view.ce.h2.result">Deduction calculation</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.ce.card.appraised">Appraised value</div>
                    <div class="value">$${state.appraised_value.toLocaleString()}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.ce.card.agi_cap">AGI cap (${(agiCap * 100).toFixed(0)}%)</div>
                    <div class="value">$${deductionCap.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.ce.card.year1_deduction">Year-1 deduction</div>
                    <div class="value">$${yearDeduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.ce.card.carryforward">Carryforward (15 yr)</div>
                    <div class="value">$${carryforward.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.ce.card.tax_savings">Total tax savings (over 15 yr)</div>
                    <div class="value">$${taxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${state.is_syndicated ? `
                    <div class="card ${netCost < 0 ? 'pos' : 'neg'}">
                        <div class="label" data-i18n="view.ce.card.net_cost">Net cost (or gain)</div>
                        <div class="value">$${netCost.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
                ${isListedTransaction ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.ce.card.listed_transaction">⚠ LISTED TRANSACTION</div>
                        <div class="value">${esc(t('view.ce.warning.listed'))}</div>
                    </div>
                ` : ''}
            </div>
        </div>
        ${isListedTransaction ? `
            <div class="chart-panel neg">
                <h2 data-i18n="view.ce.h2.listed_warning">⚠ Syndicated easement — AUDIT MAGNET</h2>
                <p data-i18n="view.ce.listed_warning_body">
                    Notice 2017-10 and Notice 2023-30 listed syndicated conservation easements
                    as "Listed Transactions." IRS won 90%+ of these cases at Tax Court (Bosque
                    Canyon, Mill Road, Plateau Holdings, etc.). Form 8886 required for every
                    investor + every promoter. 40% accuracy-related penalty if disallowed.
                    Inflation Reduction Act 2022 capped deduction at 2.5× basis for
                    pass-through syndications. Most CPAs will refuse to sign these.
                </p>
            </div>
        ` : ''}
        <div class="chart-panel">
            <h2 data-i18n="view.ce.h2.requirements">Qualification requirements</h2>
            <ol class="muted small">
                <li data-i18n="view.ce.req.qualified_org">Donee must be qualified organization (governmental unit OR 501(c)(3) land trust)</li>
                <li data-i18n="view.ce.req.qualified_purpose">Conservation purpose: outdoor recreation, natural habitat, open space, historic preservation</li>
                <li data-i18n="view.ce.req.perpetuity">Restriction must be PERPETUAL (binds future owners forever)</li>
                <li data-i18n="view.ce.req.appraisal">Qualified appraisal required (deductions &gt; $500,000 attach to return)</li>
                <li data-i18n="view.ce.req.baseline">Baseline documentation report at time of donation</li>
                <li data-i18n="view.ce.req.real_property">Real property interest — easements, remainder interests, restrictions</li>
                <li data-i18n="view.ce.req.surface_water">Mineral interests retained must not allow surface mining</li>
            </ol>
        </div>
    `;
}
