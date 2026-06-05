// IRC § 460 — Long-Term Contract Accounting (Percentage of Completion).
// Required: any contract not completed in same tax year started, with services + manufacturing.
// Method: Percentage-of-Completion (PCM) generally required.
// Exceptions: small contractor (≤ $30M avg gross receipts) for home construction + ≤ 2-yr contracts.
// Look-back: actual vs estimated; pay/receive interest on under/over-paid tax.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    contract_price: 0,
    estimated_total_costs: 0,
    costs_incurred_to_date: 0,
    revenue_recognized_prior: 0,
    contract_started_year: 2024,
    contract_estimated_completion_year: 2026,
    is_home_construction: false,
    avg_gross_receipts_3yr: 0,
    is_residential_construction: false,
    pcm_method: 'cost_to_cost',
    elect_10pct_method: false,
    elect_completed_contract: false,
    elect_amt_pcm: true,
    look_back_year: 0,
    actual_costs_final: 0,
};

export async function renderSection460(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s460.h1.title">// § 460 LONG-TERM CONTRACTS</span></h1>
        <p class="muted small" data-i18n="view.s460.hint.intro">
            <strong>PCM (Percentage-of-Completion) required</strong> for contracts NOT completed in same year
            started. <strong>Cost-to-cost method:</strong> revenue = costs incurred / total estimated costs ×
            contract price. <strong>Exceptions:</strong> small contractor ≤ $30M avg gross receipts + home
            construction + ≤ 2-yr contracts → Completed-Contract Method (CCM) or cash. <strong>10% method:</strong>
            defer all income/expense until 10% costs incurred. <strong>Look-back:</strong> Form 8697 — compute
            actual vs estimated profit; pay / receive interest on differences. <strong>§ 460(e) certain
            residential</strong> 70/30 split.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s460.h2.inputs">Inputs</h2>
            <form id="s460-form" class="inline-form">
                <label><span data-i18n="view.s460.label.price">Contract price ($)</span>
                    <input type="number" step="0.01" name="contract_price" value="${state.contract_price}"></label>
                <label><span data-i18n="view.s460.label.est">Estimated total costs ($)</span>
                    <input type="number" step="0.01" name="estimated_total_costs" value="${state.estimated_total_costs}"></label>
                <label><span data-i18n="view.s460.label.incurred">Costs incurred to date ($)</span>
                    <input type="number" step="0.01" name="costs_incurred_to_date" value="${state.costs_incurred_to_date}"></label>
                <label><span data-i18n="view.s460.label.prior_rev">Revenue recognized prior years ($)</span>
                    <input type="number" step="0.01" name="revenue_recognized_prior" value="${state.revenue_recognized_prior}"></label>
                <label><span data-i18n="view.s460.label.start">Contract start year</span>
                    <input type="number" step="1" name="contract_started_year" value="${state.contract_started_year}"></label>
                <label><span data-i18n="view.s460.label.complete">Estimated completion year</span>
                    <input type="number" step="1" name="contract_estimated_completion_year" value="${state.contract_estimated_completion_year}"></label>
                <label><span data-i18n="view.s460.label.home">Home construction contract?</span>
                    <input type="checkbox" name="is_home_construction" ${state.is_home_construction ? 'checked' : ''}></label>
                <label><span data-i18n="view.s460.label.gross">Avg gross receipts 3-yr ($)</span>
                    <input type="number" step="0.01" name="avg_gross_receipts_3yr" value="${state.avg_gross_receipts_3yr}"></label>
                <label><span data-i18n="view.s460.label.residential">Residential ≥ 4 units?</span>
                    <input type="checkbox" name="is_residential_construction" ${state.is_residential_construction ? 'checked' : ''}></label>
                <label><span data-i18n="view.s460.label.method">PCM method</span>
                    <select name="pcm_method">
                        <option value="cost_to_cost" ${state.pcm_method === 'cost_to_cost' ? 'selected' : ''}>Cost-to-cost (default)</option>
                        <option value="efforts" ${state.pcm_method === 'efforts' ? 'selected' : ''}>Efforts-expended (labor hours)</option>
                        <option value="units" ${state.pcm_method === 'units' ? 'selected' : ''}>Units-of-delivery</option>
                        <option value="ccm" ${state.pcm_method === 'ccm' ? 'selected' : ''}>Completed-Contract Method</option>
                    </select>
                </label>
                <label><span data-i18n="view.s460.label.ten">Elect 10% deferral?</span>
                    <input type="checkbox" name="elect_10pct_method" ${state.elect_10pct_method ? 'checked' : ''}></label>
                <label><span data-i18n="view.s460.label.ccm">Elect CCM (small contractor)?</span>
                    <input type="checkbox" name="elect_completed_contract" ${state.elect_completed_contract ? 'checked' : ''}></label>
                <label><span data-i18n="view.s460.label.amt">Elect AMT-PCM (alt min tax)?</span>
                    <input type="checkbox" name="elect_amt_pcm" ${state.elect_amt_pcm ? 'checked' : ''}></label>
                <label><span data-i18n="view.s460.label.lookback">Look-back year (for actual)</span>
                    <input type="number" step="1" name="look_back_year" value="${state.look_back_year}"></label>
                <label><span data-i18n="view.s460.label.actual">Actual final costs ($)</span>
                    <input type="number" step="0.01" name="actual_costs_final" value="${state.actual_costs_final}"></label>
                <button class="primary" type="submit" data-i18n="view.s460.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s460-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s460.h2.required">When PCM required vs exceptions</h2>
            <ul class="muted small">
                <li data-i18n="view.s460.req.default">Default: PCM required for contracts not completed in same year started</li>
                <li data-i18n="view.s460.req.home">Home construction: CCM allowed (any size)</li>
                <li data-i18n="view.s460.req.small_2yr">Small contractor (≤ $30M avg gross receipts) + ≤ 2-yr contract: CCM allowed</li>
                <li data-i18n="view.s460.req.long_pcm_residential">Residential ≥ 4 units: 70/30 PCM/CCM split</li>
                <li data-i18n="view.s460.req.ship">Ship construction: special § 460(c)(6) exemption</li>
                <li data-i18n="view.s460.req.contracts">Manufacturing contracts: include in PCM if unique items or > 12-mo production</li>
                <li data-i18n="view.s460.req.services_only">Services-only contracts: NOT § 460 (use general accounting methods)</li>
                <li data-i18n="view.s460.req.gov_contracts">Government contracts: typically PCM</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s460.h2.cost_to_cost">Cost-to-cost computation</h2>
            <ol class="muted small">
                <li data-i18n="view.s460.cost.percent">% Complete = Costs incurred to date / Total estimated costs</li>
                <li data-i18n="view.s460.cost.revenue">Cumulative revenue = % Complete × Contract price</li>
                <li data-i18n="view.s460.cost.current">Current year revenue = Cumulative revenue − Prior years revenue</li>
                <li data-i18n="view.s460.cost.profit">Gross profit = Revenue recognized − Costs incurred this year</li>
                <li data-i18n="view.s460.cost.estimate_change">Estimate change: cumulative catch-up in year of change</li>
                <li data-i18n="view.s460.cost.loss_recognize">Anticipated loss: recognize FULLY in current year (no spreading)</li>
                <li data-i18n="view.s460.cost.excludes">Excludes: warranty + post-completion administrative costs from PCM</li>
                <li data-i18n="view.s460.cost.subcontracts">Subcontracts: prime contractor still uses PCM on subcontracted portion</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s460.h2.lookback">Look-back interest (Form 8697)</h2>
            <ul class="muted small">
                <li data-i18n="view.s460.look.purpose">Purpose: prevent abuse of estimates to defer / accelerate tax</li>
                <li data-i18n="view.s460.look.compare">Compare actual final profit to estimated profit at each year</li>
                <li data-i18n="view.s460.look.rate">Interest at IRS underpayment rate</li>
                <li data-i18n="view.s460.look.under">Underpaid tax: pay interest TO IRS</li>
                <li data-i18n="view.s460.look.over">Overpaid tax: receive interest FROM IRS</li>
                <li data-i18n="view.s460.look.exception">Exception: small contracts ($1M cumulative, &lt; 2-yr) — opt-out elective</li>
                <li data-i18n="view.s460.look.elect_out">Elect-out: § 460(b)(6) Form 8697 attach</li>
                <li data-i18n="view.s460.look.s460_h">§ 460(h): Look-back interest method exemption for delayed reapplication</li>
            </ul>
        </div>
    `;
    document.getElementById('s460-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.contract_price = Number(fd.get('contract_price')) || 0;
        state.estimated_total_costs = Number(fd.get('estimated_total_costs')) || 0;
        state.costs_incurred_to_date = Number(fd.get('costs_incurred_to_date')) || 0;
        state.revenue_recognized_prior = Number(fd.get('revenue_recognized_prior')) || 0;
        state.contract_started_year = Number(fd.get('contract_started_year')) || 0;
        state.contract_estimated_completion_year = Number(fd.get('contract_estimated_completion_year')) || 0;
        state.is_home_construction = !!fd.get('is_home_construction');
        state.avg_gross_receipts_3yr = Number(fd.get('avg_gross_receipts_3yr')) || 0;
        state.is_residential_construction = !!fd.get('is_residential_construction');
        state.pcm_method = fd.get('pcm_method');
        state.elect_10pct_method = !!fd.get('elect_10pct_method');
        state.elect_completed_contract = !!fd.get('elect_completed_contract');
        state.elect_amt_pcm = !!fd.get('elect_amt_pcm');
        state.look_back_year = Number(fd.get('look_back_year')) || 0;
        state.actual_costs_final = Number(fd.get('actual_costs_final')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s460-output');
    if (!el) return;
    const pctComplete = state.estimated_total_costs > 0 ? (state.costs_incurred_to_date / state.estimated_total_costs) : 0;
    const cumulativeRevenue = pctComplete * state.contract_price;
    const currentRevenue = cumulativeRevenue - state.revenue_recognized_prior;
    const grossProfit = currentRevenue - (state.costs_incurred_to_date - (state.revenue_recognized_prior * state.estimated_total_costs / state.contract_price));
    const isSmallContractor = state.avg_gross_receipts_3yr <= 30_000_000;
    const ccmAllowed = state.is_home_construction || (isSmallContractor && (state.contract_estimated_completion_year - state.contract_started_year <= 2));
    const lookBackDifference = state.actual_costs_final > 0 ? state.actual_costs_final - state.estimated_total_costs : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s460.h2.result">§ 460 PCM computation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s460.card.pct">% Complete</div>
                    <div class="value">${(pctComplete * 100).toFixed(2)}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s460.card.cumulative">Cumulative revenue</div>
                    <div class="value">$${cumulativeRevenue.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s460.card.current">Current yr revenue</div>
                    <div class="value">$${currentRevenue.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s460.card.profit">Current yr gross profit</div>
                    <div class="value">$${grossProfit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${ccmAllowed ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s460.card.ccm_allowed">CCM allowed?</div>
                    <div class="value">${ccmAllowed ? esc(t('view.s460.status.yes')) : esc(t('view.s460.status.no'))}</div>
                </div>
                <div class="card ${lookBackDifference !== 0 ? 'neg' : ''}">
                    <div class="label" data-i18n="view.s460.card.lookback_diff">Look-back diff</div>
                    <div class="value">$${lookBackDifference.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${ccmAllowed && state.elect_completed_contract ? `
                <p class="muted small pos" style="margin-top:10px" data-i18n="view.s460.ccm_note">
                    CCM (Completed-Contract Method) elected: defer ALL revenue + costs until contract done.
                    Benefit: avoid PCM cash flow on long projects with milestone delays. Cost: no early income;
                    AMT-PCM may still apply for alternative minimum tax purposes.
                </p>
            ` : ''}
        </div>
    `;
}
