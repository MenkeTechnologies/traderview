// IRC § 408A — Roth IRA (tax-free withdrawals on qualified distributions).
// 2024 contribution limit: $7,000 / $8,000 catch-up 50+.
// MAGI phase-out: $146K-$161K single / $230K-$240K MFJ.
// Backdoor Roth + Mega backdoor Roth + Roth conversion.

import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    filing_status: 'mfj',
    magi: 0,
    age: 0,
    contribution_amount: 0,
    is_conversion: false,
    conversion_amount: 0,
    pre_tax_iras_balance: 0,
    after_tax_iras_balance: 0,
    is_backdoor: false,
    is_mega_backdoor: false,
    employer_401k_after_tax: 0,
    has_qualified_distribution: false,
    days_since_first_contribution: 0,
    is_first_time_home: false,
    is_education_distribution: false,
    is_disability: false,
    is_inherited_roth: false,
    secure_act_10yr: false,
    s72_5yr_rule: false,
    pro_rata_balance: 0,
    qcd_offset_age: 70.5,
    year: 2024,
};

export async function renderSection408A(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s408a.h1.title">// § 408A ROTH IRA</span></h1>
        <p class="muted small" data-i18n="view.s408a.hint.intro">
            <strong>Roth IRA</strong> = after-tax contributions + tax-free qualified distributions.
            <strong>2024 limits:</strong> $7,000 contribution + $1,000 catch-up (age 50+).
            <strong>MAGI phase-out:</strong> $146K-$161K single / $230K-$240K MFJ.
            <strong>Qualified distribution:</strong> 5-YEAR rule (from first contribution OR conversion)
            + AGE 59½ OR death OR disability OR first-time home ($10K). <strong>NO RMD</strong> during
            owner's lifetime (vs Traditional IRA). <strong>Backdoor Roth:</strong> nondeductible
            traditional IRA → conversion. <strong>Mega backdoor:</strong> after-tax 401(k) →
            in-service conversion. <strong>Pro-rata rule</strong> § 408(d)(2) — all IRAs aggregated
            for taxable portion of conversion.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s408a.h2.inputs">Inputs</h2>
            <form id="s408a-form" class="inline-form">
                <label><span data-i18n="view.s408a.label.status">Filing status</span>
                    <select name="filing_status">
                        <option value="single" ${state.filing_status === 'single' ? 'selected' : ''}>Single</option>
                        <option value="mfj" ${state.filing_status === 'mfj' ? 'selected' : ''}>MFJ</option>
                        <option value="hoh" ${state.filing_status === 'hoh' ? 'selected' : ''}>HOH</option>
                        <option value="mfs" ${state.filing_status === 'mfs' ? 'selected' : ''}>MFS (lives w/ spouse)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s408a.label.magi">MAGI ($)</span>
                    <input type="number" step="0.01" name="magi" value="${state.magi}"></label>
                <label><span data-i18n="view.s408a.label.age">Age</span>
                    <input type="number" step="1" name="age" value="${state.age}"></label>
                <label><span data-i18n="view.s408a.label.contrib">Contribution amount ($)</span>
                    <input type="number" step="0.01" name="contribution_amount" value="${state.contribution_amount}"></label>
                <label><span data-i18n="view.s408a.label.conversion">Roth conversion?</span>
                    <input type="checkbox" name="is_conversion" ${state.is_conversion ? 'checked' : ''}></label>
                <label><span data-i18n="view.s408a.label.conv_amt">Conversion amount ($)</span>
                    <input type="number" step="0.01" name="conversion_amount" value="${state.conversion_amount}"></label>
                <label><span data-i18n="view.s408a.label.pretax">Pre-tax IRA balance ($)</span>
                    <input type="number" step="0.01" name="pre_tax_iras_balance" value="${state.pre_tax_iras_balance}"></label>
                <label><span data-i18n="view.s408a.label.aftertax">After-tax IRA balance ($)</span>
                    <input type="number" step="0.01" name="after_tax_iras_balance" value="${state.after_tax_iras_balance}"></label>
                <label><span data-i18n="view.s408a.label.backdoor">Backdoor Roth?</span>
                    <input type="checkbox" name="is_backdoor" ${state.is_backdoor ? 'checked' : ''}></label>
                <label><span data-i18n="view.s408a.label.mega">Mega backdoor?</span>
                    <input type="checkbox" name="is_mega_backdoor" ${state.is_mega_backdoor ? 'checked' : ''}></label>
                <label><span data-i18n="view.s408a.label.401k_aftertax">After-tax 401(k) ($)</span>
                    <input type="number" step="0.01" name="employer_401k_after_tax" value="${state.employer_401k_after_tax}"></label>
                <label><span data-i18n="view.s408a.label.qualified">Qualified distribution?</span>
                    <input type="checkbox" name="has_qualified_distribution" ${state.has_qualified_distribution ? 'checked' : ''}></label>
                <label><span data-i18n="view.s408a.label.days">Days since 1st contrib</span>
                    <input type="number" step="1" name="days_since_first_contribution" value="${state.days_since_first_contribution}"></label>
                <label><span data-i18n="view.s408a.label.first_home">First-time home?</span>
                    <input type="checkbox" name="is_first_time_home" ${state.is_first_time_home ? 'checked' : ''}></label>
                <label><span data-i18n="view.s408a.label.education">Education distribution?</span>
                    <input type="checkbox" name="is_education_distribution" ${state.is_education_distribution ? 'checked' : ''}></label>
                <label><span data-i18n="view.s408a.label.disability">Disability?</span>
                    <input type="checkbox" name="is_disability" ${state.is_disability ? 'checked' : ''}></label>
                <label><span data-i18n="view.s408a.label.inherited">Inherited Roth?</span>
                    <input type="checkbox" name="is_inherited_roth" ${state.is_inherited_roth ? 'checked' : ''}></label>
                <label><span data-i18n="view.s408a.label.secure_10yr">SECURE Act 10-yr rule?</span>
                    <input type="checkbox" name="secure_act_10yr" ${state.secure_act_10yr ? 'checked' : ''}></label>
                <label><span data-i18n="view.s408a.label.s72_5yr">§ 72 5-yr rule?</span>
                    <input type="checkbox" name="s72_5yr_rule" ${state.s72_5yr_rule ? 'checked' : ''}></label>
                <label><span data-i18n="view.s408a.label.prorata">Pro-rata aggregate balance ($)</span>
                    <input type="number" step="0.01" name="pro_rata_balance" value="${state.pro_rata_balance}"></label>
                <label><span data-i18n="view.s408a.label.qcd">QCD age</span>
                    <input type="number" step="0.5" name="qcd_offset_age" value="${state.qcd_offset_age}"></label>
                <label><span data-i18n="view.s408a.label.year">Year</span>
                    <input type="number" step="1" name="year" value="${state.year}"></label>
                <button class="primary" type="submit" data-i18n="view.s408a.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s408a-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s408a.h2.limits">2024 contribution + phase-out</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s408a.tbl.status">Status</th><th data-i18n="view.s408a.tbl.lower">Phase-out start</th><th data-i18n="view.s408a.tbl.upper">Phase-out end</th><th data-i18n="view.s408a.tbl.contrib">Max contrib (under 50)</th><th data-i18n="view.s408a.tbl.catchup">Catch-up (50+)</th></tr></thead>
                <tbody>
                    <tr><td>Single / HOH</td><td>$146,000</td><td>$161,000</td><td>$7,000</td><td>$1,000</td></tr>
                    <tr><td>MFJ</td><td>$230,000</td><td>$240,000</td><td>$7,000</td><td>$1,000</td></tr>
                    <tr><td>MFS (lives with spouse)</td><td>$0</td><td>$10,000</td><td>$7,000</td><td>$1,000</td></tr>
                    <tr><td data-i18n="view.s408a.tbl.mfs_separate">MFS (does NOT live with spouse)</td><td>$146,000</td><td>$161,000</td><td>$7,000</td><td>$1,000</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s408a.h2.qualified">Qualified distribution requirements</h2>
            <ol class="muted small">
                <li data-i18n="view.s408a.qual.5year">5-YEAR rule: from January 1 of year of first contribution</li>
                <li data-i18n="view.s408a.qual.conversion_separate">Each Roth conversion: separate 5-year clock (penalty avoidance only)</li>
                <li data-i18n="view.s408a.qual.age_595">Age 59½ — OR — death — OR — disability — OR — first-time home</li>
                <li data-i18n="view.s408a.qual.firsttime">First-time home: $10,000 lifetime limit (NOT first-EVER but no home in 2 yrs)</li>
                <li data-i18n="view.s408a.qual.tax_free">Tax-free + penalty-free if qualified</li>
                <li data-i18n="view.s408a.qual.ordering">Ordering rule: contributions FIRST, then conversions (oldest first), then earnings</li>
                <li data-i18n="view.s408a.qual.contributions">Contributions: always tax-free + penalty-free</li>
                <li data-i18n="view.s408a.qual.earnings">Earnings: tax + 10% § 72(t) penalty if not qualified</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s408a.h2.backdoor">Backdoor Roth mechanics</h2>
            <ul class="muted small">
                <li data-i18n="view.s408a.bd.step1">Step 1: Make NONDEDUCTIBLE contribution to traditional IRA ($7,000 limit)</li>
                <li data-i18n="view.s408a.bd.step2">Step 2: CONVERT immediately to Roth IRA</li>
                <li data-i18n="view.s408a.bd.no_phaseout">NO MAGI phase-out for nondeductible contribution OR conversion</li>
                <li data-i18n="view.s408a.bd.prorata">§ 408(d)(2) PRO-RATA rule: ALL IRAs (pre + after-tax) aggregated</li>
                <li data-i18n="view.s408a.bd.prorata_formula">Taxable % = pre-tax balance / total IRA balance</li>
                <li data-i18n="view.s408a.bd.pretax_workaround">Workaround: roll pre-tax IRAs to 401(k) before conversion (if employer permits)</li>
                <li data-i18n="view.s408a.bd.form_8606">Form 8606 reports basis + conversion</li>
                <li data-i18n="view.s408a.bd.spousal">Spouse can do separately — $7K each ($14K combined)</li>
                <li data-i18n="view.s408a.bd.step_transaction">Step transaction doctrine: IRS announced will NOT challenge</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s408a.h2.mega">Mega backdoor Roth (in-plan)</h2>
            <ul class="muted small">
                <li data-i18n="view.s408a.mega.s415_limit">§ 415(c) annual addition limit: $69,000 (2024) per plan</li>
                <li data-i18n="view.s408a.mega.s402g_first">§ 402(g) elective deferral: $23,000 (2024)</li>
                <li data-i18n="view.s408a.mega.aftertax_pool">$46,000 ($69K - $23K) potentially available for after-tax voluntary</li>
                <li data-i18n="view.s408a.mega.matching_reduces">Employer matching contributions REDUCE available after-tax space</li>
                <li data-i18n="view.s408a.mega.in_service">In-service conversion to Roth 401(k) OR rollover to Roth IRA</li>
                <li data-i18n="view.s408a.mega.plan_must_allow">Plan must SPECIFICALLY permit (most do not)</li>
                <li data-i18n="view.s408a.mega.no_pretax_dilution">After-tax separate accounting — pro-rata NOT diluted by pre-tax</li>
                <li data-i18n="view.s408a.mega.deemed_irrev">Deemed irrevocable: cannot recharacterize after 2018 TCJA</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s408a.h2.secure">SECURE Act inheritance + RMD</h2>
            <ul class="muted small">
                <li data-i18n="view.s408a.secure.no_rmd_owner">NO RMD during owner's lifetime (vs Traditional)</li>
                <li data-i18n="view.s408a.secure.beneficiary_10yr">Non-eligible designated beneficiary: 10-year rule (full distribution)</li>
                <li data-i18n="view.s408a.secure.eligible_dl">Eligible designated beneficiary: stretch over life expectancy</li>
                <li data-i18n="view.s408a.secure.spouse">Spouse: treat as own + recalculate as owner</li>
                <li data-i18n="view.s408a.secure.minor_child">Minor child: stretch until age 21, then 10-year clock</li>
                <li data-i18n="view.s408a.secure.disabled">Disabled / chronically ill: stretch over life expectancy</li>
                <li data-i18n="view.s408a.secure.10yr_no_rmd">10-year rule: NO annual RMD — just full distribution by year 10</li>
                <li data-i18n="view.s408a.secure.reg_2024">Reg § 1.401(a)(9) final rules 2024: annual RMD required for non-EDBs of pre-RBD owner</li>
            </ul>
        </div>
    `;
    document.getElementById('s408a-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.filing_status = fd.get('filing_status');
        state.magi = Number(fd.get('magi')) || 0;
        state.age = Number(fd.get('age')) || 0;
        state.contribution_amount = Number(fd.get('contribution_amount')) || 0;
        state.is_conversion = !!fd.get('is_conversion');
        state.conversion_amount = Number(fd.get('conversion_amount')) || 0;
        state.pre_tax_iras_balance = Number(fd.get('pre_tax_iras_balance')) || 0;
        state.after_tax_iras_balance = Number(fd.get('after_tax_iras_balance')) || 0;
        state.is_backdoor = !!fd.get('is_backdoor');
        state.is_mega_backdoor = !!fd.get('is_mega_backdoor');
        state.employer_401k_after_tax = Number(fd.get('employer_401k_after_tax')) || 0;
        state.has_qualified_distribution = !!fd.get('has_qualified_distribution');
        state.days_since_first_contribution = Number(fd.get('days_since_first_contribution')) || 0;
        state.is_first_time_home = !!fd.get('is_first_time_home');
        state.is_education_distribution = !!fd.get('is_education_distribution');
        state.is_disability = !!fd.get('is_disability');
        state.is_inherited_roth = !!fd.get('is_inherited_roth');
        state.secure_act_10yr = !!fd.get('secure_act_10yr');
        state.s72_5yr_rule = !!fd.get('s72_5yr_rule');
        state.pro_rata_balance = Number(fd.get('pro_rata_balance')) || 0;
        state.qcd_offset_age = Number(fd.get('qcd_offset_age')) || 0;
        state.year = Number(fd.get('year')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s408a-output');
    if (!el) return;
    const single = state.filing_status === 'single' || state.filing_status === 'hoh';
    const phase_start = single ? 146_000 : 230_000;
    const phase_end = single ? 161_000 : 240_000;
    const base_limit = state.age >= 50 ? 8_000 : 7_000;
    let allowed_contrib = base_limit;
    if (state.magi >= phase_end) allowed_contrib = 0;
    else if (state.magi > phase_start) allowed_contrib = base_limit * ((phase_end - state.magi) / (phase_end - phase_start));
    const total_iras = state.pre_tax_iras_balance + state.after_tax_iras_balance;
    const pro_rata_taxable_pct = total_iras > 0 ? state.pre_tax_iras_balance / total_iras : 0;
    const taxable_conversion = state.conversion_amount * pro_rata_taxable_pct;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s408a.h2.result">§ 408A Roth assessment</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.s408a.card.allowed">Allowed contribution</div><div class="value">$${allowed_contrib.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div></div>
                <div class="card"><div class="label" data-i18n="view.s408a.card.taxable_conv">Taxable conversion</div><div class="value">$${taxable_conversion.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div></div>
                <div class="card ${pro_rata_taxable_pct > 0 ? 'warn' : 'pos'}"><div class="label" data-i18n="view.s408a.card.prorata">Pro-rata taxable %</div><div class="value">${(pro_rata_taxable_pct * 100).toFixed(1)}%</div></div>
                <div class="card ${state.has_qualified_distribution ? 'pos' : 'neg'}"><div class="label" data-i18n="view.s408a.card.qual">Qualified distribution?</div><div class="value">${state.has_qualified_distribution ? 'YES (tax-free)' : 'NO (10% + tax)'}</div></div>
            </div>
        </div>
    `;
}
