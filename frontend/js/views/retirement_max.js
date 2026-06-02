// Retirement Contribution Maximizer — SEP IRA, Solo 401k, traditional IRA.
// Self-employed traders can shelter 20-25% of business income tax-deferred.
// 2024 limits: SEP $69k, Solo 401k $69k (+$7,500 catch-up if 50+).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LIMITS = {
    2024: {
        sep_pct: 0.25, sep_cap: 69_000,
        solo401k_employee: 23_000, solo401k_total: 69_000, solo401k_catchup: 7_500,
        ira_trad: 7_000, ira_catchup: 1_000,
        se_deduction: 0.9235,
        // Employer side of Solo 401k = 20% of SE net earnings, employee side = $23k flat
        solo401k_employer_pct: 0.20,
    },
    2025: {
        sep_pct: 0.25, sep_cap: 70_000,
        solo401k_employee: 23_500, solo401k_total: 70_000, solo401k_catchup: 7_500,
        ira_trad: 7_000, ira_catchup: 1_000,
        se_deduction: 0.9235,
        solo401k_employer_pct: 0.20,
    },
    2026: {
        sep_pct: 0.25, sep_cap: 72_000,
        solo401k_employee: 24_000, solo401k_total: 72_000, solo401k_catchup: 7_500,
        ira_trad: 7_000, ira_catchup: 1_000,
        se_deduction: 0.9235,
        solo401k_employer_pct: 0.20,
    },
};

let state = {
    se_income: 0,
    age: 35,
    year: new Date().getFullYear(),
    marginal_rate: 0.32,
    spouse_income: 0,
};

