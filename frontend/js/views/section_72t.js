// § 72(t) SEPP — Substantially Equal Periodic Payments.
// Early withdrawal (before 59½) escapes the 10% additional tax via three approved
// methods: RMD, Amortization, Annuitization. Must continue ≥5 yrs OR until 59½
// (whichever later). MODIFY and you owe the 10% back to year 1 + interest.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const REASONABLE_RATE_FLOOR = 0.05;  // SECURE 2.0 §322: max(5%, 120% mid-term AFR)

let state = {
    account_balance: 0,
    age: 50,
    spouse_age: 50,
    use_joint_life: false,
    interest_rate: 0.05,
    method: 'amortization',
};

// Single Life Expectancy Table (simplified - § 1.401(a)(9)-9 Q&A 1)
const SLE_TABLE = {
    35: 51.0, 36: 50.0, 37: 49.0, 38: 48.0, 39: 47.0, 40: 46.0, 41: 45.0,
    42: 44.0, 43: 43.0, 44: 42.0, 45: 41.0, 46: 40.0, 47: 39.0, 48: 38.0,
    49: 37.1, 50: 36.2, 51: 35.3, 52: 34.3, 53: 33.4, 54: 32.5, 55: 31.6,
    56: 30.6, 57: 29.8, 58: 28.9, 59: 28.0,
};

export async function renderSection72t(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s72t.h1.title">// § 72(t) SEPP CALCULATOR</span></h1>
        <p class="muted small" data-i18n="view.s72t.hint.intro">
            Early IRA / 401(k) withdrawal before 59½ without 10% penalty via Substantially Equal
            Periodic Payments. THREE approved methods: <strong>RMD, Amortization, Annuitization</strong>.
            Must continue ≥ 5 years OR until 59½ (whichever later). MODIFY in those years and 10%
            penalty applies back to year 1 + interest. SECURE 2.0 floors rate at max(5%, 120% AFR).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s72t.h2.inputs">Inputs</h2>
            <form id="s72t-form" class="inline-form">
                <label><span data-i18n="view.s72t.label.balance">Account balance ($)</span>
                    <input type="number" step="1000" name="account_balance" value="${state.account_balance}"></label>
                <label><span data-i18n="view.s72t.label.age">Your age</span>
                    <input type="number" step="1" name="age" value="${state.age}"></label>
                <label><span data-i18n="view.s72t.label.spouse_age">Spouse age</span>
                    <input type="number" step="1" name="spouse_age" value="${state.spouse_age}"></label>
                <label><span data-i18n="view.s72t.label.joint">Use joint life expectancy?</span>
                    <input type="checkbox" name="use_joint_life" ${state.use_joint_life ? 'checked' : ''}></label>
                <label><span data-i18n="view.s72t.label.rate">Interest rate (max 120% mid-term AFR or 5%)</span>
                    <input type="number" step="0.001" name="interest_rate" value="${state.interest_rate}"></label>
                <label><span data-i18n="view.s72t.label.method">Method</span>
                    <select name="method">
                        <option value="rmd" ${state.method === 'rmd' ? 'selected' : ''}>RMD (lowest, recalc yearly)</option>
                        <option value="amortization" ${state.method === 'amortization' ? 'selected' : ''}>Amortization (fixed)</option>
                        <option value="annuitization" ${state.method === 'annuitization' ? 'selected' : ''}>Annuitization (fixed, mortality factor)</option>
                    </select>
                </label>
                <button class="primary" type="submit" data-i18n="view.s72t.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s72t-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s72t.h2.methods">Method comparison</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s72t.th.method">Method</th>
                    <th data-i18n="view.s72t.th.amount">Annual amount</th>
                    <th data-i18n="view.s72t.th.flex">Flexibility</th>
                    <th data-i18n="view.s72t.th.notes">Notes</th>
                </tr></thead>
                <tbody>
                    <tr><td>RMD</td><td>Lowest</td><td>Recalc each year (var)</td><td data-i18n="view.s72t.note.rmd">Balance / life expectancy each year</td></tr>
                    <tr><td>Amortization</td><td>Mid</td><td>Fixed (one-time switch to RMD allowed)</td><td data-i18n="view.s72t.note.amort">Balance amortized at rate over life expectancy</td></tr>
                    <tr><td>Annuitization</td><td>Highest typically</td><td>Fixed</td><td data-i18n="view.s72t.note.annuit">Annuity factor from § 1.401(a)(9)-9 tables</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s72t.h2.modifications">What counts as MODIFICATION (triggers penalty)</h2>
            <ul class="muted small">
                <li data-i18n="view.s72t.mod.skip">Skipping or doubling a payment</li>
                <li data-i18n="view.s72t.mod.amount_change">Changing the amount (except amort→RMD one-time switch)</li>
                <li data-i18n="view.s72t.mod.combine_split">Combining or splitting source IRAs</li>
                <li data-i18n="view.s72t.mod.rollover">Rolling money IN OR OUT of the SEPP account</li>
                <li data-i18n="view.s72t.mod.account_zero">Account goes to zero (no longer "substantially equal")</li>
            </ul>
        </div>
    `;
    document.getElementById('s72t-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.account_balance = Number(fd.get('account_balance')) || 0;
        state.age = Number(fd.get('age')) || 50;
        state.spouse_age = Number(fd.get('spouse_age')) || 50;
        state.use_joint_life = !!fd.get('use_joint_life');
        state.interest_rate = Math.max(state.interest_rate, 0);
        state.interest_rate = Number(fd.get('interest_rate')) || 0.05;
        state.method = fd.get('method');
        renderOutput();
    });
    renderOutput();
}

function lifeExp(age) {
    return SLE_TABLE[age] ?? SLE_TABLE[Math.max(35, Math.min(59, age))] ?? 30;
}

function renderOutput() {
    const el = document.getElementById('s72t-output');
    if (!el) return;
    const r = state.interest_rate;
    const yourLE = lifeExp(state.age);
    const spouseLE = lifeExp(state.spouse_age);
    const usedLE = state.use_joint_life ? Math.max(yourLE, spouseLE) + 2 : yourLE;
    // RMD
    const rmd = state.account_balance / usedLE;
    // Amortization
    const amortFactor = (1 - Math.pow(1 + r, -usedLE)) / r;
    const amort = state.account_balance / amortFactor;
    // Annuitization (approx using same amortization but with mortality table - simplify)
    const annuit = amort * 1.02;
    const chosen = state.method === 'rmd' ? rmd : (state.method === 'amortization' ? amort : annuit);
    const yearsUntil59h = Math.max(0, 59.5 - state.age);
    const minSeppYears = Math.max(5, yearsUntil59h);
    const totalDistributions = chosen * minSeppYears;
    const tenPctIfBlown = chosen * 0.10 * Math.ceil(yearsUntil59h);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s72t.h2.result">Annual distribution by method</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s72t.card.rmd">RMD method</div>
                    <div class="value">$${rmd.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s72t.card.amort">Amortization method</div>
                    <div class="value">$${amort.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s72t.card.annuit">Annuitization method</div>
                    <div class="value">$${annuit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s72t.card.chosen">Chosen method (annual)</div>
                    <div class="value">$${chosen.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s72t.card.min_years">Min SEPP years</div>
                    <div class="value">${minSeppYears.toFixed(1)}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s72t.card.total">Total distributions</div>
                    <div class="value">$${totalDistributions.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s72t.card.if_blown">10% if modified</div>
                    <div class="value">$${tenPctIfBlown.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
