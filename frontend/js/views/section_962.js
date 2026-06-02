// IRC § 962 — Election by Individual to be Taxed at Corporate Rates on Subpart F + GILTI.
// US individual shareholder of CFC elects to be taxed at corporate 21% on subpart F + GILTI income.
// Allows § 250 GILTI deduction (50%) + § 960 indirect foreign tax credit.
// Trade-off: dividends from CFC are NOT excluded later (taxable again).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    subpart_f_income: 0,
    gilti_inclusion: 0,
    ftc_basket: 'gilti',
    foreign_tax_paid: 0,
    foreign_tax_paid_basis: 0,
    cfc_country: 'Ireland',
    election_year: 2024,
    individual_top_rate: 37,
    is_election_filed: false,
    s250_50pct_deduction: 0,
    s951a_gilti: 0,
    qbai_amount: 0,
    s960_indirect_ftc: 0,
    cfc_e_p: 0,
    later_dividend: 0,
    is_pti_distribution: false,
    s962b_basis_in_pti: 0,
    multiple_cfc_aggregation: false,
    high_tax_exclusion_election: false,
    high_tax_rate_threshold: 18.9,
};

export async function renderSection962(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s962.h1.title">// § 962 ELECTION (CORPORATE RATES ON CFC)</span></h1>
        <p class="muted small" data-i18n="view.s962.hint.intro">
            <strong>§ 962 election:</strong> US individual shareholder of CFC elects taxed at corporate
            <strong>21% rate</strong> on subpart F + GILTI income — INSTEAD of individual rate (up to
            37%). <strong>UNLOCKS:</strong> § 250 50% GILTI deduction + § 960 indirect foreign tax
            credit. <strong>TRADE-OFF:</strong> Subsequent CFC dividends to individual are TAXABLE
            (vs § 959 PTI exclusion would apply). <strong>Use case:</strong> high-tax CFC + foreign
            tax credit usable + total tax less than 37% individual. <strong>§ 962(b) basis:</strong>
            individual gets basis in PTI equal to corporate tax paid; subsequent distribution treated
            as taxable dividend MINUS basis recovery. <strong>Annual election</strong> on Form 1040
            attachment.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s962.h2.inputs">Inputs</h2>
            <form id="s962-form" class="inline-form">
                <label><span data-i18n="view.s962.label.subpart_f">Subpart F income ($)</span>
                    <input type="number" step="10000" name="subpart_f_income" value="${state.subpart_f_income}"></label>
                <label><span data-i18n="view.s962.label.gilti">GILTI inclusion ($)</span>
                    <input type="number" step="10000" name="gilti_inclusion" value="${state.gilti_inclusion}"></label>
                <label><span data-i18n="view.s962.label.basket">FTC basket</span>
                    <select name="ftc_basket">
                        <option value="general" ${state.ftc_basket === 'general' ? 'selected' : ''}>General</option>
                        <option value="passive" ${state.ftc_basket === 'passive' ? 'selected' : ''}>Passive (FPHCI)</option>
                        <option value="gilti" ${state.ftc_basket === 'gilti' ? 'selected' : ''}>GILTI</option>
                        <option value="foreign_branch" ${state.ftc_basket === 'foreign_branch' ? 'selected' : ''}>Foreign branch</option>
                    </select>
                </label>
                <label><span data-i18n="view.s962.label.ftc">Foreign tax paid ($)</span>
                    <input type="number" step="10000" name="foreign_tax_paid" value="${state.foreign_tax_paid}"></label>
                <label><span data-i18n="view.s962.label.ftc_basis">FTC basis ($)</span>
                    <input type="number" step="10000" name="foreign_tax_paid_basis" value="${state.foreign_tax_paid_basis}"></label>
                <label><span data-i18n="view.s962.label.country">CFC country</span>
                    <input type="text" name="cfc_country" value="${esc(state.cfc_country)}"></label>
                <label><span data-i18n="view.s962.label.year">Election year</span>
                    <input type="number" step="1" name="election_year" value="${state.election_year}"></label>
                <label><span data-i18n="view.s962.label.individual_rate">Individual top rate (%)</span>
                    <input type="number" step="0.1" name="individual_top_rate" value="${state.individual_top_rate}"></label>
                <label><span data-i18n="view.s962.label.filed">Election filed?</span>
                    <input type="checkbox" name="is_election_filed" ${state.is_election_filed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s962.label.s250">§ 250 50% deduction ($)</span>
                    <input type="number" step="10000" name="s250_50pct_deduction" value="${state.s250_50pct_deduction}"></label>
                <label><span data-i18n="view.s962.label.s951a">§ 951A GILTI ($)</span>
                    <input type="number" step="10000" name="s951a_gilti" value="${state.s951a_gilti}"></label>
                <label><span data-i18n="view.s962.label.qbai">QBAI ($)</span>
                    <input type="number" step="10000" name="qbai_amount" value="${state.qbai_amount}"></label>
                <label><span data-i18n="view.s962.label.s960">§ 960 indirect FTC ($)</span>
                    <input type="number" step="10000" name="s960_indirect_ftc" value="${state.s960_indirect_ftc}"></label>
                <label><span data-i18n="view.s962.label.ep">CFC E&P ($)</span>
                    <input type="number" step="10000" name="cfc_e_p" value="${state.cfc_e_p}"></label>
                <label><span data-i18n="view.s962.label.later_div">Later dividend ($)</span>
                    <input type="number" step="10000" name="later_dividend" value="${state.later_dividend}"></label>
                <label><span data-i18n="view.s962.label.pti">PTI distribution?</span>
                    <input type="checkbox" name="is_pti_distribution" ${state.is_pti_distribution ? 'checked' : ''}></label>
                <label><span data-i18n="view.s962.label.s962b_basis">§ 962(b) basis in PTI ($)</span>
                    <input type="number" step="10000" name="s962b_basis_in_pti" value="${state.s962b_basis_in_pti}"></label>
                <label><span data-i18n="view.s962.label.multi_cfc">Multiple CFC aggregation?</span>
                    <input type="checkbox" name="multiple_cfc_aggregation" ${state.multiple_cfc_aggregation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s962.label.htex">§ 954(b)(4) high-tax exclusion?</span>
                    <input type="checkbox" name="high_tax_exclusion_election" ${state.high_tax_exclusion_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s962.label.htex_rate">High-tax threshold (%)</span>
                    <input type="number" step="0.1" name="high_tax_rate_threshold" value="${state.high_tax_rate_threshold}"></label>
                <button class="primary" type="submit" data-i18n="view.s962.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s962-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s962.h2.mechanics">§ 962 election mechanics</h2>
            <ol class="muted small">
                <li data-i18n="view.s962.mech.election">Annual election on Form 1040 (attached statement)</li>
                <li data-i18n="view.s962.mech.rate">Pay tax at CORPORATE 21% rate (vs individual 37%)</li>
                <li data-i18n="view.s962.mech.s250">§ 250 GILTI 50% deduction (effective 10.5% rate)</li>
                <li data-i18n="view.s962.mech.s960">§ 960 indirect FTC: 80% of foreign tax × pro-rata share</li>
                <li data-i18n="view.s962.mech.basis_pti">§ 962(b): basis in PTI = corporate tax paid</li>
                <li data-i18n="view.s962.mech.dividend">Subsequent CFC dividends: taxable to extent exceed § 962(b) basis</li>
                <li data-i18n="view.s962.mech.no_step_up">NO step-up in CFC stock basis from inclusions</li>
                <li data-i18n="view.s962.mech.binding">Binding for that year — cannot revoke</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s962.h2.calculation">Tax calculation comparison</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s962.tbl.scenario">Scenario</th><th data-i18n="view.s962.tbl.rate">Effective rate</th><th data-i18n="view.s962.tbl.note">Notes</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s962.tbl.individual_no">Individual, no § 962</td><td>37%</td><td data-i18n="view.s962.tbl.no_s250">No § 250 50% deduction available</td></tr>
                    <tr><td data-i18n="view.s962.tbl.s962_no_ftc">§ 962 + no FTC</td><td>10.5%</td><td>21% × 50% GILTI deduction</td></tr>
                    <tr><td data-i18n="view.s962.tbl.s962_ftc">§ 962 + 80% FTC</td><td>~0% (if high-tax CFC)</td><td>21% × 50% × (1 - 80%)</td></tr>
                    <tr><td data-i18n="view.s962.tbl.dividend_later">Later dividend</td><td>up to 37%</td><td data-i18n="view.s962.tbl.no_dre_individual">Individual NOT eligible for § 245A DRD</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s962.h2.use_cases">When § 962 election makes sense</h2>
            <ul class="muted small">
                <li data-i18n="view.s962.uc.high_tax_cfc">CFC in high-tax jurisdiction (Ireland 12.5%, Israel 23%, etc.)</li>
                <li data-i18n="view.s962.uc.large_inclusion">Large GILTI inclusion that would otherwise be at 37%</li>
                <li data-i18n="view.s962.uc.no_distribution">Plan NOT to distribute CFC earnings near-term</li>
                <li data-i18n="view.s962.uc.s954_high_tax">Combined with § 954(b)(4) GILTI high-tax exclusion</li>
                <li data-i18n="view.s962.uc.tested_loss">Multiple CFCs with mixed tested income/loss (aggregation benefits)</li>
                <li data-i18n="view.s962.uc.qualified_basis">Sufficient QBAI to reduce tested income inclusion</li>
                <li data-i18n="view.s962.uc.simple_ownership">Simple ownership structure (1-tier CFC)</li>
                <li data-i18n="view.s962.uc.basis_growth">Plan to grow basis via PTI for later tax-free distribution</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s962.h2.traps">Traps + downsides</h2>
            <ul class="muted small">
                <li data-i18n="view.s962.trap.dividend">Subsequent CFC dividend: taxable AGAIN (no § 245A DRD for individuals)</li>
                <li data-i18n="view.s962.trap.smith_v_commissioner">Smith v. Comm. — pre-2018 case held § 962 dividend treated as PTI distribution</li>
                <li data-i18n="view.s962.trap.s962b_basis_dividend">§ 962(b)(2) basis only reduces dividend to extent of corp tax paid</li>
                <li data-i18n="view.s962.trap.state_tax">State tax: most states do not respect § 962 election — full individual rate</li>
                <li data-i18n="view.s962.trap.amt">Pre-TCJA AMT: § 962 didn't apply to AMT</li>
                <li data-i18n="view.s962.trap.s199a">§ 199A QBI: NOT available for § 962-elected income</li>
                <li data-i18n="view.s962.trap.s250_loss">If tested loss, § 250 deduction limited to tested income</li>
                <li data-i18n="view.s962.trap.s960_80pct">§ 960 FTC only 80% — 20% leakage</li>
                <li data-i18n="view.s962.trap.s86">Social Security taxation NOT affected (still individual income)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s962.h2.alternatives">Alternatives to § 962</h2>
            <ul class="muted small">
                <li data-i18n="view.s962.alt.s954_b4">§ 954(b)(4) GILTI high-tax exclusion (if CFC ETR > 18.9%)</li>
                <li data-i18n="view.s962.alt.entity_blocker">Insert US C-corp blocker between individual + CFC</li>
                <li data-i18n="view.s962.alt.partnership">Use partnership instead of CFC (no § 951(a) inclusion)</li>
                <li data-i18n="view.s962.alt.dle">Disregarded entity election (Form 8832)</li>
                <li data-i18n="view.s962.alt.expatriation">Expatriation (§ 877A) — but covered expatriate rules apply</li>
                <li data-i18n="view.s962.alt.cfcsale">Sell CFC stock + recapture via § 1248</li>
                <li data-i18n="view.s962.alt.distribute_first">Distribute pre-2018 E&P first (transition tax already paid)</li>
                <li data-i18n="view.s962.alt.dividend_treaty">Treaty country: reduce withholding on later dividend</li>
            </ul>
        </div>
    `;
    document.getElementById('s962-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.subpart_f_income = Number(fd.get('subpart_f_income')) || 0;
        state.gilti_inclusion = Number(fd.get('gilti_inclusion')) || 0;
        state.ftc_basket = fd.get('ftc_basket');
        state.foreign_tax_paid = Number(fd.get('foreign_tax_paid')) || 0;
        state.foreign_tax_paid_basis = Number(fd.get('foreign_tax_paid_basis')) || 0;
        state.cfc_country = fd.get('cfc_country') || '';
        state.election_year = Number(fd.get('election_year')) || 0;
        state.individual_top_rate = Number(fd.get('individual_top_rate')) || 0;
        state.is_election_filed = !!fd.get('is_election_filed');
        state.s250_50pct_deduction = Number(fd.get('s250_50pct_deduction')) || 0;
        state.s951a_gilti = Number(fd.get('s951a_gilti')) || 0;
        state.qbai_amount = Number(fd.get('qbai_amount')) || 0;
        state.s960_indirect_ftc = Number(fd.get('s960_indirect_ftc')) || 0;
        state.cfc_e_p = Number(fd.get('cfc_e_p')) || 0;
        state.later_dividend = Number(fd.get('later_dividend')) || 0;
        state.is_pti_distribution = !!fd.get('is_pti_distribution');
        state.s962b_basis_in_pti = Number(fd.get('s962b_basis_in_pti')) || 0;
        state.multiple_cfc_aggregation = !!fd.get('multiple_cfc_aggregation');
        state.high_tax_exclusion_election = !!fd.get('high_tax_exclusion_election');
        state.high_tax_rate_threshold = Number(fd.get('high_tax_rate_threshold')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s962-output');
    if (!el) return;
    const total_inclusion = state.subpart_f_income + state.gilti_inclusion;
    const after_s250 = state.gilti_inclusion - state.s250_50pct_deduction + state.subpart_f_income;
    const corporate_tax = after_s250 * 0.21;
    const ftc_credit = state.s960_indirect_ftc * 0.8;
    const net_tax_s962 = Math.max(0, corporate_tax - ftc_credit);
    const tax_without_s962 = total_inclusion * (state.individual_top_rate / 100);
    const savings = tax_without_s962 - net_tax_s962;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s962.h2.result">§ 962 election analysis</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s962.card.inclusion">Total inclusion</div><div class="value">$${total_inclusion.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s962.card.corp">Corporate tax (21%)</div><div class="value">$${corporate_tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.s962.card.ftc">§ 960 FTC (80%)</div><div class="value">$${ftc_credit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.s962.card.s962_tax">§ 962 net tax</div><div class="value">$${net_tax_s962.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.s962.card.individual_tax">Without § 962 (individual)</div><div class="value">$${tax_without_s962.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div></div>
                <div class="card ${savings > 0 ? 'pos' : 'neg'}"><div class="label" data-i18n="view.s962.card.savings">Savings (current year)</div><div class="value">$${savings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div></div>
            </div>
        </div>
    `;
}
