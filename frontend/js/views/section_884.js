// IRC § 884 — Branch Profits Tax (BPT) on Foreign Corp.
// 30% (or treaty rate) on Dividend Equivalent Amount (DEA) — branch profits not reinvested in US.
// DEA = ECI earnings × (1 − change in US-connected E&P kept in US) − after-tax ECI.
// Coordinate with § 882 ECI tax → DOUBLE LAYER (ECI tax + BPT) similar to corp + shareholder.
// Treaty reductions: 5%/15% typical (similar to dividends).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    eci_taxable_income: 0,
    eci_tax_paid: 0,
    us_connected_ep_increase: 0,
    us_connected_ep_decrease: 0,
    treaty_country: '',
    treaty_bpt_rate: 30,
    qualified_resident: false,
    interest_paid_to_foreign: 0,
    branch_interest_excess_paid: 0,
    election_to_terminate: false,
    consolidated_returns: false,
    cumulative_us_assets: 0,
    cumulative_us_liabilities: 0,
    treaty_lob_passed: false,
};

export async function renderSection884(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s884.h1.title">// § 884 BRANCH PROFITS TAX</span></h1>
        <p class="muted small" data-i18n="view.s884.hint.intro">
            <strong>30%</strong> (or treaty rate) on <strong>Dividend Equivalent Amount (DEA)</strong> — branch
            profits NOT reinvested in US. <strong>DEA = ECI earnings × (1 − change in US-connected E&P</strong>
            kept in US) − after-tax ECI. <strong>Coordinate with § 882:</strong> ECI taxed at 21%, then DEA
            taxed at 30% → DOUBLE LAYER (similar to corp + dividend). <strong>Treaty reductions:</strong> 5%/15%
            typical (similar to dividends). <strong>§ 884(f):</strong> branch interest at 30% similar
            to dividend equivalents. <strong>Form 1120-F + Form 8848.</strong>
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s884.h2.inputs">Inputs</h2>
            <form id="s884-form" class="inline-form">
                <label><span data-i18n="view.s884.label.eci_income">ECI taxable income ($)</span>
                    <input type="number" step="0.01" name="eci_taxable_income" value="${state.eci_taxable_income}"></label>
                <label><span data-i18n="view.s884.label.eci_tax">ECI tax paid (§ 882) ($)</span>
                    <input type="number" step="0.01" name="eci_tax_paid" value="${state.eci_tax_paid}"></label>
                <label><span data-i18n="view.s884.label.ep_increase">US E&P increase (reinvested) ($)</span>
                    <input type="number" step="0.01" name="us_connected_ep_increase" value="${state.us_connected_ep_increase}"></label>
                <label><span data-i18n="view.s884.label.ep_decrease">US E&P decrease (repatriated) ($)</span>
                    <input type="number" step="0.01" name="us_connected_ep_decrease" value="${state.us_connected_ep_decrease}"></label>
                <label><span data-i18n="view.s884.label.country">Treaty country</span>
                    <input type="text" name="treaty_country" value="${esc(state.treaty_country)}"></label>
                <label><span data-i18n="view.s884.label.rate">Treaty BPT rate %</span>
                    <input type="number" step="0.1" name="treaty_bpt_rate" value="${state.treaty_bpt_rate}"></label>
                <label><span data-i18n="view.s884.label.qualified">Qualified resident (LOB)?</span>
                    <input type="checkbox" name="qualified_resident" ${state.qualified_resident ? 'checked' : ''}></label>
                <label><span data-i18n="view.s884.label.interest_paid">Interest paid to foreign ($)</span>
                    <input type="number" step="0.01" name="interest_paid_to_foreign" value="${state.interest_paid_to_foreign}"></label>
                <label><span data-i18n="view.s884.label.excess">Excess interest paid ($)</span>
                    <input type="number" step="0.01" name="branch_interest_excess_paid" value="${state.branch_interest_excess_paid}"></label>
                <label><span data-i18n="view.s884.label.terminate">Election to terminate?</span>
                    <input type="checkbox" name="election_to_terminate" ${state.election_to_terminate ? 'checked' : ''}></label>
                <label><span data-i18n="view.s884.label.consolidated">Consolidated returns?</span>
                    <input type="checkbox" name="consolidated_returns" ${state.consolidated_returns ? 'checked' : ''}></label>
                <label><span data-i18n="view.s884.label.us_assets">Cumulative US assets ($)</span>
                    <input type="number" step="0.01" name="cumulative_us_assets" value="${state.cumulative_us_assets}"></label>
                <label><span data-i18n="view.s884.label.us_liab">Cumulative US liabilities ($)</span>
                    <input type="number" step="0.01" name="cumulative_us_liabilities" value="${state.cumulative_us_liabilities}"></label>
                <label><span data-i18n="view.s884.label.lob">Treaty LOB passed?</span>
                    <input type="checkbox" name="treaty_lob_passed" ${state.treaty_lob_passed ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s884.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s884-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s884.h2.dea_computation">DEA computation</h2>
            <ol class="muted small">
                <li data-i18n="view.s884.dea.step1">Step 1: After-tax ECI = ECI - § 882 tax (21% corp rate)</li>
                <li data-i18n="view.s884.dea.step2">Step 2: Increase / decrease in US-connected E&P (reinvested in US)</li>
                <li data-i18n="view.s884.dea.step3">Step 3: DEA = after-tax ECI − increase in US E&P + decrease</li>
                <li data-i18n="view.s884.dea.step4">Step 4: Apply 30% (or treaty rate) to DEA</li>
                <li data-i18n="view.s884.dea.s884_e">§ 884(e): election to treat as accumulated branch profits</li>
                <li data-i18n="view.s884.dea.termination">§ 884(d): election to terminate USTB → triggers final BPT on accumulated</li>
                <li data-i18n="view.s884.dea.minimum">Minimum: BPT applies even if no current-year ECI if accumulated US E&P</li>
                <li data-i18n="view.s884.dea.character">Character: treated as if dividend paid to foreign parent</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s884.h2.treaty_relief">Treaty relief structure</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s884.th.country">Country</th>
                    <th data-i18n="view.s884.th.rate">Treaty BPT rate</th>
                    <th data-i18n="view.s884.th.notes">Notes</th>
                </tr></thead>
                <tbody>
                    <tr><td>UK</td><td>0% / 5% / 15%</td><td>Like dividend rates</td></tr>
                    <tr><td>Canada</td><td>5% / 15%</td><td>Maple Leaf treaty</td></tr>
                    <tr><td>Germany</td><td>5% / 15%</td><td>Comprehensive</td></tr>
                    <tr><td>Japan</td><td>0% / 5%</td><td>Reduced rates</td></tr>
                    <tr><td>Mexico</td><td>5% / 10%</td><td>NAFTA / USMCA</td></tr>
                    <tr><td>Ireland</td><td>5% / 15%</td><td>Holding company popular</td></tr>
                    <tr><td>Netherlands</td><td>0% / 5%</td><td>Conduit jurisdiction</td></tr>
                    <tr><td>Luxembourg</td><td>5% / 15%</td><td>Holding co popular</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s884.h2.lob">Limitation on Benefits (LOB)</h2>
            <ul class="muted small">
                <li data-i18n="view.s884.lob.purpose">Prevent treaty shopping (third-country residents claiming treaty benefits)</li>
                <li data-i18n="view.s884.lob.qualified_resident">"Qualified Person" tests: publicly traded, ownership tests, active business</li>
                <li data-i18n="view.s884.lob.derivative_benefits">Derivative benefits: 95%/95% ownership by qualified residents of treaty countries</li>
                <li data-i18n="view.s884.lob.competent_authority">Discretionary relief: competent authority application</li>
                <li data-i18n="view.s884.lob.objective_tests">Objective tests: ownership, base erosion, active trade</li>
                <li data-i18n="view.s884.lob.us_treaties">All US treaties have LOB articles since US Model 2006</li>
                <li data-i18n="view.s884.lob.form_8833">Form 8833 to invoke treaty position</li>
                <li data-i18n="view.s884.lob.fdap_separate">FDAP withholding: separate W-8BEN-E + treaty claim</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s884.h2.branch_interest">§ 884(f) branch interest tax</h2>
            <ul class="muted small">
                <li data-i18n="view.s884.bi.purpose">Prevent base erosion by paying interest to foreign parent / affiliates</li>
                <li data-i18n="view.s884.bi.rate">30% on EXCESS interest paid to foreign related party</li>
                <li data-i18n="view.s884.bi.actually_paid">Tax on interest actually paid (not deemed)</li>
                <li data-i18n="view.s884.bi.allowable_interest">Allowable interest: per allocation rules § 882-5</li>
                <li data-i18n="view.s884.bi.excess">Excess interest = paid - allowable = subject to 30% BPT-like tax</li>
                <li data-i18n="view.s884.bi.treaty_reduction">Treaty rates apply similar to dividends</li>
                <li data-i18n="view.s884.bi.fdap_overlap">May overlap with § 871(a) / § 881 withholding</li>
                <li data-i18n="view.s884.bi.deduction_limits">§ 163(j) limit may also apply to interest deduction</li>
            </ul>
        </div>
    `;
    document.getElementById('s884-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.eci_taxable_income = Number(fd.get('eci_taxable_income')) || 0;
        state.eci_tax_paid = Number(fd.get('eci_tax_paid')) || 0;
        state.us_connected_ep_increase = Number(fd.get('us_connected_ep_increase')) || 0;
        state.us_connected_ep_decrease = Number(fd.get('us_connected_ep_decrease')) || 0;
        state.treaty_country = fd.get('treaty_country');
        state.treaty_bpt_rate = Number(fd.get('treaty_bpt_rate')) || 0;
        state.qualified_resident = !!fd.get('qualified_resident');
        state.interest_paid_to_foreign = Number(fd.get('interest_paid_to_foreign')) || 0;
        state.branch_interest_excess_paid = Number(fd.get('branch_interest_excess_paid')) || 0;
        state.election_to_terminate = !!fd.get('election_to_terminate');
        state.consolidated_returns = !!fd.get('consolidated_returns');
        state.cumulative_us_assets = Number(fd.get('cumulative_us_assets')) || 0;
        state.cumulative_us_liabilities = Number(fd.get('cumulative_us_liabilities')) || 0;
        state.treaty_lob_passed = !!fd.get('treaty_lob_passed');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s884-output');
    if (!el) return;
    const after_tax_eci = state.eci_taxable_income - state.eci_tax_paid;
    const dea = Math.max(0, after_tax_eci - state.us_connected_ep_increase + state.us_connected_ep_decrease);
    const treaty_eligible = state.qualified_resident && state.treaty_lob_passed && state.treaty_bpt_rate < 30;
    const applicable_rate = treaty_eligible ? (state.treaty_bpt_rate / 100) : 0.30;
    const bpt = dea * applicable_rate;
    const branch_int_tax = state.branch_interest_excess_paid * applicable_rate;
    const total_tax = state.eci_tax_paid + bpt + branch_int_tax;
    const effective_rate = state.eci_taxable_income > 0 ? (total_tax / state.eci_taxable_income * 100) : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s884.h2.result">§ 884 BPT computation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s884.card.after_tax">After-tax ECI</div>
                    <div class="value">$${after_tax_eci.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s884.card.dea">DEA</div>
                    <div class="value">$${dea.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${treaty_eligible ? 'pos' : ''}">
                    <div class="label" data-i18n="view.s884.card.treaty">Treaty rate eligible</div>
                    <div class="value">${treaty_eligible ? esc(t('view.s884.status.yes')) : esc(t('view.s884.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s884.card.rate">Applicable rate</div>
                    <div class="value">${(applicable_rate * 100).toFixed(1)}%</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s884.card.bpt">BPT on DEA</div>
                    <div class="value">$${bpt.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s884.card.branch_int">§ 884(f) branch int tax</div>
                    <div class="value">$${branch_int_tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s884.card.total">Total US tax (ECI + BPT)</div>
                    <div class="value">$${total_tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s884.card.effective">Effective tax rate</div>
                    <div class="value">${effective_rate.toFixed(2)}%</div>
                </div>
            </div>
            ${!treaty_eligible && state.treaty_country ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s884.no_treaty_note">
                    Treaty BPT reduction NOT applied. Verify LOB qualification (Qualified Person tests):
                    publicly traded, ownership / base erosion tests, or active trade. Form 8833 required to
                    invoke treaty. Default 30% BPT applied. Common error: treaty residence does not equal
                    LOB eligibility — foreign holding companies often fail LOB.
                </p>
            ` : ''}
        </div>
    `;
}
