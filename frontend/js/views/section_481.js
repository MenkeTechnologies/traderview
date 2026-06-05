// IRC § 481 — Adjustments Required by Changes in Method of Accounting.
// Single-year catch-up adjustment when method changes from previously used.
// § 481(a) adjustment = cumulative difference between current method and new method.
// Spread: 4 years for unfavorable (taxpayer to IRS) / 1 year for favorable (IRS to taxpayer).

import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    accounting_method_old: 'cash',
    accounting_method_new: 'accrual',
    is_automatic_change: false,
    is_non_automatic_change: false,
    s481_a_adjustment: 0,
    is_favorable_adjustment: false,
    spread_period_years: 4,
    current_year_adjustment: 0,
    taxpayer_initiated: false,
    irs_initiated: false,
    rev_proc_2022_14_automatic: false,
    form_3115_filed: false,
    short_form_3115: false,
    f3115_due_with_return: false,
    days_late_filing: 0,
    consent_required: false,
    cuts_taxpayer_payment_due: 0,
    s481_b_3_year_avg_election: false,
    s481_b_relief_for_high_income: false,
    accounting_method_principle: 'cash',
    s162_clear_reflection: false,
    consistent_application_used: false,
    multiple_year_inconsistency: false,
    s482_related_party_method: false,
    s263a_uniform_capitalization: false,
    s451_b_income_inclusion: false,
    s461_h_economic_performance: false,
    s471_inventory_method: false,
    s263a_full_absorption: false,
    overall_change_principal_method: false,
    item_specific_change: false,
    duplication_omission_correction: false,
    cumulative_basis_difference: 0,
};

