// IRC § 351 — Transfer to Controlled Corporation (Tax-Free Reorganization).
// Nonrecognition of gain/loss when one or more persons transfer property to corporation in
// exchange SOLELY for stock + (immediately after) transferors collectively control ≥ 80%.
// § 358 carryover basis + § 362 carryover basis for property to corp.

import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    transferor_count: 1,
    property_fmv_transferred: 0,
    property_basis_transferred: 0,
    stock_received_fmv: 0,
    boot_received: 0,
    boot_type: 'cash',
    liabilities_assumed_by_corp: 0,
    control_post_transfer_pct: 80,
    has_section_357c_excess: false,
    s357_c_excess_amount: 0,
    s357_c_tax_avoidance: false,
    is_disqualified_property: false,
    services_in_exchange: false,
    services_value: 0,
    s351g_nq_preferred_stock: false,
    nqps_value: 0,
    is_existing_corp: false,
    affected_by_invested_company_rule: false,
    s721_partnership: false,
    s368_reorganization: false,
    s355_spinoff: false,
    momentary_control_doctrine: false,
};

export async function renderSection351(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s351.h1.title">// § 351 TAX-FREE TRANSFER TO CONTROLLED CORP</span></h1>
        <p class="muted small" data-i18n="view.s351.hint.intro">
            <strong>Nonrecognition of gain/loss</strong> when person(s) transfer property to corporation
            in exchange SOLELY for stock + (immediately after) transferor(s) collectively control
            <strong>≥ 80%</strong> (combined voting + total). <strong>Boot recognition:</strong>
            recognize gain to extent of cash + other property received (lesser of gain realized OR
            boot). <strong>Liabilities (§ 357):</strong> general rule no gain, but (b) tax-avoidance
            purpose treats as boot, (c) excess over basis = gain recognized. <strong>§ 358 outside
            basis:</strong> stock basis = property basis + gain recognized − boot − liabilities.
            <strong>§ 362 inside basis:</strong> corporate basis = transferor basis + gain recognized
            (limited to FMV under § 362(e)).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s351.h2.inputs">Inputs</h2>
            <form id="s351-form" class="inline-form">
                <label><span data-i18n="view.s351.label.count">Transferor count</span>
                    <input type="number" step="1" name="transferor_count" value="${state.transferor_count}"></label>
                <label><span data-i18n="view.s351.label.fmv">Property FMV transferred ($)</span>
                    <input type="number" step="0.01" name="property_fmv_transferred" value="${state.property_fmv_transferred}"></label>
                <label><span data-i18n="view.s351.label.basis">Property basis ($)</span>
                    <input type="number" step="0.01" name="property_basis_transferred" value="${state.property_basis_transferred}"></label>
                <label><span data-i18n="view.s351.label.stock">Stock received FMV ($)</span>
                    <input type="number" step="0.01" name="stock_received_fmv" value="${state.stock_received_fmv}"></label>
                <label><span data-i18n="view.s351.label.boot">Boot received ($)</span>
                    <input type="number" step="0.01" name="boot_received" value="${state.boot_received}"></label>
                <label><span data-i18n="view.s351.label.boot_type">Boot type</span>
                    <select name="boot_type">
                        <option value="cash" ${state.boot_type === 'cash' ? 'selected' : ''}>Cash</option>
                        <option value="property" ${state.boot_type === 'property' ? 'selected' : ''}>Other property</option>
                        <option value="securities" ${state.boot_type === 'securities' ? 'selected' : ''}>Securities (NOT stock)</option>
                        <option value="nqps" ${state.boot_type === 'nqps' ? 'selected' : ''}>NQPS (§ 351(g))</option>
                    </select>
                </label>
                <label><span data-i18n="view.s351.label.liabilities">Liabilities assumed by corp ($)</span>
                    <input type="number" step="0.01" name="liabilities_assumed_by_corp" value="${state.liabilities_assumed_by_corp}"></label>
                <label><span data-i18n="view.s351.label.control">Control % post-transfer</span>
                    <input type="number" step="0.1" name="control_post_transfer_pct" value="${state.control_post_transfer_pct}"></label>
                <label><span data-i18n="view.s351.label.s357c">§ 357(c) excess?</span>
                    <input type="checkbox" name="has_section_357c_excess" ${state.has_section_357c_excess ? 'checked' : ''}></label>
                <label><span data-i18n="view.s351.label.s357c_amt">§ 357(c) excess ($)</span>
                    <input type="number" step="0.01" name="s357_c_excess_amount" value="${state.s357_c_excess_amount}"></label>
                <label><span data-i18n="view.s351.label.s357c_avoidance">§ 357(b) tax avoidance?</span>
                    <input type="checkbox" name="s357_c_tax_avoidance" ${state.s357_c_tax_avoidance ? 'checked' : ''}></label>
                <label><span data-i18n="view.s351.label.disqualified">Disqualified property?</span>
                    <input type="checkbox" name="is_disqualified_property" ${state.is_disqualified_property ? 'checked' : ''}></label>
                <label><span data-i18n="view.s351.label.services">Services in exchange?</span>
                    <input type="checkbox" name="services_in_exchange" ${state.services_in_exchange ? 'checked' : ''}></label>
                <label><span data-i18n="view.s351.label.services_val">Services value ($)</span>
                    <input type="number" step="0.01" name="services_value" value="${state.services_value}"></label>
                <label><span data-i18n="view.s351.label.nqps">§ 351(g) NQPS?</span>
                    <input type="checkbox" name="s351g_nq_preferred_stock" ${state.s351g_nq_preferred_stock ? 'checked' : ''}></label>
                <label><span data-i18n="view.s351.label.nqps_val">NQPS value ($)</span>
                    <input type="number" step="0.01" name="nqps_value" value="${state.nqps_value}"></label>
                <label><span data-i18n="view.s351.label.existing">Existing corp?</span>
                    <input type="checkbox" name="is_existing_corp" ${state.is_existing_corp ? 'checked' : ''}></label>
                <label><span data-i18n="view.s351.label.invested">Affected by investment company rule?</span>
                    <input type="checkbox" name="affected_by_invested_company_rule" ${state.affected_by_invested_company_rule ? 'checked' : ''}></label>
                <label><span data-i18n="view.s351.label.s721">§ 721 partnership?</span>
                    <input type="checkbox" name="s721_partnership" ${state.s721_partnership ? 'checked' : ''}></label>
                <label><span data-i18n="view.s351.label.s368">§ 368 reorganization?</span>
                    <input type="checkbox" name="s368_reorganization" ${state.s368_reorganization ? 'checked' : ''}></label>
                <label><span data-i18n="view.s351.label.s355">§ 355 spin-off?</span>
                    <input type="checkbox" name="s355_spinoff" ${state.s355_spinoff ? 'checked' : ''}></label>
                <label><span data-i18n="view.s351.label.momentary">Momentary control doctrine?</span>
                    <input type="checkbox" name="momentary_control_doctrine" ${state.momentary_control_doctrine ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s351.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s351-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s351.h2.requirements">3 statutory requirements</h2>
            <ol class="muted small">
                <li data-i18n="view.s351.req.property">Transferor transfers PROPERTY (NOT services)</li>
                <li data-i18n="view.s351.req.stock">Receives ONLY STOCK in exchange (boot allowed but recognized)</li>
                <li data-i18n="view.s351.req.control">Transferor(s) immediately after exchange CONTROL 80%+ (voting + total)</li>
                <li data-i18n="view.s351.req.s368c">§ 368(c) control = 80% of voting + 80% of each non-voting class</li>
                <li data-i18n="view.s351.req.transferors_aggregate">"Transferors" aggregate — all who transfer in single transaction</li>
                <li data-i18n="view.s351.req.related_steps">Related step transactions: integrated under § 351 (Rev. Rul. 70-140)</li>
                <li data-i18n="view.s351.req.s368b">§ 368(b) — single class of stock not required (vs S-corp)</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s351.h2.boot">Boot recognition (§ 351(b))</h2>
            <ul class="muted small">
                <li data-i18n="view.s351.boot.cash">Cash is boot — recognize gain up to boot amount</li>
                <li data-i18n="view.s351.boot.property">Other property = boot — basis carries over</li>
                <li data-i18n="view.s351.boot.securities">Securities (notes / bonds) treated as boot under § 354(a)(2)</li>
                <li data-i18n="view.s351.boot.short_term">Short-term debt instruments treated as cash</li>
                <li data-i18n="view.s351.boot.nqps">§ 351(g) NQPS = boot — even though "stock" form</li>
                <li data-i18n="view.s351.boot.gain_limit">Recognize LESSER of (gain realized) OR (boot received)</li>
                <li data-i18n="view.s351.boot.no_loss">NO loss recognition under § 351 (separate rule)</li>
                <li data-i18n="view.s351.boot.allocation">Multiple properties: allocate boot proportionally by FMV</li>
                <li data-i18n="view.s351.boot.character">Boot gain character = character of property (cap vs ordinary)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s351.h2.liabilities">Liability assumption (§ 357)</h2>
            <ul class="muted small">
                <li data-i18n="view.s351.liab.general">§ 357(a) general: liability assumption ≠ boot</li>
                <li data-i18n="view.s351.liab.s357b">§ 357(b) tax avoidance / non-business purpose: liab = boot</li>
                <li data-i18n="view.s351.liab.s357c">§ 357(c) liabilities exceed basis: gain to extent of excess</li>
                <li data-i18n="view.s351.liab.s357c_2">§ 357(c)(2) - § 736 / § 461(h) economic performance — disregarded liabilities</li>
                <li data-i18n="view.s351.liab.s357c_3">§ 357(c)(3) deductible liabilities (A/P, accrued exp) NOT counted in liab</li>
                <li data-i18n="view.s351.liab.s357d">§ 357(d) — recourse vs nonrecourse — only deemed assumed if economically borne</li>
                <li data-i18n="view.s351.liab.s358d">§ 358(d) treat assumed liability as money received for basis (for stock)</li>
                <li data-i18n="view.s351.liab.s362e_1">§ 362(e)(1) limited to FMV for net built-in loss property</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s351.h2.basis">Basis rules</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s351.tbl.party">Party</th><th data-i18n="view.s351.tbl.formula">Basis formula</th><th data-i18n="view.s351.tbl.citation">Citation</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s351.tbl.transferor">Transferor's stock</td><td data-i18n="view.s351.tbl.t_formula">Property basis + gain recognized − boot − liabilities</td><td>§ 358(a)</td></tr>
                    <tr><td data-i18n="view.s351.tbl.corp">Corporation's property</td><td data-i18n="view.s351.tbl.corp_formula">Transferor basis + gain recognized (limited FMV)</td><td>§ 362(a)</td></tr>
                    <tr><td data-i18n="view.s351.tbl.holding">Transferor's holding period</td><td data-i18n="view.s351.tbl.tacks">Tacks (§ 1223(1))</td><td>§ 1223(1)</td></tr>
                    <tr><td data-i18n="view.s351.tbl.s362e">§ 362(e) net loss</td><td data-i18n="view.s351.tbl.fmv">Limited to FMV of property</td><td>§ 362(e)</td></tr>
                    <tr><td data-i18n="view.s351.tbl.boot_received">Boot received</td><td data-i18n="view.s351.tbl.boot_basis">FMV at exchange</td><td>§ 1012</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s351.h2.related">Related provisions</h2>
            <ul class="muted small">
                <li data-i18n="view.s351.rel.s721">§ 721 — analogue for partnership formation</li>
                <li data-i18n="view.s351.rel.s368">§ 368 — corporate reorganizations (A/B/C/D/E/F/G)</li>
                <li data-i18n="view.s351.rel.s355">§ 355 — spin-offs / split-offs</li>
                <li data-i18n="view.s351.rel.s361">§ 361 — corporation level on reorganizations</li>
                <li data-i18n="view.s351.rel.s362e2">§ 362(e)(2) anti-loss-duplication (post-2004)</li>
                <li data-i18n="view.s351.rel.s7701l">§ 7701(l) anti-conduit for cross-border § 351 with foreign acquirer</li>
                <li data-i18n="view.s351.rel.s367a">§ 367(a) outbound transfers to foreign corp — gain recognition</li>
                <li data-i18n="view.s351.rel.s897">§ 897 FIRPTA — US real property to foreign corp</li>
                <li data-i18n="view.s351.rel.s269">§ 269 anti-abuse — tax avoidance acquisition</li>
            </ul>
        </div>
    `;
    document.getElementById('s351-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.transferor_count = Number(fd.get('transferor_count')) || 0;
        state.property_fmv_transferred = Number(fd.get('property_fmv_transferred')) || 0;
        state.property_basis_transferred = Number(fd.get('property_basis_transferred')) || 0;
        state.stock_received_fmv = Number(fd.get('stock_received_fmv')) || 0;
        state.boot_received = Number(fd.get('boot_received')) || 0;
        state.boot_type = fd.get('boot_type');
        state.liabilities_assumed_by_corp = Number(fd.get('liabilities_assumed_by_corp')) || 0;
        state.control_post_transfer_pct = Number(fd.get('control_post_transfer_pct')) || 0;
        state.has_section_357c_excess = !!fd.get('has_section_357c_excess');
        state.s357_c_excess_amount = Number(fd.get('s357_c_excess_amount')) || 0;
        state.s357_c_tax_avoidance = !!fd.get('s357_c_tax_avoidance');
        state.is_disqualified_property = !!fd.get('is_disqualified_property');
        state.services_in_exchange = !!fd.get('services_in_exchange');
        state.services_value = Number(fd.get('services_value')) || 0;
        state.s351g_nq_preferred_stock = !!fd.get('s351g_nq_preferred_stock');
        state.nqps_value = Number(fd.get('nqps_value')) || 0;
        state.is_existing_corp = !!fd.get('is_existing_corp');
        state.affected_by_invested_company_rule = !!fd.get('affected_by_invested_company_rule');
        state.s721_partnership = !!fd.get('s721_partnership');
        state.s368_reorganization = !!fd.get('s368_reorganization');
        state.s355_spinoff = !!fd.get('s355_spinoff');
        state.momentary_control_doctrine = !!fd.get('momentary_control_doctrine');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s351-output');
    if (!el) return;
    const gain_realized = state.property_fmv_transferred - state.property_basis_transferred;
    const gain_recognized = Math.min(Math.max(0, gain_realized), state.boot_received + state.s357_c_excess_amount);
    const stock_basis = state.property_basis_transferred + gain_recognized - state.boot_received - state.liabilities_assumed_by_corp;
    const corp_basis = state.property_basis_transferred + gain_recognized;
    const control_satisfied = state.control_post_transfer_pct >= 80;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s351.h2.result">§ 351 result</h2>
            <div class="cards">
                <div class="card ${control_satisfied ? 'pos' : 'neg'}"><div class="label" data-i18n="view.s351.card.control">Control ≥ 80%?</div><div class="value">${control_satisfied ? 'YES' : 'NO (FAIL)'}</div></div>
                <div class="card"><div class="label" data-i18n="view.s351.card.realized">Gain realized</div><div class="value">$${gain_realized.toLocaleString()}</div></div>
                <div class="card ${gain_recognized > 0 ? 'neg' : 'pos'}"><div class="label" data-i18n="view.s351.card.recognized">Gain recognized</div><div class="value">$${gain_recognized.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s351.card.stock_basis">Stock basis</div><div class="value">$${stock_basis.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s351.card.corp_basis">Corp basis in property</div><div class="value">$${corp_basis.toLocaleString()}</div></div>
            </div>
        </div>
    `;
}
