// IRC § 989 — Functional Currency Rules.
// Defines "functional currency" + "qualified business unit" (QBU) for cross-border tax accounting.
// § 989(a): functional currency = primary currency of economic environment.
// § 989(b): QBU = separate, clearly identified set of activities + books in foreign currency.

import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    entity_type: 'us_corp',
    primary_currency_USD: true,
    functional_currency: 'USD',
    is_qbu: false,
    qbu_separate_books: false,
    qbu_clearly_identified: false,
    qbu_separate_business_activities: false,
    qbu_status_test: 'facts_circumstances',
    s989_a_dollar_approximate_separate_transactions: false,
    s989_b_qbu_definition_met: false,
    s989_b_2_qbu_book_records: false,
    qualifying_business_units_count: 0,
    s989_c_translation_rate: 'weighted_avg',
    weighted_avg_exchange_rate: 0,
    spot_rate_year_end: 0,
    historical_rate: 0,
    s989_d_special_provisions: false,
    s987_qbu_remittance_gain: 0,
    s987_qbu_remittance_loss: 0,
    s987_election_made: false,
    s987_2024_regs_effective: false,
    is_branch_or_qbu: false,
    is_dre_disregarded_entity: false,
    is_cfc: false,
    is_partnership_with_foreign_partner: false,
    s988_transaction_count: 0,
    s988_transactions_within_qbu: 0,
    s988_transactions_outside_qbu: 0,
    s988_currency_gain_loss_recognized: 0,
    foreign_corp_treaty: false,
    treaty_residence: false,
    s367_b_outbound_transfer: false,
    s367_b_branch_loss_recapture: false,
    s954_b_subpart_f_high_tax: false,
    branch_profits_tax_s884: false,
    s884_30pct_branch_tax: 0,
    s987_dollar_basis_pool: 0,
    s987_net_unrecognized_FX_gain_loss: 0,
    s989_d_2_blocked_currency: false,
    is_inflation_high_currency: false,
};

