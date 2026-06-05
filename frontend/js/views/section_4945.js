// IRC § 4945 — Private Foundation Taxable Expenditures.
// 20% initial PF + 5% on manager (knowing) for: (1) lobbying / political, (2) grants to
// individuals without expenditure responsibility, (3) grants to non-public-charity orgs
// without expenditure responsibility, (4) any non-charitable purpose.
// 100% PF + 50% manager if not corrected.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const PF_INITIAL = 0.20;
const MANAGER_INITIAL = 0.05;
const PF_SECOND = 1.00;
const MANAGER_SECOND = 0.50;
const MANAGER_PENALTY_CAP = 20_000;

let state = {
    expenditure_amount: 0,
    expenditure_kind: 'individual_grant',
    has_expenditure_responsibility: false,
    has_advance_irs_approval: false,
    grantee_type: 'individual',
    purpose: 'travel_study',
    manager_knew: false,
    months_uncorrected: 0,
};

export async function renderSection4945(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s4945.h1.title">// § 4945 PF TAXABLE EXPENDITURES</span></h1>
        <p class="muted small" data-i18n="view.s4945.hint.intro">
            <strong>20% initial PF + 5% manager</strong> (knowing) for: (1) lobbying / political,
            (2) grants to individuals without expenditure responsibility, (3) grants to non-public-
            charity orgs without expenditure responsibility, (4) non-charitable purpose.
            <strong>100% PF + 50% manager</strong> if not corrected within taxable period.
            Manager penalty capped at $20,000 per expenditure.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s4945.h2.inputs">Inputs</h2>
            <form id="s4945-form" class="inline-form">
                <label><span data-i18n="view.s4945.label.amount">Expenditure amount ($)</span>
                    <input type="number" step="0.01" name="expenditure_amount" value="${state.expenditure_amount}"></label>
                <label><span data-i18n="view.s4945.label.kind">Expenditure kind</span>
                    <select name="expenditure_kind">
                        <option value="individual_grant">Grant to individual</option>
                        <option value="org_grant_non_charity">Grant to non-public-charity org</option>
                        <option value="org_grant_public_charity">Grant to public charity</option>
                        <option value="lobbying">Lobbying / influence legislation</option>
                        <option value="political">Political campaign / specific candidate</option>
                        <option value="influence_election">Voter registration / influence election</option>
                        <option value="set_aside">Set-aside without approval</option>
                        <option value="non_charitable">Non-charitable purpose</option>
                    </select>
                </label>
                <label><span data-i18n="view.s4945.label.ER">Has expenditure responsibility (§ 4945(h))?</span>
                    <input type="checkbox" name="has_expenditure_responsibility" ${state.has_expenditure_responsibility ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4945.label.advance">Has advance IRS approval (Rev. Proc. 2024-14)?</span>
                    <input type="checkbox" name="has_advance_irs_approval" ${state.has_advance_irs_approval ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4945.label.grantee">Grantee type</span>
                    <select name="grantee_type">
                        <option value="individual">Individual</option>
                        <option value="public_charity">Public charity 501(c)(3)</option>
                        <option value="private_foundation">Private foundation</option>
                        <option value="foreign_charity">Foreign charity</option>
                        <option value="for_profit">For-profit organization</option>
                        <option value="government">Government unit</option>
                    </select>
                </label>
                <label><span data-i18n="view.s4945.label.purpose">Purpose of grant</span>
                    <select name="purpose">
                        <option value="travel_study">Travel / study / similar</option>
                        <option value="scholarship">Scholarship / fellowship</option>
                        <option value="prize_award">Prize / award (for charitable achievement)</option>
                        <option value="emergency_aid">Emergency hardship aid</option>
                        <option value="achievement">Charitable achievement</option>
                        <option value="other">Other charitable purpose</option>
                    </select>
                </label>
                <label><span data-i18n="view.s4945.label.knew">Manager knew problematic?</span>
                    <input type="checkbox" name="manager_knew" ${state.manager_knew ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4945.label.months_uncorr">Months uncorrected</span>
                    <input type="number" step="1" name="months_uncorrected" value="${state.months_uncorrected}"></label>
                <button class="primary" type="submit" data-i18n="view.s4945.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s4945-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4945.h2.expenditure_responsibility">§ 4945(h) Expenditure Responsibility</h2>
            <ul class="muted small">
                <li data-i18n="view.s4945.er.pre_grant">Pre-grant inquiry: ensure grantee + funds will be used per grant terms</li>
                <li data-i18n="view.s4945.er.written">Written grant agreement specifying charitable use</li>
                <li data-i18n="view.s4945.er.reports">Annual + final grantee reports required</li>
                <li data-i18n="view.s4945.er.records">PF retains records demonstrating proper use</li>
                <li data-i18n="view.s4945.er.recovery">Take appropriate steps if funds misused (refund / lawsuit)</li>
                <li data-i18n="view.s4945.er.cost_segregation">Grantee separately accounts for grant funds</li>
                <li data-i18n="view.s4945.er.report_to_irs">PF reports grants under expenditure responsibility on Form 990-PF Schedule O</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4945.h2.individual_grant_approval">Individual grant pre-approval (Rev. Proc. 2024-14)</h2>
            <ul class="muted small">
                <li data-i18n="view.s4945.indiv.objective">Objective + non-discriminatory selection criteria</li>
                <li data-i18n="view.s4945.indiv.qualified">Selected from qualified group sufficiently broad</li>
                <li data-i18n="view.s4945.indiv.purposes">For travel / study / similar charitable / scientific / educational purposes</li>
                <li data-i18n="view.s4945.indiv.advance">Pre-approval request to IRS prior to grant</li>
                <li data-i18n="view.s4945.indiv.report">Grantee reports per agreement; PF reviews</li>
                <li data-i18n="view.s4945.indiv.employer_relate">No relationship to PF / disqualified persons (anti-abuse)</li>
                <li data-i18n="view.s4945.indiv.alternative_er">Alternative: use expenditure responsibility instead of pre-approval</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4945.h2.lobbying">Lobbying + political restrictions</h2>
            <ul class="muted small">
                <li data-i18n="view.s4945.lob.no_propaganda">No carrying on propaganda / influencing legislation</li>
                <li data-i18n="view.s4945.lob.educational">Educational research with discussion OK (Reg § 53.4945-2)</li>
                <li data-i18n="view.s4945.lob.technical">Technical advice in response to written request from gov body OK</li>
                <li data-i18n="view.s4945.lob.public_charity">Grant to public charity for general support: not lobbying even if charity lobbies</li>
                <li data-i18n="view.s4945.lob.specific_project">Specific lobbying-purpose grants ARE taxable expenditures</li>
                <li data-i18n="view.s4945.lob.non_partisan">Non-partisan voter education + registration OK</li>
                <li data-i18n="view.s4945.lob.ballot_measures">Ballot measure advocacy MAY be lobbying (state-specific)</li>
            </ul>
        </div>
    `;
    document.getElementById('s4945-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.expenditure_amount = Number(fd.get('expenditure_amount')) || 0;
        state.expenditure_kind = fd.get('expenditure_kind');
        state.has_expenditure_responsibility = !!fd.get('has_expenditure_responsibility');
        state.has_advance_irs_approval = !!fd.get('has_advance_irs_approval');
        state.grantee_type = fd.get('grantee_type');
        state.purpose = fd.get('purpose');
        state.manager_knew = !!fd.get('manager_knew');
        state.months_uncorrected = Number(fd.get('months_uncorrected')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s4945-output');
    if (!el) return;
    let isTaxable = false;
    if (state.expenditure_kind === 'individual_grant') {
        isTaxable = !state.has_advance_irs_approval && !state.has_expenditure_responsibility;
    } else if (state.expenditure_kind === 'org_grant_non_charity') {
        isTaxable = !state.has_expenditure_responsibility;
    } else if (state.expenditure_kind === 'lobbying' || state.expenditure_kind === 'political' || state.expenditure_kind === 'non_charitable') {
        isTaxable = true;
    } else if (state.expenditure_kind === 'org_grant_public_charity' || state.expenditure_kind === 'set_aside') {
        isTaxable = false;
    }
    const pfInitial = isTaxable ? state.expenditure_amount * PF_INITIAL : 0;
    const managerInitialRaw = (isTaxable && state.manager_knew) ? state.expenditure_amount * MANAGER_INITIAL : 0;
    const managerInitial = Math.min(managerInitialRaw, MANAGER_PENALTY_CAP);
    const pfSecond = (isTaxable && state.months_uncorrected >= 24) ? state.expenditure_amount * PF_SECOND : 0;
    const managerSecondRaw = (isTaxable && state.manager_knew && state.months_uncorrected >= 24) ? state.expenditure_amount * MANAGER_SECOND : 0;
    const managerSecond = Math.min(managerSecondRaw, MANAGER_PENALTY_CAP);
    const totalExcise = pfInitial + managerInitial + pfSecond + managerSecond;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s4945.h2.result">Taxable expenditure analysis</h2>
            <div class="cards">
                <div class="card ${isTaxable ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s4945.card.taxable">Taxable expenditure?</div>
                    <div class="value">${isTaxable ? esc(t('view.s4945.status.yes')) : esc(t('view.s4945.status.no'))}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s4945.card.pf_initial">PF 20% initial</div>
                    <div class="value">$${pfInitial.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${managerInitial > 0 ? 'neg' : ''}">
                    <div class="label" data-i18n="view.s4945.card.manager_initial">Manager 5% (capped $20k)</div>
                    <div class="value">$${managerInitial.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${pfSecond > 0 ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.s4945.card.pf_second">PF SECOND 100%</div>
                        <div class="value">$${pfSecond.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
                <div class="card neg">
                    <div class="label" data-i18n="view.s4945.card.total">Total excise</div>
                    <div class="value">$${totalExcise.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
