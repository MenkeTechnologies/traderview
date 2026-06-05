// IRC § 7430 — Recovery of Attorney Fees + Litigation Costs.
// Prevailing taxpayer can recover REASONABLE administrative + litigation costs from gov't.
// Conditions: (1) substantially prevailed on amount or most significant issue,
// (2) net worth < $2M individual / $7M corp / 500 employees,
// (3) IRS position was NOT substantially justified.
// Capped at $230/hour (2024 indexed for cost of living).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const HOURLY_CAP_2024 = 230;
const NET_WORTH_CAP_INDIVIDUAL = 2_000_000;
const NET_WORTH_CAP_CORP = 7_000_000;
const EMPLOYEE_CAP_CORP = 500;

let state = {
    taxpayer_kind: 'individual',
    net_worth: 0,
    employees: 0,
    attorney_hours: 0,
    actual_hourly_rate: 0,
    expert_fees: 0,
    other_litigation_costs: 0,
    substantially_prevailed: false,
    irs_substantially_justified: true,
    qualified_offer_made: false,
    qualified_offer_amount: 0,
    final_judgment_amount: 0,
};

export async function renderSection7430(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s7430.h1.title">// § 7430 ATTORNEY FEES RECOVERY</span></h1>
        <p class="muted small" data-i18n="view.s7430.hint.intro">
            Prevailing taxpayer can recover reasonable admin + litigation costs from gov't.
            Conditions: (1) <strong>substantially prevailed</strong> on amount or most significant
            issue, (2) <strong>net worth &lt; $2M individual / $7M corp / 500 employees</strong>,
            (3) <strong>IRS position NOT substantially justified</strong>. Cap: <strong>$230/hr
            (2024, indexed)</strong>. <strong>Qualified offer:</strong> recover even if didn't
            "prevail" if IRS later got worse outcome.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s7430.h2.inputs">Inputs</h2>
            <form id="s7430-form" class="inline-form">
                <label><span data-i18n="view.s7430.label.kind">Taxpayer kind</span>
                    <select name="taxpayer_kind">
                        <option value="individual" ${state.taxpayer_kind === 'individual' ? 'selected' : ''}>Individual</option>
                        <option value="estate_trust" ${state.taxpayer_kind === 'estate_trust' ? 'selected' : ''}>Estate / trust</option>
                        <option value="corporation" ${state.taxpayer_kind === 'corporation' ? 'selected' : ''}>Corporation / partnership</option>
                        <option value="nonprofit" ${state.taxpayer_kind === 'nonprofit' ? 'selected' : ''}>Nonprofit 501(c)(3)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s7430.label.net_worth">Net worth ($)</span>
                    <input type="number" step="0.01" name="net_worth" value="${state.net_worth}"></label>
                <label><span data-i18n="view.s7430.label.employees">Employees (corp only)</span>
                    <input type="number" step="1" name="employees" value="${state.employees}"></label>
                <label><span data-i18n="view.s7430.label.hours">Attorney hours</span>
                    <input type="number" step="1" name="attorney_hours" value="${state.attorney_hours}"></label>
                <label><span data-i18n="view.s7430.label.actual_rate">Actual hourly rate ($)</span>
                    <input type="number" step="0.01" name="actual_hourly_rate" value="${state.actual_hourly_rate}"></label>
                <label><span data-i18n="view.s7430.label.expert">Expert witness fees ($)</span>
                    <input type="number" step="0.01" name="expert_fees" value="${state.expert_fees}"></label>
                <label><span data-i18n="view.s7430.label.other">Other litigation costs ($)</span>
                    <input type="number" step="0.01" name="other_litigation_costs" value="${state.other_litigation_costs}"></label>
                <label><span data-i18n="view.s7430.label.prevailed">Substantially prevailed?</span>
                    <input type="checkbox" name="substantially_prevailed" ${state.substantially_prevailed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7430.label.justified">IRS position substantially justified?</span>
                    <input type="checkbox" name="irs_substantially_justified" ${state.irs_substantially_justified ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7430.label.qualified_offer">Qualified offer made?</span>
                    <input type="checkbox" name="qualified_offer_made" ${state.qualified_offer_made ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7430.label.offer_amount">Qualified offer amount ($)</span>
                    <input type="number" step="0.01" name="qualified_offer_amount" value="${state.qualified_offer_amount}"></label>
                <label><span data-i18n="view.s7430.label.final">Final judgment amount ($)</span>
                    <input type="number" step="0.01" name="final_judgment_amount" value="${state.final_judgment_amount}"></label>
                <button class="primary" type="submit" data-i18n="view.s7430.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s7430-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7430.h2.qualifying_costs">Qualifying costs</h2>
            <ul class="muted small">
                <li data-i18n="view.s7430.costs.attorney">Attorney fees at lesser of actual rate or $230/hr (2024)</li>
                <li data-i18n="view.s7430.costs.special">Special factor enhancement: $230/hr can be exceeded for unusual specialization</li>
                <li data-i18n="view.s7430.costs.experts">Expert witness fees + studies / analyses</li>
                <li data-i18n="view.s7430.costs.copying">Court costs + filing fees + copying / postage</li>
                <li data-i18n="view.s7430.costs.travel">Reasonable travel costs for taxpayer + counsel</li>
                <li data-i18n="view.s7430.costs.deposition">Deposition fees</li>
                <li data-i18n="view.s7430.costs.no_punitive">NO punitive damages — actual costs only</li>
                <li data-i18n="view.s7430.costs.admin">Administrative costs from time IRS could have settled</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7430.h2.qualified_offer">Qualified offer (§ 7430(g))</h2>
            <p class="muted small" data-i18n="view.s7430.qo.body">
                Taxpayer makes settlement offer in writing during admin process. If IRS rejects + later
                obtains a LESS favorable outcome (taxpayer wins more), § 7430 entitles taxpayer to costs
                from date of offer onwards REGARDLESS of substantially-justified question. Strong tool
                to flush out marginal cases + force IRS settlement.
            </p>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7430.h2.substantially_justified">"Substantially justified" standard</h2>
            <ul class="muted small">
                <li data-i18n="view.s7430.sj.pierce">Pierce v. Underwood: "reasonable basis in both law + fact"</li>
                <li data-i18n="view.s7430.sj.objective">Objective test from gov't perspective at time of position</li>
                <li data-i18n="view.s7430.sj.law_change">Subsequent change in law doesn't make prior position unjustified</li>
                <li data-i18n="view.s7430.sj.split_decisions">Cases where appellate courts split favor gov't (no clearly settled rule)</li>
                <li data-i18n="view.s7430.sj.factual_dispute">Factual disputes generally favor gov't if reasonable inferences possible</li>
                <li data-i18n="view.s7430.sj.taxpayer_burden">Burden ON taxpayer to prove NOT substantially justified</li>
                <li data-i18n="view.s7430.sj.amorphous">Standard is amorphous; success depends heavily on facts</li>
            </ul>
        </div>
    `;
    document.getElementById('s7430-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.taxpayer_kind = fd.get('taxpayer_kind');
        state.net_worth = Number(fd.get('net_worth')) || 0;
        state.employees = Number(fd.get('employees')) || 0;
        state.attorney_hours = Number(fd.get('attorney_hours')) || 0;
        state.actual_hourly_rate = Number(fd.get('actual_hourly_rate')) || 0;
        state.expert_fees = Number(fd.get('expert_fees')) || 0;
        state.other_litigation_costs = Number(fd.get('other_litigation_costs')) || 0;
        state.substantially_prevailed = !!fd.get('substantially_prevailed');
        state.irs_substantially_justified = !!fd.get('irs_substantially_justified');
        state.qualified_offer_made = !!fd.get('qualified_offer_made');
        state.qualified_offer_amount = Number(fd.get('qualified_offer_amount')) || 0;
        state.final_judgment_amount = Number(fd.get('final_judgment_amount')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s7430-output');
    if (!el) return;
    let qualifies;
    if (state.taxpayer_kind === 'individual') qualifies = state.net_worth < NET_WORTH_CAP_INDIVIDUAL;
    else if (state.taxpayer_kind === 'corporation') qualifies = state.net_worth < NET_WORTH_CAP_CORP && state.employees <= EMPLOYEE_CAP_CORP;
    else qualifies = state.net_worth < NET_WORTH_CAP_INDIVIDUAL;
    const qualifiedOfferTriggered = state.qualified_offer_made
        && state.final_judgment_amount < state.qualified_offer_amount;
    const meetsConditions = qualifiedOfferTriggered
        || (state.substantially_prevailed && !state.irs_substantially_justified);
    const recoverableHourly = Math.min(state.actual_hourly_rate, HOURLY_CAP_2024);
    const attorneyFees = state.attorney_hours * recoverableHourly;
    const totalCosts = attorneyFees + state.expert_fees + state.other_litigation_costs;
    const recoverable = qualifies && meetsConditions ? totalCosts : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s7430.h2.result">Recovery analysis</h2>
            <div class="cards">
                <div class="card ${qualifies ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s7430.card.size_qualifies">Size qualifies</div>
                    <div class="value">${qualifies ? esc(t('view.s7430.status.yes')) : esc(t('view.s7430.status.no'))}</div>
                </div>
                <div class="card ${meetsConditions ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s7430.card.conditions">Met conditions</div>
                    <div class="value">${meetsConditions ? esc(t('view.s7430.status.yes')) : esc(t('view.s7430.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s7430.card.recoverable_hourly">Recoverable hourly</div>
                    <div class="value">$${recoverableHourly.toLocaleString()}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s7430.card.attorney_fees">Attorney fees</div>
                    <div class="value">$${attorneyFees.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s7430.card.total_costs">Total costs</div>
                    <div class="value">$${totalCosts.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s7430.card.recoverable">Recoverable</div>
                    <div class="value">$${recoverable.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
