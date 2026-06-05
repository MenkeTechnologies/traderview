// IRC § 988 — Treatment of Certain Foreign Currency Transactions.
// Ordinary gain/loss on FX transactions (vs. § 1256 60/40 for major currency futures).
// Source = residence (§ 988(a)(3)).
// Election to treat as capital under § 988(a)(1)(B) for forwards/futures/options.

import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    transaction_type: 'spot_fx',
    currency_pair: 'EUR/USD',
    notional_amount: 0,
    acquisition_rate: 0,
    settlement_rate: 0,
    gain_loss_amount: 0,
    is_ordinary_character: true,
    is_s988_transaction: false,
    s988_a_1_b_election_capital: false,
    s988_d_hedging_transaction: false,
    s988_e_personal_transactions: false,
    personal_transaction_under_200_excluded: false,
    is_personal_use: false,
    s988_c_b_forward_contract: false,
    s988_c_c_currency_swap: false,
    s988_c_d_currency_option: false,
    is_traded_on_qualified_board: false,
    s1256_election_alternative: false,
    s1256_60_40_treatment: false,
    holding_period_days: 0,
    functional_currency: 'USD',
    is_qualified_business_unit: false,
    qbu_election_made: false,
    is_branch_remittance: false,
    s987_remittance_amount: 0,
    s987_gain_loss_recognized: 0,
    is_passive_activity: false,
    is_foreign_partnership: false,
    source_residence_taxpayer: 'US',
    s988_a_3_residence_source_rule: true,
    s988_b_payment_in_non_functional: false,
    s988_b_loan_principal: 0,
    s988_b_accrued_interest: 0,
    nonfunctional_currency_borrowed: 0,
    treated_as_loan_principal_repayment: false,
    s988_d_hedging_identification: false,
};

