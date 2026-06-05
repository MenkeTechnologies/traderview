// IRC § 4972 — Excise Tax on Nondeductible Contributions to Qualified Plans.
// 10% excise on nondeductible contributions to qualified plans (defined contribution + DB).
// Sister to § 4973 excess IRA + § 4974 RMD shortfall.
// Plans: 401(k), profit-sharing, SEP-IRA, money purchase, defined benefit pension.
// Form 5330 reports + due by last day of 7th month after plan year end.

import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    plan_type: '401k',
    employer_contribution: 0,
    s404_limit: 0,
    nondeductible_amount: 0,
    plan_year: 2024,
    plan_year_end_month: 12,
    plan_year_end_day: 31,
    correction_made: false,
    s402g_excess_elective: 0,
    catchup_eligible: false,
    catchup_amount: 0,
    is_top_heavy: false,
    is_safe_harbor: false,
    matching_contribution: 0,
    profit_sharing_contribution: 0,
    is_multi_year_carryover: false,
    prior_year_excess: 0,
};

export async function renderSection4972(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s4972.h1.title">// § 4972 NONDEDUCTIBLE PLAN CONTRIBUTION</span></h1>
        <p class="muted small" data-i18n="view.s4972.hint.intro">
            <strong>10% excise tax</strong> on nondeductible contributions to qualified plans
            (defined contribution + defined benefit). <strong>Plans:</strong> 401(k), profit-sharing,
            money purchase, SEP-IRA, SIMPLE, defined benefit pension. <strong>§ 404 limit:</strong>
            generally 25% of participant compensation (DC) or actuarial amount (DB).
            <strong>Sister:</strong> § 4973 excess IRA, § 4974 RMD shortfall. <strong>Form 5330</strong>
            reports + due last day of 7th month after plan year end.
            <strong>§ 402(g) elective deferral limit:</strong> $23,000 (2024, indexed).
            <strong>Catch-up:</strong> $7,500 (age 50+) + $11,250 (age 60-63 SECURE 2.0).
            <strong>Cumulative excise:</strong> 10% applies EACH year excess remains.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s4972.h2.inputs">Inputs</h2>
            <form id="s4972-form" class="inline-form">
                <label><span data-i18n="view.s4972.label.plan">Plan type</span>
                    <select name="plan_type">
                        <option value="401k" ${state.plan_type === '401k' ? 'selected' : ''}>401(k)</option>
                        <option value="profit_sharing" ${state.plan_type === 'profit_sharing' ? 'selected' : ''}>Profit-sharing</option>
                        <option value="money_purchase" ${state.plan_type === 'money_purchase' ? 'selected' : ''}>Money purchase</option>
                        <option value="sep_ira" ${state.plan_type === 'sep_ira' ? 'selected' : ''}>SEP-IRA</option>
                        <option value="simple" ${state.plan_type === 'simple' ? 'selected' : ''}>SIMPLE</option>
                        <option value="defined_benefit" ${state.plan_type === 'defined_benefit' ? 'selected' : ''}>Defined benefit pension</option>
                        <option value="cash_balance" ${state.plan_type === 'cash_balance' ? 'selected' : ''}>Cash balance plan</option>
                    </select>
                </label>
                <label><span data-i18n="view.s4972.label.employer">Total employer contribution ($)</span>
                    <input type="number" step="0.01" name="employer_contribution" value="${state.employer_contribution}"></label>
                <label><span data-i18n="view.s4972.label.s404">§ 404 deduction limit ($)</span>
                    <input type="number" step="0.01" name="s404_limit" value="${state.s404_limit}"></label>
                <label><span data-i18n="view.s4972.label.nondeductible">Computed nondeductible amount ($)</span>
                    <input type="number" step="0.01" name="nondeductible_amount" value="${state.nondeductible_amount}"></label>
                <label><span data-i18n="view.s4972.label.year">Plan year</span>
                    <input type="number" step="1" name="plan_year" value="${state.plan_year}"></label>
                <label><span data-i18n="view.s4972.label.month">Year-end month</span>
                    <input type="number" step="1" name="plan_year_end_month" value="${state.plan_year_end_month}"></label>
                <label><span data-i18n="view.s4972.label.day">Year-end day</span>
                    <input type="number" step="1" name="plan_year_end_day" value="${state.plan_year_end_day}"></label>
                <label><span data-i18n="view.s4972.label.correction">Correction made?</span>
                    <input type="checkbox" name="correction_made" ${state.correction_made ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4972.label.s402g">§ 402(g) excess elective ($)</span>
                    <input type="number" step="0.01" name="s402g_excess_elective" value="${state.s402g_excess_elective}"></label>
                <label><span data-i18n="view.s4972.label.catchup">Catch-up eligible?</span>
                    <input type="checkbox" name="catchup_eligible" ${state.catchup_eligible ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4972.label.catchup_amount">Catch-up amount ($)</span>
                    <input type="number" step="0.01" name="catchup_amount" value="${state.catchup_amount}"></label>
                <label><span data-i18n="view.s4972.label.topheavy">Top-heavy plan?</span>
                    <input type="checkbox" name="is_top_heavy" ${state.is_top_heavy ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4972.label.safe_harbor">Safe harbor?</span>
                    <input type="checkbox" name="is_safe_harbor" ${state.is_safe_harbor ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4972.label.matching">Matching contribution ($)</span>
                    <input type="number" step="0.01" name="matching_contribution" value="${state.matching_contribution}"></label>
                <label><span data-i18n="view.s4972.label.ps">Profit-sharing contribution ($)</span>
                    <input type="number" step="0.01" name="profit_sharing_contribution" value="${state.profit_sharing_contribution}"></label>
                <label><span data-i18n="view.s4972.label.carryover">Multi-year carryover?</span>
                    <input type="checkbox" name="is_multi_year_carryover" ${state.is_multi_year_carryover ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4972.label.prior">Prior year excess ($)</span>
                    <input type="number" step="0.01" name="prior_year_excess" value="${state.prior_year_excess}"></label>
                <button class="primary" type="submit" data-i18n="view.s4972.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s4972-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4972.h2.limits">2024 contribution + deduction limits</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s4972.tbl.limit_type">Limit type</th><th data-i18n="view.s4972.tbl.amount">2024 amount</th><th data-i18n="view.s4972.tbl.citation">Citation</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s4972.tbl.s415c">§ 415(c) DC annual addition</td><td>$69,000</td><td>§ 415(c)(1)</td></tr>
                    <tr><td data-i18n="view.s4972.tbl.s415b">§ 415(b) DB annual benefit</td><td>$275,000</td><td>§ 415(b)(1)</td></tr>
                    <tr><td data-i18n="view.s4972.tbl.s402g">§ 402(g) elective deferral</td><td>$23,000</td><td>§ 402(g)(1)</td></tr>
                    <tr><td data-i18n="view.s4972.tbl.catchup_50">Catch-up (age 50+)</td><td>$7,500</td><td>§ 414(v)</td></tr>
                    <tr><td data-i18n="view.s4972.tbl.catchup_60">Catch-up (age 60-63)</td><td>$11,250</td><td>SECURE 2.0 § 109</td></tr>
                    <tr><td data-i18n="view.s4972.tbl.s401a17">§ 401(a)(17) comp limit</td><td>$345,000</td><td>§ 401(a)(17)</td></tr>
                    <tr><td data-i18n="view.s4972.tbl.s404a">§ 404(a)(3) DC deduction</td><td>25% of comp</td><td>§ 404(a)(3)</td></tr>
                    <tr><td data-i18n="view.s4972.tbl.simple">SIMPLE elective deferral</td><td>$16,000</td><td>§ 408(p)(2)(E)</td></tr>
                    <tr><td data-i18n="view.s4972.tbl.simple_catchup">SIMPLE catch-up (50+)</td><td>$3,500</td><td>§ 414(v)</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4972.h2.cumulative">Cumulative excise application</h2>
            <ul class="muted small">
                <li data-i18n="view.s4972.cum.annual">10% applied EACH year excess remains undistributed</li>
                <li data-i18n="view.s4972.cum.compound">Compound effect: $100K excess for 3 years = $30K total excise</li>
                <li data-i18n="view.s4972.cum.deemed">Excess deemed reduced by current-year contribution shortfall</li>
                <li data-i18n="view.s4972.cum.distribution">Distribution of excess = withdrawal subject to income tax + § 72 early</li>
                <li data-i18n="view.s4972.cum.return">Trustee return of excess by Apr 15 next year avoids further excise</li>
                <li data-i18n="view.s4972.cum.allocation">Multiple-employer plans: excise on employer making excess contribution</li>
                <li data-i18n="view.s4972.cum.es_correction">Self-correction under EPCRS § 6.06 + Rev. Proc. 2021-30</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4972.h2.coordination">Coordination + special rules</h2>
            <ul class="muted small">
                <li data-i18n="view.s4972.coord.s404">§ 404(a) deduction limit (25% comp DC, actuarial DB) is the bright line</li>
                <li data-i18n="view.s4972.coord.s415">§ 415(c) annual addition limit DIFFERENT from § 404 deduction limit</li>
                <li data-i18n="view.s4972.coord.dc_db">DC + DB combined: § 404(a)(7) lesser of 25% comp OR aggregate deduction</li>
                <li data-i18n="view.s4972.coord.s4972_d">§ 4972(d) — calculation considers carryover + current year</li>
                <li data-i18n="view.s4972.coord.s4973_excess_ira">§ 4973 6% on excess IRA (SEP-IRA / SIMPLE IRA)</li>
                <li data-i18n="view.s4972.coord.s4979">§ 4979 10% excise on excess elective deferrals (HCE refund)</li>
                <li data-i18n="view.s4972.coord.s4980f">§ 4980F 100% excise on disqualified plan reversion (top-heavy)</li>
                <li data-i18n="view.s4972.coord.epcrs">EPCRS self-correction + voluntary correction + audit closing agreement</li>
            </ul>
        </div>
    `;
    document.getElementById('s4972-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.plan_type = fd.get('plan_type');
        state.employer_contribution = Number(fd.get('employer_contribution')) || 0;
        state.s404_limit = Number(fd.get('s404_limit')) || 0;
        state.nondeductible_amount = Number(fd.get('nondeductible_amount')) || 0;
        state.plan_year = Number(fd.get('plan_year')) || 0;
        state.plan_year_end_month = Number(fd.get('plan_year_end_month')) || 0;
        state.plan_year_end_day = Number(fd.get('plan_year_end_day')) || 0;
        state.correction_made = !!fd.get('correction_made');
        state.s402g_excess_elective = Number(fd.get('s402g_excess_elective')) || 0;
        state.catchup_eligible = !!fd.get('catchup_eligible');
        state.catchup_amount = Number(fd.get('catchup_amount')) || 0;
        state.is_top_heavy = !!fd.get('is_top_heavy');
        state.is_safe_harbor = !!fd.get('is_safe_harbor');
        state.matching_contribution = Number(fd.get('matching_contribution')) || 0;
        state.profit_sharing_contribution = Number(fd.get('profit_sharing_contribution')) || 0;
        state.is_multi_year_carryover = !!fd.get('is_multi_year_carryover');
        state.prior_year_excess = Number(fd.get('prior_year_excess')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s4972-output');
    if (!el) return;
    const computed_excess = Math.max(0, state.employer_contribution - state.s404_limit);
    const excise = computed_excess * 0.10;
    const prior_excise = state.is_multi_year_carryover ? state.prior_year_excess * 0.10 : 0;
    const total_excise = excise + prior_excise;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s4972.h2.result">§ 4972 nondeductible contribution excise</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s4972.card.contrib">Contribution</div>
                    <div class="value">$${state.employer_contribution.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s4972.card.limit">§ 404 limit</div>
                    <div class="value">$${state.s404_limit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${computed_excess > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s4972.card.excess">Excess</div>
                    <div class="value">$${computed_excess.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s4972.card.excise">Current excise</div>
                    <div class="value">$${excise.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${total_excise > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s4972.card.total">Total (incl prior)</div>
                    <div class="value">$${total_excise.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
