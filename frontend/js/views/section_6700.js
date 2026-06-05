// IRC § 6700 — Promoting Abusive Tax Shelters / False Statements.
// Penalty on promoter of abusive tax shelters who makes false / fraudulent statements OR provides gross valuation overstatements.
// Penalty: GREATER of $1,000 OR 100% of gross income derived from activity.
// Distinct from § 6701 aid + abet penalty (anyone who helps reduce tax via misstatement).
// IRS Notice 2008-7 + Notice 2014-2 + Listed Transactions disclosure.

import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    gross_income_from_activity: 0,
    activity_type: 'tax_shelter',
    is_promoter: true,
    is_advisor: false,
    is_organizer: false,
    is_seller: false,
    false_statement: true,
    gross_valuation_overstatement: false,
    overstatement_amount: 0,
    listed_transaction: false,
    reportable_transaction: false,
    transactions_count: 0,
    penalty_per_transaction: 1_000,
    aiding_abetting_s6701: false,
    aiding_abetting_count: 0,
    statute_of_limitations: 'open',
    voluntary_disclosure: false,
};

export async function renderSection6700(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6700.h1.title">// § 6700 PROMOTER PENALTY</span></h1>
        <p class="muted small" data-i18n="view.s6700.hint.intro">
            Penalty on <strong>PROMOTERS</strong> of abusive tax shelters who: (1) make <strong>false/fraudulent
            statements</strong> about tax benefits OR (2) provide <strong>GROSS VALUATION OVERSTATEMENTS</strong>.
            <strong>Penalty:</strong> GREATER of <strong>$1,000 per activity</strong> OR <strong>100% of gross
            income</strong> derived from the activity. <strong>Separate from § 6701 aid + abet</strong> ($1,000
            individuals; $10,000 corps). <strong>Listed Transactions</strong> + <strong>Reportable Transactions</strong>:
            disclosure requirements + heightened penalties. <strong>Forms 8918 + 8886.</strong>
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6700.h2.inputs">Inputs</h2>
            <form id="s6700-form" class="inline-form">
                <label><span data-i18n="view.s6700.label.gross">Gross income from activity ($)</span>
                    <input type="number" step="0.01" name="gross_income_from_activity" value="${state.gross_income_from_activity}"></label>
                <label><span data-i18n="view.s6700.label.activity">Activity type</span>
                    <select name="activity_type">
                        <option value="tax_shelter" ${state.activity_type === 'tax_shelter' ? 'selected' : ''}>Tax shelter</option>
                        <option value="abusive_partnership" ${state.activity_type === 'abusive_partnership' ? 'selected' : ''}>Abusive partnership</option>
                        <option value="conservation_easement" ${state.activity_type === 'conservation_easement' ? 'selected' : ''}>Syndicated conservation easement</option>
                        <option value="micro_captive" ${state.activity_type === 'micro_captive' ? 'selected' : ''}>Micro-captive insurance</option>
                        <option value="offshore_arrangement" ${state.activity_type === 'offshore_arrangement' ? 'selected' : ''}>Offshore arrangement</option>
                        <option value="malta_pension" ${state.activity_type === 'malta_pension' ? 'selected' : ''}>Malta pension scheme</option>
                        <option value="trust_promoter" ${state.activity_type === 'trust_promoter' ? 'selected' : ''}>Trust promoter (sham)</option>
                        <option value="other" ${state.activity_type === 'other' ? 'selected' : ''}>Other abusive</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6700.label.promoter">Promoter?</span>
                    <input type="checkbox" name="is_promoter" ${state.is_promoter ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6700.label.advisor">Material advisor?</span>
                    <input type="checkbox" name="is_advisor" ${state.is_advisor ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6700.label.organizer">Organizer?</span>
                    <input type="checkbox" name="is_organizer" ${state.is_organizer ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6700.label.seller">Seller?</span>
                    <input type="checkbox" name="is_seller" ${state.is_seller ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6700.label.false">False statement?</span>
                    <input type="checkbox" name="false_statement" ${state.false_statement ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6700.label.overstate">Gross valuation overstatement?</span>
                    <input type="checkbox" name="gross_valuation_overstatement" ${state.gross_valuation_overstatement ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6700.label.overstate_amt">Overstatement amount ($)</span>
                    <input type="number" step="0.01" name="overstatement_amount" value="${state.overstatement_amount}"></label>
                <label><span data-i18n="view.s6700.label.listed">Listed transaction?</span>
                    <input type="checkbox" name="listed_transaction" ${state.listed_transaction ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6700.label.reportable">Reportable transaction?</span>
                    <input type="checkbox" name="reportable_transaction" ${state.reportable_transaction ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6700.label.count">Transactions count</span>
                    <input type="number" step="1" name="transactions_count" value="${state.transactions_count}"></label>
                <label><span data-i18n="view.s6700.label.per_tx">Penalty per transaction ($)</span>
                    <input type="number" step="0.01" name="penalty_per_transaction" value="${state.penalty_per_transaction}"></label>
                <label><span data-i18n="view.s6700.label.s6701">Aid + abet § 6701?</span>
                    <input type="checkbox" name="aiding_abetting_s6701" ${state.aiding_abetting_s6701 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6700.label.s6701_count">Aid + abet count</span>
                    <input type="number" step="1" name="aiding_abetting_count" value="${state.aiding_abetting_count}"></label>
                <label><span data-i18n="view.s6700.label.sol">Statute of limitations</span>
                    <select name="statute_of_limitations">
                        <option value="open" ${state.statute_of_limitations === 'open' ? 'selected' : ''}>Open (assessable)</option>
                        <option value="closed" ${state.statute_of_limitations === 'closed' ? 'selected' : ''}>Closed (expired)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6700.label.voluntary">Voluntary disclosure?</span>
                    <input type="checkbox" name="voluntary_disclosure" ${state.voluntary_disclosure ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s6700.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6700-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6700.h2.scope">§ 6700 scope — who is liable</h2>
            <ul class="muted small">
                <li data-i18n="view.s6700.scope.promoter">Promoter: organizes / participates in sale of plan / arrangement</li>
                <li data-i18n="view.s6700.scope.organizer">Organizer: designs the structure</li>
                <li data-i18n="view.s6700.scope.seller">Seller: who sells interests in the arrangement</li>
                <li data-i18n="view.s6700.scope.advisor">Material advisor (§ 6111): provides tax-related material aid</li>
                <li data-i18n="view.s6700.scope.knowledge_or_should">"Knew or had reason to know" of false statement</li>
                <li data-i18n="view.s6700.scope.gross_valuation">Gross valuation overstatement = 200%+ overstated</li>
                <li data-i18n="view.s6700.scope.tax_benefits">"Tax benefits" includes deductions, credits, exclusions, deferral</li>
                <li data-i18n="view.s6700.scope.compensation_link">Penalty applies to person who derives INCOME from promotion</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6700.h2.listed_transactions">Listed transactions (current IRS list)</h2>
            <ul class="muted small">
                <li data-i18n="view.s6700.lt.malta">Malta pension scheme — Notice 2023-30</li>
                <li data-i18n="view.s6700.lt.micro_captive">Micro-captive insurance § 831(b) abuse — Notice 2016-66</li>
                <li data-i18n="view.s6700.lt.conservation_easement">Syndicated conservation easement — Notice 2017-10</li>
                <li data-i18n="view.s6700.lt.crispr">CRISPR / R&D credit abuse arrangements — recent</li>
                <li data-i18n="view.s6700.lt.welfare_benefit">Welfare benefit fund abuse — Notice 95-34</li>
                <li data-i18n="view.s6700.lt.s453">§ 453 monetized installment sale — Notice 2023-71</li>
                <li data-i18n="view.s6700.lt.partnership_basis">Partnership basis shifting — Notice 2024-54</li>
                <li data-i18n="view.s6700.lt.son_of_boss">Son of BOSS / similar — Notice 99-59 (older but active enforcement)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6700.h2.reporting">Reporting requirements</h2>
            <ul class="muted small">
                <li data-i18n="view.s6700.rep.form_8918">Form 8918 — Material Advisor disclosure (within 30 days)</li>
                <li data-i18n="view.s6700.rep.form_8886">Form 8886 — Reportable Transaction disclosure by taxpayer</li>
                <li data-i18n="view.s6700.rep.list">Maintained list of advisees (Material Advisor List § 6112)</li>
                <li data-i18n="view.s6700.rep.list_30day">Provide list to IRS within 20 days of request</li>
                <li data-i18n="view.s6700.rep.failure_disclosure">Failure to disclose: $50,000 per failure (more for listed)</li>
                <li data-i18n="view.s6700.rep.crossborder">FATCA + CRS reporting overlap for offshore arrangements</li>
                <li data-i18n="view.s6700.rep.reportable_transaction_types">5 reportable transaction categories: listed, confidential, contractual, loss, transaction of interest</li>
                <li data-i18n="view.s6700.rep.statute_listed">Listed transactions: statute of limitations re-opens for 1 yr after disclosure required</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6700.h2.related_penalties">Related penalties</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s6700.th.code">Code</th>
                    <th data-i18n="view.s6700.th.target">Target</th>
                    <th data-i18n="view.s6700.th.penalty">Penalty</th>
                </tr></thead>
                <tbody>
                    <tr><td>§ 6700</td><td>Promoter / organizer</td><td>$1K / 100% gross income</td></tr>
                    <tr><td>§ 6701</td><td>Aid + abet understatement</td><td>$1K individuals / $10K corps</td></tr>
                    <tr><td>§ 6707</td><td>Failure to register tax shelter</td><td>$50K listed</td></tr>
                    <tr><td>§ 6707A</td><td>Failure to disclose reportable transaction</td><td>$5K-$200K</td></tr>
                    <tr><td>§ 6708</td><td>Failure to maintain list</td><td>$10K / day</td></tr>
                    <tr><td>§ 6662</td><td>Accuracy-related (taxpayer)</td><td>20% understatement</td></tr>
                    <tr><td>§ 6663</td><td>Civil fraud (taxpayer)</td><td>75% understatement</td></tr>
                    <tr><td>§ 7201</td><td>Criminal tax evasion</td><td>Felony 5 yrs</td></tr>
                </tbody>
            </table>
        </div>
    `;
    document.getElementById('s6700-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.gross_income_from_activity = Number(fd.get('gross_income_from_activity')) || 0;
        state.activity_type = fd.get('activity_type');
        state.is_promoter = !!fd.get('is_promoter');
        state.is_advisor = !!fd.get('is_advisor');
        state.is_organizer = !!fd.get('is_organizer');
        state.is_seller = !!fd.get('is_seller');
        state.false_statement = !!fd.get('false_statement');
        state.gross_valuation_overstatement = !!fd.get('gross_valuation_overstatement');
        state.overstatement_amount = Number(fd.get('overstatement_amount')) || 0;
        state.listed_transaction = !!fd.get('listed_transaction');
        state.reportable_transaction = !!fd.get('reportable_transaction');
        state.transactions_count = Number(fd.get('transactions_count')) || 0;
        state.penalty_per_transaction = Number(fd.get('penalty_per_transaction')) || 0;
        state.aiding_abetting_s6701 = !!fd.get('aiding_abetting_s6701');
        state.aiding_abetting_count = Number(fd.get('aiding_abetting_count')) || 0;
        state.statute_of_limitations = fd.get('statute_of_limitations');
        state.voluntary_disclosure = !!fd.get('voluntary_disclosure');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6700-output');
    if (!el) return;
    const s6700_per_tx = Math.max(1_000, state.gross_income_from_activity);
    const s6700_total = s6700_per_tx * Math.max(1, state.transactions_count);
    const s6701_total = state.aiding_abetting_s6701 ? state.aiding_abetting_count * 10_000 : 0;
    const total_penalty = s6700_total + s6701_total;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6700.h2.result">§ 6700 / § 6701 penalty</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s6700.card.per_tx">Per-transaction penalty</div>
                    <div class="value">$${s6700_per_tx.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6700.card.tx_count">Transactions count</div>
                    <div class="value">${state.transactions_count.toLocaleString()}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6700.card.s6700">§ 6700 total</div>
                    <div class="value">$${s6700_total.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6700.card.s6701">§ 6701 aid + abet</div>
                    <div class="value">$${s6701_total.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6700.card.total">TOTAL PENALTY</div>
                    <div class="value">$${total_penalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.listed_transaction ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s6700.listed_note">
                    LISTED TRANSACTION: heightened enforcement. § 6707 $50K failure-to-register + § 6707A
                    $5-200K failure-to-disclose. Material advisor list § 6112 must be maintained + provided
                    to IRS within 20 days. Statute of limitations re-opens 1 yr after disclosure required.
                    DOJ injunction also available (§ 7408).
                </p>
            ` : ''}
        </div>
    `;
}