export async function renderSection988(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s988.h1.title">// § 988 FOREIGN CURRENCY TRANSACTIONS</span></h1>
        <p class="muted small" data-i18n="view.s988.hint.intro">
            <strong>§ 988</strong> — gain/loss on certain foreign currency transactions = ORDINARY
            (NOT capital). <strong>Covered transactions:</strong> nonfunctional currency debt
            instruments (§ 988(c)(1)(B)(i)), forward contracts (§ 988(c)(1)(B)(ii)), payables/receivables
            in nonfunctional currency (§ 988(c)(1)(B)(iii)). <strong>§ 988(a)(1)(B) election</strong>
            for forward/futures/options not § 1256: capital character (must be identified by close of
            day acquired). <strong>§ 988(c)(1)(C)(i) exception:</strong> § 1256 contracts — 60/40 MTM
            character preserved. <strong>§ 988(e) personal transactions:</strong> $200 de minimis
            on individual exchange of nonfunctional currency. <strong>§ 988(d) hedging:</strong>
            integrated identified hedge — character matches underlying. <strong>Source = RESIDENCE</strong>
            of taxpayer (§ 988(a)(3)). <strong>§ 987 functional currency QBU</strong> coordinates with
            § 988 branch remittances. <strong>Form 8865 / 5471</strong> report foreign currency
            transactions of foreign entities.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s988.h2.inputs">Inputs</h2>
            <form id="s988-form" class="inline-form">
                <label><span data-i18n="view.s988.label.type">Transaction type</span>
                    <select name="transaction_type">
                        <option value="spot_fx" ${state.transaction_type === 'spot_fx' ? 'selected' : ''}>Spot FX</option>
                        <option value="forward" ${state.transaction_type === 'forward' ? 'selected' : ''}>Forward contract</option>
                        <option value="future" ${state.transaction_type === 'future' ? 'selected' : ''}>Future (§ 1256)</option>
                        <option value="option" ${state.transaction_type === 'option' ? 'selected' : ''}>Currency option</option>
                        <option value="swap" ${state.transaction_type === 'swap' ? 'selected' : ''}>Currency swap</option>
                        <option value="debt" ${state.transaction_type === 'debt' ? 'selected' : ''}>Nonfunctional debt</option>
                        <option value="ar_ap" ${state.transaction_type === 'ar_ap' ? 'selected' : ''}>A/R or A/P</option>
                        <option value="hedge" ${state.transaction_type === 'hedge' ? 'selected' : ''}>Hedging transaction</option>
                    </select>
                </label>
                <label><span data-i18n="view.s988.label.pair">Currency pair</span>
                    <input type="text" name="currency_pair" value="${esc(state.currency_pair)}"></label>
                <label><span data-i18n="view.s988.label.notional">Notional ($)</span>
                    <input type="number" step="0.01" name="notional_amount" value="${state.notional_amount}"></label>
                <label><span data-i18n="view.s988.label.acq_rate">Acq rate</span>
                    <input type="number" step="0.0001" name="acquisition_rate" value="${state.acquisition_rate}"></label>
                <label><span data-i18n="view.s988.label.settle_rate">Settle rate</span>
                    <input type="number" step="0.0001" name="settlement_rate" value="${state.settlement_rate}"></label>
                <label><span data-i18n="view.s988.label.gain">Gain/loss ($)</span>
                    <input type="number" step="0.01" name="gain_loss_amount" value="${state.gain_loss_amount}"></label>
                <label><span data-i18n="view.s988.label.ordinary">Ordinary?</span>
                    <input type="checkbox" name="is_ordinary_character" ${state.is_ordinary_character ? 'checked' : ''}></label>
                <label><span data-i18n="view.s988.label.is_988">Is § 988 transaction?</span>
                    <input type="checkbox" name="is_s988_transaction" ${state.is_s988_transaction ? 'checked' : ''}></label>
                <label><span data-i18n="view.s988.label.s988_a1b">§ 988(a)(1)(B) capital elect?</span>
                    <input type="checkbox" name="s988_a_1_b_election_capital" ${state.s988_a_1_b_election_capital ? 'checked' : ''}></label>
                <label><span data-i18n="view.s988.label.s988d">§ 988(d) hedging?</span>
                    <input type="checkbox" name="s988_d_hedging_transaction" ${state.s988_d_hedging_transaction ? 'checked' : ''}></label>
                <label><span data-i18n="view.s988.label.s988e">§ 988(e) personal?</span>
                    <input type="checkbox" name="s988_e_personal_transactions" ${state.s988_e_personal_transactions ? 'checked' : ''}></label>
                <label><span data-i18n="view.s988.label.under200">&lt; $200 excluded?</span>
                    <input type="checkbox" name="personal_transaction_under_200_excluded" ${state.personal_transaction_under_200_excluded ? 'checked' : ''}></label>
                <label><span data-i18n="view.s988.label.personal">Personal use?</span>
                    <input type="checkbox" name="is_personal_use" ${state.is_personal_use ? 'checked' : ''}></label>
                <label><span data-i18n="view.s988.label.s988cb">§ 988(c)(1)(B) forward?</span>
                    <input type="checkbox" name="s988_c_b_forward_contract" ${state.s988_c_b_forward_contract ? 'checked' : ''}></label>
                <label><span data-i18n="view.s988.label.s988cc">§ 988(c)(1)(C) swap?</span>
                    <input type="checkbox" name="s988_c_c_currency_swap" ${state.s988_c_c_currency_swap ? 'checked' : ''}></label>
                <label><span data-i18n="view.s988.label.s988cd">§ 988(c)(1)(D) option?</span>
                    <input type="checkbox" name="s988_c_d_currency_option" ${state.s988_c_d_currency_option ? 'checked' : ''}></label>
                <label><span data-i18n="view.s988.label.exchange">Qualified board?</span>
                    <input type="checkbox" name="is_traded_on_qualified_board" ${state.is_traded_on_qualified_board ? 'checked' : ''}></label>
                <label><span data-i18n="view.s988.label.s1256_alt">§ 1256 alternative?</span>
                    <input type="checkbox" name="s1256_election_alternative" ${state.s1256_election_alternative ? 'checked' : ''}></label>
                <label><span data-i18n="view.s988.label.s1256_60_40">§ 1256 60/40?</span>
                    <input type="checkbox" name="s1256_60_40_treatment" ${state.s1256_60_40_treatment ? 'checked' : ''}></label>
                <label><span data-i18n="view.s988.label.holding">Holding days</span>
                    <input type="number" step="1" name="holding_period_days" value="${state.holding_period_days}"></label>
                <label><span data-i18n="view.s988.label.functional">Functional currency</span>
                    <input type="text" name="functional_currency" value="${esc(state.functional_currency)}"></label>
                <label><span data-i18n="view.s988.label.qbu">QBU?</span>
                    <input type="checkbox" name="is_qualified_business_unit" ${state.is_qualified_business_unit ? 'checked' : ''}></label>
                <label><span data-i18n="view.s988.label.qbu_elect">QBU election?</span>
                    <input type="checkbox" name="qbu_election_made" ${state.qbu_election_made ? 'checked' : ''}></label>
                <label><span data-i18n="view.s988.label.branch">Branch remittance?</span>
                    <input type="checkbox" name="is_branch_remittance" ${state.is_branch_remittance ? 'checked' : ''}></label>
                <label><span data-i18n="view.s988.label.s987">§ 987 remittance ($)</span>
                    <input type="number" step="0.01" name="s987_remittance_amount" value="${state.s987_remittance_amount}"></label>
                <label><span data-i18n="view.s988.label.s987_gain">§ 987 gain/loss ($)</span>
                    <input type="number" step="0.01" name="s987_gain_loss_recognized" value="${state.s987_gain_loss_recognized}"></label>
                <label><span data-i18n="view.s988.label.passive">Passive?</span>
                    <input type="checkbox" name="is_passive_activity" ${state.is_passive_activity ? 'checked' : ''}></label>
                <label><span data-i18n="view.s988.label.foreign_ps">Foreign PS?</span>
                    <input type="checkbox" name="is_foreign_partnership" ${state.is_foreign_partnership ? 'checked' : ''}></label>
                <label><span data-i18n="view.s988.label.residence">Residence</span>
                    <select name="source_residence_taxpayer">
                        <option value="US" ${state.source_residence_taxpayer === 'US' ? 'selected' : ''}>US person</option>
                        <option value="foreign" ${state.source_residence_taxpayer === 'foreign' ? 'selected' : ''}>Foreign</option>
                    </select>
                </label>
                <label><span data-i18n="view.s988.label.s988a3">§ 988(a)(3) residence source?</span>
                    <input type="checkbox" name="s988_a_3_residence_source_rule" ${state.s988_a_3_residence_source_rule ? 'checked' : ''}></label>
                <label><span data-i18n="view.s988.label.s988b">§ 988(b) NFC payment?</span>
                    <input type="checkbox" name="s988_b_payment_in_non_functional" ${state.s988_b_payment_in_non_functional ? 'checked' : ''}></label>
                <label><span data-i18n="view.s988.label.principal">Loan principal ($)</span>
                    <input type="number" step="0.01" name="s988_b_loan_principal" value="${state.s988_b_loan_principal}"></label>
                <label><span data-i18n="view.s988.label.interest">Accrued interest ($)</span>
                    <input type="number" step="0.01" name="s988_b_accrued_interest" value="${state.s988_b_accrued_interest}"></label>
                <label><span data-i18n="view.s988.label.borrowed">NFC borrowed ($)</span>
                    <input type="number" step="0.01" name="nonfunctional_currency_borrowed" value="${state.nonfunctional_currency_borrowed}"></label>
                <label><span data-i18n="view.s988.label.repayment">Loan repayment?</span>
                    <input type="checkbox" name="treated_as_loan_principal_repayment" ${state.treated_as_loan_principal_repayment ? 'checked' : ''}></label>
                <label><span data-i18n="view.s988.label.hedge_id">§ 988(d) hedge ID?</span>
                    <input type="checkbox" name="s988_d_hedging_identification" ${state.s988_d_hedging_identification ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s988.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s988-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s988.h2.covered">§ 988 covered transactions</h2>
            <ul class="muted small">
                <li data-i18n="view.s988.cov.i">§ 988(c)(1)(B)(i) — debt instrument in nonfunctional currency</li>
                <li data-i18n="view.s988.cov.ii">§ 988(c)(1)(B)(ii) — forward/futures/option in nonfunctional currency</li>
                <li data-i18n="view.s988.cov.iii">§ 988(c)(1)(B)(iii) — receivable/payable in nonfunctional currency</li>
                <li data-i18n="view.s988.cov.iv">§ 988(c)(1)(B)(iv) — accrued items + similar in nonfunctional currency</li>
                <li data-i18n="view.s988.cov.s988_c_2">§ 988(c)(2) — currency swap (treated as series of forward contracts)</li>
                <li data-i18n="view.s988.cov.notional_principal">Notional principal contracts (§ 1.446-3) — generally NOT § 988 unless currency-denominated</li>
                <li data-i18n="view.s988.cov.s988_c_c">§ 988(c)(1)(C) — § 1256 contracts EXCLUDED (60/40 preserved)</li>
                <li data-i18n="view.s988.cov.s_988_c_d_i">§ 988(c)(1)(D)(i) — debt subject to original holding period</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s988.h2.character">Character election (§ 988(a)(1)(B))</h2>
            <ul class="muted small">
                <li data-i18n="view.s988.char.default">DEFAULT § 988(a)(1)(A): ORDINARY character</li>
                <li data-i18n="view.s988.char.election">§ 988(a)(1)(B) election: CAPITAL character</li>
                <li data-i18n="view.s988.char.scope">Election scope: forward/futures/options NOT subject to § 1256</li>
                <li data-i18n="view.s988.char.id_required">Identification required: by close of day acquired</li>
                <li data-i18n="view.s988.char.binding">Election BINDING — separately for each transaction</li>
                <li data-i18n="view.s988.char.s988_e_personal">§ 988(e) personal transactions: capital character + $200 de minimis</li>
                <li data-i18n="view.s988.char.s988_b_principal">§ 988(b) loan principal: capital (basis); interest: ordinary</li>
                <li data-i18n="view.s988.char.s988_d_hedge">§ 988(d) hedging: character matches underlying hedged item</li>
                <li data-i18n="view.s988.char.s1092">Straddle § 1092 may apply to combinations</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s988.h2.s1256_overlap">§ 1256 overlap</h2>
            <ul class="muted small">
                <li data-i18n="view.s988.s1256.preserved">§ 988(c)(1)(C)(i): § 1256 contracts retain 60/40 + MTM character</li>
                <li data-i18n="view.s988.s1256.fx_futures">Major currency futures (CME EUR/USD, JPY/USD) — § 1256 (regulated futures)</li>
                <li data-i18n="view.s988.s1256.opt_election">Option election: § 988(a)(1)(B) capital OR § 1256 60/40 + MTM</li>
                <li data-i18n="view.s988.s1256.qualified_board">CFTC-regulated futures exchange = qualified board</li>
                <li data-i18n="view.s988.s1256.forex_etf">Forex ETF (e.g., FXE) — generally § 988 ordinary, may elect § 1256 in some cases</li>
                <li data-i18n="view.s988.s1256.spot_fx">Spot FX: ALWAYS § 988 — ordinary (unless cap election + identification)</li>
                <li data-i18n="view.s988.s1256.dealer_election">Dealer election: § 475 + § 988 interaction — careful coordination</li>
                <li data-i18n="view.s988.s1256.retail_forex">Retail forex traders: complex — see Notice 2007-71, IRS Memo TAM 9809001</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s988.h2.s987">§ 987 — functional currency / QBU</h2>
            <ul class="muted small">
                <li data-i18n="view.s988.s987.qbu_def">Qualified Business Unit (QBU): separate + clearly identified set of books in foreign currency</li>
                <li data-i18n="view.s988.s987.functional">Functional currency: currency of economic environment of operations</li>
                <li data-i18n="view.s988.s987.us_default">US person functional currency: USD by default</li>
                <li data-i18n="view.s988.s987.translation">QBU income translated to USD using weighted-average exchange rate</li>
                <li data-i18n="view.s988.s987.remittance">Remittance from QBU to US owner: § 987 gain/loss recognized</li>
                <li data-i18n="view.s988.s987.section_987_final_regs">§ 987 final regs (2024): mark-to-market QBU equity + recognize FX gain/loss</li>
                <li data-i18n="view.s988.s987.deferral">Deferral periods for non-economic FX volatility</li>
                <li data-i18n="view.s988.s987.s988_coordination">§ 988 transactions WITHIN QBU translated, not separately reported</li>
                <li data-i18n="view.s988.s987.foreign_branch">Foreign branch: typically a QBU for § 987</li>
                <li data-i18n="view.s988.s987.dre">Disregarded entity: aggregated with owner's QBU</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s988.h2.special_situations">Special situations</h2>
            <ul class="muted small">
                <li data-i18n="view.s988.spec.personal_de_minimis">Personal transactions: $200 de minimis exclusion (per transaction)</li>
                <li data-i18n="view.s988.spec.travel">Travel + meals abroad: typically below $200 threshold per transaction</li>
                <li data-i18n="view.s988.spec.large_personal">Large personal FX transaction: capital character (under § 988(e))</li>
                <li data-i18n="view.s988.spec.foreign_real_estate">Foreign real estate sale: separate § 988 + § 1031 analysis</li>
                <li data-i18n="view.s988.spec.foreign_mortgage">Foreign currency mortgage: § 988(b) — capital on principal + ordinary on interest</li>
                <li data-i18n="view.s988.spec.crypto">Cryptocurrency: NOT § 988 currency (Notice 2014-21 — property, not currency)</li>
                <li data-i18n="view.s988.spec.s988_d_hedging">§ 988(d) hedge identification: written + contemporaneous</li>
                <li data-i18n="view.s988.spec.s988_d_failure">Failed hedge identification: ordinary character preserved (no character matching)</li>
                <li data-i18n="view.s988.spec.f8865_reporting">Form 8865 reports foreign partnership FX transactions</li>
                <li data-i18n="view.s988.spec.f5471_reporting">Form 5471 reports CFC FX transactions</li>
            </ul>
        </div>
    `;
    document.getElementById('s988-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.transaction_type = fd.get('transaction_type');
        state.currency_pair = fd.get('currency_pair') || '';
        state.notional_amount = Number(fd.get('notional_amount')) || 0;
        state.acquisition_rate = Number(fd.get('acquisition_rate')) || 0;
        state.settlement_rate = Number(fd.get('settlement_rate')) || 0;
        state.gain_loss_amount = Number(fd.get('gain_loss_amount')) || 0;
        state.is_ordinary_character = !!fd.get('is_ordinary_character');
        state.is_s988_transaction = !!fd.get('is_s988_transaction');
        state.s988_a_1_b_election_capital = !!fd.get('s988_a_1_b_election_capital');
        state.s988_d_hedging_transaction = !!fd.get('s988_d_hedging_transaction');
        state.s988_e_personal_transactions = !!fd.get('s988_e_personal_transactions');
        state.personal_transaction_under_200_excluded = !!fd.get('personal_transaction_under_200_excluded');
        state.is_personal_use = !!fd.get('is_personal_use');
        state.s988_c_b_forward_contract = !!fd.get('s988_c_b_forward_contract');
        state.s988_c_c_currency_swap = !!fd.get('s988_c_c_currency_swap');
        state.s988_c_d_currency_option = !!fd.get('s988_c_d_currency_option');
        state.is_traded_on_qualified_board = !!fd.get('is_traded_on_qualified_board');
        state.s1256_election_alternative = !!fd.get('s1256_election_alternative');
        state.s1256_60_40_treatment = !!fd.get('s1256_60_40_treatment');
        state.holding_period_days = Number(fd.get('holding_period_days')) || 0;
        state.functional_currency = fd.get('functional_currency') || '';
        state.is_qualified_business_unit = !!fd.get('is_qualified_business_unit');
        state.qbu_election_made = !!fd.get('qbu_election_made');
        state.is_branch_remittance = !!fd.get('is_branch_remittance');
        state.s987_remittance_amount = Number(fd.get('s987_remittance_amount')) || 0;
        state.s987_gain_loss_recognized = Number(fd.get('s987_gain_loss_recognized')) || 0;
        state.is_passive_activity = !!fd.get('is_passive_activity');
        state.is_foreign_partnership = !!fd.get('is_foreign_partnership');
        state.source_residence_taxpayer = fd.get('source_residence_taxpayer');
        state.s988_a_3_residence_source_rule = !!fd.get('s988_a_3_residence_source_rule');
        state.s988_b_payment_in_non_functional = !!fd.get('s988_b_payment_in_non_functional');
        state.s988_b_loan_principal = Number(fd.get('s988_b_loan_principal')) || 0;
        state.s988_b_accrued_interest = Number(fd.get('s988_b_accrued_interest')) || 0;
        state.nonfunctional_currency_borrowed = Number(fd.get('nonfunctional_currency_borrowed')) || 0;
        state.treated_as_loan_principal_repayment = !!fd.get('treated_as_loan_principal_repayment');
        state.s988_d_hedging_identification = !!fd.get('s988_d_hedging_identification');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s988-output');
    if (!el) return;
    let character = state.is_ordinary_character ? 'ORDINARY' : 'CAPITAL';
    if (state.s988_a_1_b_election_capital) character = 'CAPITAL (election)';
    if (state.s1256_60_40_treatment) character = '60% LTCG / 40% STCG (§ 1256)';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s988.h2.result">§ 988 character analysis</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s988.card.type">Transaction</div><div class="value">${esc(state.transaction_type)}</div></div>
                <div class="card"><div class="label" data-i18n="view.s988.card.character">Character</div><div class="value">${character}</div></div>
                <div class="card"><div class="label" data-i18n="view.s988.card.gain">Gain/loss</div><div class="value">$${state.gain_loss_amount.toLocaleString()}</div></div>
                <div class="card ${state.s988_e_personal_transactions && state.gain_loss_amount <= 200 ? 'pos' : ''}"><div class="label" data-i18n="view.s988.card.personal">Personal &lt; $200?</div><div class="value">${state.s988_e_personal_transactions && state.gain_loss_amount <= 200 ? 'EXCLUDED' : 'NO'}</div></div>
            </div>
        </div>
    `;
}
