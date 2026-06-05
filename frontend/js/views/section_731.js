// IRC § 731 — Recognition of Gain or Loss on Partnership Distribution.
// § 731(a)(1) — gain to extent cash distributed exceeds outside basis.
// § 731(a)(2) — loss only on liquidating distribution of cash + receivables + inventory.
// § 731(c) — distribution of marketable securities treated as money (post-1994).

import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    distribution_type: 'current',
    partner_outside_basis_before: 0,
    cash_distributed: 0,
    marketable_securities_fmv: 0,
    other_property_fmv: 0,
    other_property_basis: 0,
    is_liquidating: false,
    receivables_distributed: 0,
    inventory_distributed: 0,
    other_capital_property: 0,
    inside_basis_property: 0,
    gain_recognized: 0,
    loss_recognized: 0,
    s732_basis_distribution: 0,
    s731_c_securities_money: false,
    securities_gain_recognized: 0,
    s754_election: false,
    s734_b_adjustment: 0,
    s752_b_liability_relief: 0,
    s751_b_disproportionate: false,
    s751_b_exchange_amount: 0,
    s737_seven_year_lookback: false,
    s737_precontribution_gain: 0,
    s704_c_property: false,
    s704_c_built_in_gain: 0,
    s731_a_2_liquidation_loss: false,
    is_partial_liquidation: false,
};

