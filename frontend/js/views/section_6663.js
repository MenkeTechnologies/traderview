// IRC § 6663 — Civil Fraud Penalty (75% of underpayment).
// IRS proves by CLEAR + CONVINCING EVIDENCE that any portion of underpayment is due to fraud.
// Once fraud established, ENTIRE underpayment presumed fraudulent (unless taxpayer rebuts).
// No SOL limit (§ 6501(c)(1) unlimited assessment period for fraud returns).
// 50% interest add-on prior to 1989 returns. Not deductible.

import { currentViewToken, viewIsCurrent } from '../app.js';

const FRAUD_RATE = 0.75;
const ACCURACY_RATE = 0.20;
const SUBSTANTIAL_UNDERSTATEMENT_THRESHOLD_PCT = 0.10;
const SUBSTANTIAL_UNDERSTATEMENT_MIN = 5_000;

let state = {
    total_underpayment: 0,
    fraud_portion: 0,
    is_partial_fraud: false,
    can_rebut_presumption: false,
    is_substantial_understatement: false,
    fraud_proven_clear_convincing: false,
    correctly_reported_tax_liability: 0,
    reduced_to_offer_in_compromise: false,
    marginal_rate: 0.32,
};

export async function renderSection6663(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6663.h1.title">// § 6663 CIVIL FRAUD 75%</span></h1>
        <p class="muted small" data-i18n="view.s6663.hint.intro">
            <strong>75% of underpayment</strong> portion due to fraud. IRS proves by
            <strong>clear + convincing evidence</strong>. Once any fraud established, ENTIRE
            underpayment presumed fraudulent (rebuttable by taxpayer's evidence). <strong>No SOL
            limit</strong> (§ 6501(c)(1)). 0.5%/month interest. <strong>Civil vs criminal:</strong>
            § 6663 civil; § 7201 criminal (5-yr felony) — government usually wins civil first then
            indicts.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6663.h2.inputs">Inputs</h2>
            <form id="s6663-form" class="inline-form">
                <label><span data-i18n="view.s6663.label.under">Total underpayment ($)</span>
                    <input type="number" step="0.01" name="total_underpayment" value="${state.total_underpayment}"></label>
                <label><span data-i18n="view.s6663.label.fraud">Portion alleged fraudulent ($)</span>
                    <input type="number" step="0.01" name="fraud_portion" value="${state.fraud_portion}"></label>
                <label><span data-i18n="view.s6663.label.partial">Partial fraud (rest negligence)?</span>
                    <input type="checkbox" name="is_partial_fraud" ${state.is_partial_fraud ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6663.label.rebut">Can rebut presumption?</span>
                    <input type="checkbox" name="can_rebut_presumption" ${state.can_rebut_presumption ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6663.label.proven">Fraud PROVEN by clear+convincing?</span>
                    <input type="checkbox" name="fraud_proven_clear_convincing" ${state.fraud_proven_clear_convincing ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6663.label.understatement">Substantial understatement (&gt; 10% or $5k)?</span>
                    <input type="checkbox" name="is_substantial_understatement" ${state.is_substantial_understatement ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6663.label.correctly">Correct tax that should have been reported ($)</span>
                    <input type="number" step="0.01" name="correctly_reported_tax_liability" value="${state.correctly_reported_tax_liability}"></label>
                <label><span data-i18n="view.s6663.label.oic">In OIC negotiation?</span>
                    <input type="checkbox" name="reduced_to_offer_in_compromise" ${state.reduced_to_offer_in_compromise ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6663.label.marginal">Marginal %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s6663.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6663-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6663.h2.badges">"Badges of fraud" (Spies indicia)</h2>
            <ul class="muted small">
                <li data-i18n="view.s6663.badge.double_books">Maintaining double books / false ledgers</li>
                <li data-i18n="view.s6663.badge.false_entries">False entries / alterations to records</li>
                <li data-i18n="view.s6663.badge.false_invoices">False invoices / fabricated receipts</li>
                <li data-i18n="view.s6663.badge.destroyed_records">Destruction of records</li>
                <li data-i18n="view.s6663.badge.concealed_assets">Concealment of assets / income</li>
                <li data-i18n="view.s6663.badge.cash_handling">Excessive cash handling without records</li>
                <li data-i18n="view.s6663.badge.failure_cooperate">Failure to cooperate with auditors</li>
                <li data-i18n="view.s6663.badge.false_statements">False statements during audit</li>
                <li data-i18n="view.s6663.badge.evasion">Repeated underpayment + filing failures over multiple years</li>
                <li data-i18n="view.s6663.badge.consciousness_guilt">"Consciousness of guilt" behaviors (rescheduling, evading IRS contacts)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6663.h2.related">Related civil penalties</h2>
            <ul class="muted small">
                <li data-i18n="view.s6663.rel.6662">§ 6662 Accuracy: 20% (negligence, substantial understatement, valuation misstatement)</li>
                <li data-i18n="view.s6663.rel.6662a">§ 6662A 20%/30%: Reportable transaction understatement</li>
                <li data-i18n="view.s6663.rel.6694">§ 6694 Return preparer: $5k or 75% of preparer's fee for unreasonable position</li>
                <li data-i18n="view.s6663.rel.6707a">§ 6707A 75%: Reportable transaction not disclosed (min $10k / $50k)</li>
                <li data-i18n="view.s6663.rel.6651">§ 6651 Failure to file: 5%/month up to 25% (or 15%/month if fraudulent)</li>
                <li data-i18n="view.s6663.rel.6657">§ 6657 Bad check: lesser of 2% or $25</li>
            </ul>
        </div>
    `;
    document.getElementById('s6663-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.total_underpayment = Number(fd.get('total_underpayment')) || 0;
        state.fraud_portion = Number(fd.get('fraud_portion')) || 0;
        state.is_partial_fraud = !!fd.get('is_partial_fraud');
        state.can_rebut_presumption = !!fd.get('can_rebut_presumption');
        state.fraud_proven_clear_convincing = !!fd.get('fraud_proven_clear_convincing');
        state.is_substantial_understatement = !!fd.get('is_substantial_understatement');
        state.correctly_reported_tax_liability = Number(fd.get('correctly_reported_tax_liability')) || 0;
        state.reduced_to_offer_in_compromise = !!fd.get('reduced_to_offer_in_compromise');
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.32;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6663-output');
    if (!el) return;
    const fraudPenaltyBase = state.fraud_proven_clear_convincing
        ? (state.is_partial_fraud && state.can_rebut_presumption ? state.fraud_portion : state.total_underpayment)
        : 0;
    const fraudPenalty = fraudPenaltyBase * FRAUD_RATE;
    const accuracyBaseRest = state.total_underpayment - state.fraud_portion;
    const accuracyPenalty = state.is_partial_fraud && state.can_rebut_presumption
        ? accuracyBaseRest * ACCURACY_RATE
        : 0;
    const totalPenalty = fraudPenalty + accuracyPenalty;
    const tax_plus_penalty = state.total_underpayment + totalPenalty;
    const interest_estimate = state.total_underpayment * 0.08 * 3;  // simple 3-year estimate
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6663.h2.result">§ 6663 penalty calculation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s6663.card.total_under">Total underpayment</div>
                    <div class="value">$${state.total_underpayment.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6663.card.fraud_base">Fraud penalty base</div>
                    <div class="value">$${fraudPenaltyBase.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6663.card.fraud_penalty">§ 6663 75% fraud penalty</div>
                    <div class="value">$${fraudPenalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${accuracyPenalty > 0 ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.s6663.card.accuracy">§ 6662 20% on non-fraud</div>
                        <div class="value">$${accuracyPenalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
                <div class="card neg">
                    <div class="label" data-i18n="view.s6663.card.tax_penalty">Tax + penalties</div>
                    <div class="value">$${tax_plus_penalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6663.card.interest">~3-yr interest</div>
                    <div class="value">$${interest_estimate.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6663.card.grand_total">Grand total exposure</div>
                    <div class="value">$${(tax_plus_penalty + interest_estimate).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
