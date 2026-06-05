// IRC § 174 — Mandatory R&D Capitalization (TCJA 2017, effective 2022).
// Pre-2022: deductible currently. Post-2022: 5-yr SL amortization (US) / 15-yr (foreign).
// Mid-year convention → year-1 deduction is only 10% of US QREs (50% / 60 mo × 12 mo).
// HUGE working-capital hit for R&D-heavy businesses. Sister to § 41 R&D Credit.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const US_AMORT_MONTHS = 60;
const FOREIGN_AMORT_MONTHS = 180;
const PRE_TCJA_DEDUCT_FULL = 1.00;

let state = {
    tax_year: new Date().getFullYear(),
    us_qre: 0,
    foreign_qre: 0,
    prior_year_us_qre: 0,
    prior_year_foreign_qre: 0,
    marginal_rate: 0.21,
    npv_discount: 0.06,
};

export async function renderSection174(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s174.h1.title">// § 174 R&D CAPITALIZATION</span></h1>
        <p class="muted small" data-i18n="view.s174.hint.intro">
            TCJA 2017 forced <strong>mandatory capitalization</strong> of R&D effective tax years
            after 12/31/2021. <strong>US-based R&D: 5-year SL (60 months); foreign: 15-year SL
            (180 months)</strong>. Mid-year convention: year-1 deduction is only <strong>10%
            of US QREs</strong> (half of 1/5). Huge working-capital hit for tech / pharma /
            startups. § 41 R&D Credit unaffected. Repeal efforts ongoing.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s174.h2.inputs">Inputs</h2>
            <form id="s174-form" class="inline-form">
                <label><span data-i18n="view.s174.label.year">Tax year</span>
                    <input type="number" step="1" name="tax_year" value="${state.tax_year}"></label>
                <label><span data-i18n="view.s174.label.us_qre">Current year US QREs ($)</span>
                    <input type="number" step="0.01" name="us_qre" value="${state.us_qre}"></label>
                <label><span data-i18n="view.s174.label.foreign_qre">Current year foreign QREs ($)</span>
                    <input type="number" step="0.01" name="foreign_qre" value="${state.foreign_qre}"></label>
                <label><span data-i18n="view.s174.label.prior_us">Prior year US QREs ($)</span>
                    <input type="number" step="0.01" name="prior_year_us_qre" value="${state.prior_year_us_qre}"></label>
                <label><span data-i18n="view.s174.label.prior_foreign">Prior year foreign QREs ($)</span>
                    <input type="number" step="0.01" name="prior_year_foreign_qre" value="${state.prior_year_foreign_qre}"></label>
                <label><span data-i18n="view.s174.label.marginal">Marginal rate</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <label><span data-i18n="view.s174.label.npv">NPV discount rate</span>
                    <input type="number" step="0.01" name="npv_discount" value="${state.npv_discount}"></label>
                <button class="primary" type="submit" data-i18n="view.s174.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s174-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s174.h2.qualifying">§ 174 qualifying R&D expenditures</h2>
            <ul class="muted small">
                <li data-i18n="view.s174.qual.wages">W-2 wages of researchers + supporting employees</li>
                <li data-i18n="view.s174.qual.contractors">Contractor R&D (65% allowed for credit; 100% capitalized)</li>
                <li data-i18n="view.s174.qual.supplies">Supplies consumed in R&D (chemicals, prototypes, sensors)</li>
                <li data-i18n="view.s174.qual.cloud">Cloud computing for R&D</li>
                <li data-i18n="view.s174.qual.software">In-house software development costs (Rev. Proc. 2000-50 prior, now § 174)</li>
                <li data-i18n="view.s174.qual.patents">Patent attorney fees (acquisition costs, NOT defense)</li>
                <li data-i18n="view.s174.qual.facility_alloc">Facility allocations supporting R&D activity</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s174.h2.transition">Transition + repeal context</h2>
            <ul class="muted small">
                <li data-i18n="view.s174.trans.first_year">First mandatory year: TY beginning after 12/31/2021</li>
                <li data-i18n="view.s174.trans.method_change">Automatic method change (DCN 265) — no IRS consent</li>
                <li data-i18n="view.s174.trans.481">No § 481(a) catch-up (cut-off transition)</li>
                <li data-i18n="view.s174.trans.repeal_efforts">Multiple bipartisan repeal bills (Tax Relief for American Families 2024)</li>
                <li data-i18n="view.s174.trans.retroactive">Likely retroactive if repealed — file protective extension</li>
                <li data-i18n="view.s174.trans.cash_impact">Industry impact: many startups forced into book/tax NOLs</li>
            </ul>
        </div>
    `;
    document.getElementById('s174-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.tax_year = Number(fd.get('tax_year')) || new Date().getFullYear();
        state.us_qre = Number(fd.get('us_qre')) || 0;
        state.foreign_qre = Number(fd.get('foreign_qre')) || 0;
        state.prior_year_us_qre = Number(fd.get('prior_year_us_qre')) || 0;
        state.prior_year_foreign_qre = Number(fd.get('prior_year_foreign_qre')) || 0;
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.21;
        state.npv_discount = Number(fd.get('npv_discount')) || 0.06;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s174-output');
    if (!el) return;
    const postTcja = state.tax_year >= 2022;
    const usY1 = postTcja ? state.us_qre * 0.10 : state.us_qre;
    const foreignY1 = postTcja ? state.foreign_qre * (1 / 30) : state.foreign_qre;
    const priorUsY2 = postTcja ? state.prior_year_us_qre * 0.20 : 0;
    const priorForeignY2 = postTcja ? state.prior_year_foreign_qre * (2 / 30) : 0;
    const currentYearDeduction = usY1 + foreignY1 + priorUsY2 + priorForeignY2;
    const wouldHaveDeductedFull = state.us_qre + state.foreign_qre;
    const deferralAmount = wouldHaveDeductedFull - usY1 - foreignY1;
    const taxDeferred = deferralAmount * state.marginal_rate;
    // NPV cost
    const usDeductionsByYear = [0.10, 0.20, 0.20, 0.20, 0.20, 0.10];
    const npvUs = usDeductionsByYear.reduce((s, frac, idx) =>
        s + (state.us_qre * frac * state.marginal_rate) / Math.pow(1 + state.npv_discount, idx), 0);
    const npvFullDeduct = state.us_qre * state.marginal_rate;
    const npvCost = npvFullDeduct - npvUs;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s174.h2.result">§ 174 amortization impact</h2>
            <div class="cards">
                <div class="card ${postTcja ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s174.card.regime">Post-TCJA capitalize</div>
                    <div class="value">${postTcja ? esc(t('view.s174.status.yes')) : esc(t('view.s174.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s174.card.us_year1">US year-1 deduction (10%)</div>
                    <div class="value">$${usY1.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s174.card.foreign_year1">Foreign year-1 deduction</div>
                    <div class="value">$${foreignY1.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s174.card.total_current">Total current-year deduction</div>
                    <div class="value">$${currentYearDeduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s174.card.would_have">Would have deducted pre-TCJA</div>
                    <div class="value">$${wouldHaveDeductedFull.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s174.card.deferral">Deferred deduction</div>
                    <div class="value">$${deferralAmount.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s174.card.cashflow">Year-1 cashflow hit</div>
                    <div class="value">$${taxDeferred.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s174.card.npv_cost">NPV cost (US 5-yr amort)</div>
                    <div class="value">$${npvCost.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
