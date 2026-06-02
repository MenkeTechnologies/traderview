// IRC § 951 — Subpart F Income (CFC Anti-Deferral).
// CFC = > 50% US shareholders own (vote OR value); US shareholder = 10%+ vote OR value.
// US shareholders include pro-rata subpart F income currently (no actual distribution required).
// Categories: FBCI (FBC Sales / Services / Oil), insurance, foreign personal holding co income.
// § 954(b)(4) high-tax exception: foreign rate ≥ 90% × US rate → exclude.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    cfc_income_total: 0,
    fbc_sales: 0,
    fbc_services: 0,
    fpcsi: 0,
    insurance: 0,
    us_shareholder_pct: 0,
    foreign_tax_rate: 0,
    high_tax_election: false,
    de_minimis_test: false,
    full_inclusion_test: false,
    deemed_paid_credit: 0,
    s962_election: false,
};

export async function renderSection951(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s951.h1.title">// § 951 SUBPART F (CFC ANTI-DEFERRAL)</span></h1>
        <p class="muted small" data-i18n="view.s951.hint.intro">
            <strong>CFC test:</strong> &gt;50% US shareholders by vote OR value; <strong>US shareholder:</strong>
            10%+ vote OR value. US shareholders include pro-rata <strong>subpart F income currently</strong>
            (no distribution needed). Categories: FBC Sales / Services / Income from oil + insurance + FPHC
            income. <strong>§ 954(b)(4) high-tax exception:</strong> foreign rate ≥ 90% × US (≥ 18.9%) → exclude.
            <strong>De minimis &lt; 5% / $1M lesser; full inclusion &gt; 70%.</strong>
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s951.h2.inputs">Inputs</h2>
            <form id="s951-form" class="inline-form">
                <label><span data-i18n="view.s951.label.cfc_total">CFC gross income total ($)</span>
                    <input type="number" step="1000" name="cfc_income_total" value="${state.cfc_income_total}"></label>
                <label><span data-i18n="view.s951.label.fbc_sales">FBC Sales income ($)</span>
                    <input type="number" step="100" name="fbc_sales" value="${state.fbc_sales}"></label>
                <label><span data-i18n="view.s951.label.fbc_services">FBC Services income ($)</span>
                    <input type="number" step="100" name="fbc_services" value="${state.fbc_services}"></label>
                <label><span data-i18n="view.s951.label.fpcsi">FPHC (interest, div, rents) ($)</span>
                    <input type="number" step="100" name="fpcsi" value="${state.fpcsi}"></label>
                <label><span data-i18n="view.s951.label.insurance">Insurance income ($)</span>
                    <input type="number" step="100" name="insurance" value="${state.insurance}"></label>
                <label><span data-i18n="view.s951.label.us_pct">US shareholder pct %</span>
                    <input type="number" step="0.01" name="us_shareholder_pct" value="${state.us_shareholder_pct}"></label>
                <label><span data-i18n="view.s951.label.foreign_rate">Foreign effective tax rate %</span>
                    <input type="number" step="0.1" name="foreign_tax_rate" value="${state.foreign_tax_rate}"></label>
                <label><span data-i18n="view.s951.label.high_tax">§ 954(b)(4) high-tax election?</span>
                    <input type="checkbox" name="high_tax_election" ${state.high_tax_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s951.label.de_minimis">De minimis &lt; 5% / $1M?</span>
                    <input type="checkbox" name="de_minimis_test" ${state.de_minimis_test ? 'checked' : ''}></label>
                <label><span data-i18n="view.s951.label.full_inclusion">Full inclusion &gt; 70%?</span>
                    <input type="checkbox" name="full_inclusion_test" ${state.full_inclusion_test ? 'checked' : ''}></label>
                <label><span data-i18n="view.s951.label.dpc">§ 960 deemed paid credit ($)</span>
                    <input type="number" step="100" name="deemed_paid_credit" value="${state.deemed_paid_credit}"></label>
                <label><span data-i18n="view.s951.label.s962">§ 962 election (individual)?</span>
                    <input type="checkbox" name="s962_election" ${state.s962_election ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s951.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s951-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s951.h2.cfc_test">CFC determination test</h2>
            <ul class="muted small">
                <li data-i18n="view.s951.cfc.threshold">CFC: &gt; 50% combined US shareholder ownership (vote OR value), any time during year</li>
                <li data-i18n="view.s951.cfc.us_shareholder">US shareholder = 10%+ vote OR value (post-TCJA also value, not just vote)</li>
                <li data-i18n="view.s951.cfc.attribution">Constructive ownership rules: family, partnerships, trusts, corps</li>
                <li data-i18n="view.s951.cfc.10_percent">Sub-10% shareholders not US shareholders → escape currrent inclusion</li>
                <li data-i18n="view.s951.cfc.downward_attribution">2017 TCJA removed § 958(b)(4): downward attribution from foreign now applies</li>
                <li data-i18n="view.s951.cfc.foreign_corp">Foreign corp must be incorporated outside US</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s951.h2.categories">Subpart F income categories (§ 952)</h2>
            <ul class="muted small">
                <li data-i18n="view.s951.cat.fpcsi">FPHC income (§ 954(c)): interest, div, royalties, rents, capital gains, currency gains</li>
                <li data-i18n="view.s951.cat.fbc_sales">FBC Sales income (§ 954(d)): sales to / from related party + outside country of incorp</li>
                <li data-i18n="view.s951.cat.fbc_services">FBC Services income (§ 954(e)): services performed for related party outside country of incorp</li>
                <li data-i18n="view.s951.cat.fbc_oil">FBC Oil Related income (§ 954(g))</li>
                <li data-i18n="view.s951.cat.insurance">Insurance income (§ 953): underwriting risk outside country of incorp</li>
                <li data-i18n="view.s951.cat.boycott">International boycott income (§ 999)</li>
                <li data-i18n="view.s951.cat.bribery">Illegal bribes + kickbacks</li>
            </ul>
        </div>
    `;
    document.getElementById('s951-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.cfc_income_total = Number(fd.get('cfc_income_total')) || 0;
        state.fbc_sales = Number(fd.get('fbc_sales')) || 0;
        state.fbc_services = Number(fd.get('fbc_services')) || 0;
        state.fpcsi = Number(fd.get('fpcsi')) || 0;
        state.insurance = Number(fd.get('insurance')) || 0;
        state.us_shareholder_pct = Number(fd.get('us_shareholder_pct')) || 0;
        state.foreign_tax_rate = Number(fd.get('foreign_tax_rate')) || 0;
        state.high_tax_election = !!fd.get('high_tax_election');
        state.de_minimis_test = !!fd.get('de_minimis_test');
        state.full_inclusion_test = !!fd.get('full_inclusion_test');
        state.deemed_paid_credit = Number(fd.get('deemed_paid_credit')) || 0;
        state.s962_election = !!fd.get('s962_election');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s951-output');
    if (!el) return;
    const subFGross = state.fbc_sales + state.fbc_services + state.fpcsi + state.insurance;
    const highTaxQualifies = state.foreign_tax_rate >= 18.9;
    let subFAfterExclusions = subFGross;
    if (state.de_minimis_test) subFAfterExclusions = 0;
    if (state.high_tax_election && highTaxQualifies) subFAfterExclusions = 0;
    if (state.full_inclusion_test) subFAfterExclusions = state.cfc_income_total;
    const inclusionShare = subFAfterExclusions * (state.us_shareholder_pct / 100);
    const usTaxRate = state.s962_election ? 0.21 : 0.37;
    const grossUp = state.s962_election ? state.deemed_paid_credit : 0;
    const taxableUSInclusion = inclusionShare + grossUp;
    const usTax = taxableUSInclusion * usTaxRate;
    const netTaxAfterFTC = Math.max(0, usTax - state.deemed_paid_credit);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s951.h2.result">Subpart F inclusion</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s951.card.subF_gross">Subpart F gross</div>
                    <div class="value">$${subFGross.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${state.high_tax_election && highTaxQualifies ? 'pos' : ''}">
                    <div class="label" data-i18n="view.s951.card.high_tax">High-tax excl. (≥18.9%)</div>
                    <div class="value">${state.high_tax_election && highTaxQualifies ? esc(t('view.s951.status.applied')) : esc(t('view.s951.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s951.card.after_excl">After exclusions</div>
                    <div class="value">$${subFAfterExclusions.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s951.card.inclusion">US inclusion</div>
                    <div class="value">$${inclusionShare.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s951.card.us_rate">US rate</div>
                    <div class="value">${(usTaxRate * 100).toFixed(0)}%</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s951.card.us_tax">US tax pre-FTC</div>
                    <div class="value">$${usTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s951.card.net_tax">Net tax after FTC</div>
                    <div class="value">$${netTaxAfterFTC.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.s962_election ? `
                <p class="muted small" style="margin-top:10px" data-i18n="view.s951.s962_note">
                    § 962 individual election: tax at 21% corporate rate + claim § 960 deemed paid credit
                    (otherwise unavailable to individuals). Subsequent distribution still taxed as dividend
                    minus already-included PTEP.
                </p>
            ` : ''}
        </div>
    `;
}
