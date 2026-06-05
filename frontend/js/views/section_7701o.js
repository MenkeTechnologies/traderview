// IRC § 7701(o) — Economic Substance Doctrine (Codified 2010).
// Transaction has economic substance only if: (1) meaningful change in economic position + (2) substantial non-tax purpose.
// Both prongs (CONJUNCTIVE) required after Health Care Act 2010.
// Violations: § 6662(b)(6) 20% strict-liability accuracy penalty; § 6664(c)(2) "reasonable cause" exception unavailable.
// Targets: tax shelter transactions, abusive transactions, listed transactions.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    transaction_value: 0,
    tax_benefits_claimed: 0,
    pre_tax_profit: 0,
    non_tax_purpose: false,
    meaningful_change_position: false,
    listed_transaction: false,
    reportable_transaction: false,
    pre_existing_business_purpose: true,
    relevance_facts: 'taxpayer_conduct',
    statutory_safe_harbor: false,
    doctrine_applies: true,
    accuracy_penalty_marginal: 21,
    reasonable_basis_belief: false,
};

export async function renderSection7701o(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s7701o.h1.title">// § 7701(o) ECONOMIC SUBSTANCE</span></h1>
        <p class="muted small" data-i18n="view.s7701o.hint.intro">
            <strong>Economic substance doctrine</strong> CODIFIED 2010 (Health Care Act). Transaction has
            economic substance only if BOTH: (1) <strong>meaningful change in economic position</strong> + (2)
            <strong>substantial non-tax purpose</strong>. CONJUNCTIVE — both required. <strong>Violations:</strong>
            § 6662(b)(6) 20% accuracy penalty (40% if undisclosed); STRICT LIABILITY (no reasonable
            cause defense). <strong>Targets:</strong> tax shelters, listed transactions, abusive transactions.
            <strong>Pre-2010:</strong> common law "two-prong" was disjunctive in some circuits, conjunctive in others.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s7701o.h2.inputs">Inputs</h2>
            <form id="s7701o-form" class="inline-form">
                <label><span data-i18n="view.s7701o.label.value">Transaction value ($)</span>
                    <input type="number" step="0.01" name="transaction_value" value="${state.transaction_value}"></label>
                <label><span data-i18n="view.s7701o.label.tax_benefit">Tax benefits claimed ($)</span>
                    <input type="number" step="0.01" name="tax_benefits_claimed" value="${state.tax_benefits_claimed}"></label>
                <label><span data-i18n="view.s7701o.label.pre_tax_profit">Pre-tax economic profit ($)</span>
                    <input type="number" step="0.01" name="pre_tax_profit" value="${state.pre_tax_profit}"></label>
                <label><span data-i18n="view.s7701o.label.non_tax">Substantial non-tax purpose?</span>
                    <input type="checkbox" name="non_tax_purpose" ${state.non_tax_purpose ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7701o.label.meaningful">Meaningful change in economic position?</span>
                    <input type="checkbox" name="meaningful_change_position" ${state.meaningful_change_position ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7701o.label.listed">Listed transaction (Notice 2009-7 et al)?</span>
                    <input type="checkbox" name="listed_transaction" ${state.listed_transaction ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7701o.label.reportable">Reportable transaction?</span>
                    <input type="checkbox" name="reportable_transaction" ${state.reportable_transaction ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7701o.label.pre_existing">Pre-existing business purpose?</span>
                    <input type="checkbox" name="pre_existing_business_purpose" ${state.pre_existing_business_purpose ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7701o.label.relevance">Relevance facts</span>
                    <select name="relevance_facts">
                        <option value="taxpayer_conduct" ${state.relevance_facts === 'taxpayer_conduct' ? 'selected' : ''}>Taxpayer's conduct</option>
                        <option value="aggregate" ${state.relevance_facts === 'aggregate' ? 'selected' : ''}>Aggregate transactional</option>
                        <option value="related_party" ${state.relevance_facts === 'related_party' ? 'selected' : ''}>Related party arrangement</option>
                    </select>
                </label>
                <label><span data-i18n="view.s7701o.label.safe_harbor">Statutory safe harbor?</span>
                    <input type="checkbox" name="statutory_safe_harbor" ${state.statutory_safe_harbor ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7701o.label.doctrine">Doctrine "relevant" to transaction?</span>
                    <input type="checkbox" name="doctrine_applies" ${state.doctrine_applies ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7701o.label.marginal">Marginal tax rate %</span>
                    <input type="number" step="0.1" name="accuracy_penalty_marginal" value="${state.accuracy_penalty_marginal}"></label>
                <label><span data-i18n="view.s7701o.label.reasonable">Reasonable basis belief defense?</span>
                    <input type="checkbox" name="reasonable_basis_belief" ${state.reasonable_basis_belief ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s7701o.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s7701o-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7701o.h2.test">Two-prong test (CONJUNCTIVE post-2010)</h2>
            <ol class="muted small">
                <li data-i18n="view.s7701o.t.objective">OBJECTIVE prong: meaningful change in economic position — pre-tax profit potential</li>
                <li data-i18n="view.s7701o.t.subjective">SUBJECTIVE prong: substantial non-tax purpose (business / personal motive)</li>
                <li data-i18n="view.s7701o.t.both">BOTH required — pre-2010 some circuits used disjunctive (either / or)</li>
                <li data-i18n="view.s7701o.t.profit_substantial">Pre-tax profit must be SUBSTANTIAL relative to TAX BENEFIT</li>
                <li data-i18n="view.s7701o.t.financial_accounting">Financial accounting purpose alone DOES NOT count as substantial non-tax</li>
                <li data-i18n="view.s7701o.t.aggregate_or_separate">Aggregate or separately analyze — Commissioner discretion</li>
                <li data-i18n="view.s7701o.t.taxpayer_burden">Taxpayer burden of proof on both prongs</li>
                <li data-i18n="view.s7701o.t.relevant">Doctrine RELEVANT to transaction (not all transactions)</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7701o.h2.penalties">§ 6662(b)(6) accuracy-related penalty</h2>
            <ul class="muted small">
                <li data-i18n="view.s7701o.pen.20pct">20% on understatement attributable to economic substance violation</li>
                <li data-i18n="view.s7701o.pen.40pct">40% on UNDISCLOSED economic substance transaction (Form 8275)</li>
                <li data-i18n="view.s7701o.pen.strict">STRICT LIABILITY: § 6664(c)(2) — no "reasonable cause" defense for § 7701(o)</li>
                <li data-i18n="view.s7701o.pen.contrast">Contrast: regular § 6662 penalty has reasonable cause defense</li>
                <li data-i18n="view.s7701o.pen.disclosure">Disclosure via Form 8275: lowers penalty from 40% to 20%</li>
                <li data-i18n="view.s7701o.pen.qualified_opinion">Qualified opinion (more-likely-than-not) does NOT avoid § 7701(o) penalty</li>
                <li data-i18n="view.s7701o.pen.return_preparer">Return preparer penalty § 6694 applies for "unreasonable position"</li>
                <li data-i18n="view.s7701o.pen.no_settlement">No settlement / amnesty programs for § 7701(o) violations historically</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7701o.h2.case_law">Historic case law (pre-codification)</h2>
            <ul class="muted small">
                <li data-i18n="view.s7701o.case.gregory">Gregory v. Helvering (1935): foundation case — "business purpose" required</li>
                <li data-i18n="view.s7701o.case.frank_lyon">Frank Lyon (1978): sale-leaseback substance over form</li>
                <li data-i18n="view.s7701o.case.compaq_castle">Compaq v. Comm'r (5th Cir. 2002): foreign tax credit shelter struck</li>
                <li data-i18n="view.s7701o.case.coltec">Coltec Industries (Fed. Cir. 2006): two-prong test articulated</li>
                <li data-i18n="view.s7701o.case.son_of_boss">"Son of BOSS" shelters: 2000s wave struck down by economic substance doctrine</li>
                <li data-i18n="view.s7701o.case.castle_harbor">Castle Harbor (CA-3 2009): partnership tax shelter rebuffed</li>
                <li data-i18n="view.s7701o.case.dewees_klamath">Dewees, Klamath: § 6662(b)(6) imposed post-2010</li>
                <li data-i18n="view.s7701o.case.salem_financial">Salem Financial (2014): UK / US dividend stripping shelter</li>
            </ul>
        </div>
    `;
    document.getElementById('s7701o-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.transaction_value = Number(fd.get('transaction_value')) || 0;
        state.tax_benefits_claimed = Number(fd.get('tax_benefits_claimed')) || 0;
        state.pre_tax_profit = Number(fd.get('pre_tax_profit')) || 0;
        state.non_tax_purpose = !!fd.get('non_tax_purpose');
        state.meaningful_change_position = !!fd.get('meaningful_change_position');
        state.listed_transaction = !!fd.get('listed_transaction');
        state.reportable_transaction = !!fd.get('reportable_transaction');
        state.pre_existing_business_purpose = !!fd.get('pre_existing_business_purpose');
        state.relevance_facts = fd.get('relevance_facts');
        state.statutory_safe_harbor = !!fd.get('statutory_safe_harbor');
        state.doctrine_applies = !!fd.get('doctrine_applies');
        state.accuracy_penalty_marginal = Number(fd.get('accuracy_penalty_marginal')) || 0;
        state.reasonable_basis_belief = !!fd.get('reasonable_basis_belief');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s7701o-output');
    if (!el) return;
    const profitRatio = state.tax_benefits_claimed > 0 ? state.pre_tax_profit / state.tax_benefits_claimed : 0;
    const subjective = state.non_tax_purpose;
    const objective = state.meaningful_change_position && profitRatio > 0.10;
    const passesTest = subjective && objective;
    const penalty_pct = state.reportable_transaction ? 0.20 : 0.40;
    const taxAdj = passesTest ? 0 : state.tax_benefits_claimed * (state.accuracy_penalty_marginal / 100);
    const penalty = passesTest ? 0 : state.tax_benefits_claimed * penalty_pct * (state.accuracy_penalty_marginal / 100);
    const totalCost = taxAdj + penalty;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s7701o.h2.result">§ 7701(o) outcome</h2>
            <div class="cards">
                <div class="card ${passesTest ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s7701o.card.passes">Passes test?</div>
                    <div class="value">${passesTest ? esc(t('view.s7701o.status.yes')) : esc(t('view.s7701o.status.no'))}</div>
                </div>
                <div class="card ${subjective ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s7701o.card.subjective">Non-tax purpose?</div>
                    <div class="value">${subjective ? esc(t('view.s7701o.status.yes')) : esc(t('view.s7701o.status.no'))}</div>
                </div>
                <div class="card ${objective ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s7701o.card.objective">Meaningful economic change?</div>
                    <div class="value">${objective ? esc(t('view.s7701o.status.yes')) : esc(t('view.s7701o.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s7701o.card.profit_ratio">Profit/benefit ratio</div>
                    <div class="value">${(profitRatio * 100).toFixed(1)}%</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s7701o.card.tax_adj">Tax adjustment</div>
                    <div class="value">$${taxAdj.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s7701o.card.penalty">§ 6662(b)(6) penalty (${(penalty_pct * 100).toFixed(0)}%)</div>
                    <div class="value">$${penalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s7701o.card.total">TOTAL COST</div>
                    <div class="value">$${totalCost.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${!passesTest ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s7701o.fail_note">
                    Transaction FAILS economic substance: TAX BENEFITS DISALLOWED + 20-40% STRICT LIABILITY
                    penalty (no reasonable cause defense). Disclosure (Form 8275) reduces penalty 40% → 20%.
                    Verify BOTH prongs PRE-transaction with qualified tax opinion + contemporaneous documentation
                    of business purpose.
                </p>
            ` : ''}
        </div>
    `;
}
