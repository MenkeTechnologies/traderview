// IRC § 362 — Basis to Corporations on Tax-Free Transfers.
// § 362(a) — basis on § 351 / § 1032 contribution = transferor's basis + gain recognized.
// § 362(b) — basis on reorganization receipts.
// § 362(e)(2) — anti-loss-duplication: limit basis to FMV when net built-in loss + multiple properties.

import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    transaction_type: 's351_contribution',
    transferor_adjusted_basis: 0,
    fmv_at_transfer: 0,
    gain_recognized_by_transferor: 0,
    corporate_basis_in_property: 0,
    s362_a_carryover_basis: 0,
    s362_b_reorganization_basis: 0,
    s362_c_property_purchase: 0,
    s362_d_contribution_to_capital: 0,
    s362_e_2_anti_loss_duplication: false,
    net_built_in_loss: false,
    multiple_properties_transferred: false,
    aggregate_basis_excess_fmv: 0,
    s362_e_2_a_basis_limit_fmv: 0,
    s362_e_2_b_election_outside_basis_reduction: false,
    s362_e_election_made: false,
    transferor_stock_basis_after_election: 0,
    boot_received: 0,
    s357_c_liabilities_excess_basis: 0,
    s357_b_tax_avoidance: false,
    is_section_351_a: false,
    is_section_368_a_reorganization: false,
    reorg_type: 'A',
    is_section_355_spinoff: false,
    is_section_332_subsidiary_liquidation: false,
    s334_b_carryover_subsidiary: false,
    is_acquired_via_purchase_s338: false,
    s338_election: false,
    s338_h_10_election: false,
    s338_g_election: false,
    holding_period_carryover: false,
    s1223_2_tacking: false,
    s197_intangible_carryover: false,
    s197_15_year_amortization: false,
    s362_a_2_property_for_stock: false,
    s362_b_acquisition_solely_for_stock: false,
    s382_h_built_in_gain: 0,
    s382_h_built_in_loss: 0,
    aggregated_basis_in_property: 0,
    aggregated_fmv: 0,
    net_built_in_loss_amount: 0,
    is_transferor_consolidated: false,
};

