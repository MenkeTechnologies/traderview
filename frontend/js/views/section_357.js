// IRC § 357 — Liabilities Assumed in § 351 Transaction.
// General rule: liability assumption NOT boot — no immediate gain.
// § 357(b): tax avoidance / no business purpose → ENTIRE LIABILITY treated as boot.
// § 357(c): liabilities EXCEED basis → gain to extent of excess.
// Affects basis: transferor's basis -= liabilities; transferee inherits transferor's basis.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    property_basis: 0,
    property_fmv: 0,
    liabilities_assumed: 0,
    other_consideration_received: 0,
    boot_received: 0,
    tax_avoidance_motive: false,
    business_purpose: true,
    s357c_recourse: true,
    transferor_other_basis: 0,
    is_s351_transfer: true,
    nq_preferred_received: 0,
};

export async function renderSection357(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s357.h1.title">// § 357 LIABILITIES IN § 351</span></h1>
        <p class="muted small" data-i18n="view.s357.hint.intro">
            <strong>§ 357(a):</strong> Liability assumption generally NOT boot (no immediate gain).
            <strong>§ 357(b):</strong> Tax avoidance / no business purpose → ENTIRE liability treated as boot
            → gain to transferor. <strong>§ 357(c):</strong> Liabilities EXCEED basis → gain to extent of
            excess. <strong>Basis impact:</strong> transferor's stock basis = property basis − liabilities;
            transferee corporation inherits transferor's basis in property. Coordinate with § 351 control test.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s357.h2.inputs">Inputs</h2>
            <form id="s357-form" class="inline-form">
                <label><span data-i18n="view.s357.label.basis">Property basis ($)</span>
                    <input type="number" step="0.01" name="property_basis" value="${state.property_basis}"></label>
                <label><span data-i18n="view.s357.label.fmv">Property FMV ($)</span>
                    <input type="number" step="0.01" name="property_fmv" value="${state.property_fmv}"></label>
                <label><span data-i18n="view.s357.label.liab">Liabilities assumed by corp ($)</span>
                    <input type="number" step="0.01" name="liabilities_assumed" value="${state.liabilities_assumed}"></label>
                <label><span data-i18n="view.s357.label.other_cons">Other consideration received ($)</span>
                    <input type="number" step="0.01" name="other_consideration_received" value="${state.other_consideration_received}"></label>
                <label><span data-i18n="view.s357.label.boot">Boot (cash, other) received ($)</span>
                    <input type="number" step="0.01" name="boot_received" value="${state.boot_received}"></label>
                <label><span data-i18n="view.s357.label.avoid">Tax avoidance motive (§ 357(b))?</span>
                    <input type="checkbox" name="tax_avoidance_motive" ${state.tax_avoidance_motive ? 'checked' : ''}></label>
                <label><span data-i18n="view.s357.label.business">Business purpose for liabilities?</span>
                    <input type="checkbox" name="business_purpose" ${state.business_purpose ? 'checked' : ''}></label>
                <label><span data-i18n="view.s357.label.recourse">Recourse debt?</span>
                    <input type="checkbox" name="s357c_recourse" ${state.s357c_recourse ? 'checked' : ''}></label>
                <label><span data-i18n="view.s357.label.other_basis">Transferor's basis in OTHER prop transferred ($)</span>
                    <input type="number" step="0.01" name="transferor_other_basis" value="${state.transferor_other_basis}"></label>
                <label><span data-i18n="view.s357.label.is351">§ 351 transfer (control 80%)?</span>
                    <input type="checkbox" name="is_s351_transfer" ${state.is_s351_transfer ? 'checked' : ''}></label>
                <label><span data-i18n="view.s357.label.nqpref">Nonqualified preferred received ($)</span>
                    <input type="number" step="0.01" name="nq_preferred_received" value="${state.nq_preferred_received}"></label>
                <button class="primary" type="submit" data-i18n="view.s357.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s357-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s357.h2.s357a">§ 357(a) general rule</h2>
            <ul class="muted small">
                <li data-i18n="view.s357.a.no_boot">Assumption of liabilities NOT treated as boot</li>
                <li data-i18n="view.s357.a.basis_reduce">Transferor's stock basis REDUCED by liabilities assumed</li>
                <li data-i18n="view.s357.a.example">Example: transfer property basis $100K, FMV $300K, liab $80K → no gain; stock basis = $20K</li>
                <li data-i18n="view.s357.a.transferee_basis">Transferee inherits transferor's basis ($100K)</li>
                <li data-i18n="view.s357.a.bookkeeping">No current tax, but built-in gain preserved</li>
                <li data-i18n="view.s357.a.rationale">Economic substance: transferor's net economic position unchanged</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s357.h2.s357b">§ 357(b) exception — tax avoidance</h2>
            <ul class="muted small">
                <li data-i18n="view.s357.b.trigger">Tax avoidance motive OR no business purpose for transfer of liability</li>
                <li data-i18n="view.s357.b.all_boot">ENTIRE liability amount treated as boot → gain to extent of FMV − basis ratio</li>
                <li data-i18n="view.s357.b.intent">IRS often finds avoidance when liability incurred shortly before transfer</li>
                <li data-i18n="view.s357.b.refinance">Refinancing before incorporation: scrutinized as bootstrap</li>
                <li data-i18n="view.s357.b.cash_out">Transferor extracts cash via debt → § 357(b) more likely applies</li>
                <li data-i18n="view.s357.b.recent_debt">Recent debt + cash extraction = "tax avoidance" presumption</li>
                <li data-i18n="view.s357.b.economic_substance">§ 7701(o) economic substance doctrine reinforces § 357(b) policy</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s357.h2.s357c">§ 357(c) excess liabilities → gain</h2>
            <ul class="muted small">
                <li data-i18n="view.s357.c.basic">Liabilities ASSUMED &gt; TOTAL BASIS of property transferred → recognize gain</li>
                <li data-i18n="view.s357.c.amount">Gain amount = liabilities − basis</li>
                <li data-i18n="view.s357.c.example">Example: basis $50K + liabilities $80K → gain $30K</li>
                <li data-i18n="view.s357.c.character">Character: same as if property sold (capital, ordinary, recapture)</li>
                <li data-i18n="view.s357.c.s357c_3">§ 357(c)(3): liabilities arising from deductible-when-paid items NOT counted (accounts payable)</li>
                <li data-i18n="view.s357.c.basis_floor">Transferor's stock basis cannot go below zero — § 357(c) prevents negative basis</li>
                <li data-i18n="view.s357.c.solution_boot">Workaround: take back NOTE / boot for excess (recognize gain anyway but more flexibility)</li>
                <li data-i18n="view.s357.c.s361_reorg">§ 361 reorg context: § 357(c) does NOT apply to plain D reorgs</li>
            </ul>
        </div>
    `;
    document.getElementById('s357-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.property_basis = Number(fd.get('property_basis')) || 0;
        state.property_fmv = Number(fd.get('property_fmv')) || 0;
        state.liabilities_assumed = Number(fd.get('liabilities_assumed')) || 0;
        state.other_consideration_received = Number(fd.get('other_consideration_received')) || 0;
        state.boot_received = Number(fd.get('boot_received')) || 0;
        state.tax_avoidance_motive = !!fd.get('tax_avoidance_motive');
        state.business_purpose = !!fd.get('business_purpose');
        state.s357c_recourse = !!fd.get('s357c_recourse');
        state.transferor_other_basis = Number(fd.get('transferor_other_basis')) || 0;
        state.is_s351_transfer = !!fd.get('is_s351_transfer');
        state.nq_preferred_received = Number(fd.get('nq_preferred_received')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s357-output');
    if (!el) return;
    const totalBasis = state.property_basis + state.transferor_other_basis;
    const builtInGain = Math.max(0, state.property_fmv - state.property_basis);
    const s357bTriggered = state.tax_avoidance_motive || !state.business_purpose;
    const s357cTriggered = !s357bTriggered && state.liabilities_assumed > totalBasis;
    const s357cGain = s357cTriggered ? state.liabilities_assumed - totalBasis : 0;
    const s357bGain = s357bTriggered ? Math.min(state.liabilities_assumed, builtInGain) : 0;
    const bootGain = state.boot_received + state.nq_preferred_received;
    const totalGain = Math.max(s357bGain, s357cGain) + bootGain;
    const tax = totalGain * 0.20;
    const stockBasis = Math.max(0, state.property_basis - state.liabilities_assumed + totalGain - bootGain);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s357.h2.result">§ 357 outcome</h2>
            <div class="cards">
                <div class="card ${s357bTriggered ? 'neg' : ''}">
                    <div class="label" data-i18n="view.s357.card.s357b">§ 357(b) triggered?</div>
                    <div class="value">${s357bTriggered ? esc(t('view.s357.status.yes')) : esc(t('view.s357.status.no'))}</div>
                </div>
                <div class="card ${s357cTriggered ? 'neg' : ''}">
                    <div class="label" data-i18n="view.s357.card.s357c">§ 357(c) triggered?</div>
                    <div class="value">${s357cTriggered ? esc(t('view.s357.status.yes')) : esc(t('view.s357.status.no'))}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s357.card.s357b_gain">§ 357(b) gain</div>
                    <div class="value">$${s357bGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s357.card.s357c_gain">§ 357(c) gain</div>
                    <div class="value">$${s357cGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s357.card.boot_gain">Boot gain</div>
                    <div class="value">$${bootGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s357.card.total">Total gain recognized</div>
                    <div class="value">$${totalGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s357.card.tax">Tax (20%)</div>
                    <div class="value">$${tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s357.card.basis">Stock basis</div>
                    <div class="value">$${stockBasis.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${s357cTriggered ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s357.s357c_note">
                    § 357(c) gain triggered — liabilities exceeded basis. Workarounds: (1) contribute
                    additional high-basis property to absorb excess, (2) transferor retains liability
                    instead of letting corp assume it, (3) take back installment note to spread gain.
                </p>
            ` : ''}
        </div>
    `;
}
