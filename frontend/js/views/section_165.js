// IRC § 165 — Losses (general rule for casualty + theft + worthless securities + abandoned).
// Allowed losses sustained during taxable year + not compensated by insurance.
// 4 categories: trade or business, transactions entered for profit, personal casualty/theft
// (after TCJA: only federally declared disasters), wagering.

import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    loss_category: 'trade_business',
    loss_amount: 0,
    fmv_before: 0,
    fmv_after: 0,
    adjusted_basis: 0,
    insurance_reimbursement: 0,
    is_federally_declared_disaster: false,
    is_personal_use: false,
    is_business_use: false,
    is_investment_use: false,
    is_theft: false,
    is_casualty: false,
    is_worthless_security: false,
    security_basis: 0,
    is_wagering_loss: false,
    wagering_gains: 0,
    wagering_losses: 0,
    is_abandonment: false,
    abandonment_basis: 0,
    is_ponzi: false,
    rev_proc_2009_20: false,
    year: 2024,
    is_filing_4684: false,
};

export async function renderSection165(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s165.h1.title">// § 165 LOSSES</span></h1>
        <p class="muted small" data-i18n="view.s165.hint.intro">
            <strong>§ 165(a)</strong> general rule: losses sustained during taxable year + not
            compensated by insurance. <strong>4 categories:</strong> (1) trade or business losses,
            (2) profit-motive transactions, (3) personal casualty/theft (POST-TCJA: ONLY federally
            declared disasters), (4) wagering losses (limited to wagering gains).
            <strong>Worthless securities:</strong> § 165(g) — capital loss as of last day of year.
            <strong>Theft:</strong> § 165(e) — year of discovery, not year of theft.
            <strong>Ponzi safe harbor:</strong> Rev. Proc. 2009-20 — 75%/95% deduction in year of
            discovery + claim adjustment for recoveries. <strong>Form 4684</strong> reports.
            <strong>Casualty calc:</strong> lesser of (FMV decline) OR (adjusted basis).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s165.h2.inputs">Inputs</h2>
            <form id="s165-form" class="inline-form">
                <label><span data-i18n="view.s165.label.category">Loss category</span>
                    <select name="loss_category">
                        <option value="trade_business" ${state.loss_category === 'trade_business' ? 'selected' : ''}>Trade or business (§ 165(c)(1))</option>
                        <option value="profit_transaction" ${state.loss_category === 'profit_transaction' ? 'selected' : ''}>Profit transaction (§ 165(c)(2))</option>
                        <option value="personal_casualty" ${state.loss_category === 'personal_casualty' ? 'selected' : ''}>Personal casualty (§ 165(c)(3))</option>
                        <option value="wagering" ${state.loss_category === 'wagering' ? 'selected' : ''}>Wagering (§ 165(d))</option>
                        <option value="worthless_security" ${state.loss_category === 'worthless_security' ? 'selected' : ''}>Worthless security (§ 165(g))</option>
                        <option value="theft" ${state.loss_category === 'theft' ? 'selected' : ''}>Theft (§ 165(e))</option>
                        <option value="abandonment" ${state.loss_category === 'abandonment' ? 'selected' : ''}>Abandonment</option>
                        <option value="ponzi" ${state.loss_category === 'ponzi' ? 'selected' : ''}>Ponzi (Rev Proc 2009-20)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s165.label.amount">Loss amount ($)</span>
                    <input type="number" step="0.01" name="loss_amount" value="${state.loss_amount}"></label>
                <label><span data-i18n="view.s165.label.fmv_before">FMV before ($)</span>
                    <input type="number" step="0.01" name="fmv_before" value="${state.fmv_before}"></label>
                <label><span data-i18n="view.s165.label.fmv_after">FMV after ($)</span>
                    <input type="number" step="0.01" name="fmv_after" value="${state.fmv_after}"></label>
                <label><span data-i18n="view.s165.label.basis">Adjusted basis ($)</span>
                    <input type="number" step="0.01" name="adjusted_basis" value="${state.adjusted_basis}"></label>
                <label><span data-i18n="view.s165.label.insurance">Insurance reimbursement ($)</span>
                    <input type="number" step="0.01" name="insurance_reimbursement" value="${state.insurance_reimbursement}"></label>
                <label><span data-i18n="view.s165.label.fdda">Federally declared disaster?</span>
                    <input type="checkbox" name="is_federally_declared_disaster" ${state.is_federally_declared_disaster ? 'checked' : ''}></label>
                <label><span data-i18n="view.s165.label.personal">Personal use?</span>
                    <input type="checkbox" name="is_personal_use" ${state.is_personal_use ? 'checked' : ''}></label>
                <label><span data-i18n="view.s165.label.business">Business use?</span>
                    <input type="checkbox" name="is_business_use" ${state.is_business_use ? 'checked' : ''}></label>
                <label><span data-i18n="view.s165.label.investment">Investment use?</span>
                    <input type="checkbox" name="is_investment_use" ${state.is_investment_use ? 'checked' : ''}></label>
                <label><span data-i18n="view.s165.label.theft">Theft?</span>
                    <input type="checkbox" name="is_theft" ${state.is_theft ? 'checked' : ''}></label>
                <label><span data-i18n="view.s165.label.casualty">Casualty?</span>
                    <input type="checkbox" name="is_casualty" ${state.is_casualty ? 'checked' : ''}></label>
                <label><span data-i18n="view.s165.label.worthless">Worthless security?</span>
                    <input type="checkbox" name="is_worthless_security" ${state.is_worthless_security ? 'checked' : ''}></label>
                <label><span data-i18n="view.s165.label.sec_basis">Security basis ($)</span>
                    <input type="number" step="0.01" name="security_basis" value="${state.security_basis}"></label>
                <label><span data-i18n="view.s165.label.wagering">Wagering loss?</span>
                    <input type="checkbox" name="is_wagering_loss" ${state.is_wagering_loss ? 'checked' : ''}></label>
                <label><span data-i18n="view.s165.label.gains">Wagering gains ($)</span>
                    <input type="number" step="0.01" name="wagering_gains" value="${state.wagering_gains}"></label>
                <label><span data-i18n="view.s165.label.losses">Wagering losses ($)</span>
                    <input type="number" step="0.01" name="wagering_losses" value="${state.wagering_losses}"></label>
                <label><span data-i18n="view.s165.label.abandon">Abandonment?</span>
                    <input type="checkbox" name="is_abandonment" ${state.is_abandonment ? 'checked' : ''}></label>
                <label><span data-i18n="view.s165.label.abandon_basis">Abandonment basis ($)</span>
                    <input type="number" step="0.01" name="abandonment_basis" value="${state.abandonment_basis}"></label>
                <label><span data-i18n="view.s165.label.ponzi">Ponzi?</span>
                    <input type="checkbox" name="is_ponzi" ${state.is_ponzi ? 'checked' : ''}></label>
                <label><span data-i18n="view.s165.label.rp_2009">Rev Proc 2009-20 election?</span>
                    <input type="checkbox" name="rev_proc_2009_20" ${state.rev_proc_2009_20 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s165.label.year">Year</span>
                    <input type="number" step="1" name="year" value="${state.year}"></label>
                <label><span data-i18n="view.s165.label.f4684">Filing Form 4684?</span>
                    <input type="checkbox" name="is_filing_4684" ${state.is_filing_4684 ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s165.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s165-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s165.h2.casualty_calc">Personal casualty calculation</h2>
            <ol class="muted small">
                <li data-i18n="view.s165.calc.lesser">Loss = LESSER of (FMV before − FMV after) OR (adjusted basis)</li>
                <li data-i18n="view.s165.calc.insurance">Subtract insurance reimbursement received</li>
                <li data-i18n="view.s165.calc.100">Apply $100 per-event floor (§ 165(h)(1))</li>
                <li data-i18n="view.s165.calc.10pct">Apply 10% AGI floor (§ 165(h)(2)(A))</li>
                <li data-i18n="view.s165.calc.federally_declared">POST-TCJA: ONLY federally declared disaster (§ 165(h)(5))</li>
                <li data-i18n="view.s165.calc.netting">Net all losses + gains within same disaster</li>
                <li data-i18n="view.s165.calc.special_business">Business casualty: NO $100 floor + NO 10% AGI</li>
                <li data-i18n="view.s165.calc.qualified_disaster_relief">Qualified disaster relief: additional rules per § 139</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s165.h2.ponzi">Ponzi safe harbor (Rev. Proc. 2009-20)</h2>
            <ul class="muted small">
                <li data-i18n="view.s165.ponzi.deduction">75% of qualified investment in discovery year (95% if NO third-party recovery action)</li>
                <li data-i18n="view.s165.ponzi.theft">Treated as § 165(c)(2) theft loss (NOT capital loss)</li>
                <li data-i18n="view.s165.ponzi.year_of_discovery">Year of discovery — not year of investment</li>
                <li data-i18n="view.s165.ponzi.investment_basis">Qualified investment = cash invested − cash + property received − amounts claimed</li>
                <li data-i18n="view.s165.ponzi.claim_adj">Subsequent recoveries: include as ordinary income (claim of right § 1341)</li>
                <li data-i18n="view.s165.ponzi.docs">Documentation: criminal indictment + ponzi scheme classification by IRS</li>
                <li data-i18n="view.s165.ponzi.election">Election made on Form 4684 + Section C (theft loss)</li>
                <li data-i18n="view.s165.ponzi.no_amortization">Above-line deduction — NOT subject to 10% AGI floor + § 67 misc 2% limit</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s165.h2.worthless">Worthless securities (§ 165(g))</h2>
            <ul class="muted small">
                <li data-i18n="view.s165.worth.year">Capital loss as of LAST DAY of taxable year (regardless of when worthless)</li>
                <li data-i18n="view.s165.worth.character">Capital character: LTCG if held > 1 year, STCG if held ≤ 1 year</li>
                <li data-i18n="view.s165.worth.amount">Amount = adjusted basis of security</li>
                <li data-i18n="view.s165.worth.identifiable">Identifiable event required (bankruptcy filing, dissolution, etc.)</li>
                <li data-i18n="view.s165.worth.sol_7y">§ 6511(d) 7-year refund SOL (vs normal 3 years)</li>
                <li data-i18n="view.s165.worth.s165g3">§ 165(g)(3) ordinary loss treatment for affiliated subsidiaries</li>
                <li data-i18n="view.s165.worth.sale">If sold for nominal amount → not worthless, capital loss instead</li>
                <li data-i18n="view.s165.worth.contrast_bad">Contrast § 166 bad debt: § 165(g) for securities + § 166 for non-security debts</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s165.h2.wagering">Wagering losses (§ 165(d))</h2>
            <ul class="muted small">
                <li data-i18n="view.s165.wag.limited">Limited to wagering GAINS (not other income)</li>
                <li data-i18n="view.s165.wag.gambler">Casual gambler: itemized deduction (Schedule A)</li>
                <li data-i18n="view.s165.wag.pro">Professional gambler: above-line Schedule C deduction (Comm. v. Groetzinger, 480 US 23 (1987))</li>
                <li data-i18n="view.s165.wag.expenses">Pro: § 162 ordinary + necessary expenses (travel, lodging, supplies)</li>
                <li data-i18n="view.s165.wag.session">Per-session netting NOT per-bet (Rev. Rul. 2009-22)</li>
                <li data-i18n="view.s165.wag.daily_fantasy">Daily fantasy sports: wagering (PLR 201608039)</li>
                <li data-i18n="view.s165.wag.documentation">Diary requirement: dates, amounts, types, places, witnesses (Rev. Proc. 77-29)</li>
                <li data-i18n="view.s165.wag.illegal">Illegal gambling still deductible (Comm. v. Tellier, 383 US 687 (1966))</li>
            </ul>
        </div>
    `;
    document.getElementById('s165-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.loss_category = fd.get('loss_category');
        state.loss_amount = Number(fd.get('loss_amount')) || 0;
        state.fmv_before = Number(fd.get('fmv_before')) || 0;
        state.fmv_after = Number(fd.get('fmv_after')) || 0;
        state.adjusted_basis = Number(fd.get('adjusted_basis')) || 0;
        state.insurance_reimbursement = Number(fd.get('insurance_reimbursement')) || 0;
        state.is_federally_declared_disaster = !!fd.get('is_federally_declared_disaster');
        state.is_personal_use = !!fd.get('is_personal_use');
        state.is_business_use = !!fd.get('is_business_use');
        state.is_investment_use = !!fd.get('is_investment_use');
        state.is_theft = !!fd.get('is_theft');
        state.is_casualty = !!fd.get('is_casualty');
        state.is_worthless_security = !!fd.get('is_worthless_security');
        state.security_basis = Number(fd.get('security_basis')) || 0;
        state.is_wagering_loss = !!fd.get('is_wagering_loss');
        state.wagering_gains = Number(fd.get('wagering_gains')) || 0;
        state.wagering_losses = Number(fd.get('wagering_losses')) || 0;
        state.is_abandonment = !!fd.get('is_abandonment');
        state.abandonment_basis = Number(fd.get('abandonment_basis')) || 0;
        state.is_ponzi = !!fd.get('is_ponzi');
        state.rev_proc_2009_20 = !!fd.get('rev_proc_2009_20');
        state.year = Number(fd.get('year')) || 0;
        state.is_filing_4684 = !!fd.get('is_filing_4684');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s165-output');
    if (!el) return;
    const fmv_decline = state.fmv_before - state.fmv_after;
    const lesser = Math.min(fmv_decline, state.adjusted_basis);
    const after_insurance = Math.max(0, lesser - state.insurance_reimbursement);
    const wager_allowed = Math.min(state.wagering_losses, state.wagering_gains);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s165.h2.result">§ 165 loss assessment</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s165.card.category">Category</div><div class="value">${esc(state.loss_category)}</div></div>
                <div class="card"><div class="label" data-i18n="view.s165.card.fmv_decline">FMV decline</div><div class="value">$${fmv_decline.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s165.card.lesser">Lesser of FMV / basis</div><div class="value">$${lesser.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s165.card.after_insurance">After insurance</div><div class="value">$${after_insurance.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s165.card.wager">Wagering allowed</div><div class="value">$${wager_allowed.toLocaleString()}</div></div>
            </div>
        </div>
    `;
}
