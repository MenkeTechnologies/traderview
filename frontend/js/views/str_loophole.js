// Short-Term Rental Tax Loophole — Reg. § 1.469-1T(e)(3)(ii)(A).
// If average guest stay ≤ 7 days, the property is NOT a "rental activity"
// under § 469 — it's a trade/business. Material participation makes losses
// fully deductible against ORDINARY income (vs PAL-suspended).
// 7-test material participation. Most STR owners pass test #3: 100+ hrs and
// more than anyone else.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    average_stay_days: 4,
    nightly_rate: 200,
    annual_revenue: 80_000,
    operating_expenses: 30_000,
    depreciation: 35_000,  // cost-seg accelerated
    your_hours: 120,
    other_largest_hours: 80,
    other_total_hours: 150,
    your_other_business_hours: 0,
    marginal_rate: 0.32,
    state_rate: 0.05,
};

export async function renderStrLoophole(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.str.h1.title">// SHORT-TERM RENTAL LOOPHOLE</span></h1>
        <p class="muted small" data-i18n="view.str.hint.intro">
            <strong>Reg. § 1.469-1T(e)(3)(ii)(A):</strong> if average guest stay ≤ 7 days,
            the property is NOT a "rental activity" under § 469 — it's a trade/business.
            Material participation makes losses fully deductible against ORDINARY income
            (vs PAL-suspended). Combined with cost-seg + bonus dep = massive year-1
            paper loss that shelters W-2 / trading income. Heavily audited.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.str.h2.inputs">Inputs</h2>
            <form id="str-form" class="inline-form">
                <label><span data-i18n="view.str.label.average_stay_days">Average guest stay (days)</span>
                    <input type="number" step="0.1" name="average_stay_days" value="${state.average_stay_days}"></label>
                <label><span data-i18n="view.str.label.nightly_rate">Nightly rate ($)</span>
                    <input type="number" step="10" name="nightly_rate" value="${state.nightly_rate}"></label>
                <label><span data-i18n="view.str.label.annual_revenue">Annual revenue ($)</span>
                    <input type="number" step="1000" name="annual_revenue" value="${state.annual_revenue}"></label>
                <label><span data-i18n="view.str.label.operating_expenses">Operating expenses ($)</span>
                    <input type="number" step="500" name="operating_expenses" value="${state.operating_expenses}"></label>
                <label><span data-i18n="view.str.label.depreciation">Depreciation incl. cost-seg ($)</span>
                    <input type="number" step="1000" name="depreciation" value="${state.depreciation}"></label>
                <label><span data-i18n="view.str.label.your_hours">Your hours on the property</span>
                    <input type="number" step="1" name="your_hours" value="${state.your_hours}"></label>
                <label><span data-i18n="view.str.label.other_largest_hours">Largest non-owner contributor hours</span>
                    <input type="number" step="1" name="other_largest_hours" value="${state.other_largest_hours}"></label>
                <label><span data-i18n="view.str.label.other_total_hours">All non-owner contributors total</span>
                    <input type="number" step="1" name="other_total_hours" value="${state.other_total_hours}"></label>
                <label><span data-i18n="view.str.label.your_other_business_hours">Your other business hours</span>
                    <input type="number" step="1" name="your_other_business_hours" value="${state.your_other_business_hours}"></label>
                <label><span data-i18n="view.str.label.marginal_rate">Marginal federal %</span>
                    <input type="number" step="0.5" name="marginal_rate" value="${(state.marginal_rate * 100).toFixed(1)}"></label>
                <label><span data-i18n="view.str.label.state_rate">State %</span>
                    <input type="number" step="0.5" name="state_rate" value="${(state.state_rate * 100).toFixed(1)}"></label>
                <button class="primary" type="submit" data-i18n="view.str.btn.compute">Compute</button>
            </form>
        </div>
        <div id="str-output"></div>
    `;
    document.getElementById('str-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.average_stay_days = Number(fd.get('average_stay_days')) || 0;
        state.nightly_rate = Number(fd.get('nightly_rate')) || 0;
        state.annual_revenue = Number(fd.get('annual_revenue')) || 0;
        state.operating_expenses = Number(fd.get('operating_expenses')) || 0;
        state.depreciation = Number(fd.get('depreciation')) || 0;
        state.your_hours = Number(fd.get('your_hours')) || 0;
        state.other_largest_hours = Number(fd.get('other_largest_hours')) || 0;
        state.other_total_hours = Number(fd.get('other_total_hours')) || 0;
        state.your_other_business_hours = Number(fd.get('your_other_business_hours')) || 0;
        state.marginal_rate = (Number(fd.get('marginal_rate')) || 32) / 100;
        state.state_rate = (Number(fd.get('state_rate')) || 0) / 100;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('str-output');
    if (!el) return;
    const qualifies7day = state.average_stay_days <= 7;
    // Material participation tests
    const test1 = state.your_hours > 500;
    const test2 = state.your_hours > state.other_total_hours;
    const test3 = state.your_hours >= 100 && state.your_hours > state.other_largest_hours;
    const test4 = state.your_hours >= 100 && state.your_hours + state.your_other_business_hours > 500;
    // (5, 6, 7 are advanced — skip)
    const matParticipates = test1 || test2 || test3 || test4;
    const netLoss = state.annual_revenue - state.operating_expenses - state.depreciation;
    const taxSavings = matParticipates && qualifies7day && netLoss < 0
        ? Math.abs(netLoss) * (state.marginal_rate + state.state_rate)
        : 0;
    const cls = qualifies7day && matParticipates ? 'pos' : 'neg';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.str.h2.qualification">Qualification</h2>
            <div class="cards">
                <div class="card ${qualifies7day ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.str.card.7_day">≤ 7-day average stay</div>
                    <div class="value">${qualifies7day ? esc(t('view.str.status.yes')) : esc(t('view.str.status.no'))}</div>
                </div>
                <div class="card ${matParticipates ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.str.card.mat_part">Material participation</div>
                    <div class="value">${matParticipates ? esc(t('view.str.status.yes')) : esc(t('view.str.status.no'))}</div>
                </div>
                <div class="card ${cls}">
                    <div class="label" data-i18n="view.str.card.qualifies">Loophole applies?</div>
                    <div class="value">${(qualifies7day && matParticipates) ? esc(t('view.str.status.yes')) : esc(t('view.str.status.no'))}</div>
                </div>
            </div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.str.h2.tests">Material participation tests (any 1)</h2>
            <table class="trades"><tbody>
                <tr><td data-i18n="view.str.test.1">Test 1: &gt; 500 hours in activity</td>
                    <td class="${test1 ? 'pos' : 'muted'}">${test1 ? '✓ PASS' : '×'}</td></tr>
                <tr><td data-i18n="view.str.test.2">Test 2: Your hours &gt; all others combined</td>
                    <td class="${test2 ? 'pos' : 'muted'}">${test2 ? '✓ PASS' : '×'}</td></tr>
                <tr><td data-i18n="view.str.test.3">Test 3: 100+ hours AND more than anyone else</td>
                    <td class="${test3 ? 'pos' : 'muted'}">${test3 ? '✓ PASS' : '×'}</td></tr>
                <tr><td data-i18n="view.str.test.4">Test 4: 100+ hrs + total all activities &gt; 500 hrs</td>
                    <td class="${test4 ? 'pos' : 'muted'}">${test4 ? '✓ PASS' : '×'}</td></tr>
            </tbody></table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.str.h2.outcome">Tax outcome</h2>
            <div class="cards">
                <div class="card ${netLoss < 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.str.card.net_pnl">Net P&L</div>
                    <div class="value">$${netLoss.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.str.card.deductible">Deductible against ordinary?</div>
                    <div class="value">${(qualifies7day && matParticipates && netLoss < 0) ? '$' + Math.abs(netLoss).toLocaleString() : '$0'}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.str.card.tax_savings">Tax savings</div>
                    <div class="value">$${taxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            <p class="muted small" style="margin-top:10px" data-i18n="view.str.audit_note">
                AUDIT RISK: this is one of the most heavily audited tax positions. Keep
                CONTEMPORANEOUS time logs (date, hours, task description). IRS has won
                multiple recent cases (Coleman, Sezonov) on insufficient logs. Use Stessa /
                REI Hub / time-tracking app. Cost-seg + STR loophole + W-2 income = the
                "lazy 1040" strategy that shelters six-figure W-2 income.
            </p>
        </div>
    `;
}
