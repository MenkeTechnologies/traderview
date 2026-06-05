// IRC § 59A — BEAT (Base Erosion Anti-Abuse Tax).
// 10% (post-2025: 12.5%) × Modified Taxable Income − regular tax.
// Applies to "applicable taxpayer": large corp + 3% base-erosion pct + $500M avg gross receipts.
// MTI = TI + base-erosion tax benefits (deductible payments to foreign related parties).
// Defends against shifting US deductions offshore — a minimum tax on US base.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    taxable_income: 0,
    base_erosion_payments: 0,
    total_deductions: 0,
    avg_gross_receipts_3yr: 0,
    regular_tax: 0,
    foreign_tax_credit: 0,
    pre_2026: true,
    is_bank: false,
    is_securities_dealer: false,
    cost_of_goods_excluded: 0,
};

export async function renderSection59A(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s59A.h1.title">// § 59A BEAT</span></h1>
        <p class="muted small" data-i18n="view.s59A.hint.intro">
            <strong>BEAT = 10% (post-2025: 12.5%) × MTI − regular tax.</strong> Applies to "applicable taxpayer":
            large corp + base-erosion pct ≥ 3% (2% for banks + securities dealers) + avg gross receipts ≥ $500M
            (3-yr). <strong>MTI = TI + base-erosion tax benefits</strong> (deductible payments to foreign
            related parties). Defends US base from offshore deduction shifting. Filed on <strong>Form 8991</strong>.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s59A.h2.inputs">Inputs</h2>
            <form id="s59A-form" class="inline-form">
                <label><span data-i18n="view.s59A.label.ti">Taxable income ($)</span>
                    <input type="number" step="0.01" name="taxable_income" value="${state.taxable_income}"></label>
                <label><span data-i18n="view.s59A.label.bep">Base-erosion payments to foreign related ($)</span>
                    <input type="number" step="0.01" name="base_erosion_payments" value="${state.base_erosion_payments}"></label>
                <label><span data-i18n="view.s59A.label.deductions">Total deductions ($)</span>
                    <input type="number" step="0.01" name="total_deductions" value="${state.total_deductions}"></label>
                <label><span data-i18n="view.s59A.label.gross">Avg gross receipts 3-yr ($)</span>
                    <input type="number" step="0.01" name="avg_gross_receipts_3yr" value="${state.avg_gross_receipts_3yr}"></label>
                <label><span data-i18n="view.s59A.label.regular_tax">Regular tax ($)</span>
                    <input type="number" step="0.01" name="regular_tax" value="${state.regular_tax}"></label>
                <label><span data-i18n="view.s59A.label.ftc">Foreign tax credit ($)</span>
                    <input type="number" step="0.01" name="foreign_tax_credit" value="${state.foreign_tax_credit}"></label>
                <label><span data-i18n="view.s59A.label.pre_2026">Pre-2026 (10%)?</span>
                    <input type="checkbox" name="pre_2026" ${state.pre_2026 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s59A.label.bank">Bank?</span>
                    <input type="checkbox" name="is_bank" ${state.is_bank ? 'checked' : ''}></label>
                <label><span data-i18n="view.s59A.label.dealer">Securities dealer?</span>
                    <input type="checkbox" name="is_securities_dealer" ${state.is_securities_dealer ? 'checked' : ''}></label>
                <label><span data-i18n="view.s59A.label.cogs_excluded">COGS payments (excluded from BEP) ($)</span>
                    <input type="number" step="0.01" name="cost_of_goods_excluded" value="${state.cost_of_goods_excluded}"></label>
                <button class="primary" type="submit" data-i18n="view.s59A.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s59A-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s59A.h2.bep">Base-erosion payments (BEPs)</h2>
            <ul class="muted small">
                <li data-i18n="view.s59A.bep.related">Payments to FOREIGN RELATED PARTY (≥ 25% ownership)</li>
                <li data-i18n="view.s59A.bep.deductible">Must give rise to a DEDUCTION (interest, royalties, services)</li>
                <li data-i18n="view.s59A.bep.cogs_excluded">COGS payments excluded (until 2025 SCM rules)</li>
                <li data-i18n="view.s59A.bep.services_excluded">SCM (Services Cost Method) services without markup excluded</li>
                <li data-i18n="view.s59A.bep.qd">Qualified derivative payments (mark-to-market) excluded</li>
                <li data-i18n="view.s59A.bep.tloan_relief">TLAC interest excluded for Globally Systemically Important Banks</li>
                <li data-i18n="view.s59A.bep.reinsurance">Reinsurance + ceded premiums included</li>
                <li data-i18n="view.s59A.bep.dcl">Dual consolidated losses included</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s59A.h2.applicable">Applicable taxpayer test</h2>
            <ol class="muted small">
                <li data-i18n="view.s59A.app.corp">Corporation (not S-corp, not RIC, not REIT)</li>
                <li data-i18n="view.s59A.app.gross">Avg gross receipts ≥ $500M (3-yr) — aggregated group</li>
                <li data-i18n="view.s59A.app.bep_pct">Base-erosion % ≥ 3% (2% banks + securities dealers)</li>
                <li data-i18n="view.s59A.app.aggregate">Aggregated group: § 1563(a) controlled groups + § 52 brother-sister</li>
                <li data-i18n="view.s59A.app.foreign_owned">Foreign-owned US corps especially exposed</li>
                <li data-i18n="view.s59A.app.derivatives">FDAP withholding NOT enough to escape BEAT</li>
            </ol>
        </div>
    `;
    document.getElementById('s59A-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.taxable_income = Number(fd.get('taxable_income')) || 0;
        state.base_erosion_payments = Number(fd.get('base_erosion_payments')) || 0;
        state.total_deductions = Number(fd.get('total_deductions')) || 0;
        state.avg_gross_receipts_3yr = Number(fd.get('avg_gross_receipts_3yr')) || 0;
        state.regular_tax = Number(fd.get('regular_tax')) || 0;
        state.foreign_tax_credit = Number(fd.get('foreign_tax_credit')) || 0;
        state.pre_2026 = !!fd.get('pre_2026');
        state.is_bank = !!fd.get('is_bank');
        state.is_securities_dealer = !!fd.get('is_securities_dealer');
        state.cost_of_goods_excluded = Number(fd.get('cost_of_goods_excluded')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s59A-output');
    if (!el) return;
    const grossReceiptsMet = state.avg_gross_receipts_3yr >= 500_000_000;
    const bepPctThreshold = (state.is_bank || state.is_securities_dealer) ? 0.02 : 0.03;
    const bepPct = state.total_deductions > 0 ? (state.base_erosion_payments / state.total_deductions) : 0;
    const bepPctMet = bepPct >= bepPctThreshold;
    const applicableTaxpayer = grossReceiptsMet && bepPctMet;
    const mti = state.taxable_income + state.base_erosion_payments;
    const beatRate = state.pre_2026 ? 0.10 : 0.125;
    const tentativeBEAT = mti * beatRate;
    const regularTaxNetFTC = Math.max(0, state.regular_tax - state.foreign_tax_credit);
    const beatLiability = applicableTaxpayer ? Math.max(0, tentativeBEAT - regularTaxNetFTC) : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s59A.h2.result">BEAT computation</h2>
            <div class="cards">
                <div class="card ${applicableTaxpayer ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s59A.card.applicable">Applicable taxpayer?</div>
                    <div class="value">${applicableTaxpayer ? esc(t('view.s59A.status.yes')) : esc(t('view.s59A.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s59A.card.bep_pct">Base-erosion %</div>
                    <div class="value">${(bepPct * 100).toFixed(2)}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s59A.card.threshold">BEP % threshold</div>
                    <div class="value">${(bepPctThreshold * 100).toFixed(1)}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s59A.card.mti">Modified TI</div>
                    <div class="value">$${mti.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s59A.card.tentative">Tentative BEAT (${(beatRate * 100).toFixed(1)}%)</div>
                    <div class="value">$${tentativeBEAT.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s59A.card.liability">BEAT liability</div>
                    <div class="value">$${beatLiability.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${beatLiability > 0 ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s59A.note">
                    BEAT triggered. Form 8991 required. Consider: SCM service restructuring,
                    royalty rate adjustments, COGS recharacterization, qualified derivative payment
                    election, intercompany loan refinancing to non-related sources.
                </p>
            ` : ''}
        </div>
    `;
}