export async function renderRetirementMax(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.retire.h1.title">// RETIREMENT MAXIMIZER</span></h1>
        <p class="muted small" data-i18n="view.retire.hint.intro">
            Self-employed traders can shelter 20-25% of net business income tax-deferred
            via SEP IRA or Solo 401k. Compare side-by-side and pick the structure that
            shelters more given your numbers.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.retire.h2.inputs">Inputs</h2>
            <form id="ret-form" class="inline-form">
                <label><span data-i18n="view.retire.label.se_income">SE net business income ($)</span>
                    <input type="number" step="100" name="se_income" value="${state.se_income}"></label>
                <label><span data-i18n="view.retire.label.age">Age</span>
                    <input type="number" step="1" name="age" value="${state.age}" min="18" max="100"></label>
                <label><span data-i18n="view.retire.label.year">Year</span>
                    <select name="year">
                        ${Object.keys(LIMITS).map(y =>
                            `<option value="${y}" ${Number(y) === state.year ? 'selected' : ''}>${y}</option>`
                        ).join('')}
                    </select>
                </label>
                <label><span data-i18n="view.retire.label.marginal_rate">Marginal tax rate %</span>
                    <input type="number" step="0.5" name="marginal_rate" value="${(state.marginal_rate * 100).toFixed(1)}" min="10" max="50"></label>
                <label><span data-i18n="view.retire.label.spouse_income">Spouse W-2 income ($)</span>
                    <input type="number" step="100" name="spouse_income" value="${state.spouse_income}"></label>
                <button class="primary" type="submit" data-i18n="view.retire.btn.recompute">Recompute</button>
            </form>
        </div>
        <div id="ret-output"></div>
    `;
    document.getElementById('ret-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.se_income = Number(fd.get('se_income')) || 0;
        state.age = Number(fd.get('age')) || 35;
        state.year = Number(fd.get('year'));
        state.marginal_rate = (Number(fd.get('marginal_rate')) || 32) / 100;
        state.spouse_income = Number(fd.get('spouse_income')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('ret-output');
    if (!el) return;
    const limits = LIMITS[state.year] || LIMITS[2024];
    const seBase = state.se_income * limits.se_deduction;
    const seTax = computeSETax(seBase, limits);
    const halfSe = seTax / 2;
    // Compensation for retirement = SE income - half SE tax
    const compForRetire = state.se_income - halfSe;
    // SEP: 25% of comp, capped at $69k
    const sepMax = Math.min(compForRetire * limits.sep_pct, limits.sep_cap);
    // Solo 401k: $23k employee + 20% employer = up to $69k total
    const solo401kEmployee = limits.solo401k_employee + (state.age >= 50 ? limits.solo401k_catchup : 0);
    const solo401kEmployer = compForRetire * limits.solo401k_employer_pct;
    const solo401kTotal = Math.min(solo401kEmployee + solo401kEmployer,
        limits.solo401k_total + (state.age >= 50 ? limits.solo401k_catchup : 0));
    // Traditional IRA (deductibility may be limited if covered by employer plan)
    const iraTrad = limits.ira_trad + (state.age >= 50 ? limits.ira_catchup : 0);
    // Spousal IRA (if applicable)
    const spousalIra = state.spouse_income > 0
        ? iraTrad
        : 0;
    // Best strategy: Solo 401k almost always wins for SE traders unless income < $20k
    const bestShelter = Math.max(sepMax, solo401kTotal);
    const taxSavings = bestShelter * state.marginal_rate;
    const recommendation = solo401kTotal > sepMax
        ? t('view.retire.recommend.solo401k')
        : t('view.retire.recommend.sep');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.retire.h2.summary">Maximum tax-deferred shelter</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.retire.card.best_shelter">Best shelter</div>
                    <div class="value">$${bestShelter.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.retire.card.tax_savings">Tax savings</div>
                    <div class="value">$${taxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.retire.card.recommend">Recommended</div>
                    <div class="value">${esc(recommendation)}</div>
                </div>
            </div>
        </div>
        <div class="panel-grid">
            <div class="chart-panel">
                <h2 data-i18n="view.retire.h2.solo401k">Solo 401(k)</h2>
                <table class="trades"><tbody>
                    <tr><td data-i18n="view.retire.row.employee">Employee deferral</td>
                        <td>$${solo401kEmployee.toLocaleString()}</td></tr>
                    <tr><td data-i18n="view.retire.row.employer">Employer profit-share (20%)</td>
                        <td>$${solo401kEmployer.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td></tr>
                    <tr><td><strong data-i18n="view.retire.row.total">Total contribution</strong></td>
                        <td><strong class="pos">$${solo401kTotal.toLocaleString(undefined, { maximumFractionDigits: 0 })}</strong></td></tr>
                    <tr><td data-i18n="view.retire.row.tax_savings">Tax savings @ marginal</td>
                        <td class="pos">$${(solo401kTotal * state.marginal_rate).toLocaleString(undefined, { maximumFractionDigits: 0 })}</td></tr>
                </tbody></table>
                <p class="muted small" data-i18n="view.retire.solo401k.note">
                    Best for: high-income SE traders. Allows Roth + after-tax (mega-backdoor)
                    contributions. Requires solo 401k plan setup ($0-300 typical).
                </p>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.retire.h2.sep">SEP IRA</h2>
                <table class="trades"><tbody>
                    <tr><td data-i18n="view.retire.row.contribution">Contribution (25% × comp)</td>
                        <td>$${sepMax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td></tr>
                    <tr><td data-i18n="view.retire.row.tax_savings">Tax savings @ marginal</td>
                        <td class="pos">$${(sepMax * state.marginal_rate).toLocaleString(undefined, { maximumFractionDigits: 0 })}</td></tr>
                </tbody></table>
                <p class="muted small" data-i18n="view.retire.sep.note">
                    Best for: simple setup, no plan administration. Lower shelter than Solo 401k
                    once income exceeds ~$200k. SIMPLE to open at most brokers (Fidelity / Schwab).
                </p>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.retire.h2.ira">Traditional IRA</h2>
                <table class="trades"><tbody>
                    <tr><td data-i18n="view.retire.row.self_contribution">Self contribution</td>
                        <td>$${iraTrad.toLocaleString()}</td></tr>
                    <tr><td data-i18n="view.retire.row.spouse_contribution">Spousal IRA</td>
                        <td>$${spousalIra.toLocaleString()}</td></tr>
                </tbody></table>
                <p class="muted small" data-i18n="view.retire.ira.note">
                    Deduction phased out by AGI if covered by employer plan (incl. SEP/Solo 401k).
                    Roth IRA phases out at high income too — backdoor Roth required.
                </p>
            </div>
        </div>
    `;
}

function computeSETax(seBase, limits) {
    const SS_BASE = 168_600;
    const SS_RATE = 0.124;
    const MEDICARE_RATE = 0.029;
    const ssTaxable = Math.min(seBase, SS_BASE);
    return ssTaxable * SS_RATE + seBase * MEDICARE_RATE;
}
