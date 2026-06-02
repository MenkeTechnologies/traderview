// IRC § 482 — Transfer Pricing + Arm's-Length Principle.
// IRS may allocate income / deductions among related parties to clearly reflect income.
// Standards: arm's-length transactions, comparability + best method analysis.
// Methods: CUP, RPM, CPM, PSM, TNMM (for tangible goods + services + intangibles).
// Documentation: § 6662(e) 20%/40% penalty if no contemporaneous documentation.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    related_party_kind: 'controlled_subsidiary',
    transaction_type: 'tangible_goods',
    intercompany_price_used: 0,
    arms_length_low_quartile: 0,
    arms_length_high_quartile: 0,
    transaction_volume: 0,
    has_contemporaneous_documentation: false,
    used_best_method: true,
    apa_in_place: false,
    documentation_includes_six_principal_documents: false,
    transfer_pricing_method: 'cpm',
    foreign_party_country: '',
    is_substantial_understatement: false,
    is_gross_misstatement: false,
    marginal_rate: 0.21,
};

export async function renderSection482(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s482.h1.title">// § 482 TRANSFER PRICING</span></h1>
        <p class="muted small" data-i18n="view.s482.hint.intro">
            IRS may allocate income / deductions among related parties to clearly reflect income.
            <strong>Standards:</strong> arm's-length transactions, comparability + best method
            analysis. <strong>Methods:</strong> CUP, RPM, CPM, PSM, TNMM. <strong>§ 6662(e)
            penalty:</strong> 20% substantial / 40% gross misstatement if no contemporaneous
            documentation. <strong>Six principal documents</strong> required for protection.
            <strong>APA (Advance Pricing Agreement)</strong> provides certainty.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s482.h2.inputs">Inputs</h2>
            <form id="s482-form" class="inline-form">
                <label><span data-i18n="view.s482.label.related">Related party type</span>
                    <select name="related_party_kind">
                        <option value="controlled_subsidiary" ${state.related_party_kind === 'controlled_subsidiary' ? 'selected' : ''}>US parent — foreign sub</option>
                        <option value="foreign_parent_us_sub" ${state.related_party_kind === 'foreign_parent_us_sub' ? 'selected' : ''}>Foreign parent — US sub</option>
                        <option value="sister_companies" ${state.related_party_kind === 'sister_companies' ? 'selected' : ''}>Sister companies</option>
                        <option value="brother_sister" ${state.related_party_kind === 'brother_sister' ? 'selected' : ''}>Brother-sister (common ownership)</option>
                        <option value="ce_partnership" ${state.related_party_kind === 'ce_partnership' ? 'selected' : ''}>Partnership / LLC related party</option>
                    </select>
                </label>
                <label><span data-i18n="view.s482.label.transaction">Transaction type</span>
                    <select name="transaction_type">
                        <option value="tangible_goods" ${state.transaction_type === 'tangible_goods' ? 'selected' : ''}>Tangible goods sale</option>
                        <option value="services" ${state.transaction_type === 'services' ? 'selected' : ''}>Services</option>
                        <option value="intangibles" ${state.transaction_type === 'intangibles' ? 'selected' : ''}>Intangibles license</option>
                        <option value="cost_sharing" ${state.transaction_type === 'cost_sharing' ? 'selected' : ''}>Cost sharing arrangement</option>
                        <option value="loans" ${state.transaction_type === 'loans' ? 'selected' : ''}>Intercompany loans</option>
                        <option value="guarantees" ${state.transaction_type === 'guarantees' ? 'selected' : ''}>Guarantees</option>
                        <option value="rentals" ${state.transaction_type === 'rentals' ? 'selected' : ''}>Rentals / leases</option>
                    </select>
                </label>
                <label><span data-i18n="view.s482.label.method">Best method used</span>
                    <select name="transfer_pricing_method">
                        <option value="cup" ${state.transfer_pricing_method === 'cup' ? 'selected' : ''}>CUP (Comparable Uncontrolled Price)</option>
                        <option value="rpm" ${state.transfer_pricing_method === 'rpm' ? 'selected' : ''}>RPM (Resale Price Method)</option>
                        <option value="cpm" ${state.transfer_pricing_method === 'cpm' ? 'selected' : ''}>CPM (Comparable Profits Method)</option>
                        <option value="psm" ${state.transfer_pricing_method === 'psm' ? 'selected' : ''}>PSM (Profit Split Method)</option>
                        <option value="tnmm" ${state.transfer_pricing_method === 'tnmm' ? 'selected' : ''}>TNMM (Transactional Net Margin)</option>
                        <option value="other" ${state.transfer_pricing_method === 'other' ? 'selected' : ''}>Other / unspecified</option>
                    </select>
                </label>
                <label><span data-i18n="view.s482.label.price_used">Intercompany price used ($)</span>
                    <input type="number" step="1000" name="intercompany_price_used" value="${state.intercompany_price_used}"></label>
                <label><span data-i18n="view.s482.label.low_quartile">Arm's-length range LOW (25th %) ($)</span>
                    <input type="number" step="1000" name="arms_length_low_quartile" value="${state.arms_length_low_quartile}"></label>
                <label><span data-i18n="view.s482.label.high_quartile">Arm's-length range HIGH (75th %) ($)</span>
                    <input type="number" step="1000" name="arms_length_high_quartile" value="${state.arms_length_high_quartile}"></label>
                <label><span data-i18n="view.s482.label.volume">Transaction volume ($)</span>
                    <input type="number" step="10000" name="transaction_volume" value="${state.transaction_volume}"></label>
                <label><span data-i18n="view.s482.label.doc">Contemporaneous documentation?</span>
                    <input type="checkbox" name="has_contemporaneous_documentation" ${state.has_contemporaneous_documentation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s482.label.six_documents">6 principal documents complete?</span>
                    <input type="checkbox" name="documentation_includes_six_principal_documents" ${state.documentation_includes_six_principal_documents ? 'checked' : ''}></label>
                <label><span data-i18n="view.s482.label.best_method">Best method analysis done?</span>
                    <input type="checkbox" name="used_best_method" ${state.used_best_method ? 'checked' : ''}></label>
                <label><span data-i18n="view.s482.label.apa">Advance Pricing Agreement?</span>
                    <input type="checkbox" name="apa_in_place" ${state.apa_in_place ? 'checked' : ''}></label>
                <label><span data-i18n="view.s482.label.foreign">Foreign party country</span>
                    <input type="text" name="foreign_party_country" value="${state.foreign_party_country}"></label>
                <label><span data-i18n="view.s482.label.substantial">Substantial understatement?</span>
                    <input type="checkbox" name="is_substantial_understatement" ${state.is_substantial_understatement ? 'checked' : ''}></label>
                <label><span data-i18n="view.s482.label.gross">Gross misstatement (200%+)?</span>
                    <input type="checkbox" name="is_gross_misstatement" ${state.is_gross_misstatement ? 'checked' : ''}></label>
                <label><span data-i18n="view.s482.label.marginal">Marginal %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s482.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s482-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s482.h2.six_documents">Six principal documents (§ 1.6662-6)</h2>
            <ol class="muted small">
                <li data-i18n="view.s482.doc.overview">Overview of business: organizational structure, ownership, operations, market</li>
                <li data-i18n="view.s482.doc.org_chart">Organization chart + comp data + related party identification</li>
                <li data-i18n="view.s482.doc.functional">Functional + risk analysis</li>
                <li data-i18n="view.s482.doc.economic">Economic analysis: method selection + comparable selection</li>
                <li data-i18n="view.s482.doc.applicable_law">Applicable § 482 + Treas. Reg analysis</li>
                <li data-i18n="view.s482.doc.background">Background documents: contracts, transfer-pricing studies, agreements</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s482.h2.methods">Best method by transaction type</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s482.th.transaction">Transaction</th>
                    <th data-i18n="view.s482.th.preferred">Preferred method</th>
                </tr></thead>
                <tbody>
                    <tr><td>Tangible goods</td><td>CUP > RPM > CPM > TNMM</td></tr>
                    <tr><td>Services</td><td>CPM / TNMM > Comparable Services > Cost Plus</td></tr>
                    <tr><td>Intangibles license</td><td>CUT (Comparable Uncontrolled Transaction) > CPM with profit split</td></tr>
                    <tr><td>Cost sharing</td><td>Specified per § 1.482-7 (CSA / IDC contribution analyses)</td></tr>
                    <tr><td>Loans</td><td>CUP based on credit-rated debt + safe harbor rates</td></tr>
                    <tr><td>Distribution</td><td>RPM where reseller has limited functions</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s482.h2.apa">Advance Pricing Agreement (APA)</h2>
            <p class="muted small" data-i18n="view.s482.apa.body">
                Multi-year prospective agreement with IRS on transfer pricing methodology.
                Three types: <strong>Unilateral</strong> (IRS only), <strong>Bilateral</strong>
                (US + 1 foreign country), <strong>Multilateral</strong> (US + multiple). 5-year
                term typical, renewable. Strong protection from § 6662 penalties + foreign
                disputes (competent authority). User fee $113,500 (small business reduced).
            </p>
        </div>
    `;
    document.getElementById('s482-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.related_party_kind = fd.get('related_party_kind');
        state.transaction_type = fd.get('transaction_type');
        state.transfer_pricing_method = fd.get('transfer_pricing_method');
        state.intercompany_price_used = Number(fd.get('intercompany_price_used')) || 0;
        state.arms_length_low_quartile = Number(fd.get('arms_length_low_quartile')) || 0;
        state.arms_length_high_quartile = Number(fd.get('arms_length_high_quartile')) || 0;
        state.transaction_volume = Number(fd.get('transaction_volume')) || 0;
        state.has_contemporaneous_documentation = !!fd.get('has_contemporaneous_documentation');
        state.documentation_includes_six_principal_documents = !!fd.get('documentation_includes_six_principal_documents');
        state.used_best_method = !!fd.get('used_best_method');
        state.apa_in_place = !!fd.get('apa_in_place');
        state.foreign_party_country = fd.get('foreign_party_country') || '';
        state.is_substantial_understatement = !!fd.get('is_substantial_understatement');
        state.is_gross_misstatement = !!fd.get('is_gross_misstatement');
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.21;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s482-output');
    if (!el) return;
    const withinRange = state.intercompany_price_used >= state.arms_length_low_quartile
        && state.intercompany_price_used <= state.arms_length_high_quartile;
    let adjustment = 0;
    if (!withinRange) {
        const median = (state.arms_length_low_quartile + state.arms_length_high_quartile) / 2;
        adjustment = Math.abs(median - state.intercompany_price_used);
    }
    const taxAdjustment = adjustment * state.marginal_rate;
    let penaltyRate = 0;
    if (state.is_gross_misstatement) penaltyRate = 0.40;
    else if (state.is_substantial_understatement) penaltyRate = 0.20;
    const documentationProtects = state.has_contemporaneous_documentation
        && state.documentation_includes_six_principal_documents
        && state.used_best_method;
    const apaProtection = state.apa_in_place;
    const penalty = (documentationProtects || apaProtection) ? 0 : taxAdjustment * penaltyRate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s482.h2.result">Transfer pricing analysis</h2>
            <div class="cards">
                <div class="card ${withinRange ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s482.card.within_range">Within arm's-length range</div>
                    <div class="value">${withinRange ? esc(t('view.s482.status.yes')) : esc(t('view.s482.status.no'))}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s482.card.adjustment">Potential IRS adjustment</div>
                    <div class="value">$${adjustment.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s482.card.tax_due">Additional tax due</div>
                    <div class="value">$${taxAdjustment.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${documentationProtects || apaProtection ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s482.card.protection">Penalty protection</div>
                    <div class="value">${documentationProtects || apaProtection ? esc(t('view.s482.status.yes')) : esc(t('view.s482.status.no'))}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s482.card.penalty">Penalty exposure</div>
                    <div class="value">$${penalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
