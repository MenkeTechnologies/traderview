// IRC § 2056 — Marital Deduction + QTIP (Qualified Terminable Interest Property).
// Unlimited marital deduction for property passing to US-citizen surviving spouse.
// Non-citizen spouse: $185,000 (2024) annual exclusion + QDOT trust required.
// QTIP § 2056(b)(7): qualifying income interest for life → first spouse uses exemption,
// surviving spouse gets income, remainder per first spouse's wishes.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const NON_CITIZEN_ANNUAL_EXCLUSION_2024 = 185_000;
const LIFETIME_EXEMPTION_2024 = 13_610_000;
const FED_ESTATE_RATE = 0.40;

let state = {
    spouse_is_us_citizen: true,
    gross_estate: 0,
    bequest_to_spouse_outright: 0,
    bequest_to_spouse_qtip: 0,
    bequest_to_qdot: 0,
    other_bequests: 0,
    deceased_lifetime_used: 0,
    state_estate_rate: 0,
    surviving_estate_growth: 0.05,
    years_until_second_death: 15,
};

export async function renderSection2056(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s2056.h1.title">// § 2056 MARITAL DEDUCTION + QTIP</span></h1>
        <p class="muted small" data-i18n="view.s2056.hint.intro">
            Unlimited marital deduction for property passing to <strong>US-citizen</strong>
            surviving spouse. <strong>Non-citizen spouse:</strong> $185,000 (2024) annual exclusion
            + <strong>QDOT trust required</strong> for marital deduction. <strong>QTIP § 2056(b)(7):</strong>
            qualifying income-interest for life. First spouse uses exemption; second spouse gets
            income, NOT principal; remainder goes per first spouse's wishes. Common in blended families.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s2056.h2.inputs">Inputs</h2>
            <form id="s2056-form" class="inline-form">
                <label><span data-i18n="view.s2056.label.us_citizen">Spouse US citizen?</span>
                    <input type="checkbox" name="spouse_is_us_citizen" ${state.spouse_is_us_citizen ? 'checked' : ''}></label>
                <label><span data-i18n="view.s2056.label.gross_estate">Gross estate ($)</span>
                    <input type="number" step="100000" name="gross_estate" value="${state.gross_estate}"></label>
                <label><span data-i18n="view.s2056.label.outright">Bequest to spouse OUTRIGHT ($)</span>
                    <input type="number" step="100000" name="bequest_to_spouse_outright" value="${state.bequest_to_spouse_outright}"></label>
                <label><span data-i18n="view.s2056.label.qtip">QTIP trust to spouse ($)</span>
                    <input type="number" step="100000" name="bequest_to_spouse_qtip" value="${state.bequest_to_spouse_qtip}"></label>
                <label><span data-i18n="view.s2056.label.qdot">QDOT trust (non-citizen) ($)</span>
                    <input type="number" step="100000" name="bequest_to_qdot" value="${state.bequest_to_qdot}"></label>
                <label><span data-i18n="view.s2056.label.other">Other bequests (kids / charity) ($)</span>
                    <input type="number" step="100000" name="other_bequests" value="${state.other_bequests}"></label>
                <label><span data-i18n="view.s2056.label.lifetime_used">Lifetime exemption used ($)</span>
                    <input type="number" step="100000" name="deceased_lifetime_used" value="${state.deceased_lifetime_used}"></label>
                <label><span data-i18n="view.s2056.label.state_rate">State estate rate</span>
                    <input type="number" step="0.01" name="state_estate_rate" value="${state.state_estate_rate}"></label>
                <label><span data-i18n="view.s2056.label.growth">Survivor estate growth %</span>
                    <input type="number" step="0.01" name="surviving_estate_growth" value="${state.surviving_estate_growth}"></label>
                <label><span data-i18n="view.s2056.label.years">Years until second death</span>
                    <input type="number" step="1" name="years_until_second_death" value="${state.years_until_second_death}"></label>
                <button class="primary" type="submit" data-i18n="view.s2056.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s2056-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s2056.h2.qtip">QTIP requirements</h2>
            <ul class="muted small">
                <li data-i18n="view.s2056.qtip.income_interest">Surviving spouse must receive ALL income annually for life</li>
                <li data-i18n="view.s2056.qtip.no_others">No other person may receive income or principal during spouse's life</li>
                <li data-i18n="view.s2056.qtip.election">Executor must affirmatively elect QTIP on Form 706 within 9 mo + 6 mo extension</li>
                <li data-i18n="view.s2056.qtip.income_required">Income must be distributed at LEAST annually</li>
                <li data-i18n="view.s2056.qtip.spouse_estate">QTIP property INCLUDED in surviving spouse's estate at death (§ 2044)</li>
                <li data-i18n="view.s2056.qtip.no_gst">QTIP DOES NOT use GST exemption — separate election needed</li>
                <li data-i18n="view.s2056.qtip.reverse">Reverse QTIP election for GST: partial elect on portion for QTIP, reverse on rest</li>
                <li data-i18n="view.s2056.qtip.partial">Partial election allowed (only portion of trust qualifies)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s2056.h2.qdot">QDOT requirements (non-citizen spouse)</h2>
            <ul class="muted small">
                <li data-i18n="view.s2056.qdot.us_trustee">At least one US-citizen / US-corp trustee required</li>
                <li data-i18n="view.s2056.qdot.security">Trust assets &gt; $2M: security bond or escrow OR &gt; 35% in US real property</li>
                <li data-i18n="view.s2056.qdot.distribution_tax">Distributions of principal trigger DEFERRED estate tax at first decedent's rate</li>
                <li data-i18n="view.s2056.qdot.naturalization">Surviving spouse naturalizes → QDOT requirement ends</li>
                <li data-i18n="view.s2056.qdot.hardship">Income distributions + § 2056A hardship distributions exempt from estate tax</li>
                <li data-i18n="view.s2056.qdot.formula">Marital deduction reduces estate to LOWER OF actual transfer or QDOT amount</li>
            </ul>
        </div>
    `;
    document.getElementById('s2056-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.spouse_is_us_citizen = !!fd.get('spouse_is_us_citizen');
        state.gross_estate = Number(fd.get('gross_estate')) || 0;
        state.bequest_to_spouse_outright = Number(fd.get('bequest_to_spouse_outright')) || 0;
        state.bequest_to_spouse_qtip = Number(fd.get('bequest_to_spouse_qtip')) || 0;
        state.bequest_to_qdot = Number(fd.get('bequest_to_qdot')) || 0;
        state.other_bequests = Number(fd.get('other_bequests')) || 0;
        state.deceased_lifetime_used = Number(fd.get('deceased_lifetime_used')) || 0;
        state.state_estate_rate = Number(fd.get('state_estate_rate')) || 0;
        state.surviving_estate_growth = Number(fd.get('surviving_estate_growth')) || 0.05;
        state.years_until_second_death = Number(fd.get('years_until_second_death')) || 15;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s2056-output');
    if (!el) return;
    const maritalDeduction = state.spouse_is_us_citizen
        ? state.bequest_to_spouse_outright + state.bequest_to_spouse_qtip
        : state.bequest_to_qdot;
    const taxableEstate = Math.max(0, state.gross_estate - maritalDeduction - state.other_bequests);
    const exemption = Math.max(0, LIFETIME_EXEMPTION_2024 - state.deceased_lifetime_used);
    const taxableAfterExemption = Math.max(0, taxableEstate - exemption);
    const fedTaxFirst = taxableAfterExemption * FED_ESTATE_RATE;
    const stateTaxFirst = taxableEstate * state.state_estate_rate;
    // Survivor's estate at second death
    const inheritedAmount = maritalDeduction;
    const survivorEstateAtDeath = inheritedAmount * Math.pow(1 + state.surviving_estate_growth, state.years_until_second_death);
    const survivorExemption = LIFETIME_EXEMPTION_2024;
    const survivorTaxable = Math.max(0, survivorEstateAtDeath - survivorExemption);
    const survivorTax = survivorTaxable * FED_ESTATE_RATE;
    const totalCombinedTax = fedTaxFirst + stateTaxFirst + survivorTax;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s2056.h2.result">Estate planning outcome</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.s2056.card.marital">Marital deduction</div>
                    <div class="value">$${maritalDeduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s2056.card.taxable">Taxable estate</div>
                    <div class="value">$${taxableEstate.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s2056.card.exemption">Exemption used</div>
                    <div class="value">$${Math.min(taxableEstate, exemption).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s2056.card.fed_first">Federal estate (first)</div>
                    <div class="value">$${fedTaxFirst.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s2056.card.state_first">State estate (first)</div>
                    <div class="value">$${stateTaxFirst.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s2056.card.survivor_value">Survivor's estate at death</div>
                    <div class="value">$${survivorEstateAtDeath.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s2056.card.survivor_tax">Survivor estate tax</div>
                    <div class="value">$${survivorTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s2056.card.combined">Combined estate taxes</div>
                    <div class="value">$${totalCombinedTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${!state.spouse_is_us_citizen ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s2056.warning.non_citizen">
                    Non-citizen spouse: marital deduction DENIED on outright transfers. Use QDOT
                    trust. $185k (2024) annual exclusion for inter-spouse gifts during life;
                    death-time transfers ONLY through QDOT.
                </p>
            ` : ''}
        </div>
    `;
}
