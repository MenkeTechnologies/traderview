// R&D Tax Credit § 41 (Alternative Simplified Credit method).
// 14% of QREs above 50% of 3-year average. Trader-developed algos, backtesting
// frameworks, custom data pipelines all qualify if they pass the 4-part test:
// (1) permitted purpose, (2) elimination of uncertainty, (3) process of
// experimentation, (4) technological in nature. Also: payroll tax offset
// for startups (<5 yr) up to $500k/yr against employer FICA.

import { currentViewToken, viewIsCurrent } from '../app.js';

const ASC_RATE = 0.14;
const STARTUP_PAYROLL_CAP = 500_000;

let state = {
    current_year_qre: 0,
    prior_year_qre_1: 0,
    prior_year_qre_2: 0,
    prior_year_qre_3: 0,
    is_startup: false,
    years_in_business: 1,
    capitalize_174: true,  // Post-TCJA (2022+): mandatory 5-year amortization
};

export async function renderRdCredit(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.rd.h1.title">// R&D CREDIT § 41</span></h1>
        <p class="muted small" data-i18n="view.rd.hint.intro">
            Alternative Simplified Credit (ASC): <strong>14% of QREs above 50% of 3-year average</strong>.
            Trader algos, backtesting frameworks, ML models qualify if they pass the 4-part test.
            Startups (&lt; 5 yrs revenue) can offset up to $500k/yr against payroll tax instead
            of income tax.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.rd.h2.4_part_test">4-part qualification test</h2>
            <ol class="muted small">
                <li data-i18n="view.rd.test.purpose"><strong>Permitted purpose:</strong> develop new or improved product, process, or software</li>
                <li data-i18n="view.rd.test.uncertainty"><strong>Elimination of uncertainty:</strong> tech feasibility / methodology unknown at start</li>
                <li data-i18n="view.rd.test.experimentation"><strong>Process of experimentation:</strong> systematic trial-and-error / modeling</li>
                <li data-i18n="view.rd.test.technological"><strong>Technological in nature:</strong> CS / engineering / math / physical sciences</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.rd.h2.inputs">Inputs</h2>
            <form id="rd-form" class="inline-form">
                <label><span data-i18n="view.rd.label.current_year_qre">Current year QREs ($)</span>
                    <input type="number" step="0.01" name="current_year_qre" value="${state.current_year_qre}"></label>
                <label><span data-i18n="view.rd.label.prior_year_1">Prior year QREs ($)</span>
                    <input type="number" step="0.01" name="prior_year_qre_1" value="${state.prior_year_qre_1}"></label>
                <label><span data-i18n="view.rd.label.prior_year_2">Prior year -2 QREs ($)</span>
                    <input type="number" step="0.01" name="prior_year_qre_2" value="${state.prior_year_qre_2}"></label>
                <label><span data-i18n="view.rd.label.prior_year_3">Prior year -3 QREs ($)</span>
                    <input type="number" step="0.01" name="prior_year_qre_3" value="${state.prior_year_qre_3}"></label>
                <label><span data-i18n="view.rd.label.is_startup">Qualified small business (startup)?</span>
                    <input type="checkbox" name="is_startup" ${state.is_startup ? 'checked' : ''}></label>
                <label><span data-i18n="view.rd.label.years_in_business">Years with revenue</span>
                    <input type="number" step="1" name="years_in_business" value="${state.years_in_business}" min="0"></label>
                <button class="primary" type="submit" data-i18n="view.rd.btn.recompute">Recompute</button>
            </form>
        </div>
        <div id="rd-output"></div>
    `;
    document.getElementById('rd-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.current_year_qre = Number(fd.get('current_year_qre')) || 0;
        state.prior_year_qre_1 = Number(fd.get('prior_year_qre_1')) || 0;
        state.prior_year_qre_2 = Number(fd.get('prior_year_qre_2')) || 0;
        state.prior_year_qre_3 = Number(fd.get('prior_year_qre_3')) || 0;
        state.is_startup = !!fd.get('is_startup');
        state.years_in_business = Number(fd.get('years_in_business')) || 1;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('rd-output');
    if (!el) return;
    const priorYears = [state.prior_year_qre_1, state.prior_year_qre_2, state.prior_year_qre_3]
        .filter(v => v > 0);
    const priorAvg = priorYears.length > 0
        ? priorYears.reduce((a, b) => a + b, 0) / priorYears.length
        : 0;
    const halfPriorAvg = priorAvg * 0.50;
    const incrementalQRE = Math.max(0, state.current_year_qre - halfPriorAvg);
    // ASC rate: 14% if base period 3+ years, otherwise 6% (startup ASC)
    const ascRate = priorYears.length >= 3 ? ASC_RATE : 0.06;
    const credit = incrementalQRE * ascRate;
    const eligibleStartupOffset = state.is_startup && state.years_in_business <= 5
        ? Math.min(credit, STARTUP_PAYROLL_CAP) : 0;
    const incomeTaxOffset = credit - eligibleStartupOffset;
    // Post-TCJA 174 mandatory capitalization: QRE depreciated over 5 years
    // (15 for foreign R&D). Impact: smaller immediate income deduction, but
    // credit remains.
    const cap174AmortYear1 = state.current_year_qre / 10;  // mid-year convention = 10%
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.rd.h2.result">Credit calculation</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.rd.card.credit">R&D credit</div>
                    <div class="value">$${credit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.rd.card.asc_rate">ASC rate applied</div>
                    <div class="value">${(ascRate * 100).toFixed(0)}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.rd.card.incremental_qre">Incremental QRE</div>
                    <div class="value">$${incrementalQRE.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.rd.card.half_prior_avg">50% of 3-yr avg base</div>
                    <div class="value">$${halfPriorAvg.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${eligibleStartupOffset > 0 ? `
                    <div class="card pos">
                        <div class="label" data-i18n="view.rd.card.payroll_offset">Payroll tax offset</div>
                        <div class="value">$${eligibleStartupOffset.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
                <div class="card">
                    <div class="label" data-i18n="view.rd.card.income_tax_offset">Against income tax</div>
                    <div class="value">$${incomeTaxOffset.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.rd.h2.174_capitalization">§ 174 capitalization (post-TCJA 2022+)</h2>
            <p data-i18n="view.rd.cap_174.body">
                R&D expenses are no longer deductible in the year incurred — must be
                amortized over 5 years (15 for foreign). Year-1 deduction = 10% of QREs
                (mid-year convention). Big cash-flow impact on R&D-heavy businesses.
                Credit (Form 6765) is unaffected.
            </p>
            <p>
                <strong data-i18n="view.rd.cap_174.year_1">Year-1 § 174 amortization:</strong>
                $${cap174AmortYear1.toLocaleString(undefined, { maximumFractionDigits: 0 })}
            </p>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.rd.h2.qre_examples">QRE examples (trader-relevant)</h2>
            <ul class="muted small">
                <li data-i18n="view.rd.qre.algo">Algo trading model development + backtesting time</li>
                <li data-i18n="view.rd.qre.ml">ML model training (signals, NLP on news, sentiment)</li>
                <li data-i18n="view.rd.qre.data_pipeline">Custom data ingestion pipeline development</li>
                <li data-i18n="view.rd.qre.execution_engine">Smart-order-router / execution engine</li>
                <li data-i18n="view.rd.qre.risk_engine">Risk-management system development</li>
                <li data-i18n="view.rd.qre.contractor">65% of qualified contractor wages (vs 100% of W-2)</li>
                <li data-i18n="view.rd.qre.cloud">Cloud compute used for R&D (AWS / GCP / Lambda)</li>
                <li data-i18n="view.rd.qre.supplies">Tangible supplies consumed in development (sensors, FPGAs)</li>
            </ul>
        </div>
    `;
}
