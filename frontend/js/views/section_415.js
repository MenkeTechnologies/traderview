// IRC § 415 — Combined Defined Contribution Limits.
// 2024: $69,000 annual additions limit (employee + employer + after-tax) per individual per employer.
// § 415(b) Defined benefit: $275,000 annual benefit cap.
// Compensation cap (§ 401(a)(17)): $345,000 (2024).
// HCE threshold (§ 414(q)): $155,000 (2024).

import { currentViewToken, viewIsCurrent } from '../app.js';

const DEFINED_CONTRIBUTION_2024 = 69_000;
const DEFINED_BENEFIT_2024 = 275_000;
const COMPENSATION_CAP_2024 = 345_000;
const HCE_THRESHOLD_2024 = 155_000;
const CATCH_UP_50 = 7_500;
const CATCH_UP_60 = 11_250;
const EMPLOYEE_DEFERRAL_2024 = 23_000;

let state = {
    age: 40,
    employer_count: 1,
    employee_deferral_a: 0,
    employee_deferral_b: 0,
    employer_contribution_a: 0,
    employer_contribution_b: 0,
    after_tax_contribution_a: 0,
    after_tax_contribution_b: 0,
    has_457b: false,
    deferral_457b: 0,
    total_compensation: 0,
    marginal_rate: 0.32,
};

