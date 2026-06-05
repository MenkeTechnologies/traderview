// IRC § 250 — FDII + GILTI Deduction (TCJA Carrot).
// FDII = 37.5% deduction → effective rate 13.125% (TCJA carrot to keep IP in US).
// GILTI = 50% deduction → effective rate 10.5% (anti-deferral counterweight).
// Post-2025: drops to 21.875% FDII (16.4% effective) + 37.5% GILTI (13.125% effective).
// Limitation: total deduction ≤ taxable income (no NOL creation).
// C-corps ONLY (and § 962-electing individuals).

import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    fdii_gross: 0,
    qbai: 0,
    foreign_use_pct: 100,
    foreign_use_doc: false,
    gilti_inclusion: 0,
    taxable_income_before_250: 0,
    pre_2026: true,
    c_corp: true,
};

export async function renderSection250(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s250.h1.title">// § 250 FDII + GILTI DEDUCTION</span></h1>
        <p class="muted small" data-i18n="view.s250.hint.intro">
            <strong>FDII = 37.5% deduction</strong> → ~13.125% effective rate (TCJA carrot to keep IP in US).
            <strong>GILTI = 50% deduction</strong> → ~10.5% effective rate (anti-deferral counterweight).
            <strong>Post-2025:</strong> FDII drops to 21.875% (16.4% effective); GILTI drops to 37.5%
            (13.125% effective). <strong>Limitation:</strong> deduction ≤ taxable income (no NOL).
            <strong>C-corps only</strong> (and § 962-electing individuals).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s250.h2.inputs">Inputs</h2>
            <form id="s250-form" class="inline-form">
                <label><span data-i18n="view.s250.label.fdii_gross">FDII gross (foreign-use income) ($)</span>
                    <input type="number" step="0.01" name="fdii_gross" value="${state.fdii_gross}"></label>
                <label><span data-i18n="view.s250.label.qbai">Domestic QBAI ($)</span>
                    <input type="number" step="0.01" name="qbai" value="${state.qbai}"></label>
                <label><span data-i18n="view.s250.label.foreign_pct">Foreign-use % of income</span>
                    <input type="number" step="0.1" name="foreign_use_pct" value="${state.foreign_use_pct}"></label>
                <label><span data-i18n="view.s250.label.foreign_doc">Foreign-use documentation in place?</span>
                    <input type="checkbox" name="foreign_use_doc" ${state.foreign_use_doc ? 'checked' : ''}></label>
                <label><span data-i18n="view.s250.label.gilti">GILTI inclusion ($)</span>
                    <input type="number" step="0.01" name="gilti_inclusion" value="${state.gilti_inclusion}"></label>
                <label><span data-i18n="view.s250.label.ti">Taxable income pre-§ 250 ($)</span>
                    <input type="number" step="0.01" name="taxable_income_before_250" value="${state.taxable_income_before_250}"></label>
                <label><span data-i18n="view.s250.label.pre_2026">Pre-2026 (50% / 37.5%)?</span>
                    <input type="checkbox" name="pre_2026" ${state.pre_2026 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s250.label.c_corp">C-corp?</span>
                    <input type="checkbox" name="c_corp" ${state.c_corp ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s250.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s250-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s250.h2.fdii">FDII determination</h2>
            <ol class="muted small">
                <li data-i18n="view.s250.fdii.dei">DEI = Deduction Eligible Income (gross income less subF, GILTI, dividends, etc.)</li>
                <li data-i18n="view.s250.fdii.fdii">FDII = (DEII / DEI) × DEII (Foreign-Derived Income from Eligible Use)</li>
                <li data-i18n="view.s250.fdii.deemed">Deemed Intangible Income = DEI − 10% × Domestic QBAI</li>
                <li data-i18n="view.s250.fdii.foreign_pct">Foreign-use fraction = sales to foreign / total sales (services or sales)</li>
                <li data-i18n="view.s250.fdii.documentation">Documentation required: sales contracts + delivery proof + foreign-use evidence</li>
                <li data-i18n="view.s250.fdii.sales_threshold">$50K small-business exemption: less rigorous docs</li>
                <li data-i18n="view.s250.fdii.related_party">Related-party sales to non-US end user qualifies if documented to ultimate use</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s250.h2.qualifying">Qualifying property + services for FDII</h2>
            <ul class="muted small">
                <li data-i18n="view.s250.qual.goods">Tangible property sold for foreign use (manufactured + customized + warranty)</li>
                <li data-i18n="view.s250.qual.services">Services performed FOR foreign person (delivery point matters)</li>
                <li data-i18n="view.s250.qual.intangibles">Licenses + royalties on IP used outside US</li>
                <li data-i18n="view.s250.qual.digital">Digital services + SaaS (foreign user = qualifying)</li>
                <li data-i18n="view.s250.qual.related_us">NOT qualifying: sales to foreign related party for US-end use</li>
                <li data-i18n="view.s250.qual.shipping">Shipping income + transportation generally excluded</li>
                <li data-i18n="view.s250.qual.commodities">Commodity trading + financial product trading excluded</li>
            </ul>
        </div>
    `;
    document.getElementById('s250-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.fdii_gross = Number(fd.get('fdii_gross')) || 0;
        state.qbai = Number(fd.get('qbai')) || 0;
        state.foreign_use_pct = Number(fd.get('foreign_use_pct')) || 0;
        state.foreign_use_doc = !!fd.get('foreign_use_doc');
        state.gilti_inclusion = Number(fd.get('gilti_inclusion')) || 0;
        state.taxable_income_before_250 = Number(fd.get('taxable_income_before_250')) || 0;
        state.pre_2026 = !!fd.get('pre_2026');
        state.c_corp = !!fd.get('c_corp');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s250-output');
    if (!el) return;
    const dei = state.fdii_gross;
    const deii = dei * (state.foreign_use_pct / 100);
    const deemedTangible = 0.10 * state.qbai;
    const deemedIntangible = Math.max(0, dei - deemedTangible);
    const fdii = dei > 0 ? deemedIntangible * (deii / dei) : 0;
    const fdiiRate = state.pre_2026 ? 0.375 : 0.21875;
    const giltiRate = state.pre_2026 ? 0.50 : 0.375;
    const allowedDeduction = state.c_corp;
    const fdiiDeduction = allowedDeduction ? fdii * fdiiRate : 0;
    const giltiDeduction = allowedDeduction ? state.gilti_inclusion * giltiRate : 0;
    const totalDeductionUncapped = fdiiDeduction + giltiDeduction;
    const totalDeduction = Math.min(totalDeductionUncapped, Math.max(0, state.taxable_income_before_250));
    const usRateFDII = (1 - fdiiRate) * 0.21;
    const usRateGILTI = (1 - giltiRate) * 0.21;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s250.h2.result">§ 250 deduction</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s250.card.dei">DEI</div>
                    <div class="value">$${dei.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s250.card.deii">DEII</div>
                    <div class="value">$${deii.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s250.card.fdii">FDII</div>
                    <div class="value">$${fdii.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s250.card.fdii_ded">FDII deduction (${(fdiiRate * 100).toFixed(1)}%)</div>
                    <div class="value">$${fdiiDeduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s250.card.gilti_ded">GILTI deduction (${(giltiRate * 100).toFixed(1)}%)</div>
                    <div class="value">$${giltiDeduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s250.card.total">Total § 250 (TI-capped)</div>
                    <div class="value">$${totalDeduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s250.card.fdii_eff">FDII effective rate</div>
                    <div class="value">${(usRateFDII * 100).toFixed(3)}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s250.card.gilti_eff">GILTI effective rate</div>
                    <div class="value">${(usRateGILTI * 100).toFixed(3)}%</div>
                </div>
            </div>
            ${totalDeductionUncapped > totalDeduction ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s250.ti_cap_note">
                    Taxable income limitation triggered: full uncapped deduction exceeds TI. Deduction reduced
                    to prevent NOL creation. Unused portion does NOT carry forward — lost permanently.
                </p>
            ` : ''}
        </div>
    `;
}
