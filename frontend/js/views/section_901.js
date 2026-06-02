// IRC § 901 — Foreign Tax Credit.
// Direct credit for foreign income taxes PAID or ACCRUED to foreign country / US possession.
// Election: credit vs deduction (credit usually better since dollar-for-dollar).
// § 904 limitation: FTC capped at (foreign source TI / worldwide TI) × US tax.
// Separate baskets: passive, GILTI, foreign branch, general, treaty resourced.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    foreign_source_income: 0,
    foreign_tax_paid: 0,
    worldwide_taxable_income: 0,
    us_tax_before_ftc: 0,
    basket: 'general',
    treaty_resourced: false,
    indirect_credit_subF: 0,
    carry_back: 0,
    carry_forward: 0,
};

export async function renderSection901(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s901.h1.title">// § 901 FOREIGN TAX CREDIT</span></h1>
        <p class="muted small" data-i18n="view.s901.hint.intro">
            <strong>Direct credit</strong> for foreign income taxes PAID or ACCRUED. <strong>Election:</strong>
            credit vs deduction — credit usually wins (dollar-for-dollar reduction). <strong>§ 904 limit:</strong>
            FTC ≤ (foreign source TI / worldwide TI) × US tax. <strong>Separate baskets:</strong> passive,
            GILTI, foreign branch, general, treaty resourced. <strong>Excess:</strong> carry back 1 year,
            forward 10 years.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s901.h2.inputs">Inputs</h2>
            <form id="s901-form" class="inline-form">
                <label><span data-i18n="view.s901.label.foreign_income">Foreign source income ($)</span>
                    <input type="number" step="1000" name="foreign_source_income" value="${state.foreign_source_income}"></label>
                <label><span data-i18n="view.s901.label.foreign_tax">Foreign tax paid ($)</span>
                    <input type="number" step="100" name="foreign_tax_paid" value="${state.foreign_tax_paid}"></label>
                <label><span data-i18n="view.s901.label.worldwide">Worldwide taxable income ($)</span>
                    <input type="number" step="1000" name="worldwide_taxable_income" value="${state.worldwide_taxable_income}"></label>
                <label><span data-i18n="view.s901.label.us_tax">US tax pre-FTC ($)</span>
                    <input type="number" step="100" name="us_tax_before_ftc" value="${state.us_tax_before_ftc}"></label>
                <label><span data-i18n="view.s901.label.basket">FTC basket</span>
                    <select name="basket">
                        <option value="general" ${state.basket === 'general' ? 'selected' : ''}>General (active biz)</option>
                        <option value="passive" ${state.basket === 'passive' ? 'selected' : ''}>Passive (interest, div, etc.)</option>
                        <option value="gilti" ${state.basket === 'gilti' ? 'selected' : ''}>GILTI § 951A (80% only)</option>
                        <option value="branch" ${state.basket === 'branch' ? 'selected' : ''}>Foreign branch</option>
                        <option value="treaty" ${state.basket === 'treaty' ? 'selected' : ''}>Treaty resourced</option>
                    </select>
                </label>
                <label><span data-i18n="view.s901.label.treaty">Treaty resourced?</span>
                    <input type="checkbox" name="treaty_resourced" ${state.treaty_resourced ? 'checked' : ''}></label>
                <label><span data-i18n="view.s901.label.indirect">Indirect credit § 960 subF ($)</span>
                    <input type="number" step="100" name="indirect_credit_subF" value="${state.indirect_credit_subF}"></label>
                <label><span data-i18n="view.s901.label.carry_back">Carry back 1-yr available ($)</span>
                    <input type="number" step="100" name="carry_back" value="${state.carry_back}"></label>
                <label><span data-i18n="view.s901.label.carry_forward">Carry forward avail 10-yr ($)</span>
                    <input type="number" step="100" name="carry_forward" value="${state.carry_forward}"></label>
                <button class="primary" type="submit" data-i18n="view.s901.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s901-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s901.h2.baskets">§ 904 separate baskets</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s901.th.basket">Basket</th>
                    <th data-i18n="view.s901.th.includes">Includes</th>
                    <th data-i18n="view.s901.th.haircut">Section 78 gross-up / haircut</th>
                </tr></thead>
                <tbody>
                    <tr><td>General</td><td>Active foreign business operations, royalties, etc.</td><td>—</td></tr>
                    <tr><td>Passive</td><td>Interest, dividends, royalties, rents (non-active)</td><td>—</td></tr>
                    <tr><td>GILTI § 951A</td><td>Tested income inclusions</td><td>20% haircut (80% only)</td></tr>
                    <tr><td>Foreign branch</td><td>Income of qualified business units</td><td>—</td></tr>
                    <tr><td>Treaty resourced</td><td>Income resourced to foreign per treaty</td><td>—</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s901.h2.notes">Practical FTC notes</h2>
            <ul class="muted small">
                <li data-i18n="view.s901.note.election">Annual election: credit OR deduction (not both same year)</li>
                <li data-i18n="view.s901.note.simplified">§ 904(j) simplified election: ≤ $300 / $600 MFJ — no Form 1116 required</li>
                <li data-i18n="view.s901.note.1116">Form 1116 (individual) or 1118 (corp)</li>
                <li data-i18n="view.s901.note.cash_basis">Cash method: foreign tax credit when PAID</li>
                <li data-i18n="view.s901.note.accrual_basis">Accrual method: foreign tax credit when ACCRUED</li>
                <li data-i18n="view.s901.note.economic_benefit">Subsidies / cost-share denied credit (economic benefit rule)</li>
                <li data-i18n="view.s901.note.refund">§ 905(c) redetermination required if foreign tax changes (refund / settlement)</li>
                <li data-i18n="view.s901.note.high_tax">§ 954(b)(4) high-tax exception: GILTI / subF excluded if foreign rate ≥ 90% US rate</li>
                <li data-i18n="view.s901.note.creditable">Test "creditable" tax: net gain on income basis + realization + cost recovery</li>
                <li data-i18n="view.s901.note.2022_regs">2022 final regs tightened "creditable" test — many DST + WHT regimes no longer creditable</li>
            </ul>
        </div>
    `;
    document.getElementById('s901-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.foreign_source_income = Number(fd.get('foreign_source_income')) || 0;
        state.foreign_tax_paid = Number(fd.get('foreign_tax_paid')) || 0;
        state.worldwide_taxable_income = Number(fd.get('worldwide_taxable_income')) || 0;
        state.us_tax_before_ftc = Number(fd.get('us_tax_before_ftc')) || 0;
        state.basket = fd.get('basket');
        state.treaty_resourced = !!fd.get('treaty_resourced');
        state.indirect_credit_subF = Number(fd.get('indirect_credit_subF')) || 0;
        state.carry_back = Number(fd.get('carry_back')) || 0;
        state.carry_forward = Number(fd.get('carry_forward')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s901-output');
    if (!el) return;
    const ratio = state.worldwide_taxable_income > 0 ? state.foreign_source_income / state.worldwide_taxable_income : 0;
    const limitation = ratio * state.us_tax_before_ftc;
    const haircut = state.basket === 'gilti' ? 0.8 : 1.0;
    const creditAfterHaircut = state.foreign_tax_paid * haircut;
    const ftcAllowed = Math.min(creditAfterHaircut, limitation);
    const excess = Math.max(0, creditAfterHaircut - limitation);
    const usTaxAfterFTC = Math.max(0, state.us_tax_before_ftc - ftcAllowed);
    const effectiveRate = state.foreign_source_income > 0 ? (state.foreign_tax_paid / state.foreign_source_income * 100) : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s901.h2.result">FTC computation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s901.card.ratio">Foreign / Worldwide</div>
                    <div class="value">${(ratio * 100).toFixed(2)}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s901.card.foreign_rate">Foreign effective rate</div>
                    <div class="value">${effectiveRate.toFixed(2)}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s901.card.limit">§ 904 limitation</div>
                    <div class="value">$${limitation.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s901.card.ftc_allowed">FTC allowed</div>
                    <div class="value">$${ftcAllowed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${excess > 0 ? 'neg' : ''}">
                    <div class="label" data-i18n="view.s901.card.excess">Excess (carry 1B / 10F)</div>
                    <div class="value">$${excess.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s901.card.net_us_tax">Net US tax after FTC</div>
                    <div class="value">$${usTaxAfterFTC.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.basket === 'gilti' ? `
                <p class="muted small" style="margin-top:10px" data-i18n="view.s901.gilti_note">
                    GILTI basket: only 80% of foreign tax creditable (20% haircut). Combined with § 250 50%
                    GILTI deduction, foreign rate ≥ 13.125% generally fully shields US tax.
                </p>
            ` : ''}
        </div>
    `;
}
