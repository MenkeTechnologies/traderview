// IRC § 367(d) — Outbound Transfer of Intangible Property.
// US transferor of IP to foreign corp must include annual DEEMED ROYALTY over useful life.
// Royalty = arm's-length annual royalty on transferred IP (similar to ongoing license).
// TCJA 2017 expanded "intangible property" definition to include workforce in place, goodwill, going concern.
// Avoid via: cost sharing arrangements (CSA § 482), keep IP in US, royalty structure.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    ip_fmv_at_transfer: 0,
    annual_arm_length_royalty: 0,
    useful_life_years: 0,
    foreign_recipient_corp: 'cfc',
    is_cfc: true,
    cost_sharing_arrangement: false,
    csa_share_costs: 0,
    csa_buy_in_payment: 0,
    transferor_basis: 0,
    tcja_broad_definition: true,
    domestic_filing_corp: true,
    transfer_year: 2024,
    annual_royalty_received: 0,
};

export async function renderSection367D(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s367d.h1.title">// § 367(d) IP OUTBOUND</span></h1>
        <p class="muted small" data-i18n="view.s367d.hint.intro">
            US transferor of <strong>INTANGIBLE PROPERTY</strong> to foreign corp must include <strong>ANNUAL DEEMED
            ROYALTY</strong> over IP's useful life. <strong>TCJA 2017 expansion:</strong> "intangible" now
            includes <strong>workforce in place, goodwill, going concern, customer lists</strong>. Royalty =
            arm's-length annual amount (parallel to § 482 transfer pricing). <strong>Cost Sharing
            Arrangement (CSA) § 482</strong> alternative — buy-in + ongoing R&D shares. <strong>2017 effective:</strong>
            applies post-Dec 31 2017 transfers. <strong>§ 6038B reporting</strong>. Avoid: keep IP in US,
            or CSA, or royalty structure (not outbound transfer).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s367d.h2.inputs">Inputs</h2>
            <form id="s367d-form" class="inline-form">
                <label><span data-i18n="view.s367d.label.fmv">IP FMV at transfer ($)</span>
                    <input type="number" step="0.01" name="ip_fmv_at_transfer" value="${state.ip_fmv_at_transfer}"></label>
                <label><span data-i18n="view.s367d.label.royalty">Annual arm's-length royalty ($)</span>
                    <input type="number" step="0.01" name="annual_arm_length_royalty" value="${state.annual_arm_length_royalty}"></label>
                <label><span data-i18n="view.s367d.label.life">Useful life years</span>
                    <input type="number" step="1" name="useful_life_years" value="${state.useful_life_years}"></label>
                <label><span data-i18n="view.s367d.label.recipient">Foreign recipient corp type</span>
                    <select name="foreign_recipient_corp">
                        <option value="cfc" ${state.foreign_recipient_corp === 'cfc' ? 'selected' : ''}>CFC (Controlled Foreign Corp)</option>
                        <option value="related_foreign" ${state.foreign_recipient_corp === 'related_foreign' ? 'selected' : ''}>Related foreign (not CFC)</option>
                        <option value="unrelated_foreign" ${state.foreign_recipient_corp === 'unrelated_foreign' ? 'selected' : ''}>Unrelated foreign</option>
                        <option value="us_corp" ${state.foreign_recipient_corp === 'us_corp' ? 'selected' : ''}>US corp (no § 367(d))</option>
                    </select>
                </label>
                <label><span data-i18n="view.s367d.label.cfc">Recipient is CFC?</span>
                    <input type="checkbox" name="is_cfc" ${state.is_cfc ? 'checked' : ''}></label>
                <label><span data-i18n="view.s367d.label.csa">Cost Sharing Arrangement § 482?</span>
                    <input type="checkbox" name="cost_sharing_arrangement" ${state.cost_sharing_arrangement ? 'checked' : ''}></label>
                <label><span data-i18n="view.s367d.label.csa_costs">CSA share of costs ($)</span>
                    <input type="number" step="0.01" name="csa_share_costs" value="${state.csa_share_costs}"></label>
                <label><span data-i18n="view.s367d.label.buyin">CSA buy-in payment ($)</span>
                    <input type="number" step="0.01" name="csa_buy_in_payment" value="${state.csa_buy_in_payment}"></label>
                <label><span data-i18n="view.s367d.label.basis">Transferor's basis ($)</span>
                    <input type="number" step="0.01" name="transferor_basis" value="${state.transferor_basis}"></label>
                <label><span data-i18n="view.s367d.label.tcja">TCJA broad definition?</span>
                    <input type="checkbox" name="tcja_broad_definition" ${state.tcja_broad_definition ? 'checked' : ''}></label>
                <label><span data-i18n="view.s367d.label.us_corp">US filing corp transferor?</span>
                    <input type="checkbox" name="domestic_filing_corp" ${state.domestic_filing_corp ? 'checked' : ''}></label>
                <label><span data-i18n="view.s367d.label.year">Transfer year</span>
                    <input type="number" step="1" name="transfer_year" value="${state.transfer_year}"></label>
                <label><span data-i18n="view.s367d.label.received">Annual royalty received ($)</span>
                    <input type="number" step="0.01" name="annual_royalty_received" value="${state.annual_royalty_received}"></label>
                <button class="primary" type="submit" data-i18n="view.s367d.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s367d-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s367d.h2.intangible_def">TCJA expanded "intangible property" definition</h2>
            <ul class="muted small">
                <li data-i18n="view.s367d.def.patents">Patents, inventions, formulas, processes, designs, patterns, knowhow</li>
                <li data-i18n="view.s367d.def.copyrights">Copyrights, literary, musical, artistic compositions</li>
                <li data-i18n="view.s367d.def.trademarks">Trademarks, trade names, brand names, franchises</li>
                <li data-i18n="view.s367d.def.licenses">Licenses, permits, other rights granted by governmental units</li>
                <li data-i18n="view.s367d.def.contracts">Contracts (incl. employee contracts, supplier contracts)</li>
                <li data-i18n="view.s367d.def.customer_lists">Customer lists, customer relationships</li>
                <li data-i18n="view.s367d.def.workforce">Workforce in place — TCJA NEW</li>
                <li data-i18n="view.s367d.def.goodwill">Goodwill and going concern value — TCJA NEW</li>
                <li data-i18n="view.s367d.def.other">Methods of doing business, business records, operating systems</li>
                <li data-i18n="view.s367d.def.s936_no">§ 936(h)(3)(B) "intangible" definition broadened post-TCJA</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s367d.h2.csa">Cost Sharing Arrangement (CSA) § 482</h2>
            <ul class="muted small">
                <li data-i18n="view.s367d.csa.purpose">Avoid § 367(d) by SHARING development costs of new IP</li>
                <li data-i18n="view.s367d.csa.entry_fee">Foreign sub pays BUY-IN to US sub for existing IP value</li>
                <li data-i18n="view.s367d.csa.ongoing">Ongoing R&D costs SHARED in proportion to expected benefits</li>
                <li data-i18n="view.s367d.csa.platform_contribution">Platform Contribution: existing IP buy-in determined via Income Method / CUT / DCM</li>
                <li data-i18n="view.s367d.csa.stock_based">Stock-based compensation MUST be included in cost base (Altera v. Comm'r)</li>
                <li data-i18n="view.s367d.csa.financial_substance">Each party must contribute substantial cost + receive substantial benefit</li>
                <li data-i18n="view.s367d.csa.documentation">Written agreement + contemporaneous documentation required</li>
                <li data-i18n="view.s367d.csa.amazon_altera">Tax Court precedent: Amazon (CSA cost methodology), Altera (SBC must be shared)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s367d.h2.computation">Annual deemed royalty computation</h2>
            <ol class="muted small">
                <li data-i18n="view.s367d.comp.useful_life">Useful life: estimated economic life of IP (typically 5-15 years)</li>
                <li data-i18n="view.s367d.comp.royalty">Annual royalty: arm's-length comparable (CUT, Profit Split, Income Method)</li>
                <li data-i18n="view.s367d.comp.amortization">Annual inclusion declines as IP's productive capacity decreases</li>
                <li data-i18n="view.s367d.comp.character">Character: ORDINARY royalty income (not capital gain)</li>
                <li data-i18n="view.s367d.comp.subF_overlap">Subject to subF / GILTI if held in CFC: avoid double-tax via § 78 gross-up</li>
                <li data-i18n="view.s367d.comp.transferee_basis">Transferee basis = FMV at transfer (capital expenditure)</li>
                <li data-i18n="view.s367d.comp.no_loss">No loss on transfer even if FMV &lt; basis</li>
                <li data-i18n="view.s367d.comp.no_capital_gain">No capital gain even on appreciated IP at transfer (annual royalty instead)</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s367d.h2.coordination">Coordination with § 367(a), § 482, § 936</h2>
            <ul class="muted small">
                <li data-i18n="view.s367d.coord.s367a">§ 367(a)(1): tangible property outbound — gain recognition (vs § 367(d) ongoing royalty)</li>
                <li data-i18n="view.s367d.coord.s367a3">§ 367(a)(3): active trade/business exception (TCJA removed for IP)</li>
                <li data-i18n="view.s367d.coord.s482">§ 482 transfer pricing: same arm's-length standard applies on royalty determination</li>
                <li data-i18n="view.s367d.coord.s951a">§ 951A GILTI: IP income now subject to GILTI inclusion if held in CFC</li>
                <li data-i18n="view.s367d.coord.s250">§ 250 FDII: incentive to keep IP in US (13.125% effective rate)</li>
                <li data-i18n="view.s367d.coord.s936">§ 936 possessions tax credit — repealed but historical context</li>
                <li data-i18n="view.s367d.coord.s367b">§ 367(b) inbound regulations — exit-charge for CFC liquidations</li>
                <li data-i18n="view.s367d.coord.s956">§ 956 investment in US property: keeps IP from US shareholder access</li>
            </ul>
        </div>
    `;
    document.getElementById('s367d-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.ip_fmv_at_transfer = Number(fd.get('ip_fmv_at_transfer')) || 0;
        state.annual_arm_length_royalty = Number(fd.get('annual_arm_length_royalty')) || 0;
        state.useful_life_years = Number(fd.get('useful_life_years')) || 0;
        state.foreign_recipient_corp = fd.get('foreign_recipient_corp');
        state.is_cfc = !!fd.get('is_cfc');
        state.cost_sharing_arrangement = !!fd.get('cost_sharing_arrangement');
        state.csa_share_costs = Number(fd.get('csa_share_costs')) || 0;
        state.csa_buy_in_payment = Number(fd.get('csa_buy_in_payment')) || 0;
        state.transferor_basis = Number(fd.get('transferor_basis')) || 0;
        state.tcja_broad_definition = !!fd.get('tcja_broad_definition');
        state.domestic_filing_corp = !!fd.get('domestic_filing_corp');
        state.transfer_year = Number(fd.get('transfer_year')) || 0;
        state.annual_royalty_received = Number(fd.get('annual_royalty_received')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s367d-output');
    if (!el) return;
    const appliesS367d = (state.foreign_recipient_corp === 'cfc' || state.foreign_recipient_corp === 'related_foreign') && state.tcja_broad_definition && !state.cost_sharing_arrangement;
    const annualRoyalty = appliesS367d ? state.annual_arm_length_royalty : 0;
    const lifeTotal = state.annual_arm_length_royalty * state.useful_life_years;
    const taxAnnual = annualRoyalty * 0.21;
    const taxLifetime = lifeTotal * 0.21;
    const csaSavings = state.cost_sharing_arrangement ? lifeTotal * 0.21 : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s367d.h2.result">§ 367(d) computation</h2>
            <div class="cards">
                <div class="card ${appliesS367d ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s367d.card.applies">§ 367(d) applies?</div>
                    <div class="value">${appliesS367d ? esc(t('view.s367d.status.yes')) : esc(t('view.s367d.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s367d.card.fmv">IP FMV at transfer</div>
                    <div class="value">$${state.ip_fmv_at_transfer.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s367d.card.annual">Annual deemed royalty</div>
                    <div class="value">$${annualRoyalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s367d.card.life">Useful life total</div>
                    <div class="value">$${lifeTotal.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s367d.card.tax_annual">Annual tax (21%)</div>
                    <div class="value">$${taxAnnual.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s367d.card.tax_lifetime">Total tax over life</div>
                    <div class="value">$${taxLifetime.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s367d.card.csa_savings">CSA tax savings</div>
                    <div class="value">$${csaSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.cost_sharing_arrangement ? `
                <p class="muted small pos" style="margin-top:10px" data-i18n="view.s367d.csa_note">
                    CSA in place: § 367(d) deemed royalty AVOIDED. Buy-in $${state.csa_buy_in_payment.toLocaleString()}
                    + ongoing cost share. Critical: stock-based compensation MUST be included in cost base (Altera).
                    Document IP development costs + benefit allocation contemporaneously. Tax Court has reviewed
                    Amazon + Altera challenges to enforce arm's-length CSA pricing.
                </p>
            ` : ''}
        </div>
    `;
}