export async function renderSection481(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s481.h1.title">// § 481 ACCOUNTING METHOD CHANGE ADJUSTMENT</span></h1>
        <p class="muted small" data-i18n="view.s481.hint.intro">
            <strong>§ 481(a)</strong> requires single-year CUMULATIVE catch-up when accounting method
            changes. <strong>Computed as</strong> cumulative income/deduction difference between
            current method + new method applied retroactively (back to method origination).
            <strong>Spread:</strong> UNFAVORABLE (increase in income) = 4 YEARS; FAVORABLE (decrease)
            = 1 YEAR. <strong>Automatic change procedures:</strong> Rev. Proc. 2022-14 (current) +
            successive — Form 3115 filed with return for first year. <strong>Non-automatic:</strong>
            Form 3115 filed during year + IRS consent required + user fee. <strong>§ 481(b) relief:</strong>
            (1) 3-year average election to limit tax increase, (2) limitation for high-income years.
            <strong>Common changes:</strong> cash ↔ accrual, § 263A UNICAP, § 451(b) AFS income
            inclusion, § 461(h) economic performance, inventory valuation, depreciation methods.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s481.h2.inputs">Inputs</h2>
            <form id="s481-form" class="inline-form">
                <label><span data-i18n="view.s481.label.old">Old method</span>
                    <select name="accounting_method_old">
                        <option value="cash" ${state.accounting_method_old === 'cash' ? 'selected' : ''}>Cash</option>
                        <option value="accrual" ${state.accounting_method_old === 'accrual' ? 'selected' : ''}>Accrual</option>
                        <option value="hybrid" ${state.accounting_method_old === 'hybrid' ? 'selected' : ''}>Hybrid</option>
                        <option value="installment" ${state.accounting_method_old === 'installment' ? 'selected' : ''}>Installment (§ 453)</option>
                        <option value="completed_contract" ${state.accounting_method_old === 'completed_contract' ? 'selected' : ''}>Completed contract</option>
                        <option value="percentage_completion" ${state.accounting_method_old === 'percentage_completion' ? 'selected' : ''}>Percentage of completion</option>
                    </select>
                </label>
                <label><span data-i18n="view.s481.label.new">New method</span>
                    <select name="accounting_method_new">
                        <option value="cash" ${state.accounting_method_new === 'cash' ? 'selected' : ''}>Cash</option>
                        <option value="accrual" ${state.accounting_method_new === 'accrual' ? 'selected' : ''}>Accrual</option>
                        <option value="hybrid" ${state.accounting_method_new === 'hybrid' ? 'selected' : ''}>Hybrid</option>
                        <option value="installment" ${state.accounting_method_new === 'installment' ? 'selected' : ''}>Installment</option>
                        <option value="completed_contract" ${state.accounting_method_new === 'completed_contract' ? 'selected' : ''}>Completed contract</option>
                        <option value="percentage_completion" ${state.accounting_method_new === 'percentage_completion' ? 'selected' : ''}>Percentage of completion</option>
                    </select>
                </label>
                <label><span data-i18n="view.s481.label.automatic">Automatic change?</span>
                    <input type="checkbox" name="is_automatic_change" ${state.is_automatic_change ? 'checked' : ''}></label>
                <label><span data-i18n="view.s481.label.non_auto">Non-automatic?</span>
                    <input type="checkbox" name="is_non_automatic_change" ${state.is_non_automatic_change ? 'checked' : ''}></label>
                <label><span data-i18n="view.s481.label.adjustment">§ 481(a) adjustment ($)</span>
                    <input type="number" step="0.01" name="s481_a_adjustment" value="${state.s481_a_adjustment}"></label>
                <label><span data-i18n="view.s481.label.favorable">Favorable?</span>
                    <input type="checkbox" name="is_favorable_adjustment" ${state.is_favorable_adjustment ? 'checked' : ''}></label>
                <label><span data-i18n="view.s481.label.spread">Spread years</span>
                    <input type="number" step="1" name="spread_period_years" value="${state.spread_period_years}"></label>
                <label><span data-i18n="view.s481.label.current">Current year ($)</span>
                    <input type="number" step="0.01" name="current_year_adjustment" value="${state.current_year_adjustment}"></label>
                <label><span data-i18n="view.s481.label.tp_init">Taxpayer initiated?</span>
                    <input type="checkbox" name="taxpayer_initiated" ${state.taxpayer_initiated ? 'checked' : ''}></label>
                <label><span data-i18n="view.s481.label.irs_init">IRS initiated?</span>
                    <input type="checkbox" name="irs_initiated" ${state.irs_initiated ? 'checked' : ''}></label>
                <label><span data-i18n="view.s481.label.rev_proc">Rev Proc 2022-14?</span>
                    <input type="checkbox" name="rev_proc_2022_14_automatic" ${state.rev_proc_2022_14_automatic ? 'checked' : ''}></label>
                <label><span data-i18n="view.s481.label.3115">Form 3115 filed?</span>
                    <input type="checkbox" name="form_3115_filed" ${state.form_3115_filed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s481.label.short_form">Short Form 3115?</span>
                    <input type="checkbox" name="short_form_3115" ${state.short_form_3115 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s481.label.due_return">Due with return?</span>
                    <input type="checkbox" name="f3115_due_with_return" ${state.f3115_due_with_return ? 'checked' : ''}></label>
                <label><span data-i18n="view.s481.label.late">Days late filing</span>
                    <input type="number" step="1" name="days_late_filing" value="${state.days_late_filing}"></label>
                <label><span data-i18n="view.s481.label.consent">Consent required?</span>
                    <input type="checkbox" name="consent_required" ${state.consent_required ? 'checked' : ''}></label>
                <label><span data-i18n="view.s481.label.cuts_pmt">CUTS payment ($)</span>
                    <input type="number" step="0.01" name="cuts_taxpayer_payment_due" value="${state.cuts_taxpayer_payment_due}"></label>
                <label><span data-i18n="view.s481.label.s481b">§ 481(b) 3-yr avg?</span>
                    <input type="checkbox" name="s481_b_3_year_avg_election" ${state.s481_b_3_year_avg_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s481.label.s481b_relief">§ 481(b) relief?</span>
                    <input type="checkbox" name="s481_b_relief_for_high_income" ${state.s481_b_relief_for_high_income ? 'checked' : ''}></label>
                <label><span data-i18n="view.s481.label.principle">Accounting principle</span>
                    <select name="accounting_method_principle">
                        <option value="cash" ${state.accounting_method_principle === 'cash' ? 'selected' : ''}>Cash</option>
                        <option value="accrual" ${state.accounting_method_principle === 'accrual' ? 'selected' : ''}>Accrual</option>
                        <option value="hybrid" ${state.accounting_method_principle === 'hybrid' ? 'selected' : ''}>Hybrid</option>
                    </select>
                </label>
                <label><span data-i18n="view.s481.label.clear">§ 446 clear reflection?</span>
                    <input type="checkbox" name="s162_clear_reflection" ${state.s162_clear_reflection ? 'checked' : ''}></label>
                <label><span data-i18n="view.s481.label.consistent">Consistent application?</span>
                    <input type="checkbox" name="consistent_application_used" ${state.consistent_application_used ? 'checked' : ''}></label>
                <label><span data-i18n="view.s481.label.multi_year">Multi-year inconsistent?</span>
                    <input type="checkbox" name="multiple_year_inconsistency" ${state.multiple_year_inconsistency ? 'checked' : ''}></label>
                <label><span data-i18n="view.s481.label.s482">§ 482 related party?</span>
                    <input type="checkbox" name="s482_related_party_method" ${state.s482_related_party_method ? 'checked' : ''}></label>
                <label><span data-i18n="view.s481.label.s263a">§ 263A UNICAP?</span>
                    <input type="checkbox" name="s263a_uniform_capitalization" ${state.s263a_uniform_capitalization ? 'checked' : ''}></label>
                <label><span data-i18n="view.s481.label.s451b">§ 451(b)?</span>
                    <input type="checkbox" name="s451_b_income_inclusion" ${state.s451_b_income_inclusion ? 'checked' : ''}></label>
                <label><span data-i18n="view.s481.label.s461h">§ 461(h) econ perf?</span>
                    <input type="checkbox" name="s461_h_economic_performance" ${state.s461_h_economic_performance ? 'checked' : ''}></label>
                <label><span data-i18n="view.s481.label.s471">§ 471 inventory?</span>
                    <input type="checkbox" name="s471_inventory_method" ${state.s471_inventory_method ? 'checked' : ''}></label>
                <label><span data-i18n="view.s481.label.s263a_full">§ 263A full absorption?</span>
                    <input type="checkbox" name="s263a_full_absorption" ${state.s263a_full_absorption ? 'checked' : ''}></label>
                <label><span data-i18n="view.s481.label.principal">Overall principal change?</span>
                    <input type="checkbox" name="overall_change_principal_method" ${state.overall_change_principal_method ? 'checked' : ''}></label>
                <label><span data-i18n="view.s481.label.item_specific">Item-specific?</span>
                    <input type="checkbox" name="item_specific_change" ${state.item_specific_change ? 'checked' : ''}></label>
                <label><span data-i18n="view.s481.label.dup_omit">Duplication/omission?</span>
                    <input type="checkbox" name="duplication_omission_correction" ${state.duplication_omission_correction ? 'checked' : ''}></label>
                <label><span data-i18n="view.s481.label.cumulative">Cumulative basis diff ($)</span>
                    <input type="number" step="0.01" name="cumulative_basis_difference" value="${state.cumulative_basis_difference}"></label>
                <button class="primary" type="submit" data-i18n="view.s481.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s481-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s481.h2.spread_rules">Spread rules</h2>
            <ul class="muted small">
                <li data-i18n="view.s481.spread.unfavorable">UNFAVORABLE (increase income): 4-year ratable spread</li>
                <li data-i18n="view.s481.spread.favorable">FAVORABLE (decrease income): 1-year (full in year of change)</li>
                <li data-i18n="view.s481.spread.small_dollar">$50K or less unfavorable: ANNUAL election for 1-year (taxpayer's choice)</li>
                <li data-i18n="view.s481.spread.cessation">If business ceases / no longer using method: remaining § 481(a) accelerated</li>
                <li data-i18n="view.s481.spread.s481_b">§ 481(b) relief: 3-yr average tax computation limits tax increase</li>
                <li data-i18n="view.s481.spread.s481_b_election">Election: tax = 3-yr avg × 3 OR 1-yr × current rate — taxpayer chooses lower</li>
                <li data-i18n="view.s481.spread.audit_initiated">IRS-initiated change: typically 1-year for unfavorable (no taxpayer relief)</li>
                <li data-i18n="view.s481.spread.taxpayer_initiated">Taxpayer-initiated: 4-year unfavorable + various relief</li>
                <li data-i18n="view.s481.spread.s481_a_post_tcja">Post-TCJA: § 451(b) AFS inclusion conformity — major mass change</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s481.h2.examples">Common method changes</h2>
            <ul class="muted small">
                <li data-i18n="view.s481.ex.cash_accrual">Cash → Accrual (small business hit $30M GR threshold)</li>
                <li data-i18n="view.s481.ex.accrual_cash">Accrual → Cash (small business eligible post-TCJA)</li>
                <li data-i18n="view.s481.ex.s263a">UNICAP § 263A — start or stop applying</li>
                <li data-i18n="view.s481.ex.s451_b">§ 451(b) AFS income inclusion (post-TCJA)</li>
                <li data-i18n="view.s481.ex.s263A_safe_harbor">§ 263A simplified resale method election</li>
                <li data-i18n="view.s481.ex.depreciation">Depreciation method (incl bonus depreciation election out)</li>
                <li data-i18n="view.s481.ex.inventory">Inventory: FIFO vs LIFO vs average vs lower of cost or market</li>
                <li data-i18n="view.s481.ex.advance_payment">Advance payment recognition (§ 451(c) deferral)</li>
                <li data-i18n="view.s481.ex.deferred_revenue">Deferred revenue recognition</li>
                <li data-i18n="view.s481.ex.installment">Installment method election (§ 453)</li>
                <li data-i18n="view.s481.ex.completed_contract">Completed contract → percentage of completion (long-term contracts)</li>
                <li data-i18n="view.s481.ex.s174_rd">§ 174 R&D capitalization (mandatory post-2022)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s481.h2.process">Form 3115 process</h2>
            <ol class="muted small">
                <li data-i18n="view.s481.proc.identify">Identify if method change needed (Rev. Proc. 2022-14 etc. lists automatic changes)</li>
                <li data-i18n="view.s481.proc.automatic">Automatic: Form 3115 with return, no IRS consent, no user fee, late filing relief available</li>
                <li data-i18n="view.s481.proc.non_automatic">Non-automatic: Form 3115 within year of change, IRS consent + $9,500-$23,000 user fee</li>
                <li data-i18n="view.s481.proc.compute">Compute § 481(a) adjustment (cumulative method-change effect)</li>
                <li data-i18n="view.s481.proc.spread">Apply spread (4-yr unfavorable / 1-yr favorable)</li>
                <li data-i18n="view.s481.proc.attach">Attach Form 3115 to first year's return (signed schedule + supporting computation)</li>
                <li data-i18n="view.s481.proc.audit_consequences">If IRS-audit-initiated: may treat as taxpayer-initiated under Rev. Proc. 2002-18</li>
                <li data-i18n="view.s481.proc.duplicate">If duplicate or omitted: § 481(a) catches up — no penalty if voluntary correction</li>
                <li data-i18n="view.s481.proc.s481_d">§ 481(d) recovery of items omitted under prior method - special rules</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s481.h2.consistency">§ 446(e) — consent + change rules</h2>
            <ul class="muted small">
                <li data-i18n="view.s481.s446.consent">§ 446(e) requires consent of Secretary for method change</li>
                <li data-i18n="view.s481.s446.clear_reflection">§ 446(b) — method must clearly reflect income</li>
                <li data-i18n="view.s481.s446.consistent">Consistent year-over-year application required</li>
                <li data-i18n="view.s481.s446.first_year">First year using method: NOT a method change (no § 481(a))</li>
                <li data-i18n="view.s481.s446.correction_of_error">Correction of error in prior method NOT subject to consent (revocation possible)</li>
                <li data-i18n="view.s481.s446.s481_safe">Compliance: filing Form 3115 = "filed return" with method</li>
                <li data-i18n="view.s481.s446.s481_aceflyer">Schedule M-3 / M-1 reconciliation may reveal book/tax difference triggering change</li>
                <li data-i18n="view.s481.s446.s481_e">§ 481(e) — IRS may use deemed-consent approach</li>
                <li data-i18n="view.s481.s446.short_form">Short Form 3115: certain automatic changes (less detailed)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s481.h2.related">Related provisions</h2>
            <ul class="muted small">
                <li data-i18n="view.s481.rel.s446">§ 446 — General accounting method requirements</li>
                <li data-i18n="view.s481.rel.s263A">§ 263A — UNICAP (frequent target of § 481)</li>
                <li data-i18n="view.s481.rel.s451_b">§ 451(b) — AFS income inclusion (TCJA)</li>
                <li data-i18n="view.s481.rel.s461_h">§ 461(h) — Economic performance</li>
                <li data-i18n="view.s481.rel.s471">§ 471 — Inventory method</li>
                <li data-i18n="view.s481.rel.s263A_simplified">§ 263A simplified method elections</li>
                <li data-i18n="view.s481.rel.s162">§ 162 — Trade or business expenses</li>
                <li data-i18n="view.s481.rel.s174">§ 174 — R&D capitalization (mandatory post-2022)</li>
                <li data-i18n="view.s481.rel.s481_d">§ 481(d) — recovery of omitted items</li>
                <li data-i18n="view.s481.rel.s9100">§ 9100 relief — automatic 12-month extension for late Form 3115</li>
                <li data-i18n="view.s481.rel.s6662">§ 6662 accuracy-related penalty — may apply if § 481 not properly reported</li>
            </ul>
        </div>
    `;
    document.getElementById('s481-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.accounting_method_old = fd.get('accounting_method_old');
        state.accounting_method_new = fd.get('accounting_method_new');
        state.is_automatic_change = !!fd.get('is_automatic_change');
        state.is_non_automatic_change = !!fd.get('is_non_automatic_change');
        state.s481_a_adjustment = Number(fd.get('s481_a_adjustment')) || 0;
        state.is_favorable_adjustment = !!fd.get('is_favorable_adjustment');
        state.spread_period_years = Number(fd.get('spread_period_years')) || 0;
        state.current_year_adjustment = Number(fd.get('current_year_adjustment')) || 0;
        state.taxpayer_initiated = !!fd.get('taxpayer_initiated');
        state.irs_initiated = !!fd.get('irs_initiated');
        state.rev_proc_2022_14_automatic = !!fd.get('rev_proc_2022_14_automatic');
        state.form_3115_filed = !!fd.get('form_3115_filed');
        state.short_form_3115 = !!fd.get('short_form_3115');
        state.f3115_due_with_return = !!fd.get('f3115_due_with_return');
        state.days_late_filing = Number(fd.get('days_late_filing')) || 0;
        state.consent_required = !!fd.get('consent_required');
        state.cuts_taxpayer_payment_due = Number(fd.get('cuts_taxpayer_payment_due')) || 0;
        state.s481_b_3_year_avg_election = !!fd.get('s481_b_3_year_avg_election');
        state.s481_b_relief_for_high_income = !!fd.get('s481_b_relief_for_high_income');
        state.accounting_method_principle = fd.get('accounting_method_principle');
        state.s162_clear_reflection = !!fd.get('s162_clear_reflection');
        state.consistent_application_used = !!fd.get('consistent_application_used');
        state.multiple_year_inconsistency = !!fd.get('multiple_year_inconsistency');
        state.s482_related_party_method = !!fd.get('s482_related_party_method');
        state.s263a_uniform_capitalization = !!fd.get('s263a_uniform_capitalization');
        state.s451_b_income_inclusion = !!fd.get('s451_b_income_inclusion');
        state.s461_h_economic_performance = !!fd.get('s461_h_economic_performance');
        state.s471_inventory_method = !!fd.get('s471_inventory_method');
        state.s263a_full_absorption = !!fd.get('s263a_full_absorption');
        state.overall_change_principal_method = !!fd.get('overall_change_principal_method');
        state.item_specific_change = !!fd.get('item_specific_change');
        state.duplication_omission_correction = !!fd.get('duplication_omission_correction');
        state.cumulative_basis_difference = Number(fd.get('cumulative_basis_difference')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s481-output');
    if (!el) return;
    const spread = state.is_favorable_adjustment ? 1 : 4;
    const per_year = state.s481_a_adjustment / spread;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s481.h2.result">§ 481(a) adjustment</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s481.card.adj">Total adjustment</div><div class="value">$${state.s481_a_adjustment.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s481.card.spread">Spread (years)</div><div class="value">${spread}</div></div>
                <div class="card ${state.is_favorable_adjustment ? 'pos' : 'neg'}"><div class="label" data-i18n="view.s481.card.character">Character</div><div class="value">${state.is_favorable_adjustment ? 'FAVORABLE (1-yr)' : 'UNFAVORABLE (4-yr)'}</div></div>
                <div class="card"><div class="label" data-i18n="view.s481.card.per_year">Per year</div><div class="value">$${per_year.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div></div>
            </div>
        </div>
    `;
}
