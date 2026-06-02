// IRC § 45Q — Carbon Capture Credit.
// IRA 2022 dramatically expanded: $85/ton (geologic), $60/ton (utilization), $180/ton (DAC).
// 12-year credit period post-placed-in-service (vs prior 10-yr).
// Direct Pay (§ 6417) elective for tax-exempts + nonprofits.
// Transferability (§ 6418) allows sale of credit to third parties.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    tons_captured_annual: 0,
    capture_type: 'geologic_sequestration',
    is_direct_air_capture: false,
    prevailing_wage_compliant: true,
    apprenticeship_compliant: true,
    placed_in_service_year: 2024,
    years_in_credit_period: 0,
    elect_direct_pay: false,
    elect_transferability: false,
    co2_price_per_ton: 60,
    is_qualified_facility: true,
    minimum_threshold_met: false,
};

export async function renderSection45Q(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s45Q.h1.title">// § 45Q CARBON CAPTURE</span></h1>
        <p class="muted small" data-i18n="view.s45Q.hint.intro">
            IRA 2022 expanded: <strong>$85/ton</strong> (geologic sequestration), <strong>$60/ton</strong>
            (EOR + utilization), <strong>$180/ton DAC</strong> (Direct Air Capture). Prevailing wage +
            apprenticeship multiplier 5×: ($17 / $12 / $36). <strong>12-year credit period</strong>
            post-placed-in-service (vs prior 10). <strong>§ 6417 direct pay</strong> for tax-exempts +
            5-yr cap for taxable; <strong>§ 6418 transferability</strong> allows credit sale to 3rd parties.
            Minimum threshold: 1K tons/yr (DAC), 12.5K (industrial), 18.75K (electricity).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s45Q.h2.inputs">Inputs</h2>
            <form id="s45Q-form" class="inline-form">
                <label><span data-i18n="view.s45Q.label.tons">Tons CO2 captured / yr</span>
                    <input type="number" step="100" name="tons_captured_annual" value="${state.tons_captured_annual}"></label>
                <label><span data-i18n="view.s45Q.label.type">Capture type</span>
                    <select name="capture_type">
                        <option value="geologic_sequestration" ${state.capture_type === 'geologic_sequestration' ? 'selected' : ''}>Geologic sequestration ($85)</option>
                        <option value="utilization" ${state.capture_type === 'utilization' ? 'selected' : ''}>Utilization ($60)</option>
                        <option value="eor" ${state.capture_type === 'eor' ? 'selected' : ''}>Enhanced Oil Recovery ($60)</option>
                        <option value="dac_geologic" ${state.capture_type === 'dac_geologic' ? 'selected' : ''}>DAC + geologic ($180)</option>
                        <option value="dac_utilization" ${state.capture_type === 'dac_utilization' ? 'selected' : ''}>DAC + utilization ($130)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s45Q.label.dac">Direct Air Capture facility?</span>
                    <input type="checkbox" name="is_direct_air_capture" ${state.is_direct_air_capture ? 'checked' : ''}></label>
                <label><span data-i18n="view.s45Q.label.wage">Prevailing wage compliant?</span>
                    <input type="checkbox" name="prevailing_wage_compliant" ${state.prevailing_wage_compliant ? 'checked' : ''}></label>
                <label><span data-i18n="view.s45Q.label.apprentice">Apprenticeship compliant?</span>
                    <input type="checkbox" name="apprenticeship_compliant" ${state.apprenticeship_compliant ? 'checked' : ''}></label>
                <label><span data-i18n="view.s45Q.label.year">Placed in service year</span>
                    <input type="number" step="1" name="placed_in_service_year" value="${state.placed_in_service_year}"></label>
                <label><span data-i18n="view.s45Q.label.years_credit">Years in 12-yr credit period</span>
                    <input type="number" step="1" name="years_in_credit_period" value="${state.years_in_credit_period}"></label>
                <label><span data-i18n="view.s45Q.label.direct_pay">Elect § 6417 direct pay?</span>
                    <input type="checkbox" name="elect_direct_pay" ${state.elect_direct_pay ? 'checked' : ''}></label>
                <label><span data-i18n="view.s45Q.label.transfer">Elect § 6418 transferability?</span>
                    <input type="checkbox" name="elect_transferability" ${state.elect_transferability ? 'checked' : ''}></label>
                <label><span data-i18n="view.s45Q.label.price">Market price per ton ($) for transfer</span>
                    <input type="number" step="1" name="co2_price_per_ton" value="${state.co2_price_per_ton}"></label>
                <label><span data-i18n="view.s45Q.label.qualified">Qualified facility (post-2022)?</span>
                    <input type="checkbox" name="is_qualified_facility" ${state.is_qualified_facility ? 'checked' : ''}></label>
                <label><span data-i18n="view.s45Q.label.threshold">Minimum threshold met?</span>
                    <input type="checkbox" name="minimum_threshold_met" ${state.minimum_threshold_met ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s45Q.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s45Q-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s45Q.h2.rates">Rate structure (IRA 2022)</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s45Q.th.type">Capture pathway</th>
                    <th data-i18n="view.s45Q.th.base">Base rate</th>
                    <th data-i18n="view.s45Q.th.bonus">Prevailing wage + apprentice (5×)</th>
                </tr></thead>
                <tbody>
                    <tr><td>Industrial / power (geologic)</td><td>$17/ton</td><td>$85/ton</td></tr>
                    <tr><td>Industrial / power (utilization, EOR)</td><td>$12/ton</td><td>$60/ton</td></tr>
                    <tr><td>Direct Air Capture (geologic)</td><td>$36/ton</td><td>$180/ton</td></tr>
                    <tr><td>Direct Air Capture (utilization)</td><td>$26/ton</td><td>$130/ton</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s45Q.h2.thresholds">Minimum thresholds + qualified facility</h2>
            <ul class="muted small">
                <li data-i18n="view.s45Q.thr.dac">DAC: minimum 1,000 tons/year</li>
                <li data-i18n="view.s45Q.thr.industrial">Industrial: 12,500 tons/year</li>
                <li data-i18n="view.s45Q.thr.electricity">Electricity generation: 18,750 tons/year (post-IRA, lower than prior 500K)</li>
                <li data-i18n="view.s45Q.thr.placed">Placed in service before 2033 (originally) — IRA extended to "construction begin" before Jan 1 2033</li>
                <li data-i18n="view.s45Q.thr.contract">Carbon Capture Equipment Contract: written, binding contract</li>
                <li data-i18n="view.s45Q.thr.epa">EPA Subpart RR reporting required for geologic sequestration</li>
                <li data-i18n="view.s45Q.thr.lca">Life Cycle Assessment required for utilization (net reduction certification)</li>
                <li data-i18n="view.s45Q.thr.recapture">Recapture: leakage in 3-yr period triggers credit recapture pro-rata</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s45Q.h2.monetization">§ 6417 direct pay vs § 6418 transfer</h2>
            <ul class="muted small">
                <li data-i18n="view.s45Q.mon.direct_pay_eligible">§ 6417 direct pay: tax-exempt entities + states + tribal + electric coops fully eligible</li>
                <li data-i18n="view.s45Q.mon.direct_pay_taxable">Taxable entities: § 6417 elective ONLY for first 5 years (CCS, hydrogen, advanced manufacturing)</li>
                <li data-i18n="view.s45Q.mon.direct_pay_clawback">Direct pay clawback: 15% / yr if domestic content not met (CCS exception applies)</li>
                <li data-i18n="view.s45Q.mon.transfer_cash">§ 6418 transferability: sell credit for CASH to unrelated buyer at any discount</li>
                <li data-i18n="view.s45Q.mon.transfer_no_recapture">Credit purchaser not subject to recapture if seller in good faith</li>
                <li data-i18n="view.s45Q.mon.market_pricing">Current market: 88-95¢ on $1 for verified projects</li>
                <li data-i18n="view.s45Q.mon.same_year">Election made each year; specified credit amount</li>
                <li data-i18n="view.s45Q.mon.same_or_better">Carbon offset markets bidding up — voluntary carbon markets often above tax credit</li>
            </ul>
        </div>
    `;
    document.getElementById('s45Q-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.tons_captured_annual = Number(fd.get('tons_captured_annual')) || 0;
        state.capture_type = fd.get('capture_type');
        state.is_direct_air_capture = !!fd.get('is_direct_air_capture');
        state.prevailing_wage_compliant = !!fd.get('prevailing_wage_compliant');
        state.apprenticeship_compliant = !!fd.get('apprenticeship_compliant');
        state.placed_in_service_year = Number(fd.get('placed_in_service_year')) || 0;
        state.years_in_credit_period = Number(fd.get('years_in_credit_period')) || 0;
        state.elect_direct_pay = !!fd.get('elect_direct_pay');
        state.elect_transferability = !!fd.get('elect_transferability');
        state.co2_price_per_ton = Number(fd.get('co2_price_per_ton')) || 0;
        state.is_qualified_facility = !!fd.get('is_qualified_facility');
        state.minimum_threshold_met = !!fd.get('minimum_threshold_met');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s45Q-output');
    if (!el) return;
    let baseRate = 0;
    switch (state.capture_type) {
        case 'geologic_sequestration': baseRate = 17; break;
        case 'utilization': case 'eor': baseRate = 12; break;
        case 'dac_geologic': baseRate = 36; break;
        case 'dac_utilization': baseRate = 26; break;
    }
    const bonusMultiplier = (state.prevailing_wage_compliant && state.apprenticeship_compliant) ? 5 : 1;
    const ratePerTon = baseRate * bonusMultiplier;
    const eligible = state.is_qualified_facility && state.minimum_threshold_met && state.years_in_credit_period < 12;
    const creditAmount = eligible ? state.tons_captured_annual * ratePerTon : 0;
    const transferValue = state.elect_transferability ? state.tons_captured_annual * state.co2_price_per_ton : creditAmount;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s45Q.h2.result">§ 45Q credit computation</h2>
            <div class="cards">
                <div class="card ${eligible ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s45Q.card.eligible">Eligible?</div>
                    <div class="value">${eligible ? esc(t('view.s45Q.status.yes')) : esc(t('view.s45Q.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s45Q.card.base">Base rate / ton</div>
                    <div class="value">$${baseRate}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s45Q.card.rate">Effective rate / ton</div>
                    <div class="value">$${ratePerTon}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s45Q.card.credit">§ 45Q credit (yr)</div>
                    <div class="value">$${creditAmount.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s45Q.card.12yr">Total over 12-yr</div>
                    <div class="value">$${(creditAmount * 12).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s45Q.card.transfer">Transfer value (current yr)</div>
                    <div class="value">$${transferValue.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.prevailing_wage_compliant && state.apprenticeship_compliant ? `
                <p class="muted small pos" style="margin-top:10px" data-i18n="view.s45Q.bonus_note">
                    5× bonus multiplier active: prevailing wage + apprenticeship compliance unlocks full
                    $85 / $60 / $180 rate vs $17 / $12 / $36 base. CRITICAL for project economics —
                    documentation + Davis-Bacon Act compliance required throughout construction + 5-yr
                    repair period post-PIS.
                </p>
            ` : ''}
        </div>
    `;
}
