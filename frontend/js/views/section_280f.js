// IRC § 280F Luxury Auto Depreciation Limits.
// Caps annual depreciation deduction for passenger vehicles + light trucks.
// 2024 first-year limit: $20,400 with bonus depreciation, $12,400 without.
// Heavy vehicles (GVWR > 6,000 lbs SUVs / pickups): exempt from § 280F, eligible
// for § 179 expensing up to $30,500 (2024) + bonus.
// Personal use > 50%: § 280F(d)(1) recapture; lose all bonus + § 179.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const LIMITS_2024 = {
    passenger: {
        year1_bonus: 20_400,
        year1_no_bonus: 12_400,
        year2: 19_800,
        year3: 11_900,
        year4_plus: 7_160,
    },
    heavy_suv: {
        section_179_cap: 30_500,  // 2024
    },
};
const BONUS_2024_PCT = 0.60;  // TCJA bonus phase-down

let state = {
    vehicle_cost: 0,
    business_use_pct: 100,
    is_heavy_suv: false,
    is_passenger: true,
    elect_section_179: false,
    take_bonus: true,
    marginal_rate: 0.32,
};

export async function renderSection280f(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s280f.h1.title">// § 280F LUXURY AUTO DEPRECIATION</span></h1>
        <p class="muted small" data-i18n="view.s280f.hint.intro">
            Caps depreciation on passenger autos + light trucks. <strong>Heavy SUVs
            (GVWR &gt; 6,000 lbs)</strong> escape § 280F, get up to $30,500 § 179 + bonus.
            Personal use must be ≤ 50% to take bonus/§179. 2024 first-year limits: $20,400 with
            bonus, $12,400 without. Year 2: $19,800. Year 3: $11,900. Year 4+: $7,160/yr.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s280f.h2.inputs">Inputs</h2>
            <form id="s280f-form" class="inline-form">
                <label><span data-i18n="view.s280f.label.cost">Vehicle cost ($)</span>
                    <input type="number" step="0.01" name="vehicle_cost" value="${state.vehicle_cost}"></label>
                <label><span data-i18n="view.s280f.label.business_use">Business use %</span>
                    <input type="number" step="1" name="business_use_pct" value="${state.business_use_pct}"></label>
                <label><span data-i18n="view.s280f.label.is_heavy">Heavy SUV / pickup (GVWR &gt; 6,000)?</span>
                    <input type="checkbox" name="is_heavy_suv" ${state.is_heavy_suv ? 'checked' : ''}></label>
                <label><span data-i18n="view.s280f.label.take_bonus">Take bonus depreciation (60% in 2024)?</span>
                    <input type="checkbox" name="take_bonus" ${state.take_bonus ? 'checked' : ''}></label>
                <label><span data-i18n="view.s280f.label.elect_179">Elect § 179?</span>
                    <input type="checkbox" name="elect_section_179" ${state.elect_section_179 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s280f.label.marginal">Marginal tax %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s280f.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s280f-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s280f.h2.heavy_examples">Common heavy-SUV qualifiers (GVWR &gt; 6,000 lbs)</h2>
            <ul class="muted small">
                <li data-i18n="view.s280f.heavy.suv">Cadillac Escalade, Chevy Suburban / Tahoe, Ford Expedition, GMC Yukon</li>
                <li data-i18n="view.s280f.heavy.trucks">F-150 / F-250 / F-350, Silverado 1500+, Ram 1500+, Tundra</li>
                <li data-i18n="view.s280f.heavy.luxury">Range Rover, Mercedes G-Wagen, Lexus LX, BMW X7, Audi Q7/Q8</li>
                <li data-i18n="view.s280f.heavy.ev">Tesla Model X (most builds), Rivian R1T/R1S, F-150 Lightning</li>
                <li data-i18n="view.s280f.heavy.note">Always verify the SPECIFIC model + trim's GVWR on the manufacturer doorjamb sticker</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s280f.h2.recapture">Personal use recapture trap</h2>
            <p class="muted small" data-i18n="view.s280f.recapture.body">
                If business use drops to ≤ 50% in any later year, § 280F(d)(1) requires
                <strong>recapture</strong> of all bonus + § 179 deducted in prior years —
                included as ordinary income. Keep contemporaneous mileage log (Form 4562 line 30).
            </p>
        </div>
    `;
    document.getElementById('s280f-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.vehicle_cost = Number(fd.get('vehicle_cost')) || 0;
        state.business_use_pct = Number(fd.get('business_use_pct')) || 0;
        state.is_heavy_suv = !!fd.get('is_heavy_suv');
        state.take_bonus = !!fd.get('take_bonus');
        state.elect_section_179 = !!fd.get('elect_section_179');
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.32;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s280f-output');
    if (!el) return;
    const businessShare = state.business_use_pct / 100;
    const businessBasis = state.vehicle_cost * businessShare;
    const eligibleForBonus = state.business_use_pct > 50;
    let year1Deduction = 0, year2 = 0, year3 = 0, year4_plus = 0;
    let strategy = '';
    if (state.is_heavy_suv) {
        const section179 = state.elect_section_179 && eligibleForBonus
            ? Math.min(LIMITS_2024.heavy_suv.section_179_cap, businessBasis)
            : 0;
        const afterSection179 = businessBasis - section179;
        const bonus = state.take_bonus && eligibleForBonus ? afterSection179 * BONUS_2024_PCT : 0;
        const afterBonus = afterSection179 - bonus;
        const macrs1 = afterBonus * 0.20;
        year1Deduction = section179 + bonus + macrs1;
        year2 = afterBonus * 0.32;
        year3 = afterBonus * 0.192;
        year4_plus = afterBonus * 0.1152;  // simplified
        strategy = t('view.s280f.strategy.heavy');
    } else if (state.is_passenger) {
        const cap1 = state.take_bonus && eligibleForBonus
            ? LIMITS_2024.passenger.year1_bonus
            : LIMITS_2024.passenger.year1_no_bonus;
        year1Deduction = Math.min(businessBasis, cap1 * businessShare);
        year2 = Math.min(LIMITS_2024.passenger.year2 * businessShare, businessBasis - year1Deduction);
        year3 = Math.min(LIMITS_2024.passenger.year3 * businessShare, businessBasis - year1Deduction - year2);
        year4_plus = LIMITS_2024.passenger.year4_plus * businessShare;
        strategy = t('view.s280f.strategy.passenger');
    }
    const year1Savings = year1Deduction * state.marginal_rate;
    const lifetimeDeduction = year1Deduction + year2 + year3 + (year4_plus * 3);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s280f.h2.result">Depreciation schedule</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s280f.card.business_basis">Business basis</div>
                    <div class="value">$${businessBasis.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s280f.card.year1">Year 1 deduction</div>
                    <div class="value">$${year1Deduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s280f.card.year2">Year 2</div>
                    <div class="value">$${year2.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s280f.card.year3">Year 3</div>
                    <div class="value">$${year3.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s280f.card.year4_plus">Year 4+ (each)</div>
                    <div class="value">$${year4_plus.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s280f.card.year1_savings">Year 1 tax savings</div>
                    <div class="value">$${year1Savings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s280f.card.lifetime">Lifetime deduction (6 yr)</div>
                    <div class="value">$${lifetimeDeduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s280f.card.strategy">Strategy</div>
                    <div class="value">${esc(strategy)}</div>
                </div>
            </div>
            ${!eligibleForBonus ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s280f.warning.under50">
                    Business use ≤ 50% — bonus depreciation + § 179 not available.
                    Straight-line over 5 yrs only.
                </p>
            ` : ''}
        </div>
    `;
}
