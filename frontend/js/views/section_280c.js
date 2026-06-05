// IRC § 280C — No Deduction Allowed for Credit-Generating Expenses.
// Disallows § 162 deduction equal to certain credits claimed (R&D, employer SS for tips, etc.).
// § 280C(a) general rule + § 280C(b) reduced credit election + § 280C(c) R&D credit interaction.
// Key example: § 41 R&D credit — must reduce § 174 R&D deduction by credit amount OR elect § 280C(b) reduced credit.

import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    credit_type: 's41_rd',
    credit_amount: 0,
    is_s280c_b_election: false,
    s174_rd_expense: 0,
    s41_qre: 0,
    s41_aspc: 0,
    has_280c_a_disallowance: false,
    s162_deduction_taken: 0,
    is_s45b_tips: false,
    s45b_credit: 0,
    s45a_indian_employment: 0,
    s45p_armed_forces: 0,
    s51_wotc: 0,
    s41_alt_simplified_credit: false,
    s174_capitalize_amortize: true,
    tax_year: 2024,
    is_corporate: false,
    corporate_rate: 21,
    marginal_individual_rate: 37,
    s174_research_amortization_period: 5,
};

export async function renderSection280C(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s280c.h1.title">// § 280C CREDITS DEDUCTION DISALLOWANCE</span></h1>
        <p class="muted small" data-i18n="view.s280c.hint.intro">
            <strong>§ 280C</strong> denies § 162 deduction equal to certain credits. <strong>(a)</strong>
            wage / salary credits (WOTC § 51, Indian employment § 45A, employer SS on tips § 45B, etc.).
            <strong>(b)</strong> RESERVED for elected reduced § 41 R&D credit. <strong>(c)</strong>
            applies to § 41 R&D credit — taxpayer must REDUCE § 174 R&D expense by credit OR ELECT
            reduced credit (current 79% × normal credit per TCJA: 21% × 100% = ~21% × normal credit).
            <strong>Post-TCJA § 174 capitalization</strong> (5-yr domestic / 15-yr foreign amortization)
            DRAMATICALLY changes calculus. <strong>Pre-2022:</strong> § 174 was fully deductible — election
            was about preserving the current-year benefit. <strong>Post-2022:</strong> already amortized
            — election compares 21% × full credit vs 79% × full credit.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s280c.h2.inputs">Inputs</h2>
            <form id="s280c-form" class="inline-form">
                <label><span data-i18n="view.s280c.label.type">Credit type</span>
                    <select name="credit_type">
                        <option value="s41_rd" ${state.credit_type === 's41_rd' ? 'selected' : ''}>§ 41 R&D credit</option>
                        <option value="s45b_tips" ${state.credit_type === 's45b_tips' ? 'selected' : ''}>§ 45B employer SS on tips</option>
                        <option value="s51_wotc" ${state.credit_type === 's51_wotc' ? 'selected' : ''}>§ 51 Work Opportunity (WOTC)</option>
                        <option value="s45a_indian" ${state.credit_type === 's45a_indian' ? 'selected' : ''}>§ 45A Indian employment</option>
                        <option value="s45p_armed_forces" ${state.credit_type === 's45p_armed_forces' ? 'selected' : ''}>§ 45P Armed Forces differential pay</option>
                        <option value="s45f_childcare" ${state.credit_type === 's45f_childcare' ? 'selected' : ''}>§ 45F Employer-provided childcare</option>
                        <option value="s45s_paid_family" ${state.credit_type === 's45s_paid_family' ? 'selected' : ''}>§ 45S Paid family/medical leave</option>
                    </select>
                </label>
                <label><span data-i18n="view.s280c.label.credit">Credit amount ($)</span>
                    <input type="number" step="0.01" name="credit_amount" value="${state.credit_amount}"></label>
                <label><span data-i18n="view.s280c.label.s280cb">§ 280C(b) reduced credit election?</span>
                    <input type="checkbox" name="is_s280c_b_election" ${state.is_s280c_b_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s280c.label.s174">§ 174 R&D expense ($)</span>
                    <input type="number" step="0.01" name="s174_rd_expense" value="${state.s174_rd_expense}"></label>
                <label><span data-i18n="view.s280c.label.qre">§ 41 QRE ($)</span>
                    <input type="number" step="0.01" name="s41_qre" value="${state.s41_qre}"></label>
                <label><span data-i18n="view.s280c.label.aspc">§ 41 ASPC ($)</span>
                    <input type="number" step="0.01" name="s41_aspc" value="${state.s41_aspc}"></label>
                <label><span data-i18n="view.s280c.label.s280c_a">§ 280C(a) disallowance?</span>
                    <input type="checkbox" name="has_280c_a_disallowance" ${state.has_280c_a_disallowance ? 'checked' : ''}></label>
                <label><span data-i18n="view.s280c.label.s162">§ 162 deduction taken ($)</span>
                    <input type="number" step="0.01" name="s162_deduction_taken" value="${state.s162_deduction_taken}"></label>
                <label><span data-i18n="view.s280c.label.tips">§ 45B tips?</span>
                    <input type="checkbox" name="is_s45b_tips" ${state.is_s45b_tips ? 'checked' : ''}></label>
                <label><span data-i18n="view.s280c.label.s45b">§ 45B credit ($)</span>
                    <input type="number" step="0.01" name="s45b_credit" value="${state.s45b_credit}"></label>
                <label><span data-i18n="view.s280c.label.s45a">§ 45A Indian employment ($)</span>
                    <input type="number" step="0.01" name="s45a_indian_employment" value="${state.s45a_indian_employment}"></label>
                <label><span data-i18n="view.s280c.label.s45p">§ 45P Armed Forces ($)</span>
                    <input type="number" step="0.01" name="s45p_armed_forces" value="${state.s45p_armed_forces}"></label>
                <label><span data-i18n="view.s280c.label.s51">§ 51 WOTC ($)</span>
                    <input type="number" step="0.01" name="s51_wotc" value="${state.s51_wotc}"></label>
                <label><span data-i18n="view.s280c.label.asc">ASC alternative simplified?</span>
                    <input type="checkbox" name="s41_alt_simplified_credit" ${state.s41_alt_simplified_credit ? 'checked' : ''}></label>
                <label><span data-i18n="view.s280c.label.s174_amort">§ 174 5-yr amortization (post-2022)?</span>
                    <input type="checkbox" name="s174_capitalize_amortize" ${state.s174_capitalize_amortize ? 'checked' : ''}></label>
                <label><span data-i18n="view.s280c.label.year">Tax year</span>
                    <input type="number" step="1" name="tax_year" value="${state.tax_year}"></label>
                <label><span data-i18n="view.s280c.label.corp">Corporate?</span>
                    <input type="checkbox" name="is_corporate" ${state.is_corporate ? 'checked' : ''}></label>
                <label><span data-i18n="view.s280c.label.corp_rate">Corporate rate (%)</span>
                    <input type="number" step="0.1" name="corporate_rate" value="${state.corporate_rate}"></label>
                <label><span data-i18n="view.s280c.label.indiv_rate">Individual marginal (%)</span>
                    <input type="number" step="0.1" name="marginal_individual_rate" value="${state.marginal_individual_rate}"></label>
                <label><span data-i18n="view.s280c.label.period">§ 174 amortization period (yrs)</span>
                    <input type="number" step="1" name="s174_research_amortization_period" value="${state.s174_research_amortization_period}"></label>
                <button class="primary" type="submit" data-i18n="view.s280c.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s280c-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s280c.h2.s280c_a">§ 280C(a) — wage/salary credits (no election)</h2>
            <ul class="muted small">
                <li data-i18n="view.s280c.a.s45b">§ 45B employer Social Security on tips</li>
                <li data-i18n="view.s280c.a.s51">§ 51 Work Opportunity Tax Credit (WOTC) — targeted groups</li>
                <li data-i18n="view.s280c.a.s45a">§ 45A Indian employment credit</li>
                <li data-i18n="view.s280c.a.s45p">§ 45P Armed Forces differential pay</li>
                <li data-i18n="view.s280c.a.s1397">§ 1397 Empowerment zone employment</li>
                <li data-i18n="view.s280c.a.consequence">§ 162 deduction REDUCED by credit amount — no election available</li>
                <li data-i18n="view.s280c.a.book">Book-tax adjustment: temporary book/tax difference</li>
                <li data-i18n="view.s280c.a.character">Wages paid still deductible — only matching credit reduces deduction</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s280c.h2.s280c_c">§ 280C(c) — § 41 R&D credit interaction</h2>
            <ol class="muted small">
                <li data-i18n="view.s280c.c.default">Default: reduce § 174 R&D expense by full § 41 credit</li>
                <li data-i18n="view.s280c.c.election">§ 280C(c)(2) ELECTION: reduce credit by (federal rate × credit) — currently 21% × credit</li>
                <li data-i18n="view.s280c.c.formula_corp">Reduced credit = credit × (1 - federal corporate tax rate)</li>
                <li data-i18n="view.s280c.c.post_tcja">Post-TCJA 21% corporate rate: reduced credit = 79% × normal credit</li>
                <li data-i18n="view.s280c.c.pre_tcja">Pre-TCJA 35% corporate rate: reduced credit = 65% × normal credit</li>
                <li data-i18n="view.s280c.c.election_timing">Election made on Form 6765 + timely filed (with extensions)</li>
                <li data-i18n="view.s280c.c.binding">Election BINDING for that year (cannot be revoked without consent)</li>
                <li data-i18n="view.s280c.c.passthrough">Pass-through entities: election at entity level binds all members</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s280c.h2.post_2022">Post-2022 § 174 capitalization changes calculus</h2>
            <ul class="muted small">
                <li data-i18n="view.s280c.post.s174_must_amort">§ 174 R&D must be capitalized + amortized (5-yr domestic / 15-yr foreign) post-2022</li>
                <li data-i18n="view.s280c.post.no_immediate">NO immediate § 174 deduction — only ~10-20% in current year</li>
                <li data-i18n="view.s280c.post.s280c_full_reduction">§ 280C(c) full credit reduction = reduce amortizable basis</li>
                <li data-i18n="view.s280c.post.s280c_b_election_favored">§ 280C(c)(2) election becomes more attractive — preserves full deduction over 5 yrs</li>
                <li data-i18n="view.s280c.post.election_quantitative">Election analysis: 21% × $1M = $210K loss vs preserve $1M × 21% × 5/5 amortization</li>
                <li data-i18n="view.s280c.post.tax_dept_impact">Mid-size R&D companies: § 280C(c)(2) election now standard</li>
                <li data-i18n="view.s280c.post.s174_basis_recovery">Basis recovery over 5/15 yrs reduces benefit of immediate § 280C(c) reduction</li>
                <li data-i18n="view.s280c.post.amt_consideration">§ 38 GBC AMT pre-emption: § 41 credit counts</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s280c.h2.comparison">Election comparison example (post-2022)</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s280c.tbl.scenario">Scenario</th><th data-i18n="view.s280c.tbl.credit">Credit</th><th data-i18n="view.s280c.tbl.deduction">§ 174 amortization basis</th><th data-i18n="view.s280c.tbl.net">Net benefit (Year 1)</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s280c.tbl.no_election">No § 280C(c)(2) election</td><td>$100,000</td><td>$900,000 (reduced)</td><td>$100K - $20K (5/5 of $100K basis loss) = $80K</td></tr>
                    <tr><td data-i18n="view.s280c.tbl.election">§ 280C(c)(2) election</td><td>$79,000 (79% × $100K)</td><td>$1,000,000 (full)</td><td>$79K (no basis reduction)</td></tr>
                </tbody>
            </table>
        </div>
    `;
    document.getElementById('s280c-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.credit_type = fd.get('credit_type');
        state.credit_amount = Number(fd.get('credit_amount')) || 0;
        state.is_s280c_b_election = !!fd.get('is_s280c_b_election');
        state.s174_rd_expense = Number(fd.get('s174_rd_expense')) || 0;
        state.s41_qre = Number(fd.get('s41_qre')) || 0;
        state.s41_aspc = Number(fd.get('s41_aspc')) || 0;
        state.has_280c_a_disallowance = !!fd.get('has_280c_a_disallowance');
        state.s162_deduction_taken = Number(fd.get('s162_deduction_taken')) || 0;
        state.is_s45b_tips = !!fd.get('is_s45b_tips');
        state.s45b_credit = Number(fd.get('s45b_credit')) || 0;
        state.s45a_indian_employment = Number(fd.get('s45a_indian_employment')) || 0;
        state.s45p_armed_forces = Number(fd.get('s45p_armed_forces')) || 0;
        state.s51_wotc = Number(fd.get('s51_wotc')) || 0;
        state.s41_alt_simplified_credit = !!fd.get('s41_alt_simplified_credit');
        state.s174_capitalize_amortize = !!fd.get('s174_capitalize_amortize');
        state.tax_year = Number(fd.get('tax_year')) || 0;
        state.is_corporate = !!fd.get('is_corporate');
        state.corporate_rate = Number(fd.get('corporate_rate')) || 0;
        state.marginal_individual_rate = Number(fd.get('marginal_individual_rate')) || 0;
        state.s174_research_amortization_period = Number(fd.get('s174_research_amortization_period')) || 1;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s280c-output');
    if (!el) return;
    const reduced_credit = state.credit_amount * (1 - state.corporate_rate / 100);
    const no_election_net = state.credit_amount - state.credit_amount * (state.corporate_rate / 100) / state.s174_research_amortization_period;
    const election_net = reduced_credit;
    const better_election = election_net > no_election_net;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s280c.h2.result">§ 280C analysis</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s280c.card.credit">Full credit</div><div class="value">$${state.credit_amount.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s280c.card.reduced">Reduced credit (79%)</div><div class="value">$${reduced_credit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div></div>
                <div class="card"><div class="label" data-i18n="view.s280c.card.no_election">No election net (Y1)</div><div class="value">$${no_election_net.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.s280c.card.election">§ 280C(c)(2) election net</div><div class="value">$${election_net.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div></div>
                <div class="card ${better_election ? 'pos' : 'neg'}"><div class="label" data-i18n="view.s280c.card.choice">Better choice</div><div class="value">${better_election ? '§ 280C(c)(2) ELECT' : 'NO ELECTION'}</div></div>
            </div>
        </div>
    `;
}
