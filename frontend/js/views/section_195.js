// IRC § 195 Start-Up Cost Amortization.
// $5,000 immediate deduction in year of opening; rest amortized straight-line over 180 months.
// Phase-out: $5k cap reduced $1-for-$1 starting at $50,000 total start-up; zero at $55,000.
// § 248 (organizational) follows same rule for corps/partnerships.

import { currentViewToken, viewIsCurrent } from '../app.js';

const IMMEDIATE_DEDUCTION_CAP = 5_000;
const PHASE_OUT_START = 50_000;
const PHASE_OUT_END = 55_000;
const AMORT_MONTHS = 180;  // 15 years

let state = {
    total_startup_costs: 0,
    total_org_costs: 0,
    opening_year: new Date().getFullYear(),
    opening_month: 1,
    marginal_rate: 0.32,
};

export async function renderSection195(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s195.h1.title">// § 195 START-UP COST AMORTIZATION</span></h1>
        <p class="muted small" data-i18n="view.s195.hint.intro">
            <strong>$5,000 immediate deduction</strong> in year of business opening + remainder
            amortized over <strong>180 months</strong> (15 years). Phase-out: $5k cap reduces
            $1-for-$1 starting at $50k total expenses; gone at $55k. § 248 follows same rule
            for organizational costs (legal fees to form C-corp / partnership).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s195.h2.qualifying">Qualifying start-up costs</h2>
            <ul class="muted small">
                <li data-i18n="view.s195.qual.market">Market research, surveys, feasibility studies</li>
                <li data-i18n="view.s195.qual.advertising">Advertising prior to opening</li>
                <li data-i18n="view.s195.qual.salaries">Salaries / wages paid before opening</li>
                <li data-i18n="view.s195.qual.travel">Travel + consulting fees to secure suppliers, customers</li>
                <li data-i18n="view.s195.qual.training">Training of new employees</li>
                <li data-i18n="view.s195.qual.legal">Legal / accounting fees BEFORE opening (not § 248 org)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s195.h2.not_qualifying">NOT qualifying</h2>
            <ul class="muted small">
                <li data-i18n="view.s195.nonqual.equipment">Equipment + machinery (capitalize per § 168 MACRS)</li>
                <li data-i18n="view.s195.nonqual.inventory">Inventory (per § 263A)</li>
                <li data-i18n="view.s195.nonqual.investigating">Investigating but NOT starting (only if business not pursued)</li>
                <li data-i18n="view.s195.nonqual.section_197">§ 197 intangibles (goodwill, customer list — separate 15-yr amort)</li>
                <li data-i18n="view.s195.nonqual.expansion">Expansion of existing business (deduct currently)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s195.h2.inputs">Inputs</h2>
            <form id="s195-form" class="inline-form">
                <label><span data-i18n="view.s195.label.startup_costs">Total start-up costs ($)</span>
                    <input type="number" step="100" name="total_startup_costs" value="${state.total_startup_costs}"></label>
                <label><span data-i18n="view.s195.label.org_costs">Total § 248 organizational costs ($)</span>
                    <input type="number" step="100" name="total_org_costs" value="${state.total_org_costs}"></label>
                <label><span data-i18n="view.s195.label.opening_year">Business opening year</span>
                    <input type="number" step="1" name="opening_year" value="${state.opening_year}"></label>
                <label><span data-i18n="view.s195.label.opening_month">Opening month (1-12)</span>
                    <input type="number" step="1" min="1" max="12" name="opening_month" value="${state.opening_month}"></label>
                <label><span data-i18n="view.s195.label.marginal">Marginal tax %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s195.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s195-output"></div>
    `;
    document.getElementById('s195-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.total_startup_costs = Number(fd.get('total_startup_costs')) || 0;
        state.total_org_costs = Number(fd.get('total_org_costs')) || 0;
        state.opening_year = Number(fd.get('opening_year')) || new Date().getFullYear();
        state.opening_month = Math.max(1, Math.min(12, Number(fd.get('opening_month')) || 1));
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.32;
        renderOutput();
    });
    renderOutput();
}

function computeRule(total) {
    const reduction = Math.max(0, total - PHASE_OUT_START);
    const cap = Math.max(0, IMMEDIATE_DEDUCTION_CAP - reduction);
    const immediate = Math.min(total, cap);
    const remainder = Math.max(0, total - immediate);
    const monthly = remainder / AMORT_MONTHS;
    return { immediate, remainder, monthly };
}

function renderOutput() {
    const el = document.getElementById('s195-output');
    if (!el) return;
    const startup = computeRule(state.total_startup_costs);
    const org = computeRule(state.total_org_costs);
    const totalImmediate = startup.immediate + org.immediate;
    const monthsRemainingY1 = 13 - state.opening_month;
    const y1AmortStartup = startup.monthly * monthsRemainingY1;
    const y1AmortOrg = org.monthly * monthsRemainingY1;
    const totalY1Deduction = totalImmediate + y1AmortStartup + y1AmortOrg;
    const y1Savings = totalY1Deduction * state.marginal_rate;
    const totalLifetimeSavings = (state.total_startup_costs + state.total_org_costs) * state.marginal_rate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s195.h2.result">Year-1 calculation</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.s195.card.startup_immediate">§ 195 immediate</div>
                    <div class="value">$${startup.immediate.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s195.card.startup_amort">§ 195 monthly amort</div>
                    <div class="value">$${startup.monthly.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s195.card.org_immediate">§ 248 immediate</div>
                    <div class="value">$${org.immediate.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s195.card.org_amort">§ 248 monthly amort</div>
                    <div class="value">$${org.monthly.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s195.card.y1_deduction">Year-1 total deduction</div>
                    <div class="value">$${totalY1Deduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s195.card.y1_savings">Year-1 tax savings</div>
                    <div class="value">$${y1Savings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s195.card.lifetime">Lifetime tax savings</div>
                    <div class="value">$${totalLifetimeSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.total_startup_costs > PHASE_OUT_START ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s195.warning.phaseout">
                    Total start-up costs exceed $50,000 phase-out threshold. Immediate deduction reduced $1-for-$1.
                    Above $55,000 the entire $5k cap is gone — all costs go to 180-month amortization.
                </p>
            ` : ''}
        </div>
    `;
}