export async function renderSection415(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s415.h1.title">// § 415 401(k) COMBINED LIMITS</span></h1>
        <p class="muted small" data-i18n="view.s415.hint.intro">
            <strong>2024 annual additions:</strong> $69,000 (employee + employer + after-tax) per
            individual <strong>per employer</strong>. <strong>§ 415(b) Defined benefit:</strong>
            $275,000 annual benefit cap. <strong>Comp cap (§ 401(a)(17)):</strong> $345,000.
            <strong>HCE threshold (§ 414(q)):</strong> $155,000. <strong>457(b) stacks</strong>
            on 401(k) limit (separate $23k). <strong>Multiple unrelated employers:</strong> separate
            $69k limits but combined employee deferral $23k.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s415.h2.inputs">Inputs</h2>
            <form id="s415-form" class="inline-form">
                <label><span data-i18n="view.s415.label.age">Your age</span>
                    <input type="number" step="1" name="age" value="${state.age}"></label>
                <label><span data-i18n="view.s415.label.employers">Employer count</span>
                    <input type="number" step="1" name="employer_count" value="${state.employer_count}"></label>
                <label><span data-i18n="view.s415.label.deferral_a">Employer A: Employee deferral ($)</span>
                    <input type="number" step="0.01" name="employee_deferral_a" value="${state.employee_deferral_a}"></label>
                <label><span data-i18n="view.s415.label.deferral_b">Employer B: Employee deferral ($)</span>
                    <input type="number" step="0.01" name="employee_deferral_b" value="${state.employee_deferral_b}"></label>
                <label><span data-i18n="view.s415.label.employer_a">Employer A: Employer match / profit-share ($)</span>
                    <input type="number" step="0.01" name="employer_contribution_a" value="${state.employer_contribution_a}"></label>
                <label><span data-i18n="view.s415.label.employer_b">Employer B: Employer contribution ($)</span>
                    <input type="number" step="0.01" name="employer_contribution_b" value="${state.employer_contribution_b}"></label>
                <label><span data-i18n="view.s415.label.after_tax_a">Employer A: After-tax ($)</span>
                    <input type="number" step="0.01" name="after_tax_contribution_a" value="${state.after_tax_contribution_a}"></label>
                <label><span data-i18n="view.s415.label.after_tax_b">Employer B: After-tax ($)</span>
                    <input type="number" step="0.01" name="after_tax_contribution_b" value="${state.after_tax_contribution_b}"></label>
                <label><span data-i18n="view.s415.label.457b">Has 457(b) plan?</span>
                    <input type="checkbox" name="has_457b" ${state.has_457b ? 'checked' : ''}></label>
                <label><span data-i18n="view.s415.label.deferral_457b">457(b) deferral ($)</span>
                    <input type="number" step="0.01" name="deferral_457b" value="${state.deferral_457b}"></label>
                <label><span data-i18n="view.s415.label.comp">Total compensation ($)</span>
                    <input type="number" step="0.01" name="total_compensation" value="${state.total_compensation}"></label>
                <label><span data-i18n="view.s415.label.marginal">Marginal %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s415.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s415-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s415.h2.limits_2024">2024 retirement plan limits</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s415.th.limit">Limit</th>
                    <th data-i18n="view.s415.th.amount">Amount</th>
                </tr></thead>
                <tbody>
                    <tr><td>§ 415(c) Annual additions cap (DC)</td><td>$69,000</td></tr>
                    <tr><td>§ 402(g) Employee deferral cap</td><td>$23,000</td></tr>
                    <tr><td>§ 414(v) Catch-up 50+</td><td>$7,500</td></tr>
                    <tr><td>§ 414(v) Super catch-up 60-63 (SECURE 2.0)</td><td>$11,250</td></tr>
                    <tr><td>§ 415(b) Defined benefit cap</td><td>$275,000/yr</td></tr>
                    <tr><td>§ 401(a)(17) Compensation cap</td><td>$345,000</td></tr>
                    <tr><td>§ 414(q) HCE threshold</td><td>$155,000</td></tr>
                    <tr><td>§ 416 Key Employee threshold</td><td>$220,000</td></tr>
                    <tr><td>§ 401(k)(11) SIMPLE 401(k) deferral</td><td>$16,000</td></tr>
                    <tr><td>§ 457(b) Government deferral</td><td>$23,000 (stacks on 401(k))</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s415.h2.aggregation">§ 414(b)(c)(m)(o) Aggregation rules</h2>
            <ul class="muted small">
                <li data-i18n="view.s415.agg.controlled">Controlled group (parent-sub 80%+ OR brother-sister 80% identical owners): treated as ONE employer for § 415</li>
                <li data-i18n="view.s415.agg.affiliated">Affiliated service group: management or service relationship + ownership / common service</li>
                <li data-i18n="view.s415.agg.brother_sister">Brother-sister: 5 or fewer owners + each control 80% combined + 50% identical</li>
                <li data-i18n="view.s415.agg.unrelated">Unrelated employers: SEPARATE $69k limits (employee can stack but $23k 402(g) combined)</li>
                <li data-i18n="view.s415.agg.self_employed">Self-employed + W-2 job: separate; can max both but 402(g) deferral shared</li>
                <li data-i18n="view.s415.agg.spouse_attribution">§ 318 spouse attribution can convert "unrelated" to "related"</li>
            </ul>
        </div>
    `;
    document.getElementById('s415-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.age = Number(fd.get('age')) || 40;
        state.employer_count = Number(fd.get('employer_count')) || 1;
        state.employee_deferral_a = Number(fd.get('employee_deferral_a')) || 0;
        state.employee_deferral_b = Number(fd.get('employee_deferral_b')) || 0;
        state.employer_contribution_a = Number(fd.get('employer_contribution_a')) || 0;
        state.employer_contribution_b = Number(fd.get('employer_contribution_b')) || 0;
        state.after_tax_contribution_a = Number(fd.get('after_tax_contribution_a')) || 0;
        state.after_tax_contribution_b = Number(fd.get('after_tax_contribution_b')) || 0;
        state.has_457b = !!fd.get('has_457b');
        state.deferral_457b = Number(fd.get('deferral_457b')) || 0;
        state.total_compensation = Number(fd.get('total_compensation')) || 0;
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.32;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s415-output');
    if (!el) return;
    const catchUp = state.age >= 60 ? CATCH_UP_60 : (state.age >= 50 ? CATCH_UP_50 : 0);
    const deferralCap = EMPLOYEE_DEFERRAL_2024 + catchUp;
    const totalDeferral = state.employee_deferral_a + state.employee_deferral_b;
    const deferralExcess = Math.max(0, totalDeferral - deferralCap);
    const cappedA = Math.min(state.employee_deferral_a + state.employer_contribution_a + state.after_tax_contribution_a, DEFINED_CONTRIBUTION_2024 + catchUp);
    const cappedB = Math.min(state.employee_deferral_b + state.employer_contribution_b + state.after_tax_contribution_b, DEFINED_CONTRIBUTION_2024 + catchUp);
    const total415 = cappedA + cappedB;
    const total457b = state.has_457b ? Math.min(state.deferral_457b, EMPLOYEE_DEFERRAL_2024) : 0;
    const totalTaxAdvantaged = total415 + total457b;
    const compensationCapApplies = state.total_compensation > COMPENSATION_CAP_2024;
    const taxSavings = (totalDeferral + state.employer_contribution_a + state.employer_contribution_b + total457b) * state.marginal_rate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s415.h2.result">Combined limits analysis</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s415.card.deferral_cap">Deferral cap (incl. catch-up)</div>
                    <div class="value">$${deferralCap.toLocaleString()}</div>
                </div>
                <div class="card ${deferralExcess > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s415.card.deferral_excess">402(g) deferral excess</div>
                    <div class="value">$${deferralExcess.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s415.card.cap_a">Employer A 415(c) capped</div>
                    <div class="value">$${cappedA.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s415.card.cap_b">Employer B 415(c) capped</div>
                    <div class="value">$${cappedB.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s415.card.457b">457(b) (separate)</div>
                    <div class="value">$${total457b.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s415.card.total">Total tax-advantaged</div>
                    <div class="value">$${totalTaxAdvantaged.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s415.card.savings">Tax savings</div>
                    <div class="value">$${taxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${compensationCapApplies ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.s415.card.comp_cap">Comp cap exceeded</div>
                        <div class="value">$${COMPENSATION_CAP_2024.toLocaleString()}</div>
                    </div>
                ` : ''}
            </div>
        </div>
    `;
}
