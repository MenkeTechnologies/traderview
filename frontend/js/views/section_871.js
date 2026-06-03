// IRC § 871 — Tax on Nonresident Alien Individuals.
// § 871(a) — 30% gross tax on US-source FDAP income (interest, dividends, royalties, etc.).
// § 871(b) — graduated tax on ECI (income effectively connected with US trade or business).
// § 871(m) — dividend equivalents on equity-linked instruments.

import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    nra_country: 'India',
    has_us_taxpayer_id: false,
    days_present_year: 0,
    days_present_3yr_substantial: 0,
    is_substantial_presence: false,
    is_green_card_holder: false,
    is_nra_for_year: true,
    has_treaty_benefit: false,
    treaty_dividend_rate: 30,
    treaty_interest_rate: 30,
    treaty_royalty_rate: 30,
    treaty_country: 'none',
    us_dividends_received: 0,
    us_interest_received: 0,
    us_royalties_received: 0,
    us_capital_gain: 0,
    us_real_estate_gain: 0,
    s897_firpta: false,
    us_compensation: 0,
    eci_amount: 0,
    is_personal_services: false,
    days_us_for_dependent_services: 0,
    fixed_base_us: false,
    has_us_branch: false,
    portfolio_interest_exemption: 0,
    bank_deposit_interest: 0,
    s871m_dividend_equivalent: 0,
    s871m_delta_threshold: false,
    short_term_capital_gain: 0,
    us_trade_business: false,
    s7701b_first_year_election: false,
    closer_connection_country: false,
    days_us_4yr: 0,
};

