// IRC § 409A — Nonqualified Deferred Compensation Rules.
// Defers tax on comp not actually/constructively received; if violates § 409A, ALL deferred amounts taxed + 20% additional tax + interest at underpayment rate + 1%.
// Six permitted distribution events: separation, disability, death, time/schedule, change in control, unforeseeable emergency.
// "Initial" deferral election: 30 days post-eligibility OR before year start; performance-based extra 6 months.
// Subsequent deferrals: 1-yr advance + 5-yr push-out.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    deferred_amount: 0,
    distribution_event: 'separation',
    initial_election_timely: true,
    subsequent_deferral_timely: true,
    five_year_push_out: false,
    is_short_term_deferral: false,
    is_separation_pay_arrangement: false,
    arrangement_type: 'salary_continuation',
    is_specified_employee: false,
    violation_year_tax: 0,
    market_rate_interest: 4.0,
    is_independent_contractor: false,
    is_tax_qualified_plan: false,
};

export async function renderSection409A(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s409A.h1.title">// § 409A NONQUAL DEFERRED COMP</span></h1>
        <p class="muted small" data-i18n="view.s409A.hint.intro">
            Defers tax on comp not actually / constructively received. <strong>VIOLATION:</strong> ALL deferred
            amounts taxed in current year + <strong>20% additional tax</strong> + underpayment interest at
            <strong>1% above market rate</strong>. <strong>6 permitted distribution events:</strong> separation,
            disability, death, time/schedule, change in control, unforeseeable emergency. <strong>Initial
            election:</strong> 30 days post-eligibility OR before year start. <strong>Subsequent deferrals:</strong>
            1-yr advance + 5-yr push-out. <strong>Specified employee (public co):</strong> 6-month delay on
            separation pay. Form W-2 box 12 code Y/Z.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s409A.h2.inputs">Inputs</h2>
            <form id="s409A-form" class="inline-form">
                <label><span data-i18n="view.s409A.label.deferred">Total deferred amount ($)</span>
                    <input type="number" step="0.01" name="deferred_amount" value="${state.deferred_amount}"></label>
                <label><span data-i18n="view.s409A.label.event">Distribution event</span>
                    <select name="distribution_event">
                        <option value="separation" ${state.distribution_event === 'separation' ? 'selected' : ''}>Separation from service</option>
                        <option value="disability" ${state.distribution_event === 'disability' ? 'selected' : ''}>Disability</option>
                        <option value="death" ${state.distribution_event === 'death' ? 'selected' : ''}>Death</option>
                        <option value="time_schedule" ${state.distribution_event === 'time_schedule' ? 'selected' : ''}>Time / schedule</option>
                        <option value="cic" ${state.distribution_event === 'cic' ? 'selected' : ''}>Change in control</option>
                        <option value="emergency" ${state.distribution_event === 'emergency' ? 'selected' : ''}>Unforeseeable emergency</option>
                        <option value="other_violation" ${state.distribution_event === 'other_violation' ? 'selected' : ''}>Other (VIOLATION)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s409A.label.initial">Initial deferral election timely?</span>
                    <input type="checkbox" name="initial_election_timely" ${state.initial_election_timely ? 'checked' : ''}></label>
                <label><span data-i18n="view.s409A.label.subsequent">Subsequent deferral timely?</span>
                    <input type="checkbox" name="subsequent_deferral_timely" ${state.subsequent_deferral_timely ? 'checked' : ''}></label>
                <label><span data-i18n="view.s409A.label.5yr">Subsequent 5-yr push-out?</span>
                    <input type="checkbox" name="five_year_push_out" ${state.five_year_push_out ? 'checked' : ''}></label>
                <label><span data-i18n="view.s409A.label.short_term">Short-term deferral exception?</span>
                    <input type="checkbox" name="is_short_term_deferral" ${state.is_short_term_deferral ? 'checked' : ''}></label>
                <label><span data-i18n="view.s409A.label.sep_pay">Separation pay arrangement?</span>
                    <input type="checkbox" name="is_separation_pay_arrangement" ${state.is_separation_pay_arrangement ? 'checked' : ''}></label>
                <label><span data-i18n="view.s409A.label.arrangement">Arrangement type</span>
                    <select name="arrangement_type">
                        <option value="salary_continuation" ${state.arrangement_type === 'salary_continuation' ? 'selected' : ''}>Salary continuation</option>
                        <option value="bonus_deferral" ${state.arrangement_type === 'bonus_deferral' ? 'selected' : ''}>Bonus deferral</option>
                        <option value="serp" ${state.arrangement_type === 'serp' ? 'selected' : ''}>SERP (top-hat)</option>
                        <option value="rsu_settled_cash" ${state.arrangement_type === 'rsu_settled_cash' ? 'selected' : ''}>RSU settled in cash</option>
                        <option value="phantom_stock" ${state.arrangement_type === 'phantom_stock' ? 'selected' : ''}>Phantom stock / SAR</option>
                        <option value="discounted_option" ${state.arrangement_type === 'discounted_option' ? 'selected' : ''}>Discounted stock option (409A trap)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s409A.label.specified">Specified employee (public)?</span>
                    <input type="checkbox" name="is_specified_employee" ${state.is_specified_employee ? 'checked' : ''}></label>
                <label><span data-i18n="view.s409A.label.violation">Violation: tax in current year?</span>
                    <input type="number" step="0.01" name="violation_year_tax" value="${state.violation_year_tax}"></label>
                <label><span data-i18n="view.s409A.label.market_rate">Market rate interest %</span>
                    <input type="number" step="0.1" name="market_rate_interest" value="${state.market_rate_interest}"></label>
                <label><span data-i18n="view.s409A.label.contractor">Independent contractor?</span>
                    <input type="checkbox" name="is_independent_contractor" ${state.is_independent_contractor ? 'checked' : ''}></label>
                <label><span data-i18n="view.s409A.label.qualified">Tax-qualified plan (401(k) etc.)?</span>
                    <input type="checkbox" name="is_tax_qualified_plan" ${state.is_tax_qualified_plan ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s409A.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s409A-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s409A.h2.exceptions">Exempt from § 409A</h2>
            <ul class="muted small">
                <li data-i18n="view.s409A.exc.qualified">Tax-qualified plans: § 401(k), 403(b), 457(b), pensions, ESOPs</li>
                <li data-i18n="view.s409A.exc.s125">§ 125 cafeteria plans (medical, dependent care FSAs)</li>
                <li data-i18n="view.s409A.exc.s127">§ 127 educational assistance</li>
                <li data-i18n="view.s409A.exc.short_term">Short-term deferral: paid within 2.5 months of vesting</li>
                <li data-i18n="view.s409A.exc.separation_safe_harbor">Separation pay: 2-yr cap + 2× FICA wages limit ($330K × 2 = $660K)</li>
                <li data-i18n="view.s409A.exc.stock_rights">Stock rights: at-the-money options + RSUs (settled when vest)</li>
                <li data-i18n="view.s409A.exc.iso">ISO + § 423 ESPP § 421-424 (stock plans)</li>
                <li data-i18n="view.s409A.exc.contractor">Non-employee director / contractor with multiple unrelated customers</li>
                <li data-i18n="view.s409A.exc.welfare">Welfare benefit plans (group health, life, disability)</li>
                <li data-i18n="view.s409A.exc.foreign">Foreign plans of bona fide foreign nationals (limited carve-out)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s409A.h2.distribution">6 permitted distribution events</h2>
            <ol class="muted small">
                <li data-i18n="view.s409A.dist.separation">Separation from service (resignation, retirement, termination)</li>
                <li data-i18n="view.s409A.dist.disability">Disability — § 409A definition (NOT employer's)</li>
                <li data-i18n="view.s409A.dist.death">Death</li>
                <li data-i18n="view.s409A.dist.time">Specified time or schedule — must be objectively determinable</li>
                <li data-i18n="view.s409A.dist.cic">Change in control — § 409A specific definitions (not always M&A)</li>
                <li data-i18n="view.s409A.dist.emergency">Unforeseeable emergency — limited; no medical / casualty insurance</li>
                <li data-i18n="view.s409A.dist.specified_employee">SPECIFIED EMPLOYEE (public co): 6-month delay on separation distributions</li>
                <li data-i18n="view.s409A.dist.violation">"Other" event = VIOLATION (accelerated distribution, anti-forfeiture clauses)</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s409A.h2.violations">Common § 409A violations</h2>
            <ul class="muted small">
                <li data-i18n="view.s409A.viol.discount">Discounted stock options: strike &lt; FMV = built-in deferred comp</li>
                <li data-i18n="view.s409A.viol.no_election">Failure to make timely initial election</li>
                <li data-i18n="view.s409A.viol.late_change">Late change of distribution timing without 1-yr advance + 5-yr push-out</li>
                <li data-i18n="view.s409A.viol.acceleration">Acceleration of distribution to event other than 6 permitted</li>
                <li data-i18n="view.s409A.viol.no_six_month">Specified employee separation pay distributed before 6-month delay</li>
                <li data-i18n="view.s409A.viol.haircut">"Haircut" provisions (extra benefits for cause-related distributions)</li>
                <li data-i18n="view.s409A.viol.cic_definition">Inaccurate change in control definition (must match § 409A regs)</li>
                <li data-i18n="view.s409A.viol.severance">Severance &gt; safe harbor without 6-month delay (public co)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s409A.h2.penalty">§ 409A penalty mechanics</h2>
            <ul class="muted small">
                <li data-i18n="view.s409A.pen.all_deferred">ALL deferred amounts taxable in year of violation (not just current vested)</li>
                <li data-i18n="view.s409A.pen.20pct">+ 20% additional tax (Form 5329)</li>
                <li data-i18n="view.s409A.pen.interest">+ Interest at underpayment rate + 1% from year of deferral</li>
                <li data-i18n="view.s409A.pen.california">California: 20% state + interest stacks on top (parallel state law)</li>
                <li data-i18n="view.s409A.pen.no_employee_relief">No relief for employee even if employer's fault</li>
                <li data-i18n="view.s409A.pen.correction">Notice 2010-6 + 2008-113: limited correction safe harbors (some violations)</li>
                <li data-i18n="view.s409A.pen.amended_return">Amended return required for corrected violations</li>
                <li data-i18n="view.s409A.pen.aggregation">Aggregation rules: all plans of same type aggregated for violation</li>
            </ul>
        </div>
    `;
    document.getElementById('s409A-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.deferred_amount = Number(fd.get('deferred_amount')) || 0;
        state.distribution_event = fd.get('distribution_event');
        state.initial_election_timely = !!fd.get('initial_election_timely');
        state.subsequent_deferral_timely = !!fd.get('subsequent_deferral_timely');
        state.five_year_push_out = !!fd.get('five_year_push_out');
        state.is_short_term_deferral = !!fd.get('is_short_term_deferral');
        state.is_separation_pay_arrangement = !!fd.get('is_separation_pay_arrangement');
        state.arrangement_type = fd.get('arrangement_type');
        state.is_specified_employee = !!fd.get('is_specified_employee');
        state.violation_year_tax = Number(fd.get('violation_year_tax')) || 0;
        state.market_rate_interest = Number(fd.get('market_rate_interest')) || 0;
        state.is_independent_contractor = !!fd.get('is_independent_contractor');
        state.is_tax_qualified_plan = !!fd.get('is_tax_qualified_plan');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s409A-output');
    if (!el) return;
    const exempt = state.is_tax_qualified_plan || state.is_short_term_deferral || state.arrangement_type === 'iso';
    const violationFlags = state.arrangement_type === 'discounted_option' || state.distribution_event === 'other_violation' ||
        !state.initial_election_timely || !state.subsequent_deferral_timely;
    const isViolation = !exempt && violationFlags;
    const incomeRecognized = isViolation ? state.deferred_amount : 0;
    const incomeTax = incomeRecognized * 0.37;
    const additionalTax = isViolation ? state.deferred_amount * 0.20 : 0;
    const interest = isViolation ? state.deferred_amount * 0.04 * ((state.market_rate_interest + 1) / 100) : 0;
    const totalPenalty = additionalTax + interest;
    const totalCost = incomeTax + totalPenalty;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s409A.h2.result">§ 409A outcome</h2>
            <div class="cards">
                <div class="card ${exempt ? 'pos' : ''}">
                    <div class="label" data-i18n="view.s409A.card.exempt">Exempt from § 409A?</div>
                    <div class="value">${exempt ? esc(t('view.s409A.status.yes')) : esc(t('view.s409A.status.no'))}</div>
                </div>
                <div class="card ${isViolation ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s409A.card.violation">Violation triggered?</div>
                    <div class="value">${isViolation ? esc(t('view.s409A.status.yes')) : esc(t('view.s409A.status.no'))}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s409A.card.income">Income recognized</div>
                    <div class="value">$${incomeRecognized.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s409A.card.income_tax">Income tax (37%)</div>
                    <div class="value">$${incomeTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s409A.card.20pct">+20% additional tax</div>
                    <div class="value">$${additionalTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s409A.card.interest">+ Interest</div>
                    <div class="value">$${interest.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s409A.card.total">TOTAL COST</div>
                    <div class="value">$${totalCost.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${isViolation ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s409A.violation_note">
                    § 409A violation triggers ALL deferred amounts current taxation + 20% penalty + interest.
                    Effective tax rate often EXCEEDS 60%. Consider Notice 2010-6 / 2008-113 correction safe
                    harbors for operational failures. Document failure prevention via plan administration audit.
                </p>
            ` : ''}
        </div>
    `;
}
