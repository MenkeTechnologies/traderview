// IRC § 311 — Corporate Distribution of Appreciated Property.
// § 311(a): generally NO gain / loss on distribution to shareholder.
// § 311(b): EXCEPT distributing corp recognizes GAIN on appreciated property (FMV > basis).
// No loss on depreciated property (asymmetric — same as § 351 boot rule).
// Liabilities assumed by shareholder: treated as cash → FMV = greater of FMV or liab.

import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    property_fmv: 0,
    property_basis: 0,
    liabilities_assumed: 0,
    is_distribution_to_shareholder: true,
    is_qualified_dividend: false,
    is_redemption: false,
    e_and_p_available: 0,
    shareholder_basis_in_stock: 0,
    is_c_corp_shareholder: false,
    drd_pct: 0,
    is_liquidating_distribution: false,
    is_corporation_dist: false,
    related_party_shareholder: false,
};

export async function renderSection311(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s311.h1.title">// § 311 CORP DISTRIBUTIONS</span></h1>
        <p class="muted small" data-i18n="view.s311.hint.intro">
            <strong>§ 311(a):</strong> Distributing corp generally recognizes NO gain / loss on non-liquidating
            distribution. <strong>§ 311(b):</strong> EXCEPT corp recognizes <strong>GAIN</strong> on
            APPRECIATED property (FMV &gt; basis). <strong>NO LOSS</strong> on depreciated property — asymmetric.
            <strong>Liabilities assumed:</strong> treated as cash; FMV = greater of FMV or liab.
            <strong>Shareholder side:</strong> § 301 dividend (ordinary) to extent of E&P, then return
            of capital, then capital gain. <strong>Liquidating:</strong> § 336 (general gain) or § 337
            (≥ 80% sub).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s311.h2.inputs">Inputs</h2>
            <form id="s311-form" class="inline-form">
                <label><span data-i18n="view.s311.label.fmv">Property FMV ($)</span>
                    <input type="number" step="0.01" name="property_fmv" value="${state.property_fmv}"></label>
                <label><span data-i18n="view.s311.label.basis">Property corp basis ($)</span>
                    <input type="number" step="0.01" name="property_basis" value="${state.property_basis}"></label>
                <label><span data-i18n="view.s311.label.liab">Liabilities assumed by shareholder ($)</span>
                    <input type="number" step="0.01" name="liabilities_assumed" value="${state.liabilities_assumed}"></label>
                <label><span data-i18n="view.s311.label.is_dist">Non-liquidating distribution?</span>
                    <input type="checkbox" name="is_distribution_to_shareholder" ${state.is_distribution_to_shareholder ? 'checked' : ''}></label>
                <label><span data-i18n="view.s311.label.qualified">Qualified dividend for shareholder?</span>
                    <input type="checkbox" name="is_qualified_dividend" ${state.is_qualified_dividend ? 'checked' : ''}></label>
                <label><span data-i18n="view.s311.label.redemption">Redemption (§ 302)?</span>
                    <input type="checkbox" name="is_redemption" ${state.is_redemption ? 'checked' : ''}></label>
                <label><span data-i18n="view.s311.label.ep">E&P available ($)</span>
                    <input type="number" step="0.01" name="e_and_p_available" value="${state.e_and_p_available}"></label>
                <label><span data-i18n="view.s311.label.basis_stock">Shareholder basis in stock ($)</span>
                    <input type="number" step="0.01" name="shareholder_basis_in_stock" value="${state.shareholder_basis_in_stock}"></label>
                <label><span data-i18n="view.s311.label.corp_share">C-corp shareholder (DRD applies)?</span>
                    <input type="checkbox" name="is_c_corp_shareholder" ${state.is_c_corp_shareholder ? 'checked' : ''}></label>
                <label><span data-i18n="view.s311.label.drd">DRD % (50/65/100)</span>
                    <input type="number" step="0.1" name="drd_pct" value="${state.drd_pct}"></label>
                <label><span data-i18n="view.s311.label.liquidating">Liquidating distribution?</span>
                    <input type="checkbox" name="is_liquidating_distribution" ${state.is_liquidating_distribution ? 'checked' : ''}></label>
                <label><span data-i18n="view.s311.label.corp_dist">Distributing corp is C-corp?</span>
                    <input type="checkbox" name="is_corporation_dist" ${state.is_corporation_dist ? 'checked' : ''}></label>
                <label><span data-i18n="view.s311.label.related">Related-party shareholder?</span>
                    <input type="checkbox" name="related_party_shareholder" ${state.related_party_shareholder ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s311.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s311-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s311.h2.shareholder_side">Shareholder-level treatment (§ 301)</h2>
            <ol class="muted small">
                <li data-i18n="view.s311.share.fmv">Amount received = FMV of distributed property + liab discharged</li>
                <li data-i18n="view.s311.share.dividend">Dividend (ordinary) to extent of E&P (current + accumulated)</li>
                <li data-i18n="view.s311.share.return_capital">Return of capital — reduces stock basis to zero</li>
                <li data-i18n="view.s311.share.gain">Remaining = capital gain (LTCG if stock held &gt; 1 yr)</li>
                <li data-i18n="view.s311.share.basis">Shareholder takes FMV BASIS in distributed property (§ 301(d))</li>
                <li data-i18n="view.s311.share.qualified_div">Qualified dividend rate (20%) if individual + 60-day holding + domestic</li>
                <li data-i18n="view.s311.share.corp_drd">Corp shareholder: § 243 DRD (50/65/100%) may reduce dividend tax</li>
                <li data-i18n="view.s311.share.holding_period">Holding period for distributed property starts on distribution date</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s311.h2.corp_side">Corporate-level treatment (§ 311)</h2>
            <ul class="muted small">
                <li data-i18n="view.s311.corp.no_loss">§ 311(a) general rule: NO gain / loss recognized</li>
                <li data-i18n="view.s311.corp.gain">§ 311(b): GAIN recognized on appreciated property (deemed sale at FMV)</li>
                <li data-i18n="view.s311.corp.no_loss_b">§ 311(b)(1): NO LOSS allowed on depreciated property — asymmetric</li>
                <li data-i18n="view.s311.corp.liab">If liab &gt; FMV, treated as FMV = liab (Crane / § 7701(g))</li>
                <li data-i18n="view.s311.corp.character">Character of gain: same as if sold (ordinary, capital, depreciation recapture)</li>
                <li data-i18n="view.s311.corp.ep_reduction">E&P reduced by greater of FMV or liab (§ 312(b))</li>
                <li data-i18n="view.s311.corp.recapture">§ 1245 / § 1250 recapture triggered as if sold</li>
                <li data-i18n="view.s311.corp.s355_distinction">§ 355 spinoffs: different rules (§ 361 — corp side, § 355 — shareholder)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s311.h2.planning">Distribution planning levers</h2>
            <ul class="muted small">
                <li data-i18n="view.s311.plan.dist_loss_assets">Distribute LOSS property first (no loss recognized but reduce E&P / basis)</li>
                <li data-i18n="view.s311.plan.sell_gain_keep">Better: SELL gain assets first (recognize gain) → distribute cash</li>
                <li data-i18n="view.s311.plan.liquidating">Liquidating: § 336 allows loss recognition (different rule)</li>
                <li data-i18n="view.s311.plan.s_corp">S-corp pass-through: gain flows to shareholders (single tax) — better than C-corp</li>
                <li data-i18n="view.s311.plan.installment">§ 453 installment method on sale → defer cash + gain</li>
                <li data-i18n="view.s311.plan.related_party">§ 267 related party rules: loss disallowed even on outright sale</li>
                <li data-i18n="view.s311.plan.basis_step">Basis step-up to shareholder may offset corp gain economically</li>
                <li data-i18n="view.s311.plan.high_low">Distribute high-basis property; sell low-basis</li>
            </ul>
        </div>
    `;
    document.getElementById('s311-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.property_fmv = Number(fd.get('property_fmv')) || 0;
        state.property_basis = Number(fd.get('property_basis')) || 0;
        state.liabilities_assumed = Number(fd.get('liabilities_assumed')) || 0;
        state.is_distribution_to_shareholder = !!fd.get('is_distribution_to_shareholder');
        state.is_qualified_dividend = !!fd.get('is_qualified_dividend');
        state.is_redemption = !!fd.get('is_redemption');
        state.e_and_p_available = Number(fd.get('e_and_p_available')) || 0;
        state.shareholder_basis_in_stock = Number(fd.get('shareholder_basis_in_stock')) || 0;
        state.is_c_corp_shareholder = !!fd.get('is_c_corp_shareholder');
        state.drd_pct = Number(fd.get('drd_pct')) || 0;
        state.is_liquidating_distribution = !!fd.get('is_liquidating_distribution');
        state.is_corporation_dist = !!fd.get('is_corporation_dist');
        state.related_party_shareholder = !!fd.get('related_party_shareholder');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s311-output');
    if (!el) return;
    const effectiveFMV = Math.max(state.property_fmv, state.liabilities_assumed);
    const corpGainBuiltIn = effectiveFMV - state.property_basis;
    const corpGainRecognized = corpGainBuiltIn > 0 ? corpGainBuiltIn : 0;
    const corpTax = corpGainRecognized * 0.21;
    const amountToShareholder = effectiveFMV - state.liabilities_assumed;
    const dividendAmount = Math.min(amountToShareholder, state.e_and_p_available);
    const returnOfCapital = Math.min(Math.max(0, amountToShareholder - dividendAmount), state.shareholder_basis_in_stock);
    const capitalGain = amountToShareholder - dividendAmount - returnOfCapital;
    const shareholderDividendRate = state.is_c_corp_shareholder ? (0.21 * (1 - state.drd_pct / 100)) : (state.is_qualified_dividend ? 0.20 : 0.37);
    const dividendTax = dividendAmount * shareholderDividendRate;
    const capitalGainTax = capitalGain * 0.20;
    const totalShareholderTax = dividendTax + capitalGainTax;
    const totalSystemTax = corpTax + totalShareholderTax;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s311.h2.result">§ 311 / § 301 outcome</h2>
            <div class="cards">
                <div class="card ${corpGainBuiltIn < 0 ? 'neg' : ''}">
                    <div class="label" data-i18n="view.s311.card.builtin">Built-in gain / loss</div>
                    <div class="value">$${corpGainBuiltIn.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s311.card.corp_gain">Corp gain recognized</div>
                    <div class="value">$${corpGainRecognized.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s311.card.corp_tax">Corp tax (21%)</div>
                    <div class="value">$${corpTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s311.card.dividend">Dividend portion</div>
                    <div class="value">$${dividendAmount.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s311.card.return_cap">Return of capital</div>
                    <div class="value">$${returnOfCapital.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s311.card.cap_gain">Capital gain to shareholder</div>
                    <div class="value">$${capitalGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s311.card.dividend_tax">Shareholder dividend tax</div>
                    <div class="value">$${dividendTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s311.card.total">Total system tax</div>
                    <div class="value">$${totalSystemTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${corpGainBuiltIn < 0 ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s311.no_loss_note">
                    Asymmetric § 311(b): corp gain on appreciation recognized BUT corp loss on depreciation
                    NEVER recognized. Distribute LOSS property to reduce E&P without tax cost; sell GAIN
                    property to control timing.
                </p>
            ` : ''}
        </div>
    `;
}
