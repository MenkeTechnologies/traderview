// EV Tax Credit § 30D (Clean Vehicle Credit).
// $7,500 new EV / $4,000 used EV. MSRP cap + MAGI cap + battery sourcing.
// Transferable to dealer at point of sale 2024+. Used credit is dealer-only.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const NEW_CREDIT = 7_500;
const NEW_CREDIT_HALF_CRITICAL = 3_750;
const NEW_CREDIT_HALF_BATTERY = 3_750;
const NEW_MAGI_SINGLE = 150_000;
const NEW_MAGI_HOH = 225_000;
const NEW_MAGI_MFJ = 300_000;
const NEW_MSRP_CAR = 55_000;
const NEW_MSRP_SUV_VAN_PICKUP = 80_000;

const USED_CREDIT_MAX = 4_000;
const USED_CREDIT_PCT_OF_PRICE = 0.30;
const USED_MAGI_SINGLE = 75_000;
const USED_MAGI_HOH = 112_500;
const USED_MAGI_MFJ = 150_000;
const USED_PRICE_CAP = 25_000;
const USED_AGE_MIN_YEARS = 2;

let state = {
    is_new: true,
    is_truck_suv_van: false,
    msrp_or_price: 45_000,
    filing: 'mfj',
    magi: 200_000,
    critical_minerals_met: true,  // 50% N. America/free-trade-country, 2024
    battery_components_met: true,  // 60% N. America, 2024
    vehicle_age_years: 0,  // used only
};