export async function renderSection871(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s871.h1.title">// § 871 NONRESIDENT ALIEN TAX</span></h1>
        <p class="muted small" data-i18n="view.s871.hint.intro">
            <strong>§ 871(a):</strong> 30% gross flat tax on US-source FDAP (fixed/determinable/annual/
            periodic) income — dividends, interest, royalties, rents (passive). <strong>§ 871(b):</strong>
            graduated tax on ECI (effectively connected income) from US trade or business — net basis,
            ordinary brackets. <strong>§ 871(m) (post-2010):</strong> dividend equivalents on equity-
            linked instruments (NPCs, swaps, options) — treats notional dividends as US-source.
            <strong>Substantial presence test:</strong> 183-day weighted (current × 1 + Y-1 × 1/3 +
            Y-2 × 1/6) for treating as US resident. <strong>Treaty rates</strong> override default 30%.
            <strong>§ 897 FIRPTA</strong> — US real property gain treated as ECI regardless. <strong>Capital
            gains</strong> on stocks/securities: NOT taxable to NRA UNLESS US present 183+ days
            (§ 871(a)(2)) OR ECI (§ 871(b)).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s871.h2.inputs">Inputs</h2>
            <form id="s871-form" class="inline-form">
                <label><span data-i18n="view.s871.label.country">Country</span>
                    <input type="text" name="nra_country" value="${esc(state.nra_country)}"></label>
                <label><span data-i18n="view.s871.label.tid">US TID?</span>
                    <input type="checkbox" name="has_us_taxpayer_id" ${state.has_us_taxpayer_id ? 'checked' : ''}></label>
                <label><span data-i18n="view.s871.label.days_year">Days in US</span>
                    <input type="number" step="1" name="days_present_year" value="${state.days_present_year}"></label>
                <label><span data-i18n="view.s871.label.days_3yr">3-yr substantial weighted</span>
                    <input type="number" step="1" name="days_present_3yr_substantial" value="${state.days_present_3yr_substantial}"></label>
                <label><span data-i18n="view.s871.label.substantial">Substantial presence?</span>
                    <input type="checkbox" name="is_substantial_presence" ${state.is_substantial_presence ? 'checked' : ''}></label>
                <label><span data-i18n="view.s871.label.gc">Green card?</span>
                    <input type="checkbox" name="is_green_card_holder" ${state.is_green_card_holder ? 'checked' : ''}></label>
                <label><span data-i18n="view.s871.label.nra">NRA for year?</span>
                    <input type="checkbox" name="is_nra_for_year" ${state.is_nra_for_year ? 'checked' : ''}></label>
                <label><span data-i18n="view.s871.label.treaty">Treaty benefit?</span>
                    <input type="checkbox" name="has_treaty_benefit" ${state.has_treaty_benefit ? 'checked' : ''}></label>
                <label><span data-i18n="view.s871.label.div_rate">Dividend treaty %</span>
                    <input type="number" step="0.1" name="treaty_dividend_rate" value="${state.treaty_dividend_rate}"></label>
                <label><span data-i18n="view.s871.label.int_rate">Interest treaty %</span>
                    <input type="number" step="0.1" name="treaty_interest_rate" value="${state.treaty_interest_rate}"></label>
                <label><span data-i18n="view.s871.label.roy_rate">Royalty treaty %</span>
                    <input type="number" step="0.1" name="treaty_royalty_rate" value="${state.treaty_royalty_rate}"></label>
                <label><span data-i18n="view.s871.label.treaty_country">Treaty country</span>
                    <input type="text" name="treaty_country" value="${esc(state.treaty_country)}"></label>
                <label><span data-i18n="view.s871.label.div">US dividends ($)</span>
                    <input type="number" step="1000" name="us_dividends_received" value="${state.us_dividends_received}"></label>
                <label><span data-i18n="view.s871.label.int">US interest ($)</span>
                    <input type="number" step="1000" name="us_interest_received" value="${state.us_interest_received}"></label>
                <label><span data-i18n="view.s871.label.roy">US royalties ($)</span>
                    <input type="number" step="1000" name="us_royalties_received" value="${state.us_royalties_received}"></label>
                <label><span data-i18n="view.s871.label.cap_gain">US capital gain ($)</span>
                    <input type="number" step="1000" name="us_capital_gain" value="${state.us_capital_gain}"></label>
                <label><span data-i18n="view.s871.label.real_estate">US real estate gain ($)</span>
                    <input type="number" step="1000" name="us_real_estate_gain" value="${state.us_real_estate_gain}"></label>
                <label><span data-i18n="view.s871.label.firpta">§ 897 FIRPTA?</span>
                    <input type="checkbox" name="s897_firpta" ${state.s897_firpta ? 'checked' : ''}></label>
                <label><span data-i18n="view.s871.label.comp">US compensation ($)</span>
                    <input type="number" step="1000" name="us_compensation" value="${state.us_compensation}"></label>
                <label><span data-i18n="view.s871.label.eci">ECI ($)</span>
                    <input type="number" step="1000" name="eci_amount" value="${state.eci_amount}"></label>
                <label><span data-i18n="view.s871.label.services">Personal services?</span>
                    <input type="checkbox" name="is_personal_services" ${state.is_personal_services ? 'checked' : ''}></label>
                <label><span data-i18n="view.s871.label.dep_services">Days for dep services</span>
                    <input type="number" step="1" name="days_us_for_dependent_services" value="${state.days_us_for_dependent_services}"></label>
                <label><span data-i18n="view.s871.label.fixed_base">Fixed base US?</span>
                    <input type="checkbox" name="fixed_base_us" ${state.fixed_base_us ? 'checked' : ''}></label>
                <label><span data-i18n="view.s871.label.branch">US branch?</span>
                    <input type="checkbox" name="has_us_branch" ${state.has_us_branch ? 'checked' : ''}></label>
                <label><span data-i18n="view.s871.label.portfolio">Portfolio interest exempt ($)</span>
                    <input type="number" step="1000" name="portfolio_interest_exemption" value="${state.portfolio_interest_exemption}"></label>
                <label><span data-i18n="view.s871.label.bank">Bank deposit interest ($)</span>
                    <input type="number" step="1000" name="bank_deposit_interest" value="${state.bank_deposit_interest}"></label>
                <label><span data-i18n="view.s871.label.871m">§ 871(m) div equiv ($)</span>
                    <input type="number" step="1000" name="s871m_dividend_equivalent" value="${state.s871m_dividend_equivalent}"></label>
                <label><span data-i18n="view.s871.label.delta">§ 871(m) delta ≥ 0.8?</span>
                    <input type="checkbox" name="s871m_delta_threshold" ${state.s871m_delta_threshold ? 'checked' : ''}></label>
                <label><span data-i18n="view.s871.label.st_cap">S/T capital gain ($)</span>
                    <input type="number" step="1000" name="short_term_capital_gain" value="${state.short_term_capital_gain}"></label>
                <label><span data-i18n="view.s871.label.tb">US trade/business?</span>
                    <input type="checkbox" name="us_trade_business" ${state.us_trade_business ? 'checked' : ''}></label>
                <label><span data-i18n="view.s871.label.s7701b">§ 7701(b) first-yr election?</span>
                    <input type="checkbox" name="s7701b_first_year_election" ${state.s7701b_first_year_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s871.label.closer">Closer connection?</span>
                    <input type="checkbox" name="closer_connection_country" ${state.closer_connection_country ? 'checked' : ''}></label>
                <label><span data-i18n="view.s871.label.days_4yr">Days US 4-yr</span>
                    <input type="number" step="1" name="days_us_4yr" value="${state.days_us_4yr}"></label>
                <button class="primary" type="submit" data-i18n="view.s871.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s871-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s871.h2.fdap">FDAP income (§ 871(a))</h2>
            <ul class="muted small">
                <li data-i18n="view.s871.fdap.rate">30% gross flat tax (no deductions)</li>
                <li data-i18n="view.s871.fdap.dividends">US-source dividends from US corporations</li>
                <li data-i18n="view.s871.fdap.interest">US-source interest (with major exceptions)</li>
                <li data-i18n="view.s871.fdap.royalties">US-source royalties (patents, trademarks, copyright)</li>
                <li data-i18n="view.s871.fdap.rents">Real property rents (no net election)</li>
                <li data-i18n="view.s871.fdap.gain_securities">Gain on US securities IF 183+ days physical presence § 871(a)(2)</li>
                <li data-i18n="view.s871.fdap.no_se">No social security or Medicare tax (NRA)</li>
                <li data-i18n="view.s871.fdap.withheld">Withheld at source by payor (Form 1042 / 1042-S reporting)</li>
                <li data-i18n="view.s871.fdap.treaty">Treaty rates override 30% (typically 5/10/15%)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s871.h2.exemptions">Major exemptions from 30%</h2>
            <ul class="muted small">
                <li data-i18n="view.s871.ex.portfolio">§ 871(h) Portfolio interest exemption: bonds issued post-July 1984 to non-bank NRA</li>
                <li data-i18n="view.s871.ex.bank">§ 871(i)(2) Bank deposit interest: US bank deposits exempt (most countries)</li>
                <li data-i18n="view.s871.ex.os_securities">§ 871(k) Original issue discount on short-term (≤ 183 days) obligations</li>
                <li data-i18n="view.s871.ex.s7701l">§ 7701(l) Conduit arrangements: anti-abuse</li>
                <li data-i18n="view.s871.ex.short_term_gain">Short-term securities gains (NOT covered by 30% unless 183+ days)</li>
                <li data-i18n="view.s871.ex.s897">FIRPTA real estate: § 897(a) treats as ECI regardless</li>
                <li data-i18n="view.s871.ex.s165_g_3">§ 165(g)(3) worthless stock: NOT excluded</li>
                <li data-i18n="view.s871.ex.scholarship">Scholarship + grant from US source: § 117 partial exclusion</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s871.h2.eci">ECI (Effectively Connected Income) — § 871(b)</h2>
            <ul class="muted small">
                <li data-i18n="view.s871.eci.basis">Net basis taxation — deductions allowed (Form 1040-NR)</li>
                <li data-i18n="view.s871.eci.brackets">Graduated brackets (10%-37%) — same as US persons</li>
                <li data-i18n="view.s871.eci.us_tb">Requires US trade or business (§ 864(b))</li>
                <li data-i18n="view.s871.eci.s_864c">§ 864(c) attribution: income "effectively connected" with US TB</li>
                <li data-i18n="view.s871.eci.personal_services">Personal services in US = US TB (§ 864(b))</li>
                <li data-i18n="view.s871.eci.merchandising">Merchandising activity in US = US TB</li>
                <li data-i18n="view.s871.eci.dealer">Trading securities for OWN account by dealer = US TB</li>
                <li data-i18n="view.s871.eci.investor">Trading securities by investor (not dealer) NOT US TB (Reg § 1.864-2(c)(2))</li>
                <li data-i18n="view.s871.eci.s897_real">§ 897(a) US real property gain → automatic ECI treatment</li>
                <li data-i18n="view.s871.eci.s871d">§ 871(d) election to treat real property rents as ECI (net basis)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s871.h2.871m">§ 871(m) dividend equivalents</h2>
            <ul class="muted small">
                <li data-i18n="view.s871.m.purpose">Anti-avoidance: treats notional dividends in equity derivatives as US-source dividends</li>
                <li data-i18n="view.s871.m.delta">Delta-1 instruments (futures, options w/ delta ≥ 0.8): subject</li>
                <li data-i18n="view.s871.m.simple_contracts">Simple contracts: tested at issuance</li>
                <li data-i18n="view.s871.m.complex_contracts">Complex contracts (variable rate, multi-leg): tested under "substantial equivalence"</li>
                <li data-i18n="view.s871.m.30pct">30% withholding (or treaty rate) on notional dividend portion</li>
                <li data-i18n="view.s871.m.notional">Notional principal amount × actual underlying dividend</li>
                <li data-i18n="view.s871.m.qualified_index">QDD (qualified derivatives dealer) safe harbor: post-2017 limited relief</li>
                <li data-i18n="view.s871.m.no_treaty_relief">Substitute payments NOT eligible for treaty benefits (most treaties)</li>
                <li data-i18n="view.s871.m.s959_p3">§ 959(p)(3) PTI exception for CFC-paid dividends</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s871.h2.substantial">Substantial presence test (§ 7701(b))</h2>
            <ol class="muted small">
                <li data-i18n="view.s871.sp.formula">Current year days × 1 + Y-1 × 1/3 + Y-2 × 1/6 ≥ 183</li>
                <li data-i18n="view.s871.sp.minimum">Minimum 31 days in current year</li>
                <li data-i18n="view.s871.sp.closer_connection">§ 7701(b)(3)(B) closer connection exception: &lt; 183 days current + tax home in foreign country</li>
                <li data-i18n="view.s871.sp.f8840">Form 8840 closer connection statement</li>
                <li data-i18n="view.s871.sp.medical_exception">Medical condition exception: emergency medical treatment</li>
                <li data-i18n="view.s871.sp.canada_mexico">Canada / Mexico daily commuters: excluded</li>
                <li data-i18n="view.s871.sp.diplomat">Diplomats, teachers, students: limited exclusions</li>
                <li data-i18n="view.s871.sp.first_year">§ 7701(b)(4) first-year election: split-year treatment</li>
                <li data-i18n="view.s871.sp.tax_treaty">Treaty tie-breaker can override substantial presence (Form 8833)</li>
            </ol>
        </div>
    `;
    document.getElementById('s871-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.nra_country = fd.get('nra_country') || '';
        state.has_us_taxpayer_id = !!fd.get('has_us_taxpayer_id');
        state.days_present_year = Number(fd.get('days_present_year')) || 0;
        state.days_present_3yr_substantial = Number(fd.get('days_present_3yr_substantial')) || 0;
        state.is_substantial_presence = !!fd.get('is_substantial_presence');
        state.is_green_card_holder = !!fd.get('is_green_card_holder');
        state.is_nra_for_year = !!fd.get('is_nra_for_year');
        state.has_treaty_benefit = !!fd.get('has_treaty_benefit');
        state.treaty_dividend_rate = Number(fd.get('treaty_dividend_rate')) || 0;
        state.treaty_interest_rate = Number(fd.get('treaty_interest_rate')) || 0;
        state.treaty_royalty_rate = Number(fd.get('treaty_royalty_rate')) || 0;
        state.treaty_country = fd.get('treaty_country') || '';
        state.us_dividends_received = Number(fd.get('us_dividends_received')) || 0;
        state.us_interest_received = Number(fd.get('us_interest_received')) || 0;
        state.us_royalties_received = Number(fd.get('us_royalties_received')) || 0;
        state.us_capital_gain = Number(fd.get('us_capital_gain')) || 0;
        state.us_real_estate_gain = Number(fd.get('us_real_estate_gain')) || 0;
        state.s897_firpta = !!fd.get('s897_firpta');
        state.us_compensation = Number(fd.get('us_compensation')) || 0;
        state.eci_amount = Number(fd.get('eci_amount')) || 0;
        state.is_personal_services = !!fd.get('is_personal_services');
        state.days_us_for_dependent_services = Number(fd.get('days_us_for_dependent_services')) || 0;
        state.fixed_base_us = !!fd.get('fixed_base_us');
        state.has_us_branch = !!fd.get('has_us_branch');
        state.portfolio_interest_exemption = Number(fd.get('portfolio_interest_exemption')) || 0;
        state.bank_deposit_interest = Number(fd.get('bank_deposit_interest')) || 0;
        state.s871m_dividend_equivalent = Number(fd.get('s871m_dividend_equivalent')) || 0;
        state.s871m_delta_threshold = !!fd.get('s871m_delta_threshold');
        state.short_term_capital_gain = Number(fd.get('short_term_capital_gain')) || 0;
        state.us_trade_business = !!fd.get('us_trade_business');
        state.s7701b_first_year_election = !!fd.get('s7701b_first_year_election');
        state.closer_connection_country = !!fd.get('closer_connection_country');
        state.days_us_4yr = Number(fd.get('days_us_4yr')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s871-output');
    if (!el) return;
    const div_rate = state.has_treaty_benefit ? state.treaty_dividend_rate : 30;
    const int_rate = state.has_treaty_benefit ? state.treaty_interest_rate : 30;
    const roy_rate = state.has_treaty_benefit ? state.treaty_royalty_rate : 30;
    const tax_dividends = state.us_dividends_received * (div_rate / 100);
    const tax_interest = Math.max(0, state.us_interest_received - state.portfolio_interest_exemption - state.bank_deposit_interest) * (int_rate / 100);
    const tax_royalties = state.us_royalties_received * (roy_rate / 100);
    const tax_871m = state.s871m_dividend_equivalent * (div_rate / 100);
    const total_fdap_tax = tax_dividends + tax_interest + tax_royalties + tax_871m;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s871.h2.result">§ 871 NRA tax</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s871.card.div">Dividend tax</div><div class="value">$${tax_dividends.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div></div>
                <div class="card"><div class="label" data-i18n="view.s871.card.int">Interest tax</div><div class="value">$${tax_interest.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div></div>
                <div class="card"><div class="label" data-i18n="view.s871.card.roy">Royalty tax</div><div class="value">$${tax_royalties.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div></div>
                <div class="card"><div class="label" data-i18n="view.s871.card.871m">§ 871(m) tax</div><div class="value">$${tax_871m.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.s871.card.total">Total § 871(a) FDAP tax</div><div class="value">$${total_fdap_tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div></div>
            </div>
        </div>
    `;
}
