// IRC § 2010(c) DSUE — Deceased Spousal Unused Exemption Portability.
// Surviving spouse can use deceased spouse's UNUSED lifetime exemption — must elect
// on Form 706 filed within 9 mo of death (15 mo with extension; Rev. Proc. 2022-32
// allows 5-yr late election with simplified relief). Combined exemption can double
// to ~$27M (2024) without trusts. Loss-of-election results in entire DSUE forfeited.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const EXEMPTION_2024 = 13_610_000;
const EXEMPTION_2026_SUNSET = 7_000_000;
const FED_ESTATE_RATE = 0.40;
const FILING_DEADLINE_MONTHS = 9;
const SIMPLIFIED_RELIEF_YEARS = 5;

let state = {
    death_year: new Date().getFullYear() - 1,
    deceased_lifetime_used: 0,
    deceased_estate_size: 0,
    surviving_age: 75,
    survivor_remaining_lifetime: 0,
    survivor_expected_growth_yrs: 10,
    survivor_growth_rate: 0.05,
    state_estate_rate: 0,
    elected_portability: false,
    months_since_death: 0,
};

export async function renderSection2010c(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s2010c.h1.title">// § 2010(c) DSUE PORTABILITY</span></h1>
        <p class="muted small" data-i18n="view.s2010c.hint.intro">
            Surviving spouse claims <strong>Deceased Spousal Unused Exemption (DSUE)</strong>.
            <strong>Election deadline: 9 months from death</strong> (15 with auto extension).
            <strong>Rev. Proc. 2022-32:</strong> simplified relief if not required to file —
            up to <strong>5 years late</strong>. Without election, deceased spouse's unused
            exemption is FORFEITED. Combined exemption up to ~$27M (2024) without trust planning.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s2010c.h2.inputs">Inputs</h2>
            <form id="s2010c-form" class="inline-form">
                <label><span data-i18n="view.s2010c.label.death_year">Year of first-to-die death</span>
                    <input type="number" step="1" name="death_year" value="${state.death_year}"></label>
                <label><span data-i18n="view.s2010c.label.months_since">Months since death</span>
                    <input type="number" step="1" name="months_since_death" value="${state.months_since_death}"></label>
                <label><span data-i18n="view.s2010c.label.dec_used">Deceased lifetime exemption already used ($)</span>
                    <input type="number" step="0.01" name="deceased_lifetime_used" value="${state.deceased_lifetime_used}"></label>
                <label><span data-i18n="view.s2010c.label.dec_estate">Deceased spouse's gross estate ($)</span>
                    <input type="number" step="0.01" name="deceased_estate_size" value="${state.deceased_estate_size}"></label>
                <label><span data-i18n="view.s2010c.label.surv_age">Survivor age</span>
                    <input type="number" step="1" name="surviving_age" value="${state.surviving_age}"></label>
                <label><span data-i18n="view.s2010c.label.surv_used">Survivor remaining lifetime ($)</span>
                    <input type="number" step="0.01" name="survivor_remaining_lifetime" value="${state.survivor_remaining_lifetime}"></label>
                <label><span data-i18n="view.s2010c.label.growth_yrs">Years to project survivor estate</span>
                    <input type="number" step="1" name="survivor_expected_growth_yrs" value="${state.survivor_expected_growth_yrs}"></label>
                <label><span data-i18n="view.s2010c.label.growth_rate">Estate growth rate</span>
                    <input type="number" step="0.01" name="survivor_growth_rate" value="${state.survivor_growth_rate}"></label>
                <label><span data-i18n="view.s2010c.label.state_rate">State estate tax rate</span>
                    <input type="number" step="0.01" name="state_estate_rate" value="${state.state_estate_rate}"></label>
                <label><span data-i18n="view.s2010c.label.elected">Election made on Form 706?</span>
                    <input type="checkbox" name="elected_portability" ${state.elected_portability ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s2010c.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s2010c-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s2010c.h2.advantages">Portability vs Bypass Trust</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s2010c.th.feature">Feature</th>
                    <th data-i18n="view.s2010c.th.portability">Portability</th>
                    <th data-i18n="view.s2010c.th.bypass">Bypass / Credit Shelter Trust</th>
                </tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s2010c.row.simplicity">Simplicity</td><td>HIGH (just file 706)</td><td>LOW (complex trust)</td></tr>
                    <tr><td data-i18n="view.s2010c.row.indexed">DSUE indexed for inflation?</td><td>NO (frozen at first death)</td><td>YES (trust grows)</td></tr>
                    <tr><td data-i18n="view.s2010c.row.gst">GST exemption portable?</td><td>NO</td><td>YES</td></tr>
                    <tr><td data-i18n="view.s2010c.row.step_up">Step-up at survivor death?</td><td>YES (full)</td><td>NO (carryover for trust)</td></tr>
                    <tr><td data-i18n="view.s2010c.row.state">State estate planning</td><td>Limited</td><td>Better for state-tax states</td></tr>
                    <tr><td data-i18n="view.s2010c.row.creditor">Creditor protection</td><td>NONE</td><td>STRONG (trust)</td></tr>
                    <tr><td data-i18n="view.s2010c.row.remarriage">Remarriage protection</td><td>NONE</td><td>STRONG</td></tr>
                    <tr><td data-i18n="view.s2010c.row.cost">Setup cost</td><td>$2-5k Form 706 only</td><td>$10-30k + ongoing admin</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s2010c.h2.warnings">Common DSUE mistakes</h2>
            <ul class="muted small">
                <li data-i18n="view.s2010c.warn.assumed_below">Assumed estate too small to need 706 → DSUE forfeited</li>
                <li data-i18n="view.s2010c.warn.late_deadline">9-month deadline missed → use Rev. Proc. 2022-32 simplified relief (5 yr)</li>
                <li data-i18n="view.s2010c.warn.no_inflation">DSUE FROZEN — survivor exemption indexes, deceased's does not</li>
                <li data-i18n="view.s2010c.warn.last_remarriage">"Last DSUE rule" — remarriage + death of new spouse without 706 → original DSUE lost</li>
                <li data-i18n="view.s2010c.warn.gst_separate">GST exemption is NEVER portable — separate planning needed</li>
                <li data-i18n="view.s2010c.warn.audit_open">Survivor's estate can re-open deceased's return to challenge DSUE valuation</li>
            </ul>
        </div>
    `;
    document.getElementById('s2010c-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.death_year = Number(fd.get('death_year')) || new Date().getFullYear();
        state.months_since_death = Number(fd.get('months_since_death')) || 0;
        state.deceased_lifetime_used = Number(fd.get('deceased_lifetime_used')) || 0;
        state.deceased_estate_size = Number(fd.get('deceased_estate_size')) || 0;
        state.surviving_age = Number(fd.get('surviving_age')) || 75;
        state.survivor_remaining_lifetime = Number(fd.get('survivor_remaining_lifetime')) || 0;
        state.survivor_expected_growth_yrs = Number(fd.get('survivor_expected_growth_yrs')) || 10;
        state.survivor_growth_rate = Number(fd.get('survivor_growth_rate')) || 0.05;
        state.state_estate_rate = Number(fd.get('state_estate_rate')) || 0;
        state.elected_portability = !!fd.get('elected_portability');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s2010c-output');
    if (!el) return;
    const exemptionAtDeath = state.death_year >= 2026 ? EXEMPTION_2026_SUNSET : EXEMPTION_2024;
    const dsue = Math.max(0, exemptionAtDeath - state.deceased_lifetime_used);
    const withinDeadline = state.months_since_death <= FILING_DEADLINE_MONTHS + 6;  // +extension
    const eligibleSimplified = state.months_since_death <= SIMPLIFIED_RELIEF_YEARS * 12;
    const survivor2026Exemption = EXEMPTION_2026_SUNSET;
    const projectedEstate = state.deceased_estate_size * Math.pow(1 + state.survivor_growth_rate, state.survivor_expected_growth_yrs);
    const totalExemptionWithPortability = survivor2026Exemption + dsue;
    const totalExemptionWithoutPortability = survivor2026Exemption;
    const taxableEstate = Math.max(0, projectedEstate - (state.elected_portability ? totalExemptionWithPortability : totalExemptionWithoutPortability));
    const federalTax = taxableEstate * FED_ESTATE_RATE;
    const stateTax = projectedEstate * state.state_estate_rate;
    const totalEstateTaxesIfElected = (state.elected_portability ? federalTax : 0) + stateTax;
    const totalEstateTaxesIfNot = (Math.max(0, projectedEstate - totalExemptionWithoutPortability) * FED_ESTATE_RATE) + stateTax;
    const savingsFromElection = totalEstateTaxesIfNot - totalEstateTaxesIfElected;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s2010c.h2.result">DSUE outcome</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.s2010c.card.dsue">DSUE available</div>
                    <div class="value">$${dsue.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${withinDeadline ? 'pos' : (eligibleSimplified ? '' : 'neg')}">
                    <div class="label" data-i18n="view.s2010c.card.deadline">Election status</div>
                    <div class="value">${withinDeadline ? esc(t('view.s2010c.status.within')) : (eligibleSimplified ? esc(t('view.s2010c.status.simplified')) : esc(t('view.s2010c.status.too_late')))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s2010c.card.projected">Survivor projected estate</div>
                    <div class="value">$${projectedEstate.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s2010c.card.with_portability">Exemption WITH portability</div>
                    <div class="value">$${totalExemptionWithPortability.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s2010c.card.without">Exemption WITHOUT portability</div>
                    <div class="value">$${totalExemptionWithoutPortability.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s2010c.card.savings">Estate tax saved by election</div>
                    <div class="value">$${savingsFromElection.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${!withinDeadline && eligibleSimplified ? `
                <p class="muted small" style="margin-top:10px" data-i18n="view.s2010c.note.simplified">
                    Past 15-mo deadline but within 5 yrs — file under Rev. Proc. 2022-32 simplified
                    relief. Mark Form 706 with "FILED PURSUANT TO REV. PROC. 2022-32" at top.
                </p>
            ` : ''}
        </div>
    `;
}
