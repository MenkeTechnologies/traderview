// IRC § 416 — Top-Heavy Plan Rules.
// Plan is top-heavy if KEY EMPLOYEES own > 60% of plan balance (vesting acceleration triggers).
// Key employee: officer earning > $220k (2024), > 5% owner, OR > 1% owner earning > $150k.
// Minimum contribution requirement for non-key employees: 3% (or top-heavy match for some plans).
// Faster vesting: 3-year cliff OR 2-6 year graded.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const KEY_EMPLOYEE_OFFICER_THRESHOLD = 220_000;
const KEY_EMPLOYEE_1PCT_THRESHOLD = 150_000;
const TOP_HEAVY_THRESHOLD_PCT = 0.60;
const MIN_CONTRIBUTION_PCT = 0.03;

let state = {
    key_employee_balance: 0,
    non_key_employee_balance: 0,
    key_employee_count: 0,
    non_key_employee_count: 0,
    avg_non_key_compensation: 0,
    has_safe_harbor: false,
    is_minimum_vesting: true,
    plan_includes_defined_benefit: false,
    marginal_rate: 0.21,
};

export async function renderSection416(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s416.h1.title">// § 416 TOP-HEAVY PLAN</span></h1>
        <p class="muted small" data-i18n="view.s416.hint.intro">
            Plan is <strong>top-heavy if KEY EMPLOYEES own &gt; 60% of plan balance</strong>.
            <strong>Key employee:</strong> officer earning &gt; $220k (2024), &gt; 5% owner,
            OR &gt; 1% owner earning &gt; $150k. <strong>Top-heavy minimum:</strong> 3% non-key
            employee contribution. <strong>Faster vesting:</strong> 3-year cliff OR 2-6 year
            graded. <strong>Safe Harbor 401(k):</strong> deemed not top-heavy + skips ACP/ADP testing.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s416.h2.inputs">Inputs</h2>
            <form id="s416-form" class="inline-form">
                <label><span data-i18n="view.s416.label.key_balance">Key employee balance ($)</span>
                    <input type="number" step="0.01" name="key_employee_balance" value="${state.key_employee_balance}"></label>
                <label><span data-i18n="view.s416.label.non_key_balance">Non-key employee balance ($)</span>
                    <input type="number" step="0.01" name="non_key_employee_balance" value="${state.non_key_employee_balance}"></label>
                <label><span data-i18n="view.s416.label.key_count">Key employees</span>
                    <input type="number" step="1" name="key_employee_count" value="${state.key_employee_count}"></label>
                <label><span data-i18n="view.s416.label.non_key_count">Non-key employees</span>
                    <input type="number" step="1" name="non_key_employee_count" value="${state.non_key_employee_count}"></label>
                <label><span data-i18n="view.s416.label.avg_comp">Avg non-key compensation ($)</span>
                    <input type="number" step="0.01" name="avg_non_key_compensation" value="${state.avg_non_key_compensation}"></label>
                <label><span data-i18n="view.s416.label.safe_harbor">Safe Harbor 401(k) in place?</span>
                    <input type="checkbox" name="has_safe_harbor" ${state.has_safe_harbor ? 'checked' : ''}></label>
                <label><span data-i18n="view.s416.label.vesting">Already using minimum vesting?</span>
                    <input type="checkbox" name="is_minimum_vesting" ${state.is_minimum_vesting ? 'checked' : ''}></label>
                <label><span data-i18n="view.s416.label.defined_benefit">Includes defined benefit?</span>
                    <input type="checkbox" name="plan_includes_defined_benefit" ${state.plan_includes_defined_benefit ? 'checked' : ''}></label>
                <label><span data-i18n="view.s416.label.marginal">Marginal %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s416.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s416-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s416.h2.key_employee">Key employee tests (any one)</h2>
            <ul class="muted small">
                <li data-i18n="view.s416.ke.officer">Officer with compensation &gt; $220,000 (2024)</li>
                <li data-i18n="view.s416.ke.5pct">&gt; 5% owner (any compensation level)</li>
                <li data-i18n="view.s416.ke.1pct">&gt; 1% owner with compensation &gt; $150,000</li>
                <li data-i18n="view.s416.ke.officers_count">Officers: only top 50 by salary count (capped)</li>
                <li data-i18n="view.s416.ke.attribution">Ownership includes § 318 family attribution</li>
                <li data-i18n="view.s416.ke.former_keys">Former key employees still counted for 5-yr lookback</li>
                <li data-i18n="view.s416.ke.spouse">Spouse + family ownership attributed (§ 318)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s416.h2.vesting">Top-heavy vesting (faster than normal)</h2>
            <ul class="muted small">
                <li data-i18n="view.s416.v.3_year_cliff">3-year cliff: 0% vesting yrs 1-2, 100% after year 3</li>
                <li data-i18n="view.s416.v.2_6_graded">2-6 year graded: 20% per year starting year 2 → 100% year 6</li>
                <li data-i18n="view.s416.v.faster_than_normal">Compare to 5-yr cliff OR 3-7 year graded for non-top-heavy</li>
                <li data-i18n="view.s416.v.match">Employer matching contributions: separate vesting (2-6 year graded since EGTRRA)</li>
                <li data-i18n="view.s416.v.safe_harbor">Safe Harbor: 100% immediate vesting of safe harbor contributions</li>
                <li data-i18n="view.s416.v.qaca">Qualified Automatic Contribution Arrangement (QACA): 2-year cliff</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s416.h2.relief">Top-heavy relief strategies</h2>
            <ul class="muted small">
                <li data-i18n="view.s416.r.safe_harbor">Adopt Safe Harbor 401(k) — deemed not top-heavy</li>
                <li data-i18n="view.s416.r.qaca">QACA (Qualified Auto Contribution Arrangement) — also relief</li>
                <li data-i18n="view.s416.r.simple">SIMPLE plans: not subject to top-heavy</li>
                <li data-i18n="view.s416.r.401k_match_only">401(k) with no employer contributions in top-heavy year</li>
                <li data-i18n="view.s416.r.aggregate">Aggregation rules: merge multiple plans to dilute key % below 60%</li>
                <li data-i18n="view.s416.r.distributions">Distributions to key employees count IF within last 5 years</li>
                <li data-i18n="view.s416.r.terminate">Terminate top-heavy plan, restart non-top-heavy (extreme)</li>
            </ul>
        </div>
    `;
    document.getElementById('s416-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.key_employee_balance = Number(fd.get('key_employee_balance')) || 0;
        state.non_key_employee_balance = Number(fd.get('non_key_employee_balance')) || 0;
        state.key_employee_count = Number(fd.get('key_employee_count')) || 0;
        state.non_key_employee_count = Number(fd.get('non_key_employee_count')) || 0;
        state.avg_non_key_compensation = Number(fd.get('avg_non_key_compensation')) || 0;
        state.has_safe_harbor = !!fd.get('has_safe_harbor');
        state.is_minimum_vesting = !!fd.get('is_minimum_vesting');
        state.plan_includes_defined_benefit = !!fd.get('plan_includes_defined_benefit');
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.21;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s416-output');
    if (!el) return;
    const totalBalance = state.key_employee_balance + state.non_key_employee_balance;
    const keyPct = totalBalance > 0 ? state.key_employee_balance / totalBalance : 0;
    const isTopHeavy = keyPct > TOP_HEAVY_THRESHOLD_PCT && !state.has_safe_harbor;
    const minContribTotal = state.non_key_employee_count * state.avg_non_key_compensation * MIN_CONTRIBUTION_PCT;
    const taxImpact = minContribTotal * state.marginal_rate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s416.h2.result">Top-heavy analysis</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s416.card.key_pct">Key employee balance %</div>
                    <div class="value">${(keyPct * 100).toFixed(0)}%</div>
                </div>
                <div class="card ${isTopHeavy ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s416.card.top_heavy">Top-heavy?</div>
                    <div class="value">${isTopHeavy ? esc(t('view.s416.status.yes')) : esc(t('view.s416.status.no'))}</div>
                </div>
                ${isTopHeavy ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.s416.card.min_contrib">Required 3% min for non-key</div>
                        <div class="value">$${minContribTotal.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                    <div class="card neg">
                        <div class="label" data-i18n="view.s416.card.tax_cost">After-tax cost</div>
                        <div class="value">$${(minContribTotal - taxImpact).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
                <div class="card ${state.has_safe_harbor ? 'pos' : ''}">
                    <div class="label" data-i18n="view.s416.card.safe_harbor">Safe Harbor relief</div>
                    <div class="value">${state.has_safe_harbor ? esc(t('view.s416.status.yes')) : esc(t('view.s416.status.no'))}</div>
                </div>
            </div>
        </div>
    `;
}
