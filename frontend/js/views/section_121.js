// IRC § 121 — Principal Residence Capital Gain Exclusion.
// $250k single / $500k MFJ excluded from gain. Ownership test: ≥ 2 of last 5 yrs.
// Use test: ≥ 2 of last 5 yrs as principal residence. Once per 2 years.
// Non-qualified-use period (rentals after 2008): allocate gain pro-rata.
// Job/health/unforeseen circumstance: partial exclusion (months / 24).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const EXCL_SINGLE = 250_000;
const EXCL_MFJ = 500_000;

let state = {
    filing_status: 'single',
    purchase_year: new Date().getFullYear() - 5,
    purchase_month: 1,
    sale_year: new Date().getFullYear(),
    sale_month: 1,
    purchase_price: 0,
    improvements: 0,
    sale_price: 0,
    sale_costs: 0,
    years_used_as_residence: 5,
    years_used_as_rental: 0,
    depreciation_taken: 0,
    partial_qualifying_reason: false,
    months_qualifying: 0,
};

export async function renderSection121(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s121.h1.title">// § 121 PRINCIPAL RESIDENCE EXCLUSION</span></h1>
        <p class="muted small" data-i18n="view.s121.hint.intro">
            <strong>$250k single / $500k MFJ</strong> excluded from gain on sale of principal
            residence. <strong>Ownership test:</strong> ≥ 2 of last 5 yrs. <strong>Use test:</strong>
            ≥ 2 of last 5 yrs as principal residence. Once per 2 years. Post-2008 NON-QUALIFIED USE
            (rentals, vacation) allocated pro-rata. Job/health/unforeseen circumstance:
            partial exclusion (months ÷ 24).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s121.h2.inputs">Inputs</h2>
            <form id="s121-form" class="inline-form">
                <label><span data-i18n="view.s121.label.filing">Filing status</span>
                    <select name="filing_status">
                        <option value="single" ${state.filing_status === 'single' ? 'selected' : ''}>Single / HoH / MFS</option>
                        <option value="mfj" ${state.filing_status === 'mfj' ? 'selected' : ''}>MFJ</option>
                    </select>
                </label>
                <label><span data-i18n="view.s121.label.purchase_year">Purchase year</span>
                    <input type="number" step="1" name="purchase_year" value="${state.purchase_year}"></label>
                <label><span data-i18n="view.s121.label.sale_year">Sale year</span>
                    <input type="number" step="1" name="sale_year" value="${state.sale_year}"></label>
                <label><span data-i18n="view.s121.label.purchase_price">Purchase price ($)</span>
                    <input type="number" step="100" name="purchase_price" value="${state.purchase_price}"></label>
                <label><span data-i18n="view.s121.label.improvements">Capital improvements ($)</span>
                    <input type="number" step="100" name="improvements" value="${state.improvements}"></label>
                <label><span data-i18n="view.s121.label.sale_price">Sale price ($)</span>
                    <input type="number" step="100" name="sale_price" value="${state.sale_price}"></label>
                <label><span data-i18n="view.s121.label.sale_costs">Selling costs (commission, etc.) ($)</span>
                    <input type="number" step="100" name="sale_costs" value="${state.sale_costs}"></label>
                <label><span data-i18n="view.s121.label.years_residence">Years used as residence</span>
                    <input type="number" step="0.1" name="years_used_as_residence" value="${state.years_used_as_residence}"></label>
                <label><span data-i18n="view.s121.label.years_rental">Years rented out (post-2008)</span>
                    <input type="number" step="0.1" name="years_used_as_rental" value="${state.years_used_as_rental}"></label>
                <label><span data-i18n="view.s121.label.depreciation">Depreciation taken ($)</span>
                    <input type="number" step="100" name="depreciation_taken" value="${state.depreciation_taken}"></label>
                <label><span data-i18n="view.s121.label.partial">Partial qualifying (job/health/UC)?</span>
                    <input type="checkbox" name="partial_qualifying_reason" ${state.partial_qualifying_reason ? 'checked' : ''}></label>
                <label><span data-i18n="view.s121.label.months_qualifying">Months actually used / owned (if partial)</span>
                    <input type="number" step="1" name="months_qualifying" value="${state.months_qualifying}"></label>
                <button class="primary" type="submit" data-i18n="view.s121.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s121-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s121.h2.special">Special situations</h2>
            <ul class="muted small">
                <li data-i18n="view.s121.spec.widow">Surviving spouse: $500k MFJ exclusion preserved if sold within 2 years of spouse death</li>
                <li data-i18n="view.s121.spec.divorce">Divorce: time tacks via § 121(d)(3) for transferred-in spouse</li>
                <li data-i18n="view.s121.spec.disabled">Disabled person facility care counts as residence use</li>
                <li data-i18n="view.s121.spec.military">Military / Foreign Service / Intel: 10-yr suspended period</li>
                <li data-i18n="view.s121.spec.dep_recapture">§ 1250 unrecaptured dep recapture EXCLUDED from § 121 — taxed at 25% max</li>
                <li data-i18n="view.s121.spec.no_capital_loss">Personal residence: § 165 NO loss deductible (only investment / rental)</li>
            </ul>
        </div>
    `;
    document.getElementById('s121-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.filing_status = fd.get('filing_status');
        state.purchase_year = Number(fd.get('purchase_year'));
        state.sale_year = Number(fd.get('sale_year'));
        state.purchase_price = Number(fd.get('purchase_price')) || 0;
        state.improvements = Number(fd.get('improvements')) || 0;
        state.sale_price = Number(fd.get('sale_price')) || 0;
        state.sale_costs = Number(fd.get('sale_costs')) || 0;
        state.years_used_as_residence = Number(fd.get('years_used_as_residence')) || 0;
        state.years_used_as_rental = Number(fd.get('years_used_as_rental')) || 0;
        state.depreciation_taken = Number(fd.get('depreciation_taken')) || 0;
        state.partial_qualifying_reason = !!fd.get('partial_qualifying_reason');
        state.months_qualifying = Number(fd.get('months_qualifying')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s121-output');
    if (!el) return;
    const adjustedBasis = state.purchase_price + state.improvements - state.depreciation_taken;
    const totalAmount = state.sale_price - state.sale_costs;
    const totalGain = totalAmount - adjustedBasis;
    const recapture = Math.min(state.depreciation_taken, totalGain);  // § 1250 unrecaptured (max 25%)
    const gainAfterRecapture = totalGain - recapture;
    const totalOwnershipYears = state.years_used_as_residence + state.years_used_as_rental;
    const nonQualifiedRatio = totalOwnershipYears > 0
        ? state.years_used_as_rental / totalOwnershipYears
        : 0;
    const nonQualifiedGain = gainAfterRecapture * nonQualifiedRatio;
    const qualifiedGain = gainAfterRecapture - nonQualifiedGain;
    let exclusionCap = state.filing_status === 'mfj' ? EXCL_MFJ : EXCL_SINGLE;
    const passesOwnership = state.years_used_as_residence >= 2 || state.partial_qualifying_reason;
    if (state.partial_qualifying_reason && state.months_qualifying < 24) {
        exclusionCap = (exclusionCap * state.months_qualifying) / 24;
    }
    const excluded = passesOwnership ? Math.min(qualifiedGain, exclusionCap) : 0;
    const taxableLT = Math.max(0, gainAfterRecapture - excluded) + Math.max(0, gainAfterRecapture - qualifiedGain - excluded);
    // Simplified: just sum
    const totalTaxable = recapture + Math.max(0, gainAfterRecapture - excluded);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s121.h2.result">Exclusion calculation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s121.card.amount_realized">Amount realized (net)</div>
                    <div class="value">$${totalAmount.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s121.card.basis">Adjusted basis</div>
                    <div class="value">$${adjustedBasis.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s121.card.total_gain">Total gain</div>
                    <div class="value">$${totalGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${recapture > 0 ? 'neg' : ''}">
                    <div class="label" data-i18n="view.s121.card.dep_recapture">§ 1250 dep recapture (25% max)</div>
                    <div class="value">$${recapture.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${nonQualifiedGain > 0 ? 'neg' : ''}">
                    <div class="label" data-i18n="view.s121.card.non_qualified">Non-qualified use gain (post-2008)</div>
                    <div class="value">$${nonQualifiedGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s121.card.exclusion_cap">Exclusion cap</div>
                    <div class="value">$${exclusionCap.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s121.card.excluded">Excluded from income</div>
                    <div class="value">$${excluded.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${totalTaxable > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s121.card.taxable">Total taxable</div>
                    <div class="value">$${totalTaxable.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${passesOwnership ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s121.card.passes">Passes 2-of-5 test</div>
                    <div class="value">${passesOwnership ? esc(t('view.s121.status.yes')) : esc(t('view.s121.status.no'))}</div>
                </div>
            </div>
        </div>
    `;
}
