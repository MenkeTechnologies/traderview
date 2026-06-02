// IRC § 4980D — Non-Compliant Group Health Plan Excise Tax.
// $100/day per affected individual = up to $36,500/year/employee for ACA group market reform violations.
// Common triggers: HRA paying individual market premiums (Notice 2013-54 prohibition pre-2020),
// non-ACA-compliant grandfathered + non-grandfathered plans, fee-for-service HRAs, ICHRAs without proper notices.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const PER_DAY_PENALTY = 100;
const DAYS_PER_YEAR = 365;

let state = {
    employee_count: 0,
    days_of_violation: 0,
    violation_type: 'individual_market_hra',
    is_small_employer: false,
    is_self_insured: false,
    is_qsehra: false,
    is_ichra: false,
    notice_provided: true,
    affordability_compliant: true,
};

export async function renderSection4980d(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s4980d.h1.title">// § 4980D HRA / GROUP HEALTH EXCISE</span></h1>
        <p class="muted small" data-i18n="view.s4980d.hint.intro">
            <strong>$100/day per affected individual</strong> = up to $36,500/employee/year for
            ACA group market reform violations. Common: <strong>pre-2020 HRA</strong> paying
            individual-market premiums (Notice 2013-54), <strong>fee-for-service HRAs</strong>,
            <strong>ICHRAs</strong> without proper notices / classes. <strong>QSEHRA</strong>
            (small employer) safe under Cures Act. Annual report on Form 8928 if violation.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s4980d.h2.inputs">Inputs</h2>
            <form id="s4980d-form" class="inline-form">
                <label><span data-i18n="view.s4980d.label.employees">Affected employee count</span>
                    <input type="number" step="1" name="employee_count" value="${state.employee_count}"></label>
                <label><span data-i18n="view.s4980d.label.days">Days of violation</span>
                    <input type="number" step="1" name="days_of_violation" value="${state.days_of_violation}"></label>
                <label><span data-i18n="view.s4980d.label.violation_type">Violation type</span>
                    <select name="violation_type">
                        <option value="individual_market_hra">HRA paying individual market premium (no QSEHRA/ICHRA)</option>
                        <option value="fee_for_service_hra">Fee-for-service HRA (post-2020 prohibited)</option>
                        <option value="no_notice_ichra">ICHRA without 90-day notice</option>
                        <option value="ichra_class_violation">ICHRA class structure violation</option>
                        <option value="grandfather_lost">Grandfathered plan lost status</option>
                        <option value="mhpaea">Mental Health Parity (MHPAEA) violation</option>
                        <option value="qmcso">QMCSO violation</option>
                        <option value="other">Other group health plan violation</option>
                    </select>
                </label>
                <label><span data-i18n="view.s4980d.label.small">Small employer (&lt; 50 FTE)?</span>
                    <input type="checkbox" name="is_small_employer" ${state.is_small_employer ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4980d.label.self_insured">Self-insured plan?</span>
                    <input type="checkbox" name="is_self_insured" ${state.is_self_insured ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4980d.label.qsehra">QSEHRA election in place?</span>
                    <input type="checkbox" name="is_qsehra" ${state.is_qsehra ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4980d.label.ichra">ICHRA in place?</span>
                    <input type="checkbox" name="is_ichra" ${state.is_ichra ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4980d.label.notice">Required notice provided?</span>
                    <input type="checkbox" name="notice_provided" ${state.notice_provided ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4980d.label.affordable">Affordable + min value?</span>
                    <input type="checkbox" name="affordability_compliant" ${state.affordability_compliant ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s4980d.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s4980d-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4980d.h2.allowed_hra">Allowed HRA structures (post-2020)</h2>
            <ul class="muted small">
                <li data-i18n="view.s4980d.allowed.qsehra">QSEHRA: small employer (&lt; 50 FTE), $5,850/$11,800 (2024) caps, reimburses individual market</li>
                <li data-i18n="view.s4980d.allowed.ichra">ICHRA: any employer, structured by class (full-time, part-time, etc.), requires individual mkt coverage</li>
                <li data-i18n="view.s4980d.allowed.excepted">Excepted Benefit HRA: $2,100/yr cap, supplemental to group coverage</li>
                <li data-i18n="view.s4980d.allowed.integrated">Integrated HRA: pairs with employer group plan</li>
                <li data-i18n="view.s4980d.allowed.retiree">Retiree-only HRA: exempt from § 4980D when only covering ex-employees</li>
                <li data-i18n="view.s4980d.allowed.fsa">Health FSA: limited annual carryover + 2.5-month grace period</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4980d.h2.relief">§ 9831(d) / § 4980D(c)(4) penalty relief</h2>
            <ul class="muted small">
                <li data-i18n="view.s4980d.relief.reasonable">Reasonable cause + good-faith violations</li>
                <li data-i18n="view.s4980d.relief.30_days">Cured within 30 days of discovery</li>
                <li data-i18n="view.s4980d.relief.500k_cap">Annual cap: $500k or 10% of group plan aggregate health expenses</li>
                <li data-i18n="view.s4980d.relief.unintentional">Unintentional failure + first 30 days: no penalty</li>
                <li data-i18n="view.s4980d.relief.small_cap">Single-employer small business: $20k annual cap</li>
                <li data-i18n="view.s4980d.relief.de_minimis">De minimis HRA contributions excluded (≤ $1,800/yr / $150/mo)</li>
            </ul>
        </div>
    `;
    document.getElementById('s4980d-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.employee_count = Number(fd.get('employee_count')) || 0;
        state.days_of_violation = Number(fd.get('days_of_violation')) || 0;
        state.violation_type = fd.get('violation_type');
        state.is_small_employer = !!fd.get('is_small_employer');
        state.is_self_insured = !!fd.get('is_self_insured');
        state.is_qsehra = !!fd.get('is_qsehra');
        state.is_ichra = !!fd.get('is_ichra');
        state.notice_provided = !!fd.get('notice_provided');
        state.affordability_compliant = !!fd.get('affordability_compliant');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s4980d-output');
    if (!el) return;
    const cappedDays = Math.min(state.days_of_violation, DAYS_PER_YEAR);
    const grossPenalty = state.employee_count * PER_DAY_PENALTY * cappedDays;
    const smallCap = 20_000;
    const generalCap = 500_000;
    const cappedPenalty = state.is_small_employer
        ? Math.min(grossPenalty, smallCap)
        : Math.min(grossPenalty, generalCap);
    const qsehraExempt = state.is_qsehra && state.is_small_employer && state.notice_provided
        && (state.violation_type === 'individual_market_hra' || state.violation_type === 'no_notice_ichra');
    const ichraExempt = state.is_ichra && state.notice_provided
        && (state.violation_type === 'individual_market_hra');
    const finalPenalty = qsehraExempt || ichraExempt ? 0 : cappedPenalty;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s4980d.h2.result">Excise tax exposure</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s4980d.card.gross">Gross calculation</div>
                    <div class="value">$${grossPenalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s4980d.card.cap">Applicable cap</div>
                    <div class="value">$${(state.is_small_employer ? smallCap : generalCap).toLocaleString()}</div>
                </div>
                <div class="card ${qsehraExempt ? 'pos' : ''}">
                    <div class="label" data-i18n="view.s4980d.card.qsehra">QSEHRA safe harbor</div>
                    <div class="value">${qsehraExempt ? esc(t('view.s4980d.status.yes')) : esc(t('view.s4980d.status.no'))}</div>
                </div>
                <div class="card ${ichraExempt ? 'pos' : ''}">
                    <div class="label" data-i18n="view.s4980d.card.ichra">ICHRA safe harbor</div>
                    <div class="value">${ichraExempt ? esc(t('view.s4980d.status.yes')) : esc(t('view.s4980d.status.no'))}</div>
                </div>
                <div class="card ${finalPenalty > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s4980d.card.final">Final penalty</div>
                    <div class="value">$${finalPenalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
