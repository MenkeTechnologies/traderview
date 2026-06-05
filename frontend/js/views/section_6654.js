// § 6654 Estimated Tax Underpayment Penalty.
// Safe harbors: pay 90% current-year tax OR 100% prior year (110% if AGI > $150k).
// Quarterly due: Apr 15 (Q1), Jun 15 (Q2), Sep 15 (Q3), Jan 15 next year (Q4).
// Penalty = federal short-term AFR + 3% on each quarter's underpayment.
// Annualized income method (Form 2210 Schedule AI) for lumpy income.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const HIGH_INCOME_THRESHOLD = 150_000;
const SAFE_HARBOR_LOW = 1.00;
const SAFE_HARBOR_HIGH = 1.10;
const SAFE_HARBOR_CURRENT = 0.90;
const PENALTY_RATE_PER_QUARTER = 0.08 / 4;  // approx

let state = {
    current_year_estimated_tax: 0,
    prior_year_total_tax: 0,
    prior_year_agi: 0,
    q1_paid: 0,
    q2_paid: 0,
    q3_paid: 0,
    q4_paid: 0,
    withholding: 0,
    is_farmer_fisherman: false,
};

export async function renderSection6654(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6654.h1.title">// § 6654 ESTIMATED TAX SAFE HARBOR</span></h1>
        <p class="muted small" data-i18n="view.s6654.hint.intro">
            Safe harbors avoid underpayment penalty: <strong>(1) 90% of current-year tax</strong> OR
            <strong>(2) 100% of prior-year tax (110% if prior AGI &gt; $150k)</strong>. Quarterly
            due: <strong>Apr 15, Jun 15, Sep 15, Jan 15</strong>. Withholding is treated as paid
            evenly across the year. Penalty = federal short-term AFR + 3% per quarter on each
            quarter's shortage. Farmers / fishermen: 66.67% safe harbor + Jan 15 single-date pay.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6654.h2.inputs">Inputs</h2>
            <form id="s6654-form" class="inline-form">
                <label><span data-i18n="view.s6654.label.current_est">Current-year tax (est) ($)</span>
                    <input type="number" step="0.01" name="current_year_estimated_tax" value="${state.current_year_estimated_tax}"></label>
                <label><span data-i18n="view.s6654.label.prior_tax">Prior-year total tax ($)</span>
                    <input type="number" step="0.01" name="prior_year_total_tax" value="${state.prior_year_total_tax}"></label>
                <label><span data-i18n="view.s6654.label.prior_agi">Prior-year AGI ($)</span>
                    <input type="number" step="0.01" name="prior_year_agi" value="${state.prior_year_agi}"></label>
                <label><span data-i18n="view.s6654.label.q1">Q1 paid ($)</span>
                    <input type="number" step="0.01" name="q1_paid" value="${state.q1_paid}"></label>
                <label><span data-i18n="view.s6654.label.q2">Q2 paid ($)</span>
                    <input type="number" step="0.01" name="q2_paid" value="${state.q2_paid}"></label>
                <label><span data-i18n="view.s6654.label.q3">Q3 paid ($)</span>
                    <input type="number" step="0.01" name="q3_paid" value="${state.q3_paid}"></label>
                <label><span data-i18n="view.s6654.label.q4">Q4 paid ($)</span>
                    <input type="number" step="0.01" name="q4_paid" value="${state.q4_paid}"></label>
                <label><span data-i18n="view.s6654.label.withholding">Total withholding ($)</span>
                    <input type="number" step="0.01" name="withholding" value="${state.withholding}"></label>
                <label><span data-i18n="view.s6654.label.farmer">Farmer / fisherman?</span>
                    <input type="checkbox" name="is_farmer_fisherman" ${state.is_farmer_fisherman ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s6654.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6654-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6654.h2.due_dates">Quarterly due dates</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s6654.th.quarter">Quarter</th>
                    <th data-i18n="view.s6654.th.period">Income period</th>
                    <th data-i18n="view.s6654.th.due">Due date</th>
                </tr></thead>
                <tbody>
                    <tr><td>Q1</td><td>Jan 1 - Mar 31</td><td>Apr 15</td></tr>
                    <tr><td>Q2</td><td>Apr 1 - May 31</td><td>Jun 15</td></tr>
                    <tr><td>Q3</td><td>Jun 1 - Aug 31</td><td>Sep 15</td></tr>
                    <tr><td>Q4</td><td>Sep 1 - Dec 31</td><td>Jan 15 (next year)</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6654.h2.workarounds">Penalty mitigation</h2>
            <ul class="muted small">
                <li data-i18n="view.s6654.work.withhold_dec">Increase W-2 withholding in December — treated as paid evenly across year</li>
                <li data-i18n="view.s6654.work.annualized">Form 2210 Schedule AI: annualized income method for lumpy / Q4-loaded income</li>
                <li data-i18n="view.s6654.work.january">January 15 final due date — pay shortage then</li>
                <li data-i18n="view.s6654.work.farmer">Farmer / fisherman: pay 66.67% by Jan 15 OR full tax by Mar 1, no quarterly</li>
                <li data-i18n="view.s6654.work.first_year">First-year filer with no prior-year liability: no penalty</li>
                <li data-i18n="view.s6654.work.casualty">Disaster / casualty / unusual: § 6654(e)(3) waiver</li>
                <li data-i18n="view.s6654.work.under_1000">Tax under $1,000 owed: no penalty</li>
            </ul>
        </div>
    `;
    document.getElementById('s6654-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.current_year_estimated_tax = Number(fd.get('current_year_estimated_tax')) || 0;
        state.prior_year_total_tax = Number(fd.get('prior_year_total_tax')) || 0;
        state.prior_year_agi = Number(fd.get('prior_year_agi')) || 0;
        state.q1_paid = Number(fd.get('q1_paid')) || 0;
        state.q2_paid = Number(fd.get('q2_paid')) || 0;
        state.q3_paid = Number(fd.get('q3_paid')) || 0;
        state.q4_paid = Number(fd.get('q4_paid')) || 0;
        state.withholding = Number(fd.get('withholding')) || 0;
        state.is_farmer_fisherman = !!fd.get('is_farmer_fisherman');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6654-output');
    if (!el) return;
    const useHighSafe = state.prior_year_agi > HIGH_INCOME_THRESHOLD;
    const priorSafeHarbor = state.prior_year_total_tax * (useHighSafe ? SAFE_HARBOR_HIGH : SAFE_HARBOR_LOW);
    const currentSafeHarbor = state.current_year_estimated_tax * (state.is_farmer_fisherman ? 0.6667 : SAFE_HARBOR_CURRENT);
    const requiredTotal = Math.min(priorSafeHarbor, currentSafeHarbor);
    const requiredPerQuarter = requiredTotal / 4;
    const wh_per_q = state.withholding / 4;
    const quarters = [
        { q: 1, paid: state.q1_paid + wh_per_q, due: requiredPerQuarter, quartersOpen: 4 },
        { q: 2, paid: state.q2_paid + wh_per_q, due: requiredPerQuarter, quartersOpen: 3 },
        { q: 3, paid: state.q3_paid + wh_per_q, due: requiredPerQuarter, quartersOpen: 2 },
        { q: 4, paid: state.q4_paid + wh_per_q, due: requiredPerQuarter, quartersOpen: 1 },
    ];
    let cumPaid = 0, cumRequired = 0, totalPenalty = 0;
    const quarterRows = quarters.map(r => {
        cumPaid += r.paid;
        cumRequired += r.due;
        const shortage = Math.max(0, cumRequired - cumPaid);
        const quarterPenalty = shortage * PENALTY_RATE_PER_QUARTER * r.quartersOpen;
        totalPenalty += quarterPenalty;
        return { ...r, cumPaid, cumRequired, shortage, quarterPenalty };
    });
    const totalPaid = state.q1_paid + state.q2_paid + state.q3_paid + state.q4_paid + state.withholding;
    const meetsRequirement = totalPaid >= requiredTotal;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6654.h2.result">Safe harbor</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s6654.card.prior_safe">Prior-year safe harbor</div>
                    <div class="value">$${priorSafeHarbor.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6654.card.current_safe">Current-year safe harbor</div>
                    <div class="value">$${currentSafeHarbor.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6654.card.high_income">High-income (110%)?</div>
                    <div class="value">${useHighSafe ? esc(t('view.s6654.status.yes')) : esc(t('view.s6654.status.no'))}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s6654.card.required">Required total payments</div>
                    <div class="value">$${requiredTotal.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6654.card.paid">Total paid</div>
                    <div class="value">$${totalPaid.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${meetsRequirement ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s6654.card.meets">Meets safe harbor</div>
                    <div class="value">${meetsRequirement ? esc(t('view.s6654.status.yes')) : esc(t('view.s6654.status.no'))}</div>
                </div>
                <div class="card ${totalPenalty > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s6654.card.penalty">Est. underpayment penalty</div>
                    <div class="value">$${totalPenalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6654.h2.quarter_table">Quarter-by-quarter</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s6654.th.q">Q</th>
                    <th data-i18n="view.s6654.th.required">Required cum</th>
                    <th data-i18n="view.s6654.th.paid_cum">Paid cum</th>
                    <th data-i18n="view.s6654.th.shortage">Shortage</th>
                    <th data-i18n="view.s6654.th.penalty">Penalty</th>
                </tr></thead>
                <tbody>${quarterRows.map(r => `
                    <tr>
                        <td>Q${r.q}</td>
                        <td>$${r.cumRequired.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                        <td>$${r.cumPaid.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                        <td class="${r.shortage > 0 ? 'neg' : 'pos'}">$${r.shortage.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                        <td class="${r.quarterPenalty > 0 ? 'neg' : ''}">$${r.quarterPenalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        </div>
    `;
}
