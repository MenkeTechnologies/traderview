// IRC § 30D — Clean Vehicle Credit (NEW EV).
// Up to $7,500: $3,750 critical minerals + $3,750 battery components (IRA 2022).
// Income limit: $300K MFJ / $225K HoH / $150K single (MAGI).
// MSRP cap: $80K SUVs/vans/trucks, $55K cars.
// Final assembly: MUST occur in NORTH AMERICA. List on FuelEconomy.gov.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    msrp: 0,
    vehicle_type: 'car',
    final_assembly_in_na: false,
    critical_minerals_qualifies: false,
    battery_components_qualifies: false,
    magi: 0,
    filing_status: 'single',
    purchase_year: 2024,
    transfer_at_dealer: false,
    business_use_pct: 0,
    placed_in_service_date: '',
};

export async function renderSection30D(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s30D.h1.title">// § 30D CLEAN VEHICLE</span></h1>
        <p class="muted small" data-i18n="view.s30D.hint.intro">
            Up to <strong>$7,500</strong>: $3,750 critical minerals + $3,750 battery components (IRA 2022).
            <strong>Income limit:</strong> $300K MFJ / $225K HoH / $150K single (MAGI, current OR prior year).
            <strong>MSRP cap:</strong> $80K SUVs/vans/trucks; $55K cars. <strong>Final assembly:</strong>
            MUST occur in NORTH AMERICA. <strong>Transfer at dealer</strong> (2024+): get $7,500 off price
            at purchase. <strong>Form 8936.</strong> Used EV: § 25E ≤ $4,000 / 30% (separate, lower limits).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s30D.h2.inputs">Inputs</h2>
            <form id="s30D-form" class="inline-form">
                <label><span data-i18n="view.s30D.label.msrp">MSRP ($)</span>
                    <input type="number" step="1000" name="msrp" value="${state.msrp}"></label>
                <label><span data-i18n="view.s30D.label.type">Vehicle type</span>
                    <select name="vehicle_type">
                        <option value="car" ${state.vehicle_type === 'car' ? 'selected' : ''}>Car ($55K MSRP cap)</option>
                        <option value="suv" ${state.vehicle_type === 'suv' ? 'selected' : ''}>SUV ($80K)</option>
                        <option value="van" ${state.vehicle_type === 'van' ? 'selected' : ''}>Van ($80K)</option>
                        <option value="truck" ${state.vehicle_type === 'truck' ? 'selected' : ''}>Truck ($80K)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s30D.label.assembly">Final assembly in NA?</span>
                    <input type="checkbox" name="final_assembly_in_na" ${state.final_assembly_in_na ? 'checked' : ''}></label>
                <label><span data-i18n="view.s30D.label.minerals">Critical minerals qualifies ($3,750)?</span>
                    <input type="checkbox" name="critical_minerals_qualifies" ${state.critical_minerals_qualifies ? 'checked' : ''}></label>
                <label><span data-i18n="view.s30D.label.battery">Battery components qualifies ($3,750)?</span>
                    <input type="checkbox" name="battery_components_qualifies" ${state.battery_components_qualifies ? 'checked' : ''}></label>
                <label><span data-i18n="view.s30D.label.magi">MAGI ($)</span>
                    <input type="number" step="1000" name="magi" value="${state.magi}"></label>
                <label><span data-i18n="view.s30D.label.filing">Filing status</span>
                    <select name="filing_status">
                        <option value="single" ${state.filing_status === 'single' ? 'selected' : ''}>Single / MFS</option>
                        <option value="hoh" ${state.filing_status === 'hoh' ? 'selected' : ''}>Head of Household</option>
                        <option value="mfj" ${state.filing_status === 'mfj' ? 'selected' : ''}>MFJ</option>
                    </select>
                </label>
                <label><span data-i18n="view.s30D.label.year">Purchase year</span>
                    <input type="number" step="1" name="purchase_year" value="${state.purchase_year}"></label>
                <label><span data-i18n="view.s30D.label.transfer">Transfer at dealer (POS)?</span>
                    <input type="checkbox" name="transfer_at_dealer" ${state.transfer_at_dealer ? 'checked' : ''}></label>
                <label><span data-i18n="view.s30D.label.biz">Business use %</span>
                    <input type="number" step="0.1" name="business_use_pct" value="${state.business_use_pct}"></label>
                <label><span data-i18n="view.s30D.label.date">Placed in service date</span>
                    <input type="date" name="placed_in_service_date" value="${state.placed_in_service_date}"></label>
                <button class="primary" type="submit" data-i18n="view.s30D.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s30D-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s30D.h2.eligibility">Eligibility cascade</h2>
            <ol class="muted small">
                <li data-i18n="view.s30D.elig.assembly">Final assembly: must occur in North America (FuelEconomy.gov verifies)</li>
                <li data-i18n="view.s30D.elig.msrp">MSRP cap met: $80K SUVs/vans/trucks; $55K cars</li>
                <li data-i18n="view.s30D.elig.minerals">Critical minerals threshold met: 50%+ (2024), 60% (2025), 70% (2026), 80% (2027), 90% (2028)</li>
                <li data-i18n="view.s30D.elig.battery">Battery components threshold: 60% (2024), 70% (2026), 80% (2027), 90% (2028), 100% (2029)</li>
                <li data-i18n="view.s30D.elig.foreign_entity">FEOC (Foreign Entity of Concern) restriction: starting 2024 batteries, 2025 critical minerals</li>
                <li data-i18n="view.s30D.elig.magi">MAGI limit: $300K MFJ / $225K HoH / $150K single (CURRENT or PRIOR yr — pick lower)</li>
                <li data-i18n="view.s30D.elig.battery_kwh">Battery capacity: 7 kWh min for plug-in hybrid (PHEV); BEV no min</li>
                <li data-i18n="view.s30D.elig.gross_vehicle">Gross Vehicle Weight Rating: ≤ 14,000 lbs (commercial trucks use § 45W)</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s30D.h2.transfer">§ 30D(g) transfer at dealer election (2024+)</h2>
            <ul class="muted small">
                <li data-i18n="view.s30D.tx.dealer">Buyer transfers credit to dealer at point of sale → discount $ off MSRP</li>
                <li data-i18n="view.s30D.tx.cash">Dealer receives advance payment from IRS (Form 8300)</li>
                <li data-i18n="view.s30D.tx.no_tax_owed">Buyer benefits even if no tax liability — non-refundable becomes refundable in practice</li>
                <li data-i18n="view.s30D.tx.income_excess">If MAGI exceeds limit retroactively: REPAY credit on next return</li>
                <li data-i18n="view.s30D.tx.dealer_must_register">Dealer registers via IRS Energy Credits Online (ECO) portal</li>
                <li data-i18n="view.s30D.tx.report_form_15400">Report on Form 8936 + 8936 Schedule A; dealer issues Time-of-Sale Report</li>
                <li data-i18n="view.s30D.tx.repayment_safe_harbor">No clawback if buyer not aware of income excess</li>
                <li data-i18n="view.s30D.tx.lease_loophole">§ 45W lease "loophole" (commercial credit) — dealer credit available even if vehicle non-qualifying for § 30D</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s30D.h2.compare">Compare clean vehicle credits</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s30D.th.code">Code</th>
                    <th data-i18n="view.s30D.th.target">Target</th>
                    <th data-i18n="view.s30D.th.max">Max credit</th>
                    <th data-i18n="view.s30D.th.notes">Notes</th>
                </tr></thead>
                <tbody>
                    <tr><td>§ 30D</td><td>New BEV / PHEV</td><td>$7,500</td><td>$150K/$225K/$300K MAGI limit; MSRP cap; NA assembly</td></tr>
                    <tr><td>§ 25E</td><td>Used BEV / PHEV</td><td>$4,000 / 30% sale price</td><td>$75K/$112.5K/$150K MAGI; ≤ $25K sale price; ≥ 2 yrs old</td></tr>
                    <tr><td>§ 45W</td><td>Commercial clean vehicles</td><td>$7,500 / $40K (heavy)</td><td>NO income limit, NO MSRP cap, NO domestic assembly — "lease loophole"</td></tr>
                    <tr><td>§ 30C</td><td>Alternative refueling property (charger)</td><td>30% / $1K personal; $100K business</td><td>Low-income community / non-urban only</td></tr>
                </tbody>
            </table>
        </div>
    `;
    document.getElementById('s30D-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.msrp = Number(fd.get('msrp')) || 0;
        state.vehicle_type = fd.get('vehicle_type');
        state.final_assembly_in_na = !!fd.get('final_assembly_in_na');
        state.critical_minerals_qualifies = !!fd.get('critical_minerals_qualifies');
        state.battery_components_qualifies = !!fd.get('battery_components_qualifies');
        state.magi = Number(fd.get('magi')) || 0;
        state.filing_status = fd.get('filing_status');
        state.purchase_year = Number(fd.get('purchase_year')) || 0;
        state.transfer_at_dealer = !!fd.get('transfer_at_dealer');
        state.business_use_pct = Number(fd.get('business_use_pct')) || 0;
        state.placed_in_service_date = fd.get('placed_in_service_date');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s30D-output');
    if (!el) return;
    const msrpCap = state.vehicle_type === 'car' ? 55_000 : 80_000;
    const msrpMet = state.msrp <= msrpCap;
    const magiLimit = state.filing_status === 'mfj' ? 300_000 : state.filing_status === 'hoh' ? 225_000 : 150_000;
    const magiMet = state.magi <= magiLimit;
    const eligible = state.final_assembly_in_na && msrpMet && magiMet;
    const credit = eligible ? ((state.critical_minerals_qualifies ? 3_750 : 0) + (state.battery_components_qualifies ? 3_750 : 0)) : 0;
    const personalCredit = credit * (1 - state.business_use_pct / 100);
    const businessCredit = credit * (state.business_use_pct / 100);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s30D.h2.result">§ 30D credit computation</h2>
            <div class="cards">
                <div class="card ${eligible ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s30D.card.eligible">Eligible?</div>
                    <div class="value">${eligible ? esc(t('view.s30D.status.yes')) : esc(t('view.s30D.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s30D.card.msrp_cap">MSRP cap</div>
                    <div class="value">$${msrpCap.toLocaleString()}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s30D.card.magi_limit">MAGI limit</div>
                    <div class="value">$${magiLimit.toLocaleString()}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s30D.card.credit">§ 30D credit</div>
                    <div class="value">$${credit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s30D.card.personal">Personal portion</div>
                    <div class="value">$${personalCredit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s30D.card.business">Business portion (GBC)</div>
                    <div class="value">$${businessCredit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${!eligible ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s30D.fail_note">
                    Not eligible. Common failures: MSRP exceeds cap, MAGI exceeds limit, vehicle assembled
                    outside NA, missing critical minerals / battery components qualifications. Consider used
                    EV (§ 25E) or commercial route (§ 45W) — different rules + no income / MSRP limit.
                </p>
            ` : ''}
        </div>
    `;
}
