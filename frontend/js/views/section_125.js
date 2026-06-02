// IRC § 125 — Cafeteria Plan (Section 125 Plan).
// Allows employees to CHOOSE between cash + qualified benefits without constructive receipt.
// Common: pre-tax health insurance, FSA, HSA contributions, dependent care.
// Non-discrimination tests: eligibility, contribution + benefits, key employee concentration.
// Written plan required + irrevocable elections (except qualifying status changes).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const FSA_LIMIT_2024 = 3_200;
const DCFSA_LIMIT_2024 = 5_000;

let state = {
    health_insurance_pretax: 0,
    health_fsa_election: 0,
    dependent_care_fsa: 0,
    hsa_pretax_contribution: 0,
    other_qualified_benefits: 0,
    annual_salary: 0,
    fed_marginal_rate: 0.32,
    state_marginal_rate: 0.06,
    has_written_plan: true,
    passes_eligibility_test: true,
    passes_contribution_test: true,
    passes_concentration_test: true,
};

export async function renderSection125(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s125.h1.title">// § 125 CAFETERIA PLAN</span></h1>
        <p class="muted small" data-i18n="view.s125.hint.intro">
            Allows employees to CHOOSE between cash + qualified benefits without constructive
            receipt. Common: <strong>pre-tax health insurance, Health FSA ($3,200/yr),
            Dependent Care FSA ($5,000), HSA contributions</strong>. Salary reduction reduces
            both income tax + FICA. <strong>Non-discrimination tests:</strong> eligibility,
            contribution + benefits, key employee concentration. <strong>Written plan + irrevocable
            elections</strong> (except qualifying status changes).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s125.h2.inputs">Inputs</h2>
            <form id="s125-form" class="inline-form">
                <label><span data-i18n="view.s125.label.salary">Annual salary ($)</span>
                    <input type="number" step="1000" name="annual_salary" value="${state.annual_salary}"></label>
                <label><span data-i18n="view.s125.label.health_ins">Health insurance pre-tax ($)</span>
                    <input type="number" step="100" name="health_insurance_pretax" value="${state.health_insurance_pretax}"></label>
                <label><span data-i18n="view.s125.label.fsa">Health FSA election ($)</span>
                    <input type="number" step="100" name="health_fsa_election" value="${state.health_fsa_election}"></label>
                <label><span data-i18n="view.s125.label.dcfsa">Dependent Care FSA ($)</span>
                    <input type="number" step="100" name="dependent_care_fsa" value="${state.dependent_care_fsa}"></label>
                <label><span data-i18n="view.s125.label.hsa">HSA pre-tax contribution ($)</span>
                    <input type="number" step="100" name="hsa_pretax_contribution" value="${state.hsa_pretax_contribution}"></label>
                <label><span data-i18n="view.s125.label.other">Other qualified benefits ($)</span>
                    <input type="number" step="100" name="other_qualified_benefits" value="${state.other_qualified_benefits}"></label>
                <label><span data-i18n="view.s125.label.fed_rate">Federal marginal %</span>
                    <input type="number" step="0.01" name="fed_marginal_rate" value="${state.fed_marginal_rate}"></label>
                <label><span data-i18n="view.s125.label.state_rate">State marginal %</span>
                    <input type="number" step="0.01" name="state_marginal_rate" value="${state.state_marginal_rate}"></label>
                <label><span data-i18n="view.s125.label.written">Written plan?</span>
                    <input type="checkbox" name="has_written_plan" ${state.has_written_plan ? 'checked' : ''}></label>
                <label><span data-i18n="view.s125.label.eligibility">Passes eligibility test?</span>
                    <input type="checkbox" name="passes_eligibility_test" ${state.passes_eligibility_test ? 'checked' : ''}></label>
                <label><span data-i18n="view.s125.label.contribution">Passes contribution test?</span>
                    <input type="checkbox" name="passes_contribution_test" ${state.passes_contribution_test ? 'checked' : ''}></label>
                <label><span data-i18n="view.s125.label.concentration">Passes concentration test?</span>
                    <input type="checkbox" name="passes_concentration_test" ${state.passes_concentration_test ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s125.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s125-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s125.h2.qualified_benefits">Qualified benefits</h2>
            <ul class="muted small">
                <li data-i18n="view.s125.qb.health">Group-term life insurance up to $50k</li>
                <li data-i18n="view.s125.qb.medical">Accident + health insurance premiums</li>
                <li data-i18n="view.s125.qb.fsa">Health FSA: $3,200 (2024), $640 carryover, 2.5 mo grace period</li>
                <li data-i18n="view.s125.qb.dcfsa">Dependent Care FSA: $5,000 ($2,500 MFS)</li>
                <li data-i18n="view.s125.qb.hsa">HSA contributions if HDHP</li>
                <li data-i18n="view.s125.qb.adoption">Adoption assistance up to $16,810 (2024) via § 137</li>
                <li data-i18n="view.s125.qb.transportation">Transportation fringe via § 132(f) [separate plan, not § 125]</li>
                <li data-i18n="view.s125.qb.401k">401(k) contributions (pre-tax employee deferrals; usually separate plan but can interact)</li>
                <li data-i18n="view.s125.qb.disability">Long-term disability (limited)</li>
                <li data-i18n="view.s125.qb.no_education">NOT educational assistance (use § 127)</li>
                <li data-i18n="view.s125.qb.no_individual">NOT individual long-term care insurance</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s125.h2.status_changes">Qualifying status changes (mid-year election change)</h2>
            <ul class="muted small">
                <li data-i18n="view.s125.sc.marriage">Marriage / divorce / legal separation</li>
                <li data-i18n="view.s125.sc.birth">Birth / adoption / death of dependent</li>
                <li data-i18n="view.s125.sc.employment">Change in employment status (you / spouse / dependent)</li>
                <li data-i18n="view.s125.sc.dependent">Dependent satisfies / ceases to satisfy eligibility</li>
                <li data-i18n="view.s125.sc.residence">Residence change affecting eligibility</li>
                <li data-i18n="view.s125.sc.medicare">Medicare / Medicaid eligibility change</li>
                <li data-i18n="view.s125.sc.cobra">COBRA event for spouse / dependent</li>
                <li data-i18n="view.s125.sc.court_order">Court order (QMCSO, etc.)</li>
                <li data-i18n="view.s125.sc.must_be_consistent">Change must be consistent with status event</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s125.h2.nondiscrim_tests">Non-discrimination tests</h2>
            <ul class="muted small">
                <li data-i18n="view.s125.nd.eligibility">Eligibility: no group of HCEs treated more favorably for participation</li>
                <li data-i18n="view.s125.nd.contributions">Contributions + benefits: comparable terms for all participants</li>
                <li data-i18n="view.s125.nd.key_employee">Key employee concentration: ≤ 25% of nontaxable benefits go to key employees</li>
                <li data-i18n="view.s125.nd.simple_cafeteria">Simple cafeteria plan (≤ 100 employees): safe harbor passes all tests automatically</li>
                <li data-i18n="view.s125.nd.failed">Failed: HCEs lose tax-favored treatment for full year (recharacterized income)</li>
                <li data-i18n="view.s125.nd.testing_date">Annual testing on last day of plan year</li>
            </ul>
        </div>
    `;
    document.getElementById('s125-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.annual_salary = Number(fd.get('annual_salary')) || 0;
        state.health_insurance_pretax = Number(fd.get('health_insurance_pretax')) || 0;
        state.health_fsa_election = Math.min(Number(fd.get('health_fsa_election')) || 0, FSA_LIMIT_2024);
        state.dependent_care_fsa = Math.min(Number(fd.get('dependent_care_fsa')) || 0, DCFSA_LIMIT_2024);
        state.hsa_pretax_contribution = Number(fd.get('hsa_pretax_contribution')) || 0;
        state.other_qualified_benefits = Number(fd.get('other_qualified_benefits')) || 0;
        state.fed_marginal_rate = Number(fd.get('fed_marginal_rate')) || 0.32;
        state.state_marginal_rate = Number(fd.get('state_marginal_rate')) || 0.06;
        state.has_written_plan = !!fd.get('has_written_plan');
        state.passes_eligibility_test = !!fd.get('passes_eligibility_test');
        state.passes_contribution_test = !!fd.get('passes_contribution_test');
        state.passes_concentration_test = !!fd.get('passes_concentration_test');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s125-output');
    if (!el) return;
    const planQualifies = state.has_written_plan && state.passes_eligibility_test
        && state.passes_contribution_test && state.passes_concentration_test;
    const totalSalaryReduction = state.health_insurance_pretax + state.health_fsa_election
        + state.dependent_care_fsa + state.hsa_pretax_contribution + state.other_qualified_benefits;
    const totalRate = state.fed_marginal_rate + state.state_marginal_rate + 0.0765;
    const fedStateSavings = totalSalaryReduction * (state.fed_marginal_rate + state.state_marginal_rate);
    const ficaSavings = totalSalaryReduction * 0.0765;
    const totalSavings = fedStateSavings + ficaSavings;
    const adjustedW2Box1 = Math.max(0, state.annual_salary - totalSalaryReduction);
    const adjustedW2Box3 = Math.max(0, state.annual_salary - totalSalaryReduction);  // most reduce FICA too
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s125.h2.result">Salary reduction + tax savings</h2>
            <div class="cards">
                <div class="card ${planQualifies ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s125.card.qualifies">Plan qualifies</div>
                    <div class="value">${planQualifies ? esc(t('view.s125.status.yes')) : esc(t('view.s125.status.no'))}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s125.card.total_reduction">Total salary reduction</div>
                    <div class="value">$${totalSalaryReduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s125.card.fed_state">Federal + state savings</div>
                    <div class="value">$${fedStateSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s125.card.fica">FICA savings</div>
                    <div class="value">$${ficaSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s125.card.total_savings">Total tax savings</div>
                    <div class="value">$${totalSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s125.card.adjusted_w2">Adjusted W-2 Box 1</div>
                    <div class="value">$${adjustedW2Box1.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
