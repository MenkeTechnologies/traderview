// IRC § 45W — Commercial Clean Vehicle Credit (IRA 2022).
// $7,500 (≤ 14K lbs GVWR) OR $40K (14K+ GVWR) — LESSER of (1) 15% basis (30% if BEV/FCV)
// OR (2) incremental cost over comparable ICE.
// NO income limit, NO MSRP cap, NO domestic assembly — "lease loophole" to bypass § 30D limits.
// Commercial / business use only; tax-exempt eligible via § 6417 direct pay.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    vehicle_basis: 0,
    incremental_cost: 0,
    gvwr_lbs: 0,
    is_bev_or_fcv: true,
    is_phev: false,
    placed_in_service_year: 2024,
    business_use_pct: 100,
    is_lease: false,
    is_tax_exempt: false,
    elect_direct_pay: false,
    elect_transferability: false,
    vin_documented: true,
};

export async function renderSection45W(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s45W.h1.title">// § 45W COMMERCIAL CLEAN VEHICLE</span></h1>
        <p class="muted small" data-i18n="view.s45W.hint.intro">
            <strong>$7,500</strong> (≤ 14,000 lbs GVWR) OR <strong>$40,000</strong> (14K+ GVWR — heavy duty).
            <strong>LESSER of:</strong> (1) <strong>15% basis</strong> (30% if BEV/FCV) OR (2) <strong>incremental
            cost</strong> over comparable ICE vehicle. <strong>NO income limit, NO MSRP cap, NO domestic
            assembly</strong> — the "lease loophole" bypassing § 30D limits. <strong>Commercial / business
            use only.</strong> <strong>Tax-exempts:</strong> § 6417 direct pay; <strong>taxable:</strong>
            § 6418 transferability. <strong>Lease "loophole":</strong> dealer claims § 45W → passes through
            as lease discount. Form 8936.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s45W.h2.inputs">Inputs</h2>
            <form id="s45W-form" class="inline-form">
                <label><span data-i18n="view.s45W.label.basis">Vehicle basis ($)</span>
                    <input type="number" step="1000" name="vehicle_basis" value="${state.vehicle_basis}"></label>
                <label><span data-i18n="view.s45W.label.incremental">Incremental cost vs ICE ($)</span>
                    <input type="number" step="500" name="incremental_cost" value="${state.incremental_cost}"></label>
                <label><span data-i18n="view.s45W.label.gvwr">GVWR (lbs)</span>
                    <input type="number" step="100" name="gvwr_lbs" value="${state.gvwr_lbs}"></label>
                <label><span data-i18n="view.s45W.label.bev_fcv">BEV or FCV (30% rate)?</span>
                    <input type="checkbox" name="is_bev_or_fcv" ${state.is_bev_or_fcv ? 'checked' : ''}></label>
                <label><span data-i18n="view.s45W.label.phev">PHEV (15% rate)?</span>
                    <input type="checkbox" name="is_phev" ${state.is_phev ? 'checked' : ''}></label>
                <label><span data-i18n="view.s45W.label.year">Placed in service year</span>
                    <input type="number" step="1" name="placed_in_service_year" value="${state.placed_in_service_year}"></label>
                <label><span data-i18n="view.s45W.label.biz_pct">Business use %</span>
                    <input type="number" step="0.1" name="business_use_pct" value="${state.business_use_pct}"></label>
                <label><span data-i18n="view.s45W.label.lease">Lease structure?</span>
                    <input type="checkbox" name="is_lease" ${state.is_lease ? 'checked' : ''}></label>
                <label><span data-i18n="view.s45W.label.exempt">Tax-exempt entity?</span>
                    <input type="checkbox" name="is_tax_exempt" ${state.is_tax_exempt ? 'checked' : ''}></label>
                <label><span data-i18n="view.s45W.label.direct">§ 6417 direct pay?</span>
                    <input type="checkbox" name="elect_direct_pay" ${state.elect_direct_pay ? 'checked' : ''}></label>
                <label><span data-i18n="view.s45W.label.transfer">§ 6418 transferability?</span>
                    <input type="checkbox" name="elect_transferability" ${state.elect_transferability ? 'checked' : ''}></label>
                <label><span data-i18n="view.s45W.label.vin">VIN documented?</span>
                    <input type="checkbox" name="vin_documented" ${state.vin_documented ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s45W.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s45W-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s45W.h2.eligibility">Eligibility cascade</h2>
            <ul class="muted small">
                <li data-i18n="view.s45W.elig.qualifying">Qualifying vehicle: BEV / FCV / PHEV with battery ≥ 15 kWh (or 7 kWh light duty)</li>
                <li data-i18n="view.s45W.elig.taxpayer">Acquired by taxpayer for USE OR LEASE</li>
                <li data-i18n="view.s45W.elig.business">USED in TRADE OR BUSINESS (Schedule C, F, Form 1120, etc.)</li>
                <li data-i18n="view.s45W.elig.no_income">NO income limit on taxpayer</li>
                <li data-i18n="view.s45W.elig.no_msrp">NO MSRP cap</li>
                <li data-i18n="view.s45W.elig.no_assembly">NO North American assembly requirement (unlike § 30D)</li>
                <li data-i18n="view.s45W.elig.no_minerals">NO critical minerals + battery components requirements</li>
                <li data-i18n="view.s45W.elig.no_resale">NOT acquired for resale (dealer inventory excluded)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s45W.h2.lease_loophole">"Lease loophole" — leveraging § 45W via § 30D</h2>
            <ul class="muted small">
                <li data-i18n="view.s45W.lease.mechanism">Mechanism: leasing company (dealer / captive finance) claims § 45W</li>
                <li data-i18n="view.s45W.lease.pass_through">Dealer passes credit to lessee via reduced monthly payment or down payment credit</li>
                <li data-i18n="view.s45W.lease.bypass_30d">Lessee bypasses ALL § 30D limits: income, MSRP, assembly, minerals/components</li>
                <li data-i18n="view.s45W.lease.kia_hyundai">Particularly used by Kia / Hyundai (assembled in Korea — failed § 30D NA test)</li>
                <li data-i18n="view.s45W.lease.high_income">Used by HIGH-INCOME consumers who fail § 30D MAGI limit</li>
                <li data-i18n="view.s45W.lease.statutory_blessing">Treasury Notice 2022-58 + IR-2023-24 confirmed eligibility</li>
                <li data-i18n="view.s45W.lease.lobby_concerns">UAW + congressional concerns; some attempted closure but unchanged</li>
                <li data-i18n="view.s45W.lease.audit_risk">Watch IRS audit — must be GENUINE lease (not disguised sale)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s45W.h2.compare_30d">Compare § 45W vs § 30D</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s45W.th.feature">Feature</th>
                    <th data-i18n="view.s45W.th.45w">§ 45W Commercial</th>
                    <th data-i18n="view.s45W.th.30d">§ 30D Personal</th>
                </tr></thead>
                <tbody>
                    <tr><td>Max credit</td><td>$7,500 / $40K (heavy)</td><td>$7,500</td></tr>
                    <tr><td>Income limit</td><td>NONE</td><td>$150K / $225K / $300K MAGI</td></tr>
                    <tr><td>MSRP cap</td><td>NONE</td><td>$55K cars / $80K SUV+</td></tr>
                    <tr><td>NA assembly required</td><td>NO</td><td>YES</td></tr>
                    <tr><td>Critical minerals</td><td>NOT required</td><td>50%+ rising to 90%</td></tr>
                    <tr><td>Battery components</td><td>NOT required</td><td>60%+ rising to 100%</td></tr>
                    <tr><td>Battery capacity min</td><td>7-15 kWh</td><td>7 kWh PHEV</td></tr>
                    <tr><td>Tax-exempt eligible</td><td>YES (§ 6417 direct pay)</td><td>YES (transfer to dealer)</td></tr>
                    <tr><td>Transferability</td><td>YES (§ 6418)</td><td>NO (dealer transfer at POS)</td></tr>
                    <tr><td>Carryforward</td><td>20 yrs (§ 39 GBC)</td><td>None (offset in year)</td></tr>
                </tbody>
            </table>
        </div>
    `;
    document.getElementById('s45W-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.vehicle_basis = Number(fd.get('vehicle_basis')) || 0;
        state.incremental_cost = Number(fd.get('incremental_cost')) || 0;
        state.gvwr_lbs = Number(fd.get('gvwr_lbs')) || 0;
        state.is_bev_or_fcv = !!fd.get('is_bev_or_fcv');
        state.is_phev = !!fd.get('is_phev');
        state.placed_in_service_year = Number(fd.get('placed_in_service_year')) || 0;
        state.business_use_pct = Number(fd.get('business_use_pct')) || 0;
        state.is_lease = !!fd.get('is_lease');
        state.is_tax_exempt = !!fd.get('is_tax_exempt');
        state.elect_direct_pay = !!fd.get('elect_direct_pay');
        state.elect_transferability = !!fd.get('elect_transferability');
        state.vin_documented = !!fd.get('vin_documented');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s45W-output');
    if (!el) return;
    const rate = state.is_bev_or_fcv ? 0.30 : 0.15;
    const isHeavy = state.gvwr_lbs >= 14_000;
    const credit_cap = isHeavy ? 40_000 : 7_500;
    const basis_credit = state.vehicle_basis * rate;
    const lesser = Math.min(basis_credit, state.incremental_cost);
    const cred_before_cap = Math.min(lesser, credit_cap);
    const credit_business_pct = cred_before_cap * (state.business_use_pct / 100);
    const transferProceeds = state.elect_transferability ? credit_business_pct * 0.92 : credit_business_pct;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s45W.h2.result">§ 45W credit computation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s45W.card.rate">Rate</div>
                    <div class="value">${(rate * 100).toFixed(0)}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s45W.card.heavy">Heavy duty (14K+ lbs)?</div>
                    <div class="value">${isHeavy ? esc(t('view.s45W.status.yes')) : esc(t('view.s45W.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s45W.card.cap">Statutory cap</div>
                    <div class="value">$${credit_cap.toLocaleString()}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s45W.card.basis">Basis × rate</div>
                    <div class="value">$${basis_credit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s45W.card.incremental">Incremental cost</div>
                    <div class="value">$${state.incremental_cost.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s45W.card.credit">Final credit</div>
                    <div class="value">$${credit_business_pct.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s45W.card.transfer">Transfer cash (92%)</div>
                    <div class="value">$${transferProceeds.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.is_lease ? `
                <p class="muted small pos" style="margin-top:10px" data-i18n="view.s45W.lease_note">
                    Lease structure: dealer (lessor) claims § 45W → passes through as discount.
                    Bypasses § 30D income / MSRP / NA-assembly limits. Major route for HIGH-INCOME consumers
                    and foreign-built vehicles. Verify genuine lease structure (not disguised sale) to maintain
                    legitimate § 45W eligibility.
                </p>
            ` : ''}
        </div>
    `;
}
