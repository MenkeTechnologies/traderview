// Grantor Retained Annuity Trust (GRAT) calculator.
// Walton GRAT (zeroed-out): annuity = principal × (1 + 7520-rate)^term / annuity-factor.
// All appreciation above the IRS 7520 rate passes to remainder beneficiaries
// gift-tax free. Survivor risk: if grantor dies during term, assets back to estate.

import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    principal: 0,
    section_7520_rate: 0.0520,  // Approx current
    term_years: 2,
    growth_rate: 0.10,
    grantor_marginal_rate: 0.37,
};

function annuityFactor(rate, term) {
    return (1 - Math.pow(1 + rate, -term)) / rate;
}

export async function renderGrat(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.grat.h1.title">// GRAT — GRANTOR RETAINED ANNUITY TRUST</span></h1>
        <p class="muted small" data-i18n="view.grat.hint.intro">
            Estate-freezing technique. You contribute volatile / appreciating asset (pre-IPO
            stock, RSU shares, growth stock). Receive back an annuity over 2-10 yrs equal
            to principal + 7520 rate. <strong>Walton zeroed-out GRAT</strong>: gift value = $0.
            Excess appreciation passes to remainder beneficiaries (kids, trust) GIFT-TAX-FREE.
            Risk: grantor must SURVIVE the term or assets reverse to estate.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.grat.h2.inputs">Inputs</h2>
            <form id="grat-form" class="inline-form">
                <label><span data-i18n="view.grat.label.principal">Asset contributed ($)</span>
                    <input type="number" step="10000" name="principal" value="${state.principal}"></label>
                <label><span data-i18n="view.grat.label.7520">IRC § 7520 rate (current month)</span>
                    <input type="number" step="0.0001" name="section_7520_rate" value="${state.section_7520_rate}"></label>
                <label><span data-i18n="view.grat.label.term">GRAT term (years)</span>
                    <input type="number" step="1" min="2" max="20" name="term_years" value="${state.term_years}"></label>
                <label><span data-i18n="view.grat.label.growth">Expected asset growth rate</span>
                    <input type="number" step="0.01" name="growth_rate" value="${state.growth_rate}"></label>
                <label><span data-i18n="view.grat.label.marginal">Grantor marginal rate</span>
                    <input type="number" step="0.01" name="grantor_marginal_rate" value="${state.grantor_marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.grat.btn.compute">Compute</button>
            </form>
        </div>
        <div id="grat-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.grat.h2.mechanics">Mechanics</h2>
            <ul class="muted small">
                <li data-i18n="view.grat.mech.zero">Walton GRAT zeroes out the gift: annuity stream actuarially = principal × 7520</li>
                <li data-i18n="view.grat.mech.short">Short term (2-3 yrs) maximizes appreciation transfer + survives mortality risk</li>
                <li data-i18n="view.grat.mech.rolling">"Rolling GRAT" strategy: chain 2-yr GRATs back-to-back, lock in gains as you go</li>
                <li data-i18n="view.grat.mech.grantor_trust">Grantor pays income tax on trust income — additional wealth transfer mechanism</li>
                <li data-i18n="view.grat.mech.swap_power">Substitution power lets grantor swap depreciated property out for cash</li>
                <li data-i18n="view.grat.mech.no_gst">GRATs DON'T allocate GST exemption efficiently (use SLAT / dynasty for grandchildren)</li>
                <li data-i18n="view.grat.mech.no_step_up">Remainder gets carryover basis — NO step-up at grantor death</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.grat.h2.risks">Risks</h2>
            <ul class="muted small">
                <li data-i18n="view.grat.risk.mortality">Grantor death during term: all assets revert to estate</li>
                <li data-i18n="view.grat.risk.under_perform">Underperformance: annuity exhausts trust, nothing to remainder</li>
                <li data-i18n="view.grat.risk.bba_proposal">Biden / BBB proposals threatened to REQUIRE 10-yr minimum, eliminate zero-GRAT</li>
                <li data-i18n="view.grat.risk.audit">IRS scrutiny on hard-to-value assets (private stock, real estate)</li>
            </ul>
        </div>
    `;
    document.getElementById('grat-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.principal = Number(fd.get('principal')) || 0;
        state.section_7520_rate = Number(fd.get('section_7520_rate')) || 0.05;
        state.term_years = Math.max(2, Math.min(20, Number(fd.get('term_years')) || 2));
        state.growth_rate = Number(fd.get('growth_rate')) || 0.10;
        state.grantor_marginal_rate = Number(fd.get('grantor_marginal_rate')) || 0.37;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('grat-output');
    if (!el) return;
    const af = annuityFactor(state.section_7520_rate, state.term_years);
    const annualAnnuity = state.principal / af;
    let trustBalance = state.principal;
    const yearlyTable = [];
    for (let y = 1; y <= state.term_years; y++) {
        trustBalance = trustBalance * (1 + state.growth_rate);
        const begOfYear = trustBalance;
        trustBalance -= annualAnnuity;
        yearlyTable.push({ year: y, beg: begOfYear, annuity: annualAnnuity, end: Math.max(0, trustBalance) });
    }
    const transferredToRemainder = Math.max(0, trustBalance);
    const totalReceivedByGrantor = annualAnnuity * state.term_years;
    const totalAppreciation = totalReceivedByGrantor + transferredToRemainder - state.principal;
    const estateTaxSavedAt40 = transferredToRemainder * 0.40;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.grat.h2.result">GRAT projection</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.grat.card.annuity">Annual annuity (Walton)</div>
                    <div class="value">$${annualAnnuity.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.grat.card.received">Grantor receives total</div>
                    <div class="value">$${totalReceivedByGrantor.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.grat.card.transferred">Remainder transferred (gift-free)</div>
                    <div class="value">$${transferredToRemainder.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.grat.card.appreciation">Total appreciation</div>
                    <div class="value">$${totalAppreciation.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.grat.card.estate_saved">Est. estate tax saved (40%)</div>
                    <div class="value">$${estateTaxSavedAt40.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.grat.card.gift_value">Gift value (Walton zeroed)</div>
                    <div class="value">$0</div>
                </div>
            </div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.grat.h2.year_table">Year-by-year</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.grat.th.year">Year</th>
                    <th data-i18n="view.grat.th.beg">Beg-of-year</th>
                    <th data-i18n="view.grat.th.annuity_paid">Annuity paid</th>
                    <th data-i18n="view.grat.th.end">End-of-year</th>
                </tr></thead>
                <tbody>${yearlyTable.map(r => `
                    <tr>
                        <td>${r.year}</td>
                        <td>$${r.beg.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                        <td>$${r.annuity.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                        <td class="pos">$${r.end.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        </div>
    `;
}
