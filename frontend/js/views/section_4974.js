// IRC § 4974 — Excise Tax on Underdistribution from Qualified Retirement Plans.
// 25% (post-SECURE 2.0) excise tax on shortfall between Required Minimum Distribution (RMD) and actual.
// Reduced to 10% if corrected within 2-year window.
// Apply to: 401(k), 403(b), 457(b), Traditional IRAs (NOT Roth IRAs while owner alive).
// RBD (Required Beginning Date): April 1 of year after age 73 (SECURE 2.0).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    account_type: 'traditional_ira',
    rmd_required: 0,
    actual_distribution: 0,
    age_current: 0,
    is_post_rbd: false,
    is_first_year_rmd: false,
    rmd_year: 2024,
    is_inherited_account: false,
    correction_made: false,
    days_since_year_end: 0,
    is_qcd_offset: false,
    qcd_amount: 0,
    secure_2_0_25pct_rate: true,
    correction_within_2yr: false,
    waiver_request: false,
    multiple_accounts: false,
};

export async function renderSection4974(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s4974.h1.title">// § 4974 RMD UNDERDISTRIBUTION</span></h1>
        <p class="muted small" data-i18n="view.s4974.hint.intro">
            <strong>25% excise tax</strong> (post-SECURE 2.0) on shortfall between Required Minimum
            Distribution (RMD) and actual. <strong>Reduced to 10%</strong> if corrected within 2-year window
            (post-SECURE 2.0). <strong>Apply to:</strong> 401(k), 403(b), 457(b), Traditional IRAs (NOT Roth
            while owner alive). <strong>RBD:</strong> April 1 of year after age <strong>73</strong> (SECURE
            2.0, rising to 75 in 2033). <strong>First-year delay:</strong> can defer first RMD to April 1
            of next year (but then take 2 in that year). <strong>Form 5329</strong> Part IX reports.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s4974.h2.inputs">Inputs</h2>
            <form id="s4974-form" class="inline-form">
                <label><span data-i18n="view.s4974.label.type">Account type</span>
                    <select name="account_type">
                        <option value="traditional_ira" ${state.account_type === 'traditional_ira' ? 'selected' : ''}>Traditional IRA</option>
                        <option value="401k" ${state.account_type === '401k' ? 'selected' : ''}>401(k)</option>
                        <option value="403b" ${state.account_type === '403b' ? 'selected' : ''}>403(b)</option>
                        <option value="457b" ${state.account_type === '457b' ? 'selected' : ''}>457(b) gov't</option>
                        <option value="sep_ira" ${state.account_type === 'sep_ira' ? 'selected' : ''}>SEP-IRA</option>
                        <option value="simple_ira" ${state.account_type === 'simple_ira' ? 'selected' : ''}>SIMPLE IRA</option>
                        <option value="roth_ira_owner" ${state.account_type === 'roth_ira_owner' ? 'selected' : ''}>Roth IRA (owner — NO RMD)</option>
                        <option value="inherited_roth" ${state.account_type === 'inherited_roth' ? 'selected' : ''}>Inherited Roth (RMD post-2020)</option>
                        <option value="defined_benefit" ${state.account_type === 'defined_benefit' ? 'selected' : ''}>Defined benefit pension</option>
                    </select>
                </label>
                <label><span data-i18n="view.s4974.label.required">RMD required ($)</span>
                    <input type="number" step="1000" name="rmd_required" value="${state.rmd_required}"></label>
                <label><span data-i18n="view.s4974.label.actual">Actual distribution ($)</span>
                    <input type="number" step="1000" name="actual_distribution" value="${state.actual_distribution}"></label>
                <label><span data-i18n="view.s4974.label.age">Current age</span>
                    <input type="number" step="1" name="age_current" value="${state.age_current}"></label>
                <label><span data-i18n="view.s4974.label.post_rbd">Post-RBD?</span>
                    <input type="checkbox" name="is_post_rbd" ${state.is_post_rbd ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4974.label.first_year">First RMD year?</span>
                    <input type="checkbox" name="is_first_year_rmd" ${state.is_first_year_rmd ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4974.label.year">RMD year</span>
                    <input type="number" step="1" name="rmd_year" value="${state.rmd_year}"></label>
                <label><span data-i18n="view.s4974.label.inherited">Inherited account?</span>
                    <input type="checkbox" name="is_inherited_account" ${state.is_inherited_account ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4974.label.corrected">Correction made?</span>
                    <input type="checkbox" name="correction_made" ${state.correction_made ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4974.label.days">Days since year-end</span>
                    <input type="number" step="1" name="days_since_year_end" value="${state.days_since_year_end}"></label>
                <label><span data-i18n="view.s4974.label.qcd">QCD offset applied?</span>
                    <input type="checkbox" name="is_qcd_offset" ${state.is_qcd_offset ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4974.label.qcd_amount">QCD amount ($)</span>
                    <input type="number" step="1000" name="qcd_amount" value="${state.qcd_amount}"></label>
                <label><span data-i18n="view.s4974.label.s20_25pct">SECURE 2.0 25% rate (post-2022)?</span>
                    <input type="checkbox" name="secure_2_0_25pct_rate" ${state.secure_2_0_25pct_rate ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4974.label.correction_2yr">Correction within 2-yr window?</span>
                    <input type="checkbox" name="correction_within_2yr" ${state.correction_within_2yr ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4974.label.waiver">Waiver request (reasonable cause)?</span>
                    <input type="checkbox" name="waiver_request" ${state.waiver_request ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4974.label.multiple">Multiple IRA accounts?</span>
                    <input type="checkbox" name="multiple_accounts" ${state.multiple_accounts ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s4974.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s4974-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4974.h2.rbd">RBD timing (post-SECURE 2.0)</h2>
            <ul class="muted small">
                <li data-i18n="view.s4974.rbd.age">Age 73 (SECURE 2.0, since 2023) — was age 72 prior</li>
                <li data-i18n="view.s4974.rbd.future">Age 75 starting 2033 (further SECURE 2.0 increase)</li>
                <li data-i18n="view.s4974.rbd.first_year">First-year delay: April 1 of year AFTER reaching RBD age</li>
                <li data-i18n="view.s4974.rbd.subsequent">Subsequent RMDs: December 31 each year</li>
                <li data-i18n="view.s4974.rbd.deferral_strategy">Deferral strategy: take 1st RMD by April 1, take 2nd in same year</li>
                <li data-i18n="view.s4974.rbd.tax_implications">Both RMDs in same year may push income into higher tax bracket</li>
                <li data-i18n="view.s4974.rbd.employer_plans_delay">Employer plans: RBD = later of age 73 OR retirement (5%+ owners: age 73 regardless)</li>
                <li data-i18n="view.s4974.rbd.s401_a_9">§ 401(a)(9) Uniform Lifetime Table for calculation</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4974.h2.calculation">RMD calculation</h2>
            <ol class="muted small">
                <li data-i18n="view.s4974.calc.balance">Account balance: December 31 of PRIOR year</li>
                <li data-i18n="view.s4974.calc.divisor">Life expectancy: Uniform Lifetime Table (Reg § 1.401(a)(9)-9)</li>
                <li data-i18n="view.s4974.calc.formula">RMD = Prior year-end balance / Distribution period divisor</li>
                <li data-i18n="view.s4974.calc.example">Example: $500K balance, age 73 → divisor 26.5 → RMD = $18,868</li>
                <li data-i18n="view.s4974.calc.aggregation_ira">Aggregation: multiple IRAs total RMD; satisfy from any one</li>
                <li data-i18n="view.s4974.calc.aggregation_401k">401(k) / 403(b): NO aggregation — separate per account</li>
                <li data-i18n="view.s4974.calc.beneficiary">Beneficiary RMDs: single life table (more aggressive)</li>
                <li data-i18n="view.s4974.calc.spousal_delay">Surviving spouse: may delay until spouse's own RBD</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4974.h2.qcd">QCD offset (§ 408(d)(8))</h2>
            <ul class="muted small">
                <li data-i18n="view.s4974.qcd.purpose">Direct IRA → 501(c)(3) charity transfer counts as RMD</li>
                <li data-i18n="view.s4974.qcd.limit_2024">$105,000 limit per year (2024, indexed)</li>
                <li data-i18n="view.s4974.qcd.age_70_5">Age 70.5+ required (NOT 73 like RMD)</li>
                <li data-i18n="view.s4974.qcd.no_double">Not taxable + does NOT count as itemized deduction (vs cash gift)</li>
                <li data-i18n="view.s4974.qcd.tax_efficient">Tax-efficient: avoids inclusion in AGI (lowers Medicare premiums, SS taxation)</li>
                <li data-i18n="view.s4974.qcd.satisfies_rmd">Counts dollar-for-dollar toward RMD requirement</li>
                <li data-i18n="view.s4974.qcd.coordination">Cannot send to donor-advised fund or supporting organization</li>
                <li data-i18n="view.s4974.qcd.reportable">Reported on 1099-R Box 1 + adjustment in Form 1040 Line 4b</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4974.h2.correction">Correction + waiver</h2>
            <ul class="muted small">
                <li data-i18n="view.s4974.corr.10pct">SECURE 2.0: 10% (reduced from 25%) if corrected within 2-year window</li>
                <li data-i18n="view.s4974.corr.calculation">2-year window: from year RMD shortfall first occurred</li>
                <li data-i18n="view.s4974.corr.distribution">Correction: take missed distribution amount</li>
                <li data-i18n="view.s4974.corr.amended">Amended return: Form 1040X + Form 5329</li>
                <li data-i18n="view.s4974.corr.waiver">Reasonable cause waiver: still available; reasonable explanation + documentation</li>
                <li data-i18n="view.s4974.corr.illness">Reasonable cause examples: illness, financial advisor error, IRA custodian error</li>
                <li data-i18n="view.s4974.corr.late">Late even with waiver: penalty + amended return + Form 5329</li>
                <li data-i18n="view.s4974.corr.administrative">IRS administrative relief: typically waived if cured promptly + reasonable</li>
            </ul>
        </div>
    `;
    document.getElementById('s4974-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.account_type = fd.get('account_type');
        state.rmd_required = Number(fd.get('rmd_required')) || 0;
        state.actual_distribution = Number(fd.get('actual_distribution')) || 0;
        state.age_current = Number(fd.get('age_current')) || 0;
        state.is_post_rbd = !!fd.get('is_post_rbd');
        state.is_first_year_rmd = !!fd.get('is_first_year_rmd');
        state.rmd_year = Number(fd.get('rmd_year')) || 0;
        state.is_inherited_account = !!fd.get('is_inherited_account');
        state.correction_made = !!fd.get('correction_made');
        state.days_since_year_end = Number(fd.get('days_since_year_end')) || 0;
        state.is_qcd_offset = !!fd.get('is_qcd_offset');
        state.qcd_amount = Number(fd.get('qcd_amount')) || 0;
        state.secure_2_0_25pct_rate = !!fd.get('secure_2_0_25pct_rate');
        state.correction_within_2yr = !!fd.get('correction_within_2yr');
        state.waiver_request = !!fd.get('waiver_request');
        state.multiple_accounts = !!fd.get('multiple_accounts');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s4974-output');
    if (!el) return;
    const total_distribution = state.actual_distribution + (state.is_qcd_offset ? state.qcd_amount : 0);
    const shortfall = Math.max(0, state.rmd_required - total_distribution);
    let rate = state.secure_2_0_25pct_rate ? 0.25 : 0.50;
    if (state.correction_within_2yr) rate = 0.10;
    const excise_tax = shortfall * rate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s4974.h2.result">§ 4974 RMD excise tax</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s4974.card.required">RMD required</div>
                    <div class="value">$${state.rmd_required.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s4974.card.actual">Total distributed (incl QCD)</div>
                    <div class="value">$${total_distribution.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${shortfall > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s4974.card.shortfall">Shortfall</div>
                    <div class="value">$${shortfall.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s4974.card.rate">Excise rate</div>
                    <div class="value">${(rate * 100).toFixed(0)}%</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s4974.card.tax">Excise tax</div>
                    <div class="value">$${excise_tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${shortfall > 0 && !state.correction_made ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s4974.shortfall_note">
                    RMD SHORTFALL: 25% (or 10% if within 2-yr window) excise tax applies. Take missed
                    distribution IMMEDIATELY + file Form 5329 Part IX. SECURE 2.0 introduced 2-year cure
                    window for reduced 10% rate. Reasonable cause waiver still available — typically
                    illness, error by IRA custodian / advisor. Avoid: setup automatic year-end RMD via
                    brokerage.
                </p>
            ` : ''}
        </div>
    `;
}
