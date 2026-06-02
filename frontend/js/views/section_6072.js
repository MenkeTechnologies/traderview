// IRC § 6072 — Time for Filing Income Tax Returns.
// Individual (calendar year): April 15.
// C-corp: 15th day of 4th month after year-end (April 15 for calendar).
// Partnership + S-corp: 15th day of 3rd month after year-end (March 15 for calendar).
// Extension: 6 months for individual / corp; 6 months for partnership + S-corp (no auto extension to pay).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    entity_type: 'individual',
    year_end_month: 12,
    extension_filed: false,
    extension_type: 'automatic_6month',
    payment_made_by_original: false,
    return_filed_date: '',
    is_combat_zone: false,
    is_disaster_area: false,
    is_living_abroad: false,
    fiscal_year_psc: false,
    short_period_return: false,
    decedent_date: '',
    estimated_tax_due: 0,
    payment_made: 0,
    has_estimated_safe_harbor: false,
};

export async function renderSection6072(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6072.h1.title">// § 6072 RETURN DUE DATES</span></h1>
        <p class="muted small" data-i18n="view.s6072.hint.intro">
            <strong>Individual:</strong> April 15 (calendar). <strong>C-corp:</strong> 15th day of 4th month
            after year-end. <strong>Partnership + S-corp:</strong> 15th day of <strong>3rd month</strong>
            after year-end (March 15 calendar). <strong>Trust / estate:</strong> 15th day of 4th month
            (April 15 calendar). <strong>Extension:</strong> 6-month automatic via Form 4868 (individual) /
            Form 7004 (entities). <strong>Extension to FILE — NOT to PAY.</strong> <strong>Combat zone:</strong>
            6+ months. <strong>Abroad:</strong> June 15 automatic for individuals.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6072.h2.inputs">Inputs</h2>
            <form id="s6072-form" class="inline-form">
                <label><span data-i18n="view.s6072.label.entity">Entity type</span>
                    <select name="entity_type">
                        <option value="individual" ${state.entity_type === 'individual' ? 'selected' : ''}>Individual (Form 1040)</option>
                        <option value="c_corp" ${state.entity_type === 'c_corp' ? 'selected' : ''}>C-Corp (Form 1120)</option>
                        <option value="s_corp" ${state.entity_type === 's_corp' ? 'selected' : ''}>S-Corp (Form 1120-S)</option>
                        <option value="partnership" ${state.entity_type === 'partnership' ? 'selected' : ''}>Partnership (Form 1065)</option>
                        <option value="trust" ${state.entity_type === 'trust' ? 'selected' : ''}>Trust (Form 1041)</option>
                        <option value="estate" ${state.entity_type === 'estate' ? 'selected' : ''}>Estate (Form 1041)</option>
                        <option value="exempt" ${state.entity_type === 'exempt' ? 'selected' : ''}>Tax-Exempt (Form 990)</option>
                        <option value="psc_fiscal" ${state.entity_type === 'psc_fiscal' ? 'selected' : ''}>PSC fiscal yr</option>
                        <option value="nonresident_alien" ${state.entity_type === 'nonresident_alien' ? 'selected' : ''}>NRA (Form 1040-NR)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6072.label.year_end">Year-end month (1-12)</span>
                    <input type="number" step="1" name="year_end_month" value="${state.year_end_month}"></label>
                <label><span data-i18n="view.s6072.label.extension">Extension filed?</span>
                    <input type="checkbox" name="extension_filed" ${state.extension_filed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6072.label.extension_type">Extension type</span>
                    <select name="extension_type">
                        <option value="automatic_6month" ${state.extension_type === 'automatic_6month' ? 'selected' : ''}>Automatic 6 month</option>
                        <option value="abroad_4month" ${state.extension_type === 'abroad_4month' ? 'selected' : ''}>Abroad 4 month (additional)</option>
                        <option value="combat_180day" ${state.extension_type === 'combat_180day' ? 'selected' : ''}>Combat zone + 180 days</option>
                        <option value="disaster_relief" ${state.extension_type === 'disaster_relief' ? 'selected' : ''}>Disaster relief</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6072.label.paid">Estimated payment made by original?</span>
                    <input type="checkbox" name="payment_made_by_original" ${state.payment_made_by_original ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6072.label.filed">Return filed date</span>
                    <input type="date" name="return_filed_date" value="${state.return_filed_date}"></label>
                <label><span data-i18n="view.s6072.label.combat">Combat zone?</span>
                    <input type="checkbox" name="is_combat_zone" ${state.is_combat_zone ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6072.label.disaster">Federally declared disaster?</span>
                    <input type="checkbox" name="is_disaster_area" ${state.is_disaster_area ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6072.label.abroad">Living abroad?</span>
                    <input type="checkbox" name="is_living_abroad" ${state.is_living_abroad ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6072.label.psc_fiscal">PSC fiscal year?</span>
                    <input type="checkbox" name="fiscal_year_psc" ${state.fiscal_year_psc ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6072.label.short">Short period return?</span>
                    <input type="checkbox" name="short_period_return" ${state.short_period_return ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6072.label.decedent">Decedent date</span>
                    <input type="date" name="decedent_date" value="${state.decedent_date}"></label>
                <label><span data-i18n="view.s6072.label.tax_due">Estimated tax due ($)</span>
                    <input type="number" step="1000" name="estimated_tax_due" value="${state.estimated_tax_due}"></label>
                <label><span data-i18n="view.s6072.label.paid_amount">Payment made ($)</span>
                    <input type="number" step="1000" name="payment_made" value="${state.payment_made}"></label>
                <label><span data-i18n="view.s6072.label.safe_harbor">Estimated tax safe harbor met?</span>
                    <input type="checkbox" name="has_estimated_safe_harbor" ${state.has_estimated_safe_harbor ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s6072.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6072-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6072.h2.due_dates">Due dates by entity type</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s6072.th.entity">Entity</th>
                    <th data-i18n="view.s6072.th.form">Form</th>
                    <th data-i18n="view.s6072.th.due">Due date (calendar)</th>
                    <th data-i18n="view.s6072.th.ext">Extension to</th>
                </tr></thead>
                <tbody>
                    <tr><td>Individual</td><td>1040</td><td>April 15</td><td>October 15 (Form 4868)</td></tr>
                    <tr><td>NRA</td><td>1040-NR</td><td>April 15 (June 15 abroad)</td><td>October 15 (Form 4868)</td></tr>
                    <tr><td>Decedent</td><td>1040</td><td>April 15 of year following death</td><td>October 15</td></tr>
                    <tr><td>C-Corp</td><td>1120</td><td>April 15 (4th month)</td><td>October 15 (Form 7004)</td></tr>
                    <tr><td>S-Corp</td><td>1120-S</td><td>March 15 (3rd month)</td><td>September 15 (Form 7004)</td></tr>
                    <tr><td>Partnership</td><td>1065</td><td>March 15 (3rd month)</td><td>September 15 (Form 7004)</td></tr>
                    <tr><td>Trust / Estate</td><td>1041</td><td>April 15</td><td>September 30 (Form 7004 — 5.5 month)</td></tr>
                    <tr><td>501(c)</td><td>990</td><td>May 15 (5th month)</td><td>November 15</td></tr>
                    <tr><td>PSC fiscal</td><td>1120 / 1120-S</td><td>15th of 3rd or 4th month after year-end</td><td>+6 month</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6072.h2.extension">Extension mechanics</h2>
            <ul class="muted small">
                <li data-i18n="view.s6072.ext.individual">Individual: Form 4868 by April 15 → automatic 6-month extension to Oct 15</li>
                <li data-i18n="view.s6072.ext.entity">Entity: Form 7004 by original due date → automatic 6-month (5.5 for trust)</li>
                <li data-i18n="view.s6072.ext.pay_required">CRITICAL: extension to FILE only, NOT to pay — interest + penalty start from original due date</li>
                <li data-i18n="view.s6072.ext.abroad">Abroad: automatic 2-month extension to June 15 — file by then OR file 4868 for additional 4 months</li>
                <li data-i18n="view.s6072.ext.combat">Combat zone: extension to 180 days after leaving zone</li>
                <li data-i18n="view.s6072.ext.disaster">Federally declared disaster: IRS announces specific extended dates per region</li>
                <li data-i18n="view.s6072.ext.psc">PSC: 6-month extension to file 1120 OR 1120-S</li>
                <li data-i18n="view.s6072.ext.no_combo">Extension does NOT extend SE tax payment due date</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6072.h2.late_penalties">Late filing + payment penalties</h2>
            <ul class="muted small">
                <li data-i18n="view.s6072.late.6651a1">§ 6651(a)(1) failure to file: 5% / month up to 25% of unpaid tax</li>
                <li data-i18n="view.s6072.late.6651a2">§ 6651(a)(2) failure to pay: 0.5% / month up to 25% of unpaid tax</li>
                <li data-i18n="view.s6072.late.combined">Combined cap: 47.5% (file + pay)</li>
                <li data-i18n="view.s6072.late.minimum">Minimum failure-to-file penalty: smaller of $485 (2024) or 100% of unpaid tax</li>
                <li data-i18n="view.s6072.late.interest">Interest at federal short-term rate + 3% (compounded daily)</li>
                <li data-i18n="view.s6072.late.reasonable_cause">Reasonable cause exception: Boyle case very strict (limited to extraordinary circumstances)</li>
                <li data-i18n="view.s6072.late.first_time">First Time Penalty Abatement (FTA): one-time relief if compliant 3 prior yrs</li>
                <li data-i18n="view.s6072.late.collection_alternative">Collection alternatives: installment agreement, OIC, CNC</li>
            </ul>
        </div>
    `;
    document.getElementById('s6072-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.entity_type = fd.get('entity_type');
        state.year_end_month = Number(fd.get('year_end_month')) || 0;
        state.extension_filed = !!fd.get('extension_filed');
        state.extension_type = fd.get('extension_type');
        state.payment_made_by_original = !!fd.get('payment_made_by_original');
        state.return_filed_date = fd.get('return_filed_date');
        state.is_combat_zone = !!fd.get('is_combat_zone');
        state.is_disaster_area = !!fd.get('is_disaster_area');
        state.is_living_abroad = !!fd.get('is_living_abroad');
        state.fiscal_year_psc = !!fd.get('fiscal_year_psc');
        state.short_period_return = !!fd.get('short_period_return');
        state.decedent_date = fd.get('decedent_date');
        state.estimated_tax_due = Number(fd.get('estimated_tax_due')) || 0;
        state.payment_made = Number(fd.get('payment_made')) || 0;
        state.has_estimated_safe_harbor = !!fd.get('has_estimated_safe_harbor');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6072-output');
    if (!el) return;
    let original_months_after_ye = 4;
    if (state.entity_type === 's_corp' || state.entity_type === 'partnership') original_months_after_ye = 3;
    if (state.entity_type === 'exempt') original_months_after_ye = 5;
    const original_due_month = ((state.year_end_month + original_months_after_ye - 1) % 12) + 1;
    const extension_months = state.extension_filed ? 6 : 0;
    const final_due_month = ((original_due_month + extension_months - 1) % 12) + 1;
    const unpaid = Math.max(0, state.estimated_tax_due - state.payment_made);
    const failure_to_file_eligible = !state.extension_filed && state.return_filed_date !== '';
    const failure_to_pay = unpaid > 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6072.h2.result">§ 6072 outcome</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s6072.card.original_due">Original due (month)</div>
                    <div class="value">${original_due_month}/15</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6072.card.extended">Extended due (if 7004)</div>
                    <div class="value">${final_due_month}/15</div>
                </div>
                <div class="card ${unpaid > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s6072.card.unpaid">Unpaid balance</div>
                    <div class="value">$${unpaid.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${failure_to_file_eligible ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s6072.card.ftf">Failure to file risk?</div>
                    <div class="value">${failure_to_file_eligible ? esc(t('view.s6072.status.yes')) : esc(t('view.s6072.status.no'))}</div>
                </div>
                <div class="card ${failure_to_pay ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s6072.card.ftp">Failure to pay risk?</div>
                    <div class="value">${failure_to_pay ? esc(t('view.s6072.status.yes')) : esc(t('view.s6072.status.no'))}</div>
                </div>
                <div class="card ${state.is_combat_zone || state.is_disaster_area ? 'pos' : ''}">
                    <div class="label" data-i18n="view.s6072.card.special_relief">Combat / disaster relief?</div>
                    <div class="value">${state.is_combat_zone || state.is_disaster_area ? esc(t('view.s6072.status.yes')) : esc(t('view.s6072.status.no'))}</div>
                </div>
            </div>
            ${unpaid > 0 && !state.extension_filed ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s6072.unpaid_note">
                    Unpaid balance + no extension: failure-to-file PLUS failure-to-pay penalties accrue
                    from original due date. Combined penalty up to 47.5%. File ASAP even if can't pay.
                    Form 9465 installment agreement available. First Time Penalty Abatement may waive
                    if compliant 3 prior years.
                </p>
            ` : ''}
        </div>
    `;
}
