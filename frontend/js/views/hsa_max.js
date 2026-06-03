// HSA Contribution Maximizer — IRC § 223.
// Triple tax advantage: deductible going in, tax-free growth, tax-free withdrawal
// for medical. Use as a stealth retirement account — pay medical out-of-pocket
// while keeping receipts, reimburse decades later tax-free.

import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const LIMITS = {
    2024: { self: 4_150, family: 8_300, catchup_55: 1_000 },
    2025: { self: 4_300, family: 8_550, catchup_55: 1_000 },
    2026: { self: 4_400, family: 8_800, catchup_55: 1_000 },
};

let state = {
    year: new Date().getFullYear(),
    coverage: 'self',
    age: 35,
    spouse_age: 35,
    spouse_separate_hsa: false,
    marginal_rate: 0.32,
    state_rate: 0,
    months_eligible: 12,
    current_balance: 0,
    expected_return: 0.07,
    years_to_retirement: 30,
};

export async function renderHsaMax(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.hsa.h1.title">// HSA MAXIMIZER</span></h1>
        <p class="muted small" data-i18n="view.hsa.hint.intro">
            <strong>Triple tax advantage:</strong> deductible going in, tax-free growth,
            tax-free withdrawal for qualified medical. Stealth retirement account:
            pay medical out-of-pocket now, save receipts, reimburse yourself decades later.
            Requires HDHP (high-deductible health plan) coverage.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.hsa.h2.inputs">Inputs</h2>
            <form id="hsa-form" class="inline-form">
                <label><span data-i18n="view.hsa.label.year">Year</span>
                    <select name="year">${Object.keys(LIMITS).map(y =>
                        `<option value="${y}" ${Number(y) === state.year ? 'selected' : ''}>${y}</option>`
                    ).join('')}</select>
                </label>
                <label><span data-i18n="view.hsa.label.coverage">HDHP coverage</span>
                    <select name="coverage">
                        <option value="self"   ${state.coverage === 'self' ? 'selected' : ''}>Self-only</option>
                        <option value="family" ${state.coverage === 'family' ? 'selected' : ''}>Family</option>
                    </select>
                </label>
                <label><span data-i18n="view.hsa.label.age">Your age</span>
                    <input type="number" step="1" name="age" value="${state.age}" min="18" max="100"></label>
                <label><span data-i18n="view.hsa.label.spouse_age">Spouse age (family only)</span>
                    <input type="number" step="1" name="spouse_age" value="${state.spouse_age}" min="18" max="100"></label>
                <label><span data-i18n="view.hsa.label.spouse_separate">Spouse has separate HSA?</span>
                    <input type="checkbox" name="spouse_separate_hsa" ${state.spouse_separate_hsa ? 'checked' : ''}></label>
                <label><span data-i18n="view.hsa.label.months">Months eligible (HDHP coverage)</span>
                    <input type="number" step="1" name="months_eligible" value="${state.months_eligible}" min="0" max="12"></label>
                <label><span data-i18n="view.hsa.label.marginal_rate">Marginal federal rate %</span>
                    <input type="number" step="0.5" name="marginal_rate" value="${(state.marginal_rate * 100).toFixed(1)}"></label>
                <label><span data-i18n="view.hsa.label.state_rate">State rate % (CA / NJ taxed)</span>
                    <input type="number" step="0.5" name="state_rate" value="${(state.state_rate * 100).toFixed(1)}"></label>
                <label><span data-i18n="view.hsa.label.current_balance">Current HSA balance ($)</span>
                    <input type="number" step="100" name="current_balance" value="${state.current_balance}"></label>
                <label><span data-i18n="view.hsa.label.expected_return">Expected return %</span>
                    <input type="number" step="0.5" name="expected_return" value="${(state.expected_return * 100).toFixed(1)}"></label>
                <label><span data-i18n="view.hsa.label.years">Years to age 65</span>
                    <input type="number" step="1" name="years_to_retirement" value="${state.years_to_retirement}"></label>
                <button class="primary" type="submit" data-i18n="view.hsa.btn.recompute">Recompute</button>
            </form>
        </div>
        <div id="hsa-output"></div>
    `;
    document.getElementById('hsa-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.year = Number(fd.get('year'));
        state.coverage = fd.get('coverage');
        state.age = Number(fd.get('age')) || 35;
        state.spouse_age = Number(fd.get('spouse_age')) || 35;
        state.spouse_separate_hsa = !!fd.get('spouse_separate_hsa');
        state.months_eligible = Number(fd.get('months_eligible')) || 12;
        state.marginal_rate = (Number(fd.get('marginal_rate')) || 32) / 100;
        state.state_rate = (Number(fd.get('state_rate')) || 0) / 100;
        state.current_balance = Number(fd.get('current_balance')) || 0;
        state.expected_return = (Number(fd.get('expected_return')) || 7) / 100;
        state.years_to_retirement = Number(fd.get('years_to_retirement')) || 30;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('hsa-output');
    if (!el) return;
    const limits = LIMITS[state.year] || LIMITS[2024];
    const baseLimit = state.coverage === 'family' ? limits.family : limits.self;
    const yourCatchup = state.age >= 55 ? limits.catchup_55 : 0;
    const spouseCatchup = (state.coverage === 'family' && state.spouse_age >= 55 && state.spouse_separate_hsa) ? limits.catchup_55 : 0;
    const prorated = baseLimit * (state.months_eligible / 12);
    const yourMax = prorated + yourCatchup;
    const totalHousehold = yourMax + spouseCatchup;
    const totalRate = state.marginal_rate + state.state_rate + 0.0765; // FICA-equivalent if cafeteria-plan
    const taxSavings = totalHousehold * totalRate;
    // 30-year FV projection.
    const fv = projectFV(state.current_balance, totalHousehold, state.expected_return, state.years_to_retirement);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.hsa.h2.contribution">${state.year} contribution limits</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.hsa.card.your_max">Your max</div>
                    <div class="value">$${yourMax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.hsa.card.household">Household max</div>
                    <div class="value">$${totalHousehold.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.hsa.card.base_limit">Base limit</div>
                    <div class="value">$${baseLimit.toLocaleString()}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.hsa.card.catchup">Catch-up (≥55)</div>
                    <div class="value">$${(yourCatchup + spouseCatchup).toLocaleString()}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.hsa.card.year_savings">Year-1 tax savings</div>
                    <div class="value">$${taxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.hsa.card.fv">FV in ${state.years_to_retirement}y @ ${(state.expected_return * 100).toFixed(0)}%</div>
                    <div class="value">$${fv.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.hsa.h2.strategy">Stealth retirement strategy</h2>
            <p data-i18n="view.hsa.strategy.body">
                Max HSA + invest in low-fee index funds. Pay current medical out-of-pocket
                from taxable account. Save every medical receipt. At age 65+, reimburse
                yourself for decades of receipts tax-free — or use after 65 for any
                purpose (taxed as ordinary, no penalty, like a Traditional IRA).
            </p>
            <p class="muted small" data-i18n="view.hsa.warning.ca_nj">
                California + New Jersey + Alabama do NOT recognize HSAs — contributions are
                subject to state income tax. NH + TN don't tax interest/dividends so HSA
                growth still escapes state tax there.
            </p>
        </div>
    `;
}

function projectFV(pv, pmt, rate, years) {
    let bal = pv;
    for (let i = 0; i < years; i++) {
        bal = (bal + pmt) * (1 + rate);
    }
    return bal;
}
