// IRC § 4941 — Self-Dealing in Private Foundations.
// PRIVATE foundation (§ 509(a)) prohibited from any direct/indirect transaction with
// "disqualified person" (substantial contributor, manager, family). Strict liability.
// 10% excise on disqualified person + 5% on foundation manager who knowingly participated.
// 200% on disqualified person + 50% on manager if not corrected within taxable period.

import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const INITIAL_DQ_RATE = 0.10;
const INITIAL_MANAGER_RATE = 0.05;
const SECOND_DQ_RATE = 2.00;
const SECOND_MANAGER_RATE = 0.50;

let state = {
    transaction_type: 'sale',
    amount_involved: 0,
    years_uncorrected: 0,
    manager_knowing: false,
    disqualified_person_count: 1,
    is_intermediate_sanction: false,
};

export async function renderSection4941(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s4941.h1.title">// § 4941 PF SELF-DEALING</span></h1>
        <p class="muted small" data-i18n="view.s4941.hint.intro">
            Private foundation can't transact with "<strong>disqualified persons</strong>"
            (substantial contributor, manager, family, 35% owned entity). STRICT LIABILITY — no
            intent required. <strong>10% on disqualified person + 5% on knowing manager</strong>
            INITIAL. <strong>200% on DP + 50% on manager</strong> if not corrected within
            taxable period. Even "fair price" transactions PROHIBITED. Form 4720 + Form 990-PF.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s4941.h2.inputs">Inputs</h2>
            <form id="s4941-form" class="inline-form">
                <label><span data-i18n="view.s4941.label.kind">Transaction type</span>
                    <select name="transaction_type">
                        <option value="sale">Sale / exchange</option>
                        <option value="lease">Lease</option>
                        <option value="loan">Loan</option>
                        <option value="services">Services compensation</option>
                        <option value="transfer">Transfer of assets / income</option>
                        <option value="payment">Payment to government official</option>
                        <option value="indemnity">Indemnification / insurance</option>
                    </select>
                </label>
                <label><span data-i18n="view.s4941.label.amount">Amount involved ($)</span>
                    <input type="number" step="1000" name="amount_involved" value="${state.amount_involved}"></label>
                <label><span data-i18n="view.s4941.label.years">Years uncorrected</span>
                    <input type="number" step="1" name="years_uncorrected" value="${state.years_uncorrected}"></label>
                <label><span data-i18n="view.s4941.label.manager">Manager knowingly participated?</span>
                    <input type="checkbox" name="manager_knowing" ${state.manager_knowing ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4941.label.dq_count">Disqualified person count</span>
                    <input type="number" step="1" name="disqualified_person_count" value="${state.disqualified_person_count}"></label>
                <button class="primary" type="submit" data-i18n="view.s4941.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s4941-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4941.h2.disqualified">Disqualified persons</h2>
            <ul class="muted small">
                <li data-i18n="view.s4941.dq.substantial">Substantial contributor (≥ $5k AND &gt; 2% of total contributions historically)</li>
                <li data-i18n="view.s4941.dq.foundation_manager">Foundation manager (officer, director, trustee)</li>
                <li data-i18n="view.s4941.dq.20_owner">20%+ owner of substantial contributor entity</li>
                <li data-i18n="view.s4941.dq.family">Family member (spouse, ancestor, descendant + their spouses)</li>
                <li data-i18n="view.s4941.dq.35_owned">Entity 35%+ owned by combination of disqualified persons</li>
                <li data-i18n="view.s4941.dq.government">Government official (for § 4941(d)(1)(F))</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4941.h2.exceptions">§ 4941(d)(2) exceptions</h2>
            <ul class="muted small">
                <li data-i18n="view.s4941.ex.no_charge">Free use of property by foundation</li>
                <li data-i18n="view.s4941.ex.fair_rental_no">Rent-free housing for fundraising</li>
                <li data-i18n="view.s4941.ex.reasonable_comp">Reasonable comp for personal services NECESSARY to foundation</li>
                <li data-i18n="view.s4941.ex.incidental">Incidental benefit from charitable activities (e.g., banquet seat)</li>
                <li data-i18n="view.s4941.ex.bank_general">Bank deposit accounts at general commercial terms</li>
                <li data-i18n="view.s4941.ex.no_interest_loan">Interest-free / no-interest loans TO foundation (from DP)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4941.h2.related_excises">Other PF excises</h2>
            <ul class="muted small">
                <li data-i18n="view.s4941.rel.4940">§ 4940: 1.39% net investment income tax</li>
                <li data-i18n="view.s4941.rel.4942">§ 4942: 30% on undistributed income (5% minimum payout)</li>
                <li data-i18n="view.s4941.rel.4943">§ 4943: 10% on excess business holdings (20-35% rule)</li>
                <li data-i18n="view.s4941.rel.4944">§ 4944: 10% on jeopardizing investments (high-risk speculation)</li>
                <li data-i18n="view.s4941.rel.4945">§ 4945: 20% on taxable expenditures (lobbying, political, individual grants)</li>
                <li data-i18n="view.s4941.rel.4945_political">§ 4955: 10% political expenditure (public charities + PFs)</li>
            </ul>
        </div>
    `;
    document.getElementById('s4941-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.transaction_type = fd.get('transaction_type');
        state.amount_involved = Number(fd.get('amount_involved')) || 0;
        state.years_uncorrected = Number(fd.get('years_uncorrected')) || 0;
        state.manager_knowing = !!fd.get('manager_knowing');
        state.disqualified_person_count = Number(fd.get('disqualified_person_count')) || 1;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s4941-output');
    if (!el) return;
    const dqExciseInitial = state.amount_involved * INITIAL_DQ_RATE * state.years_uncorrected * state.disqualified_person_count;
    const managerExciseInitial = state.manager_knowing
        ? state.amount_involved * INITIAL_MANAGER_RATE * state.years_uncorrected
        : 0;
    const dqExciseSecond = state.years_uncorrected >= 2
        ? state.amount_involved * SECOND_DQ_RATE * state.disqualified_person_count
        : 0;
    const managerExciseSecond = (state.years_uncorrected >= 2 && state.manager_knowing)
        ? state.amount_involved * SECOND_MANAGER_RATE
        : 0;
    const totalExciseInitial = dqExciseInitial + managerExciseInitial;
    const totalExciseSecond = dqExciseSecond + managerExciseSecond;
    const totalAll = totalExciseInitial + totalExciseSecond;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s4941.h2.result">Excise tax exposure</h2>
            <div class="cards">
                <div class="card neg">
                    <div class="label" data-i18n="view.s4941.card.dq_initial">DP initial 10%</div>
                    <div class="value">$${dqExciseInitial.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${managerExciseInitial > 0 ? 'neg' : ''}">
                    <div class="label" data-i18n="view.s4941.card.mgr_initial">Manager initial 5%</div>
                    <div class="value">$${managerExciseInitial.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${dqExciseSecond > 0 ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.s4941.card.dq_second">DP SECOND 200% (uncorrected)</div>
                        <div class="value">$${dqExciseSecond.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
                ${managerExciseSecond > 0 ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.s4941.card.mgr_second">Manager SECOND 50%</div>
                        <div class="value">$${managerExciseSecond.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
                <div class="card neg">
                    <div class="label" data-i18n="view.s4941.card.total">TOTAL excise exposure</div>
                    <div class="value">$${totalAll.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
