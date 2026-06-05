// IRC § 336 — Corporate Liquidation (General Rule for Non-Subsidiary).
// § 336(a): Corp recognizes GAIN AND LOSS on liquidating distribution at FMV.
// Contrast § 311 (non-liquidating, gain only). Loss disallowed in some related-party situations.
// § 336(d)(1): no loss if "disqualified property" + § 351 contribution within 2 years.
// § 336(d)(2): "tax avoidance" loss reduced if contributed property's basis exceeds FMV.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    asset_fmv_total: 0,
    asset_basis_total: 0,
    liab_assumed: 0,
    shareholder_basis_in_stock: 0,
    is_complete_liquidation: true,
    is_solvent: true,
    related_party_distribution: false,
    disqualified_property: false,
    contributed_within_2yr: false,
    s336e_election: false,
    foreign_distributee: false,
    appreciation_amount: 0,
    depreciation_amount: 0,
};

export async function renderSection336(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s336.h1.title">// § 336 CORP LIQUIDATION</span></h1>
        <p class="muted small" data-i18n="view.s336.hint.intro">
            <strong>§ 336(a):</strong> Corp recognizes <strong>GAIN AND LOSS</strong> on liquidating distribution
            at FMV. Contrast § 311 (non-liquidating, gain ONLY). <strong>§ 336(d)(1):</strong> NO LOSS on
            distribution of "disqualified property" (contributed in § 351 / capital contrib within 2 yrs to
            related party). <strong>§ 336(d)(2):</strong> "Tax avoidance" loss reduced if basis &gt; FMV.
            <strong>Shareholder side:</strong> § 331 — gain / loss on stock; FMV basis in distributed property.
            <strong>§ 332 alternate:</strong> tax-free if 80%+ parent (§ 337 for sub).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s336.h2.inputs">Inputs</h2>
            <form id="s336-form" class="inline-form">
                <label><span data-i18n="view.s336.label.fmv">Total asset FMV ($)</span>
                    <input type="number" step="0.01" name="asset_fmv_total" value="${state.asset_fmv_total}"></label>
                <label><span data-i18n="view.s336.label.basis">Total asset basis ($)</span>
                    <input type="number" step="0.01" name="asset_basis_total" value="${state.asset_basis_total}"></label>
                <label><span data-i18n="view.s336.label.liab">Liabilities assumed by shareholder ($)</span>
                    <input type="number" step="0.01" name="liab_assumed" value="${state.liab_assumed}"></label>
                <label><span data-i18n="view.s336.label.stock_basis">Shareholder stock basis ($)</span>
                    <input type="number" step="0.01" name="shareholder_basis_in_stock" value="${state.shareholder_basis_in_stock}"></label>
                <label><span data-i18n="view.s336.label.complete">Complete liquidation?</span>
                    <input type="checkbox" name="is_complete_liquidation" ${state.is_complete_liquidation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s336.label.solvent">Solvent?</span>
                    <input type="checkbox" name="is_solvent" ${state.is_solvent ? 'checked' : ''}></label>
                <label><span data-i18n="view.s336.label.related">Related-party distribution?</span>
                    <input type="checkbox" name="related_party_distribution" ${state.related_party_distribution ? 'checked' : ''}></label>
                <label><span data-i18n="view.s336.label.disqualified">"Disqualified property" (§ 336(d)(1))?</span>
                    <input type="checkbox" name="disqualified_property" ${state.disqualified_property ? 'checked' : ''}></label>
                <label><span data-i18n="view.s336.label.contributed">Contributed within 2 yrs?</span>
                    <input type="checkbox" name="contributed_within_2yr" ${state.contributed_within_2yr ? 'checked' : ''}></label>
                <label><span data-i18n="view.s336.label.s336e">§ 336(e) elective (sister to 338(h)(10))?</span>
                    <input type="checkbox" name="s336e_election" ${state.s336e_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s336.label.foreign">Foreign distributee?</span>
                    <input type="checkbox" name="foreign_distributee" ${state.foreign_distributee ? 'checked' : ''}></label>
                <label><span data-i18n="view.s336.label.app">Appreciation amount ($)</span>
                    <input type="number" step="0.01" name="appreciation_amount" value="${state.appreciation_amount}"></label>
                <label><span data-i18n="view.s336.label.dep">Depreciation amount ($)</span>
                    <input type="number" step="0.01" name="depreciation_amount" value="${state.depreciation_amount}"></label>
                <button class="primary" type="submit" data-i18n="view.s336.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s336-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s336.h2.shareholder">§ 331 shareholder treatment</h2>
            <ul class="muted small">
                <li data-i18n="view.s336.share.amount">Amount realized = FMV of property + cash + liability discharged (not assumed)</li>
                <li data-i18n="view.s336.share.gain_loss">Gain / loss = amount realized − stock basis (capital character)</li>
                <li data-i18n="view.s336.share.holding">Holding period: from original acquisition; LTCG if &gt; 1 yr</li>
                <li data-i18n="view.s336.share.basis_property">Basis in distributed property = FMV (step-up)</li>
                <li data-i18n="view.s336.share.depreciation_recap">No depreciation recapture passes through to shareholder (corp recognizes at § 336)</li>
                <li data-i18n="view.s336.share.installment">§ 453(h): installment note received in liquidation reported on installment method</li>
                <li data-i18n="view.s336.share.s453B_d">§ 453B(d): NO recognition on dist of installment notes (deferred)</li>
                <li data-i18n="view.s336.share.minority">Minority shareholders (&lt; 20%): always § 331 (even if 80%+ parent uses § 332)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s336.h2.loss_limits">Loss limitation rules</h2>
            <ul class="muted small">
                <li data-i18n="view.s336.loss.s336d1">§ 336(d)(1): NO LOSS on distribution of disqualified property to related party (§ 267)</li>
                <li data-i18n="view.s336.loss.disqualified">"Disqualified property" = property contributed in § 351 / capital contrib + tax-avoidance purpose</li>
                <li data-i18n="view.s336.loss.s336d2">§ 336(d)(2): "tax avoidance" loss reduced if asset's basis &gt; FMV at contribution date</li>
                <li data-i18n="view.s336.loss.related">Related party: § 267 — &gt; 50% owner, family, controlled corps</li>
                <li data-i18n="view.s336.loss.allocation">Loss allocated pro-rata across distributee shareholders</li>
                <li data-i18n="view.s336.loss.disqual_period">5-year period: contribution + tax avoidance presumed within 2 yrs</li>
                <li data-i18n="view.s336.loss.s336d3">§ 336(d)(3): no loss to 80% parent under § 332 (general non-recognition)</li>
                <li data-i18n="view.s336.loss.foreign">§ 337(b)(2) foreign sub: liquidation triggers § 367(b) inclusion</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s336.h2.s336e">§ 336(e) election (sister to § 338(h)(10))</h2>
            <ul class="muted small">
                <li data-i18n="view.s336.s336e.scope">Stock sale to UNRELATED party by single SELLER (not buyer) recharacterized as asset sale</li>
                <li data-i18n="view.s336.s336e.s_corp">S-corp target OR consolidated sub eligible (similar to § 338(h)(10))</li>
                <li data-i18n="view.s336.s336e.consent">Seller-side election; no buyer consent required</li>
                <li data-i18n="view.s336.s336e.basis_step">Buyer gets stepped-up basis to AGUB (similar to 338(h)(10))</li>
                <li data-i18n="view.s336.s336e.gain_seller">Gain recognized at seller / consolidated group level</li>
                <li data-i18n="view.s336.s336e.s_corp_basis">S-corp shareholders adjust basis for inside-built gain</li>
                <li data-i18n="view.s336.s336e.election">Election: attached to seller's tax return for year of sale</li>
                <li data-i18n="view.s336.s336e.use_case">Useful when buyer is reluctant to join § 338(h)(10) but seller wants asset-sale treatment</li>
            </ul>
        </div>
    `;
    document.getElementById('s336-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.asset_fmv_total = Number(fd.get('asset_fmv_total')) || 0;
        state.asset_basis_total = Number(fd.get('asset_basis_total')) || 0;
        state.liab_assumed = Number(fd.get('liab_assumed')) || 0;
        state.shareholder_basis_in_stock = Number(fd.get('shareholder_basis_in_stock')) || 0;
        state.is_complete_liquidation = !!fd.get('is_complete_liquidation');
        state.is_solvent = !!fd.get('is_solvent');
        state.related_party_distribution = !!fd.get('related_party_distribution');
        state.disqualified_property = !!fd.get('disqualified_property');
        state.contributed_within_2yr = !!fd.get('contributed_within_2yr');
        state.s336e_election = !!fd.get('s336e_election');
        state.foreign_distributee = !!fd.get('foreign_distributee');
        state.appreciation_amount = Number(fd.get('appreciation_amount')) || 0;
        state.depreciation_amount = Number(fd.get('depreciation_amount')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s336-output');
    if (!el) return;
    const lossAllowed = !(state.disqualified_property && state.related_party_distribution);
    const corpGain = state.appreciation_amount;
    const corpLoss = lossAllowed ? state.depreciation_amount : 0;
    const netCorpResult = corpGain - corpLoss;
    const corpTax = Math.max(0, netCorpResult) * 0.21;
    const amountToShareholder = state.asset_fmv_total - state.liab_assumed;
    const shareholderGainLoss = amountToShareholder - state.shareholder_basis_in_stock;
    const shareholderTax = shareholderGainLoss > 0 ? shareholderGainLoss * 0.20 : 0;
    const totalTax = corpTax + shareholderTax;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s336.h2.result">§ 336 outcome</h2>
            <div class="cards">
                <div class="card ${lossAllowed ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s336.card.loss_allowed">Loss allowed?</div>
                    <div class="value">${lossAllowed ? esc(t('view.s336.status.yes')) : esc(t('view.s336.status.no'))}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s336.card.corp_gain">Corp gain (appreciation)</div>
                    <div class="value">$${corpGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s336.card.corp_loss">Corp loss (depreciation)</div>
                    <div class="value">$${corpLoss.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s336.card.corp_tax">Corp tax (21%)</div>
                    <div class="value">$${corpTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${shareholderGainLoss > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s336.card.share_gain">Shareholder gain / loss</div>
                    <div class="value">$${shareholderGainLoss.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s336.card.share_tax">Shareholder tax (20%)</div>
                    <div class="value">$${shareholderTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s336.card.total">Total system tax</div>
                    <div class="value">$${totalTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${!lossAllowed && state.depreciation_amount > 0 ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s336.disqualified_note">
                    Loss DISALLOWED — § 336(d)(1) disqualified property + related-party distribution.
                    Allocate loss-property to UNRELATED shareholders, OR sell BEFORE liquidation, OR avoid
                    § 351 contributions within 2 yrs of planned liquidation.
                </p>
            ` : ''}
        </div>
    `;
}
