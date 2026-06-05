// IRC § 911 Foreign Earned Income Exclusion (FEIE).
// US citizens / residents working abroad exclude up to $126,500 (2024) earned income.
// Two qualifying tests: (1) Bona Fide Resident — full calendar year + foreign-residence intent,
// OR (2) Physical Presence — 330 full days in any 12-month period.
// Stack with § 911(c) Housing Cost exclusion (additional ~$17,000 base, higher in HCOL).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const FEIE_2024 = 126_500;
const FEIE_2025 = 130_000;  // projected
const HOUSING_BASE_2024 = 16_944;  // 16% of FEIE
const HOUSING_CAP_DEFAULT_PCT = 0.30;
const FOREIGN_HCOL_CITIES = ['Tokyo', 'London', 'Paris', 'Singapore', 'Hong Kong', 'Geneva', 'Zurich', 'Sydney'];

let state = {
    tax_year: new Date().getFullYear(),
    qualifying_test: 'physical_presence',
    foreign_earned_income: 0,
    foreign_housing_paid: 0,
    foreign_city: '',
    days_abroad: 0,
    is_employee: true,
    foreign_taxes_paid: 0,
    us_marginal_rate: 0.32,
};

export async function renderSection911(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s911.h1.title">// § 911 FOREIGN EARNED INCOME EXCLUSION</span></h1>
        <p class="muted small" data-i18n="view.s911.hint.intro">
            US citizens / residents working abroad exclude up to <strong>$126,500 (2024)</strong>
            of foreign earned income. Two tests: <strong>Bona Fide Resident</strong> (full calendar
            year + intent) OR <strong>Physical Presence</strong> (330 full days / any 12 mo). Stacks
            with <strong>§ 911(c) Housing Cost exclusion</strong> (~$17k base, higher in HCOL cities).
            Self-employed get housing DEDUCTION not exclusion. Election made on Form 2555.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s911.h2.inputs">Inputs</h2>
            <form id="s911-form" class="inline-form">
                <label><span data-i18n="view.s911.label.tax_year">Tax year</span>
                    <input type="number" step="1" name="tax_year" value="${state.tax_year}"></label>
                <label><span data-i18n="view.s911.label.test">Qualifying test</span>
                    <select name="qualifying_test">
                        <option value="bona_fide" ${state.qualifying_test === 'bona_fide' ? 'selected' : ''}>Bona Fide Resident</option>
                        <option value="physical_presence" ${state.qualifying_test === 'physical_presence' ? 'selected' : ''}>Physical Presence (330 days)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s911.label.feic">Foreign earned income ($)</span>
                    <input type="number" step="0.01" name="foreign_earned_income" value="${state.foreign_earned_income}"></label>
                <label><span data-i18n="view.s911.label.housing">Foreign housing paid ($)</span>
                    <input type="number" step="0.01" name="foreign_housing_paid" value="${state.foreign_housing_paid}"></label>
                <label><span data-i18n="view.s911.label.city">City of residence</span>
                    <input type="text" name="foreign_city" value="${state.foreign_city}" placeholder="e.g. Tokyo, Berlin"></label>
                <label><span data-i18n="view.s911.label.days">Days abroad</span>
                    <input type="number" step="1" name="days_abroad" value="${state.days_abroad}"></label>
                <label><span data-i18n="view.s911.label.is_employee">Employee (vs SE)?</span>
                    <input type="checkbox" name="is_employee" ${state.is_employee ? 'checked' : ''}></label>
                <label><span data-i18n="view.s911.label.foreign_tax">Foreign taxes paid ($)</span>
                    <input type="number" step="0.01" name="foreign_taxes_paid" value="${state.foreign_taxes_paid}"></label>
                <label><span data-i18n="view.s911.label.marginal">US marginal %</span>
                    <input type="number" step="0.01" name="us_marginal_rate" value="${state.us_marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s911.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s911-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s911.h2.notes">Important rules</h2>
            <ul class="muted small">
                <li data-i18n="view.s911.note.election">Election on Form 2555 — once made, REVOKE requires IRS consent + 5-yr disqualification</li>
                <li data-i18n="view.s911.note.ftc_stacking">Below-excluded income still subject to stacking rule under § 911(f)</li>
                <li data-i18n="view.s911.note.fbar">Still must file FBAR + Form 8938 + FATCA + 1040</li>
                <li data-i18n="view.s911.note.investment_income">Investment / unearned income NOT excluded</li>
                <li data-i18n="view.s911.note.se_tax">FEIE does NOT exempt SE tax — pay 15.3% on net SE earnings</li>
                <li data-i18n="view.s911.note.totalization">Totalization Agreement countries: exempt SE tax via foreign social security</li>
                <li data-i18n="view.s911.note.choose_method">Compare FEIE vs Foreign Tax Credit (Form 1116) — sometimes FTC alone superior</li>
            </ul>
        </div>
    `;
    document.getElementById('s911-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.tax_year = Number(fd.get('tax_year')) || new Date().getFullYear();
        state.qualifying_test = fd.get('qualifying_test');
        state.foreign_earned_income = Number(fd.get('foreign_earned_income')) || 0;
        state.foreign_housing_paid = Number(fd.get('foreign_housing_paid')) || 0;
        state.foreign_city = fd.get('foreign_city') || '';
        state.days_abroad = Number(fd.get('days_abroad')) || 0;
        state.is_employee = !!fd.get('is_employee');
        state.foreign_taxes_paid = Number(fd.get('foreign_taxes_paid')) || 0;
        state.us_marginal_rate = Number(fd.get('us_marginal_rate')) || 0.32;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s911-output');
    if (!el) return;
    const feicCap = state.tax_year >= 2025 ? FEIE_2025 : FEIE_2024;
    let qualifies, proRata;
    if (state.qualifying_test === 'physical_presence') {
        qualifies = state.days_abroad >= 330;
        proRata = qualifies ? 1 : state.days_abroad / 330;
    } else {
        qualifies = state.days_abroad >= 330;  // simplified
        proRata = 1;
    }
    const feicAvailable = feicCap * proRata;
    const feicExcluded = Math.min(state.foreign_earned_income, feicAvailable);
    const housingBase = HOUSING_BASE_2024 * proRata;
    const isHcol = FOREIGN_HCOL_CITIES.some(c => state.foreign_city.toLowerCase().includes(c.toLowerCase()));
    const housingCap = (isHcol ? feicCap * 0.50 : feicCap * HOUSING_CAP_DEFAULT_PCT) * proRata;
    const housingEligible = Math.max(0, state.foreign_housing_paid - housingBase);
    const housingExclusion = state.is_employee ? Math.min(housingEligible, housingCap) : 0;
    const housingDeduction = !state.is_employee ? Math.min(housingEligible, housingCap) : 0;
    const totalExcludedDeducted = feicExcluded + housingExclusion + housingDeduction;
    const usTaxSaved = totalExcludedDeducted * state.us_marginal_rate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s911.h2.result">Calculation</h2>
            <div class="cards">
                <div class="card ${qualifies ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s911.card.qualifies">Qualifies?</div>
                    <div class="value">${qualifies ? esc(t('view.s911.status.yes')) : esc(t('view.s911.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s911.card.feic_cap">FEIE cap (${state.tax_year})</div>
                    <div class="value">$${feicCap.toLocaleString()}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s911.card.pro_rata">Pro-rata factor</div>
                    <div class="value">${(proRata * 100).toFixed(0)}%</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s911.card.feic_excluded">FEIE excluded</div>
                    <div class="value">$${feicExcluded.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s911.card.housing_base">Housing base (non-deductible)</div>
                    <div class="value">$${housingBase.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s911.card.housing_cap">Housing cap${isHcol ? ' (HCOL)' : ''}</div>
                    <div class="value">$${housingCap.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s911.card.housing_excluded">Housing ${state.is_employee ? 'excluded' : 'deducted'}</div>
                    <div class="value">$${(housingExclusion + housingDeduction).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s911.card.total">Total excluded / deducted</div>
                    <div class="value">$${totalExcludedDeducted.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s911.card.us_saved">US tax saved (approx)</div>
                    <div class="value">$${usTaxSaved.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
