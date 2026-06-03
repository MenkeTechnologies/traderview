// IRC § 904 — Foreign Tax Credit Limitation.
// Limit = (foreign source TI / worldwide TI) × US tax (pre-FTC).
// Separate baskets: passive, GILTI, foreign branch, general, treaty resourced.
// Excess: carry back 1 year, forward 10 years (within same basket).
// § 904(g) high-tax kick-out: high-taxed passive income moves to general basket.
// § 904(j) de minimis exception: ≤ $300/$600 MFJ — no Form 1116 required.

import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    foreign_source_taxable_income: 0,
    worldwide_taxable_income: 0,
    us_tax_pre_ftc: 0,
    foreign_taxes_paid: 0,
    basket: 'general',
    high_tax_kickout: false,
    de_minimis_election: false,
    carry_back_1yr: 0,
    carry_forward_avail: 0,
    is_corporate: false,
    section_250_deduction: 0,
    expense_allocated_foreign: 0,
};

export async function renderSection904(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s904.h1.title">// § 904 FTC LIMITATION</span></h1>
        <p class="muted small" data-i18n="view.s904.hint.intro">
            <strong>Limit = (foreign source TI / worldwide TI) × US tax (pre-FTC)</strong>. Separate baskets:
            <strong>passive, GILTI, foreign branch, general, treaty resourced</strong>. <strong>Excess:</strong>
            carry back 1 year, forward 10 years (WITHIN SAME BASKET). <strong>§ 904(d) lookthrough</strong>
            rules: pass-through entity income flows by character. <strong>§ 904(g) high-tax kickout:</strong>
            high-taxed passive income moves to GENERAL basket. <strong>§ 904(j) de minimis:</strong>
            ≤ $300 / $600 MFJ — no Form 1116. <strong>§ 904(b)(4):</strong> US-source income re-sourced
            (Worldwide). <strong>Form 1116 / 1118.</strong>
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s904.h2.inputs">Inputs</h2>
            <form id="s904-form" class="inline-form">
                <label><span data-i18n="view.s904.label.foreign_ti">Foreign source taxable income ($)</span>
                    <input type="number" step="1000" name="foreign_source_taxable_income" value="${state.foreign_source_taxable_income}"></label>
                <label><span data-i18n="view.s904.label.worldwide">Worldwide taxable income ($)</span>
                    <input type="number" step="1000" name="worldwide_taxable_income" value="${state.worldwide_taxable_income}"></label>
                <label><span data-i18n="view.s904.label.us_tax">US tax pre-FTC ($)</span>
                    <input type="number" step="1000" name="us_tax_pre_ftc" value="${state.us_tax_pre_ftc}"></label>
                <label><span data-i18n="view.s904.label.foreign_tax">Foreign taxes paid ($)</span>
                    <input type="number" step="100" name="foreign_taxes_paid" value="${state.foreign_taxes_paid}"></label>
                <label><span data-i18n="view.s904.label.basket">Basket</span>
                    <select name="basket">
                        <option value="general" ${state.basket === 'general' ? 'selected' : ''}>General (active biz)</option>
                        <option value="passive" ${state.basket === 'passive' ? 'selected' : ''}>Passive (interest, div)</option>
                        <option value="gilti" ${state.basket === 'gilti' ? 'selected' : ''}>GILTI § 951A (80% haircut)</option>
                        <option value="branch" ${state.basket === 'branch' ? 'selected' : ''}>Foreign branch</option>
                        <option value="treaty" ${state.basket === 'treaty' ? 'selected' : ''}>Treaty resourced</option>
                    </select>
                </label>
                <label><span data-i18n="view.s904.label.high_tax">High-tax kickout (passive→general)?</span>
                    <input type="checkbox" name="high_tax_kickout" ${state.high_tax_kickout ? 'checked' : ''}></label>
                <label><span data-i18n="view.s904.label.de_minimis">§ 904(j) de minimis election?</span>
                    <input type="checkbox" name="de_minimis_election" ${state.de_minimis_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s904.label.carry_back">Carry back 1-yr ($)</span>
                    <input type="number" step="100" name="carry_back_1yr" value="${state.carry_back_1yr}"></label>
                <label><span data-i18n="view.s904.label.carry_forward">Carry forward available ($)</span>
                    <input type="number" step="100" name="carry_forward_avail" value="${state.carry_forward_avail}"></label>
                <label><span data-i18n="view.s904.label.corporate">C-corp?</span>
                    <input type="checkbox" name="is_corporate" ${state.is_corporate ? 'checked' : ''}></label>
                <label><span data-i18n="view.s904.label.s250">§ 250 deduction (GILTI/FDII) ($)</span>
                    <input type="number" step="1000" name="section_250_deduction" value="${state.section_250_deduction}"></label>
                <label><span data-i18n="view.s904.label.expense">US expenses allocated to foreign ($)</span>
                    <input type="number" step="1000" name="expense_allocated_foreign" value="${state.expense_allocated_foreign}"></label>
                <button class="primary" type="submit" data-i18n="view.s904.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s904-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s904.h2.baskets">§ 904(d) separate baskets</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s904.th.basket">Basket</th>
                    <th data-i18n="view.s904.th.includes">Includes</th>
                    <th data-i18n="view.s904.th.special">Special rules</th>
                </tr></thead>
                <tbody>
                    <tr><td>General</td><td>Active business operations + royalties</td><td>Default</td></tr>
                    <tr><td>Passive (904(d)(2)(B))</td><td>Interest, div, rents, royalties (non-active)</td><td>Lookthrough rules + § 904(g) kickout</td></tr>
                    <tr><td>GILTI § 951A</td><td>Tested income inclusions</td><td>80% foreign tax haircut (§ 960(d))</td></tr>
                    <tr><td>Foreign branch</td><td>QBU (qualified business unit) income</td><td>Separate accounting</td></tr>
                    <tr><td>Treaty resourced</td><td>Income resourced to foreign per treaty</td><td>Treaty-based source rule</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s904.h2.expense_allocation">Expense allocation (§ 861)</h2>
            <ul class="muted small">
                <li data-i18n="view.s904.exp.gross">Foreign-source TI starts with foreign GROSS income</li>
                <li data-i18n="view.s904.exp.allocate">Allocate + apportion US deductions to foreign source basis</li>
                <li data-i18n="view.s904.exp.interest">Interest expense: allocated based on FMV of assets (or tax book)</li>
                <li data-i18n="view.s904.exp.rd">R&D expense: 25% / 50% / 75% to source — taxpayer election</li>
                <li data-i18n="view.s904.exp.sga">SG&A: gross income, asset, sales — taxpayer choice</li>
                <li data-i18n="view.s904.exp.directly_traceable">Directly traceable: full allocation to source (charge-off)</li>
                <li data-i18n="view.s904.exp.cm_election">Cost method election vs FMV — locked for 5 yrs</li>
                <li data-i18n="view.s904.exp.reg_864">Reg § 1.864-4 ECI determination + sourcing</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s904.h2.high_tax_kickout">§ 904(g) high-tax kickout</h2>
            <ul class="muted small">
                <li data-i18n="view.s904.htk.trigger">Trigger: passive income's foreign effective rate &gt; 90% × highest US rate</li>
                <li data-i18n="view.s904.htk.2025">2025 example: 37% × 90% = 33.3% — passive income at higher foreign rate moves to general</li>
                <li data-i18n="view.s904.htk.purpose">Purpose: prevent loading general basket with low-tax passive income</li>
                <li data-i18n="view.s904.htk.kicks_general">Income kicks to GENERAL basket (better cross-crediting)</li>
                <li data-i18n="view.s904.htk.election">2024 final regs: kick-out automatic; no election</li>
                <li data-i18n="view.s904.htk.consistency">Consistency: apply same way each year for similar income</li>
                <li data-i18n="view.s904.htk.partnership">Partnership: pass-through entity's character preserved</li>
                <li data-i18n="view.s904.htk.cfc_overlap">CFC overlap: high-tax exception § 954(b)(4) for subpart F also relevant</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s904.h2.carryover">Carry-back + carry-forward (§ 904(c))</h2>
            <ul class="muted small">
                <li data-i18n="view.s904.cf.back_1">Carry back 1 year first (file Form 1040X / 1120X)</li>
                <li data-i18n="view.s904.cf.forward_10">Then carry forward up to 10 years</li>
                <li data-i18n="view.s904.cf.same_basket">Must stay in SAME basket — no cross-basket use</li>
                <li data-i18n="view.s904.cf.character_preserve">Character preserved — passive carryforward only offsets passive</li>
                <li data-i18n="view.s904.cf.expire">After 10 years: PERMANENTLY LOST</li>
                <li data-i18n="view.s904.cf.s382_overlap">§ 382 ownership change can limit carryforward (analogous to NOL)</li>
                <li data-i18n="view.s904.cf.no_election">No election to opt out of carry-back (forced)</li>
                <li data-i18n="view.s904.cf.short_year">Short-period years: carry-back limited proportionally</li>
            </ul>
        </div>
    `;
    document.getElementById('s904-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.foreign_source_taxable_income = Number(fd.get('foreign_source_taxable_income')) || 0;
        state.worldwide_taxable_income = Number(fd.get('worldwide_taxable_income')) || 0;
        state.us_tax_pre_ftc = Number(fd.get('us_tax_pre_ftc')) || 0;
        state.foreign_taxes_paid = Number(fd.get('foreign_taxes_paid')) || 0;
        state.basket = fd.get('basket');
        state.high_tax_kickout = !!fd.get('high_tax_kickout');
        state.de_minimis_election = !!fd.get('de_minimis_election');
        state.carry_back_1yr = Number(fd.get('carry_back_1yr')) || 0;
        state.carry_forward_avail = Number(fd.get('carry_forward_avail')) || 0;
        state.is_corporate = !!fd.get('is_corporate');
        state.section_250_deduction = Number(fd.get('section_250_deduction')) || 0;
        state.expense_allocated_foreign = Number(fd.get('expense_allocated_foreign')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s904-output');
    if (!el) return;
    const adj_foreign_ti = Math.max(0, state.foreign_source_taxable_income - state.expense_allocated_foreign);
    const ratio = state.worldwide_taxable_income > 0 ? adj_foreign_ti / state.worldwide_taxable_income : 0;
    const limit = ratio * state.us_tax_pre_ftc;
    const haircut = state.basket === 'gilti' ? 0.80 : 1.0;
    const eligibleForeignTax = state.foreign_taxes_paid * haircut;
    const totalAvailable = eligibleForeignTax + state.carry_forward_avail;
    const allowed = Math.min(totalAvailable, limit);
    const excess = Math.max(0, totalAvailable - limit);
    const netUSTax = Math.max(0, state.us_tax_pre_ftc - allowed);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s904.h2.result">§ 904 limit computation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s904.card.adj_ti">Adj foreign TI</div>
                    <div class="value">$${adj_foreign_ti.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s904.card.ratio">Foreign / Worldwide</div>
                    <div class="value">${(ratio * 100).toFixed(2)}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s904.card.limit">§ 904 limitation</div>
                    <div class="value">$${limit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s904.card.haircut">Haircut (GILTI 80%)</div>
                    <div class="value">${(haircut * 100).toFixed(0)}%</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s904.card.allowed">FTC allowed</div>
                    <div class="value">$${allowed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${excess > 0 ? 'neg' : ''}">
                    <div class="label" data-i18n="view.s904.card.excess">Excess (carry 1B / 10F)</div>
                    <div class="value">$${excess.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s904.card.net_us_tax">Net US tax after FTC</div>
                    <div class="value">$${netUSTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.expense_allocated_foreign > 0 ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s904.expense_note">
                    US expenses allocated to foreign source REDUCE the § 904 limit. Reducing expense
                    allocation (consider direct-charge basis vs apportionment) preserves FTC capacity. § 904(b)(4)
                    + 2020 final regs limit reallocation of US-source income. Compare R&D 25% vs 75%
                    elections impact on basket allocation.
                </p>
            ` : ''}
        </div>
    `;
}
