// IRC § 163(j) — Business Interest Expense Limitation.
// BIE deduction limited to: business interest income + 30% × ATI + floor-plan financing interest.
// Pre-2022: ATI based on EBITDA. Post-2022: ATI based on EBIT (TCJA stricter rule kicks in).
// Small biz exception: avg gross receipts ≤ $30M (2024). Real property + farming trades may elect out.
// Disallowed interest carries forward indefinitely (no carryback).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    business_interest_expense: 0,
    business_interest_income: 0,
    adjusted_taxable_income: 0,
    floor_plan_interest: 0,
    avg_gross_receipts_3yr: 0,
    real_property_election: false,
    farming_election: false,
    is_tax_shelter: false,
    pre_2022_ebitda: false,
    carryforward_prior: 0,
};

export async function renderSection163j(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s163j.h1.title">// § 163(j) BUSINESS INTEREST LIMIT</span></h1>
        <p class="muted small" data-i18n="view.s163j.hint.intro">
            <strong>BIE deduction ≤</strong> business interest INCOME + 30% × ATI + floor-plan financing.
            <strong>Pre-2022:</strong> ATI based on EBITDA (more generous). <strong>Post-2022:</strong>
            ATI based on EBIT (TCJA stricter — depreciation + amortization excluded). <strong>Small biz
            exception:</strong> ≤ $30M avg gross receipts (3-yr). Real property + farming trades may
            <strong>irrevocably elect out</strong> (cost: ADS depreciation). Disallowed interest
            carries forward indefinitely (corp + partnership pass-through with EBIE rules).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s163j.h2.inputs">Inputs</h2>
            <form id="s163j-form" class="inline-form">
                <label><span data-i18n="view.s163j.label.bie">Business interest expense ($)</span>
                    <input type="number" step="0.01" name="business_interest_expense" value="${state.business_interest_expense}"></label>
                <label><span data-i18n="view.s163j.label.bii">Business interest income ($)</span>
                    <input type="number" step="0.01" name="business_interest_income" value="${state.business_interest_income}"></label>
                <label><span data-i18n="view.s163j.label.ati">Adjusted Taxable Income ($)</span>
                    <input type="number" step="0.01" name="adjusted_taxable_income" value="${state.adjusted_taxable_income}"></label>
                <label><span data-i18n="view.s163j.label.floor">Floor-plan financing interest ($)</span>
                    <input type="number" step="0.01" name="floor_plan_interest" value="${state.floor_plan_interest}"></label>
                <label><span data-i18n="view.s163j.label.gross">Avg gross receipts 3-yr ($)</span>
                    <input type="number" step="0.01" name="avg_gross_receipts_3yr" value="${state.avg_gross_receipts_3yr}"></label>
                <label><span data-i18n="view.s163j.label.real">Real property trade elected out?</span>
                    <input type="checkbox" name="real_property_election" ${state.real_property_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s163j.label.farming">Farming trade elected out?</span>
                    <input type="checkbox" name="farming_election" ${state.farming_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s163j.label.shelter">Tax shelter?</span>
                    <input type="checkbox" name="is_tax_shelter" ${state.is_tax_shelter ? 'checked' : ''}></label>
                <label><span data-i18n="view.s163j.label.ebitda">Pre-2022 EBITDA basis?</span>
                    <input type="checkbox" name="pre_2022_ebitda" ${state.pre_2022_ebitda ? 'checked' : ''}></label>
                <label><span data-i18n="view.s163j.label.carry_prior">Prior-year carryforward ($)</span>
                    <input type="number" step="0.01" name="carryforward_prior" value="${state.carryforward_prior}"></label>
                <button class="primary" type="submit" data-i18n="view.s163j.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s163j-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s163j.h2.exemptions">Exemptions + elections</h2>
            <ul class="muted small">
                <li data-i18n="view.s163j.exempt.small">§ 163(j)(3) small business: avg gross receipts ≤ $30M (2024)</li>
                <li data-i18n="view.s163j.exempt.tax_shelter">Tax shelter NEVER qualifies regardless of size</li>
                <li data-i18n="view.s163j.exempt.real">Real property trade: irrevocable elect-out at cost of ADS depreciation (40-yr nonres)</li>
                <li data-i18n="view.s163j.exempt.farming">Farming business: irrevocable elect-out at cost of ADS depreciation</li>
                <li data-i18n="view.s163j.exempt.utility">Electing utility business: exempt for regulated rate base</li>
                <li data-i18n="view.s163j.exempt.consumer">Floor-plan financing (auto, boat, RV dealers): always fully deductible</li>
                <li data-i18n="view.s163j.exempt.relative">Partner BIE limit applied at partnership level (not partner level) — EBIE allocation</li>
                <li data-i18n="view.s163j.exempt.s_corp">S-corp limit applied at corporate level (no partner-style EBIE)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s163j.h2.ati">ATI computation (post-2022)</h2>
            <ol class="muted small">
                <li data-i18n="view.s163j.ati.start">Start: Taxable income</li>
                <li data-i18n="view.s163j.ati.add_bie">Add back: Business interest expense</li>
                <li data-i18n="view.s163j.ati.sub_bii">Subtract: Business interest income</li>
                <li data-i18n="view.s163j.ati.add_nol">Add back: NOLs</li>
                <li data-i18n="view.s163j.ati.add_199a">Add back: § 199A deduction</li>
                <li data-i18n="view.s163j.ati.add_capital">Add back: Capital loss carryback</li>
                <li data-i18n="view.s163j.ati.post_2022">Post-2022: do NOT add back D + A (depreciation + amortization stays out of ATI)</li>
                <li data-i18n="view.s163j.ati.pre_2022">Pre-2022: ADD back D + A (EBITDA-style)</li>
            </ol>
        </div>
    `;
    document.getElementById('s163j-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.business_interest_expense = Number(fd.get('business_interest_expense')) || 0;
        state.business_interest_income = Number(fd.get('business_interest_income')) || 0;
        state.adjusted_taxable_income = Number(fd.get('adjusted_taxable_income')) || 0;
        state.floor_plan_interest = Number(fd.get('floor_plan_interest')) || 0;
        state.avg_gross_receipts_3yr = Number(fd.get('avg_gross_receipts_3yr')) || 0;
        state.real_property_election = !!fd.get('real_property_election');
        state.farming_election = !!fd.get('farming_election');
        state.is_tax_shelter = !!fd.get('is_tax_shelter');
        state.pre_2022_ebitda = !!fd.get('pre_2022_ebitda');
        state.carryforward_prior = Number(fd.get('carryforward_prior')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s163j-output');
    if (!el) return;
    const smallBizExempt = state.avg_gross_receipts_3yr <= 30_000_000 && !state.is_tax_shelter;
    const electedOut = state.real_property_election || state.farming_election;
    const isExempt = smallBizExempt || electedOut;
    const limit = state.business_interest_income + 0.30 * state.adjusted_taxable_income + state.floor_plan_interest;
    const totalBIE = state.business_interest_expense + state.carryforward_prior;
    const allowed = isExempt ? totalBIE : Math.min(totalBIE, limit);
    const disallowed = Math.max(0, totalBIE - allowed);
    const taxSavings = allowed * 0.21;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s163j.h2.result">§ 163(j) computation</h2>
            <div class="cards">
                <div class="card ${isExempt ? 'pos' : ''}">
                    <div class="label" data-i18n="view.s163j.card.exempt">Exempt from limit?</div>
                    <div class="value">${isExempt ? esc(t('view.s163j.status.yes')) : esc(t('view.s163j.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s163j.card.limit">Deduction limit</div>
                    <div class="value">$${limit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s163j.card.total_bie">Total BIE (current + CF)</div>
                    <div class="value">$${totalBIE.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s163j.card.allowed">Allowed deduction</div>
                    <div class="value">$${allowed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${disallowed > 0 ? 'neg' : ''}">
                    <div class="label" data-i18n="view.s163j.card.disallowed">Disallowed (CF indef.)</div>
                    <div class="value">$${disallowed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s163j.card.tax_savings">Tax savings (21%)</div>
                    <div class="value">$${taxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${electedOut ? `
                <p class="muted small" style="margin-top:10px" data-i18n="view.s163j.elect_note">
                    Irrevocable elect-out triggers ADS depreciation requirement (40-yr nonres real,
                    30-yr res real, longer farming class lives). Permanent — cannot reverse.
                </p>
            ` : ''}
        </div>
    `;
}
