// IRC § 475 — Mark-to-Market for Securities + Commodities Dealers + Trader Election.
// § 475(a) — DEALERS in securities must MTM at year-end + recognize gain/loss as ORDINARY.
// § 475(f) — qualified TRADER (in securities or commodities) can ELECT MTM (gain ordinary + no § 1091 wash sale).
// Election due BY April 15 (or unextended due date) BEFORE the tax year starts.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    classification: 'investor',
    is_qualified_trader: false,
    trader_volume_test_passed: false,
    trader_frequency_test_passed: false,
    days_traded_per_year: 0,
    trades_per_year: 0,
    avg_holding_days: 0,
    short_term_trading_pct: 0,
    has_substantial_activity: false,
    s475_f_election_made: false,
    s475_f_election_year: 2024,
    s475_f_election_deadline_april_15: false,
    s475_f_election_separate_business: false,
    fmv_year_end: 0,
    adjusted_basis_year_end: 0,
    mtm_gain_loss_recognized: 0,
    is_ordinary_character: true,
    s475_b_dealer_exception: false,
    s475_a_securities_dealer: false,
    s475_e_commodities_dealer: false,
    no_s1091_wash_sale: true,
    no_s1259_constructive_sale: false,
    no_s1233_short_sale: false,
    no_s263a_capitalization: true,
    schedule_c_business_filer: false,
    s1402_se_tax_exempt: true,
    s199a_qbi_eligible: false,
    is_partnership_trader: false,
    s475_no_election_for_partnership: false,
    s481_a_change_method_required: false,
    s481_a_4_yr_spread: 0,
    f3115_filed: false,
    election_revocation: false,
    irs_consent_required_for_revocation: false,
    rev_proc_99_17_election: false,
    rev_proc_2017_30_revocation: false,
    s212_investor_expenses_2pct: 0,
    deduction_above_line_trader: true,
    s165_g_2_worthless_securities: false,
    is_marked_to_market_inventory: false,
    s475_c_identification_required: false,
    s475_d_attribution: false,
};