export async function renderSection362(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s362.h1.title">// § 362 CORPORATE BASIS ON TAX-FREE TRANSFER</span></h1>
        <p class="muted small" data-i18n="view.s362.hint.intro">
            <strong>§ 362(a)</strong> — corp's basis in property received via § 351 (transfer to
            controlled corp) or § 1032 (contribution to capital) = TRANSFEROR'S BASIS + gain
            recognized by transferor (boot trigger). <strong>§ 362(b)</strong> — basis on
            reorganization (§ 368) acquisitions = transferor basis + recognized gain.
            <strong>§ 362(c)</strong> — non-shareholder property contributions (gov't, customer):
            ZERO basis (post-TCJA) or FMV (pre-TCJA). <strong>§ 362(d)</strong> — contribution to
            capital. <strong>§ 362(e)(2) ANTI-LOSS DUPLICATION:</strong> when AGGREGATE adjusted
            basis &gt; AGGREGATE FMV in § 351 contribution → reduce corp's basis to FMV (allocated
            pro-rata to net-loss properties). <strong>§ 362(e)(2)(C) ELECTION:</strong> alternative
            — reduce TRANSFEROR's OUTSIDE stock basis by same amount (preserves corp basis at
            carryover). <strong>§ 1223(2)</strong> holding period TACKS for property held in
            transferor's hands (capital or § 1231 asset).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s362.h2.inputs">Inputs</h2>
            <form id="s362-form" class="inline-form">
                <label><span data-i18n="view.s362.label.type">Transaction</span>
                    <select name="transaction_type">
                        <option value="s351_contribution" ${state.transaction_type === 's351_contribution' ? 'selected' : ''}>§ 351 contribution</option>
                        <option value="s368_reorganization" ${state.transaction_type === 's368_reorganization' ? 'selected' : ''}>§ 368 reorganization</option>
                        <option value="s332_sub_liquidation" ${state.transaction_type === 's332_sub_liquidation' ? 'selected' : ''}>§ 332 sub liquidation</option>
                        <option value="s355_spinoff" ${state.transaction_type === 's355_spinoff' ? 'selected' : ''}>§ 355 spin-off</option>
                        <option value="s338_election" ${state.transaction_type === 's338_election' ? 'selected' : ''}>§ 338 election</option>
                        <option value="capital_contribution" ${state.transaction_type === 'capital_contribution' ? 'selected' : ''}>Capital contribution (§ 118)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s362.label.basis">Transferor basis ($)</span>
                    <input type="number" step="0.01" name="transferor_adjusted_basis" value="${state.transferor_adjusted_basis}"></label>
                <label><span data-i18n="view.s362.label.fmv">FMV at transfer ($)</span>
                    <input type="number" step="0.01" name="fmv_at_transfer" value="${state.fmv_at_transfer}"></label>
                <label><span data-i18n="view.s362.label.gain">Gain recognized ($)</span>
                    <input type="number" step="0.01" name="gain_recognized_by_transferor" value="${state.gain_recognized_by_transferor}"></label>
                <label><span data-i18n="view.s362.label.corp_basis">Corp basis ($)</span>
                    <input type="number" step="0.01" name="corporate_basis_in_property" value="${state.corporate_basis_in_property}"></label>
                <label><span data-i18n="view.s362.label.s362a">§ 362(a) carryover ($)</span>
                    <input type="number" step="0.01" name="s362_a_carryover_basis" value="${state.s362_a_carryover_basis}"></label>
                <label><span data-i18n="view.s362.label.s362b">§ 362(b) reorg basis ($)</span>
                    <input type="number" step="0.01" name="s362_b_reorganization_basis" value="${state.s362_b_reorganization_basis}"></label>
                <label><span data-i18n="view.s362.label.s362c">§ 362(c) purchase ($)</span>
                    <input type="number" step="0.01" name="s362_c_property_purchase" value="${state.s362_c_property_purchase}"></label>
                <label><span data-i18n="view.s362.label.s362d">§ 362(d) cap contrib ($)</span>
                    <input type="number" step="0.01" name="s362_d_contribution_to_capital" value="${state.s362_d_contribution_to_capital}"></label>
                <label><span data-i18n="view.s362.label.s362e2">§ 362(e)(2) anti-loss?</span>
                    <input type="checkbox" name="s362_e_2_anti_loss_duplication" ${state.s362_e_2_anti_loss_duplication ? 'checked' : ''}></label>
                <label><span data-i18n="view.s362.label.nbil">Net built-in loss?</span>
                    <input type="checkbox" name="net_built_in_loss" ${state.net_built_in_loss ? 'checked' : ''}></label>
                <label><span data-i18n="view.s362.label.multi">Multiple properties?</span>
                    <input type="checkbox" name="multiple_properties_transferred" ${state.multiple_properties_transferred ? 'checked' : ''}></label>
                <label><span data-i18n="view.s362.label.excess">Excess basis-over-FMV ($)</span>
                    <input type="number" step="0.01" name="aggregate_basis_excess_fmv" value="${state.aggregate_basis_excess_fmv}"></label>
                <label><span data-i18n="view.s362.label.s362e2a">§ 362(e)(2)(A) limit ($)</span>
                    <input type="number" step="0.01" name="s362_e_2_a_basis_limit_fmv" value="${state.s362_e_2_a_basis_limit_fmv}"></label>
                <label><span data-i18n="view.s362.label.s362e2b">§ 362(e)(2)(B) election?</span>
                    <input type="checkbox" name="s362_e_2_b_election_outside_basis_reduction" ${state.s362_e_2_b_election_outside_basis_reduction ? 'checked' : ''}></label>
                <label><span data-i18n="view.s362.label.elect">Election made?</span>
                    <input type="checkbox" name="s362_e_election_made" ${state.s362_e_election_made ? 'checked' : ''}></label>
                <label><span data-i18n="view.s362.label.outside_after">Transferor stock basis after ($)</span>
                    <input type="number" step="0.01" name="transferor_stock_basis_after_election" value="${state.transferor_stock_basis_after_election}"></label>
                <label><span data-i18n="view.s362.label.boot">Boot received ($)</span>
                    <input type="number" step="0.01" name="boot_received" value="${state.boot_received}"></label>
                <label><span data-i18n="view.s362.label.s357c">§ 357(c) excess ($)</span>
                    <input type="number" step="0.01" name="s357_c_liabilities_excess_basis" value="${state.s357_c_liabilities_excess_basis}"></label>
                <label><span data-i18n="view.s362.label.s357b">§ 357(b) avoidance?</span>
                    <input type="checkbox" name="s357_b_tax_avoidance" ${state.s357_b_tax_avoidance ? 'checked' : ''}></label>
                <label><span data-i18n="view.s362.label.s351a">§ 351(a)?</span>
                    <input type="checkbox" name="is_section_351_a" ${state.is_section_351_a ? 'checked' : ''}></label>
                <label><span data-i18n="view.s362.label.s368">§ 368(a) reorg?</span>
                    <input type="checkbox" name="is_section_368_a_reorganization" ${state.is_section_368_a_reorganization ? 'checked' : ''}></label>
                <label><span data-i18n="view.s362.label.reorg_type">Reorg type</span>
                    <select name="reorg_type">
                        <option value="A" ${state.reorg_type === 'A' ? 'selected' : ''}>A (merger)</option>
                        <option value="B" ${state.reorg_type === 'B' ? 'selected' : ''}>B (stock-for-stock)</option>
                        <option value="C" ${state.reorg_type === 'C' ? 'selected' : ''}>C (stock-for-assets)</option>
                        <option value="D" ${state.reorg_type === 'D' ? 'selected' : ''}>D (divisive)</option>
                        <option value="E" ${state.reorg_type === 'E' ? 'selected' : ''}>E (recapitalization)</option>
                        <option value="F" ${state.reorg_type === 'F' ? 'selected' : ''}>F (identity change)</option>
                        <option value="G" ${state.reorg_type === 'G' ? 'selected' : ''}>G (bankruptcy)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s362.label.s355">§ 355 spin-off?</span>
                    <input type="checkbox" name="is_section_355_spinoff" ${state.is_section_355_spinoff ? 'checked' : ''}></label>
                <label><span data-i18n="view.s362.label.s332">§ 332 sub liq?</span>
                    <input type="checkbox" name="is_section_332_subsidiary_liquidation" ${state.is_section_332_subsidiary_liquidation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s362.label.s334b">§ 334(b) carryover?</span>
                    <input type="checkbox" name="s334_b_carryover_subsidiary" ${state.s334_b_carryover_subsidiary ? 'checked' : ''}></label>
                <label><span data-i18n="view.s362.label.s338_acq">§ 338 acquired?</span>
                    <input type="checkbox" name="is_acquired_via_purchase_s338" ${state.is_acquired_via_purchase_s338 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s362.label.s338_elect">§ 338 election?</span>
                    <input type="checkbox" name="s338_election" ${state.s338_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s362.label.s338h10">§ 338(h)(10)?</span>
                    <input type="checkbox" name="s338_h_10_election" ${state.s338_h_10_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s362.label.s338g">§ 338(g)?</span>
                    <input type="checkbox" name="s338_g_election" ${state.s338_g_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s362.label.tack">Holding period tacks?</span>
                    <input type="checkbox" name="holding_period_carryover" ${state.holding_period_carryover ? 'checked' : ''}></label>
                <label><span data-i18n="view.s362.label.s1223_2">§ 1223(2)?</span>
                    <input type="checkbox" name="s1223_2_tacking" ${state.s1223_2_tacking ? 'checked' : ''}></label>
                <label><span data-i18n="view.s362.label.s197">§ 197 intangible?</span>
                    <input type="checkbox" name="s197_intangible_carryover" ${state.s197_intangible_carryover ? 'checked' : ''}></label>
                <label><span data-i18n="view.s362.label.s197_15">§ 197 15-yr?</span>
                    <input type="checkbox" name="s197_15_year_amortization" ${state.s197_15_year_amortization ? 'checked' : ''}></label>
                <label><span data-i18n="view.s362.label.s362a2">§ 362(a)(2) property/stock?</span>
                    <input type="checkbox" name="s362_a_2_property_for_stock" ${state.s362_a_2_property_for_stock ? 'checked' : ''}></label>
                <label><span data-i18n="view.s362.label.s362b_solely">§ 362(b) solely stock?</span>
                    <input type="checkbox" name="s362_b_acquisition_solely_for_stock" ${state.s362_b_acquisition_solely_for_stock ? 'checked' : ''}></label>
                <label><span data-i18n="view.s362.label.s382h_gain">§ 382(h) gain ($)</span>
                    <input type="number" step="0.01" name="s382_h_built_in_gain" value="${state.s382_h_built_in_gain}"></label>
                <label><span data-i18n="view.s362.label.s382h_loss">§ 382(h) loss ($)</span>
                    <input type="number" step="0.01" name="s382_h_built_in_loss" value="${state.s382_h_built_in_loss}"></label>
                <label><span data-i18n="view.s362.label.agg_basis">Agg basis ($)</span>
                    <input type="number" step="0.01" name="aggregated_basis_in_property" value="${state.aggregated_basis_in_property}"></label>
                <label><span data-i18n="view.s362.label.agg_fmv">Agg FMV ($)</span>
                    <input type="number" step="0.01" name="aggregated_fmv" value="${state.aggregated_fmv}"></label>
                <label><span data-i18n="view.s362.label.nbil_amt">NBIL amount ($)</span>
                    <input type="number" step="0.01" name="net_built_in_loss_amount" value="${state.net_built_in_loss_amount}"></label>
                <label><span data-i18n="view.s362.label.consol">Consolidated transferor?</span>
                    <input type="checkbox" name="is_transferor_consolidated" ${state.is_transferor_consolidated ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s362.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s362-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s362.h2.basis_rules">Basis rules by transaction</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s362.tbl.tx">Transaction</th><th data-i18n="view.s362.tbl.basis_formula">Basis formula</th><th data-i18n="view.s362.tbl.citation">Citation</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s362.tbl.s351">§ 351 contribution</td><td data-i18n="view.s362.tbl.s351_formula">Transferor basis + gain recognized − § 362(e)(2) adjustment</td><td>§ 362(a)</td></tr>
                    <tr><td data-i18n="view.s362.tbl.s368">§ 368 reorganization</td><td data-i18n="view.s362.tbl.s368_formula">Transferor basis + gain recognized</td><td>§ 362(b)</td></tr>
                    <tr><td data-i18n="view.s362.tbl.s332">§ 332 sub liquidation</td><td data-i18n="view.s362.tbl.s332_formula">Carryover from liquidating sub</td><td>§ 334(b)</td></tr>
                    <tr><td data-i18n="view.s362.tbl.s355">§ 355 spin-off</td><td data-i18n="view.s362.tbl.s355_formula">Same as § 362(b) for distributing corp</td><td>§ 362(b)</td></tr>
                    <tr><td data-i18n="view.s362.tbl.purchase">Purchase</td><td data-i18n="view.s362.tbl.purchase_formula">Cost (§ 1012)</td><td>§ 1012</td></tr>
                    <tr><td data-i18n="view.s362.tbl.s338">§ 338 election</td><td data-i18n="view.s362.tbl.s338_formula">FMV (deemed asset sale + repurchase)</td><td>§ 338(b)</td></tr>
                    <tr><td data-i18n="view.s362.tbl.non_shareholder">Non-shareholder contribution</td><td data-i18n="view.s362.tbl.non_shareholder_formula">$0 (post-TCJA) / FMV (pre-TCJA)</td><td>§ 362(c) / § 118(b)(2)</td></tr>
                    <tr><td data-i18n="view.s362.tbl.bargain_purchase">Bargain purchase</td><td>FMV (Reg § 1.1032-1)</td><td>§ 1032</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s362.h2.s362e2">§ 362(e)(2) anti-loss-duplication</h2>
            <ol class="muted small">
                <li data-i18n="view.s362.e2.purpose">Anti-abuse: prevents shifting same loss to both transferor stock + corp asset basis</li>
                <li data-i18n="view.s362.e2.applies">Applies when AGGREGATE adjusted basis of contributed property &gt; AGGREGATE FMV</li>
                <li data-i18n="view.s362.e2.NBIL">"Net built-in loss" — sum of (basis − FMV) on each property &gt; 0</li>
                <li data-i18n="view.s362.e2.default">DEFAULT (no election): reduce CORP's basis to AGGREGATE FMV — pro-rata across NBIL properties only</li>
                <li data-i18n="view.s362.e2.election">§ 362(e)(2)(C) ELECTION: reduce TRANSFEROR's OUTSIDE stock basis instead → preserves corp basis at carryover</li>
                <li data-i18n="view.s362.e2.timing">Election made by ATTACHING STATEMENT to timely filed return for year of transfer</li>
                <li data-i18n="view.s362.e2.both_consent">Both transferor + corporation consent required</li>
                <li data-i18n="view.s362.e2.benefit">Election benefit: preserves loss inside corp where it may be realized later</li>
                <li data-i18n="view.s362.e2.s382">Election may HURT later if ownership change triggers § 382 limitation</li>
                <li data-i18n="view.s362.e2.binding">Election BINDING — cannot be revoked except with IRS consent</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s362.h2.holding_period">Holding period rules</h2>
            <ul class="muted small">
                <li data-i18n="view.s362.hp.s1223_2">§ 1223(2) — corp's holding period TACKS to transferor's</li>
                <li data-i18n="view.s362.hp.capital_only">Tacking applies to CAPITAL assets + § 1231 property (not inventory/ord-income)</li>
                <li data-i18n="view.s362.hp.s1223_1">§ 1223(1) — transferor's stock basis tacks to property</li>
                <li data-i18n="view.s362.hp.inventory">Inventory: NO tacking (becomes ordinary income to corp)</li>
                <li data-i18n="view.s362.hp.s1245_dep">§ 1245 / § 1250 recapture potential CARRIES OVER</li>
                <li data-i18n="view.s362.hp.s197_intangibles">§ 197 intangibles: carryover basis + remaining 15-yr amortization</li>
                <li data-i18n="view.s362.hp.s368_basis">§ 368 reorganization: stock-for-stock — shareholder tacks</li>
                <li data-i18n="view.s362.hp.s355">§ 355 spin-off: tacking applies for distributing + controlled corps</li>
                <li data-i18n="view.s362.hp.boot_property">Boot property: FMV basis + start of new holding period</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s362.h2.s338">§ 338 deemed asset purchase</h2>
            <ul class="muted small">
                <li data-i18n="view.s362.s338.purpose">Stock acquisition treated as DEEMED ASSET ACQUISITION</li>
                <li data-i18n="view.s362.s338.qualified">"Qualified stock purchase": 80%+ purchased within 12 months</li>
                <li data-i18n="view.s362.s338.g">§ 338(g): buyer-only election — buyer pays double layer of tax (buyer + seller)</li>
                <li data-i18n="view.s362.s338.h10">§ 338(h)(10): joint election — seller treats as asset sale (S-corp + consolidated)</li>
                <li data-i18n="view.s362.s338.basis_FMV">Stock cost ALLOCATED to target's assets via residual method (§ 338(b))</li>
                <li data-i18n="view.s362.s338.s197_15yr">Goodwill + § 197 intangibles: 15-year amortization</li>
                <li data-i18n="view.s362.s338.depreciation">Stepped-up basis: enhanced depreciation + amortization for buyer</li>
                <li data-i18n="view.s362.s338.election_8023">Form 8023 — § 338 election (within 8.5 months of QSP)</li>
                <li data-i18n="view.s362.s338.s336e">§ 336(e) analogous election for non-corporate seller</li>
                <li data-i18n="view.s362.s338.foreign">Foreign targets: § 338(g) election to step up basis (no seller side)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s362.h2.reorganizations">§ 368 reorganization types</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s362.tbl.type">Type</th><th data-i18n="view.s362.tbl.description">Description</th><th data-i18n="view.s362.tbl.s362b">§ 362(b) basis</th></tr></thead>
                <tbody>
                    <tr><td>A</td><td data-i18n="view.s362.tbl.a">Statutory merger / consolidation</td><td data-i18n="view.s362.tbl.a_basis">Acquiring takes target's basis</td></tr>
                    <tr><td>B</td><td data-i18n="view.s362.tbl.b">Stock-for-stock (80% control + solely voting)</td><td data-i18n="view.s362.tbl.b_basis">Target stock basis carries</td></tr>
                    <tr><td>C</td><td data-i18n="view.s362.tbl.c">Stock-for-assets (substantially all)</td><td data-i18n="view.s362.tbl.c_basis">Asset basis carries</td></tr>
                    <tr><td>D</td><td data-i18n="view.s362.tbl.d">Divisive (spin-off / split-up) — also § 355</td><td data-i18n="view.s362.tbl.d_basis">Carryover both sides</td></tr>
                    <tr><td>E</td><td data-i18n="view.s362.tbl.e">Recapitalization (capital structure change)</td><td data-i18n="view.s362.tbl.e_basis">Basis preserved</td></tr>
                    <tr><td>F</td><td data-i18n="view.s362.tbl.f">Identity / form / place change (1 corp only)</td><td data-i18n="view.s362.tbl.f_basis">No basis change</td></tr>
                    <tr><td>G</td><td data-i18n="view.s362.tbl.g">Bankruptcy reorganization (Chapter 11)</td><td data-i18n="view.s362.tbl.g_basis">Carryover</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s362.h2.related">Related provisions</h2>
            <ul class="muted small">
                <li data-i18n="view.s362.rel.s351">§ 351 — transfer to controlled corporation (80%+)</li>
                <li data-i18n="view.s362.rel.s358">§ 358 — basis of shareholder's stock (outside basis)</li>
                <li data-i18n="view.s362.rel.s357">§ 357 — assumption of liabilities</li>
                <li data-i18n="view.s362.rel.s368">§ 368 — reorganization definitions</li>
                <li data-i18n="view.s362.rel.s361">§ 361 — corporate-level reorganization basis</li>
                <li data-i18n="view.s362.rel.s355">§ 355 — spin-off / split-up</li>
                <li data-i18n="view.s362.rel.s332">§ 332 — subsidiary liquidation</li>
                <li data-i18n="view.s362.rel.s334">§ 334 — basis of sub's property in liquidation</li>
                <li data-i18n="view.s362.rel.s338">§ 338 — election to treat stock as asset purchase</li>
                <li data-i18n="view.s362.rel.s382">§ 382 — NOL limitation after ownership change</li>
                <li data-i18n="view.s362.rel.s1032">§ 1032 — corp not taxable on its own stock issuance</li>
                <li data-i18n="view.s362.rel.s1223">§ 1223 — holding period tacking</li>
                <li data-i18n="view.s362.rel.s197">§ 197 — 15-year amortization of intangibles</li>
            </ul>
        </div>
    `;
    document.getElementById('s362-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.transaction_type = fd.get('transaction_type');
        state.transferor_adjusted_basis = Number(fd.get('transferor_adjusted_basis')) || 0;
        state.fmv_at_transfer = Number(fd.get('fmv_at_transfer')) || 0;
        state.gain_recognized_by_transferor = Number(fd.get('gain_recognized_by_transferor')) || 0;
        state.corporate_basis_in_property = Number(fd.get('corporate_basis_in_property')) || 0;
        state.s362_a_carryover_basis = Number(fd.get('s362_a_carryover_basis')) || 0;
        state.s362_b_reorganization_basis = Number(fd.get('s362_b_reorganization_basis')) || 0;
        state.s362_c_property_purchase = Number(fd.get('s362_c_property_purchase')) || 0;
        state.s362_d_contribution_to_capital = Number(fd.get('s362_d_contribution_to_capital')) || 0;
        state.s362_e_2_anti_loss_duplication = !!fd.get('s362_e_2_anti_loss_duplication');
        state.net_built_in_loss = !!fd.get('net_built_in_loss');
        state.multiple_properties_transferred = !!fd.get('multiple_properties_transferred');
        state.aggregate_basis_excess_fmv = Number(fd.get('aggregate_basis_excess_fmv')) || 0;
        state.s362_e_2_a_basis_limit_fmv = Number(fd.get('s362_e_2_a_basis_limit_fmv')) || 0;
        state.s362_e_2_b_election_outside_basis_reduction = !!fd.get('s362_e_2_b_election_outside_basis_reduction');
        state.s362_e_election_made = !!fd.get('s362_e_election_made');
        state.transferor_stock_basis_after_election = Number(fd.get('transferor_stock_basis_after_election')) || 0;
        state.boot_received = Number(fd.get('boot_received')) || 0;
        state.s357_c_liabilities_excess_basis = Number(fd.get('s357_c_liabilities_excess_basis')) || 0;
        state.s357_b_tax_avoidance = !!fd.get('s357_b_tax_avoidance');
        state.is_section_351_a = !!fd.get('is_section_351_a');
        state.is_section_368_a_reorganization = !!fd.get('is_section_368_a_reorganization');
        state.reorg_type = fd.get('reorg_type');
        state.is_section_355_spinoff = !!fd.get('is_section_355_spinoff');
        state.is_section_332_subsidiary_liquidation = !!fd.get('is_section_332_subsidiary_liquidation');
        state.s334_b_carryover_subsidiary = !!fd.get('s334_b_carryover_subsidiary');
        state.is_acquired_via_purchase_s338 = !!fd.get('is_acquired_via_purchase_s338');
        state.s338_election = !!fd.get('s338_election');
        state.s338_h_10_election = !!fd.get('s338_h_10_election');
        state.s338_g_election = !!fd.get('s338_g_election');
        state.holding_period_carryover = !!fd.get('holding_period_carryover');
        state.s1223_2_tacking = !!fd.get('s1223_2_tacking');
        state.s197_intangible_carryover = !!fd.get('s197_intangible_carryover');
        state.s197_15_year_amortization = !!fd.get('s197_15_year_amortization');
        state.s362_a_2_property_for_stock = !!fd.get('s362_a_2_property_for_stock');
        state.s362_b_acquisition_solely_for_stock = !!fd.get('s362_b_acquisition_solely_for_stock');
        state.s382_h_built_in_gain = Number(fd.get('s382_h_built_in_gain')) || 0;
        state.s382_h_built_in_loss = Number(fd.get('s382_h_built_in_loss')) || 0;
        state.aggregated_basis_in_property = Number(fd.get('aggregated_basis_in_property')) || 0;
        state.aggregated_fmv = Number(fd.get('aggregated_fmv')) || 0;
        state.net_built_in_loss_amount = Number(fd.get('net_built_in_loss_amount')) || 0;
        state.is_transferor_consolidated = !!fd.get('is_transferor_consolidated');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s362-output');
    if (!el) return;
    const carryover_basis = state.transferor_adjusted_basis + state.gain_recognized_by_transferor;
    const nbil = state.aggregated_basis_in_property - state.aggregated_fmv;
    const has_nbil = nbil > 0;
    const final_basis = has_nbil && !state.s362_e_election_made ? state.aggregated_fmv : carryover_basis;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s362.h2.result">§ 362 corp basis</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s362.card.transferor_basis">Transferor basis</div><div class="value">$${state.transferor_adjusted_basis.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s362.card.gain">+ Gain recognized</div><div class="value">$${state.gain_recognized_by_transferor.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s362.card.carryover">Carryover basis</div><div class="value">$${carryover_basis.toLocaleString()}</div></div>
                <div class="card ${has_nbil ? 'warn' : 'pos'}"><div class="label" data-i18n="view.s362.card.nbil">NBIL?</div><div class="value">${has_nbil ? 'YES ($'+nbil.toLocaleString()+')' : 'NO'}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.s362.card.final">Final corp basis</div><div class="value">$${final_basis.toLocaleString()}</div></div>
            </div>
        </div>
    `;
}
