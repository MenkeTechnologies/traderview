// IRC § 414(v) — Catch-up Contributions + SECURE 2.0 Roth Mandate.
// Age 50+: $7,500 catch-up (2024) on top of $23k base 401(k).
// SECURE 2.0 Super Catch-up 60-63: $11,250 (2024).
// SECURE 2.0 Mandatory Roth Catch-up for high earners: > $145k 2023 wages → catch-up MUST be Roth (effective 2026 after delay).
// IRA catch-up: $1,000 (50+). SIMPLE catch-up: $3,500 / $5,250 super.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const CATCH_UP_50 = 7_500;
const CATCH_UP_60_63 = 11_250;
const IRA_CATCH_UP = 1_000;
const SIMPLE_CATCH_UP = 3_500;
const SIMPLE_SUPER_CATCH_UP = 5_250;
const ROTH_MANDATE_WAGE_THRESHOLD = 145_000;

let state = {
    age: 50,
    plan_type: '401k',
    prior_year_wages: 0,
    desired_catch_up: 0,
    has_employer_roth_option: true,
    marginal_rate: 0.32,
};

export async function renderSection414v(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s414v.h1.title">// § 414(v) CATCH-UP CONTRIBUTIONS</span></h1>
        <p class="muted small" data-i18n="view.s414v.hint.intro">
            Age 50+: <strong>$7,500 catch-up (2024)</strong> on top of $23k base 401(k).
            <strong>SECURE 2.0 Super Catch-up 60-63: $11,250</strong>. IRA: $1,000.
            <strong>SECURE 2.0 Mandatory Roth Catch-up</strong> for high earners (&gt; $145k 2023
            wages) → catch-up MUST be Roth (effective 2026 after delay). Plan must offer Roth
            option to provide catch-up to high earners.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s414v.h2.inputs">Inputs</h2>
            <form id="s414v-form" class="inline-form">
                <label><span data-i18n="view.s414v.label.age">Your age</span>
                    <input type="number" step="1" name="age" value="${state.age}"></label>
                <label><span data-i18n="view.s414v.label.plan">Plan type</span>
                    <select name="plan_type">
                        <option value="401k" ${state.plan_type === '401k' ? 'selected' : ''}>401(k)</option>
                        <option value="403b" ${state.plan_type === '403b' ? 'selected' : ''}>403(b)</option>
                        <option value="457b" ${state.plan_type === '457b' ? 'selected' : ''}>Government 457(b)</option>
                        <option value="ira_trad" ${state.plan_type === 'ira_trad' ? 'selected' : ''}>Traditional IRA</option>
                        <option value="ira_roth" ${state.plan_type === 'ira_roth' ? 'selected' : ''}>Roth IRA</option>
                        <option value="simple" ${state.plan_type === 'simple' ? 'selected' : ''}>SIMPLE IRA / 401(k)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s414v.label.wages">Prior year wages (for Roth mandate) ($)</span>
                    <input type="number" step="1000" name="prior_year_wages" value="${state.prior_year_wages}"></label>
                <label><span data-i18n="view.s414v.label.catch_up">Desired catch-up ($)</span>
                    <input type="number" step="500" name="desired_catch_up" value="${state.desired_catch_up}"></label>
                <label><span data-i18n="view.s414v.label.roth_option">Employer plan has Roth option?</span>
                    <input type="checkbox" name="has_employer_roth_option" ${state.has_employer_roth_option ? 'checked' : ''}></label>
                <label><span data-i18n="view.s414v.label.marginal">Marginal %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s414v.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s414v-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s414v.h2.catch_up_limits">Catch-up by plan type (2024)</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s414v.th.plan">Plan</th>
                    <th data-i18n="view.s414v.th.50">Age 50-59 / 64+</th>
                    <th data-i18n="view.s414v.th.60_63">Age 60-63 (SECURE 2.0)</th>
                </tr></thead>
                <tbody>
                    <tr><td>401(k) / 403(b) / Gov 457(b)</td><td>$7,500</td><td>$11,250</td></tr>
                    <tr><td>SIMPLE 401(k) / SIMPLE IRA</td><td>$3,500</td><td>$5,250</td></tr>
                    <tr><td>Traditional / Roth IRA</td><td>$1,000</td><td>$1,000 (no super)</td></tr>
                    <tr><td>SEP IRA</td><td>None separate (uses § 415 cap)</td><td>—</td></tr>
                    <tr><td>Defined Benefit § 415(b)</td><td>None separate</td><td>—</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s414v.h2.roth_mandate">SECURE 2.0 Mandatory Roth Catch-up</h2>
            <ul class="muted small">
                <li data-i18n="view.s414v.rm.threshold">High earners: prior-year wages &gt; $145,000 (2023+, inflation-indexed)</li>
                <li data-i18n="view.s414v.rm.must_be_roth">Catch-up MUST be Roth (post-tax) — no pre-tax option</li>
                <li data-i18n="view.s414v.rm.effective">Originally effective 2024; delayed by IRS Notice 2023-62 to 2026</li>
                <li data-i18n="view.s414v.rm.plan_offers">Plan must offer Roth option to provide catch-up to high earners</li>
                <li data-i18n="view.s414v.rm.no_option">No Roth option = no catch-up for high earners</li>
                <li data-i18n="view.s414v.rm.solo_401k">Solo 401(k): must add Roth source</li>
                <li data-i18n="view.s414v.rm.simple_ira">SIMPLE IRA: Roth option since SECURE 2.0</li>
                <li data-i18n="view.s414v.rm.wages_only">"Wages" = FICA wages from § 3121(a); SE income excluded</li>
                <li data-i18n="view.s414v.rm.first_year">First year of employment: no Roth mandate (no prior-year wages)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s414v.h2.benefits_strategy">Catch-up strategy</h2>
            <ul class="muted small">
                <li data-i18n="view.s414v.strat.late_career">Late-career catch-up: $11,250 × 4 yrs (60-63) = $45,000 SECURE 2.0 super</li>
                <li data-i18n="view.s414v.strat.roth_for_high">High earners with Roth catch-up: locks in tax-free growth at peak earnings</li>
                <li data-i18n="view.s414v.strat.no_match">Catch-up usually NOT matched by employer</li>
                <li data-i18n="view.s414v.strat.full_calendar">Catch-up fills last; max base first</li>
                <li data-i18n="view.s414v.strat.spouse">Spousal IRA catch-up: each spouse $1,000 separately</li>
                <li data-i18n="view.s414v.strat.gov_457b_separate">Government 457(b) catch-up SEPARATE from 401(k)/403(b) (can stack)</li>
                <li data-i18n="view.s414v.strat.special_457b">§ 457(b)(3) special last-3-year catch-up: 2× regular limit (use OR § 414(v) age 50, not both)</li>
            </ul>
        </div>
    `;
    document.getElementById('s414v-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.age = Number(fd.get('age')) || 50;
        state.plan_type = fd.get('plan_type');
        state.prior_year_wages = Number(fd.get('prior_year_wages')) || 0;
        state.desired_catch_up = Number(fd.get('desired_catch_up')) || 0;
        state.has_employer_roth_option = !!fd.get('has_employer_roth_option');
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.32;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s414v-output');
    if (!el) return;
    if (state.age < 50) {
        el.innerHTML = `<div class="chart-panel"><p class="muted small" data-i18n="view.s414v.note.under_50">Catch-up contributions only available age 50+.</p></div>`;
        return;
    }
    let maxCatchUp;
    if (state.plan_type === 'ira_trad' || state.plan_type === 'ira_roth') {
        maxCatchUp = IRA_CATCH_UP;
    } else if (state.plan_type === 'simple') {
        maxCatchUp = state.age >= 60 && state.age <= 63 ? SIMPLE_SUPER_CATCH_UP : SIMPLE_CATCH_UP;
    } else {
        maxCatchUp = state.age >= 60 && state.age <= 63 ? CATCH_UP_60_63 : CATCH_UP_50;
    }
    const allowedCatchUp = Math.min(state.desired_catch_up, maxCatchUp);
    const isHighEarner = state.prior_year_wages > ROTH_MANDATE_WAGE_THRESHOLD;
    const rothMandateApplies = isHighEarner && (state.plan_type === '401k' || state.plan_type === '403b' || state.plan_type === '457b');
    const blocked = rothMandateApplies && !state.has_employer_roth_option;
    const taxSavings = (rothMandateApplies || state.plan_type === 'ira_roth') ? 0 : allowedCatchUp * state.marginal_rate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s414v.h2.result">Catch-up analysis</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s414v.card.max">Max catch-up</div>
                    <div class="value">$${maxCatchUp.toLocaleString()}</div>
                </div>
                <div class="card ${blocked ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s414v.card.allowed">Allowed</div>
                    <div class="value">$${(blocked ? 0 : allowedCatchUp).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${rothMandateApplies ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s414v.card.roth_mandate">Roth catch-up mandate</div>
                    <div class="value">${rothMandateApplies ? esc(t('view.s414v.status.yes')) : esc(t('view.s414v.status.no'))}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s414v.card.savings">Year-1 tax savings (if pre-tax)</div>
                    <div class="value">$${taxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${blocked ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s414v.warning.blocked">
                    High earner but employer plan doesn't offer Roth — NO catch-up available
                    starting 2026 unless plan adds Roth option.
                </p>
            ` : ''}
        </div>
    `;
}
