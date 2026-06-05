// Historic Rehabilitation Tax Credit § 47 — 20% of qualified rehab expenditures
// for Certified Historic Structures (NPS-listed or contributing to a registered
// historic district). TCJA changed this from full year-1 to 5-year spread.
// Phase-out by AGI; passive activity rules generally apply unless REP.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const CREDIT_RATE = 0.20;
const SPREAD_YEARS = 5;

let state = {
    qre: 0,
    placed_year: new Date().getFullYear(),
    is_certified_historic: true,
    is_passive: true,
    is_real_estate_professional: false,
    marginal_rate: 0.32,
    other_passive_income: 0,
};

export async function renderHistoricRehab(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.hrc.h1.title">// HISTORIC REHAB CREDIT § 47</span></h1>
        <p class="muted small" data-i18n="view.hrc.hint.intro">
            20% credit on qualified rehabilitation expenditures (QREs) for <strong>certified historic
            structures</strong>. Building must be either individually listed on the National Register OR
            contributing to a Registered Historic District. Post-TCJA: credit must be taken
            <strong>over 5 years</strong> (20% / 20% / 20% / 20% / 20%), not year-1 in full.
            Subject to passive activity rules unless REP status.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.hrc.h2.qualification">Qualification</h2>
            <ul class="muted small">
                <li data-i18n="view.hrc.qual.certified">Building must be a Certified Historic Structure (per NPS)</li>
                <li data-i18n="view.hrc.qual.substantial">"Substantial rehabilitation": QRE &gt; basis OR &gt; $5,000, whichever is higher, in any 24-month period</li>
                <li data-i18n="view.hrc.qual.qre">QREs: structural rehab, NOT acquisition cost, NOT enlargement, NOT site work</li>
                <li data-i18n="view.hrc.qual.approval">3-part NPS approval required: Part 1 (eligibility), Part 2 (proposed work), Part 3 (completed work)</li>
                <li data-i18n="view.hrc.qual.recapture">Recapture if disposed within 5 years: pro-rata clawback</li>
                <li data-i18n="view.hrc.qual.depreciation">Building basis reduced by 100% of credit taken (since 2018)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.hrc.h2.inputs">Inputs</h2>
            <form id="hrc-form" class="inline-form">
                <label><span data-i18n="view.hrc.label.placed_year">Placed in service year</span>
                    <input type="number" step="1" name="placed_year" value="${state.placed_year}" required></label>
                <label><span data-i18n="view.hrc.label.qre">Qualified Rehab Expenditures ($)</span>
                    <input type="number" step="0.01" name="qre" value="${state.qre}"></label>
                <label><span data-i18n="view.hrc.label.is_certified">Certified Historic Structure?</span>
                    <input type="checkbox" name="is_certified_historic" ${state.is_certified_historic ? 'checked' : ''}></label>
                <label><span data-i18n="view.hrc.label.is_passive">Passive investor?</span>
                    <input type="checkbox" name="is_passive" ${state.is_passive ? 'checked' : ''}></label>
                <label><span data-i18n="view.hrc.label.is_rep">Real Estate Professional?</span>
                    <input type="checkbox" name="is_real_estate_professional" ${state.is_real_estate_professional ? 'checked' : ''}></label>
                <label><span data-i18n="view.hrc.label.marginal">Marginal tax %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <label><span data-i18n="view.hrc.label.passive_income">Other passive income ($)</span>
                    <input type="number" step="0.01" name="other_passive_income" value="${state.other_passive_income}"></label>
                <button class="primary" type="submit" data-i18n="view.hrc.btn.compute">Compute</button>
            </form>
        </div>
        <div id="hrc-output"></div>
    `;
    document.getElementById('hrc-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.placed_year = Number(fd.get('placed_year')) || new Date().getFullYear();
        state.qre = Number(fd.get('qre')) || 0;
        state.is_certified_historic = !!fd.get('is_certified_historic');
        state.is_passive = !!fd.get('is_passive');
        state.is_real_estate_professional = !!fd.get('is_real_estate_professional');
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.32;
        state.other_passive_income = Number(fd.get('other_passive_income')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('hrc-output');
    if (!el) return;
    const eligible = state.is_certified_historic;
    const totalCredit = eligible ? state.qre * CREDIT_RATE : 0;
    const yearlyCredit = totalCredit / SPREAD_YEARS;
    const basisReduction = totalCredit;
    const taxSavings = totalCredit;
    const usablePassive = state.is_real_estate_professional ? totalCredit : Math.min(totalCredit, state.other_passive_income * state.marginal_rate);
    const suspendedCredit = state.is_real_estate_professional ? 0 : Math.max(0, totalCredit - usablePassive);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.hrc.h2.result">Credit calculation</h2>
            <div class="cards">
                <div class="card ${eligible ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.hrc.card.eligible">Eligible?</div>
                    <div class="value">${eligible ? esc(t('view.hrc.status.yes')) : esc(t('view.hrc.status.no'))}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.hrc.card.total_credit">Total credit (20% × QRE)</div>
                    <div class="value">$${totalCredit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.hrc.card.yearly_credit">Annual credit (5-yr spread)</div>
                    <div class="value">$${yearlyCredit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.hrc.card.basis_reduction">Basis reduction</div>
                    <div class="value">$${basisReduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.hrc.card.tax_savings">Lifetime tax savings</div>
                    <div class="value">$${taxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${state.is_passive && !state.is_real_estate_professional ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.hrc.card.suspended">Suspended (passive limited)</div>
                        <div class="value">$${suspendedCredit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
            </div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.hrc.h2.combined">Combine with LIHTC / opportunity zone</h2>
            <p class="muted" data-i18n="view.hrc.combined.body">
                Historic rehab credits frequently combine with LIHTC § 42 (low-income housing),
                opportunity zone § 1400Z deferral, and state historic credits (NY, MA, GA, RI,
                MO, VA — most have 20-25% state add-on). A typical certified historic + LIHTC +
                state credit + OZ stack can recover 40-55% of project cost via tax incentives.
            </p>
        </div>
    `;
}
