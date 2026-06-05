// IRC § 6045 — Broker Information Reporting (Form 1099-B + Basis).
// Broker must report sales of securities + basis information to IRS + customer.
// Covered securities (since 2011): equities, mutual funds, options, futures, debt instruments.
// Basis methods: FIFO (default), LIFO, specific identification, average cost (mutual funds).
// Wash sale + dividend reinvestment + corporate action adjustments included.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    security_type: 'equity',
    is_covered_security: true,
    acquisition_year: 2020,
    sale_year: 2024,
    purchase_price: 0,
    sale_proceeds: 0,
    basis_method: 'fifo',
    wash_sale_adjustment: 0,
    corporate_action_adjustment: 0,
    ordinary_loss_adjustment: 0,
    accrued_market_discount: 0,
    is_short_term: false,
    is_market_discount_election: false,
    broker_w_2_provided: true,
    section_1256_contract: false,
    box_b_1099_b_basis_not_reported: false,
};

export async function renderSection6045(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6045.h1.title">// § 6045 BROKER REPORTING</span></h1>
        <p class="muted small" data-i18n="view.s6045.hint.intro">
            <strong>Broker must report</strong> sales of securities + BASIS information to IRS + customer
            via <strong>Form 1099-B</strong>. <strong>Covered securities (since 2011):</strong> equities,
            mutual funds, options, futures, debt instruments. <strong>Basis methods:</strong> FIFO
            (default), LIFO, specific identification, average cost (mutual funds). <strong>Wash sale</strong>
            + <strong>dividend reinvestment</strong> + <strong>corporate action</strong> adjustments included.
            <strong>Box B:</strong> basis NOT reported to IRS (legacy / uncovered). <strong>§ 6045(g):</strong>
            digital asset broker reporting (post-2023 Form 1099-DA).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6045.h2.inputs">Inputs</h2>
            <form id="s6045-form" class="inline-form">
                <label><span data-i18n="view.s6045.label.type">Security type</span>
                    <select name="security_type">
                        <option value="equity" ${state.security_type === 'equity' ? 'selected' : ''}>Equity / stock</option>
                        <option value="mutual_fund" ${state.security_type === 'mutual_fund' ? 'selected' : ''}>Mutual fund</option>
                        <option value="etf" ${state.security_type === 'etf' ? 'selected' : ''}>ETF</option>
                        <option value="bond" ${state.security_type === 'bond' ? 'selected' : ''}>Bond / debt instrument</option>
                        <option value="option" ${state.security_type === 'option' ? 'selected' : ''}>Option</option>
                        <option value="future" ${state.security_type === 'future' ? 'selected' : ''}>Future (§ 1256)</option>
                        <option value="digital" ${state.security_type === 'digital' ? 'selected' : ''}>Digital asset (1099-DA)</option>
                        <option value="commodity" ${state.security_type === 'commodity' ? 'selected' : ''}>Commodity</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6045.label.covered">Covered security?</span>
                    <input type="checkbox" name="is_covered_security" ${state.is_covered_security ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6045.label.acq_year">Acquisition year</span>
                    <input type="number" step="1" name="acquisition_year" value="${state.acquisition_year}"></label>
                <label><span data-i18n="view.s6045.label.sale_year">Sale year</span>
                    <input type="number" step="1" name="sale_year" value="${state.sale_year}"></label>
                <label><span data-i18n="view.s6045.label.purchase">Purchase price ($)</span>
                    <input type="number" step="0.01" name="purchase_price" value="${state.purchase_price}"></label>
                <label><span data-i18n="view.s6045.label.sale">Sale proceeds ($)</span>
                    <input type="number" step="0.01" name="sale_proceeds" value="${state.sale_proceeds}"></label>
                <label><span data-i18n="view.s6045.label.method">Basis method</span>
                    <select name="basis_method">
                        <option value="fifo" ${state.basis_method === 'fifo' ? 'selected' : ''}>FIFO (default)</option>
                        <option value="lifo" ${state.basis_method === 'lifo' ? 'selected' : ''}>LIFO</option>
                        <option value="specific_id" ${state.basis_method === 'specific_id' ? 'selected' : ''}>Specific identification</option>
                        <option value="average_cost" ${state.basis_method === 'average_cost' ? 'selected' : ''}>Average cost (mutual funds)</option>
                        <option value="hifo" ${state.basis_method === 'hifo' ? 'selected' : ''}>HIFO (highest in, first out)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6045.label.wash">Wash sale adjustment ($)</span>
                    <input type="number" step="0.01" name="wash_sale_adjustment" value="${state.wash_sale_adjustment}"></label>
                <label><span data-i18n="view.s6045.label.corp_action">Corp action adjustment ($)</span>
                    <input type="number" step="0.01" name="corporate_action_adjustment" value="${state.corporate_action_adjustment}"></label>
                <label><span data-i18n="view.s6045.label.ordinary">Ordinary loss adjustment ($)</span>
                    <input type="number" step="0.01" name="ordinary_loss_adjustment" value="${state.ordinary_loss_adjustment}"></label>
                <label><span data-i18n="view.s6045.label.amd">Accrued market discount ($)</span>
                    <input type="number" step="0.01" name="accrued_market_discount" value="${state.accrued_market_discount}"></label>
                <label><span data-i18n="view.s6045.label.short">Short-term holding (≤ 1 yr)?</span>
                    <input type="checkbox" name="is_short_term" ${state.is_short_term ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6045.label.md_election">§ 1278(b) market discount accrual election?</span>
                    <input type="checkbox" name="is_market_discount_election" ${state.is_market_discount_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6045.label.broker_w2">Broker provided W-2?</span>
                    <input type="checkbox" name="broker_w_2_provided" ${state.broker_w_2_provided ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6045.label.1256">§ 1256 contract (60/40)?</span>
                    <input type="checkbox" name="section_1256_contract" ${state.section_1256_contract ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6045.label.box_b">Box B 1099-B (basis not reported)?</span>
                    <input type="checkbox" name="box_b_1099_b_basis_not_reported" ${state.box_b_1099_b_basis_not_reported ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s6045.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6045-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6045.h2.covered_phases">Covered securities phase-in</h2>
            <ul class="muted small">
                <li data-i18n="view.s6045.cov.equity_2011">Equities: acquired AFTER 1/1/2011 — covered</li>
                <li data-i18n="view.s6045.cov.mutual_fund_2012">Mutual funds + ETFs: acquired AFTER 1/1/2012 — covered</li>
                <li data-i18n="view.s6045.cov.bonds_2014">Less complex bonds: acquired AFTER 1/1/2014 — covered</li>
                <li data-i18n="view.s6045.cov.complex_2016">Complex debt + options: acquired AFTER 1/1/2016 — covered</li>
                <li data-i18n="view.s6045.cov.digital_2025">Digital assets: brokers report effective 2025 (Form 1099-DA + Form 1099-DA-G)</li>
                <li data-i18n="view.s6045.cov.uncovered">Uncovered: pre-effective-date acquisitions OR transferred from non-broker</li>
                <li data-i18n="view.s6045.cov.box_a">Box A: short-term basis reported to IRS</li>
                <li data-i18n="view.s6045.cov.box_d">Box D: long-term basis reported to IRS</li>
                <li data-i18n="view.s6045.cov.box_b">Box B: short-term, basis NOT reported (taxpayer responsibility)</li>
                <li data-i18n="view.s6045.cov.box_e">Box E: long-term, basis NOT reported</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6045.h2.basis_methods">Basis methods + lot selection</h2>
            <ul class="muted small">
                <li data-i18n="view.s6045.bm.fifo_default">FIFO: oldest lots sold first — DEFAULT for stocks if no election</li>
                <li data-i18n="view.s6045.bm.lifo">LIFO: newest lots first (uncommon for equities, common bonds)</li>
                <li data-i18n="view.s6045.bm.specific_id">Specific identification: choose specific lot — REQUIRES BROKER CONFIRMATION at sale (settlement date)</li>
                <li data-i18n="view.s6045.bm.average_cost">Average cost: mutual funds only — historical default; locked once elected</li>
                <li data-i18n="view.s6045.bm.hifo">HIFO: tax-loss harvesting (highest basis first to maximize loss / minimize gain)</li>
                <li data-i18n="view.s6045.bm.tax_lot_optimization">Tax lot optimization: most brokers offer auto-HIFO / "max tax loss" mode</li>
                <li data-i18n="view.s6045.bm.dividend_reinvestment">Dividend reinvestment: each share treated as separate lot</li>
                <li data-i18n="view.s6045.bm.election_method">Election made AT or BEFORE sale (per share lot) — broker confirms in writing</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6045.h2.wash_sale">Wash sale (§ 1091) interaction</h2>
            <ul class="muted small">
                <li data-i18n="view.s6045.ws.30_days">Wash sale: buy "substantially identical" within 30 days before / after sale at loss</li>
                <li data-i18n="view.s6045.ws.disallowed">Loss DISALLOWED + ADDED to basis of replacement shares</li>
                <li data-i18n="view.s6045.ws.holding_period">Holding period of replacement shares TACKS from disallowed sale</li>
                <li data-i18n="view.s6045.ws.same_broker_account">Within same account only — IRS Rev. Rul. 2008-5 (across accounts also)</li>
                <li data-i18n="view.s6045.ws.spouse_iras">Cross-account: IRA / 401(k) / spouse account also can trigger</li>
                <li data-i18n="view.s6045.ws.crypto">Crypto: NO wash sale rule currently (proposed legislation)</li>
                <li data-i18n="view.s6045.ws.options_replacement">Option grant to buy same security: triggers wash sale</li>
                <li data-i18n="view.s6045.ws.short_sales">Short sale closure triggers similar 30-day rule (Rev. Rul. 60-159)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6045.h2.reconciliation">Form 8949 + Schedule D reconciliation</h2>
            <ul class="muted small">
                <li data-i18n="view.s6045.recon.f8949">Form 8949: list each sale + adjustment</li>
                <li data-i18n="view.s6045.recon.code">Code column: adjustments codes (W wash sale, B basis adj, T term adj, etc.)</li>
                <li data-i18n="view.s6045.recon.schedule_d">Schedule D: net gain / loss from Form 8949</li>
                <li data-i18n="view.s6045.recon.basis_difference">Basis difference from 1099-B: report adjustments + explanation</li>
                <li data-i18n="view.s6045.recon.cross_reference">Cross-reference: 1099-B box totals to Schedule D line totals</li>
                <li data-i18n="view.s6045.recon.broker_software">Tax software auto-imports 1099-B + applies adjustments</li>
                <li data-i18n="view.s6045.recon.estate_step_up">Inherited: § 1014 step-up to date-of-death FMV (often higher basis)</li>
                <li data-i18n="view.s6045.recon.gift">Gift: § 1015 carryover basis + dual basis rule</li>
            </ul>
        </div>
    `;
    document.getElementById('s6045-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.security_type = fd.get('security_type');
        state.is_covered_security = !!fd.get('is_covered_security');
        state.acquisition_year = Number(fd.get('acquisition_year')) || 0;
        state.sale_year = Number(fd.get('sale_year')) || 0;
        state.purchase_price = Number(fd.get('purchase_price')) || 0;
        state.sale_proceeds = Number(fd.get('sale_proceeds')) || 0;
        state.basis_method = fd.get('basis_method');
        state.wash_sale_adjustment = Number(fd.get('wash_sale_adjustment')) || 0;
        state.corporate_action_adjustment = Number(fd.get('corporate_action_adjustment')) || 0;
        state.ordinary_loss_adjustment = Number(fd.get('ordinary_loss_adjustment')) || 0;
        state.accrued_market_discount = Number(fd.get('accrued_market_discount')) || 0;
        state.is_short_term = !!fd.get('is_short_term');
        state.is_market_discount_election = !!fd.get('is_market_discount_election');
        state.broker_w_2_provided = !!fd.get('broker_w_2_provided');
        state.section_1256_contract = !!fd.get('section_1256_contract');
        state.box_b_1099_b_basis_not_reported = !!fd.get('box_b_1099_b_basis_not_reported');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6045-output');
    if (!el) return;
    const adjusted_basis = state.purchase_price + state.wash_sale_adjustment + state.corporate_action_adjustment;
    const gain_loss = state.sale_proceeds - adjusted_basis;
    const ordinary_portion = Math.min(Math.max(0, gain_loss), state.accrued_market_discount);
    const capital_gain = gain_loss - ordinary_portion;
    let capital_rate = 0.20;
    let character = state.is_short_term ? 'short_term' : 'long_term';
    if (state.section_1256_contract) {
        capital_rate = 0.60 * 0.20 + 0.40 * 0.37;
        character = 's1256_60_40';
    }
    const capital_tax = capital_gain > 0 ? capital_gain * capital_rate : capital_gain * 0.37;
    const ordinary_tax = ordinary_portion * 0.37;
    const total_tax = capital_tax + ordinary_tax;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6045.h2.result">§ 6045 computation</h2>
            <div class="cards">
                <div class="card ${state.is_covered_security ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s6045.card.covered">Covered security?</div>
                    <div class="value">${state.is_covered_security ? esc(t('view.s6045.status.yes')) : esc(t('view.s6045.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6045.card.adj_basis">Adjusted basis</div>
                    <div class="value">$${adjusted_basis.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${gain_loss > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s6045.card.gain_loss">Gain / Loss</div>
                    <div class="value">$${gain_loss.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6045.card.character">Character</div>
                    <div class="value">${esc(t('view.s6045.char.' + character))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6045.card.ordinary">Ordinary portion (MD)</div>
                    <div class="value">$${ordinary_portion.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6045.card.capital_rate">Capital rate</div>
                    <div class="value">${(capital_rate * 100).toFixed(1)}%</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6045.card.tax">Total tax</div>
                    <div class="value">$${total_tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.box_b_1099_b_basis_not_reported ? `
                <p class="muted small" style="margin-top:10px" data-i18n="view.s6045.box_b_note">
                    Box B (basis NOT reported to IRS): taxpayer must independently track + report basis.
                    Common for: pre-2011 stock acquisitions, ESPP / RSU shares (often broker basis incorrect),
                    inherited / gifted shares, transfers between brokers. Maintain personal records — IRS
                    may audit if numbers diverge from broker reporting.
                </p>
            ` : ''}
        </div>
    `;
}
