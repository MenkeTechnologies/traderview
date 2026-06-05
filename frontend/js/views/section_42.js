// IRC § 42 — Low-Income Housing Tax Credit (LIHTC).
// 4% credit (bond-financed) OR 9% credit (competitive allocation by state HFA).
// 10-year credit period; 15-year compliance period (30+ years extended use).
// Two threshold tests: 20-50 (≥ 20% units at ≤ 50% AMI) or 40-60 (≥ 40% at ≤ 60%) or post-2017 income averaging (40% at average ≤ 60% AMI).
// Recapture 1/3 over 15 yrs if compliance fails.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    qualified_basis: 0,
    credit_rate_pct: 9,
    is_competitive_9pct: true,
    is_4pct_bond: false,
    threshold_test: 'income_averaging',
    qualified_units_low_income: 0,
    total_units: 0,
    avg_ami_pct: 60,
    qct_dda_bonus: false,
    placed_in_service_year: 2024,
    compliance_period_yr: 0,
    cure_period: false,
    qualified_allocation_plan_score: 0,
};

export async function renderSection42(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s42.h1.title">// § 42 LIHTC</span></h1>
        <p class="muted small" data-i18n="view.s42.hint.intro">
            <strong>4% credit</strong> (bond-financed) OR <strong>9% credit</strong> (competitive allocation
            by state HFA). <strong>10-year credit period</strong>; <strong>15-year compliance</strong>; 30+
            year extended use agreement. <strong>Threshold tests:</strong> (1) <strong>20-50</strong> (≥ 20%
            units at ≤ 50% AMI), (2) <strong>40-60</strong> (≥ 40% at ≤ 60%), (3) post-2017 <strong>income
            averaging</strong> (≥ 40% at avg ≤ 60% AMI; units range 20-80%). <strong>Recapture 1/3 over 15
            yrs</strong> on noncompliance. <strong>QCT / DDA bonus +30%</strong> qualified basis. Form 8609.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s42.h2.inputs">Inputs</h2>
            <form id="s42-form" class="inline-form">
                <label><span data-i18n="view.s42.label.basis">Qualified basis ($)</span>
                    <input type="number" step="0.01" name="qualified_basis" value="${state.qualified_basis}"></label>
                <label><span data-i18n="view.s42.label.rate">Credit rate %</span>
                    <input type="number" step="0.1" name="credit_rate_pct" value="${state.credit_rate_pct}"></label>
                <label><span data-i18n="view.s42.label.9pct">9% competitive allocation?</span>
                    <input type="checkbox" name="is_competitive_9pct" ${state.is_competitive_9pct ? 'checked' : ''}></label>
                <label><span data-i18n="view.s42.label.4pct">4% bond-financed?</span>
                    <input type="checkbox" name="is_4pct_bond" ${state.is_4pct_bond ? 'checked' : ''}></label>
                <label><span data-i18n="view.s42.label.test">Threshold test</span>
                    <select name="threshold_test">
                        <option value="20_50" ${state.threshold_test === '20_50' ? 'selected' : ''}>20-50 (≥ 20% at ≤ 50% AMI)</option>
                        <option value="40_60" ${state.threshold_test === '40_60' ? 'selected' : ''}>40-60 (≥ 40% at ≤ 60%)</option>
                        <option value="income_averaging" ${state.threshold_test === 'income_averaging' ? 'selected' : ''}>Income averaging (40% avg ≤ 60%)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s42.label.low_units">Low-income units</span>
                    <input type="number" step="1" name="qualified_units_low_income" value="${state.qualified_units_low_income}"></label>
                <label><span data-i18n="view.s42.label.total_units">Total units</span>
                    <input type="number" step="1" name="total_units" value="${state.total_units}"></label>
                <label><span data-i18n="view.s42.label.avg_ami">Avg AMI %</span>
                    <input type="number" step="0.1" name="avg_ami_pct" value="${state.avg_ami_pct}"></label>
                <label><span data-i18n="view.s42.label.qct">QCT / DDA bonus (+30%)?</span>
                    <input type="checkbox" name="qct_dda_bonus" ${state.qct_dda_bonus ? 'checked' : ''}></label>
                <label><span data-i18n="view.s42.label.year">Placed in service year</span>
                    <input type="number" step="1" name="placed_in_service_year" value="${state.placed_in_service_year}"></label>
                <label><span data-i18n="view.s42.label.compliance">Year in 15-yr compliance period</span>
                    <input type="number" step="1" name="compliance_period_yr" value="${state.compliance_period_yr}"></label>
                <label><span data-i18n="view.s42.label.cure">Cure period available?</span>
                    <input type="checkbox" name="cure_period" ${state.cure_period ? 'checked' : ''}></label>
                <label><span data-i18n="view.s42.label.qap">QAP score (state competitive)</span>
                    <input type="number" step="0.1" name="qualified_allocation_plan_score" value="${state.qualified_allocation_plan_score}"></label>
                <button class="primary" type="submit" data-i18n="view.s42.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s42-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s42.h2.structure">LIHTC structure</h2>
            <ul class="muted small">
                <li data-i18n="view.s42.str.9pct">9% credit: NEW construction / substantial rehab (rehab ≥ $7K/unit + 10% basis)</li>
                <li data-i18n="view.s42.str.4pct">4% credit: tax-exempt bond-financed (≥ 50% costs from bonds)</li>
                <li data-i18n="view.s42.str.10yr">10-year credit period: claim 1/10 each year</li>
                <li data-i18n="view.s42.str.15yr">15-year compliance period: maintain affordability or recapture</li>
                <li data-i18n="view.s42.str.30yr">30-year extended use: state agreement, includes early restrictions</li>
                <li data-i18n="view.s42.str.qap">State Qualified Allocation Plan (QAP): scoring for competitive 9%</li>
                <li data-i18n="view.s42.str.basis_bonus">QCT/DDA boost: +30% qualified basis (eligible basis × 130%)</li>
                <li data-i18n="view.s42.str.partner_partnership">Typical structure: 99.99% investor + 0.01% sponsor in Limited Partnership</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s42.h2.tests">Threshold tests (§ 42(g))</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s42.th.test">Test</th>
                    <th data-i18n="view.s42.th.min_units">Min low-income</th>
                    <th data-i18n="view.s42.th.max_income">Max income limit</th>
                </tr></thead>
                <tbody>
                    <tr><td>20-50</td><td>≥ 20% of units</td><td>≤ 50% AMI</td></tr>
                    <tr><td>40-60</td><td>≥ 40% of units</td><td>≤ 60% AMI</td></tr>
                    <tr><td>Income Averaging (post-2017)</td><td>≥ 40% of units</td><td>Avg ≤ 60% AMI; units 20-80% AMI</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s42.h2.compliance">Compliance + recapture</h2>
            <ul class="muted small">
                <li data-i18n="view.s42.comp.next_available">Next Available Unit Rule (§ 42(g)(2)(D)): rent next available to qualified tenant if existing falls out of compliance</li>
                <li data-i18n="view.s42.comp.recertification">Annual tenant recertification — verify income still meets test</li>
                <li data-i18n="view.s42.comp.cure_period">90-day cure period for noncompliance — fix before annual report</li>
                <li data-i18n="view.s42.comp.s42j">Annual Form 8609 + Schedule A by state HFA</li>
                <li data-i18n="view.s42.comp.recapture_pct">Recapture: 1/3 of credit claimed for each year out of compliance + interest</li>
                <li data-i18n="view.s42.comp.exception">Recapture exceptions: casualty, partner death, change of qualified basis (de minimis)</li>
                <li data-i18n="view.s42.comp.5_year">First 5 years: heavier penalties; years 11-15 lighter</li>
                <li data-i18n="view.s42.comp.exit_strategy">Exit strategy: year 15 dispose to sponsor / nonprofit + Right of First Refusal</li>
            </ul>
        </div>
    `;
    document.getElementById('s42-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.qualified_basis = Number(fd.get('qualified_basis')) || 0;
        state.credit_rate_pct = Number(fd.get('credit_rate_pct')) || 0;
        state.is_competitive_9pct = !!fd.get('is_competitive_9pct');
        state.is_4pct_bond = !!fd.get('is_4pct_bond');
        state.threshold_test = fd.get('threshold_test');
        state.qualified_units_low_income = Number(fd.get('qualified_units_low_income')) || 0;
        state.total_units = Number(fd.get('total_units')) || 0;
        state.avg_ami_pct = Number(fd.get('avg_ami_pct')) || 0;
        state.qct_dda_bonus = !!fd.get('qct_dda_bonus');
        state.placed_in_service_year = Number(fd.get('placed_in_service_year')) || 0;
        state.compliance_period_yr = Number(fd.get('compliance_period_yr')) || 0;
        state.cure_period = !!fd.get('cure_period');
        state.qualified_allocation_plan_score = Number(fd.get('qualified_allocation_plan_score')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s42-output');
    if (!el) return;
    const lowPct = state.total_units > 0 ? (state.qualified_units_low_income / state.total_units * 100) : 0;
    let testMet = false;
    if (state.threshold_test === '20_50' && lowPct >= 20 && state.avg_ami_pct <= 50) testMet = true;
    else if (state.threshold_test === '40_60' && lowPct >= 40 && state.avg_ami_pct <= 60) testMet = true;
    else if (state.threshold_test === 'income_averaging' && lowPct >= 40 && state.avg_ami_pct <= 60) testMet = true;
    const bonusMultiplier = state.qct_dda_bonus ? 1.30 : 1.0;
    const adjustedBasis = state.qualified_basis * bonusMultiplier;
    const annualCredit = adjustedBasis * (state.credit_rate_pct / 100);
    const tenYearCredit = annualCredit * 10;
    const investorPrice = 0.92;
    const equityRaised = tenYearCredit * investorPrice;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s42.h2.result">§ 42 LIHTC computation</h2>
            <div class="cards">
                <div class="card ${testMet ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s42.card.test_met">Threshold test met?</div>
                    <div class="value">${testMet ? esc(t('view.s42.status.yes')) : esc(t('view.s42.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s42.card.pct">Low-income unit %</div>
                    <div class="value">${lowPct.toFixed(1)}%</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s42.card.qct">QCT/DDA boost</div>
                    <div class="value">${state.qct_dda_bonus ? '+30%' : '0%'}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s42.card.adj_basis">Adjusted basis</div>
                    <div class="value">$${adjustedBasis.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s42.card.annual">Annual credit</div>
                    <div class="value">$${annualCredit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s42.card.ten_year">10-year credit</div>
                    <div class="value">$${tenYearCredit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s42.card.equity">Equity raised (92¢)</div>
                    <div class="value">$${equityRaised.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.threshold_test === 'income_averaging' && testMet ? `
                <p class="muted small pos" style="margin-top:10px" data-i18n="view.s42.avg_note">
                    Income averaging (post-2017 PATH Act): provides FLEXIBILITY to serve mixed-income
                    populations. Average AMI ≤ 60% allows units up to 80% AMI to subsidize units at 20% AMI.
                    Available for compliance period 2018+; required Treasury final regs Oct 2022.
                </p>
            ` : ''}
        </div>
    `;
}
