// IRC § 7702A — Modified Endowment Contract (MEC) 7-Pay Test.
// Life insurance contract failing 7-pay test becomes MEC. MEC distributions / loans
// treated as TAXABLE INCOME first (LIFO) + 10% penalty if < 59½.
// Cum premiums > sum of 7 net level premiums in first 7 years → MEC.
// Material change (raise death benefit, certain option swaps) restarts 7-year window.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    death_benefit: 0,
    annual_net_level_premium: 0,
    cumulative_premiums_paid: [0, 0, 0, 0, 0, 0, 0],
    has_material_change: false,
    material_change_year: 0,
    age_at_distribution: 50,
    distribution_amount: 0,
    basis_in_contract: 0,
    marginal_rate: 0.32,
};

export async function renderSection7702a(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s7702a.h1.title">// § 7702A MEC 7-PAY TEST</span></h1>
        <p class="muted small" data-i18n="view.s7702a.hint.intro">
            Life insurance contract failing the <strong>7-pay test</strong> becomes a
            <strong>Modified Endowment Contract (MEC)</strong>. <strong>MEC distributions / loans
            treated as TAXABLE INCOME first (LIFO)</strong> + 10% penalty if &lt; 59½.
            Cum premiums &gt; sum of 7 net level premiums in first 7 yrs → MEC. <strong>Material
            change</strong> (raise death benefit, certain option swaps) restarts 7-year window.
            Death benefit + cash-value-loan-payoff is tax-free for non-MEC contracts.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s7702a.h2.inputs">Inputs</h2>
            <form id="s7702a-form" class="inline-form">
                <label><span data-i18n="view.s7702a.label.death_benefit">Death benefit ($)</span>
                    <input type="number" step="10000" name="death_benefit" value="${state.death_benefit}"></label>
                <label><span data-i18n="view.s7702a.label.net_level">Net level annual premium (NLP) ($)</span>
                    <input type="number" step="100" name="annual_net_level_premium" value="${state.annual_net_level_premium}"></label>
                ${state.cumulative_premiums_paid.map((p, i) => `
                    <label><span data-i18n="view.s7702a.label.year_n">Premium paid year ${i + 1} ($)</span>
                        <input type="number" step="100" name="prem_${i + 1}" value="${p}"></label>
                `).join('')}
                <label><span data-i18n="view.s7702a.label.material">Material change in year (0 = none)</span>
                    <input type="number" step="1" min="0" max="7" name="material_change_year" value="${state.material_change_year}"></label>
                <hr style="grid-column:1/-1">
                <label><span data-i18n="view.s7702a.label.age">Age at distribution / loan</span>
                    <input type="number" step="1" name="age_at_distribution" value="${state.age_at_distribution}"></label>
                <label><span data-i18n="view.s7702a.label.dist">Distribution / loan amount ($)</span>
                    <input type="number" step="1000" name="distribution_amount" value="${state.distribution_amount}"></label>
                <label><span data-i18n="view.s7702a.label.basis">Basis in contract (cum premiums - prior dist) ($)</span>
                    <input type="number" step="1000" name="basis_in_contract" value="${state.basis_in_contract}"></label>
                <label><span data-i18n="view.s7702a.label.marginal">Marginal %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s7702a.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s7702a-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7702a.h2.benefits">Non-MEC life insurance benefits</h2>
            <ul class="muted small">
                <li data-i18n="view.s7702a.ben.death">Death benefit income tax-free (§ 101)</li>
                <li data-i18n="view.s7702a.ben.tax_def_growth">Tax-deferred cash value growth</li>
                <li data-i18n="view.s7702a.ben.loans">Policy loans NOT taxable income (treated as borrowed)</li>
                <li data-i18n="view.s7702a.ben.fifo">Cash withdrawals: FIFO (basis first), then gains</li>
                <li data-i18n="view.s7702a.ben.no_penalty">No 10% early-withdrawal penalty on non-MEC</li>
                <li data-i18n="view.s7702a.ben.long_term">Used for "infinite banking" / IBC strategy</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7702a.h2.mec_penalties">MEC penalties</h2>
            <ul class="muted small">
                <li data-i18n="view.s7702a.pen.lifo">Distributions / loans treated as INCOME FIRST (LIFO)</li>
                <li data-i18n="view.s7702a.pen.ord_income">Income portion taxed as ORDINARY (not LTCG)</li>
                <li data-i18n="view.s7702a.pen.10pct">10% penalty under 59½ unless death / disability / SEPP</li>
                <li data-i18n="view.s7702a.pen.collateral">Pledge as loan collateral = deemed distribution</li>
                <li data-i18n="view.s7702a.pen.no_death_benefit">Death benefit still income tax-free</li>
                <li data-i18n="view.s7702a.pen.irrevocable">Once MEC always MEC (no fix)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7702a.h2.uses_for_mec">When MEC is INTENDED</h2>
            <ul class="muted small">
                <li data-i18n="view.s7702a.use.estate">Estate-planning vehicles (no living-access intent)</li>
                <li data-i18n="view.s7702a.use.high_premium">Maximize tax-deferred growth, pay all premiums upfront</li>
                <li data-i18n="view.s7702a.use.heirs">Plan to leave death benefit to heirs untaxed</li>
                <li data-i18n="view.s7702a.use.elderly">Elderly individuals using MEC as legacy vehicle</li>
            </ul>
        </div>
    `;
    document.getElementById('s7702a-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.death_benefit = Number(fd.get('death_benefit')) || 0;
        state.annual_net_level_premium = Number(fd.get('annual_net_level_premium')) || 0;
        state.cumulative_premiums_paid = [1,2,3,4,5,6,7].map(n => Number(fd.get('prem_' + n)) || 0);
        state.material_change_year = Number(fd.get('material_change_year')) || 0;
        state.age_at_distribution = Number(fd.get('age_at_distribution')) || 50;
        state.distribution_amount = Number(fd.get('distribution_amount')) || 0;
        state.basis_in_contract = Number(fd.get('basis_in_contract')) || 0;
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.32;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s7702a-output');
    if (!el) return;
    // 7-pay test by year
    const yearChecks = [];
    let cumulative = 0;
    let isMec = false;
    let mecYear = null;
    for (let i = 0; i < 7; i++) {
        cumulative += state.cumulative_premiums_paid[i];
        const limit = state.annual_net_level_premium * (i + 1);
        const failed = cumulative > limit;
        if (failed && !isMec) {
            isMec = true;
            mecYear = i + 1;
        }
        yearChecks.push({ year: i + 1, premiumThisYear: state.cumulative_premiums_paid[i], cumulative, limit, failed });
    }
    // Distribution analysis
    const cashValueEstimate = cumulative;  // rough proxy
    const incomeInContract = Math.max(0, cashValueEstimate - state.basis_in_contract);
    let taxableIncome, basisRecovered;
    if (isMec) {
        taxableIncome = Math.min(state.distribution_amount, incomeInContract);
        basisRecovered = Math.max(0, state.distribution_amount - taxableIncome);
    } else {
        basisRecovered = Math.min(state.distribution_amount, state.basis_in_contract);
        taxableIncome = Math.max(0, state.distribution_amount - basisRecovered);
    }
    const incomeTax = taxableIncome * state.marginal_rate;
    const earlyPenalty = (isMec && state.age_at_distribution < 59.5) ? taxableIncome * 0.10 : 0;
    const totalTax = incomeTax + earlyPenalty;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s7702a.h2.result">7-pay test + distribution</h2>
            <div class="cards">
                <div class="card ${isMec ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s7702a.card.is_mec">Is MEC?</div>
                    <div class="value">${isMec ? esc(t('view.s7702a.status.yes')) : esc(t('view.s7702a.status.no'))}</div>
                </div>
                ${isMec ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.s7702a.card.mec_year">MEC triggered year</div>
                        <div class="value">${mecYear}</div>
                    </div>
                ` : ''}
                <div class="card">
                    <div class="label" data-i18n="view.s7702a.card.taxable">Taxable portion</div>
                    <div class="value">$${taxableIncome.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s7702a.card.basis_recovered">Tax-free basis recovered</div>
                    <div class="value">$${basisRecovered.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s7702a.card.income_tax">Income tax</div>
                    <div class="value">$${incomeTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${earlyPenalty > 0 ? 'neg' : ''}">
                    <div class="label" data-i18n="view.s7702a.card.penalty">10% penalty</div>
                    <div class="value">$${earlyPenalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s7702a.card.total">Total tax cost</div>
                    <div class="value">$${totalTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7702a.h2.year_table">7-pay year-by-year</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s7702a.th.year">Year</th>
                    <th data-i18n="view.s7702a.th.premium">Premium</th>
                    <th data-i18n="view.s7702a.th.cum">Cum premiums</th>
                    <th data-i18n="view.s7702a.th.limit">7-pay limit</th>
                    <th data-i18n="view.s7702a.th.failed">Fails?</th>
                </tr></thead>
                <tbody>${yearChecks.map(r => `
                    <tr>
                        <td>${r.year}</td>
                        <td>$${r.premiumThisYear.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                        <td>$${r.cumulative.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                        <td class="muted">$${r.limit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                        <td class="${r.failed ? 'neg' : 'pos'}">${r.failed ? esc(t('view.s7702a.status.yes')) : esc(t('view.s7702a.status.no'))}</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        </div>
    `;
}
