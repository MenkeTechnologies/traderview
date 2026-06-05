// IRC § 38 — General Business Credit (Aggregator).
// Sum of business credits: § 41 R&D + § 45 PTC + § 48 ITC + § 45Q + § 30D + ESBP + many more.
// Limit: net income tax above TMT (tentative minimum tax) + 25% of next $25K.
// Carryback 1-yr; Carryforward 20-yr. § 39.
// Form 3800. § 6417 direct pay (partial); § 6418 transferability (specified credits only).

import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    s41_research: 0,
    s45_ptc_wind_etc: 0,
    s48_itc: 0,
    s45q_carbon: 0,
    s30d_clean_vehicle: 0,
    s45w_commercial_ev: 0,
    s45l_residential: 0,
    s45x_advanced_mfg: 0,
    other_business_credits: 0,
    regular_tax: 0,
    tentative_minimum_tax: 0,
    carryforward_prior: 0,
    elect_direct_pay: false,
    elect_transferability: false,
};

export async function renderSection38(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s38.h1.title">// § 38 GENERAL BUSINESS CREDIT</span></h1>
        <p class="muted small" data-i18n="view.s38.hint.intro">
            <strong>Aggregator</strong> of business credits: § 41 R&D + § 45 PTC + § 48 ITC + § 45Q + § 30D
            + § 45W + § 45L + § 45X + Empowerment Zone + WOTC + Disabled Access + many more. <strong>Limit:</strong>
            net regular tax above TMT (Tentative Minimum Tax) + 25% of next $25K. <strong>Carryback 1-yr,
            Carryforward 20-yr</strong> (§ 39). <strong>Forms 3800 + 8000-series source forms.</strong>
            <strong>§ 6417 direct pay</strong> + <strong>§ 6418 transferability</strong> for IRA 2022
            specified credits (different rules per credit).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s38.h2.inputs">Inputs</h2>
            <form id="s38-form" class="inline-form">
                <label><span data-i18n="view.s38.label.s41">§ 41 Research credit ($)</span>
                    <input type="number" step="0.01" name="s41_research" value="${state.s41_research}"></label>
                <label><span data-i18n="view.s38.label.s45">§ 45 PTC (wind, biomass, etc.) ($)</span>
                    <input type="number" step="0.01" name="s45_ptc_wind_etc" value="${state.s45_ptc_wind_etc}"></label>
                <label><span data-i18n="view.s38.label.s48">§ 48 ITC (solar, energy property) ($)</span>
                    <input type="number" step="0.01" name="s48_itc" value="${state.s48_itc}"></label>
                <label><span data-i18n="view.s38.label.s45q">§ 45Q Carbon capture ($)</span>
                    <input type="number" step="0.01" name="s45q_carbon" value="${state.s45q_carbon}"></label>
                <label><span data-i18n="view.s38.label.s30d">§ 30D Clean vehicle ($)</span>
                    <input type="number" step="0.01" name="s30d_clean_vehicle" value="${state.s30d_clean_vehicle}"></label>
                <label><span data-i18n="view.s38.label.s45w">§ 45W Commercial EV ($)</span>
                    <input type="number" step="0.01" name="s45w_commercial_ev" value="${state.s45w_commercial_ev}"></label>
                <label><span data-i18n="view.s38.label.s45l">§ 45L Residential energy ($)</span>
                    <input type="number" step="0.01" name="s45l_residential" value="${state.s45l_residential}"></label>
                <label><span data-i18n="view.s38.label.s45x">§ 45X Advanced mfg ($)</span>
                    <input type="number" step="0.01" name="s45x_advanced_mfg" value="${state.s45x_advanced_mfg}"></label>
                <label><span data-i18n="view.s38.label.other">Other business credits ($)</span>
                    <input type="number" step="0.01" name="other_business_credits" value="${state.other_business_credits}"></label>
                <label><span data-i18n="view.s38.label.regular">Net regular tax ($)</span>
                    <input type="number" step="0.01" name="regular_tax" value="${state.regular_tax}"></label>
                <label><span data-i18n="view.s38.label.tmt">Tentative minimum tax ($)</span>
                    <input type="number" step="0.01" name="tentative_minimum_tax" value="${state.tentative_minimum_tax}"></label>
                <label><span data-i18n="view.s38.label.carry">Prior-year carryforward ($)</span>
                    <input type="number" step="0.01" name="carryforward_prior" value="${state.carryforward_prior}"></label>
                <label><span data-i18n="view.s38.label.direct">§ 6417 direct pay (some credits)?</span>
                    <input type="checkbox" name="elect_direct_pay" ${state.elect_direct_pay ? 'checked' : ''}></label>
                <label><span data-i18n="view.s38.label.transfer">§ 6418 transferability (some credits)?</span>
                    <input type="checkbox" name="elect_transferability" ${state.elect_transferability ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s38.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s38-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s38.h2.components">§ 38(b) credit components</h2>
            <ul class="muted small">
                <li data-i18n="view.s38.comp.investment">Investment credit § 46: § 48 ITC, § 48C, § 48E, § 47 rehab, § 48D semi mfg</li>
                <li data-i18n="view.s38.comp.work_opportunity">§ 51 Work Opportunity Tax Credit (WOTC): up to $9,600 per qualified hire</li>
                <li data-i18n="view.s38.comp.alcohol">§ 40 Alcohol fuel mixtures + § 40A biodiesel + § 40B Sustainable Aviation Fuel</li>
                <li data-i18n="view.s38.comp.research">§ 41 Research credit (incremental QRE method or ASC)</li>
                <li data-i18n="view.s38.comp.lihc">§ 42 Low-Income Housing Credit (LIHTC)</li>
                <li data-i18n="view.s38.comp.disabled">§ 44 Disabled Access Credit + § 45A Empowerment Zone</li>
                <li data-i18n="view.s38.comp.renewable">§ 45 Renewable Electricity PTC + § 45Y Clean Electricity PTC</li>
                <li data-i18n="view.s38.comp.energy">§ 45Q Carbon + § 45V Hydrogen + § 45W Commercial EV + § 45X Manufacturing</li>
                <li data-i18n="view.s38.comp.employer">§ 45E Small Employer Pension Setup + § 45F Childcare + § 45R Health Insurance</li>
                <li data-i18n="view.s38.comp.export">§ 27 / § 38 export credit (historical) + § 901 FTC (separate path)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s38.h2.limit">§ 38(c) limit computation</h2>
            <ol class="muted small">
                <li data-i18n="view.s38.lim.basic">Limit = Net Tax (regular) above Tentative Minimum Tax (TMT)</li>
                <li data-i18n="view.s38.lim.25pct">+ 25% of remaining tax above $25K (after TMT reached)</li>
                <li data-i18n="view.s38.lim.specified">"Specified credits" (R&D portion, energy ITC, etc.) can offset AMT + TMT</li>
                <li data-i18n="view.s38.lim.eligible_small">Eligible small biz: R&D + energy + various credits can offset AMT entirely</li>
                <li data-i18n="view.s38.lim.carry_back">Unused: carry back 1 yr first, then 20-year carryforward</li>
                <li data-i18n="view.s38.lim.s39_expiration">§ 39 carryforward expires after 20 yrs → permanent loss</li>
                <li data-i18n="view.s38.lim.ordering">Use older carryforwards first within § 38 (FIFO)</li>
                <li data-i18n="view.s38.lim.pre_acquisition">§ 39(d) pre-acquisition GBCs: SRLY-limited similar to NOL</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s38.h2.monetization">IRA 2022 monetization paths</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s38.th.credit">Credit</th>
                    <th data-i18n="view.s38.th.direct">§ 6417 direct pay</th>
                    <th data-i18n="view.s38.th.transfer">§ 6418 transferability</th>
                </tr></thead>
                <tbody>
                    <tr><td>§ 45 / § 45Y PTC</td><td>Tax-exempt only</td><td>YES — taxable</td></tr>
                    <tr><td>§ 48 / § 48E ITC</td><td>Tax-exempt only</td><td>YES — taxable</td></tr>
                    <tr><td>§ 45Q Carbon</td><td>5-yr taxable + perm tax-exempt</td><td>YES — taxable</td></tr>
                    <tr><td>§ 45V Hydrogen</td><td>5-yr taxable + perm tax-exempt</td><td>YES — taxable</td></tr>
                    <tr><td>§ 45X Manufacturing</td><td>5-yr taxable + perm tax-exempt</td><td>YES — taxable</td></tr>
                    <tr><td>§ 30D Clean vehicle</td><td>NO direct pay (transfer to dealer instead)</td><td>NO (dealer transfer § 30D(g))</td></tr>
                    <tr><td>§ 41 R&D</td><td>NO</td><td>NO</td></tr>
                </tbody>
            </table>
        </div>
    `;
    document.getElementById('s38-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.s41_research = Number(fd.get('s41_research')) || 0;
        state.s45_ptc_wind_etc = Number(fd.get('s45_ptc_wind_etc')) || 0;
        state.s48_itc = Number(fd.get('s48_itc')) || 0;
        state.s45q_carbon = Number(fd.get('s45q_carbon')) || 0;
        state.s30d_clean_vehicle = Number(fd.get('s30d_clean_vehicle')) || 0;
        state.s45w_commercial_ev = Number(fd.get('s45w_commercial_ev')) || 0;
        state.s45l_residential = Number(fd.get('s45l_residential')) || 0;
        state.s45x_advanced_mfg = Number(fd.get('s45x_advanced_mfg')) || 0;
        state.other_business_credits = Number(fd.get('other_business_credits')) || 0;
        state.regular_tax = Number(fd.get('regular_tax')) || 0;
        state.tentative_minimum_tax = Number(fd.get('tentative_minimum_tax')) || 0;
        state.carryforward_prior = Number(fd.get('carryforward_prior')) || 0;
        state.elect_direct_pay = !!fd.get('elect_direct_pay');
        state.elect_transferability = !!fd.get('elect_transferability');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s38-output');
    if (!el) return;
    const totalCredits = state.s41_research + state.s45_ptc_wind_etc + state.s48_itc + state.s45q_carbon +
        state.s30d_clean_vehicle + state.s45w_commercial_ev + state.s45l_residential + state.s45x_advanced_mfg +
        state.other_business_credits + state.carryforward_prior;
    const aboveTMT = Math.max(0, state.regular_tax - state.tentative_minimum_tax);
    const limit = aboveTMT + 0.25 * Math.max(0, state.regular_tax - state.tentative_minimum_tax - 25_000);
    const allowedCredit = Math.min(totalCredits, Math.max(limit, aboveTMT));
    const unusedCarry = totalCredits - allowedCredit;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s38.h2.result">§ 38 GBC computation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s38.card.total">Total credits</div>
                    <div class="value">$${totalCredits.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s38.card.above_tmt">Net tax above TMT</div>
                    <div class="value">$${aboveTMT.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s38.card.limit">§ 38(c) limit</div>
                    <div class="value">$${limit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s38.card.allowed">Allowed current year</div>
                    <div class="value">$${allowedCredit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${unusedCarry > 0 ? 'neg' : ''}">
                    <div class="label" data-i18n="view.s38.card.carry">Carry forward (20-yr)</div>
                    <div class="value">$${unusedCarry.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${unusedCarry > 0 ? `
                <p class="muted small" style="margin-top:10px" data-i18n="view.s38.carry_note">
                    Unused credit carries back 1 yr (file Form 1045 / 1139 quick refund) OR forward 20 yrs.
                    For IRA 2022 specified credits: consider § 6417 direct pay (tax-exempts) or § 6418
                    transferability (taxables) to monetize immediately at 88-95¢ on $1.
                </p>
            ` : ''}
        </div>
    `;
}
