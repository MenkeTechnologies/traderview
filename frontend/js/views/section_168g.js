// IRC § 168(g) — Alternative Depreciation System (ADS).
// Longer recovery periods + straight-line method. Required for: tax-exempt use property,
// listed property with ≤ 50% business use, certain imported property, farming property
// (now elective post-TCJA), § 263A(d)(3) post-2025 farming, BEAT property.
// ELECTION allowed for any class of property.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const ADS_RECOVERY_PERIODS = {
    'cars_light_trucks_3yr': 5,
    'office_equipment_5yr': 6,
    'computers_software_5yr': 5,
    'office_furniture_7yr': 10,
    'agricultural_5_7_10yr': 10,
    'land_improvements_15yr': 20,
    'qip_15yr': 20,
    'residential_rental_27_5': 30,
    'non_residential_real_39': 40,
    'horses_breeding': 12,
};

let state = {
    property_kind: 'cars_light_trucks_3yr',
    cost_basis: 0,
    placed_in_service_year: 2024,
    business_use_pct: 100,
    is_required_ads: false,
    is_elective_ads: false,
    marginal_rate: 0.32,
};

export async function renderSection168g(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s168g.h1.title">// § 168(g) ALTERNATIVE DEPRECIATION SYSTEM</span></h1>
        <p class="muted small" data-i18n="view.s168g.hint.intro">
            <strong>Longer recovery periods + straight-line method.</strong> Required for:
            tax-exempt use property, listed property with ≤ 50% business use, certain imported
            property, § 263A(d)(3) post-2025 farming, BEAT property. <strong>ELECTION allowed</strong>
            for any class. <strong>No bonus depreciation</strong> on ADS property. <strong>Trade-off:</strong>
            ADS uses slower recovery but qualifies QIP for bonus retroactively (CARES Act fix).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s168g.h2.inputs">Inputs</h2>
            <form id="s168g-form" class="inline-form">
                <label><span data-i18n="view.s168g.label.kind">Property class</span>
                    <select name="property_kind">
                        <option value="cars_light_trucks_3yr" ${state.property_kind === 'cars_light_trucks_3yr' ? 'selected' : ''}>Cars / light trucks (3-yr → 5)</option>
                        <option value="office_equipment_5yr" ${state.property_kind === 'office_equipment_5yr' ? 'selected' : ''}>Office equipment (5-yr → 6)</option>
                        <option value="computers_software_5yr" ${state.property_kind === 'computers_software_5yr' ? 'selected' : ''}>Computers + software (5-yr → 5)</option>
                        <option value="office_furniture_7yr" ${state.property_kind === 'office_furniture_7yr' ? 'selected' : ''}>Office furniture (7-yr → 10)</option>
                        <option value="agricultural_5_7_10yr" ${state.property_kind === 'agricultural_5_7_10yr' ? 'selected' : ''}>Agricultural (5/7/10-yr → 10)</option>
                        <option value="land_improvements_15yr" ${state.property_kind === 'land_improvements_15yr' ? 'selected' : ''}>Land improvements (15-yr → 20)</option>
                        <option value="qip_15yr" ${state.property_kind === 'qip_15yr' ? 'selected' : ''}>Qualified Improvement Property (15-yr → 20)</option>
                        <option value="residential_rental_27_5" ${state.property_kind === 'residential_rental_27_5' ? 'selected' : ''}>Residential rental (27.5 → 30)</option>
                        <option value="non_residential_real_39" ${state.property_kind === 'non_residential_real_39' ? 'selected' : ''}>Non-residential real (39 → 40)</option>
                        <option value="horses_breeding" ${state.property_kind === 'horses_breeding' ? 'selected' : ''}>Horses (12-yr ADS)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s168g.label.cost">Cost basis ($)</span>
                    <input type="number" step="1000" name="cost_basis" value="${state.cost_basis}"></label>
                <label><span data-i18n="view.s168g.label.placed_year">Placed in service year</span>
                    <input type="number" step="1" name="placed_in_service_year" value="${state.placed_in_service_year}"></label>
                <label><span data-i18n="view.s168g.label.business_use">Business use %</span>
                    <input type="number" step="1" name="business_use_pct" value="${state.business_use_pct}"></label>
                <label><span data-i18n="view.s168g.label.required">Required ADS?</span>
                    <input type="checkbox" name="is_required_ads" ${state.is_required_ads ? 'checked' : ''}></label>
                <label><span data-i18n="view.s168g.label.elective">Elective ADS?</span>
                    <input type="checkbox" name="is_elective_ads" ${state.is_elective_ads ? 'checked' : ''}></label>
                <label><span data-i18n="view.s168g.label.marginal">Marginal %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s168g.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s168g-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s168g.h2.required_uses">When ADS is REQUIRED</h2>
            <ul class="muted small">
                <li data-i18n="view.s168g.req.tax_exempt">Tax-exempt use property (leased to govt / nonprofit / foreign person)</li>
                <li data-i18n="view.s168g.req.50pct">Listed property used ≤ 50% for business</li>
                <li data-i18n="view.s168g.req.tax_exempt_bond">Property financed by tax-exempt bonds</li>
                <li data-i18n="view.s168g.req.imported">Imported from foreign trade restriction country</li>
                <li data-i18n="view.s168g.req.farming_post_2025">Farming property post-2025 (TCJA sunset)</li>
                <li data-i18n="view.s168g.req.beat">BEAT (§ 59A) anti-base-erosion property</li>
                <li data-i18n="view.s168g.req.foreign_owned">Foreign-use property under § 168(g)(4)</li>
                <li data-i18n="view.s168g.req.passive_pal">§ 469 passive activity property in certain cases</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s168g.h2.elective">ADS elective use scenarios</h2>
            <ul class="muted small">
                <li data-i18n="view.s168g.el.amt_minimization">Minimize AMT — straight-line method preferred for AMT</li>
                <li data-i18n="view.s168g.el.future_high_rate">Defer deduction to future higher-rate year</li>
                <li data-i18n="view.s168g.el.book_tax_alignment">Align book + tax depreciation</li>
                <li data-i18n="view.s168g.el.foreign_tax_credit">Maximize foreign tax credit using slower domestic depreciation</li>
                <li data-i18n="view.s168g.el.nol_management">Manage NOL utilization timing</li>
                <li data-i18n="view.s168g.el.class_by_class">Election made class-by-class on Form 4562 line 20</li>
                <li data-i18n="view.s168g.el.irrevocable">Election IRREVOCABLE for class + year</li>
                <li data-i18n="view.s168g.el.no_bonus">No bonus depreciation on ADS property</li>
            </ul>
        </div>
    `;
    document.getElementById('s168g-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.property_kind = fd.get('property_kind');
        state.cost_basis = Number(fd.get('cost_basis')) || 0;
        state.placed_in_service_year = Number(fd.get('placed_in_service_year')) || new Date().getFullYear();
        state.business_use_pct = Number(fd.get('business_use_pct')) || 100;
        state.is_required_ads = !!fd.get('is_required_ads');
        state.is_elective_ads = !!fd.get('is_elective_ads');
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.32;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s168g-output');
    if (!el) return;
    const recoveryPeriod = ADS_RECOVERY_PERIODS[state.property_kind] || 5;
    const businessBasis = state.cost_basis * (state.business_use_pct / 100);
    const annualDepreciation = businessBasis / recoveryPeriod;
    const yearlySavings = annualDepreciation * state.marginal_rate;
    const useAds = state.is_required_ads || state.is_elective_ads || state.business_use_pct <= 50;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s168g.h2.result">ADS depreciation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s168g.card.recovery">ADS recovery period</div>
                    <div class="value">${recoveryPeriod} ${esc(t('view.s168g.units.years'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s168g.card.business_basis">Business basis</div>
                    <div class="value">$${businessBasis.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s168g.card.annual_depreciation">Annual depreciation (SL)</div>
                    <div class="value">$${annualDepreciation.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s168g.card.yearly_savings">Yearly tax savings</div>
                    <div class="value">$${yearlySavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${useAds ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s168g.card.must_use_ads">Must use ADS</div>
                    <div class="value">${useAds ? esc(t('view.s168g.status.yes')) : esc(t('view.s168g.status.no'))}</div>
                </div>
            </div>
        </div>
    `;
}
