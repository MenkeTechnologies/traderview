// IRC § 1245 — Depreciation Recapture on Personal Property (ordinary income).
// Recaptures DEPRECIATION as ordinary income on sale of § 1245 property.
// § 1245 property: depreciable personal property + certain real property components.
// Contrast § 1250 (real property — only excess over straight-line recaptured).

import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    asset_type: 'machinery',
    original_cost: 0,
    accumulated_depreciation: 0,
    s179_expensing: 0,
    bonus_depreciation: 0,
    sale_proceeds: 0,
    adjusted_basis: 0,
    realized_gain: 0,
    recapture_potential: 0,
    s1245_gain_ordinary: 0,
    s1231_gain_capital: 0,
    holding_period_months: 12,
    is_section_1245_property: false,
    is_like_kind_exchange: false,
    boot_received: 0,
    installment_sale: false,
    installment_payments: 0,
    is_personal_property: true,
    is_intangible: false,
    is_amortizable_s197: false,
    s197_amortization: 0,
    is_acrs_pre_1981: false,
    is_qualified_real_property: false,
    s1245_a3_listed: false,
    casualty_or_theft: false,
    insurance_proceeds: 0,
    s1033_replacement: false,
};

export async function renderSection1245(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1245.h1.title">// § 1245 DEPRECIATION RECAPTURE</span></h1>
        <p class="muted small" data-i18n="view.s1245.hint.intro">
            <strong>Recharacterizes</strong> gain on disposition of § 1245 property as ORDINARY INCOME
            to extent of accumulated depreciation. <strong>§ 1245 property:</strong> depreciable
            personal property (machinery, equipment, vehicles, livestock, certain components, livestock,
            single-purpose agricultural/horticultural structures), <strong>amortizable § 197 intangibles</strong>,
            certain real property categories. <strong>Recapture amount</strong> = LESSER of (a) realized
            gain OR (b) cumulative depreciation deductions taken. <strong>Excess</strong> = § 1231 gain
            (long-term capital gain treatment). <strong>Contrast § 1250</strong> (real property —
            only EXCESS over straight-line; post-1986 SL only → effectively zero recapture except for
            unrecaptured § 1250 gain at 25%).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1245.h2.inputs">Inputs</h2>
            <form id="s1245-form" class="inline-form">
                <label><span data-i18n="view.s1245.label.type">Asset type</span>
                    <select name="asset_type">
                        <option value="machinery" ${state.asset_type === 'machinery' ? 'selected' : ''}>Machinery / equipment</option>
                        <option value="vehicle" ${state.asset_type === 'vehicle' ? 'selected' : ''}>Vehicle (§ 280F listed)</option>
                        <option value="computer" ${state.asset_type === 'computer' ? 'selected' : ''}>Computer / electronics</option>
                        <option value="furniture" ${state.asset_type === 'furniture' ? 'selected' : ''}>Furniture / fixtures</option>
                        <option value="s197" ${state.asset_type === 's197' ? 'selected' : ''}>§ 197 intangible (goodwill etc.)</option>
                        <option value="livestock" ${state.asset_type === 'livestock' ? 'selected' : ''}>Livestock</option>
                        <option value="single_purpose" ${state.asset_type === 'single_purpose' ? 'selected' : ''}>Single-purpose agri/horti structure</option>
                        <option value="storage_facility" ${state.asset_type === 'storage_facility' ? 'selected' : ''}>Storage facility (§ 1245(a)(3)(B))</option>
                        <option value="elevator" ${state.asset_type === 'elevator' ? 'selected' : ''}>Elevator / escalator</option>
                    </select>
                </label>
                <label><span data-i18n="view.s1245.label.cost">Original cost ($)</span>
                    <input type="number" step="1000" name="original_cost" value="${state.original_cost}"></label>
                <label><span data-i18n="view.s1245.label.accumulated">Accumulated depreciation ($)</span>
                    <input type="number" step="1000" name="accumulated_depreciation" value="${state.accumulated_depreciation}"></label>
                <label><span data-i18n="view.s1245.label.s179">§ 179 expensing ($)</span>
                    <input type="number" step="1000" name="s179_expensing" value="${state.s179_expensing}"></label>
                <label><span data-i18n="view.s1245.label.bonus">Bonus depreciation ($)</span>
                    <input type="number" step="1000" name="bonus_depreciation" value="${state.bonus_depreciation}"></label>
                <label><span data-i18n="view.s1245.label.proceeds">Sale proceeds ($)</span>
                    <input type="number" step="1000" name="sale_proceeds" value="${state.sale_proceeds}"></label>
                <label><span data-i18n="view.s1245.label.basis">Adjusted basis ($)</span>
                    <input type="number" step="1000" name="adjusted_basis" value="${state.adjusted_basis}"></label>
                <label><span data-i18n="view.s1245.label.gain">Realized gain ($)</span>
                    <input type="number" step="1000" name="realized_gain" value="${state.realized_gain}"></label>
                <label><span data-i18n="view.s1245.label.potential">Recapture potential ($)</span>
                    <input type="number" step="1000" name="recapture_potential" value="${state.recapture_potential}"></label>
                <label><span data-i18n="view.s1245.label.ordinary">§ 1245 ordinary ($)</span>
                    <input type="number" step="1000" name="s1245_gain_ordinary" value="${state.s1245_gain_ordinary}"></label>
                <label><span data-i18n="view.s1245.label.capital">§ 1231 capital ($)</span>
                    <input type="number" step="1000" name="s1231_gain_capital" value="${state.s1231_gain_capital}"></label>
                <label><span data-i18n="view.s1245.label.holding">Holding period (months)</span>
                    <input type="number" step="1" name="holding_period_months" value="${state.holding_period_months}"></label>
                <label><span data-i18n="view.s1245.label.s1245">Is § 1245 property?</span>
                    <input type="checkbox" name="is_section_1245_property" ${state.is_section_1245_property ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1245.label.like_kind">Like-kind exchange?</span>
                    <input type="checkbox" name="is_like_kind_exchange" ${state.is_like_kind_exchange ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1245.label.boot">Boot received ($)</span>
                    <input type="number" step="1000" name="boot_received" value="${state.boot_received}"></label>
                <label><span data-i18n="view.s1245.label.installment">Installment sale?</span>
                    <input type="checkbox" name="installment_sale" ${state.installment_sale ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1245.label.payments">Installment payments ($)</span>
                    <input type="number" step="1000" name="installment_payments" value="${state.installment_payments}"></label>
                <label><span data-i18n="view.s1245.label.personal">Personal property?</span>
                    <input type="checkbox" name="is_personal_property" ${state.is_personal_property ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1245.label.intangible">Intangible?</span>
                    <input type="checkbox" name="is_intangible" ${state.is_intangible ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1245.label.s197">§ 197 amortizable?</span>
                    <input type="checkbox" name="is_amortizable_s197" ${state.is_amortizable_s197 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1245.label.s197_amt">§ 197 amortization ($)</span>
                    <input type="number" step="1000" name="s197_amortization" value="${state.s197_amortization}"></label>
                <label><span data-i18n="view.s1245.label.acrs">ACRS pre-1981?</span>
                    <input type="checkbox" name="is_acrs_pre_1981" ${state.is_acrs_pre_1981 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1245.label.qrp">Qualified real property?</span>
                    <input type="checkbox" name="is_qualified_real_property" ${state.is_qualified_real_property ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1245.label.a3listed">§ 1245(a)(3) listed?</span>
                    <input type="checkbox" name="s1245_a3_listed" ${state.s1245_a3_listed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1245.label.casualty">Casualty / theft?</span>
                    <input type="checkbox" name="casualty_or_theft" ${state.casualty_or_theft ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1245.label.insurance">Insurance ($)</span>
                    <input type="number" step="1000" name="insurance_proceeds" value="${state.insurance_proceeds}"></label>
                <label><span data-i18n="view.s1245.label.s1033">§ 1033 replacement?</span>
                    <input type="checkbox" name="s1033_replacement" ${state.s1033_replacement ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s1245.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1245-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1245.h2.property">§ 1245 property categories</h2>
            <ul class="muted small">
                <li data-i18n="view.s1245.cat.a3a">§ 1245(a)(3)(A) — depreciable personal property (machinery, equipment, vehicles)</li>
                <li data-i18n="view.s1245.cat.a3b">§ 1245(a)(3)(B) — other tangible property used in trade/business (storage facilities)</li>
                <li data-i18n="view.s1245.cat.a3c">§ 1245(a)(3)(C) — single-purpose agricultural / horticultural structures</li>
                <li data-i18n="view.s1245.cat.a3d">§ 1245(a)(3)(D) — § 169 pollution control + § 169 facility</li>
                <li data-i18n="view.s1245.cat.a3e">§ 1245(a)(3)(E) — § 179B / § 179C / § 179D / § 179E energy / mine improvements</li>
                <li data-i18n="view.s1245.cat.a3f">§ 1245(a)(3)(F) — petroleum storage facilities</li>
                <li data-i18n="view.s1245.cat.s197">§ 197 amortizable intangibles (goodwill, going concern, customer lists, etc.)</li>
                <li data-i18n="view.s1245.cat.livestock">Livestock held for breeding / dairy / draft (§ 1245(a)(3)(A) per Treas. Reg.)</li>
                <li data-i18n="view.s1245.cat.elevator">Elevators / escalators (§ 1245(a)(3)(C) pre-1986)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1245.h2.calculation">Calculation</h2>
            <ol class="muted small">
                <li data-i18n="view.s1245.calc.realized">Realized gain = sale proceeds − adjusted basis</li>
                <li data-i18n="view.s1245.calc.depreciation">Depreciation recapture potential = ALL depreciation (incl § 179 + bonus + § 168 MACRS)</li>
                <li data-i18n="view.s1245.calc.recapture">Recapture amount = LESSER of (gain) OR (recapture potential)</li>
                <li data-i18n="view.s1245.calc.ordinary">Recapture amount taxed as ORDINARY INCOME</li>
                <li data-i18n="view.s1245.calc.excess">Excess gain = § 1231 gain (LTCG if net § 1231 gain)</li>
                <li data-i18n="view.s1245.calc.loss">Loss: § 1231 ordinary loss treatment (NOT § 1245 recapture)</li>
                <li data-i18n="view.s1245.calc.full_recap">Full recapture if cost &gt; adjusted basis (i.e., depreciation taken)</li>
                <li data-i18n="view.s1245.calc.s291">§ 291 corporate add'l recapture: 20% × (§ 1250 amount that would be § 1245 recapture)</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1245.h2.comparison">§ 1245 vs § 1250 comparison</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s1245.tbl.attr">Attribute</th><th>§ 1245</th><th>§ 1250</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s1245.tbl.property">Property type</td><td data-i18n="view.s1245.tbl.personal">Personal + certain real</td><td data-i18n="view.s1245.tbl.realty">Real property (buildings + components)</td></tr>
                    <tr><td data-i18n="view.s1245.tbl.recapture">Recapture</td><td data-i18n="view.s1245.tbl.all_dep">ALL depreciation (full recapture)</td><td data-i18n="view.s1245.tbl.excess_sl">Only EXCESS over straight-line</td></tr>
                    <tr><td data-i18n="view.s1245.tbl.post1986">Post-1986</td><td data-i18n="view.s1245.tbl.full">Full recapture (MACRS DDB applies)</td><td data-i18n="view.s1245.tbl.zero">Effectively ZERO (only SL allowed)</td></tr>
                    <tr><td data-i18n="view.s1245.tbl.unrecap">Unrecaptured § 1250 gain</td><td>N/A</td><td>25% tax rate on prior SL depreciation</td></tr>
                    <tr><td data-i18n="view.s1245.tbl.rate">Rate (excess)</td><td data-i18n="view.s1245.tbl.ordinary">Ordinary income (up to 37%)</td><td>25% capped (unrecaptured § 1250)</td></tr>
                    <tr><td data-i18n="view.s1245.tbl.s291">§ 291 corporate add'l</td><td>N/A</td><td>20% extra recapture</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1245.h2.special">Special situations</h2>
            <ul class="muted small">
                <li data-i18n="view.s1245.spec.gift">Gift: § 1245(b)(1) — recapture potential carries over to donee</li>
                <li data-i18n="view.s1245.spec.death">Death: § 1245(b)(2) — basis stepped up; recapture potential extinguished</li>
                <li data-i18n="view.s1245.spec.casualty_destruction">Casualty: § 1245(b)(4) — gain to extent insurance &gt; basis triggers recapture</li>
                <li data-i18n="view.s1245.spec.s1033">§ 1033 replacement: defer gain; recapture potential preserved in replacement property</li>
                <li data-i18n="view.s1245.spec.like_kind">§ 1031 like-kind: TCJA limits to real property; § 1245 personal property NO LONGER ELIGIBLE</li>
                <li data-i18n="view.s1245.spec.installment">§ 453 installment sale: recapture in YEAR 1 — pulls forward all ordinary income</li>
                <li data-i18n="view.s1245.spec.disposition_partnership">Distribution from partnership: § 731(c) — recapture preserved</li>
                <li data-i18n="view.s1245.spec.cost_segregation">Cost segregation study: identifies § 1245 vs § 1250 split — accelerates depreciation but creates recapture exposure</li>
                <li data-i18n="view.s1245.spec.s1250_real_property">Cost seg-identified § 1245 components from real estate trigger recapture on sale</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1245.h2.intangibles">§ 197 intangibles + § 1245</h2>
            <ul class="muted small">
                <li data-i18n="view.s1245.int.s197_15yr">§ 197 amortization 15 years straight-line on goodwill, going concern, etc.</li>
                <li data-i18n="view.s1245.int.s1245_intangible">§ 1245 recapture applies to ALL § 197 intangibles</li>
                <li data-i18n="view.s1245.int.gain_ordinary">Sale of goodwill: full § 1245 recapture as ORDINARY income</li>
                <li data-i18n="view.s1245.int.no_capital">No capital gain treatment for amortized portion</li>
                <li data-i18n="view.s1245.int.partial_sale">Partial sale of asset group: recapture allocated by FMV (Reg § 1.1245-1)</li>
                <li data-i18n="view.s1245.int.s1060">§ 1060 residual method allocates basis among classes (Class VI = goodwill)</li>
                <li data-i18n="view.s1245.int.form_8594">Form 8594 reports purchase + sale allocations</li>
                <li data-i18n="view.s1245.int.contrast_pre197">Pre-§ 197 (pre-Aug 10, 1993): goodwill not amortizable, capital gain on sale</li>
            </ul>
        </div>
    `;
    document.getElementById('s1245-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.asset_type = fd.get('asset_type');
        state.original_cost = Number(fd.get('original_cost')) || 0;
        state.accumulated_depreciation = Number(fd.get('accumulated_depreciation')) || 0;
        state.s179_expensing = Number(fd.get('s179_expensing')) || 0;
        state.bonus_depreciation = Number(fd.get('bonus_depreciation')) || 0;
        state.sale_proceeds = Number(fd.get('sale_proceeds')) || 0;
        state.adjusted_basis = Number(fd.get('adjusted_basis')) || 0;
        state.realized_gain = Number(fd.get('realized_gain')) || 0;
        state.recapture_potential = Number(fd.get('recapture_potential')) || 0;
        state.s1245_gain_ordinary = Number(fd.get('s1245_gain_ordinary')) || 0;
        state.s1231_gain_capital = Number(fd.get('s1231_gain_capital')) || 0;
        state.holding_period_months = Number(fd.get('holding_period_months')) || 0;
        state.is_section_1245_property = !!fd.get('is_section_1245_property');
        state.is_like_kind_exchange = !!fd.get('is_like_kind_exchange');
        state.boot_received = Number(fd.get('boot_received')) || 0;
        state.installment_sale = !!fd.get('installment_sale');
        state.installment_payments = Number(fd.get('installment_payments')) || 0;
        state.is_personal_property = !!fd.get('is_personal_property');
        state.is_intangible = !!fd.get('is_intangible');
        state.is_amortizable_s197 = !!fd.get('is_amortizable_s197');
        state.s197_amortization = Number(fd.get('s197_amortization')) || 0;
        state.is_acrs_pre_1981 = !!fd.get('is_acrs_pre_1981');
        state.is_qualified_real_property = !!fd.get('is_qualified_real_property');
        state.s1245_a3_listed = !!fd.get('s1245_a3_listed');
        state.casualty_or_theft = !!fd.get('casualty_or_theft');
        state.insurance_proceeds = Number(fd.get('insurance_proceeds')) || 0;
        state.s1033_replacement = !!fd.get('s1033_replacement');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1245-output');
    if (!el) return;
    const realized = state.sale_proceeds - state.adjusted_basis;
    const total_dep = state.accumulated_depreciation + state.s179_expensing + state.bonus_depreciation + state.s197_amortization;
    const recapture = Math.min(Math.max(0, realized), total_dep);
    const s1231 = Math.max(0, realized - recapture);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1245.h2.result">§ 1245 recapture result</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s1245.card.realized">Realized gain</div><div class="value">$${realized.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s1245.card.depreciation">Total depreciation</div><div class="value">$${total_dep.toLocaleString()}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.s1245.card.recapture">§ 1245 ordinary recapture</div><div class="value">$${recapture.toLocaleString()}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.s1245.card.s1231">§ 1231 capital portion</div><div class="value">$${s1231.toLocaleString()}</div></div>
            </div>
        </div>
    `;
}
