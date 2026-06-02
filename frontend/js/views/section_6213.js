// IRC § 6213 — Notice of Deficiency / Tax Court Petition.
// IRS must issue NOTICE OF DEFICIENCY (90-day letter, 150 if outside US) before assessment.
// Taxpayer has 90 days to file Tax Court petition (suspends assessment + collection).
// If no petition: IRS can assess + collect. Penalty for filing late = lose Tax Court rights.
// Alternative: pay tax + sue for refund in District Court or Court of Federal Claims.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const PETITION_DEADLINE_DAYS_US = 90;
const PETITION_DEADLINE_DAYS_OUTSIDE = 150;

let state = {
    notice_date: '',
    is_outside_us: false,
    proposed_deficiency: 0,
    interest_through_date: 0,
    accuracy_penalty: 0,
    fraud_penalty: 0,
    other_penalties: 0,
    response_strategy: 'tax_court',
    can_afford_to_pay: false,
};

export async function renderSection6213(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6213.h1.title">// § 6213 NOTICE OF DEFICIENCY</span></h1>
        <p class="muted small" data-i18n="view.s6213.hint.intro">
            IRS must issue "<strong>Notice of Deficiency (90-day letter)</strong>" before
            assessment. <strong>90 days</strong> to file Tax Court petition (<strong>150 days outside US</strong>).
            Petition SUSPENDS assessment + collection. Miss deadline = lose Tax Court rights.
            <strong>Alternative paths:</strong> pay + sue for refund (District Court / Court of Federal
            Claims) OR enter Appeals (administrative).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6213.h2.inputs">Inputs</h2>
            <form id="s6213-form" class="inline-form">
                <label><span data-i18n="view.s6213.label.notice_date">Notice date (90-day letter)</span>
                    <input type="date" name="notice_date" value="${state.notice_date}"></label>
                <label><span data-i18n="view.s6213.label.outside">Address outside US?</span>
                    <input type="checkbox" name="is_outside_us" ${state.is_outside_us ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6213.label.proposed">Proposed deficiency ($)</span>
                    <input type="number" step="1000" name="proposed_deficiency" value="${state.proposed_deficiency}"></label>
                <label><span data-i18n="view.s6213.label.interest">Interest through filing date ($)</span>
                    <input type="number" step="100" name="interest_through_date" value="${state.interest_through_date}"></label>
                <label><span data-i18n="view.s6213.label.accuracy">§ 6662 accuracy penalty 20% ($)</span>
                    <input type="number" step="100" name="accuracy_penalty" value="${state.accuracy_penalty}"></label>
                <label><span data-i18n="view.s6213.label.fraud">§ 6663 fraud penalty 75% ($)</span>
                    <input type="number" step="100" name="fraud_penalty" value="${state.fraud_penalty}"></label>
                <label><span data-i18n="view.s6213.label.other">Other penalties ($)</span>
                    <input type="number" step="100" name="other_penalties" value="${state.other_penalties}"></label>
                <label><span data-i18n="view.s6213.label.strategy">Response strategy</span>
                    <select name="response_strategy">
                        <option value="tax_court" ${state.response_strategy === 'tax_court' ? 'selected' : ''}>File Tax Court petition</option>
                        <option value="pay_and_sue" ${state.response_strategy === 'pay_and_sue' ? 'selected' : ''}>Pay + sue for refund</option>
                        <option value="appeals" ${state.response_strategy === 'appeals' ? 'selected' : ''}>IRS Appeals only (pre-petition)</option>
                        <option value="agree" ${state.response_strategy === 'agree' ? 'selected' : ''}>Agree + sign Form 870</option>
                        <option value="default" ${state.response_strategy === 'default' ? 'selected' : ''}>Default (do nothing — bad)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6213.label.afford">Can afford full payment now?</span>
                    <input type="checkbox" name="can_afford_to_pay" ${state.can_afford_to_pay ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s6213.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6213-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6213.h2.forums">Tax forum comparison</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s6213.th.forum">Forum</th>
                    <th data-i18n="view.s6213.th.prepay">Pay first?</th>
                    <th data-i18n="view.s6213.th.jury">Jury trial?</th>
                    <th data-i18n="view.s6213.th.appeals">Appeals to</th>
                </tr></thead>
                <tbody>
                    <tr><td>Tax Court (regular)</td><td>NO</td><td>NO</td><td>Circuit Court of Appeals (geographic)</td></tr>
                    <tr><td>Tax Court (small case &lt;$50k)</td><td>NO</td><td>NO</td><td>FINAL — no appeals</td></tr>
                    <tr><td>District Court</td><td>YES — pay + refund suit</td><td>YES (option)</td><td>Circuit Court of Appeals</td></tr>
                    <tr><td>Court of Federal Claims</td><td>YES</td><td>NO</td><td>Federal Circuit Court of Appeals</td></tr>
                    <tr><td>Bankruptcy Court</td><td>NO</td><td>NO</td><td>District Court / BAP</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6213.h2.strategy">Strategy considerations</h2>
            <ul class="muted small">
                <li data-i18n="view.s6213.strat.tax_court_pros">Tax Court PROS: don't pay first, suspend collection, expertise, small-case option</li>
                <li data-i18n="view.s6213.strat.district_pros">District Court PROS: jury option, better for "ordinary law" issues</li>
                <li data-i18n="view.s6213.strat.precedent">Choose forum based on most favorable PRECEDENT (DC Circuit favors gov't)</li>
                <li data-i18n="view.s6213.strat.appeals_first">Always try IRS Appeals BEFORE Tax Court (no cost, can settle)</li>
                <li data-i18n="view.s6213.strat.statute_running">Petition timing = day 90 of notice (postmark counts for first-class mail)</li>
                <li data-i18n="view.s6213.strat.designated">Designated payment under § 6603 stops interest from running</li>
                <li data-i18n="view.s6213.strat.payment_during">Payment during petition recharacterizes as deposit, refunded if win</li>
                <li data-i18n="view.s6213.strat.last_known_address">Notice valid if mailed to "last known address" — even if returned</li>
            </ul>
        </div>
    `;
    document.getElementById('s6213-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.notice_date = fd.get('notice_date');
        state.is_outside_us = !!fd.get('is_outside_us');
        state.proposed_deficiency = Number(fd.get('proposed_deficiency')) || 0;
        state.interest_through_date = Number(fd.get('interest_through_date')) || 0;
        state.accuracy_penalty = Number(fd.get('accuracy_penalty')) || 0;
        state.fraud_penalty = Number(fd.get('fraud_penalty')) || 0;
        state.other_penalties = Number(fd.get('other_penalties')) || 0;
        state.response_strategy = fd.get('response_strategy');
        state.can_afford_to_pay = !!fd.get('can_afford_to_pay');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6213-output');
    if (!el) return;
    const deadlineDays = state.is_outside_us ? PETITION_DEADLINE_DAYS_OUTSIDE : PETITION_DEADLINE_DAYS_US;
    let deadlineDate = '';
    let daysRemaining = 0;
    if (state.notice_date) {
        const noticeDate = new Date(state.notice_date);
        const deadline = new Date(noticeDate);
        deadline.setDate(deadline.getDate() + deadlineDays);
        deadlineDate = deadline.toISOString().slice(0, 10);
        daysRemaining = Math.max(0, Math.floor((deadline - new Date()) / (1000 * 60 * 60 * 24)));
    }
    const totalExposure = state.proposed_deficiency + state.interest_through_date + state.accuracy_penalty + state.fraud_penalty + state.other_penalties;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6213.h2.result">Deadline + exposure</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s6213.card.deadline_days">Petition deadline</div>
                    <div class="value">${deadlineDays} ${esc(t('view.s6213.units.days'))}</div>
                </div>
                <div class="card ${daysRemaining < 14 ? 'neg' : ''}">
                    <div class="label" data-i18n="view.s6213.card.deadline_date">Deadline date</div>
                    <div class="value">${esc(deadlineDate || '—')}</div>
                </div>
                <div class="card ${daysRemaining < 14 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s6213.card.days_left">Days remaining</div>
                    <div class="value">${daysRemaining}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6213.card.deficiency">Proposed deficiency</div>
                    <div class="value">$${state.proposed_deficiency.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6213.card.penalties">Total penalties</div>
                    <div class="value">$${(state.accuracy_penalty + state.fraud_penalty + state.other_penalties).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6213.card.total">Total exposure</div>
                    <div class="value">$${totalExposure.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${daysRemaining < 14 && daysRemaining > 0 ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s6213.warning.urgent">
                    URGENT: petition deadline approaching. File Tax Court petition NOW via TaxCourt.gov
                    or pay deficiency + designate § 6603 deposit while pursuing claims.
                </p>
            ` : ''}
        </div>
    `;
}
