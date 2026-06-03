// IRC § 1248 — Sale of CFC Stock Recharacterizes E&P as Dividend.
// US 10% shareholder selling CFC stock: gain up to allocated foreign E&P treated as DIVIDEND.
// Post-TCJA + § 245A: recharacterized dividend often qualifies for 100% DRD (C-corps).
// Individual US shareholder: dividend rate (qualified if treaty country + holding pd).
// Post-2017 PTI (PTEP) excluded — already taxed under GILTI / Subpart F.

import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    sale_price: 0,
    stock_basis: 0,
    allocated_ep: 0,
    ptep_account: 0,
    holding_period_pct_yrs: 0,
    us_ownership_pct: 0,
    is_c_corp: true,
    s245A_qualifies: false,
    treaty_country: false,
    indirect_credit_pct: 0,
    foreign_tax_credit: 0,
    is_qualified_dividend: false,
};

export async function renderSection1248(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1248.h1.title">// § 1248 CFC STOCK SALE</span></h1>
        <p class="muted small" data-i18n="view.s1248.hint.intro">
            <strong>US 10% shareholder</strong> selling CFC stock: gain up to allocated <strong>foreign E&P</strong>
            recharacterized as <strong>DIVIDEND</strong>. <strong>Post-2017 PTEP</strong> (previously taxed E&P
            from GILTI / Subpart F) excluded — already taxed. <strong>C-corp shareholder + § 245A 100% DRD</strong>:
            recharacterized portion exempt → tax-free repatriation via sale. <strong>Individual:</strong>
            qualified dividend rate (20%) if treaty country + holding period. § 1248(f) also reaches outbound
            transfers (§ 367(b)). Form 5471 Sch P + § 1248 worksheet.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1248.h2.inputs">Inputs</h2>
            <form id="s1248-form" class="inline-form">
                <label><span data-i18n="view.s1248.label.sale">Sale price ($)</span>
                    <input type="number" step="100000" name="sale_price" value="${state.sale_price}"></label>
                <label><span data-i18n="view.s1248.label.basis">Stock basis ($)</span>
                    <input type="number" step="100000" name="stock_basis" value="${state.stock_basis}"></label>
                <label><span data-i18n="view.s1248.label.ep">Allocated foreign E&P ($)</span>
                    <input type="number" step="10000" name="allocated_ep" value="${state.allocated_ep}"></label>
                <label><span data-i18n="view.s1248.label.ptep">PTEP / PTI account ($)</span>
                    <input type="number" step="10000" name="ptep_account" value="${state.ptep_account}"></label>
                <label><span data-i18n="view.s1248.label.holding">Holding period (years 10%+ owner)</span>
                    <input type="number" step="0.5" name="holding_period_pct_yrs" value="${state.holding_period_pct_yrs}"></label>
                <label><span data-i18n="view.s1248.label.ownership">US ownership %</span>
                    <input type="number" step="0.1" name="us_ownership_pct" value="${state.us_ownership_pct}"></label>
                <label><span data-i18n="view.s1248.label.corp">C-corp shareholder?</span>
                    <input type="checkbox" name="is_c_corp" ${state.is_c_corp ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1248.label.s245a">§ 245A 100% DRD qualifies?</span>
                    <input type="checkbox" name="s245A_qualifies" ${state.s245A_qualifies ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1248.label.treaty">Treaty country?</span>
                    <input type="checkbox" name="treaty_country" ${state.treaty_country ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1248.label.indirect">Indirect FTC pct (deemed paid)</span>
                    <input type="number" step="0.1" name="indirect_credit_pct" value="${state.indirect_credit_pct}"></label>
                <label><span data-i18n="view.s1248.label.ftc">FTC available ($)</span>
                    <input type="number" step="1000" name="foreign_tax_credit" value="${state.foreign_tax_credit}"></label>
                <label><span data-i18n="view.s1248.label.qualified">Qualified dividend treatment (individual)?</span>
                    <input type="checkbox" name="is_qualified_dividend" ${state.is_qualified_dividend ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s1248.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1248-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1248.h2.applies">§ 1248 application</h2>
            <ol class="muted small">
                <li data-i18n="view.s1248.app.shareholder">US 10% shareholder (vote OR value) sells CFC stock</li>
                <li data-i18n="view.s1248.app.cfc">Foreign corp was CFC at any time during 5-year period before sale</li>
                <li data-i18n="view.s1248.app.shareholder_during">Seller was US 10% shareholder at any time during 5-year period</li>
                <li data-i18n="view.s1248.app.gain">Recharacterize gain up to allocated E&P (current-yr inclusion + accumulated)</li>
                <li data-i18n="view.s1248.app.ptep">PTEP / PTI excluded — already-taxed amounts not re-taxed</li>
                <li data-i18n="view.s1248.app.attribution">Constructive ownership rules apply (§ 958 + § 318)</li>
                <li data-i18n="view.s1248.app.coordination">Coordinates with § 351 / § 354 / § 355 / § 367 / § 1411</li>
                <li data-i18n="view.s1248.app.controlled_chain">Sale of foreign corp holding CFCs: chain attribution to lower-tier CFC E&P</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1248.h2.planning">§ 1248 + § 245A planning levers</h2>
            <ul class="muted small">
                <li data-i18n="view.s1248.plan.s245a">C-corp seller + § 245A: 100% DRD on recharacterized portion → tax-free</li>
                <li data-i18n="view.s1248.plan.holding">Holding period: 365 days (§ 246(c) for § 245A) — plan acquisition timing</li>
                <li data-i18n="view.s1248.plan.subF_purification">Pre-sale "purification": dividend out subF / GILTI accumulated E&P</li>
                <li data-i18n="view.s1248.plan.s367b">§ 367(b) outbound: similar mechanism on outbound transfers</li>
                <li data-i18n="view.s1248.plan.high_tax">High-tax accumulated E&P: better creditability post-recharacterization</li>
                <li data-i18n="view.s1248.plan.individual_treaty">Individual: qualified dividend if treaty country + 60-day hold (otherwise ordinary)</li>
                <li data-i18n="view.s1248.plan.ptep_first">PTEP-first accounting: deplete PTEP before § 1248 recharacterization</li>
                <li data-i18n="view.s1248.plan.s962_election">§ 962 individual: lock-in 21% corp rate + § 245A path</li>
            </ul>
        </div>
    `;
    document.getElementById('s1248-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.sale_price = Number(fd.get('sale_price')) || 0;
        state.stock_basis = Number(fd.get('stock_basis')) || 0;
        state.allocated_ep = Number(fd.get('allocated_ep')) || 0;
        state.ptep_account = Number(fd.get('ptep_account')) || 0;
        state.holding_period_pct_yrs = Number(fd.get('holding_period_pct_yrs')) || 0;
        state.us_ownership_pct = Number(fd.get('us_ownership_pct')) || 0;
        state.is_c_corp = !!fd.get('is_c_corp');
        state.s245A_qualifies = !!fd.get('s245A_qualifies');
        state.treaty_country = !!fd.get('treaty_country');
        state.indirect_credit_pct = Number(fd.get('indirect_credit_pct')) || 0;
        state.foreign_tax_credit = Number(fd.get('foreign_tax_credit')) || 0;
        state.is_qualified_dividend = !!fd.get('is_qualified_dividend');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1248-output');
    if (!el) return;
    const totalGain = Math.max(0, state.sale_price - state.stock_basis);
    const epAfterPTEP = Math.max(0, state.allocated_ep - state.ptep_account);
    const recharacterized = Math.min(totalGain, epAfterPTEP);
    const remainingCapitalGain = Math.max(0, totalGain - recharacterized);
    const s245ADRD = (state.is_c_corp && state.s245A_qualifies) ? recharacterized : 0;
    const dividendAfterDRD = recharacterized - s245ADRD;
    const dividendRate = state.is_c_corp ? 0.21 : (state.is_qualified_dividend ? 0.20 : 0.37);
    const capGainRate = 0.20;
    const dividendTax = dividendAfterDRD * dividendRate;
    const capGainTax = remainingCapitalGain * capGainRate;
    const netTax = Math.max(0, dividendTax + capGainTax - state.foreign_tax_credit);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1248.h2.result">§ 1248 outcome</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s1248.card.total_gain">Total realized gain</div>
                    <div class="value">$${totalGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1248.card.ptep_excluded">PTEP excluded</div>
                    <div class="value">$${state.ptep_account.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1248.card.recharacterized">Recharacterized → dividend</div>
                    <div class="value">$${recharacterized.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1248.card.s245a">§ 245A 100% DRD</div>
                    <div class="value">$${s245ADRD.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1248.card.cap_gain">Remaining capital gain</div>
                    <div class="value">$${remainingCapitalGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1248.card.div_tax">Dividend tax</div>
                    <div class="value">$${dividendTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1248.card.cg_tax">Cap gain tax (20%)</div>
                    <div class="value">$${capGainTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1248.card.net">Net total tax</div>
                    <div class="value">$${netTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.s245A_qualifies && state.is_c_corp ? `
                <p class="muted small pos" style="margin-top:10px" data-i18n="view.s1248.s245a_note">
                    § 245A 100% DRD applied to recharacterized portion. Result: tax-free repatriation of CFC
                    E&P via stock sale. Remaining capital gain still taxed at 20%. Combined with high-tax
                    GILTI exclusion + PTEP basis adjustments, this is the optimal exit path for US C-corps.
                </p>
            ` : ''}
        </div>
    `;
}
