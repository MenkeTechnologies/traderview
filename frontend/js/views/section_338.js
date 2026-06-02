// IRC § 338 — Election to Treat Stock Acquisition as Asset Purchase.
// § 338(g): unilateral by purchaser → target deemed to sell assets + liquidate.
// § 338(h)(10): joint by purchaser + seller → only for S-corps + consolidated groups.
// Target gets step-up to AGUB (Aggregate Grossed-Up Basis) — buyer's basis in assets.
// Trade-off: immediate gain to seller's group vs buyer's step-up + amortization.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    purchase_price: 0,
    target_inside_basis: 0,
    target_outside_basis: 0,
    target_liabilities: 0,
    target_nol_carryforward: 0,
    target_unused_credits: 0,
    election_type: '338g',
    target_type: 'subsidiary',
    s_corp_target: false,
    years_step_up_recovery: 15,
    seller_marginal_rate: 21,
    asset_class_intangible: 0,
    asset_class_real_estate: 0,
    asset_class_personal_property: 0,
};

export async function renderSection338(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s338.h1.title">// § 338 STOCK→ASSET ELECTION</span></h1>
        <p class="muted small" data-i18n="view.s338.hint.intro">
            Stock purchase treated as ASSET PURCHASE for tax. <strong>§ 338(g):</strong> unilateral by buyer
            (Buyer pays target's immediate gain tax with no offset to Seller). <strong>§ 338(h)(10):</strong>
            joint election (Buyer + Seller) for S-corp OR consolidated-group target — gain recognized
            at SELLER level (one tax). Target gets step-up to <strong>AGUB</strong> (Aggregated Grossed-Up
            Basis). Tradeoff: immediate gain tax at acquisition vs step-up (depreciation / amortization)
            recovery. Filed on Form 8023.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s338.h2.inputs">Inputs</h2>
            <form id="s338-form" class="inline-form">
                <label><span data-i18n="view.s338.label.price">Purchase price (stock) ($)</span>
                    <input type="number" step="100000" name="purchase_price" value="${state.purchase_price}"></label>
                <label><span data-i18n="view.s338.label.inside">Target inside basis ($)</span>
                    <input type="number" step="100000" name="target_inside_basis" value="${state.target_inside_basis}"></label>
                <label><span data-i18n="view.s338.label.outside">Target outside basis ($)</span>
                    <input type="number" step="100000" name="target_outside_basis" value="${state.target_outside_basis}"></label>
                <label><span data-i18n="view.s338.label.liab">Target liabilities ($)</span>
                    <input type="number" step="100000" name="target_liabilities" value="${state.target_liabilities}"></label>
                <label><span data-i18n="view.s338.label.nol">Target NOL carryforward ($)</span>
                    <input type="number" step="10000" name="target_nol_carryforward" value="${state.target_nol_carryforward}"></label>
                <label><span data-i18n="view.s338.label.credits">Target unused credits ($)</span>
                    <input type="number" step="10000" name="target_unused_credits" value="${state.target_unused_credits}"></label>
                <label><span data-i18n="view.s338.label.type">Election type</span>
                    <select name="election_type">
                        <option value="338g" ${state.election_type === '338g' ? 'selected' : ''}>§ 338(g) — unilateral</option>
                        <option value="338h10" ${state.election_type === '338h10' ? 'selected' : ''}>§ 338(h)(10) — joint</option>
                        <option value="336e" ${state.election_type === '336e' ? 'selected' : ''}>§ 336(e) — sister election</option>
                        <option value="none" ${state.election_type === 'none' ? 'selected' : ''}>No election (stock purchase)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s338.label.target_type">Target type</span>
                    <select name="target_type">
                        <option value="subsidiary" ${state.target_type === 'subsidiary' ? 'selected' : ''}>Consolidated subsidiary</option>
                        <option value="s_corp" ${state.target_type === 's_corp' ? 'selected' : ''}>S-corp</option>
                        <option value="standalone_c" ${state.target_type === 'standalone_c' ? 'selected' : ''}>Standalone C-corp</option>
                        <option value="foreign" ${state.target_type === 'foreign' ? 'selected' : ''}>Foreign target</option>
                    </select>
                </label>
                <label><span data-i18n="view.s338.label.s_corp">S-corp target?</span>
                    <input type="checkbox" name="s_corp_target" ${state.s_corp_target ? 'checked' : ''}></label>
                <label><span data-i18n="view.s338.label.years">Years step-up recovery (avg)</span>
                    <input type="number" step="1" name="years_step_up_recovery" value="${state.years_step_up_recovery}"></label>
                <label><span data-i18n="view.s338.label.rate">Seller marginal rate %</span>
                    <input type="number" step="0.1" name="seller_marginal_rate" value="${state.seller_marginal_rate}"></label>
                <label><span data-i18n="view.s338.label.intangible">Asset Class VI/VII (intangible) ($)</span>
                    <input type="number" step="100000" name="asset_class_intangible" value="${state.asset_class_intangible}"></label>
                <label><span data-i18n="view.s338.label.real_estate">Asset Class V (real estate) ($)</span>
                    <input type="number" step="100000" name="asset_class_real_estate" value="${state.asset_class_real_estate}"></label>
                <label><span data-i18n="view.s338.label.personal">Asset Class IV/V (personal prop) ($)</span>
                    <input type="number" step="100000" name="asset_class_personal_property" value="${state.asset_class_personal_property}"></label>
                <button class="primary" type="submit" data-i18n="view.s338.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s338-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s338.h2.election_types">Election variations</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s338.th.type">Type</th>
                    <th data-i18n="view.s338.th.target">Eligible target</th>
                    <th data-i18n="view.s338.th.gain">Gain recognized by</th>
                    <th data-i18n="view.s338.th.consent">Consent</th>
                </tr></thead>
                <tbody>
                    <tr><td>§ 338(g)</td><td>Any C-corp (most common: foreign tgt)</td><td>TARGET (buyer's tax problem)</td><td>Buyer only</td></tr>
                    <tr><td>§ 338(h)(10)</td><td>S-corp OR consolidated sub</td><td>SELLER (one tax level)</td><td>Buyer + Seller joint</td></tr>
                    <tr><td>§ 336(e)</td><td>Stock sale by single seller</td><td>SELLER</td><td>Seller only (sister to 338(h)(10))</td></tr>
                    <tr><td>No election</td><td>—</td><td>SELLER (on stock sale)</td><td>—</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s338.h2.asset_classes">§ 1060 / § 338 Asset Class Allocation</h2>
            <ul class="muted small">
                <li data-i18n="view.s338.cls.cash">Class I: cash + bank deposits (no allocation)</li>
                <li data-i18n="view.s338.cls.actively_traded">Class II: actively traded securities + CDs</li>
                <li data-i18n="view.s338.cls.receivables">Class III: receivables, mortgages, credit card</li>
                <li data-i18n="view.s338.cls.stock_inventory">Class IV: stock-in-trade + inventory</li>
                <li data-i18n="view.s338.cls.tangible">Class V: tangible property (RE, equipment)</li>
                <li data-i18n="view.s338.cls.intangible_non_goodwill">Class VI: intangibles not in Class VII (§ 197, customer lists)</li>
                <li data-i18n="view.s338.cls.goodwill">Class VII: goodwill + going concern value</li>
                <li data-i18n="view.s338.cls.residual">Allocation: by FMV within each class, residual to Class VI/VII (goodwill)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s338.h2.tradeoff">When § 338 wins / loses</h2>
            <ul class="muted small">
                <li data-i18n="view.s338.win.high_intangibles">WIN: Target has significant intangibles → § 197 15-year amortization</li>
                <li data-i18n="view.s338.win.high_basis">WIN: Target has high inside basis already (small step-up cost)</li>
                <li data-i18n="view.s338.win.foreign_338g">§ 338(g) wins for foreign target (gain often deferred / minimal foreign-tax burden)</li>
                <li data-i18n="view.s338.win.s_corp">§ 338(h)(10) win for S-corp targets — single level gain</li>
                <li data-i18n="view.s338.loss.high_appreciation">LOSE: Target has high appreciated assets (huge gain tax)</li>
                <li data-i18n="view.s338.loss.nol_destruction">LOSE: Target has NOLs that would otherwise carry over (lost in election)</li>
                <li data-i18n="view.s338.loss.consolidated">LOSE: Acquirer can't use sub's losses against group (SRLY) anyway</li>
                <li data-i18n="view.s338.loss.qsst_disabled">LOSE: S-corp ESBT / QSST trust beneficiary may disqualify</li>
            </ul>
        </div>
    `;
    document.getElementById('s338-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.purchase_price = Number(fd.get('purchase_price')) || 0;
        state.target_inside_basis = Number(fd.get('target_inside_basis')) || 0;
        state.target_outside_basis = Number(fd.get('target_outside_basis')) || 0;
        state.target_liabilities = Number(fd.get('target_liabilities')) || 0;
        state.target_nol_carryforward = Number(fd.get('target_nol_carryforward')) || 0;
        state.target_unused_credits = Number(fd.get('target_unused_credits')) || 0;
        state.election_type = fd.get('election_type');
        state.target_type = fd.get('target_type');
        state.s_corp_target = !!fd.get('s_corp_target');
        state.years_step_up_recovery = Number(fd.get('years_step_up_recovery')) || 0;
        state.seller_marginal_rate = Number(fd.get('seller_marginal_rate')) || 0;
        state.asset_class_intangible = Number(fd.get('asset_class_intangible')) || 0;
        state.asset_class_real_estate = Number(fd.get('asset_class_real_estate')) || 0;
        state.asset_class_personal_property = Number(fd.get('asset_class_personal_property')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s338-output');
    if (!el) return;
    const electionMade = state.election_type !== 'none';
    const agub = state.purchase_price + state.target_liabilities;
    const stepUp = Math.max(0, agub - state.target_inside_basis);
    const targetGain = electionMade ? Math.max(0, agub - state.target_inside_basis) : 0;
    const targetGainTax = (state.election_type === '338g' || state.election_type === '338h10') ? targetGain * (state.seller_marginal_rate / 100) : 0;
    const stepUpRecoveryAnnual = state.years_step_up_recovery > 0 ? stepUp / state.years_step_up_recovery : 0;
    const taxSavingsAnnual = stepUpRecoveryAnnual * 0.21;
    const npv = state.years_step_up_recovery > 0 ?
        taxSavingsAnnual * (1 - Math.pow(1 + 0.05, -state.years_step_up_recovery)) / 0.05 : 0;
    const netBenefit = npv - targetGainTax;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s338.h2.result">§ 338 outcome</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s338.card.agub">AGUB (grossed-up basis)</div>
                    <div class="value">$${agub.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s338.card.step_up">Step-up amount</div>
                    <div class="value">$${stepUp.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s338.card.target_gain">Target gain recognized</div>
                    <div class="value">$${targetGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s338.card.tax_immediate">Immediate gain tax</div>
                    <div class="value">$${targetGainTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s338.card.annual_recovery">Annual recovery</div>
                    <div class="value">$${stepUpRecoveryAnnual.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s338.card.annual_tax_savings">Annual tax savings (21%)</div>
                    <div class="value">$${taxSavingsAnnual.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s338.card.npv_savings">NPV step-up benefit (5%)</div>
                    <div class="value">$${npv.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${netBenefit > 0 ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s338.card.net">Net benefit (NPV − tax)</div>
                    <div class="value">$${netBenefit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${netBenefit < 0 ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s338.no_election_note">
                    Election cost exceeds step-up benefit. Likely better off WITHOUT § 338 election —
                    accept stock-purchase treatment, retain target's NOLs / E&P, no immediate gain tax.
                </p>
            ` : `
                <p class="muted small pos" style="margin-top:10px" data-i18n="view.s338.election_note">
                    Election produces positive NPV. Coordinate § 1060 / § 338 residual allocation method
                    to maximize Class VI/VII (15-yr § 197 amortization) over Class IV/V (longer depreciation).
                </p>
            `}
        </div>
    `;
}