export async function renderSection731(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s731.h1.title">// § 731 PARTNERSHIP DISTRIBUTIONS</span></h1>
        <p class="muted small" data-i18n="view.s731.hint.intro">
            <strong>§ 731(a)(1)</strong> — partner recognizes GAIN to extent CASH (incl. marketable
            securities under § 731(c)) distributed EXCEEDS outside basis. <strong>§ 731(a)(2)</strong>
            — LOSS only recognized on LIQUIDATING distribution and only if distributed property is
            cash + unrealized receivables + inventory. <strong>§ 731(c) (since 1994)</strong>
            marketable securities treated as money — to extent of FMV. <strong>§ 732</strong> sets
            partner's basis in distributed property. <strong>§ 737 7-year look-back:</strong>
            contributing partner recognizes precontribution gain if other property distributed within
            7 years of contribution. <strong>§ 751(b)</strong> disproportionate distribution of hot
            assets — treated as taxable exchange.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s731.h2.inputs">Inputs</h2>
            <form id="s731-form" class="inline-form">
                <label><span data-i18n="view.s731.label.type">Distribution type</span>
                    <select name="distribution_type">
                        <option value="current" ${state.distribution_type === 'current' ? 'selected' : ''}>Current (non-liquidating)</option>
                        <option value="liquidating" ${state.distribution_type === 'liquidating' ? 'selected' : ''}>Liquidating</option>
                        <option value="redemption" ${state.distribution_type === 'redemption' ? 'selected' : ''}>Redemption of interest</option>
                        <option value="partial_liquidation" ${state.distribution_type === 'partial_liquidation' ? 'selected' : ''}>Partial liquidation</option>
                    </select>
                </label>
                <label><span data-i18n="view.s731.label.basis_before">Outside basis before ($)</span>
                    <input type="number" step="0.01" name="partner_outside_basis_before" value="${state.partner_outside_basis_before}"></label>
                <label><span data-i18n="view.s731.label.cash">Cash distributed ($)</span>
                    <input type="number" step="0.01" name="cash_distributed" value="${state.cash_distributed}"></label>
                <label><span data-i18n="view.s731.label.securities">Marketable securities FMV ($)</span>
                    <input type="number" step="0.01" name="marketable_securities_fmv" value="${state.marketable_securities_fmv}"></label>
                <label><span data-i18n="view.s731.label.other_fmv">Other property FMV ($)</span>
                    <input type="number" step="0.01" name="other_property_fmv" value="${state.other_property_fmv}"></label>
                <label><span data-i18n="view.s731.label.other_basis">Other property basis ($)</span>
                    <input type="number" step="0.01" name="other_property_basis" value="${state.other_property_basis}"></label>
                <label><span data-i18n="view.s731.label.liquidating">Liquidating?</span>
                    <input type="checkbox" name="is_liquidating" ${state.is_liquidating ? 'checked' : ''}></label>
                <label><span data-i18n="view.s731.label.receivables">Receivables ($)</span>
                    <input type="number" step="0.01" name="receivables_distributed" value="${state.receivables_distributed}"></label>
                <label><span data-i18n="view.s731.label.inventory">Inventory ($)</span>
                    <input type="number" step="0.01" name="inventory_distributed" value="${state.inventory_distributed}"></label>
                <label><span data-i18n="view.s731.label.cap_prop">Other capital property ($)</span>
                    <input type="number" step="0.01" name="other_capital_property" value="${state.other_capital_property}"></label>
                <label><span data-i18n="view.s731.label.inside">Inside basis property ($)</span>
                    <input type="number" step="0.01" name="inside_basis_property" value="${state.inside_basis_property}"></label>
                <label><span data-i18n="view.s731.label.gain">Gain recognized ($)</span>
                    <input type="number" step="0.01" name="gain_recognized" value="${state.gain_recognized}"></label>
                <label><span data-i18n="view.s731.label.loss">Loss recognized ($)</span>
                    <input type="number" step="0.01" name="loss_recognized" value="${state.loss_recognized}"></label>
                <label><span data-i18n="view.s731.label.s732">§ 732 basis distributed ($)</span>
                    <input type="number" step="0.01" name="s732_basis_distribution" value="${state.s732_basis_distribution}"></label>
                <label><span data-i18n="view.s731.label.s731c">§ 731(c) securities = money?</span>
                    <input type="checkbox" name="s731_c_securities_money" ${state.s731_c_securities_money ? 'checked' : ''}></label>
                <label><span data-i18n="view.s731.label.sec_gain">Securities gain ($)</span>
                    <input type="number" step="0.01" name="securities_gain_recognized" value="${state.securities_gain_recognized}"></label>
                <label><span data-i18n="view.s731.label.s754">§ 754 election?</span>
                    <input type="checkbox" name="s754_election" ${state.s754_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s731.label.s734">§ 734(b) adj ($)</span>
                    <input type="number" step="0.01" name="s734_b_adjustment" value="${state.s734_b_adjustment}"></label>
                <label><span data-i18n="view.s731.label.liab">§ 752(b) liability relief ($)</span>
                    <input type="number" step="0.01" name="s752_b_liability_relief" value="${state.s752_b_liability_relief}"></label>
                <label><span data-i18n="view.s731.label.s751b">§ 751(b) disproportionate?</span>
                    <input type="checkbox" name="s751_b_disproportionate" ${state.s751_b_disproportionate ? 'checked' : ''}></label>
                <label><span data-i18n="view.s731.label.s751b_amt">§ 751(b) exchange amount ($)</span>
                    <input type="number" step="0.01" name="s751_b_exchange_amount" value="${state.s751_b_exchange_amount}"></label>
                <label><span data-i18n="view.s731.label.s737">§ 737 7-yr lookback?</span>
                    <input type="checkbox" name="s737_seven_year_lookback" ${state.s737_seven_year_lookback ? 'checked' : ''}></label>
                <label><span data-i18n="view.s731.label.precontrib">Precontribution gain ($)</span>
                    <input type="number" step="0.01" name="s737_precontribution_gain" value="${state.s737_precontribution_gain}"></label>
                <label><span data-i18n="view.s731.label.s704c">§ 704(c) property?</span>
                    <input type="checkbox" name="s704_c_property" ${state.s704_c_property ? 'checked' : ''}></label>
                <label><span data-i18n="view.s731.label.bug">§ 704(c) built-in gain ($)</span>
                    <input type="number" step="0.01" name="s704_c_built_in_gain" value="${state.s704_c_built_in_gain}"></label>
                <label><span data-i18n="view.s731.label.s731a2">§ 731(a)(2) liquidating loss?</span>
                    <input type="checkbox" name="s731_a_2_liquidation_loss" ${state.s731_a_2_liquidation_loss ? 'checked' : ''}></label>
                <label><span data-i18n="view.s731.label.partial">Partial liquidation?</span>
                    <input type="checkbox" name="is_partial_liquidation" ${state.is_partial_liquidation ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s731.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s731-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s731.h2.basic">Basic distribution rules</h2>
            <ol class="muted small">
                <li data-i18n="view.s731.basic.gain">§ 731(a)(1) — gain only to extent cash &gt; outside basis</li>
                <li data-i18n="view.s731.basic.loss">§ 731(a)(2) — loss only on LIQUIDATION + cash + receivables + inventory distributed</li>
                <li data-i18n="view.s731.basic.character">Gain character: capital (§ 741)</li>
                <li data-i18n="view.s731.basic.holding">Distributed property: holding period tacks (§ 735(b))</li>
                <li data-i18n="view.s731.basic.basis_property">Basis in distributed property: § 732</li>
                <li data-i18n="view.s731.basic.s731c_securities">§ 731(c) marketable securities treated as money (FMV)</li>
                <li data-i18n="view.s731.basic.s731c_reduction">§ 731(c)(3) reduction: pro-rata share of partnership's gain in securities</li>
                <li data-i18n="view.s731.basic.no_intermediate">No "intermediate" gain on appreciated non-cash property distributions</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s731.h2.s732">§ 732 basis of distributed property</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s731.tbl.dist_type">Distribution</th><th data-i18n="view.s731.tbl.basis_rule">Basis rule</th><th data-i18n="view.s731.tbl.citation">Citation</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s731.tbl.current">Current — non-liquidating</td><td data-i18n="view.s731.tbl.lesser">Lesser of (P/S basis in property) OR (partner outside basis)</td><td>§ 732(a)</td></tr>
                    <tr><td data-i18n="view.s731.tbl.liquidating">Liquidating</td><td data-i18n="view.s731.tbl.equal_basis">Partner outside basis allocated among property</td><td>§ 732(b)</td></tr>
                    <tr><td data-i18n="view.s731.tbl.cash_first">Cash first</td><td data-i18n="view.s731.tbl.applied">Applied against outside basis first</td><td>§ 731(a)(1)</td></tr>
                    <tr><td data-i18n="view.s731.tbl.receivables">Receivables + inventory</td><td data-i18n="view.s731.tbl.r_basis">Carryover basis (or lower of basis / FMV)</td><td>§ 732(c)(1)(A)</td></tr>
                    <tr><td data-i18n="view.s731.tbl.other">Other property</td><td data-i18n="view.s731.tbl.remaining">Remaining basis allocated by relative basis + FMV</td><td>§ 732(c)(2)</td></tr>
                    <tr><td data-i18n="view.s731.tbl.s732_e">Built-in loss (mandatory)</td><td data-i18n="view.s731.tbl.s732_e_basis">§ 732(e) basis reduction</td><td>§ 732(e)</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s731.h2.s737">§ 737 7-year look-back</h2>
            <ul class="muted small">
                <li data-i18n="view.s731.s737.purpose">Anti-abuse: contributing partner recognizes precontribution gain if other property distributed within 7 years</li>
                <li data-i18n="view.s731.s737.formula">Gain = LESSER of (precontribution gain) OR (excess of distributed FMV over outside basis)</li>
                <li data-i18n="view.s731.s737.character">Character: same as contributed property would have on partnership sale</li>
                <li data-i18n="view.s731.s737.basis_inc">Basis in distributed property: § 732 — but PLUS § 737(d) gain recognized</li>
                <li data-i18n="view.s731.s737.partner_only">Only contributing partner — not other partners</li>
                <li data-i18n="view.s731.s737.s704c_relation">Coordinates with § 704(c)(1)(B) similar gain recognition rule</li>
                <li data-i18n="view.s731.s737.7_year_clock">7-year clock starts from contribution</li>
                <li data-i18n="view.s731.s737.s721_exception">§ 721 contributions not triggering: gift, like-kind, charitable, etc.</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s731.h2.s731c">§ 731(c) marketable securities</h2>
            <ul class="muted small">
                <li data-i18n="view.s731.s731c.fmv">Distributed at FMV — treated as MONEY for § 731(a)(1)</li>
                <li data-i18n="view.s731.s731c.defined">Securities = stock, bonds, options, futures, foreign currency, derivatives</li>
                <li data-i18n="view.s731.s731c.actively_traded">"Actively traded" = OTC market or established exchange</li>
                <li data-i18n="view.s731.s731c.reduction">§ 731(c)(3) reduces FMV by partner's share of unrealized gain</li>
                <li data-i18n="view.s731.s731c.formula">Reduction = (partner's distributive share) × (PS unrealized gain in security)</li>
                <li data-i18n="view.s731.s731c.exceptions">Exception: investment partnership receiving securities (§ 731(c)(3)(C))</li>
                <li data-i18n="view.s731.s731c.s721_contributed">Securities CONTRIBUTED by partner: exempt from § 731(c) treatment</li>
                <li data-i18n="view.s731.s731c.s721_purchased">Securities PURCHASED by partnership: subject to § 731(c)</li>
                <li data-i18n="view.s731.s731c.basis_property">Partner's basis in distributed securities: § 732 (cash treatment)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s731.h2.s751b">§ 751(b) disproportionate distributions</h2>
            <ul class="muted small">
                <li data-i18n="view.s731.s751b.hot_assets">"Hot assets" = unrealized receivables + substantially appreciated inventory</li>
                <li data-i18n="view.s731.s751b.disproportionate">If partner receives MORE / LESS hot assets than pro-rata share</li>
                <li data-i18n="view.s731.s751b.exchange">Treated as TAXABLE EXCHANGE between partner + partnership</li>
                <li data-i18n="view.s731.s751b.deemed_sale">Deemed sale at FMV of disproportionate amount</li>
                <li data-i18n="view.s731.s751b.character">Ordinary income to recipient on hot assets received in excess</li>
                <li data-i18n="view.s731.s751b.capital_gain">Capital gain to recipient on cold assets received in excess</li>
                <li data-i18n="view.s731.s751b.s751a">§ 751(a) different: applies to sale of partnership interest (always)</li>
                <li data-i18n="view.s731.s751b.s7041">Complex 5-step "deemed distribution + deemed contribution" mechanics</li>
                <li data-i18n="view.s731.s751b.aggregate_basis">Partnership aggregate basis in hot assets adjusted accordingly</li>
            </ul>
        </div>
    `;
    document.getElementById('s731-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.distribution_type = fd.get('distribution_type');
        state.partner_outside_basis_before = Number(fd.get('partner_outside_basis_before')) || 0;
        state.cash_distributed = Number(fd.get('cash_distributed')) || 0;
        state.marketable_securities_fmv = Number(fd.get('marketable_securities_fmv')) || 0;
        state.other_property_fmv = Number(fd.get('other_property_fmv')) || 0;
        state.other_property_basis = Number(fd.get('other_property_basis')) || 0;
        state.is_liquidating = !!fd.get('is_liquidating');
        state.receivables_distributed = Number(fd.get('receivables_distributed')) || 0;
        state.inventory_distributed = Number(fd.get('inventory_distributed')) || 0;
        state.other_capital_property = Number(fd.get('other_capital_property')) || 0;
        state.inside_basis_property = Number(fd.get('inside_basis_property')) || 0;
        state.gain_recognized = Number(fd.get('gain_recognized')) || 0;
        state.loss_recognized = Number(fd.get('loss_recognized')) || 0;
        state.s732_basis_distribution = Number(fd.get('s732_basis_distribution')) || 0;
        state.s731_c_securities_money = !!fd.get('s731_c_securities_money');
        state.securities_gain_recognized = Number(fd.get('securities_gain_recognized')) || 0;
        state.s754_election = !!fd.get('s754_election');
        state.s734_b_adjustment = Number(fd.get('s734_b_adjustment')) || 0;
        state.s752_b_liability_relief = Number(fd.get('s752_b_liability_relief')) || 0;
        state.s751_b_disproportionate = !!fd.get('s751_b_disproportionate');
        state.s751_b_exchange_amount = Number(fd.get('s751_b_exchange_amount')) || 0;
        state.s737_seven_year_lookback = !!fd.get('s737_seven_year_lookback');
        state.s737_precontribution_gain = Number(fd.get('s737_precontribution_gain')) || 0;
        state.s704_c_property = !!fd.get('s704_c_property');
        state.s704_c_built_in_gain = Number(fd.get('s704_c_built_in_gain')) || 0;
        state.s731_a_2_liquidation_loss = !!fd.get('s731_a_2_liquidation_loss');
        state.is_partial_liquidation = !!fd.get('is_partial_liquidation');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s731-output');
    if (!el) return;
    const cash_treated_as = state.cash_distributed + (state.s731_c_securities_money ? state.marketable_securities_fmv : 0) + state.s752_b_liability_relief;
    const gain = Math.max(0, cash_treated_as - state.partner_outside_basis_before);
    const remaining_basis = Math.max(0, state.partner_outside_basis_before - cash_treated_as);
    const loss = (state.is_liquidating && state.other_property_fmv === 0 && remaining_basis > 0) ? remaining_basis : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s731.h2.result">§ 731 gain / loss</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s731.card.cash_treated">Cash treated as ($)</div><div class="value">$${cash_treated_as.toLocaleString()}</div></div>
                <div class="card ${gain > 0 ? 'neg' : 'pos'}"><div class="label" data-i18n="view.s731.card.gain">§ 731(a)(1) gain</div><div class="value">$${gain.toLocaleString()}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.s731.card.remaining">Remaining basis</div><div class="value">$${remaining_basis.toLocaleString()}</div></div>
                <div class="card ${loss > 0 ? 'pos' : ''}"><div class="label" data-i18n="view.s731.card.loss">§ 731(a)(2) loss</div><div class="value">$${loss.toLocaleString()}</div></div>
            </div>
        </div>
    `;
}
