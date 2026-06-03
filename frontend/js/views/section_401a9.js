// IRC § 401(a)(9) — Required Minimum Distribution (RMD) Calculator.
// SECURE Act 2019 + SECURE 2.0 2022 changed RBD (Required Beginning Date):
// Born ≤ 6/30/1949: age 70½. Born 7/1/1949 → 12/31/1950: age 72. Born 1951-1959: age 73.
// Born 1960+: age 75. Uniform Lifetime Table (most account holders).

import { currentViewToken, viewIsCurrent } from '../app.js';

const UNIFORM_LIFETIME_TABLE_2022 = {
    72: 27.4, 73: 26.5, 74: 25.5, 75: 24.6, 76: 23.7, 77: 22.9, 78: 22.0, 79: 21.1,
    80: 20.2, 81: 19.4, 82: 18.5, 83: 17.7, 84: 16.8, 85: 16.0, 86: 15.2, 87: 14.4,
    88: 13.7, 89: 12.9, 90: 12.2, 91: 11.5, 92: 10.8, 93: 10.1, 94: 9.5, 95: 8.9,
    96: 8.4, 97: 7.8, 98: 7.3, 99: 6.8, 100: 6.4,
};

const PENALTY_RATE_PRE_2023 = 0.50;
const PENALTY_RATE_2023_PLUS = 0.25;
const PENALTY_RATE_TIMELY_CORRECTION = 0.10;

let state = {
    birth_year: 1950,
    account_balance_prior_dec_31: 0,
    spouse_younger_more_than_10: false,
    spouse_age: 65,
    qcd_amount: 0,
    current_year: new Date().getFullYear(),
    fed_marginal_rate: 0.32,
    state_marginal_rate: 0.06,
};

const QCD_LIMIT_2024 = 105_000;

