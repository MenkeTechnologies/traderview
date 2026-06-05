// IRC § 106 — Employer-Provided Health Coverage.
// Employer contributions to accident / health plans EXCLUDED from employee gross income.
// Includes: health insurance premiums, HSA contributions, HRA accounts, vision / dental.
// 2%+ S-corp shareholders: NOT excluded — must include on W-2 Box 1 (W-2 exception for SE health deduction).
// Cafeteria plan § 125 + § 106: allows pre-tax salary contribution.

import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    employer_health_premium: 0,
    employer_hsa_contribution: 0,
    employer_hra_funding: 0,
    employer_dental_vision: 0,
    cafeteria_plan_pretax: 0,
    is_2_pct_s_corp: false,
    is_partner: false,
    is_self_employed: false,
    is_employee: true,
    is_retired_employee: false,
    domestic_partner_coverage: 0,
    is_kid_under_27: true,
    is_qualified_high_deductible: false,
    health_savings_per_year: 0,
    marginal_rate: 24,
};

export async function renderSection106(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s106.h1.title">// § 106 EMPLOYER HEALTH COVERAGE</span></h1>
        <p class="muted small" data-i18n="view.s106.hint.intro">
            Employer contributions to accident / health plans <strong>EXCLUDED</strong> from employee gross
            income. Includes: <strong>health insurance premiums, HSA, HRA, vision / dental</strong>. <strong>2%+ S-corp
            shareholders:</strong> NOT excluded — must INCLUDE on W-2 Box 1 (exception: can claim § 162(l)
            SE health deduction). <strong>§ 125 cafeteria plan + § 106:</strong> pre-tax salary contribution.
            <strong>ACA mandate:</strong> employer-shared responsibility for &gt; 50 FTE. <strong>NOT excluded:</strong>
            domestic partner coverage (unless dependent), wellness program incentives, gym memberships.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s106.h2.inputs">Inputs</h2>
            <form id="s106-form" class="inline-form">
                <label><span data-i18n="view.s106.label.premium">Employer health premium ($)</span>
                    <input type="number" step="0.01" name="employer_health_premium" value="${state.employer_health_premium}"></label>
                <label><span data-i18n="view.s106.label.hsa">Employer HSA contribution ($)</span>
                    <input type="number" step="0.01" name="employer_hsa_contribution" value="${state.employer_hsa_contribution}"></label>
                <label><span data-i18n="view.s106.label.hra">Employer HRA funding ($)</span>
                    <input type="number" step="0.01" name="employer_hra_funding" value="${state.employer_hra_funding}"></label>
                <label><span data-i18n="view.s106.label.dv">Dental + vision ($)</span>
                    <input type="number" step="0.01" name="employer_dental_vision" value="${state.employer_dental_vision}"></label>
                <label><span data-i18n="view.s106.label.cafeteria">§ 125 cafeteria pre-tax ($)</span>
                    <input type="number" step="0.01" name="cafeteria_plan_pretax" value="${state.cafeteria_plan_pretax}"></label>
                <label><span data-i18n="view.s106.label.s_corp">2%+ S-corp owner?</span>
                    <input type="checkbox" name="is_2_pct_s_corp" ${state.is_2_pct_s_corp ? 'checked' : ''}></label>
                <label><span data-i18n="view.s106.label.partner">Partner / LLC member?</span>
                    <input type="checkbox" name="is_partner" ${state.is_partner ? 'checked' : ''}></label>
                <label><span data-i18n="view.s106.label.se">Self-employed?</span>
                    <input type="checkbox" name="is_self_employed" ${state.is_self_employed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s106.label.employee">W-2 employee?</span>
                    <input type="checkbox" name="is_employee" ${state.is_employee ? 'checked' : ''}></label>
                <label><span data-i18n="view.s106.label.retired">Retired employee?</span>
                    <input type="checkbox" name="is_retired_employee" ${state.is_retired_employee ? 'checked' : ''}></label>
                <label><span data-i18n="view.s106.label.dp">Domestic partner coverage ($)</span>
                    <input type="number" step="0.01" name="domestic_partner_coverage" value="${state.domestic_partner_coverage}"></label>
                <label><span data-i18n="view.s106.label.kid_27">Kid &lt; 27 covered (PPACA)?</span>
                    <input type="checkbox" name="is_kid_under_27" ${state.is_kid_under_27 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s106.label.hdhp">Qualified high-deductible plan (for HSA)?</span>
                    <input type="checkbox" name="is_qualified_high_deductible" ${state.is_qualified_high_deductible ? 'checked' : ''}></label>
                <label><span data-i18n="view.s106.label.savings">HSA growth (untaxed savings) ($)</span>
                    <input type="number" step="0.01" name="health_savings_per_year" value="${state.health_savings_per_year}"></label>
                <label><span data-i18n="view.s106.label.marginal">Marginal rate %</span>
                    <input type="number" step="0.1" name="marginal_rate" value="${state.marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s106.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s106-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s106.h2.scope">§ 106 scope</h2>
            <ul class="muted small">
                <li data-i18n="view.s106.scope.premium">Health insurance premiums: full exclusion</li>
                <li data-i18n="view.s106.scope.hsa">HSA contributions (§ 223): excluded from income + FICA</li>
                <li data-i18n="view.s106.scope.hra">HRA funding (§ 105 + § 106): employer-funded reimbursement</li>
                <li data-i18n="view.s106.scope.dental_vision">Dental + vision: included in "accident / health" broadly</li>
                <li data-i18n="view.s106.scope.long_term_care">Long-term care insurance (§ 7702B): excluded up to age limits</li>
                <li data-i18n="view.s106.scope.cobra">Employer-paid COBRA premiums for terminated employees: excluded</li>
                <li data-i18n="view.s106.scope.same_sex">Same-sex spousal coverage: excluded post-Windsor (2013) / Obergefell (2015)</li>
                <li data-i18n="view.s106.scope.dp_excluded">Domestic partner (non-spouse): generally NOT § 106 unless tax dependent</li>
                <li data-i18n="view.s106.scope.opposite_sex_cohabitant">Opposite-sex cohabitant: not § 106; tax dependent test applies</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s106.h2.exclusions">Special rules + exclusions</h2>
            <ul class="muted small">
                <li data-i18n="view.s106.exc.2_pct">2%+ S-corp shareholders: NOT § 106 — must include premiums on W-2 Box 1</li>
                <li data-i18n="view.s106.exc.partners">Partners + LLC members: NOT § 106 — must include premiums on Schedule K-1</li>
                <li data-i18n="view.s106.exc.s162l">BUT § 162(l) above-the-line deduction available to all SE persons + 2%+ S-corp</li>
                <li data-i18n="view.s106.exc.kids_27">PPACA: kids ≤ 27 covered tax-free under § 106 (even if not tax dependent)</li>
                <li data-i18n="view.s106.exc.dp_dependent">Domestic partner: § 106 if qualifies as § 152 tax dependent (income + support tests)</li>
                <li data-i18n="view.s106.exc.dp_imputed">Otherwise: domestic partner premiums → IMPUTED INCOME on W-2 Box 1 / FICA</li>
                <li data-i18n="view.s106.exc.retirees">Retiree health coverage: § 106 if structured as continuing employer obligation</li>
                <li data-i18n="view.s106.exc.wellness">Wellness program incentives (cash, gym): NOT § 106 — included as wages</li>
                <li data-i18n="view.s106.exc.surrogate_pay">Surrogate / non-spousal lover coverage: NOT § 106</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s106.h2.cadillac_repealed">"Cadillac tax" (§ 4980I) — REPEALED</h2>
            <ul class="muted small">
                <li data-i18n="view.s106.cad.tcja_repealed">40% excise tax on high-cost employer plans REPEALED by Further Consolidated Appropriations Act 2020</li>
                <li data-i18n="view.s106.cad.was_2020">Was scheduled to take effect 2020 — never enforced</li>
                <li data-i18n="view.s106.cad.replaced">Replaced by: Medical Device Tax repeal + Cadillac tax repeal + Health Insurance Tax repeal</li>
                <li data-i18n="view.s106.cad.deduction_unlimited">Therefore: employer health deduction remains UNLIMITED (subject to § 162 reasonableness)</li>
                <li data-i18n="view.s106.cad.gold_plated">"Gold-plated" plans still permitted without § 4980I impact</li>
                <li data-i18n="view.s106.cad.aca_state">Some state-level cost-sharing taxes remain (CA, NY, MA)</li>
                <li data-i18n="view.s106.cad.future_pdf">Future: § 105(h) discrimination + non-discrimination still apply for self-insured plans</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s106.h2.hsa_combo">HSA + § 106 combo</h2>
            <ul class="muted small">
                <li data-i18n="view.s106.hsa.triple">Triple-tax-advantaged: contribution + growth + qualified medical withdrawal all tax-free</li>
                <li data-i18n="view.s106.hsa.hdhp_required">Requires Qualified High Deductible Plan (HDHP): 2025 min deductible $1,650/$3,300; max OOP $8,300/$16,600</li>
                <li data-i18n="view.s106.hsa.limit_2025">2025 contribution limit: $4,300 individual / $8,550 family + $1,000 catch-up age 55+</li>
                <li data-i18n="view.s106.hsa.employer_employee">Employer + employee contributions COMBINED count to limit</li>
                <li data-i18n="view.s106.hsa.cafeteria">§ 125 cafeteria pre-tax: salary reduction contribution to HSA</li>
                <li data-i18n="view.s106.hsa.investment">Investment growth: tax-free if used for qualified medical (Vanguard, Fidelity HSAs)</li>
                <li data-i18n="view.s106.hsa.retirement">Post-65: withdrawal for non-medical = INCOME (no penalty); for medical = tax-free</li>
                <li data-i18n="view.s106.hsa.medicare_no">Medicare enrollment ENDS HSA contributions (incl. spousal HSA)</li>
                <li data-i18n="view.s106.hsa.fsa_limit">FSA ≠ HSA: $3,300 FSA limit (2025); use-it-or-lose-it + carryover $660</li>
            </ul>
        </div>
    `;
    document.getElementById('s106-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.employer_health_premium = Number(fd.get('employer_health_premium')) || 0;
        state.employer_hsa_contribution = Number(fd.get('employer_hsa_contribution')) || 0;
        state.employer_hra_funding = Number(fd.get('employer_hra_funding')) || 0;
        state.employer_dental_vision = Number(fd.get('employer_dental_vision')) || 0;
        state.cafeteria_plan_pretax = Number(fd.get('cafeteria_plan_pretax')) || 0;
        state.is_2_pct_s_corp = !!fd.get('is_2_pct_s_corp');
        state.is_partner = !!fd.get('is_partner');
        state.is_self_employed = !!fd.get('is_self_employed');
        state.is_employee = !!fd.get('is_employee');
        state.is_retired_employee = !!fd.get('is_retired_employee');
        state.domestic_partner_coverage = Number(fd.get('domestic_partner_coverage')) || 0;
        state.is_kid_under_27 = !!fd.get('is_kid_under_27');
        state.is_qualified_high_deductible = !!fd.get('is_qualified_high_deductible');
        state.health_savings_per_year = Number(fd.get('health_savings_per_year')) || 0;
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s106-output');
    if (!el) return;
    const totalEmployerContrib = state.employer_health_premium + state.employer_hsa_contribution + state.employer_hra_funding + state.employer_dental_vision;
    const cafeteriaContrib = state.cafeteria_plan_pretax;
    const isExcludedTaxpayer = !(state.is_2_pct_s_corp || state.is_partner);
    const excluded = isExcludedTaxpayer ? totalEmployerContrib + cafeteriaContrib : 0;
    const dpImputed = state.domestic_partner_coverage;
    const taxSavingsIncome = excluded * (state.marginal_rate / 100);
    const ficaSavings = excluded * 0.0765;
    const hsaGrowthSavings = state.is_qualified_high_deductible ? state.health_savings_per_year * 0.20 : 0;
    const totalAnnualBenefit = taxSavingsIncome + ficaSavings + hsaGrowthSavings;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s106.h2.result">§ 106 outcome</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.s106.card.excluded">§ 106 excluded</div>
                    <div class="value">$${excluded.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s106.card.tax_savings">Income tax savings</div>
                    <div class="value">$${taxSavingsIncome.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s106.card.fica">FICA savings (7.65%)</div>
                    <div class="value">$${ficaSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s106.card.hsa_growth">HSA growth tax savings</div>
                    <div class="value">$${hsaGrowthSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s106.card.total">Total annual benefit</div>
                    <div class="value">$${totalAnnualBenefit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s106.card.dp_imputed">Domestic partner imputed</div>
                    <div class="value">$${dpImputed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.is_2_pct_s_corp ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s106.s_corp_note">
                    2%+ S-corp shareholder: § 106 EXCLUSION NOT AVAILABLE. Premiums INCLUDED on W-2 Box 1.
                    BUT claim § 162(l) above-the-line deduction (Schedule 1) for SE health insurance —
                    full income offset BUT no FICA reduction. Subject to earned-income limitation.
                </p>
            ` : ''}
        </div>
    `;
}