export async function renderSection475(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s475.h1.title">// § 475 TRADER MARK-TO-MARKET ELECTION</span></h1>
        <p class="muted small" data-i18n="view.s475.hint.intro">
            <strong>§ 475(a) MANDATORY for DEALERS</strong> in securities (Reg § 1.475(a)-1) — daily
            MTM + ORDINARY character. <strong>§ 475(e) commodities dealers</strong> — similar.
            <strong>§ 475(f) ELECTIVE for qualified TRADERS</strong> (in securities or commodities)
            — annual MTM + gain/loss ORDINARY + NO § 1091 wash sale + NO § 1259 constructive sale
            + NO § 1233 short-sale rules. <strong>Trader classification:</strong> facts &amp;
            circumstances — substantial volume, short avg holding, daily/weekly trading; based on
            Mayer v. Comm. + Holsinger + Endicott + Boatner cases. <strong>Election TIMING:</strong>
            due by April 15 (unextended) of FIRST year for which election effective. <strong>§ 481(a)
            adjustment:</strong> MTM all open positions at FMV; 4-year spread if unfavorable, 1 year
            if favorable. <strong>Strong benefits:</strong> immediate loss recognition + no wash sale
            + Schedule C above-line expense deduction (avoids 2% AGI floor on § 212 investor expenses).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s475.h2.inputs">Inputs</h2>
            <form id="s475-form" class="inline-form">
                <label><span data-i18n="view.s475.label.classification">Classification</span>
                    <select name="classification">
                        <option value="investor" ${state.classification === 'investor' ? 'selected' : ''}>Investor (§ 212)</option>
                        <option value="trader" ${state.classification === 'trader' ? 'selected' : ''}>Trader (§ 475(f) eligible)</option>
                        <option value="dealer" ${state.classification === 'dealer' ? 'selected' : ''}>Dealer (§ 475(a) mandatory)</option>
                        <option value="hybrid" ${state.classification === 'hybrid' ? 'selected' : ''}>Hybrid trader/investor</option>
                    </select>
                </label>
                <label><span data-i18n="view.s475.label.qualified">Qualified trader?</span>
                    <input type="checkbox" name="is_qualified_trader" ${state.is_qualified_trader ? 'checked' : ''}></label>
                <label><span data-i18n="view.s475.label.volume">Volume test passed?</span>
                    <input type="checkbox" name="trader_volume_test_passed" ${state.trader_volume_test_passed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s475.label.frequency">Frequency test passed?</span>
                    <input type="checkbox" name="trader_frequency_test_passed" ${state.trader_frequency_test_passed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s475.label.days">Days traded / year</span>
                    <input type="number" step="1" name="days_traded_per_year" value="${state.days_traded_per_year}"></label>
                <label><span data-i18n="view.s475.label.trades">Trades / year</span>
                    <input type="number" step="1" name="trades_per_year" value="${state.trades_per_year}"></label>
                <label><span data-i18n="view.s475.label.holding">Avg holding (days)</span>
                    <input type="number" step="1" name="avg_holding_days" value="${state.avg_holding_days}"></label>
                <label><span data-i18n="view.s475.label.st_pct">% short-term</span>
                    <input type="number" step="0.1" name="short_term_trading_pct" value="${state.short_term_trading_pct}"></label>
                <label><span data-i18n="view.s475.label.substantial">Substantial activity?</span>
                    <input type="checkbox" name="has_substantial_activity" ${state.has_substantial_activity ? 'checked' : ''}></label>
                <label><span data-i18n="view.s475.label.elected">§ 475(f) elected?</span>
                    <input type="checkbox" name="s475_f_election_made" ${state.s475_f_election_made ? 'checked' : ''}></label>
                <label><span data-i18n="view.s475.label.year">Election year</span>
                    <input type="number" step="1" name="s475_f_election_year" value="${state.s475_f_election_year}"></label>
                <label><span data-i18n="view.s475.label.deadline">Filed by Apr 15?</span>
                    <input type="checkbox" name="s475_f_election_deadline_april_15" ${state.s475_f_election_deadline_april_15 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s475.label.separate">Separate business?</span>
                    <input type="checkbox" name="s475_f_election_separate_business" ${state.s475_f_election_separate_business ? 'checked' : ''}></label>
                <label><span data-i18n="view.s475.label.fmv">FMV year-end ($)</span>
                    <input type="number" step="100" name="fmv_year_end" value="${state.fmv_year_end}"></label>
                <label><span data-i18n="view.s475.label.basis">Adj basis year-end ($)</span>
                    <input type="number" step="100" name="adjusted_basis_year_end" value="${state.adjusted_basis_year_end}"></label>
                <label><span data-i18n="view.s475.label.mtm">MTM gain/loss ($)</span>
                    <input type="number" step="100" name="mtm_gain_loss_recognized" value="${state.mtm_gain_loss_recognized}"></label>
                <label><span data-i18n="view.s475.label.ordinary">Ordinary?</span>
                    <input type="checkbox" name="is_ordinary_character" ${state.is_ordinary_character ? 'checked' : ''}></label>
                <label><span data-i18n="view.s475.label.dealer_excp">§ 475(b) dealer exception?</span>
                    <input type="checkbox" name="s475_b_dealer_exception" ${state.s475_b_dealer_exception ? 'checked' : ''}></label>
                <label><span data-i18n="view.s475.label.sec_dealer">Sec dealer?</span>
                    <input type="checkbox" name="s475_a_securities_dealer" ${state.s475_a_securities_dealer ? 'checked' : ''}></label>
                <label><span data-i18n="view.s475.label.comm_dealer">Comm dealer?</span>
                    <input type="checkbox" name="s475_e_commodities_dealer" ${state.s475_e_commodities_dealer ? 'checked' : ''}></label>
                <label><span data-i18n="view.s475.label.no_wash">No § 1091 wash?</span>
                    <input type="checkbox" name="no_s1091_wash_sale" ${state.no_s1091_wash_sale ? 'checked' : ''}></label>
                <label><span data-i18n="view.s475.label.no_construct">No § 1259?</span>
                    <input type="checkbox" name="no_s1259_constructive_sale" ${state.no_s1259_constructive_sale ? 'checked' : ''}></label>
                <label><span data-i18n="view.s475.label.no_short">No § 1233?</span>
                    <input type="checkbox" name="no_s1233_short_sale" ${state.no_s1233_short_sale ? 'checked' : ''}></label>
                <label><span data-i18n="view.s475.label.no_capital">No § 263A?</span>
                    <input type="checkbox" name="no_s263a_capitalization" ${state.no_s263a_capitalization ? 'checked' : ''}></label>
                <label><span data-i18n="view.s475.label.schedc">Schedule C filer?</span>
                    <input type="checkbox" name="schedule_c_business_filer" ${state.schedule_c_business_filer ? 'checked' : ''}></label>
                <label><span data-i18n="view.s475.label.se_exempt">§ 1402 SE-exempt?</span>
                    <input type="checkbox" name="s1402_se_tax_exempt" ${state.s1402_se_tax_exempt ? 'checked' : ''}></label>
                <label><span data-i18n="view.s475.label.s199a">§ 199A QBI?</span>
                    <input type="checkbox" name="s199a_qbi_eligible" ${state.s199a_qbi_eligible ? 'checked' : ''}></label>
                <label><span data-i18n="view.s475.label.ps_trader">PS trader?</span>
                    <input type="checkbox" name="is_partnership_trader" ${state.is_partnership_trader ? 'checked' : ''}></label>
                <label><span data-i18n="view.s475.label.no_ps">No § 475 for PS?</span>
                    <input type="checkbox" name="s475_no_election_for_partnership" ${state.s475_no_election_for_partnership ? 'checked' : ''}></label>
                <label><span data-i18n="view.s475.label.s481a">§ 481(a) required?</span>
                    <input type="checkbox" name="s481_a_change_method_required" ${state.s481_a_change_method_required ? 'checked' : ''}></label>
                <label><span data-i18n="view.s475.label.spread">4-yr spread ($)</span>
                    <input type="number" step="10000" name="s481_a_4_yr_spread" value="${state.s481_a_4_yr_spread}"></label>
                <label><span data-i18n="view.s475.label.f3115">Form 3115 filed?</span>
                    <input type="checkbox" name="f3115_filed" ${state.f3115_filed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s475.label.revoke">Revocation?</span>
                    <input type="checkbox" name="election_revocation" ${state.election_revocation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s475.label.consent">IRS consent required?</span>
                    <input type="checkbox" name="irs_consent_required_for_revocation" ${state.irs_consent_required_for_revocation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s475.label.rp99">Rev Proc 99-17?</span>
                    <input type="checkbox" name="rev_proc_99_17_election" ${state.rev_proc_99_17_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s475.label.rp2017">Rev Proc 2017-30?</span>
                    <input type="checkbox" name="rev_proc_2017_30_revocation" ${state.rev_proc_2017_30_revocation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s475.label.s212">§ 212 expenses ($)</span>
                    <input type="number" step="100" name="s212_investor_expenses_2pct" value="${state.s212_investor_expenses_2pct}"></label>
                <label><span data-i18n="view.s475.label.above">Above-line trader?</span>
                    <input type="checkbox" name="deduction_above_line_trader" ${state.deduction_above_line_trader ? 'checked' : ''}></label>
                <label><span data-i18n="view.s475.label.s165g2">§ 165(g)(2) worthless?</span>
                    <input type="checkbox" name="s165_g_2_worthless_securities" ${state.s165_g_2_worthless_securities ? 'checked' : ''}></label>
                <label><span data-i18n="view.s475.label.mtm_inv">MTM inventory?</span>
                    <input type="checkbox" name="is_marked_to_market_inventory" ${state.is_marked_to_market_inventory ? 'checked' : ''}></label>
                <label><span data-i18n="view.s475.label.identify">§ 475(c) identification?</span>
                    <input type="checkbox" name="s475_c_identification_required" ${state.s475_c_identification_required ? 'checked' : ''}></label>
                <label><span data-i18n="view.s475.label.s475d">§ 475(d) attribution?</span>
                    <input type="checkbox" name="s475_d_attribution" ${state.s475_d_attribution ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s475.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s475-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s475.h2.trader_test">Trader vs Investor classification</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s475.tbl.attribute">Attribute</th><th>Investor (§ 212)</th><th>Trader (§ 475(f))</th><th>Dealer (§ 475(a))</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s475.tbl.motivation">Motivation</td><td data-i18n="view.s475.tbl.appreciation">Appreciation + dividends</td><td data-i18n="view.s475.tbl.daily_swings">Daily price swings</td><td data-i18n="view.s475.tbl.spread">Bid-ask spread profits</td></tr>
                    <tr><td data-i18n="view.s475.tbl.activity">Activity level</td><td data-i18n="view.s475.tbl.passive">Passive holding</td><td data-i18n="view.s475.tbl.regular">Regular + substantial</td><td data-i18n="view.s475.tbl.full_time">Full-time + customers</td></tr>
                    <tr><td data-i18n="view.s475.tbl.holding">Holding period</td><td data-i18n="view.s475.tbl.long_term">Long-term focus</td><td data-i18n="view.s475.tbl.short_term">Short-term focus</td><td data-i18n="view.s475.tbl.inventory">Inventory turnover</td></tr>
                    <tr><td data-i18n="view.s475.tbl.character">Character</td><td data-i18n="view.s475.tbl.capital">Capital (§ 1221)</td><td data-i18n="view.s475.tbl.capital_or_ord">Capital default / MTM ordinary</td><td data-i18n="view.s475.tbl.ord_inventory">Ordinary (inventory)</td></tr>
                    <tr><td data-i18n="view.s475.tbl.expenses">Expenses</td><td data-i18n="view.s475.tbl.s212_2pct">§ 212 2% AGI floor (suspended 2018-2025)</td><td data-i18n="view.s475.tbl.schedule_c">Schedule C above-line</td><td data-i18n="view.s475.tbl.business">Business expenses</td></tr>
                    <tr><td data-i18n="view.s475.tbl.se_tax">SE tax</td><td>NO</td><td>NO (capital gains exempt § 1402(a)(3))</td><td>YES if sole proprietor</td></tr>
                    <tr><td data-i18n="view.s475.tbl.wash_sale">Wash sale</td><td>YES (§ 1091)</td><td>YES (default) / NO (with § 475(f) MTM)</td><td>NO (§ 475(d)(3))</td></tr>
                    <tr><td data-i18n="view.s475.tbl.mtm">MTM</td><td>NO</td><td>NO (default) / YES (§ 475(f) election)</td><td>MANDATORY (§ 475(a))</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s475.h2.qualified_trader">Qualified trader (Mayer / Holsinger / Endicott)</h2>
            <ol class="muted small">
                <li data-i18n="view.s475.qt.substantial">SUBSTANTIAL activity — typically 720+ trades / year + 4+ trades/day average</li>
                <li data-i18n="view.s475.qt.regular_frequent">Regular + frequent — trade most days the market is open (200+ trading days)</li>
                <li data-i18n="view.s475.qt.short_holding">Short avg holding period (days to weeks, NOT months)</li>
                <li data-i18n="view.s475.qt.daily_swings">Profit motivation: daily price swings, NOT long-term appreciation + dividends</li>
                <li data-i18n="view.s475.qt.facts_circumstances">Facts &amp; circumstances test — NO bright lines</li>
                <li data-i18n="view.s475.qt.endicott_factors">Endicott v. Comm. 145 T.C. 11 (2015): 4 factors weight</li>
                <li data-i18n="view.s475.qt.holsinger_quantitative">Holsinger 130 T.C. 187 (2008): 800+ trades qualified</li>
                <li data-i18n="view.s475.qt.mayer">Mayer v. Comm. T.C. Memo 1994-209: original 4-factor framework</li>
                <li data-i18n="view.s475.qt.partnership_trader">Partnership trader: each partner's election may differ</li>
                <li data-i18n="view.s475.qt.daytrader">"Day trader" generally meets definition</li>
                <li data-i18n="view.s475.qt.investor_holding">Long-term holdings + dividends → investor character (Mayer)</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s475.h2.election">§ 475(f) election mechanics</h2>
            <ul class="muted small">
                <li data-i18n="view.s475.elect.timing">Election due by April 15 (unextended) of FIRST year for which effective</li>
                <li data-i18n="view.s475.elect.attached">Statement attached to timely filed return for PRIOR year</li>
                <li data-i18n="view.s475.elect.s475_f_2_b">§ 475(f)(2)(B) — election applies starting NEXT taxable year</li>
                <li data-i18n="view.s475.elect.f3115_year_after">Form 3115 filed in YEAR AFTER election (change in accounting method)</li>
                <li data-i18n="view.s475.elect.rev_proc_99_17">Rev. Proc. 99-17 — original procedures</li>
                <li data-i18n="view.s475.elect.s481_a">§ 481(a) adjustment: MTM all open positions at FMV (favorable + unfavorable)</li>
                <li data-i18n="view.s475.elect.s481_spread">Spread: 4-yr unfavorable / 1-yr favorable (Rev. Proc. 99-17 + 2022-14)</li>
                <li data-i18n="view.s475.elect.identification">§ 475(c)(2)(C): securities held for investment must be IDENTIFIED on same day</li>
                <li data-i18n="view.s475.elect.id_failure">Failed identification → MTM applies to ALL securities</li>
                <li data-i18n="view.s475.elect.revocation">Revocation: Rev. Proc. 2017-30 — automatic with timely Form 3115</li>
                <li data-i18n="view.s475.elect.binding">Binding for that year + all subsequent years (until revoked)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s475.h2.benefits">Benefits + drawbacks</h2>
            <ul class="muted small">
                <li data-i18n="view.s475.ben.no_wash">+ NO § 1091 wash sale tracking</li>
                <li data-i18n="view.s475.ben.no_capital_loss_limit">+ NO $3,000 capital loss limitation (ordinary loss fully deductible)</li>
                <li data-i18n="view.s475.ben.s212_above_line">+ Above-line Schedule C deductions (no § 67(g) 2% AGI floor)</li>
                <li data-i18n="view.s475.ben.s195_startup">+ § 195 startup costs eligible</li>
                <li data-i18n="view.s475.ben.s162">+ § 162 ordinary &amp; necessary business expense full deductibility</li>
                <li data-i18n="view.s475.ben.s199a_nope">+ § 199A QBI — generally NOT eligible (specified service trade — investment management)</li>
                <li data-i18n="view.s475.ben.s481_a_payment">- § 481(a) catch-up adjustment may be large + payable in year of change</li>
                <li data-i18n="view.s475.ben.no_LTCG">- Loss of LTCG rates on long-term winners (all ordinary)</li>
                <li data-i18n="view.s475.ben.no_qualified_dividend">- Dividends on held positions still qualified — separately reported</li>
                <li data-i18n="view.s475.ben.unrealized_loss">- Year-end MTM forces unrealized loss recognition (cash flow + tax)</li>
                <li data-i18n="view.s475.ben.s1256_overlap">- § 1256 contracts already 60/40 MTM — § 475 makes 100% ordinary</li>
                <li data-i18n="view.s475.ben.iso_disqualifying">- ISO exercise + sale subject to MTM if held in trading account</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s475.h2.s475_c_id">§ 475(c) identification requirement</h2>
            <ul class="muted small">
                <li data-i18n="view.s475.id.purpose">Permits dealer/trader to hold INVESTMENT securities separately</li>
                <li data-i18n="view.s475.id.same_day">Identification ON SAME DAY of acquisition (Reg § 1.475(c)-2)</li>
                <li data-i18n="view.s475.id.book_record">Clear book + record identification</li>
                <li data-i18n="view.s475.id.separate_account">Best practice: separate "investment account" (NOT MTM) from "trading account" (MTM)</li>
                <li data-i18n="view.s475.id.late_id">Late identification → security MTM applied; subsequent sale = ordinary</li>
                <li data-i18n="view.s475.id.no_id">No identification: all securities MTM, ordinary</li>
                <li data-i18n="view.s475.id.s475_d">§ 475(d): identification IRREVOCABLE</li>
                <li data-i18n="view.s475.id.brokerage">Multiple brokerage accounts can serve as natural separation</li>
                <li data-i18n="view.s475.id.s475_e">§ 475(e) commodities — analogous identification</li>
            </ul>
        </div>
    `;
    document.getElementById('s475-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.classification = fd.get('classification');
        state.is_qualified_trader = !!fd.get('is_qualified_trader');
        state.trader_volume_test_passed = !!fd.get('trader_volume_test_passed');
        state.trader_frequency_test_passed = !!fd.get('trader_frequency_test_passed');
        state.days_traded_per_year = Number(fd.get('days_traded_per_year')) || 0;
        state.trades_per_year = Number(fd.get('trades_per_year')) || 0;
        state.avg_holding_days = Number(fd.get('avg_holding_days')) || 0;
        state.short_term_trading_pct = Number(fd.get('short_term_trading_pct')) || 0;
        state.has_substantial_activity = !!fd.get('has_substantial_activity');
        state.s475_f_election_made = !!fd.get('s475_f_election_made');
        state.s475_f_election_year = Number(fd.get('s475_f_election_year')) || 0;
        state.s475_f_election_deadline_april_15 = !!fd.get('s475_f_election_deadline_april_15');
        state.s475_f_election_separate_business = !!fd.get('s475_f_election_separate_business');
        state.fmv_year_end = Number(fd.get('fmv_year_end')) || 0;
        state.adjusted_basis_year_end = Number(fd.get('adjusted_basis_year_end')) || 0;
        state.mtm_gain_loss_recognized = Number(fd.get('mtm_gain_loss_recognized')) || 0;
        state.is_ordinary_character = !!fd.get('is_ordinary_character');
        state.s475_b_dealer_exception = !!fd.get('s475_b_dealer_exception');
        state.s475_a_securities_dealer = !!fd.get('s475_a_securities_dealer');
        state.s475_e_commodities_dealer = !!fd.get('s475_e_commodities_dealer');
        state.no_s1091_wash_sale = !!fd.get('no_s1091_wash_sale');
        state.no_s1259_constructive_sale = !!fd.get('no_s1259_constructive_sale');
        state.no_s1233_short_sale = !!fd.get('no_s1233_short_sale');
        state.no_s263a_capitalization = !!fd.get('no_s263a_capitalization');
        state.schedule_c_business_filer = !!fd.get('schedule_c_business_filer');
        state.s1402_se_tax_exempt = !!fd.get('s1402_se_tax_exempt');
        state.s199a_qbi_eligible = !!fd.get('s199a_qbi_eligible');
        state.is_partnership_trader = !!fd.get('is_partnership_trader');
        state.s475_no_election_for_partnership = !!fd.get('s475_no_election_for_partnership');
        state.s481_a_change_method_required = !!fd.get('s481_a_change_method_required');
        state.s481_a_4_yr_spread = Number(fd.get('s481_a_4_yr_spread')) || 0;
        state.f3115_filed = !!fd.get('f3115_filed');
        state.election_revocation = !!fd.get('election_revocation');
        state.irs_consent_required_for_revocation = !!fd.get('irs_consent_required_for_revocation');
        state.rev_proc_99_17_election = !!fd.get('rev_proc_99_17_election');
        state.rev_proc_2017_30_revocation = !!fd.get('rev_proc_2017_30_revocation');
        state.s212_investor_expenses_2pct = Number(fd.get('s212_investor_expenses_2pct')) || 0;
        state.deduction_above_line_trader = !!fd.get('deduction_above_line_trader');
        state.s165_g_2_worthless_securities = !!fd.get('s165_g_2_worthless_securities');
        state.is_marked_to_market_inventory = !!fd.get('is_marked_to_market_inventory');
        state.s475_c_identification_required = !!fd.get('s475_c_identification_required');
        state.s475_d_attribution = !!fd.get('s475_d_attribution');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s475-output');
    if (!el) return;
    const qualifies = state.days_traded_per_year >= 200 && state.trades_per_year >= 700 && state.avg_holding_days <= 31;
    const mtm = state.fmv_year_end - state.adjusted_basis_year_end;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s475.h2.result">§ 475 trader analysis</h2>
            <div class="cards">
                <div class="card ${qualifies ? 'pos' : 'neg'}"><div class="label" data-i18n="view.s475.card.qualifies">Qualifies as trader?</div><div class="value">${qualifies ? 'LIKELY YES' : 'NO/UNCLEAR'}</div></div>
                <div class="card ${state.s475_f_election_made ? 'pos' : ''}"><div class="label" data-i18n="view.s475.card.elected">§ 475(f) elected?</div><div class="value">${state.s475_f_election_made ? 'YES' : 'NO'}</div></div>
                <div class="card"><div class="label" data-i18n="view.s475.card.mtm">MTM gain/loss</div><div class="value">$${mtm.toLocaleString()}</div></div>
                <div class="card ${state.no_s1091_wash_sale ? 'pos' : 'neg'}"><div class="label" data-i18n="view.s475.card.wash">No wash sale?</div><div class="value">${state.no_s1091_wash_sale ? 'YES' : 'NO'}</div></div>
                <div class="card"><div class="label" data-i18n="view.s475.card.s481">§ 481(a) spread</div><div class="value">$${state.s481_a_4_yr_spread.toLocaleString()}</div></div>
            </div>
        </div>
    `;
}
