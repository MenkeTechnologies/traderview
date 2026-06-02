// IRC § 7508A — Disaster + Emergency Tax Deadline Postponement.
// IRS announces postponement after FEMA disaster declaration. Up to 1 year.
// Affects filing, payment, refund claim deadlines.
// Also COVID-19 (§ 7508A(d) declared 2020-2021). Combat zones (§ 7508).
// Check IRS.gov "Tax Relief in Disaster Situations" page.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const RECENT_DISASTERS_2024 = [
    { name: 'Hurricane Helene', states: 'AL, FL, GA, NC, SC, TN, VA', postponed_to: '2025-05-01' },
    { name: 'Hurricane Milton', states: 'FL', postponed_to: '2025-05-01' },
    { name: 'Hurricane Debby', states: 'FL, GA, NC, SC, VT', postponed_to: '2025-02-03' },
    { name: 'CA Wildfires (Los Angeles)', states: 'CA', postponed_to: '2025-10-15' },
    { name: 'CA Winter Storms', states: 'CA', postponed_to: '2024-06-17' },
    { name: 'Severe Storms TX', states: 'TX', postponed_to: '2025-02-03' },
    { name: 'Severe Storms OK', states: 'OK', postponed_to: '2025-02-03' },
];

let state = {
    affected_county_or_state: '',
    relevant_disaster: '',
    deadline_kind: 'individual_1040',
    original_deadline: '',
    days_postponed: 0,
    interest_accruing: 0,
};

