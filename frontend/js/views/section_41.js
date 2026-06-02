// IRC § 41 — Research Credit (R&D Tax Credit).
// 20% incremental: 20% × (Current QREs − Base Amount). Base = avg gross receipts × fixed base %.
// ASC (Alternative Simplified Credit) § 41(c)(5): 14% × (Current QREs − 50% × avg of prior 3 yrs).
// Payroll tax election: qualified small biz can use up to $500K credit against employer FICA.
// QREs = wages + supplies + contract research (65%) + computer time-share for qualified research.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    current_year_qre: 0,
    avg_gross_receipts_4yr: 0,
    fixed_base_percentage: 0,
    avg_qre_prior_3yr: 0,
    credit_method: 'asc',
    wages_for_qualified: 0,
    supplies_for_qualified: 0,
    contract_research: 0,
    computer_time_share: 0,
    is_qualified_small_business: false,
    payroll_tax_election: false,
    is_startup_pre_revenue: false,
    s174_amortization: 0,
};

export async function renderSection41(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s41.h1.title">// § 41 RESEARCH CREDIT</span></h1>
        <p class="muted small" data-i18n="view.s41.hint.intro">
            <strong>Regular method:</strong> 20% × (Current QREs − Base Amount). Base = avg gross receipts ×
            fixed base %. <strong>ASC § 41(c)(5):</strong> 14% × (Current QREs − 50% × avg prior 3 yrs).
            <strong>Payroll tax election:</strong> qualified small biz (≤ $5M current revenue + ≤ 5 yrs revenue)
            uses up to <strong>$500K</strong> credit against employer FICA (PATH Act 2015). <strong>QREs:</strong>
            wages + supplies + contract research (65%) + computer time-share. <strong>§ 174:</strong> 2022+
            mandatory 5-yr / 15-yr amortization replaces immediate expensing.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s41.h2.inputs">Inputs</h2>
            <form id="s41-form" class="inline-form">
                <label><span data-i18n="view.s41.label.current_qre">Current year QRE ($)</span>
                    <input type="number" step="10000" name="current_year_qre" value="${state.current_year_qre}"></label>
                <label><span data-i18n="view.s41.label.avg_receipts">Avg gross receipts (4-yr) ($)</span>
                    <input type="number" step="100000" name="avg_gross_receipts_4yr" value="${state.avg_gross_receipts_4yr}"></label>
                <label><span data-i18n="view.s41.label.fixed_base">Fixed base %</span>
                    <input type="number" step="0.001" name="fixed_base_percentage" value="${state.fixed_base_percentage}"></label>
                <label><span data-i18n="view.s41.label.avg_qre">Avg prior 3-yr QRE ($)</span>
                    <input type="number" step="10000" name="avg_qre_prior_3yr" value="${state.avg_qre_prior_3yr}"></label>
                <label><span data-i18n="view.s41.label.method">Credit method</span>
                    <select name="credit_method">
                        <option value="asc" ${state.credit_method === 'asc' ? 'selected' : ''}>ASC § 41(c)(5) — 14%</option>
                        <option value="regular" ${state.credit_method === 'regular' ? 'selected' : ''}>Regular — 20%</option>
                    </select>
                </label>
                <label><span data-i18n="view.s41.label.wages">Wages for qualified research ($)</span>
                    <input type="number" step="10000" name="wages_for_qualified" value="${state.wages_for_qualified}"></label>
                <label><span data-i18n="view.s41.label.supplies">Supplies for qualified research ($)</span>
                    <input type="number" step="10000" name="supplies_for_qualified" value="${state.supplies_for_qualified}"></label>
                <label><span data-i18n="view.s41.label.contract">Contract research × 65% ($)</span>
                    <input type="number" step="10000" name="contract_research" value="${state.contract_research}"></label>
                <label><span data-i18n="view.s41.label.computer">Computer time-share ($)</span>
                    <input type="number" step="10000" name="computer_time_share" value="${state.computer_time_share}"></label>
                <label><span data-i18n="view.s41.label.qsb">Qualified small biz ≤ $5M + ≤ 5 yrs?</span>
                    <input type="checkbox" name="is_qualified_small_business" ${state.is_qualified_small_business ? 'checked' : ''}></label>
                <label><span data-i18n="view.s41.label.payroll">Payroll tax election ($500K)?</span>
                    <input type="checkbox" name="payroll_tax_election" ${state.payroll_tax_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s41.label.startup">Startup pre-revenue?</span>
                    <input type="checkbox" name="is_startup_pre_revenue" ${state.is_startup_pre_revenue ? 'checked' : ''}></label>
                <label><span data-i18n="view.s41.label.s174">§ 174 amortization (current yr) ($)</span>
                    <input type="number" step="10000" name="s174_amortization" value="${state.s174_amortization}"></label>
                <button class="primary" type="submit" data-i18n="view.s41.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s41-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s41.h2.four_part">§ 41(d) "4-part test" for qualified research</h2>
            <ol class="muted small">
                <li data-i18n="view.s41.tst.permitted_purpose">PERMITTED PURPOSE: new / improved business component (function, performance, reliability, quality)</li>
                <li data-i18n="view.s41.tst.tech_uncertainty">TECHNOLOGICAL UNCERTAINTY: develop info to eliminate uncertainty about capability / method / design</li>
                <li data-i18n="view.s41.tst.experimentation">PROCESS OF EXPERIMENTATION: evaluate alternatives via systematic process (hypothesis → test → analyze)</li>
                <li data-i18n="view.s41.tst.technological">TECHNOLOGICAL IN NATURE: rely on hard sciences (engineering, physics, biology, computer science, chemistry)</li>
                <li data-i18n="view.s41.tst.all_four">ALL FOUR required — fact-intensive determination, contemporaneous documentation critical</li>
                <li data-i18n="view.s41.tst.exclusions">Exclusions § 41(d)(4): post-commercial production, adaptation, duplication, surveys, foreign research, software for internal use (3-part test)</li>
                <li data-i18n="view.s41.tst.documentation">Document: project objectives, technical challenges, alternatives evaluated, test results, time tracking</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s41.h2.qres">QRE categories</h2>
            <ul class="muted small">
                <li data-i18n="view.s41.qre.wages">Wages: W-2 Box 1 of employees engaged in qualified research (direct + supervision + support)</li>
                <li data-i18n="view.s41.qre.supplies">Supplies: tangible property used in research (NOT capital assets &gt; 1 yr)</li>
                <li data-i18n="view.s41.qre.contract">Contract research: 65% of payments to contractors (75% for qualified energy research consortium)</li>
                <li data-i18n="view.s41.qre.computer">Computer time-share: cost of cloud computing + hosted services for qualified research</li>
                <li data-i18n="view.s41.qre.no_capital">Cost of fixed assets NOT QRE — depreciation already deducted under § 168</li>
                <li data-i18n="view.s41.qre.no_overhead">Overhead, indirect costs, depreciation NOT QRE</li>
                <li data-i18n="view.s41.qre.no_funded">Funded research (paid by 3rd party) NOT QRE for performer</li>
                <li data-i18n="view.s41.qre.basic_research">Basic research: separate § 41(b)(3) treatment — credit at 20% direct</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s41.h2.s280c">§ 280C interaction</h2>
            <ul class="muted small">
                <li data-i18n="view.s41.s280c.basic">§ 280C(c): deduction reduced by credit amount (no double benefit)</li>
                <li data-i18n="view.s41.s280c.reduced">§ 280C(c)(2) reduced credit election: take 79% credit but no deduction reduction (21%-rate)</li>
                <li data-i18n="view.s41.s280c.coordination">Coordinate with § 174: § 280C applies to current QRE, but § 174 amortization spreads</li>
                <li data-i18n="view.s41.s280c.amortization_period">2022+: § 174 mandates 5-yr (US) / 15-yr (foreign) amortization → § 280C interaction more complex</li>
                <li data-i18n="view.s41.s280c.tcja">TCJA pre-2022: § 174 immediate expense; post-2022: forced amortization → credit value rises relative to deduction</li>
                <li data-i18n="view.s41.s280c.bill_pending">Various proposals (TCDTRA, etc.) would restore § 174 immediate expensing</li>
            </ul>
        </div>
    `;
    document.getElementById('s41-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.current_year_qre = Number(fd.get('current_year_qre')) || 0;
        state.avg_gross_receipts_4yr = Number(fd.get('avg_gross_receipts_4yr')) || 0;
        state.fixed_base_percentage = Number(fd.get('fixed_base_percentage')) || 0;
        state.avg_qre_prior_3yr = Number(fd.get('avg_qre_prior_3yr')) || 0;
        state.credit_method = fd.get('credit_method');
        state.wages_for_qualified = Number(fd.get('wages_for_qualified')) || 0;
        state.supplies_for_qualified = Number(fd.get('supplies_for_qualified')) || 0;
        state.contract_research = Number(fd.get('contract_research')) || 0;
        state.computer_time_share = Number(fd.get('computer_time_share')) || 0;
        state.is_qualified_small_business = !!fd.get('is_qualified_small_business');
        state.payroll_tax_election = !!fd.get('payroll_tax_election');
        state.is_startup_pre_revenue = !!fd.get('is_startup_pre_revenue');
        state.s174_amortization = Number(fd.get('s174_amortization')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s41-output');
    if (!el) return;
    let credit = 0;
    if (state.credit_method === 'regular') {
        const baseAmount = Math.max(0.50 * state.current_year_qre, state.avg_gross_receipts_4yr * (state.fixed_base_percentage / 100));
        credit = 0.20 * Math.max(0, state.current_year_qre - baseAmount);
    } else {
        const ascBase = 0.50 * state.avg_qre_prior_3yr;
        credit = state.avg_qre_prior_3yr > 0 ?
            0.14 * Math.max(0, state.current_year_qre - ascBase) :
            0.06 * state.current_year_qre;
    }
    const payrollCredit = (state.is_qualified_small_business && state.payroll_tax_election) ? Math.min(credit, 500_000) : 0;
    const incomeTaxCredit = credit - payrollCredit;
    const totalQRE = state.wages_for_qualified + state.supplies_for_qualified + 0.65 * state.contract_research + state.computer_time_share;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s41.h2.result">§ 41 credit computation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s41.card.total_qre">Computed QRE</div>
                    <div class="value">$${totalQRE.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s41.card.credit">Research credit</div>
                    <div class="value">$${credit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s41.card.payroll">Payroll tax credit ($500K max)</div>
                    <div class="value">$${payrollCredit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s41.card.income">Income tax credit</div>
                    <div class="value">$${incomeTaxCredit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s41.card.method">Method used</div>
                    <div class="value">${esc(t('view.s41.method.' + state.credit_method))}</div>
                </div>
            </div>
            ${state.is_qualified_small_business && state.payroll_tax_election ? `
                <p class="muted small pos" style="margin-top:10px" data-i18n="view.s41.qsb_note">
                    Qualified small business: up to $500K of credit applied against employer FICA (6.2%
                    OASDI portion) via Form 8974 + Form 941 quarterly. Valuable for pre-revenue startups
                    with no income tax liability. Carryforward 5 years.
                </p>
            ` : ''}
        </div>
    `;
}
