// IRC § 951A — GILTI (Global Intangible Low-Taxed Income).
// CFC tested income minus 10% deemed return on QBAI (Qualified Business Asset Investment).
// Effective rate ~10.5% for C-corps (21% × 50% § 250 deduction); will rise to ~13.125% post-2025.
// § 250 deduction: 50% × GILTI inclusion (FDII complement); 37.5% × FDII.
// § 960(d) deemed-paid credit: 80% of foreign tax on tested income.

import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    tested_income: 0,
    tested_loss: 0,
    qbai: 0,
    interest_expense_allocated: 0,
    foreign_tax_paid_on_tested: 0,
    corporate_taxpayer: true,
    s962_election: false,
    high_tax_election: false,
    foreign_tax_rate: 0,
    pre_2026: true,
};

export async function renderSection951A(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s951A.h1.title">// § 951A GILTI</span></h1>
        <p class="muted small" data-i18n="view.s951A.hint.intro">
            <strong>GILTI = Net CFC Tested Income − NDTIR</strong> (Net Deemed Tangible Income Return =
            10% × QBAI − interest expense). <strong>C-corp effective rate ~10.5%</strong>
            (21% × 50% § 250 deduction), rising to ~13.125% post-2025. <strong>§ 960(d):</strong>
            80% deemed paid credit. <strong>Individual:</strong> 37% w/o § 962; 21% with § 962 + FTC. High-tax
            election (≥ 18.9%): exclude.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s951A.h2.inputs">Inputs</h2>
            <form id="s951A-form" class="inline-form">
                <label><span data-i18n="view.s951A.label.tested_income">CFC tested income ($)</span>
                    <input type="number" step="0.01" name="tested_income" value="${state.tested_income}"></label>
                <label><span data-i18n="view.s951A.label.tested_loss">CFC tested loss ($)</span>
                    <input type="number" step="0.01" name="tested_loss" value="${state.tested_loss}"></label>
                <label><span data-i18n="view.s951A.label.qbai">QBAI (adj basis of depreciable assets) ($)</span>
                    <input type="number" step="0.01" name="qbai" value="${state.qbai}"></label>
                <label><span data-i18n="view.s951A.label.int_exp">Interest expense allocated to QBAI ($)</span>
                    <input type="number" step="0.01" name="interest_expense_allocated" value="${state.interest_expense_allocated}"></label>
                <label><span data-i18n="view.s951A.label.foreign_tax">Foreign tax on tested income ($)</span>
                    <input type="number" step="0.01" name="foreign_tax_paid_on_tested" value="${state.foreign_tax_paid_on_tested}"></label>
                <label><span data-i18n="view.s951A.label.corp">C-corp taxpayer?</span>
                    <input type="checkbox" name="corporate_taxpayer" ${state.corporate_taxpayer ? 'checked' : ''}></label>
                <label><span data-i18n="view.s951A.label.s962">§ 962 election (individual)?</span>
                    <input type="checkbox" name="s962_election" ${state.s962_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s951A.label.high_tax">High-tax election?</span>
                    <input type="checkbox" name="high_tax_election" ${state.high_tax_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s951A.label.foreign_rate">Foreign effective rate %</span>
                    <input type="number" step="0.1" name="foreign_tax_rate" value="${state.foreign_tax_rate}"></label>
                <label><span data-i18n="view.s951A.label.pre_2026">Pre-2026 (50% § 250)?</span>
                    <input type="checkbox" name="pre_2026" ${state.pre_2026 ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s951A.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s951A-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s951A.h2.mechanics">GILTI mechanics</h2>
            <ol class="muted small">
                <li data-i18n="view.s951A.mech.tested">Compute tested income for each CFC (gross less deductions less subF less ECI less high-tax excl.)</li>
                <li data-i18n="view.s951A.mech.aggregate">Aggregate net CFC tested income across all CFCs of US shareholder</li>
                <li data-i18n="view.s951A.mech.qbai">Compute NDTIR: 10% × QBAI − interest expense allocated to QBAI</li>
                <li data-i18n="view.s951A.mech.gilti">GILTI = Net CFC tested income − NDTIR (floor at zero)</li>
                <li data-i18n="view.s951A.mech.s250">C-corp: claim § 250 deduction (50% of GILTI through 2025, 37.5% after)</li>
                <li data-i18n="view.s951A.mech.s960">C-corp: claim § 960(d) deemed-paid credit (80% of allocated foreign tax)</li>
                <li data-i18n="view.s951A.mech.individual">Individual: full GILTI at 37% (no § 250, no § 960) unless § 962 elected</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s951A.h2.planning">GILTI planning levers</h2>
            <ul class="muted small">
                <li data-i18n="view.s951A.plan.qbai">Maximize QBAI: hold depreciable tangible property in CFCs</li>
                <li data-i18n="view.s951A.plan.subF">Convert tested income → subF income (high-tax excluded subF still gives PTI)</li>
                <li data-i18n="view.s951A.plan.high_tax">Make § 954(b)(4) high-tax election if effective rate ≥ 18.9%</li>
                <li data-i18n="view.s951A.plan.individual">Individual US shareholder: § 962 election or check-the-box</li>
                <li data-i18n="view.s951A.plan.bramcfc">Consolidated CFC group: tested losses offset tested income</li>
                <li data-i18n="view.s951A.plan.ctb">Check-the-box to disregarded: branch + Form 8858 reporting</li>
                <li data-i18n="view.s951A.plan.ftc_basket">GILTI sits in own FTC basket — no cross-crediting with general / passive</li>
                <li data-i18n="view.s951A.plan.expense_allocation">Reduce US-source expense allocation to GILTI basket → preserve FTC capacity</li>
            </ul>
        </div>
    `;
    document.getElementById('s951A-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.tested_income = Number(fd.get('tested_income')) || 0;
        state.tested_loss = Number(fd.get('tested_loss')) || 0;
        state.qbai = Number(fd.get('qbai')) || 0;
        state.interest_expense_allocated = Number(fd.get('interest_expense_allocated')) || 0;
        state.foreign_tax_paid_on_tested = Number(fd.get('foreign_tax_paid_on_tested')) || 0;
        state.corporate_taxpayer = !!fd.get('corporate_taxpayer');
        state.s962_election = !!fd.get('s962_election');
        state.high_tax_election = !!fd.get('high_tax_election');
        state.foreign_tax_rate = Number(fd.get('foreign_tax_rate')) || 0;
        state.pre_2026 = !!fd.get('pre_2026');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s951A-output');
    if (!el) return;
    const netTested = Math.max(0, state.tested_income - state.tested_loss);
    const ndtir = Math.max(0, 0.10 * state.qbai - state.interest_expense_allocated);
    const highTaxApplies = state.high_tax_election && state.foreign_tax_rate >= 18.9;
    const gilti = highTaxApplies ? 0 : Math.max(0, netTested - ndtir);
    const s250Rate = state.pre_2026 ? 0.50 : 0.375;
    const allowsS250 = state.corporate_taxpayer || state.s962_election;
    const s250Deduction = allowsS250 ? gilti * s250Rate : 0;
    const grossUp = (state.corporate_taxpayer || state.s962_election) ? state.foreign_tax_paid_on_tested : 0;
    const taxableGILTI = gilti + grossUp - s250Deduction;
    const usRate = (state.corporate_taxpayer || state.s962_election) ? 0.21 : 0.37;
    const usTax = taxableGILTI * usRate;
    const dpc = (state.corporate_taxpayer || state.s962_election) ? state.foreign_tax_paid_on_tested * 0.80 : 0;
    const netTax = Math.max(0, usTax - dpc);
    const effectiveRate = gilti > 0 ? (netTax / gilti * 100) : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s951A.h2.result">GILTI computation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s951A.card.net_tested">Net tested income</div>
                    <div class="value">$${netTested.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s951A.card.ndtir">NDTIR (deemed return)</div>
                    <div class="value">$${ndtir.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s951A.card.gilti">GILTI inclusion</div>
                    <div class="value">$${gilti.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s951A.card.s250">§ 250 deduction (${(s250Rate * 100).toFixed(1)}%)</div>
                    <div class="value">$${s250Deduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s951A.card.taxable">Taxable GILTI</div>
                    <div class="value">$${Math.max(0, taxableGILTI).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s951A.card.s960">§ 960(d) credit (80%)</div>
                    <div class="value">$${dpc.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s951A.card.net_tax">Net US tax on GILTI</div>
                    <div class="value">$${netTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s951A.card.effective">Effective US rate</div>
                    <div class="value">${effectiveRate.toFixed(2)}%</div>
                </div>
            </div>
            ${!state.pre_2026 ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s951A.post_2025_note">
                    Post-2025: § 250 GILTI deduction drops from 50% to 37.5% → effective rate rises from
                    10.5% to 13.125%. Plan accordingly for QBAI buildup and high-tax elections.
                </p>
            ` : ''}
        </div>
    `;
}
