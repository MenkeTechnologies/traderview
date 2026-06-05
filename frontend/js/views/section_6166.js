// IRC § 6166 — Estate Tax Installment Payment for Closely-Held Businesses.
// Up to 14 years to pay federal estate tax attributable to closely-held business interest.
// First 4 years: interest only at favorable 2% rate (on first ~$1.85M tax 2024).
// Years 5-14: principal + interest (interest at 45% of underpayment rate over $1.85M base).
// Closely-held: > 35% of adjusted gross estate must be active trade or business.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const QUALIFICATION_THRESHOLD_PCT = 0.35;
const SPECIAL_RATE_TIER_2024 = 1_850_000;
const SPECIAL_INTEREST_RATE = 0.02;
const REGULAR_INTEREST_RATE_2024 = 0.08;
const TOTAL_YEARS = 14;
const INTEREST_ONLY_YEARS = 4;
const TAX_RATE = 0.40;

let state = {
    gross_estate: 0,
    closely_held_business_value: 0,
    business_active_pct: 0.50,
    lifetime_used: 0,
    spouse_dsue: 0,
    debts_and_admin: 0,
    state_estate_rate: 0,
};

export async function renderSection6166(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6166.h1.title">// § 6166 ESTATE TAX INSTALLMENT</span></h1>
        <p class="muted small" data-i18n="view.s6166.hint.intro">
            Up to <strong>14 years</strong> to pay federal estate tax on closely-held business
            interest. <strong>Years 1-4: interest-only</strong> at favorable <strong>2% rate</strong>
            (on first $1.85M tax, 2024). Years 5-14: P+I. Closely-held = <strong>&gt; 35% of
            adjusted gross estate</strong> must be active trade or business. <strong>Acceleration</strong>
            if 50% of business sold / undistributed income / late payment. Lien required;
            Form 706 election.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6166.h2.inputs">Inputs</h2>
            <form id="s6166-form" class="inline-form">
                <label><span data-i18n="view.s6166.label.gross">Gross estate ($)</span>
                    <input type="number" step="0.01" name="gross_estate" value="${state.gross_estate}"></label>
                <label><span data-i18n="view.s6166.label.business">Closely-held business value ($)</span>
                    <input type="number" step="0.01" name="closely_held_business_value" value="${state.closely_held_business_value}"></label>
                <label><span data-i18n="view.s6166.label.active_pct">Business active (vs passive) %</span>
                    <input type="number" step="0.01" name="business_active_pct" value="${state.business_active_pct}"></label>
                <label><span data-i18n="view.s6166.label.lifetime">Lifetime exemption used ($)</span>
                    <input type="number" step="0.01" name="lifetime_used" value="${state.lifetime_used}"></label>
                <label><span data-i18n="view.s6166.label.dsue">Spouse DSUE ($)</span>
                    <input type="number" step="0.01" name="spouse_dsue" value="${state.spouse_dsue}"></label>
                <label><span data-i18n="view.s6166.label.debts">Debts + admin expenses ($)</span>
                    <input type="number" step="0.01" name="debts_and_admin" value="${state.debts_and_admin}"></label>
                <label><span data-i18n="view.s6166.label.state_rate">State estate rate</span>
                    <input type="number" step="0.01" name="state_estate_rate" value="${state.state_estate_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s6166.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6166-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6166.h2.qualifying">Qualifying business interest</h2>
            <ul class="muted small">
                <li data-i18n="view.s6166.qual.sole">Sole proprietorship (active business)</li>
                <li data-i18n="view.s6166.qual.partnership">≥ 20% of partnership / LLC capital interest</li>
                <li data-i18n="view.s6166.qual.partnership_15">&lt; 20% if &lt; 45 partners (small partnership)</li>
                <li data-i18n="view.s6166.qual.corp_voting">≥ 20% voting stock of corp (any class)</li>
                <li data-i18n="view.s6166.qual.corp_15">&lt; 20% if &lt; 45 shareholders</li>
                <li data-i18n="view.s6166.qual.aggregate">Aggregate test: combine multiple businesses (each ≥ 20% required)</li>
                <li data-i18n="view.s6166.qual.farm_business">Farms, ranches, family agricultural operations qualify</li>
                <li data-i18n="view.s6166.qual.holding_companies">Holding companies: must look through to operating sub</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6166.h2.acceleration">Acceleration events (full balance due)</h2>
            <ul class="muted small">
                <li data-i18n="view.s6166.acc.50_pct">≥ 50% of qualifying business disposed of or withdrawn</li>
                <li data-i18n="view.s6166.acc.distribution">Undistributed net income retained (any amount triggers under § 6166(g)(2))</li>
                <li data-i18n="view.s6166.acc.late">Late payment of installment after 6-month grace</li>
                <li data-i18n="view.s6166.acc.distribution_corp">Corporate redemptions to pay estate tax (§ 303 carve-out limited)</li>
                <li data-i18n="view.s6166.acc.bankruptcy">Business bankruptcy / insolvency</li>
                <li data-i18n="view.s6166.acc.surety">Surety bond / lien defaults</li>
            </ul>
        </div>
    `;
    document.getElementById('s6166-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.gross_estate = Number(fd.get('gross_estate')) || 0;
        state.closely_held_business_value = Number(fd.get('closely_held_business_value')) || 0;
        state.business_active_pct = Number(fd.get('business_active_pct')) || 0.50;
        state.lifetime_used = Number(fd.get('lifetime_used')) || 0;
        state.spouse_dsue = Number(fd.get('spouse_dsue')) || 0;
        state.debts_and_admin = Number(fd.get('debts_and_admin')) || 0;
        state.state_estate_rate = Number(fd.get('state_estate_rate')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6166-output');
    if (!el) return;
    const adjustedGrossEstate = state.gross_estate - state.debts_and_admin;
    const qualifyingBusinessValue = state.closely_held_business_value * state.business_active_pct;
    const qualifyingPct = adjustedGrossEstate > 0 ? qualifyingBusinessValue / adjustedGrossEstate : 0;
    const qualifies = qualifyingPct > QUALIFICATION_THRESHOLD_PCT;
    const exemption = 13_610_000 + state.spouse_dsue - state.lifetime_used;
    const taxableEstate = Math.max(0, adjustedGrossEstate - exemption);
    const federalTaxTotal = taxableEstate * TAX_RATE;
    const stateTax = adjustedGrossEstate * state.state_estate_rate;
    const businessShareOfTax = adjustedGrossEstate > 0
        ? federalTaxTotal * (qualifyingBusinessValue / adjustedGrossEstate)
        : 0;
    const taxEligibleForInstallment = qualifies ? businessShareOfTax : 0;
    const specialRateTaxCap = Math.min(taxEligibleForInstallment, SPECIAL_RATE_TIER_2024);
    const regularRateTaxAbove = Math.max(0, taxEligibleForInstallment - SPECIAL_RATE_TIER_2024);
    const remainingDueNow = federalTaxTotal - taxEligibleForInstallment;
    // Interest first 4 yrs (interest only)
    const yr1to4Interest = (specialRateTaxCap * SPECIAL_INTEREST_RATE
        + regularRateTaxAbove * (REGULAR_INTEREST_RATE_2024 * 0.45)) * INTEREST_ONLY_YEARS;
    // Years 5-14 (10 yrs amortization + interest)
    const principalPerYear = taxEligibleForInstallment / 10;
    let principalRemaining = taxEligibleForInstallment;
    let yr5to14Interest = 0;
    for (let y = 1; y <= 10; y++) {
        const specialPortion = Math.min(principalRemaining, SPECIAL_RATE_TIER_2024);
        const regularPortion = Math.max(0, principalRemaining - SPECIAL_RATE_TIER_2024);
        yr5to14Interest += specialPortion * SPECIAL_INTEREST_RATE + regularPortion * (REGULAR_INTEREST_RATE_2024 * 0.45);
        principalRemaining -= principalPerYear;
    }
    const totalInterest = yr1to4Interest + yr5to14Interest;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6166.h2.result">Installment analysis</h2>
            <div class="cards">
                <div class="card ${qualifies ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s6166.card.qualifies">Qualifies (&gt; 35%)</div>
                    <div class="value">${qualifies ? esc(t('view.s6166.status.yes')) : esc(t('view.s6166.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6166.card.qualifying_pct">Business % of adjusted estate</div>
                    <div class="value">${(qualifyingPct * 100).toFixed(0)}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6166.card.federal_tax">Federal estate tax</div>
                    <div class="value">$${federalTaxTotal.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s6166.card.eligible">Tax eligible for installment</div>
                    <div class="value">$${taxEligibleForInstallment.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6166.card.due_now">Due at 9 mo (non-eligible portion)</div>
                    <div class="value">$${remainingDueNow.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6166.card.principal_annual">Principal/yr (yrs 5-14)</div>
                    <div class="value">$${principalPerYear.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6166.card.total_interest">Total interest over 14 yrs</div>
                    <div class="value">$${totalInterest.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6166.card.state">State estate tax</div>
                    <div class="value">$${stateTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