export async function renderEvCredit(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ev.h1.title">// EV TAX CREDIT § 30D</span></h1>
        <p class="muted small" data-i18n="view.ev.hint.intro">
            New EV: $7,500 ($3,750 critical minerals + $3,750 battery components).
            Used EV: lesser of $4,000 or 30% of sale price. Both have MAGI caps.
            New: MSRP cap $55k cars / $80k trucks/SUVs/vans. Used: $25k price cap, 2+ yr old.
            Transferable to dealer at point of sale (immediate $7,500 off vs. tax-time credit).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.ev.h2.inputs">Inputs</h2>
            <form id="ev-form" class="inline-form">
                <label><span data-i18n="view.ev.label.is_new">New EV?</span>
                    <input type="checkbox" name="is_new" ${state.is_new ? 'checked' : ''}></label>
                <label><span data-i18n="view.ev.label.is_truck_suv_van">SUV / truck / van?</span>
                    <input type="checkbox" name="is_truck_suv_van" ${state.is_truck_suv_van ? 'checked' : ''}></label>
                <label><span data-i18n="view.ev.label.msrp_or_price">MSRP (new) or price (used) ($)</span>
                    <input type="number" step="0.01" name="msrp_or_price" value="${state.msrp_or_price}"></label>
                <label><span data-i18n="view.ev.label.filing">Filing</span>
                    <select name="filing">
                        <option value="single" ${state.filing === 'single' ? 'selected' : ''}>Single</option>
                        <option value="hoh" ${state.filing === 'hoh' ? 'selected' : ''}>HoH</option>
                        <option value="mfj" ${state.filing === 'mfj' ? 'selected' : ''}>MFJ</option>
                    </select>
                </label>
                <label><span data-i18n="view.ev.label.magi">MAGI ($)</span>
                    <input type="number" step="0.01" name="magi" value="${state.magi}"></label>
                <label><span data-i18n="view.ev.label.critical_minerals">Critical minerals criteria met?</span>
                    <input type="checkbox" name="critical_minerals_met" ${state.critical_minerals_met ? 'checked' : ''}></label>
                <label><span data-i18n="view.ev.label.battery_components">Battery components criteria met?</span>
                    <input type="checkbox" name="battery_components_met" ${state.battery_components_met ? 'checked' : ''}></label>
                <label><span data-i18n="view.ev.label.vehicle_age">Vehicle age (used only)</span>
                    <input type="number" step="1" name="vehicle_age_years" value="${state.vehicle_age_years}" min="0"></label>
                <button class="primary" type="submit" data-i18n="view.ev.btn.compute">Compute</button>
            </form>
        </div>
        <div id="ev-output"></div>
    `;
    document.getElementById('ev-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.is_new = !!fd.get('is_new');
        state.is_truck_suv_van = !!fd.get('is_truck_suv_van');
        state.msrp_or_price = Number(fd.get('msrp_or_price')) || 0;
        state.filing = fd.get('filing');
        state.magi = Number(fd.get('magi')) || 0;
        state.critical_minerals_met = !!fd.get('critical_minerals_met');
        state.battery_components_met = !!fd.get('battery_components_met');
        state.vehicle_age_years = Number(fd.get('vehicle_age_years')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('ev-output');
    if (!el) return;
    let credit = 0;
    const failures = [];

    if (state.is_new) {
        const msrpCap = state.is_truck_suv_van ? NEW_MSRP_SUV_VAN_PICKUP : NEW_MSRP_CAR;
        const magiCap = state.filing === 'mfj' ? NEW_MAGI_MFJ
            : state.filing === 'hoh' ? NEW_MAGI_HOH : NEW_MAGI_SINGLE;
        if (state.msrp_or_price > msrpCap) failures.push(t('view.ev.fail.msrp', { cap: msrpCap.toLocaleString() }));
        if (state.magi > magiCap) failures.push(t('view.ev.fail.magi', { cap: magiCap.toLocaleString() }));
        if (failures.length === 0) {
            if (state.critical_minerals_met) credit += NEW_CREDIT_HALF_CRITICAL;
            if (state.battery_components_met) credit += NEW_CREDIT_HALF_BATTERY;
        }
    } else {
        const magiCap = state.filing === 'mfj' ? USED_MAGI_MFJ
            : state.filing === 'hoh' ? USED_MAGI_HOH : USED_MAGI_SINGLE;
        if (state.msrp_or_price > USED_PRICE_CAP) failures.push(t('view.ev.fail.used_price', { cap: USED_PRICE_CAP.toLocaleString() }));
        if (state.magi > magiCap) failures.push(t('view.ev.fail.magi', { cap: magiCap.toLocaleString() }));
        if (state.vehicle_age_years < USED_AGE_MIN_YEARS) failures.push(t('view.ev.fail.age'));
        if (failures.length === 0) {
            credit = Math.min(USED_CREDIT_MAX, state.msrp_or_price * USED_CREDIT_PCT_OF_PRICE);
        }
    }

    const netCost = state.msrp_or_price - credit;
    el.innerHTML = `
        <div class="chart-panel ${credit > 0 ? 'pos' : 'neg'}">
            <h2 data-i18n="view.ev.h2.result">Credit calculation</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.ev.card.credit">Credit</div>
                    <div class="value">$${credit.toLocaleString()}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.ev.card.vehicle_price">Vehicle price</div>
                    <div class="value">$${state.msrp_or_price.toLocaleString()}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.ev.card.net_cost">Net cost after credit</div>
                    <div class="value">$${netCost.toLocaleString()}</div>
                </div>
            </div>
            ${failures.length > 0 ? `
                <h3 style="margin-top:10px" data-i18n="view.ev.h3.failures">Disqualifications</h3>
                <ul class="muted small">
                    ${failures.map(f => `<li>${esc(f)}</li>`).join('')}
                </ul>
            ` : `
                <p class="muted small" style="margin-top:10px" data-i18n="view.ev.qualified">
                    All caps met. Transfer credit to dealer at point of sale to capture
                    immediately (vs. waiting until you file).
                </p>
            `}
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.ev.h2.notes">Notes</h2>
            <ul class="muted small">
                <li data-i18n="view.ev.note.transfer">Point-of-sale transfer (2024+): credit applied as down payment at dealer; no wait until filing</li>
                <li data-i18n="view.ev.note.recapture">If MAGI later exceeds cap, credit must be repaid as tax</li>
                <li data-i18n="view.ev.note.final_assembly">Final assembly in N. America required (check fueleconomy.gov VIN lookup)</li>
                <li data-i18n="view.ev.note.lookback">MAGI test = current year OR prior year, whichever lower</li>
                <li data-i18n="view.ev.note.one_per_3yrs">Used EV: 1 credit per 3-year period</li>
                <li data-i18n="view.ev.note.dealer_only">Used EV credit ONLY available at registered dealer (no private sale)</li>
            </ul>
        </div>
    `;
}
