// IRC § 467 — Deferred Rent / Stepped Rent Agreements.
// Required: rental property agreements ≥ $250K total rent + deferred / front-loaded / stepped rents.
// Constant Rental Accrual (CRA): tax follows economic accrual, not cash payments.
// Time-value-of-money imputed rent treated as TAX BEYOND legal rent.
// Disqualified leaseback / longer than 75% useful life triggers harsher accrual.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    total_rent_obligation: 0,
    lease_term_years: 0,
    payment_pattern: 'stepped_increasing',
    is_leaseback: false,
    is_long_term_lease: false,
    useful_life_pct: 0,
    is_safe_harbor_consistent: true,
    afr_rate: 4.5,
    accrual_method: 'cra',
    current_year_payment: 0,
    cumulative_imputed_rent: 0,
    has_section_467_provisions: false,
};

export async function renderSection467(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s467.h1.title">// § 467 DEFERRED / STEPPED RENT</span></h1>
        <p class="muted small" data-i18n="view.s467.hint.intro">
            <strong>Required</strong> for rental property agreements ≥ <strong>$250K total rent</strong> +
            <strong>deferred / front-loaded / stepped rents</strong>. <strong>Constant Rental Accrual (CRA):</strong>
            tax follows economic accrual, not cash payments. Time-value-of-money <strong>imputed interest</strong>
            applied. <strong>Disqualified leaseback</strong> OR lease term &gt; 75% useful life triggers harsher
            <strong>CRA mandatory</strong>. Forms: Schedule E (real estate) / Schedule K (partnership) + § 467
            statement. Aligns lessor + lessee timing.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s467.h2.inputs">Inputs</h2>
            <form id="s467-form" class="inline-form">
                <label><span data-i18n="view.s467.label.total">Total rent obligation ($)</span>
                    <input type="number" step="0.01" name="total_rent_obligation" value="${state.total_rent_obligation}"></label>
                <label><span data-i18n="view.s467.label.term">Lease term years</span>
                    <input type="number" step="0.5" name="lease_term_years" value="${state.lease_term_years}"></label>
                <label><span data-i18n="view.s467.label.pattern">Payment pattern</span>
                    <select name="payment_pattern">
                        <option value="level" ${state.payment_pattern === 'level' ? 'selected' : ''}>Level (constant)</option>
                        <option value="stepped_increasing" ${state.payment_pattern === 'stepped_increasing' ? 'selected' : ''}>Stepped increasing</option>
                        <option value="stepped_decreasing" ${state.payment_pattern === 'stepped_decreasing' ? 'selected' : ''}>Stepped decreasing</option>
                        <option value="front_loaded" ${state.payment_pattern === 'front_loaded' ? 'selected' : ''}>Front-loaded</option>
                        <option value="deferred" ${state.payment_pattern === 'deferred' ? 'selected' : ''}>Deferred (large catch-up)</option>
                        <option value="balloon" ${state.payment_pattern === 'balloon' ? 'selected' : ''}>Balloon payment at end</option>
                    </select>
                </label>
                <label><span data-i18n="view.s467.label.leaseback">Sale-leaseback?</span>
                    <input type="checkbox" name="is_leaseback" ${state.is_leaseback ? 'checked' : ''}></label>
                <label><span data-i18n="view.s467.label.long">Long-term lease?</span>
                    <input type="checkbox" name="is_long_term_lease" ${state.is_long_term_lease ? 'checked' : ''}></label>
                <label><span data-i18n="view.s467.label.useful">Lease term as % of useful life</span>
                    <input type="number" step="0.1" name="useful_life_pct" value="${state.useful_life_pct}"></label>
                <label><span data-i18n="view.s467.label.consistent">Consistent reporting safe harbor?</span>
                    <input type="checkbox" name="is_safe_harbor_consistent" ${state.is_safe_harbor_consistent ? 'checked' : ''}></label>
                <label><span data-i18n="view.s467.label.afr">AFR rate %</span>
                    <input type="number" step="0.1" name="afr_rate" value="${state.afr_rate}"></label>
                <label><span data-i18n="view.s467.label.method">Accrual method</span>
                    <select name="accrual_method">
                        <option value="cra" ${state.accrual_method === 'cra' ? 'selected' : ''}>CRA — Constant Rental Accrual</option>
                        <option value="economic" ${state.accrual_method === 'economic' ? 'selected' : ''}>Economic accrual (payments as scheduled)</option>
                        <option value="prepaid" ${state.accrual_method === 'prepaid' ? 'selected' : ''}>Prepaid: include when received</option>
                    </select>
                </label>
                <label><span data-i18n="view.s467.label.current">Current year cash payment ($)</span>
                    <input type="number" step="0.01" name="current_year_payment" value="${state.current_year_payment}"></label>
                <label><span data-i18n="view.s467.label.cumulative">Cumulative imputed rent ($)</span>
                    <input type="number" step="0.01" name="cumulative_imputed_rent" value="${state.cumulative_imputed_rent}"></label>
                <label><span data-i18n="view.s467.label.provisions">§ 467 provisions in lease?</span>
                    <input type="checkbox" name="has_section_467_provisions" ${state.has_section_467_provisions ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s467.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s467-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s467.h2.when_applies">When § 467 applies</h2>
            <ul class="muted small">
                <li data-i18n="view.s467.app.250K">Lease w/ total rent ≥ $250,000 (any period)</li>
                <li data-i18n="view.s467.app.increasing">Stepped rent: increasing in amount across periods</li>
                <li data-i18n="view.s467.app.deferred">Deferred rent: large back-end payment</li>
                <li data-i18n="view.s467.app.prepaid">Prepaid rent: amount not for current period</li>
                <li data-i18n="view.s467.app.leaseback">Sale-leaseback: § 467 mandatory CRA</li>
                <li data-i18n="view.s467.app.long_term">Long-term lease: &gt; 75% of property useful life triggers CRA</li>
                <li data-i18n="view.s467.app.short_term">Short-term: ≤ 1 yr exempt (or "constant" or "level" rents)</li>
                <li data-i18n="view.s467.app.public_lease">Pub. lease (LPI under state law): § 467 may not apply</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s467.h2.cra">CRA — Constant Rental Accrual</h2>
            <ul class="muted small">
                <li data-i18n="view.s467.cra.calc">Calculate: present-value level annuity over lease term, AFR applied</li>
                <li data-i18n="view.s467.cra.report">Report this LEVEL amount per year regardless of payments</li>
                <li data-i18n="view.s467.cra.imputed">Difference between CRA and cash payment = IMPUTED INTEREST (separate income / deduction)</li>
                <li data-i18n="view.s467.cra.lessor">Lessor: rent income at CRA; interest income on imputed</li>
                <li data-i18n="view.s467.cra.lessee">Lessee: rent expense at CRA; interest expense on imputed (subject to § 163(j))</li>
                <li data-i18n="view.s467.cra.symmetry">SYMMETRY: parties report consistently (Form 8978 reconciliation)</li>
                <li data-i18n="view.s467.cra.gaap">GAAP straight-line lease accounting often aligns w/ CRA (with adjustments)</li>
                <li data-i18n="view.s467.cra.disqualified">"Disqualified" lease (sale-leaseback / tax-avoidance): CRA forced even if not stepped</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s467.h2.examples">Common § 467 patterns</h2>
            <ul class="muted small">
                <li data-i18n="view.s467.ex.rent_increases">Annual 3% rent increases — typical stepped pattern (CPI escalation)</li>
                <li data-i18n="view.s467.ex.tenant_improvements">Tenant improvement allowance + free rent month: § 467 prepaid analysis</li>
                <li data-i18n="view.s467.ex.percentage_rent">Percentage rent (retail): § 467 if minimum + sales-based escalators</li>
                <li data-i18n="view.s467.ex.balloon">Balloon payment at end: CRA forces level accrual; large interest imputed</li>
                <li data-i18n="view.s467.ex.holiday_period">Initial rent holiday (60-90 days): defer + spread on CRA</li>
                <li data-i18n="view.s467.ex.master_lease">Master lease + ancillary: each component analyzed separately</li>
                <li data-i18n="view.s467.ex.cam_charges">CAM (Common Area Maintenance): typically excluded from § 467 (operating exp)</li>
                <li data-i18n="view.s467.ex.gross_up">Gross-up rent provisions: treated as additional rent obligation</li>
            </ul>
        </div>
    `;
    document.getElementById('s467-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.total_rent_obligation = Number(fd.get('total_rent_obligation')) || 0;
        state.lease_term_years = Number(fd.get('lease_term_years')) || 0;
        state.payment_pattern = fd.get('payment_pattern');
        state.is_leaseback = !!fd.get('is_leaseback');
        state.is_long_term_lease = !!fd.get('is_long_term_lease');
        state.useful_life_pct = Number(fd.get('useful_life_pct')) || 0;
        state.is_safe_harbor_consistent = !!fd.get('is_safe_harbor_consistent');
        state.afr_rate = Number(fd.get('afr_rate')) || 0;
        state.accrual_method = fd.get('accrual_method');
        state.current_year_payment = Number(fd.get('current_year_payment')) || 0;
        state.cumulative_imputed_rent = Number(fd.get('cumulative_imputed_rent')) || 0;
        state.has_section_467_provisions = !!fd.get('has_section_467_provisions');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s467-output');
    if (!el) return;
    const applies = state.total_rent_obligation >= 250_000 && state.lease_term_years > 1;
    const longTermTrigger = state.useful_life_pct >= 75 || state.is_leaseback;
    const annualCRA = state.lease_term_years > 0 ? state.total_rent_obligation / state.lease_term_years : 0;
    const imputed = state.current_year_payment > 0 ? annualCRA - state.current_year_payment : annualCRA;
    const imputedInterest = state.cumulative_imputed_rent * (state.afr_rate / 100);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s467.h2.result">§ 467 outcome</h2>
            <div class="cards">
                <div class="card ${applies ? 'pos' : ''}">
                    <div class="label" data-i18n="view.s467.card.applies">§ 467 applies?</div>
                    <div class="value">${applies ? esc(t('view.s467.status.yes')) : esc(t('view.s467.status.no'))}</div>
                </div>
                <div class="card ${longTermTrigger ? 'neg' : ''}">
                    <div class="label" data-i18n="view.s467.card.long_term">Long-term / leaseback CRA?</div>
                    <div class="value">${longTermTrigger ? esc(t('view.s467.status.yes')) : esc(t('view.s467.status.no'))}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s467.card.cra">Annual CRA</div>
                    <div class="value">$${annualCRA.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s467.card.cash">Cash payment current yr</div>
                    <div class="value">$${state.current_year_payment.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${imputed > 0 ? 'neg' : ''}">
                    <div class="label" data-i18n="view.s467.card.imputed">Imputed rent (deferred)</div>
                    <div class="value">$${imputed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s467.card.interest">Imputed interest (AFR)</div>
                    <div class="value">$${imputedInterest.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${longTermTrigger ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s467.long_note">
                    Long-term lease (&gt; 75% useful life) OR sale-leaseback: CRA mandatory regardless of
                    rent pattern. Plan for level accrual + imputed interest income / expense. Coordinate
                    with § 163(j) interest limit on lessee side. AFR rate impacts deferral cost significantly.
                </p>
            ` : ''}
        </div>
    `;
}