export async function renderSection989(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s989.h1.title">// § 989 FUNCTIONAL CURRENCY + QBU</span></h1>
        <p class="muted small" data-i18n="view.s989.hint.intro">
            <strong>§ 989</strong> defines "FUNCTIONAL CURRENCY" + "QUALIFIED BUSINESS UNIT" (QBU)
            framework for international tax accounting. <strong>§ 989(a) Functional Currency:</strong>
            primary currency of the economic environment of operations — US persons default USD;
            foreign corporations may have functional currency = local. <strong>§ 989(b) QBU:</strong>
            (1) SEPARATE + CLEARLY IDENTIFIED activities, (2) maintained SEPARATE BOOKS + records,
            (3) constitutes trade or business. <strong>§ 989(c) Translation:</strong> weighted-average
            exchange rate for income statement; spot rate for balance sheet items.
            <strong>§ 987 QBU mechanics:</strong> 2024 final regs (T.D. 9985) mark-to-market QBU
            equity + recognize FX gain/loss on remittance to owner. <strong>§ 988 transactions
            WITHIN QBU:</strong> translated using functional currency, NOT separately reported.
            <strong>§ 989(d) blocked currency:</strong> special provisions for high-inflation
            currencies + repatriation restrictions.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s989.h2.inputs">Inputs</h2>
            <form id="s989-form" class="inline-form">
                <label><span data-i18n="view.s989.label.entity">Entity type</span>
                    <select name="entity_type">
                        <option value="us_corp" ${state.entity_type === 'us_corp' ? 'selected' : ''}>US C-corporation</option>
                        <option value="us_s_corp" ${state.entity_type === 'us_s_corp' ? 'selected' : ''}>US S-corporation</option>
                        <option value="us_partnership" ${state.entity_type === 'us_partnership' ? 'selected' : ''}>US partnership</option>
                        <option value="cfc" ${state.entity_type === 'cfc' ? 'selected' : ''}>CFC</option>
                        <option value="foreign_branch" ${state.entity_type === 'foreign_branch' ? 'selected' : ''}>Foreign branch (QBU)</option>
                        <option value="dre" ${state.entity_type === 'dre' ? 'selected' : ''}>Disregarded entity (DRE)</option>
                        <option value="individual" ${state.entity_type === 'individual' ? 'selected' : ''}>Individual US person</option>
                    </select>
                </label>
                <label><span data-i18n="view.s989.label.usd">USD primary?</span>
                    <input type="checkbox" name="primary_currency_USD" ${state.primary_currency_USD ? 'checked' : ''}></label>
                <label><span data-i18n="view.s989.label.functional">Functional currency</span>
                    <input type="text" name="functional_currency" value="${esc(state.functional_currency)}"></label>
                <label><span data-i18n="view.s989.label.qbu">Is QBU?</span>
                    <input type="checkbox" name="is_qbu" ${state.is_qbu ? 'checked' : ''}></label>
                <label><span data-i18n="view.s989.label.separate_books">Separate books?</span>
                    <input type="checkbox" name="qbu_separate_books" ${state.qbu_separate_books ? 'checked' : ''}></label>
                <label><span data-i18n="view.s989.label.identified">Clearly identified?</span>
                    <input type="checkbox" name="qbu_clearly_identified" ${state.qbu_clearly_identified ? 'checked' : ''}></label>
                <label><span data-i18n="view.s989.label.business">Sep business activity?</span>
                    <input type="checkbox" name="qbu_separate_business_activities" ${state.qbu_separate_business_activities ? 'checked' : ''}></label>
                <label><span data-i18n="view.s989.label.test">QBU status test</span>
                    <select name="qbu_status_test">
                        <option value="facts_circumstances" ${state.qbu_status_test === 'facts_circumstances' ? 'selected' : ''}>Facts &amp; circumstances</option>
                        <option value="entity_test" ${state.qbu_status_test === 'entity_test' ? 'selected' : ''}>Entity test</option>
                        <option value="activities_test" ${state.qbu_status_test === 'activities_test' ? 'selected' : ''}>Activities test</option>
                    </select>
                </label>
                <label><span data-i18n="view.s989.label.dast">§ 989(a) DAST?</span>
                    <input type="checkbox" name="s989_a_dollar_approximate_separate_transactions" ${state.s989_a_dollar_approximate_separate_transactions ? 'checked' : ''}></label>
                <label><span data-i18n="view.s989.label.s989b">§ 989(b) QBU met?</span>
                    <input type="checkbox" name="s989_b_qbu_definition_met" ${state.s989_b_qbu_definition_met ? 'checked' : ''}></label>
                <label><span data-i18n="view.s989.label.s989b2">§ 989(b)(2) book records?</span>
                    <input type="checkbox" name="s989_b_2_qbu_book_records" ${state.s989_b_2_qbu_book_records ? 'checked' : ''}></label>
                <label><span data-i18n="view.s989.label.count">QBU count</span>
                    <input type="number" step="1" name="qualifying_business_units_count" value="${state.qualifying_business_units_count}"></label>
                <label><span data-i18n="view.s989.label.translation">Translation rate</span>
                    <select name="s989_c_translation_rate">
                        <option value="weighted_avg" ${state.s989_c_translation_rate === 'weighted_avg' ? 'selected' : ''}>Weighted avg</option>
                        <option value="spot_rate" ${state.s989_c_translation_rate === 'spot_rate' ? 'selected' : ''}>Spot rate (FYE)</option>
                        <option value="historical_rate" ${state.s989_c_translation_rate === 'historical_rate' ? 'selected' : ''}>Historical</option>
                    </select>
                </label>
                <label><span data-i18n="view.s989.label.weighted">Weighted avg rate</span>
                    <input type="number" step="0.0001" name="weighted_avg_exchange_rate" value="${state.weighted_avg_exchange_rate}"></label>
                <label><span data-i18n="view.s989.label.spot">Spot rate FYE</span>
                    <input type="number" step="0.0001" name="spot_rate_year_end" value="${state.spot_rate_year_end}"></label>
                <label><span data-i18n="view.s989.label.hist">Historical rate</span>
                    <input type="number" step="0.0001" name="historical_rate" value="${state.historical_rate}"></label>
                <label><span data-i18n="view.s989.label.s989d">§ 989(d) special?</span>
                    <input type="checkbox" name="s989_d_special_provisions" ${state.s989_d_special_provisions ? 'checked' : ''}></label>
                <label><span data-i18n="view.s989.label.s987_gain">§ 987 gain ($)</span>
                    <input type="number" step="10000" name="s987_qbu_remittance_gain" value="${state.s987_qbu_remittance_gain}"></label>
                <label><span data-i18n="view.s989.label.s987_loss">§ 987 loss ($)</span>
                    <input type="number" step="10000" name="s987_qbu_remittance_loss" value="${state.s987_qbu_remittance_loss}"></label>
                <label><span data-i18n="view.s989.label.s987_elect">§ 987 election?</span>
                    <input type="checkbox" name="s987_election_made" ${state.s987_election_made ? 'checked' : ''}></label>
                <label><span data-i18n="view.s989.label.s987_2024">§ 987 2024 regs?</span>
                    <input type="checkbox" name="s987_2024_regs_effective" ${state.s987_2024_regs_effective ? 'checked' : ''}></label>
                <label><span data-i18n="view.s989.label.branch_qbu">Branch/QBU?</span>
                    <input type="checkbox" name="is_branch_or_qbu" ${state.is_branch_or_qbu ? 'checked' : ''}></label>
                <label><span data-i18n="view.s989.label.dre">DRE?</span>
                    <input type="checkbox" name="is_dre_disregarded_entity" ${state.is_dre_disregarded_entity ? 'checked' : ''}></label>
                <label><span data-i18n="view.s989.label.cfc">CFC?</span>
                    <input type="checkbox" name="is_cfc" ${state.is_cfc ? 'checked' : ''}></label>
                <label><span data-i18n="view.s989.label.ps">PS with foreign partner?</span>
                    <input type="checkbox" name="is_partnership_with_foreign_partner" ${state.is_partnership_with_foreign_partner ? 'checked' : ''}></label>
                <label><span data-i18n="view.s989.label.s988_count">§ 988 count</span>
                    <input type="number" step="1" name="s988_transaction_count" value="${state.s988_transaction_count}"></label>
                <label><span data-i18n="view.s989.label.within">Within QBU</span>
                    <input type="number" step="1" name="s988_transactions_within_qbu" value="${state.s988_transactions_within_qbu}"></label>
                <label><span data-i18n="view.s989.label.outside">Outside QBU</span>
                    <input type="number" step="1" name="s988_transactions_outside_qbu" value="${state.s988_transactions_outside_qbu}"></label>
                <label><span data-i18n="view.s989.label.s988_gain">§ 988 gain/loss ($)</span>
                    <input type="number" step="100" name="s988_currency_gain_loss_recognized" value="${state.s988_currency_gain_loss_recognized}"></label>
                <label><span data-i18n="view.s989.label.treaty">Treaty?</span>
                    <input type="checkbox" name="foreign_corp_treaty" ${state.foreign_corp_treaty ? 'checked' : ''}></label>
                <label><span data-i18n="view.s989.label.treaty_res">Treaty residence?</span>
                    <input type="checkbox" name="treaty_residence" ${state.treaty_residence ? 'checked' : ''}></label>
                <label><span data-i18n="view.s989.label.s367b">§ 367(b) outbound?</span>
                    <input type="checkbox" name="s367_b_outbound_transfer" ${state.s367_b_outbound_transfer ? 'checked' : ''}></label>
                <label><span data-i18n="view.s989.label.s367b_branch">Branch loss recap?</span>
                    <input type="checkbox" name="s367_b_branch_loss_recapture" ${state.s367_b_branch_loss_recapture ? 'checked' : ''}></label>
                <label><span data-i18n="view.s989.label.s954b">§ 954(b) high-tax?</span>
                    <input type="checkbox" name="s954_b_subpart_f_high_tax" ${state.s954_b_subpart_f_high_tax ? 'checked' : ''}></label>
                <label><span data-i18n="view.s989.label.s884">§ 884 branch tax?</span>
                    <input type="checkbox" name="branch_profits_tax_s884" ${state.branch_profits_tax_s884 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s989.label.s884_30">§ 884 30% ($)</span>
                    <input type="number" step="10000" name="s884_30pct_branch_tax" value="${state.s884_30pct_branch_tax}"></label>
                <label><span data-i18n="view.s989.label.dbp">§ 987 dollar basis pool ($)</span>
                    <input type="number" step="10000" name="s987_dollar_basis_pool" value="${state.s987_dollar_basis_pool}"></label>
                <label><span data-i18n="view.s989.label.unrec">Net unrec FX g/l ($)</span>
                    <input type="number" step="10000" name="s987_net_unrecognized_FX_gain_loss" value="${state.s987_net_unrecognized_FX_gain_loss}"></label>
                <label><span data-i18n="view.s989.label.blocked">§ 989(d)(2) blocked?</span>
                    <input type="checkbox" name="s989_d_2_blocked_currency" ${state.s989_d_2_blocked_currency ? 'checked' : ''}></label>
                <label><span data-i18n="view.s989.label.inflation">High inflation?</span>
                    <input type="checkbox" name="is_inflation_high_currency" ${state.is_inflation_high_currency ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s989.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s989-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s989.h2.functional">Functional currency determination</h2>
            <ul class="muted small">
                <li data-i18n="view.s989.fc.us_default">US person: USD default functional currency</li>
                <li data-i18n="view.s989.fc.cfc">CFC: local currency of country of incorporation typically</li>
                <li data-i18n="view.s989.fc.qbu">QBU: currency of primary economic environment</li>
                <li data-i18n="view.s989.fc.economic">Economic environment: where prices set, financing obtained, day-to-day operations</li>
                <li data-i18n="view.s989.fc.s989_a_2">§ 989(a)(2) — DAST (Dollar Approximate Separate Transactions) method for high-inflation</li>
                <li data-i18n="view.s989.fc.high_inflation_test">High-inflation = cumulative 100%+ over 3 years</li>
                <li data-i18n="view.s989.fc.election">Election to change functional currency: limited (Reg § 1.985-2/3)</li>
                <li data-i18n="view.s989.fc.consistency">Once established: applies to all subsequent years (consistency required)</li>
                <li data-i18n="view.s989.fc.dre_owner_currency">DRE: owner's functional currency controls</li>
                <li data-i18n="view.s989.fc.partnership">Partnership: foreign partnership may have its own functional currency</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s989.h2.qbu_test">QBU 3-prong test (§ 989(b))</h2>
            <ol class="muted small">
                <li data-i18n="view.s989.qbu.separate">SEPARATE — distinct economic unit from rest of enterprise</li>
                <li data-i18n="view.s989.qbu.identified">CLEARLY IDENTIFIED — geographically + functionally + financially</li>
                <li data-i18n="view.s989.qbu.books">SEPARATE BOOKS — clearly identified set of accounting records</li>
                <li data-i18n="view.s989.qbu.trade_business">TRADE OR BUSINESS — § 1.989(a)-1 — substantial business activities</li>
                <li data-i18n="view.s989.qbu.entity_qbu">Entity (foreign sub, CFC) automatically a QBU</li>
                <li data-i18n="view.s989.qbu.branch_qbu">Foreign branch generally a QBU</li>
                <li data-i18n="view.s989.qbu.dre_qbu">Disregarded entity: separate QBU possible if standalone</li>
                <li data-i18n="view.s989.qbu.multiple_qbu">Multiple QBUs within same entity possible (different geographies / lines of business)</li>
                <li data-i18n="view.s989.qbu.consistency_year">QBU determination annual + consistent</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s989.h2.translation">§ 989(c) translation rates</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s989.tbl.item">Item</th><th data-i18n="view.s989.tbl.rate">Rate</th><th data-i18n="view.s989.tbl.note">Note</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s989.tbl.income">Income / expense items</td><td>Weighted-average</td><td data-i18n="view.s989.tbl.year_avg">Annual weighted average</td></tr>
                    <tr><td data-i18n="view.s989.tbl.bs_monetary">Balance sheet monetary</td><td>Spot rate FYE</td><td>Receivables, payables, debt</td></tr>
                    <tr><td data-i18n="view.s989.tbl.bs_nonmonetary">Balance sheet non-monetary</td><td>Historical</td><td>Inventory, property, equipment</td></tr>
                    <tr><td data-i18n="view.s989.tbl.E_P">E&amp;P pool</td><td>Annual weighted-avg</td><td>§ 902 / § 960 indirect FTC pool</td></tr>
                    <tr><td data-i18n="view.s989.tbl.distribution">Distribution</td><td>Spot rate</td><td>Date of distribution</td></tr>
                    <tr><td data-i18n="view.s989.tbl.tax_paid">Foreign tax paid</td><td>Spot rate accrual</td><td>Or pay-date election</td></tr>
                    <tr><td data-i18n="view.s989.tbl.equity">Equity / accumulated E&amp;P</td><td>Historical</td><td>Inception or layer year</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s989.h2.s987_regs_2024">§ 987 2024 final regs (T.D. 9985)</h2>
            <ul class="muted small">
                <li data-i18n="view.s989.s987.effective">Final regulations effective Dec 11, 2024</li>
                <li data-i18n="view.s989.s987.fx_method">"FEEP method" — Foreign Exchange Exposure Pool</li>
                <li data-i18n="view.s989.s987.mark_to_market">Mark-to-market QBU equity at year-end</li>
                <li data-i18n="view.s989.s987.recognize_remittance">Recognize net unrec FX gain/loss on remittance to owner</li>
                <li data-i18n="view.s989.s987.deferral_election">Deferral election: defer FX g/l until specific events</li>
                <li data-i18n="view.s989.s987.dre">DRE rules: aggregated with owner</li>
                <li data-i18n="view.s989.s987.s987_3">§ 987(3) gain/loss on remittance recognized at FMV</li>
                <li data-i18n="view.s989.s987.partnership">Partnership: partner-level FX exposure pool</li>
                <li data-i18n="view.s989.s987.transition">2025 transition: opening pool calculation</li>
                <li data-i18n="view.s989.s987.successor">Successor functional currency change: § 987(4)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s989.h2.related">Related provisions</h2>
            <ul class="muted small">
                <li data-i18n="view.s989.rel.s987">§ 987 — translation of QBU + remittance gain/loss</li>
                <li data-i18n="view.s989.rel.s988">§ 988 — nonfunctional currency transactions</li>
                <li data-i18n="view.s989.rel.s985">§ 985 — election of functional currency (DASTM)</li>
                <li data-i18n="view.s989.rel.s986">§ 986 — translation of foreign taxes</li>
                <li data-i18n="view.s989.rel.s902_960">§ 902 (repealed) / § 960 indirect FTC pool translations</li>
                <li data-i18n="view.s989.rel.s367_b">§ 367(b) — outbound transfers</li>
                <li data-i18n="view.s989.rel.s884">§ 884 — branch profits tax + branch interest tax</li>
                <li data-i18n="view.s989.rel.s954">§ 954 — Subpart F (currency gains as FPHCI)</li>
                <li data-i18n="view.s989.rel.s951a">§ 951A GILTI — currency translations of tested income</li>
                <li data-i18n="view.s989.rel.s956">§ 956 — CFC US property investment</li>
            </ul>
        </div>
    `;
    document.getElementById('s989-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.entity_type = fd.get('entity_type');
        state.primary_currency_USD = !!fd.get('primary_currency_USD');
        state.functional_currency = fd.get('functional_currency') || '';
        state.is_qbu = !!fd.get('is_qbu');
        state.qbu_separate_books = !!fd.get('qbu_separate_books');
        state.qbu_clearly_identified = !!fd.get('qbu_clearly_identified');
        state.qbu_separate_business_activities = !!fd.get('qbu_separate_business_activities');
        state.qbu_status_test = fd.get('qbu_status_test');
        state.s989_a_dollar_approximate_separate_transactions = !!fd.get('s989_a_dollar_approximate_separate_transactions');
        state.s989_b_qbu_definition_met = !!fd.get('s989_b_qbu_definition_met');
        state.s989_b_2_qbu_book_records = !!fd.get('s989_b_2_qbu_book_records');
        state.qualifying_business_units_count = Number(fd.get('qualifying_business_units_count')) || 0;
        state.s989_c_translation_rate = fd.get('s989_c_translation_rate');
        state.weighted_avg_exchange_rate = Number(fd.get('weighted_avg_exchange_rate')) || 0;
        state.spot_rate_year_end = Number(fd.get('spot_rate_year_end')) || 0;
        state.historical_rate = Number(fd.get('historical_rate')) || 0;
        state.s989_d_special_provisions = !!fd.get('s989_d_special_provisions');
        state.s987_qbu_remittance_gain = Number(fd.get('s987_qbu_remittance_gain')) || 0;
        state.s987_qbu_remittance_loss = Number(fd.get('s987_qbu_remittance_loss')) || 0;
        state.s987_election_made = !!fd.get('s987_election_made');
        state.s987_2024_regs_effective = !!fd.get('s987_2024_regs_effective');
        state.is_branch_or_qbu = !!fd.get('is_branch_or_qbu');
        state.is_dre_disregarded_entity = !!fd.get('is_dre_disregarded_entity');
        state.is_cfc = !!fd.get('is_cfc');
        state.is_partnership_with_foreign_partner = !!fd.get('is_partnership_with_foreign_partner');
        state.s988_transaction_count = Number(fd.get('s988_transaction_count')) || 0;
        state.s988_transactions_within_qbu = Number(fd.get('s988_transactions_within_qbu')) || 0;
        state.s988_transactions_outside_qbu = Number(fd.get('s988_transactions_outside_qbu')) || 0;
        state.s988_currency_gain_loss_recognized = Number(fd.get('s988_currency_gain_loss_recognized')) || 0;
        state.foreign_corp_treaty = !!fd.get('foreign_corp_treaty');
        state.treaty_residence = !!fd.get('treaty_residence');
        state.s367_b_outbound_transfer = !!fd.get('s367_b_outbound_transfer');
        state.s367_b_branch_loss_recapture = !!fd.get('s367_b_branch_loss_recapture');
        state.s954_b_subpart_f_high_tax = !!fd.get('s954_b_subpart_f_high_tax');
        state.branch_profits_tax_s884 = !!fd.get('branch_profits_tax_s884');
        state.s884_30pct_branch_tax = Number(fd.get('s884_30pct_branch_tax')) || 0;
        state.s987_dollar_basis_pool = Number(fd.get('s987_dollar_basis_pool')) || 0;
        state.s987_net_unrecognized_FX_gain_loss = Number(fd.get('s987_net_unrecognized_FX_gain_loss')) || 0;
        state.s989_d_2_blocked_currency = !!fd.get('s989_d_2_blocked_currency');
        state.is_inflation_high_currency = !!fd.get('is_inflation_high_currency');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s989-output');
    if (!el) return;
    const qbu_qualified = state.qbu_separate_books && state.qbu_clearly_identified && state.qbu_separate_business_activities;
    const net_s987 = state.s987_qbu_remittance_gain - state.s987_qbu_remittance_loss;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s989.h2.result">§ 989 functional currency / QBU</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s989.card.functional">Functional</div><div class="value">${esc(state.functional_currency)}</div></div>
                <div class="card ${qbu_qualified ? 'pos' : 'neg'}"><div class="label" data-i18n="view.s989.card.qbu">QBU qualified?</div><div class="value">${qbu_qualified ? 'YES' : 'NO'}</div></div>
                <div class="card"><div class="label" data-i18n="view.s989.card.count">QBU count</div><div class="value">${state.qualifying_business_units_count}</div></div>
                <div class="card ${net_s987 > 0 ? 'pos' : 'neg'}"><div class="label" data-i18n="view.s989.card.s987">§ 987 net g/l</div><div class="value">$${net_s987.toLocaleString()}</div></div>
                <div class="card ${state.is_inflation_high_currency ? 'warn' : ''}"><div class="label" data-i18n="view.s989.card.dast">DAST applies?</div><div class="value">${state.is_inflation_high_currency ? 'YES' : 'NO'}</div></div>
            </div>
        </div>
    `;
}
