// IRC § 162(a)(1) — Reasonable Compensation Limit.
// Deductible compensation must be REASONABLE in amount for services actually rendered.
// Excess compensation: characterized as DISGUISED DIVIDEND (C-corp) or NOT DEDUCTIBLE distribution.
// 5 multi-factor tests (Mayson Manufacturing, Independent Investor Test, etc.).
// Often disputed: closely-held corps, family businesses, S-corp owner-employees (reverse problem).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    actual_compensation: 0,
    industry_median_comp: 0,
    industry_75th_pct: 0,
    education_level: 'graduate',
    years_experience: 0,
    employee_role: 'ceo',
    revenue_under_management: 0,
    profit_under_management: 0,
    is_c_corp: true,
    is_s_corp_reverse: false,
    salary_bonus_split: 0,
    closely_held: true,
    family_business: false,
    independent_investor_passed: true,
    five_factor_score: 70,
    irs_reclassification_risk: false,
};

export async function renderSection162a1(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s162a1.h1.title">// § 162(a)(1) REASONABLE COMP</span></h1>
        <p class="muted small" data-i18n="view.s162a1.hint.intro">
            Deductible compensation must be <strong>REASONABLE</strong> in amount for services actually rendered.
            <strong>Excess:</strong> characterized as DISGUISED DIVIDEND (C-corp) → NOT DEDUCTIBLE
            distribution. <strong>Tests:</strong> Mayson Manufacturing 5-factor + Independent Investor Test
            (Exacto Spring) + Multi-Factor (most circuits). <strong>Disputes:</strong> closely-held corps,
            family businesses. <strong>S-corp REVERSE problem:</strong> owner-employees UNDERPAY salary to
            avoid SE tax → IRS recharacterizes distributions as wages.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s162a1.h2.inputs">Inputs</h2>
            <form id="s162a1-form" class="inline-form">
                <label><span data-i18n="view.s162a1.label.actual">Actual compensation ($)</span>
                    <input type="number" step="10000" name="actual_compensation" value="${state.actual_compensation}"></label>
                <label><span data-i18n="view.s162a1.label.median">Industry median comp ($)</span>
                    <input type="number" step="10000" name="industry_median_comp" value="${state.industry_median_comp}"></label>
                <label><span data-i18n="view.s162a1.label.75th">Industry 75th percentile ($)</span>
                    <input type="number" step="10000" name="industry_75th_pct" value="${state.industry_75th_pct}"></label>
                <label><span data-i18n="view.s162a1.label.education">Education level</span>
                    <select name="education_level">
                        <option value="high_school" ${state.education_level === 'high_school' ? 'selected' : ''}>High school</option>
                        <option value="bachelors" ${state.education_level === 'bachelors' ? 'selected' : ''}>Bachelor's</option>
                        <option value="graduate" ${state.education_level === 'graduate' ? 'selected' : ''}>Graduate / professional</option>
                        <option value="phd_specialist" ${state.education_level === 'phd_specialist' ? 'selected' : ''}>PhD / specialty</option>
                    </select>
                </label>
                <label><span data-i18n="view.s162a1.label.experience">Years experience</span>
                    <input type="number" step="1" name="years_experience" value="${state.years_experience}"></label>
                <label><span data-i18n="view.s162a1.label.role">Employee role</span>
                    <select name="employee_role">
                        <option value="ceo" ${state.employee_role === 'ceo' ? 'selected' : ''}>CEO</option>
                        <option value="cfo" ${state.employee_role === 'cfo' ? 'selected' : ''}>CFO</option>
                        <option value="cto" ${state.employee_role === 'cto' ? 'selected' : ''}>CTO</option>
                        <option value="vp" ${state.employee_role === 'vp' ? 'selected' : ''}>VP</option>
                        <option value="manager" ${state.employee_role === 'manager' ? 'selected' : ''}>Manager</option>
                        <option value="staff" ${state.employee_role === 'staff' ? 'selected' : ''}>Staff</option>
                        <option value="family_member" ${state.employee_role === 'family_member' ? 'selected' : ''}>Family member (heightened scrutiny)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s162a1.label.revenue">Revenue under mgmt ($)</span>
                    <input type="number" step="1000000" name="revenue_under_management" value="${state.revenue_under_management}"></label>
                <label><span data-i18n="view.s162a1.label.profit">Profit under mgmt ($)</span>
                    <input type="number" step="1000000" name="profit_under_management" value="${state.profit_under_management}"></label>
                <label><span data-i18n="view.s162a1.label.c_corp">C-corp?</span>
                    <input type="checkbox" name="is_c_corp" ${state.is_c_corp ? 'checked' : ''}></label>
                <label><span data-i18n="view.s162a1.label.s_corp_rev">S-corp REVERSE problem?</span>
                    <input type="checkbox" name="is_s_corp_reverse" ${state.is_s_corp_reverse ? 'checked' : ''}></label>
                <label><span data-i18n="view.s162a1.label.split">Salary / bonus split %</span>
                    <input type="number" step="1" name="salary_bonus_split" value="${state.salary_bonus_split}"></label>
                <label><span data-i18n="view.s162a1.label.closely">Closely held?</span>
                    <input type="checkbox" name="closely_held" ${state.closely_held ? 'checked' : ''}></label>
                <label><span data-i18n="view.s162a1.label.family">Family business?</span>
                    <input type="checkbox" name="family_business" ${state.family_business ? 'checked' : ''}></label>
                <label><span data-i18n="view.s162a1.label.investor">Independent investor test passed?</span>
                    <input type="checkbox" name="independent_investor_passed" ${state.independent_investor_passed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s162a1.label.five_factor">5-factor score (0-100)</span>
                    <input type="number" step="1" name="five_factor_score" value="${state.five_factor_score}"></label>
                <label><span data-i18n="view.s162a1.label.risk">IRS reclassification risk?</span>
                    <input type="checkbox" name="irs_reclassification_risk" ${state.irs_reclassification_risk ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s162a1.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s162a1-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s162a1.h2.mayson">Mayson Manufacturing 5-factor test</h2>
            <ol class="muted small">
                <li data-i18n="view.s162a1.mayson.role">Employee's role: position, hours worked, duties</li>
                <li data-i18n="view.s162a1.mayson.external">External comparisons: similar positions in similar companies</li>
                <li data-i18n="view.s162a1.mayson.character">Character + condition of company: size, complexity, financial position</li>
                <li data-i18n="view.s162a1.mayson.conflict_interest">Conflict of interest: any inherent bias (owner-employee, family)</li>
                <li data-i18n="view.s162a1.mayson.internal_consistency">Internal consistency: comp pattern over time + similar employees</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s162a1.h2.independent">Independent Investor Test (Exacto Spring, 7th Cir.)</h2>
            <ul class="muted small">
                <li data-i18n="view.s162a1.iit.test">Test: would an independent investor in the corp be content with the return after compensation?</li>
                <li data-i18n="view.s162a1.iit.return">Look at: return on equity / assets after deducting compensation</li>
                <li data-i18n="view.s162a1.iit.industry">Compare to: industry benchmark return on capital</li>
                <li data-i18n="view.s162a1.iit.satisfied">If hypothetical investor would be content → comp REASONABLE</li>
                <li data-i18n="view.s162a1.iit.deficient">If hypothetical investor would be dissatisfied → comp EXCESSIVE</li>
                <li data-i18n="view.s162a1.iit.simplification">7th Circuit simplification: avoids 5-factor weighing</li>
                <li data-i18n="view.s162a1.iit.minority">Other circuits still use 5-factor (Tax Court general approach)</li>
                <li data-i18n="view.s162a1.iit.combined">Court often applies BOTH tests in combination</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s162a1.h2.s_corp_reverse">S-Corp REVERSE problem</h2>
            <ul class="muted small">
                <li data-i18n="view.s162a1.scr.problem">S-corp owners pay themselves LOW salary + take large distributions to avoid SE/FICA</li>
                <li data-i18n="view.s162a1.scr.irs">IRS recharacterizes distribution as wages → adds back SE / FICA + penalties</li>
                <li data-i18n="view.s162a1.scr.test">Test: REASONABLE comp for services rendered</li>
                <li data-i18n="view.s162a1.scr.factors">Factors: training, experience, duties, time devoted, comparable positions, employee dividend history</li>
                <li data-i18n="view.s162a1.scr.cases">Watson v. Comm'r (2010): $24K salary on $371K K-1 distribution rejected — IRS adjusted to $90K</li>
                <li data-i18n="view.s162a1.scr.s262">IRS Fact Sheet 2008-25 + Notice 2018-2: documentation expectations</li>
                <li data-i18n="view.s162a1.scr.payroll_taxes">Reasonable comp study: PRO documents salary determination</li>
                <li data-i18n="view.s162a1.scr.flow_through">Recharacterization triggers: payroll taxes + Form 941 + W-2 amendments</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s162a1.h2.documentation">Documentation best practices</h2>
            <ul class="muted small">
                <li data-i18n="view.s162a1.doc.compensation_study">Annual compensation study: benchmark to similar companies / positions</li>
                <li data-i18n="view.s162a1.doc.board_minutes">Board / shareholder minutes documenting comp decision + rationale</li>
                <li data-i18n="view.s162a1.doc.employment_agreement">Written employment agreement with specific salary / bonus provisions</li>
                <li data-i18n="view.s162a1.doc.performance_metrics">Performance metrics + bonus formulas in writing</li>
                <li data-i18n="view.s162a1.doc.contemporaneous">Decisions contemporaneous with payment (not after-the-fact rationalization)</li>
                <li data-i18n="view.s162a1.doc.bonus_clawback">Bonus clawback provisions strengthen reasonable comp argument</li>
                <li data-i18n="view.s162a1.doc.s_corp_compensation_studies">RCReports + Equilar + BLS data: industry benchmarks</li>
                <li data-i18n="view.s162a1.doc.salary_history">Maintain salary history showing pattern over years</li>
            </ul>
        </div>
    `;
    document.getElementById('s162a1-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.actual_compensation = Number(fd.get('actual_compensation')) || 0;
        state.industry_median_comp = Number(fd.get('industry_median_comp')) || 0;
        state.industry_75th_pct = Number(fd.get('industry_75th_pct')) || 0;
        state.education_level = fd.get('education_level');
        state.years_experience = Number(fd.get('years_experience')) || 0;
        state.employee_role = fd.get('employee_role');
        state.revenue_under_management = Number(fd.get('revenue_under_management')) || 0;
        state.profit_under_management = Number(fd.get('profit_under_management')) || 0;
        state.is_c_corp = !!fd.get('is_c_corp');
        state.is_s_corp_reverse = !!fd.get('is_s_corp_reverse');
        state.salary_bonus_split = Number(fd.get('salary_bonus_split')) || 0;
        state.closely_held = !!fd.get('closely_held');
        state.family_business = !!fd.get('family_business');
        state.independent_investor_passed = !!fd.get('independent_investor_passed');
        state.five_factor_score = Number(fd.get('five_factor_score')) || 0;
        state.irs_reclassification_risk = !!fd.get('irs_reclassification_risk');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s162a1-output');
    if (!el) return;
    const above_median = state.actual_compensation > state.industry_median_comp;
    const above_75 = state.actual_compensation > state.industry_75th_pct;
    const reasonable = state.five_factor_score >= 60 && state.independent_investor_passed && !above_75;
    const excessive_amt = Math.max(0, state.actual_compensation - state.industry_75th_pct);
    const reclass_tax_c_corp = excessive_amt * 0.21;
    const reclass_div_tax = excessive_amt * 0.20;
    const s_corp_reclass_se = state.is_s_corp_reverse ? excessive_amt * 0.153 : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s162a1.h2.result">§ 162(a)(1) outcome</h2>
            <div class="cards">
                <div class="card ${reasonable ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s162a1.card.reasonable">Reasonable?</div>
                    <div class="value">${reasonable ? esc(t('view.s162a1.status.yes')) : esc(t('view.s162a1.status.no'))}</div>
                </div>
                <div class="card ${above_median ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s162a1.card.above_median">Above industry median?</div>
                    <div class="value">${above_median ? esc(t('view.s162a1.status.yes')) : esc(t('view.s162a1.status.no'))}</div>
                </div>
                <div class="card ${above_75 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s162a1.card.above_75">Above 75th percentile?</div>
                    <div class="value">${above_75 ? esc(t('view.s162a1.status.yes')) : esc(t('view.s162a1.status.no'))}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s162a1.card.excessive">Excessive amount</div>
                    <div class="value">$${excessive_amt.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s162a1.card.tax_disallowed">Disallowed corp deduction (21%)</div>
                    <div class="value">$${reclass_tax_c_corp.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s162a1.card.div_tax">Disguised dividend tax (20%)</div>
                    <div class="value">$${reclass_div_tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s162a1.card.se_tax">S-corp reclass SE (15.3%)</div>
                    <div class="value">$${s_corp_reclass_se.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${!reasonable ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s162a1.unreasonable_note">
                    Compensation NOT reasonable: C-corp deduction disallowed for excess (DOUBLE TAX as
                    dividend); S-corp reclassification triggers SE tax + payroll taxes + penalties. Obtain
                    compensation study (RCReports, Equilar, BLS) + document board decision + maintain
                    contemporaneous records.
                </p>
            ` : ''}
        </div>
    `;
}
