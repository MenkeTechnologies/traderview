// IRC § 882 — Foreign Corporation Tax on Effectively Connected Income (ECI).
// Foreign corp engaged in US trade / business: taxed at regular US corporate rates on ECI.
// § 882(a)(1): general rule — taxed AS IF a US corporation on ECI.
// § 882(c): deductions allowed only with timely filing of accurate Form 1120-F.
// § 882(d): election to treat US real property income as ECI (combined with § 871(d) for individuals).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    eci_gross_income: 0,
    fdap_gross_income: 0,
    deductions_allowed: 0,
    foreign_tax_paid_on_eci: 0,
    branch_office_us: false,
    permanent_establishment: false,
    treaty_country: '',
    treaty_pe_protected: false,
    real_property_election: false,
    is_qualified_subsidiary: false,
    consolidated_with_us_parent: false,
    filed_form_1120f: true,
    days_late_filing: 0,
    estimated_tax_paid: 0,
};

export async function renderSection882(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s882.h1.title">// § 882 FOREIGN CORP ECI</span></h1>
        <p class="muted small" data-i18n="view.s882.hint.intro">
            Foreign corp <strong>engaged in US trade / business</strong>: taxed at regular US <strong>corporate rates
            on ECI</strong>. <strong>§ 882(a)(1):</strong> general rule — taxed AS IF a US corporation on ECI.
            <strong>§ 882(c):</strong> deductions allowed only with <strong>TIMELY FILING of accurate Form 1120-F</strong>.
            <strong>§ 882(d):</strong> election to treat US real property income as ECI. <strong>Treaty PE</strong>
            (Permanent Establishment) generally required if treaty country. <strong>FDAP</strong> separately
            withheld at 30% / treaty rate. <strong>§ 884:</strong> branch profits tax (BPT) on dividend equivalent.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s882.h2.inputs">Inputs</h2>
            <form id="s882-form" class="inline-form">
                <label><span data-i18n="view.s882.label.eci">ECI gross income ($)</span>
                    <input type="number" step="10000" name="eci_gross_income" value="${state.eci_gross_income}"></label>
                <label><span data-i18n="view.s882.label.fdap">FDAP gross income ($)</span>
                    <input type="number" step="10000" name="fdap_gross_income" value="${state.fdap_gross_income}"></label>
                <label><span data-i18n="view.s882.label.ded">Deductions allowed ($)</span>
                    <input type="number" step="10000" name="deductions_allowed" value="${state.deductions_allowed}"></label>
                <label><span data-i18n="view.s882.label.ftc">Foreign tax paid on ECI ($)</span>
                    <input type="number" step="1000" name="foreign_tax_paid_on_eci" value="${state.foreign_tax_paid_on_eci}"></label>
                <label><span data-i18n="view.s882.label.office">Branch office in US?</span>
                    <input type="checkbox" name="branch_office_us" ${state.branch_office_us ? 'checked' : ''}></label>
                <label><span data-i18n="view.s882.label.pe">Permanent Establishment?</span>
                    <input type="checkbox" name="permanent_establishment" ${state.permanent_establishment ? 'checked' : ''}></label>
                <label><span data-i18n="view.s882.label.country">Treaty country</span>
                    <input type="text" name="treaty_country" value="${esc(state.treaty_country)}"></label>
                <label><span data-i18n="view.s882.label.treaty_pe">Treaty PE protected?</span>
                    <input type="checkbox" name="treaty_pe_protected" ${state.treaty_pe_protected ? 'checked' : ''}></label>
                <label><span data-i18n="view.s882.label.real_prop">Real property election § 882(d)?</span>
                    <input type="checkbox" name="real_property_election" ${state.real_property_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s882.label.qualified_sub">Qualified US sub?</span>
                    <input type="checkbox" name="is_qualified_subsidiary" ${state.is_qualified_subsidiary ? 'checked' : ''}></label>
                <label><span data-i18n="view.s882.label.consolidated">Consolidated w/ US parent?</span>
                    <input type="checkbox" name="consolidated_with_us_parent" ${state.consolidated_with_us_parent ? 'checked' : ''}></label>
                <label><span data-i18n="view.s882.label.filed">Form 1120-F filed timely?</span>
                    <input type="checkbox" name="filed_form_1120f" ${state.filed_form_1120f ? 'checked' : ''}></label>
                <label><span data-i18n="view.s882.label.days_late">Days late filing</span>
                    <input type="number" step="1" name="days_late_filing" value="${state.days_late_filing}"></label>
                <label><span data-i18n="view.s882.label.est_tax">Estimated tax paid ($)</span>
                    <input type="number" step="1000" name="estimated_tax_paid" value="${state.estimated_tax_paid}"></label>
                <button class="primary" type="submit" data-i18n="view.s882.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s882-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s882.h2.eci_definition">"Effectively Connected Income" (ECI)</h2>
            <ul class="muted small">
                <li data-i18n="view.s882.eci.us_trade_business">Income effectively connected with US trade or business (USTB)</li>
                <li data-i18n="view.s882.eci.permanent_basis">Asset use test: assets used in conducting USTB</li>
                <li data-i18n="view.s882.eci.business_activities">Business activities test: substantial business activity in US giving rise to income</li>
                <li data-i18n="view.s882.eci.force_of_attraction">Force of attraction (limited): treaties often limit § 882 to permanent establishment</li>
                <li data-i18n="view.s882.eci.us_real_property">US real property: per § 897 FIRPTA — gain treated as ECI by election</li>
                <li data-i18n="view.s882.eci.dividends">Dividends + interest may be ECI if connected to USTB</li>
                <li data-i18n="view.s882.eci.s897">§ 897 FIRPTA: NRA / foreign corp gain on USRPI = ECI</li>
                <li data-i18n="view.s882.eci.s1446f">§ 1446(f): sale of US partnership interest = ECI portion deemed sold</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s882.h2.treaty_pe">Treaty Permanent Establishment</h2>
            <ul class="muted small">
                <li data-i18n="view.s882.pe.threshold">Treaty country: § 882 only if treaty PE exists (OECD Article 5)</li>
                <li data-i18n="view.s882.pe.fixed_place">PE: fixed place of business through which business carried on</li>
                <li data-i18n="view.s882.pe.examples">Examples: branch office, factory, mine, well, warehouse, dependent agent</li>
                <li data-i18n="view.s882.pe.exclusions">Excl: independent agent, storage, display, processing by third party</li>
                <li data-i18n="view.s882.pe.construction">Construction PE: > 12 months continuous</li>
                <li data-i18n="view.s882.pe.services">Services PE: services performed > 183 days within 12-month period (US-CA, US-NL)</li>
                <li data-i18n="view.s882.pe.attributable">Income attributable to PE — arms-length business</li>
                <li data-i18n="view.s882.pe.agent">Agent acting on behalf (binding contracts) — creates PE</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s882.h2.deductions">§ 882(c) deductions + protective filing</h2>
            <ul class="muted small">
                <li data-i18n="view.s882.ded.timely_filing">Deductions ALLOWED only if Form 1120-F filed within 18 months of original due date</li>
                <li data-i18n="view.s882.ded.expense_allocation">Expense allocation: directly-allocable + apportioned overhead</li>
                <li data-i18n="view.s882.ded.protective_filing">Protective filing: Form 1120-F as "Pro-forma" + statement of intent</li>
                <li data-i18n="view.s882.ded.charitable">Charitable: limited to 10% of taxable income before charitable deduction</li>
                <li data-i18n="view.s882.ded.dividends_paid">Dividends paid: NOT deductible (capital distribution)</li>
                <li data-i18n="view.s882.ded.s_corp_method">SE method: regulated investment company method (rare)</li>
                <li data-i18n="view.s882.ded.consolidated_return">Consolidated filing: with US affiliates if US sub member</li>
                <li data-i18n="view.s882.ded.s108_cod">§ 108 COD income: complex application to foreign corp ECI</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s882.h2.related">Related provisions</h2>
            <ul class="muted small">
                <li data-i18n="view.s882.rel.s871">§ 871: NRA individual analog</li>
                <li data-i18n="view.s882.rel.s884">§ 884: branch profits tax — 30% on dividend equivalent amount</li>
                <li data-i18n="view.s882.rel.s897">§ 897: FIRPTA real estate gain treated as ECI</li>
                <li data-i18n="view.s882.rel.s903_ftc">§ 903 FTC: foreign tax credit available for foreign tax on ECI</li>
                <li data-i18n="view.s882.rel.s6038c">§ 6038C: foreign-owned US corp reporting (Form 5472)</li>
                <li data-i18n="view.s882.rel.s7701b">§ 7701(b): does not apply to foreign corps (only individuals)</li>
                <li data-i18n="view.s882.rel.s1446a">§ 1446(a): partnership ECI withholding on foreign partners</li>
                <li data-i18n="view.s882.rel.s367b">§ 367(b): inbound rearrangements impact foreign corp tax</li>
            </ul>
        </div>
    `;
    document.getElementById('s882-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.eci_gross_income = Number(fd.get('eci_gross_income')) || 0;
        state.fdap_gross_income = Number(fd.get('fdap_gross_income')) || 0;
        state.deductions_allowed = Number(fd.get('deductions_allowed')) || 0;
        state.foreign_tax_paid_on_eci = Number(fd.get('foreign_tax_paid_on_eci')) || 0;
        state.branch_office_us = !!fd.get('branch_office_us');
        state.permanent_establishment = !!fd.get('permanent_establishment');
        state.treaty_country = fd.get('treaty_country');
        state.treaty_pe_protected = !!fd.get('treaty_pe_protected');
        state.real_property_election = !!fd.get('real_property_election');
        state.is_qualified_subsidiary = !!fd.get('is_qualified_subsidiary');
        state.consolidated_with_us_parent = !!fd.get('consolidated_with_us_parent');
        state.filed_form_1120f = !!fd.get('filed_form_1120f');
        state.days_late_filing = Number(fd.get('days_late_filing')) || 0;
        state.estimated_tax_paid = Number(fd.get('estimated_tax_paid')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s882-output');
    if (!el) return;
    const tax_pe_required = state.treaty_pe_protected && !state.permanent_establishment;
    const eci_taxable = tax_pe_required ? 0 : state.eci_gross_income;
    const deductions_lost = !state.filed_form_1120f || state.days_late_filing > 547;
    const allowed_deductions = deductions_lost ? 0 : state.deductions_allowed;
    const net_eci = Math.max(0, eci_taxable - allowed_deductions);
    const eci_tax = net_eci * 0.21;
    const ftc = Math.min(state.foreign_tax_paid_on_eci, eci_tax);
    const fdap_tax = state.fdap_gross_income * 0.30;
    const final_tax = Math.max(0, eci_tax - ftc) + fdap_tax;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s882.h2.result">§ 882 computation</h2>
            <div class="cards">
                <div class="card ${tax_pe_required ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s882.card.pe">Treaty PE protected?</div>
                    <div class="value">${tax_pe_required ? esc(t('view.s882.status.yes')) : esc(t('view.s882.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s882.card.eci">ECI taxable income</div>
                    <div class="value">$${eci_taxable.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${deductions_lost ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s882.card.deductions_lost">Deductions LOST (late)?</div>
                    <div class="value">${deductions_lost ? esc(t('view.s882.status.yes')) : esc(t('view.s882.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s882.card.net">Net ECI</div>
                    <div class="value">$${net_eci.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s882.card.tax">ECI tax (21%)</div>
                    <div class="value">$${eci_tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s882.card.ftc">Foreign tax credit</div>
                    <div class="value">$${ftc.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s882.card.fdap">FDAP tax (30%)</div>
                    <div class="value">$${fdap_tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s882.card.total">Total US tax</div>
                    <div class="value">$${final_tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${deductions_lost ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s882.deductions_lost_note">
                    Deductions DENIED under § 882(c) — Form 1120-F not filed within 18 months. Taxed on
                    GROSS ECI without any deductions. CRITICAL: file Form 1120-F (or protective Pro-forma)
                    within 18 months even if uncertain about ECI. Reasonable cause exception very narrow.
                </p>
            ` : ''}
        </div>
    `;
}