export async function renderSection401a9(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s401a9.h1.title">// § 401(a)(9) RMD CALCULATOR</span></h1>
        <p class="muted small" data-i18n="view.s401a9.hint.intro">
            SECURE Act 2019 + SECURE 2.0 2022 changed RBD (Required Beginning Date):
            <strong>Born ≤ 6/30/1949: age 70½. Born 7/1/1949 → 12/31/1950: age 72. Born 1951-1959:
            age 73. Born 1960+: age 75.</strong> RMD = prior-year Dec 31 balance ÷ life-expectancy
            divisor (Uniform Lifetime Table). <strong>QCD up to $105,000 (2024)</strong> from IRA
            offsets RMD without inclusion in income. <strong>Penalty for missed RMD: 25%</strong>
            (10% if timely corrected).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s401a9.h2.inputs">Inputs</h2>
            <form id="s401a9-form" class="inline-form">
                <label><span data-i18n="view.s401a9.label.birth_year">Birth year</span>
                    <input type="number" step="1" name="birth_year" value="${state.birth_year}"></label>
                <label><span data-i18n="view.s401a9.label.current_year">Current year</span>
                    <input type="number" step="1" name="current_year" value="${state.current_year}"></label>
                <label><span data-i18n="view.s401a9.label.balance">Prior Dec 31 balance ($)</span>
                    <input type="number" step="1000" name="account_balance_prior_dec_31" value="${state.account_balance_prior_dec_31}"></label>
                <label><span data-i18n="view.s401a9.label.spouse_younger">Spouse &gt; 10 yrs younger sole beneficiary?</span>
                    <input type="checkbox" name="spouse_younger_more_than_10" ${state.spouse_younger_more_than_10 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s401a9.label.spouse_age">Spouse age</span>
                    <input type="number" step="1" name="spouse_age" value="${state.spouse_age}"></label>
                <label><span data-i18n="view.s401a9.label.qcd">QCD planned ($)</span>
                    <input type="number" step="100" name="qcd_amount" value="${state.qcd_amount}"></label>
                <label><span data-i18n="view.s401a9.label.fed_rate">Federal marginal %</span>
                    <input type="number" step="0.01" name="fed_marginal_rate" value="${state.fed_marginal_rate}"></label>
                <label><span data-i18n="view.s401a9.label.state_rate">State marginal %</span>
                    <input type="number" step="0.01" name="state_marginal_rate" value="${state.state_marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s401a9.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s401a9-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s401a9.h2.qcd_advantage">QCD advantage</h2>
            <ul class="muted small">
                <li data-i18n="view.s401a9.qcd.no_agi">Excludes amount from AGI — better than itemized charitable deduction</li>
                <li data-i18n="view.s401a9.qcd.medicare">Lower AGI reduces Medicare IRMAA premiums (2-yr lookback)</li>
                <li data-i18n="view.s401a9.qcd.muni">Lower AGI keeps muni bond / Social Security from being taxed</li>
                <li data-i18n="view.s401a9.qcd.over_70">Available age 70½+ even if RMD not yet required</li>
                <li data-i18n="view.s401a9.qcd.cgi">Counts toward RMD if RMD age</li>
                <li data-i18n="view.s401a9.qcd.501c3">Direct transfer to 501(c)(3) public charity only (not DAF, foundation)</li>
                <li data-i18n="view.s401a9.qcd.cgi_2024">$105,000 limit 2024 (indexed); SECURE 2.0 split $50k one-time CRT / CGA</li>
                <li data-i18n="view.s401a9.qcd.spouse">Each spouse has separate limit if separate IRAs</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s401a9.h2.exceptions">RMD exceptions</h2>
            <ul class="muted small">
                <li data-i18n="view.s401a9.ex.roth_ira">Roth IRA: NO RMD during owner's life (SECURE 2.0 eliminated Roth 401(k) RMD too)</li>
                <li data-i18n="view.s401a9.ex.still_working">Still working + &lt; 5% owner: 401(k) RMD deferred at current employer</li>
                <li data-i18n="view.s401a9.ex.first_year">First-year RMD can be delayed to April 1 of following year (but creates double RMD year)</li>
                <li data-i18n="view.s401a9.ex.qcd_offset">QCD up to $105k offsets RMD income inclusion</li>
                <li data-i18n="view.s401a9.ex.terminal">Terminal illness: SECURE 2.0 exception (lifetime expectancy &lt; 7 yrs)</li>
                <li data-i18n="view.s401a9.ex.inherited">Inherited IRA: separate rules (10-yr rule + EDB exceptions)</li>
            </ul>
        </div>
    `;
    document.getElementById('s401a9-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.birth_year = Number(fd.get('birth_year')) || 1950;
        state.current_year = Number(fd.get('current_year')) || new Date().getFullYear();
        state.account_balance_prior_dec_31 = Number(fd.get('account_balance_prior_dec_31')) || 0;
        state.spouse_younger_more_than_10 = !!fd.get('spouse_younger_more_than_10');
        state.spouse_age = Number(fd.get('spouse_age')) || 65;
        state.qcd_amount = Number(fd.get('qcd_amount')) || 0;
        state.fed_marginal_rate = Number(fd.get('fed_marginal_rate')) || 0.32;
        state.state_marginal_rate = Number(fd.get('state_marginal_rate')) || 0.06;
        renderOutput();
    });
    renderOutput();
}

function rmdAge(birthYear) {
    if (birthYear <= 1949 && new Date(birthYear, 6, 1) > new Date(birthYear, 5, 30)) return 72;  // approx
    if (birthYear <= 1949) return 70.5;
    if (birthYear <= 1950) return 72;
    if (birthYear <= 1959) return 73;
    return 75;
}

function renderOutput() {
    const el = document.getElementById('s401a9-output');
    if (!el) return;
    const currentAge = state.current_year - state.birth_year;
    const rmdRequiredAge = rmdAge(state.birth_year);
    const isRmdYear = currentAge >= rmdRequiredAge;
    const divisor = UNIFORM_LIFETIME_TABLE_2022[Math.min(100, currentAge)] || 6.4;
    const rmd = isRmdYear ? state.account_balance_prior_dec_31 / divisor : 0;
    const qcdCapped = Math.min(state.qcd_amount, QCD_LIMIT_2024, rmd);
    const taxableRmd = Math.max(0, rmd - qcdCapped);
    const fedTax = taxableRmd * state.fed_marginal_rate;
    const stateTax = taxableRmd * state.state_marginal_rate;
    const totalTax = fedTax + stateTax;
    const qcdSavings = qcdCapped * (state.fed_marginal_rate + state.state_marginal_rate);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s401a9.h2.result">RMD calculation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s401a9.card.age">Current age</div>
                    <div class="value">${currentAge}</div>
                </div>
                <div class="card ${isRmdYear ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s401a9.card.rmd_age">RMD age</div>
                    <div class="value">${rmdRequiredAge}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s401a9.card.divisor">Life expectancy divisor</div>
                    <div class="value">${divisor.toFixed(1)}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s401a9.card.rmd">Required RMD</div>
                    <div class="value">$${rmd.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s401a9.card.qcd_used">QCD applied</div>
                    <div class="value">$${qcdCapped.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s401a9.card.taxable">Taxable distribution</div>
                    <div class="value">$${taxableRmd.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s401a9.card.fed_tax">Fed tax</div>
                    <div class="value">$${fedTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s401a9.card.state_tax">State tax</div>
                    <div class="value">$${stateTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s401a9.card.qcd_savings">QCD tax savings</div>
                    <div class="value">$${qcdSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.spouse_younger_more_than_10 ? `
                <p class="muted small" style="margin-top:10px" data-i18n="view.s401a9.note.joint">
                    Spouse &gt; 10 yrs younger sole beneficiary: use Joint Life Expectancy Table
                    (longer divisor → smaller RMD). Recalculate annually.
                </p>
            ` : ''}
        </div>
    `;
}
