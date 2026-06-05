// SLAT — Spousal Lifetime Access Trust.
// Use lifetime gift exemption ($13.61M 2024, sunsetting to ~$7M in 2026)
// while preserving indirect access via spouse-beneficiary.
// Each spouse can create one for the OTHER (reciprocal-trust trap: must be sufficiently
// different terms / timing to survive IRS scrutiny).

import { currentViewToken, viewIsCurrent } from '../app.js';

const LIFETIME_2024 = 13_610_000;
const LIFETIME_2025 = 13_990_000;
const LIFETIME_2026_SUNSET = 7_000_000;  // Post-TCJA sunset
const FEDERAL_ESTATE_RATE = 0.40;
const GST_2024 = 13_610_000;

let state = {
    gift_year: new Date().getFullYear() + 1,
    gift_amount: 0,
    other_spouse_gift: 0,  // For reciprocal-trust test
    current_lifetime_used: 0,
    spouse_current_lifetime_used: 0,
    asset_growth_rate: 0.07,
    years_until_death: 25,
    state_estate_tax_rate: 0,
};

export async function renderSlat(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.slat.h1.title">// SLAT — SPOUSAL LIFETIME ACCESS TRUST</span></h1>
        <p class="muted small" data-i18n="view.slat.hint.intro">
            Lock in <strong>$13.61M ($27.22M MFJ) lifetime exemption</strong> before 2026 sunset
            to <strong>~$7M ($14M MFJ)</strong>. Gift to irrevocable trust for spouse's benefit
            — they have access during marriage. Each spouse can create one for the OTHER.
            <strong>Reciprocal-trust trap:</strong> trusts MUST differ in material terms
            (timing, beneficiaries, distribution standards) or IRS treats both as if grantor
            kept rights → assets back in estate.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.slat.h2.inputs">Inputs</h2>
            <form id="slat-form" class="inline-form">
                <label><span data-i18n="view.slat.label.gift_year">Gift year</span>
                    <input type="number" step="1" name="gift_year" value="${state.gift_year}"></label>
                <label><span data-i18n="view.slat.label.gift_amount">Your gift to spouse's SLAT ($)</span>
                    <input type="number" step="0.01" name="gift_amount" value="${state.gift_amount}"></label>
                <label><span data-i18n="view.slat.label.other_gift">Spouse's gift to YOUR SLAT ($)</span>
                    <input type="number" step="0.01" name="other_spouse_gift" value="${state.other_spouse_gift}"></label>
                <label><span data-i18n="view.slat.label.lifetime_used">Your lifetime already used ($)</span>
                    <input type="number" step="0.01" name="current_lifetime_used" value="${state.current_lifetime_used}"></label>
                <label><span data-i18n="view.slat.label.spouse_lifetime_used">Spouse's lifetime used ($)</span>
                    <input type="number" step="0.01" name="spouse_current_lifetime_used" value="${state.spouse_current_lifetime_used}"></label>
                <label><span data-i18n="view.slat.label.growth_rate">Asset growth rate</span>
                    <input type="number" step="0.01" name="asset_growth_rate" value="${state.asset_growth_rate}"></label>
                <label><span data-i18n="view.slat.label.years_death">Years until death</span>
                    <input type="number" step="1" name="years_until_death" value="${state.years_until_death}"></label>
                <label><span data-i18n="view.slat.label.state_rate">State estate tax rate</span>
                    <input type="number" step="0.01" name="state_estate_tax_rate" value="${state.state_estate_tax_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.slat.btn.compute">Compute</button>
            </form>
        </div>
        <div id="slat-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.slat.h2.reciprocal">Avoiding reciprocal-trust doctrine</h2>
            <ul class="muted small">
                <li data-i18n="view.slat.recip.different_trustees">Different trustees</li>
                <li data-i18n="view.slat.recip.different_beneficiaries">Different remainder beneficiaries (one excludes child, etc.)</li>
                <li data-i18n="view.slat.recip.different_powers">Different distribution standards (HEMS vs ascertainable vs discretionary)</li>
                <li data-i18n="view.slat.recip.different_timing">Stagger creation 6-12 months apart</li>
                <li data-i18n="view.slat.recip.different_amounts">Materially different amounts</li>
                <li data-i18n="view.slat.recip.different_powers_appt">Different powers of appointment</li>
                <li data-i18n="view.slat.recip.estate_of_grace">Estate of Grace v. Comm'r (1969) is the leading case</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.slat.h2.risks">Risks to consider</h2>
            <ul class="muted small">
                <li data-i18n="view.slat.risk.divorce">DIVORCE: you lose indirect access via spouse; trust continues for ex-spouse</li>
                <li data-i18n="view.slat.risk.spouse_death">Spouse death before yours: loss of indirect access, but assets stay out of estate</li>
                <li data-i18n="view.slat.risk.no_step_up">No § 1014 step-up at death — beneficiary inherits carryover basis</li>
                <li data-i18n="view.slat.risk.grantor_trust">Most SLATs are grantor trusts — YOU pay the income tax (adds wealth transfer)</li>
                <li data-i18n="view.slat.risk.clawback_election">Clawback regs Rev. Proc. 2018-52: anti-clawback for completed gifts</li>
            </ul>
        </div>
    `;
    document.getElementById('slat-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.gift_year = Number(fd.get('gift_year')) || new Date().getFullYear() + 1;
        state.gift_amount = Number(fd.get('gift_amount')) || 0;
        state.other_spouse_gift = Number(fd.get('other_spouse_gift')) || 0;
        state.current_lifetime_used = Number(fd.get('current_lifetime_used')) || 0;
        state.spouse_current_lifetime_used = Number(fd.get('spouse_current_lifetime_used')) || 0;
        state.asset_growth_rate = Number(fd.get('asset_growth_rate')) || 0.07;
        state.years_until_death = Number(fd.get('years_until_death')) || 25;
        state.state_estate_tax_rate = Number(fd.get('state_estate_tax_rate')) || 0;
        renderOutput();
    });
    renderOutput();
}

function exemptionForYear(y) {
    if (y >= 2026) return LIFETIME_2026_SUNSET;
    if (y >= 2025) return LIFETIME_2025;
    return LIFETIME_2024;
}

function renderOutput() {
    const el = document.getElementById('slat-output');
    if (!el) return;
    const yourLimit = exemptionForYear(state.gift_year);
    const spouseLimit = exemptionForYear(state.gift_year);
    const yourAvailable = Math.max(0, yourLimit - state.current_lifetime_used);
    const spouseAvailable = Math.max(0, spouseLimit - state.spouse_current_lifetime_used);
    const yourUsable = Math.min(state.gift_amount, yourAvailable);
    const spouseUsable = Math.min(state.other_spouse_gift, spouseAvailable);
    const yourOver = Math.max(0, state.gift_amount - yourUsable);
    const spouseOver = Math.max(0, state.other_spouse_gift - spouseUsable);
    const totalGifted = yourUsable + spouseUsable;
    const growthFactor = Math.pow(1 + state.asset_growth_rate, state.years_until_death);
    const futureValue = totalGifted * growthFactor;
    const wouldHaveBeenInEstate = futureValue;
    const fedEstateSaved = wouldHaveBeenInEstate * FEDERAL_ESTATE_RATE;
    const stateEstateSaved = wouldHaveBeenInEstate * state.state_estate_tax_rate;
    const totalEstateSaved = fedEstateSaved + stateEstateSaved;
    const giftTaxOverages = (yourOver + spouseOver) * 0.40;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.slat.h2.result">Estate transfer outcome</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.slat.card.your_lifetime">Your lifetime available</div>
                    <div class="value">$${yourAvailable.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.slat.card.spouse_lifetime">Spouse lifetime available</div>
                    <div class="value">$${spouseAvailable.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.slat.card.gifted">Total gifted</div>
                    <div class="value">$${totalGifted.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${(yourOver + spouseOver) > 0 ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.slat.card.over_exemption">Over exemption (taxable gift)</div>
                        <div class="value">$${(yourOver + spouseOver).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                    <div class="card neg">
                        <div class="label" data-i18n="view.slat.card.gift_tax_due">Gift tax due (40%)</div>
                        <div class="value">$${giftTaxOverages.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
                <div class="card pos">
                    <div class="label" data-i18n="view.slat.card.future_value">Future value at death</div>
                    <div class="value">$${futureValue.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.slat.card.federal_saved">Federal estate tax saved</div>
                    <div class="value">$${fedEstateSaved.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.slat.card.state_saved">State estate tax saved</div>
                    <div class="value">$${stateEstateSaved.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.slat.card.total_saved">Total tax saved</div>
                    <div class="value">$${totalEstateSaved.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.gift_year >= 2026 ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.slat.warning.sunset">
                    POST-2026 sunset: lifetime exemption drops from $13.61M to ~$7M per spouse.
                    Treasury anti-clawback regs (Rev. Proc. 2018-52) preserve completed pre-sunset
                    gifts. If estate-exposed, complete SLATs BEFORE 2026.
                </p>
            ` : ''}
        </div>
    `;
}
