// ILIT — Irrevocable Life Insurance Trust.
// Get life insurance OUT of taxable estate. Trust owns policy + names trust as beneficiary.
// Crummey withdrawal rights: convert future-interest gifts into present interest → use
// $18k/$36k MFJ annual gift exclusion. 3-yr lookback: existing policy transfers within
// 3 yrs of death → assets back in estate. Pay premiums via Crummey-eligible gifts.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const ANNUAL_EXCLUSION_2024 = 18_000;
const FED_ESTATE_RATE = 0.40;

let state = {
    death_benefit: 0,
    annual_premium: 0,
    crummey_beneficiaries: 1,
    is_mfj: false,
    years_existing_policy: 0,
    estate_value: 0,
    lifetime_exemption_remaining: 13_610_000,
    state_estate_rate: 0,
};

export async function renderIlit(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ilit.h1.title">// ILIT — IRREV. LIFE INSURANCE TRUST</span></h1>
        <p class="muted small" data-i18n="view.ilit.hint.intro">
            Holds life insurance outside estate. ILIT applies for / owns / names beneficiary —
            you can't have <strong>incidents of ownership</strong>. Pay premiums via Crummey
            letter gifts (each beneficiary gets 30-day withdrawal right → converts future
            interest into <strong>present interest</strong>, qualifying for $18k annual
            exclusion). <strong>3-year lookback:</strong> existing policy transferred to ILIT
            within 3 yrs of death → insured-owned at death → in estate.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.ilit.h2.inputs">Inputs</h2>
            <form id="ilit-form" class="inline-form">
                <label><span data-i18n="view.ilit.label.death_benefit">Death benefit ($)</span>
                    <input type="number" step="0.01" name="death_benefit" value="${state.death_benefit}"></label>
                <label><span data-i18n="view.ilit.label.annual_premium">Annual premium ($)</span>
                    <input type="number" step="0.01" name="annual_premium" value="${state.annual_premium}"></label>
                <label><span data-i18n="view.ilit.label.crummey_count">Crummey beneficiary count</span>
                    <input type="number" step="1" name="crummey_beneficiaries" value="${state.crummey_beneficiaries}"></label>
                <label><span data-i18n="view.ilit.label.mfj">MFJ (split gifts)?</span>
                    <input type="checkbox" name="is_mfj" ${state.is_mfj ? 'checked' : ''}></label>
                <label><span data-i18n="view.ilit.label.existing_years">Years since policy origination (if pre-existing)</span>
                    <input type="number" step="1" name="years_existing_policy" value="${state.years_existing_policy}"></label>
                <label><span data-i18n="view.ilit.label.estate_value">Total taxable estate ($)</span>
                    <input type="number" step="0.01" name="estate_value" value="${state.estate_value}"></label>
                <label><span data-i18n="view.ilit.label.lifetime_remaining">Lifetime exemption remaining ($)</span>
                    <input type="number" step="0.01" name="lifetime_exemption_remaining" value="${state.lifetime_exemption_remaining}"></label>
                <label><span data-i18n="view.ilit.label.state_rate">State estate tax rate</span>
                    <input type="number" step="0.01" name="state_estate_rate" value="${state.state_estate_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.ilit.btn.compute">Compute</button>
            </form>
        </div>
        <div id="ilit-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.ilit.h2.compliance">Compliance checklist</h2>
            <ul class="muted small">
                <li data-i18n="view.ilit.compl.no_ownership">Insured retains NO incidents of ownership (cash value access, beneficiary change, etc.)</li>
                <li data-i18n="view.ilit.compl.crummey_letter">Crummey letter to each beneficiary within reasonable time of premium gift</li>
                <li data-i18n="view.ilit.compl.30_days">Beneficiary withdrawal window typically 30 days</li>
                <li data-i18n="view.ilit.compl.5_5_rule">"5×5 power" limit: max(5%, $5,000) lapses avoid gift tax to grantor's other beneficiaries</li>
                <li data-i18n="view.ilit.compl.hanging_powers">"Hanging Crummey powers" carry over excess → use to avoid 5×5 lapses</li>
                <li data-i18n="view.ilit.compl.split_premium">If MFJ split-gift, file 709 even if no tax due</li>
                <li data-i18n="view.ilit.compl.transfer_for_value">§ 101(a)(2) transfer-for-value rule: don't sell policy to ILIT for cash</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.ilit.h2.alternatives">Alternatives if ILIT impractical</h2>
            <ul class="muted small">
                <li data-i18n="view.ilit.alt.spouse_owned">Spouse-owned policy (simpler, but in spouse's estate)</li>
                <li data-i18n="view.ilit.alt.private_split">Private split-dollar with ILIT (preserves cash value access)</li>
                <li data-i18n="view.ilit.alt.bonus">§ 162 executive bonus / REBA (deductible to employer)</li>
                <li data-i18n="view.ilit.alt.beneficiary">Adult-child-owned policy ("beneficiary-owned")</li>
            </ul>
        </div>
    `;
    document.getElementById('ilit-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.death_benefit = Number(fd.get('death_benefit')) || 0;
        state.annual_premium = Number(fd.get('annual_premium')) || 0;
        state.crummey_beneficiaries = Number(fd.get('crummey_beneficiaries')) || 1;
        state.is_mfj = !!fd.get('is_mfj');
        state.years_existing_policy = Number(fd.get('years_existing_policy')) || 0;
        state.estate_value = Number(fd.get('estate_value')) || 0;
        state.lifetime_exemption_remaining = Number(fd.get('lifetime_exemption_remaining')) || 0;
        state.state_estate_rate = Number(fd.get('state_estate_rate')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('ilit-output');
    if (!el) return;
    const annualExclPerBenef = state.is_mfj ? ANNUAL_EXCLUSION_2024 * 2 : ANNUAL_EXCLUSION_2024;
    const totalCrummeyCapacity = state.crummey_beneficiaries * annualExclPerBenef;
    const premiumWithinCrummey = Math.min(state.annual_premium, totalCrummeyCapacity);
    const excessPremium = Math.max(0, state.annual_premium - totalCrummeyCapacity);
    const lifetimeAfterExcess = state.lifetime_exemption_remaining - excessPremium;
    const fedEstateOnDB = state.death_benefit * FED_ESTATE_RATE;
    const stateEstateOnDB = state.death_benefit * state.state_estate_rate;
    const totalEstateOnDB = fedEstateOnDB + stateEstateOnDB;
    const lookbackRisk = state.years_existing_policy > 0 && state.years_existing_policy < 3;
    const aboveExemption = state.estate_value > state.lifetime_exemption_remaining;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.ilit.h2.result">Annual gifting + estate impact</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.ilit.card.crummey_capacity">Total Crummey capacity</div>
                    <div class="value">$${totalCrummeyCapacity.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.ilit.card.crummey_used">Premium under Crummey</div>
                    <div class="value">$${premiumWithinCrummey.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${excessPremium > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.ilit.card.over_crummey">Premium over Crummey</div>
                    <div class="value">$${excessPremium.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.ilit.card.fed_avoided">Federal estate tax avoided</div>
                    <div class="value">$${(aboveExemption ? fedEstateOnDB : 0).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.ilit.card.state_avoided">State estate tax avoided</div>
                    <div class="value">$${stateEstateOnDB.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.ilit.card.total_avoided">Total estate tax avoided</div>
                    <div class="value">$${(aboveExemption ? totalEstateOnDB : stateEstateOnDB).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${lookbackRisk ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.ilit.card.lookback">3-yr lookback risk</div>
                        <div class="value">${3 - state.years_existing_policy} ${esc(t('view.ilit.units.years_left'))}</div>
                    </div>
                ` : ''}
            </div>
            ${lookbackRisk ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.ilit.warning.lookback">
                    § 2035 3-year rule: existing-policy transfer to ILIT counts as if you owned at
                    death if you die within 3 years. Death benefit FULL VALUE back in estate.
                    Better: have ILIT APPLY for and own a NEW policy from inception.
                </p>
            ` : ''}
        </div>
    `;
}
