// IRC § 36B — Premium Tax Credit (PTC) for ACA Marketplace coverage.
// Refundable credit to reduce monthly health insurance premiums on Healthcare.gov / state exchanges.
// Pre-IRA cap: 400% FPL. ARPA / IRA 2022-2025: extended to ALL incomes when premium > 8.5% of MAGI.
// Reconciliation on Form 8962. Cliff effect pre-ARPA at 400% FPL — important to traders bunching gains.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const FPL_2024 = {
    1: 15_060, 2: 20_440, 3: 25_820, 4: 31_200, 5: 36_580,
    6: 41_960, 7: 47_340, 8: 52_720,
};

const APPLICABLE_PCT_2024 = [
    [1.00, 0],
    [1.50, 0],
    [2.00, 0.02],
    [2.50, 0.04],
    [3.00, 0.06],
    [4.00, 0.085],
    [Infinity, 0.085],
];

let state = {
    household_size: 1,
    magi: 0,
    benchmark_silver_annual: 0,
    actual_plan_annual: 0,
    months_enrolled: 12,
    advance_credits_received: 0,
};

export async function renderSection36b(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s36b.h1.title">// § 36B ACA PREMIUM TAX CREDIT</span></h1>
        <p class="muted small" data-i18n="view.s36b.hint.intro">
            Refundable credit to reduce monthly health insurance premiums on Healthcare.gov.
            <strong>Pre-IRA:</strong> available only 100-400% FPL (cliff). <strong>ARPA / IRA
            2022-2025:</strong> extended to ALL incomes when premium &gt; 8.5% of MAGI. Reconciled
            on <strong>Form 8962</strong>. Critical for self-employed traders — losing one year's
            APTC due to high gain costs thousands. <strong>2026 sunset:</strong> 400% FPL cliff returns.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s36b.h2.inputs">Inputs</h2>
            <form id="s36b-form" class="inline-form">
                <label><span data-i18n="view.s36b.label.household">Household size</span>
                    <input type="number" step="1" min="1" name="household_size" value="${state.household_size}"></label>
                <label><span data-i18n="view.s36b.label.magi">MAGI for PTC ($)</span>
                    <input type="number" step="1000" name="magi" value="${state.magi}"></label>
                <label><span data-i18n="view.s36b.label.benchmark">Benchmark silver plan annual cost ($)</span>
                    <input type="number" step="100" name="benchmark_silver_annual" value="${state.benchmark_silver_annual}"></label>
                <label><span data-i18n="view.s36b.label.actual">Actual plan annual cost ($)</span>
                    <input type="number" step="100" name="actual_plan_annual" value="${state.actual_plan_annual}"></label>
                <label><span data-i18n="view.s36b.label.months">Months enrolled</span>
                    <input type="number" step="1" min="1" max="12" name="months_enrolled" value="${state.months_enrolled}"></label>
                <label><span data-i18n="view.s36b.label.advance">Advance PTC received ($)</span>
                    <input type="number" step="100" name="advance_credits_received" value="${state.advance_credits_received}"></label>
                <button class="primary" type="submit" data-i18n="view.s36b.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s36b-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s36b.h2.fpl">2024 Federal Poverty Level (48 states + DC)</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s36b.th.size">Household size</th>
                    <th data-i18n="view.s36b.th.fpl">100% FPL</th>
                    <th data-i18n="view.s36b.th.400">400% FPL (post-2026 cliff)</th>
                </tr></thead>
                <tbody>
                    ${Object.entries(FPL_2024).map(([n, fpl]) => `
                        <tr>
                            <td>${n}</td>
                            <td>$${fpl.toLocaleString()}</td>
                            <td>$${(fpl * 4).toLocaleString()}</td>
                        </tr>
                    `).join('')}
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s36b.h2.planning">Trader-specific planning</h2>
            <ul class="muted small">
                <li data-i18n="view.s36b.plan.solo_401k">Solo 401(k) max contribution to drop MAGI below cliff</li>
                <li data-i18n="view.s36b.plan.hsa">HSA contributions (after-tax) also reduce MAGI</li>
                <li data-i18n="view.s36b.plan.harvesting">Loss harvest to offset realized gains pre-year-end</li>
                <li data-i18n="view.s36b.plan.roth_timing">Time Roth conversions for low-income years</li>
                <li data-i18n="view.s36b.plan.advance">If you'll exceed advance estimate, ACT NOW to drop MAGI</li>
                <li data-i18n="view.s36b.plan.cap_repayment">Pre-ARPA: repayment cap 200% FPL — limit repayment risk</li>
                <li data-i18n="view.s36b.plan.cliff_2026">2026 cliff returns: get below 400% FPL or lose entire subsidy</li>
            </ul>
        </div>
    `;
    document.getElementById('s36b-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.household_size = Number(fd.get('household_size')) || 1;
        state.magi = Number(fd.get('magi')) || 0;
        state.benchmark_silver_annual = Number(fd.get('benchmark_silver_annual')) || 0;
        state.actual_plan_annual = Number(fd.get('actual_plan_annual')) || 0;
        state.months_enrolled = Number(fd.get('months_enrolled')) || 12;
        state.advance_credits_received = Number(fd.get('advance_credits_received')) || 0;
        renderOutput();
    });
    renderOutput();
}

function fplForHousehold(size) {
    if (size <= 8) return FPL_2024[size];
    return FPL_2024[8] + (size - 8) * 5_380;
}

function applicablePct(ratio) {
    for (let i = APPLICABLE_PCT_2024.length - 1; i >= 0; i--) {
        if (ratio >= APPLICABLE_PCT_2024[i][0]) return APPLICABLE_PCT_2024[i][1];
    }
    if (ratio >= 1.50) {
        const t = (ratio - 1.50) / (4.00 - 1.50);
        return t * 0.085;
    }
    return APPLICABLE_PCT_2024[0][1];
}

function renderOutput() {
    const el = document.getElementById('s36b-output');
    if (!el) return;
    const fpl = fplForHousehold(state.household_size);
    const fplRatio = state.magi / fpl;
    const pct = applicablePct(fplRatio);
    const expectedContribution = state.magi * pct;
    const ptcMax = Math.max(0, state.benchmark_silver_annual - expectedContribution);
    const ptcEarned = Math.min(state.actual_plan_annual, ptcMax) * (state.months_enrolled / 12);
    const reconciliation = ptcEarned - state.advance_credits_received;
    const netOutOfPocket = state.actual_plan_annual - ptcEarned;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s36b.h2.result">PTC calculation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s36b.card.fpl">Your FPL</div>
                    <div class="value">$${fpl.toLocaleString()}</div>
                </div>
                <div class="card ${fplRatio >= 4.00 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s36b.card.fpl_ratio">FPL ratio</div>
                    <div class="value">${(fplRatio * 100).toFixed(0)}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s36b.card.applicable">Applicable %</div>
                    <div class="value">${(pct * 100).toFixed(2)}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s36b.card.expected">Expected contribution</div>
                    <div class="value">$${expectedContribution.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s36b.card.ptc">PTC earned</div>
                    <div class="value">$${ptcEarned.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${reconciliation < 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s36b.card.recon">Reconciliation</div>
                    <div class="value">$${reconciliation.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s36b.card.oop">Net out-of-pocket</div>
                    <div class="value">$${netOutOfPocket.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${reconciliation < 0 ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s36b.warning.repay">
                    Advance PTC exceeds earned PTC → you OWE the difference back on Form 8962.
                    Repayment caps apply &lt; 400% FPL. ≥ 400% FPL (pre-IRA) = full repayment.
                </p>
            ` : ''}
        </div>
    `;
}