export async function renderSection7508a(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s7508a.h1.title">// § 7508A DISASTER POSTPONEMENT</span></h1>
        <p class="muted small" data-i18n="view.s7508a.hint.intro">
            IRS announces postponement after FEMA disaster declaration. Up to 1 year. Postpones
            filing, payment, refund claim deadlines. <strong>Also COVID-19 (§ 7508A(d)
            2020-2021)</strong>. <strong>Combat zones (§ 7508)</strong> separate provision.
            <strong>Automatic for taxpayers in affected county</strong> — no action required, but
            put disaster name on top of return. Late filing penalties + interest NOT charged
            during postponement.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s7508a.h2.inputs">Inputs</h2>
            <form id="s7508a-form" class="inline-form">
                <label><span data-i18n="view.s7508a.label.county">Your county / state</span>
                    <input type="text" name="affected_county_or_state" value="${state.affected_county_or_state}" placeholder="e.g. Asheville, NC"></label>
                <label><span data-i18n="view.s7508a.label.disaster">Disaster</span>
                    <select name="relevant_disaster">
                        <option value="">— select —</option>
                        ${RECENT_DISASTERS_2024.map(d => `<option value="${d.name}">${d.name} (${d.states})</option>`).join('')}
                        <option value="other">Other / Not listed</option>
                    </select>
                </label>
                <label><span data-i18n="view.s7508a.label.deadline_kind">Deadline kind</span>
                    <select name="deadline_kind">
                        <option value="individual_1040">Form 1040 individual return</option>
                        <option value="estimated_q1">Q1 estimated tax</option>
                        <option value="estimated_q2">Q2 estimated tax</option>
                        <option value="estimated_q3">Q3 estimated tax</option>
                        <option value="business_1120s">Form 1120-S corporate return</option>
                        <option value="partnership_1065">Form 1065 partnership return</option>
                        <option value="payroll_941">Form 941 payroll</option>
                        <option value="excise_720">Form 720 excise</option>
                        <option value="refund_claim">Refund claim filing</option>
                        <option value="other">Other</option>
                    </select>
                </label>
                <label><span data-i18n="view.s7508a.label.original">Original deadline</span>
                    <input type="date" name="original_deadline" value="${state.original_deadline}"></label>
                <label><span data-i18n="view.s7508a.label.days_postponed">Days postponed</span>
                    <input type="number" step="1" name="days_postponed" value="${state.days_postponed}"></label>
                <label><span data-i18n="view.s7508a.label.interest">Daily interest accruing ($)</span>
                    <input type="number" step="0.01" name="interest_accruing" value="${state.interest_accruing}"></label>
                <button class="primary" type="submit" data-i18n="view.s7508a.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s7508a-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7508a.h2.recent">2024 disasters with extended deadlines</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s7508a.th.disaster">Disaster</th>
                    <th data-i18n="view.s7508a.th.states">States</th>
                    <th data-i18n="view.s7508a.th.postponed">Postponed to</th>
                </tr></thead>
                <tbody>${RECENT_DISASTERS_2024.map(d => `
                    <tr>
                        <td>${esc(d.name)}</td>
                        <td class="muted">${esc(d.states)}</td>
                        <td>${esc(d.postponed_to)}</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7508a.h2.what_postponed">What § 7508A postpones</h2>
            <ul class="muted small">
                <li data-i18n="view.s7508a.what.filing">Filing Form 1040 / 1041 / 1065 / 1120 / 1120-S / 990</li>
                <li data-i18n="view.s7508a.what.payment">Tax payment (no late-pay penalty)</li>
                <li data-i18n="view.s7508a.what.refund">Refund claim filing (§ 6511 SOL)</li>
                <li data-i18n="view.s7508a.what.tax_court">Tax Court petition (90-day letter)</li>
                <li data-i18n="view.s7508a.what.cdp">CDP hearing request</li>
                <li data-i18n="view.s7508a.what.estimated">Quarterly estimated tax</li>
                <li data-i18n="view.s7508a.what.401k_contribution">SECURE 2.0 401(k) catch-up contribution</li>
                <li data-i18n="view.s7508a.what.ira_contribution">IRA contribution for previous year</li>
                <li data-i18n="view.s7508a.what.casualty_relief">§ 165(i) election to claim casualty loss in prior year</li>
                <li data-i18n="view.s7508a.what.no_interest">No interest accrues during postponement on tax owed</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7508a.h2.combat">§ 7508 Combat zone provisions</h2>
            <p class="muted small" data-i18n="view.s7508a.combat.body">
                Service in combat zone: <strong>180-day extension</strong> + period of service
                + period in hospital. Spouses of military in combat zone: same extension. No
                interest charged. Direct contact: Form 9000 / IRS Combat Zone Hotline.
                Spouse + dependents file under combatant's protection.
            </p>
        </div>
    `;
    document.getElementById('s7508a-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.affected_county_or_state = fd.get('affected_county_or_state') || '';
        state.relevant_disaster = fd.get('relevant_disaster') || '';
        state.deadline_kind = fd.get('deadline_kind');
        state.original_deadline = fd.get('original_deadline');
        state.days_postponed = Number(fd.get('days_postponed')) || 0;
        state.interest_accruing = Number(fd.get('interest_accruing')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s7508a-output');
    if (!el) return;
    let postponedDate = '';
    if (state.original_deadline && state.days_postponed > 0) {
        const d = new Date(state.original_deadline);
        d.setDate(d.getDate() + state.days_postponed);
        postponedDate = d.toISOString().slice(0, 10);
    }
    const matchedDisaster = RECENT_DISASTERS_2024.find(d => d.name === state.relevant_disaster);
    const interestSaved = state.interest_accruing * state.days_postponed;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s7508a.h2.result">Postponement summary</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.s7508a.card.new_deadline">New deadline</div>
                    <div class="value">${esc(postponedDate || (matchedDisaster ? matchedDisaster.postponed_to : '—'))}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s7508a.card.days_extension">Days of extension</div>
                    <div class="value">${state.days_postponed}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s7508a.card.interest_saved">Interest savings</div>
                    <div class="value">$${interestSaved.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s7508a.card.write_top">Write disaster name on return</div>
                    <div class="value">${esc(state.relevant_disaster || '—')}</div>
                </div>
            </div>
        </div>
    `;
}
