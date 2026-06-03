// IRC § 280E — Marijuana / Schedule I trafficker expense disallowance.
// Cannabis businesses CANNOT deduct ordinary + necessary business expenses (§ 162).
// Exception: COGS via § 471 — direct + indirect production/acquisition costs ONLY.
// Result: effective tax rates can EXCEED 70%. Caregiving (Olive v. Comm'r) treated
// as trafficking. Pre-2018 Champ v. Comm'r: separate non-cannabis activity OK.

import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    revenue: 0,
    cogs: 0,
    operating_expenses: 0,
    indirect_production: 0,
    is_vertically_integrated: false,
    separate_non_cannabis_revenue: 0,
    separate_non_cannabis_expenses: 0,
    entity_type: 'c_corp',
    federal_rate_c: 0.21,
    federal_marginal_passthrough: 0.37,
};

export async function renderSection280e(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s280e.h1.title">// § 280E CANNABIS DISALLOWANCE</span></h1>
        <p class="muted small" data-i18n="view.s280e.hint.intro">
            Schedule I / II trafficker businesses CANNOT deduct ordinary business expenses.
            <strong>Only COGS via § 471</strong> escapes — direct + indirect
            production / acquisition costs. Result: effective tax rates can <strong>EXCEED 70%</strong>.
            Champ v. Comm'r (2007) carve-out: separate non-cannabis activity (caregiving services)
            keeps its own deductions. Olive v. Comm'r (9th Cir. 2015) tightened that exception.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s280e.h2.inputs">Inputs</h2>
            <form id="s280e-form" class="inline-form">
                <label><span data-i18n="view.s280e.label.revenue">Cannabis revenue ($)</span>
                    <input type="number" step="1000" name="revenue" value="${state.revenue}"></label>
                <label><span data-i18n="view.s280e.label.cogs">Direct COGS ($)</span>
                    <input type="number" step="1000" name="cogs" value="${state.cogs}"></label>
                <label><span data-i18n="view.s280e.label.operating">Operating expenses (DISALLOWED) ($)</span>
                    <input type="number" step="1000" name="operating_expenses" value="${state.operating_expenses}"></label>
                <label><span data-i18n="view.s280e.label.indirect">Indirect production costs (allocate to COGS) ($)</span>
                    <input type="number" step="1000" name="indirect_production" value="${state.indirect_production}"></label>
                <label><span data-i18n="view.s280e.label.integrated">Vertically integrated?</span>
                    <input type="checkbox" name="is_vertically_integrated" ${state.is_vertically_integrated ? 'checked' : ''}></label>
                <label><span data-i18n="view.s280e.label.sep_rev">Separate non-cannabis revenue ($)</span>
                    <input type="number" step="1000" name="separate_non_cannabis_revenue" value="${state.separate_non_cannabis_revenue}"></label>
                <label><span data-i18n="view.s280e.label.sep_exp">Separate non-cannabis expenses ($)</span>
                    <input type="number" step="1000" name="separate_non_cannabis_expenses" value="${state.separate_non_cannabis_expenses}"></label>
                <label><span data-i18n="view.s280e.label.entity">Entity type</span>
                    <select name="entity_type">
                        <option value="c_corp" ${state.entity_type === 'c_corp' ? 'selected' : ''}>C-corp</option>
                        <option value="s_corp" ${state.entity_type === 's_corp' ? 'selected' : ''}>S-corp</option>
                        <option value="llc" ${state.entity_type === 'llc' ? 'selected' : ''}>LLC / partnership</option>
                    </select>
                </label>
                <label><span data-i18n="view.s280e.label.c_rate">C-corp federal rate</span>
                    <input type="number" step="0.01" name="federal_rate_c" value="${state.federal_rate_c}"></label>
                <label><span data-i18n="view.s280e.label.passthrough_rate">Pass-through marginal %</span>
                    <input type="number" step="0.01" name="federal_marginal_passthrough" value="${state.federal_marginal_passthrough}"></label>
                <button class="primary" type="submit" data-i18n="view.s280e.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s280e-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s280e.h2.allowed_cogs">COGS items allowed (per § 471 / pre-Notice 2024-???)</h2>
            <ul class="muted small">
                <li data-i18n="view.s280e.cogs.materials">Direct materials (plant, soil, nutrients, lights)</li>
                <li data-i18n="view.s280e.cogs.direct_labor">Direct cultivation labor (growers, trimmers)</li>
                <li data-i18n="view.s280e.cogs.indirect_production">Indirect production: utilities, depreciation, supervisor wages allocable to grow rooms</li>
                <li data-i18n="view.s280e.cogs.packaging">Packaging + labeling</li>
                <li data-i18n="view.s280e.cogs.testing">Compliance testing (state-mandated)</li>
                <li data-i18n="view.s280e.cogs.transport_internal">Internal transport (cultivation → processing)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s280e.h2.disallowed">DISALLOWED operating expenses</h2>
            <ul class="muted small">
                <li data-i18n="view.s280e.dis.advertising">Marketing + advertising</li>
                <li data-i18n="view.s280e.dis.retail">Retail / dispensary salaries (budtenders)</li>
                <li data-i18n="view.s280e.dis.security">Security personnel + cameras (controversial)</li>
                <li data-i18n="view.s280e.dis.rent">Dispensary rent (production area allocable to COGS)</li>
                <li data-i18n="view.s280e.dis.legal">Legal + accounting</li>
                <li data-i18n="view.s280e.dis.licenses">State licensing fees</li>
                <li data-i18n="view.s280e.dis.delivery">Outbound delivery / shipping</li>
                <li data-i18n="view.s280e.dis.office">Office supplies + IT + insurance</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s280e.h2.case_law">Key cases</h2>
            <ul class="muted small">
                <li data-i18n="view.s280e.case.champ">Champ v. Comm'r (2007) — separate caregiving activity escapes § 280E</li>
                <li data-i18n="view.s280e.case.olive">Olive v. Comm'r (9th Cir. 2015) — caregiving DOES NOT escape if too entwined</li>
                <li data-i18n="view.s280e.case.harborside">Harborside Health Center v. Comm'r (2018) — COGS strictly limited to § 471 production</li>
                <li data-i18n="view.s280e.case.alterman">Alterman v. Comm'r (2018) — denied § 471(c) attempt to recharacterize as inventory</li>
                <li data-i18n="view.s280e.case.alpenglow">Alpenglow Botanicals v. United States (10th Cir. 2018) — § 280E constitutionality upheld</li>
            </ul>
        </div>
    `;
    document.getElementById('s280e-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.revenue = Number(fd.get('revenue')) || 0;
        state.cogs = Number(fd.get('cogs')) || 0;
        state.operating_expenses = Number(fd.get('operating_expenses')) || 0;
        state.indirect_production = Number(fd.get('indirect_production')) || 0;
        state.is_vertically_integrated = !!fd.get('is_vertically_integrated');
        state.separate_non_cannabis_revenue = Number(fd.get('separate_non_cannabis_revenue')) || 0;
        state.separate_non_cannabis_expenses = Number(fd.get('separate_non_cannabis_expenses')) || 0;
        state.entity_type = fd.get('entity_type');
        state.federal_rate_c = Number(fd.get('federal_rate_c')) || 0.21;
        state.federal_marginal_passthrough = Number(fd.get('federal_marginal_passthrough')) || 0.37;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s280e-output');
    if (!el) return;
    const totalCogs = state.cogs + state.indirect_production;
    const cannabisGrossProfit = state.revenue - totalCogs;
    const cannabisTaxableIncome = cannabisGrossProfit;  // 280E: no opex deduction
    const cannabisRate = state.entity_type === 'c_corp' ? state.federal_rate_c : state.federal_marginal_passthrough;
    const cannabisTax = cannabisTaxableIncome * cannabisRate;
    const nonCannabisProfit = state.separate_non_cannabis_revenue - state.separate_non_cannabis_expenses;
    const nonCannabisTax = nonCannabisProfit * cannabisRate;
    const totalEconProfit = cannabisGrossProfit - state.operating_expenses + nonCannabisProfit;
    const totalTax = cannabisTax + nonCannabisTax;
    const effectiveRateVsEcon = totalEconProfit > 0 ? (totalTax / totalEconProfit) : 0;
    const whatNonCannabisBizWouldOwe = (cannabisGrossProfit - state.operating_expenses) * cannabisRate;
    const penaltyForCannabis = cannabisTax - whatNonCannabisBizWouldOwe;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s280e.h2.result">§ 280E impact</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s280e.card.gross_profit">Cannabis gross profit</div>
                    <div class="value">$${cannabisGrossProfit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s280e.card.taxable">§ 280E taxable income</div>
                    <div class="value">$${cannabisTaxableIncome.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s280e.card.cannabis_tax">Federal tax</div>
                    <div class="value">$${cannabisTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s280e.card.total_econ">Total economic profit</div>
                    <div class="value">$${totalEconProfit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s280e.card.effective_rate">Effective rate vs economic</div>
                    <div class="value">${(effectiveRateVsEcon * 100).toFixed(0)}%</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s280e.card.penalty">§ 280E penalty (vs normal biz)</div>
                    <div class="value">$${penaltyForCannabis.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            <p class="muted small" style="margin-top:10px" data-i18n="view.s280e.note.future">
                Federal rescheduling (DEA Schedule I → III proposal pending) would END § 280E
                disallowance. Strong push from MORE Act and SAFE Banking Act.
            </p>
        </div>
    `;
}
