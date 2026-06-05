// IRC § 1234 — Options on Property Treated as Property.
// Treats options as having same character (capital vs ordinary) as underlying property.
// § 1234(a) — option holder: gain/loss capital if underlying capital asset.
// § 1234(b) — option writer: special rules — sale/exchange capital but lapse is ordinary.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    position: 'long_call',
    underlying_type: 'equity',
    option_premium: 0,
    option_strike: 0,
    underlying_basis: 0,
    underlying_fmv: 0,
    is_capital_asset_underlying: true,
    days_held: 0,
    is_long_term: false,
    transaction_type: 'sale',
    is_lapse: false,
    is_exercise: false,
    is_close_out: false,
    is_writer: false,
    underlying_actually_received: false,
    s1234_a_character: 'capital',
    s1234_b_character: 'capital',
    s1234_b_writer_lapse: 'ordinary',
    is_dealer: false,
    is_s1234a_4_section: false,
    is_section_1256_contract: false,
    s1256_60_40_split: false,
    is_qualified_covered_call: false,
    s1092_straddle_position: false,
    s1233_short_against_box: false,
    s1259_constructive_sale: false,
    gain_loss_amount: 0,
    holding_period_tacks: false,
    is_LEAP_long_dated: false,
    is_employee_option_nso: false,
    s83_b_election: false,
    is_iso_option: false,
    s421_b_disqualifying: false,
};

export async function renderSection1234(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1234.h1.title">// § 1234 OPTIONS CHARACTER</span></h1>
        <p class="muted small" data-i18n="view.s1234.hint.intro">
            <strong>§ 1234</strong> determines CHARACTER (capital vs ordinary) of gain/loss on options.
            <strong>§ 1234(a) holder rule:</strong> option in/on capital asset → option gain/loss
            is CAPITAL. Underlying is § 1221 capital asset → option follows. <strong>§ 1234(b)
            writer rule:</strong> writer's SALE or EXCHANGE = capital gain/loss. <strong>LAPSE</strong>
            of written option (premium retained without exercise) = ORDINARY income. <strong>§ 1234A
            — termination of contract:</strong> capital treatment for cancellation/lapse/etc. of
            contractual rights. <strong>§ 1256 mark-to-market:</strong> non-equity options, broad-based
            stock index options, etc. get 60% LTCG / 40% STCG regardless of holding period.
            <strong>Qualified covered call (QCC):</strong> § 1092(c) safe harbor exempts written calls
            from straddle rules. <strong>NOT § 1234:</strong> employee stock options (§ 421/§ 422/§ 423).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1234.h2.inputs">Inputs</h2>
            <form id="s1234-form" class="inline-form">
                <label><span data-i18n="view.s1234.label.position">Position</span>
                    <select name="position">
                        <option value="long_call" ${state.position === 'long_call' ? 'selected' : ''}>Long call</option>
                        <option value="long_put" ${state.position === 'long_put' ? 'selected' : ''}>Long put</option>
                        <option value="short_call" ${state.position === 'short_call' ? 'selected' : ''}>Short call (written)</option>
                        <option value="short_put" ${state.position === 'short_put' ? 'selected' : ''}>Short put (written)</option>
                        <option value="warrant" ${state.position === 'warrant' ? 'selected' : ''}>Warrant</option>
                        <option value="convertible" ${state.position === 'convertible' ? 'selected' : ''}>Convertible (embedded)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s1234.label.underlying">Underlying type</span>
                    <select name="underlying_type">
                        <option value="equity" ${state.underlying_type === 'equity' ? 'selected' : ''}>Equity / stock</option>
                        <option value="non_equity" ${state.underlying_type === 'non_equity' ? 'selected' : ''}>Non-equity (§ 1256)</option>
                        <option value="commodity" ${state.underlying_type === 'commodity' ? 'selected' : ''}>Commodity / future</option>
                        <option value="currency" ${state.underlying_type === 'currency' ? 'selected' : ''}>Foreign currency</option>
                        <option value="index" ${state.underlying_type === 'index' ? 'selected' : ''}>Broad-based stock index</option>
                        <option value="single_stock" ${state.underlying_type === 'single_stock' ? 'selected' : ''}>Single-stock (NOT § 1256)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s1234.label.premium">Option premium ($)</span>
                    <input type="number" step="0.01" name="option_premium" value="${state.option_premium}"></label>
                <label><span data-i18n="view.s1234.label.strike">Option strike ($)</span>
                    <input type="number" step="1" name="option_strike" value="${state.option_strike}"></label>
                <label><span data-i18n="view.s1234.label.basis">Underlying basis ($)</span>
                    <input type="number" step="0.01" name="underlying_basis" value="${state.underlying_basis}"></label>
                <label><span data-i18n="view.s1234.label.fmv">Underlying FMV ($)</span>
                    <input type="number" step="0.01" name="underlying_fmv" value="${state.underlying_fmv}"></label>
                <label><span data-i18n="view.s1234.label.is_capital">Capital asset?</span>
                    <input type="checkbox" name="is_capital_asset_underlying" ${state.is_capital_asset_underlying ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1234.label.days">Days held</span>
                    <input type="number" step="1" name="days_held" value="${state.days_held}"></label>
                <label><span data-i18n="view.s1234.label.lt">Long-term?</span>
                    <input type="checkbox" name="is_long_term" ${state.is_long_term ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1234.label.tx_type">Transaction type</span>
                    <select name="transaction_type">
                        <option value="sale" ${state.transaction_type === 'sale' ? 'selected' : ''}>Sale</option>
                        <option value="exercise" ${state.transaction_type === 'exercise' ? 'selected' : ''}>Exercise</option>
                        <option value="lapse" ${state.transaction_type === 'lapse' ? 'selected' : ''}>Lapse / expire worthless</option>
                        <option value="close_out" ${state.transaction_type === 'close_out' ? 'selected' : ''}>Close out (offset)</option>
                        <option value="assignment" ${state.transaction_type === 'assignment' ? 'selected' : ''}>Assignment</option>
                    </select>
                </label>
                <label><span data-i18n="view.s1234.label.lapse">Lapse?</span>
                    <input type="checkbox" name="is_lapse" ${state.is_lapse ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1234.label.exercise">Exercise?</span>
                    <input type="checkbox" name="is_exercise" ${state.is_exercise ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1234.label.close">Close out?</span>
                    <input type="checkbox" name="is_close_out" ${state.is_close_out ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1234.label.writer">Writer?</span>
                    <input type="checkbox" name="is_writer" ${state.is_writer ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1234.label.underlying_received">Underlying actually received?</span>
                    <input type="checkbox" name="underlying_actually_received" ${state.underlying_actually_received ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1234.label.dealer">Dealer?</span>
                    <input type="checkbox" name="is_dealer" ${state.is_dealer ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1234.label.s1234a4">§ 1234A(4) ordinary?</span>
                    <input type="checkbox" name="is_s1234a_4_section" ${state.is_s1234a_4_section ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1234.label.s1256">§ 1256 contract?</span>
                    <input type="checkbox" name="is_section_1256_contract" ${state.is_section_1256_contract ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1234.label.s1256_60_40">§ 1256 60/40 split?</span>
                    <input type="checkbox" name="s1256_60_40_split" ${state.s1256_60_40_split ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1234.label.qcc">Qualified covered call?</span>
                    <input type="checkbox" name="is_qualified_covered_call" ${state.is_qualified_covered_call ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1234.label.straddle">§ 1092 straddle?</span>
                    <input type="checkbox" name="s1092_straddle_position" ${state.s1092_straddle_position ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1234.label.s1233">§ 1233 short against box?</span>
                    <input type="checkbox" name="s1233_short_against_box" ${state.s1233_short_against_box ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1234.label.s1259">§ 1259 constructive sale?</span>
                    <input type="checkbox" name="s1259_constructive_sale" ${state.s1259_constructive_sale ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1234.label.gain">Gain/loss ($)</span>
                    <input type="number" step="0.01" name="gain_loss_amount" value="${state.gain_loss_amount}"></label>
                <label><span data-i18n="view.s1234.label.tacks">Holding tacks?</span>
                    <input type="checkbox" name="holding_period_tacks" ${state.holding_period_tacks ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1234.label.leap">LEAP long-dated?</span>
                    <input type="checkbox" name="is_LEAP_long_dated" ${state.is_LEAP_long_dated ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1234.label.nso">Employee NSO?</span>
                    <input type="checkbox" name="is_employee_option_nso" ${state.is_employee_option_nso ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1234.label.s83b">§ 83(b) election?</span>
                    <input type="checkbox" name="s83_b_election" ${state.s83_b_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1234.label.iso">ISO option?</span>
                    <input type="checkbox" name="is_iso_option" ${state.is_iso_option ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1234.label.s421b">§ 421(b) disqualifying?</span>
                    <input type="checkbox" name="s421_b_disqualifying" ${state.s421_b_disqualifying ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s1234.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1234-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1234.h2.holder_rules">§ 1234(a) holder rules</h2>
            <ol class="muted small">
                <li data-i18n="view.s1234.h.character">Option in/on capital asset → option gain/loss is CAPITAL</li>
                <li data-i18n="view.s1234.h.sale">Sale of option: ordinary gain/loss UNLESS § 1234(a) applies (capital)</li>
                <li data-i18n="view.s1234.h.lapse_loss">Lapse without exercise: capital LOSS treated as if sale for $0 (Reg § 1.1234-1(b))</li>
                <li data-i18n="view.s1234.h.exercise_basis">Exercise: premium added to basis of underlying acquired</li>
                <li data-i18n="view.s1234.h.exercise_no_gain">No gain/loss at exercise — built into underlying's basis</li>
                <li data-i18n="view.s1234.h.put_exercise">Put exercise (sale of underlying): premium REDUCES sale proceeds</li>
                <li data-i18n="view.s1234.h.holding_starts_separately">Holding period for underlying STARTS at exercise — does NOT tack to option period</li>
                <li data-i18n="view.s1234.h.s1234_a_3">§ 1234(a)(3) special holding period rule for cash-settled options on capital assets</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1234.h2.writer_rules">§ 1234(b) writer rules</h2>
            <ol class="muted small">
                <li data-i18n="view.s1234.w.sale">Sale or exchange by writer: SHORT-TERM CAPITAL gain/loss</li>
                <li data-i18n="view.s1234.w.lapse">Lapse with premium retained: ORDINARY income (premium received, no offsetting outflow)</li>
                <li data-i18n="view.s1234.w.s1234_b_repeal">§ 1234(b) special lapse rule applies BEFORE § 1234A general termination rule</li>
                <li data-i18n="view.s1234.w.assignment_call">Assignment of call: writer's basis in underlying + premium = adjusted sale price</li>
                <li data-i18n="view.s1234.w.assignment_put">Assignment of put: writer's basis in underlying = strike - premium received</li>
                <li data-i18n="view.s1234.w.close_out">Close-out: STCG/STCL (writer's position generally short-term character)</li>
                <li data-i18n="view.s1234.w.no_LT_writer">Writer never gets LT character (short positions can't be held LT)</li>
                <li data-i18n="view.s1234.w.s1234_b_2">§ 1234(b)(2) — dealer exception (§ 1.1234-3 exception)</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1234.h2.s1256">§ 1256 mark-to-market contracts</h2>
            <ul class="muted small">
                <li data-i18n="view.s1234.s1256.types">Non-equity options, broad-based stock indices, regulated futures, foreign currency contracts</li>
                <li data-i18n="view.s1234.s1256.split">60% LONG-term + 40% SHORT-term regardless of actual holding</li>
                <li data-i18n="view.s1234.s1256.mtm">Mark-to-market at year-end: deemed sold at FMV Dec 31</li>
                <li data-i18n="view.s1234.s1256.spx">SPX, SPY ARE different: SPY (single-stock ETF) NOT § 1256; SPX (index option) IS § 1256</li>
                <li data-i18n="view.s1234.s1256.qualified_board">Must trade on qualified board / exchange (CFTC-regulated typically)</li>
                <li data-i18n="view.s1234.s1256.s1256_d">§ 1256(d) hedging exception: bona fide hedge → ordinary character</li>
                <li data-i18n="view.s1234.s1256.f6781">Form 6781 reports 60/40 calc</li>
                <li data-i18n="view.s1234.s1256.carryback">3-year carryback unique to § 1256 losses (§ 1212(c))</li>
                <li data-i18n="view.s1234.s1256.mixed_straddle">Mixed straddle: § 1256 + non-§ 1256 — complex elections</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1234.h2.straddle">§ 1092 straddle + § 263(g)</h2>
            <ul class="muted small">
                <li data-i18n="view.s1234.str.defined">"Straddle" = offsetting positions in personal property (e.g., long stock + put)</li>
                <li data-i18n="view.s1234.str.s1092_a">§ 1092(a) — loss deferral: loss recognized only to extent NOT offset by unrecognized gain</li>
                <li data-i18n="view.s1234.str.s1092_b">§ 1092(b) — capitalization: interest/carrying charges capitalized to basis</li>
                <li data-i18n="view.s1234.str.s1092_c_qcc">§ 1092(c)(4)(A) — Qualified Covered Call exception (limited stocks + LT positions)</li>
                <li data-i18n="view.s1234.str.s263_g">§ 263(g) — capitalize carrying charges on straddle positions</li>
                <li data-i18n="view.s1234.str.s1092_d">§ 1092(d) — broadens definition of "personal property"</li>
                <li data-i18n="view.s1234.str.identified">Identified straddle: special set of rules — basis adjustments</li>
                <li data-i18n="view.s1234.str.s1234_b_holding_term">Straddle terminates LT holding period of long leg (rolls back to date of offsetting position)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1234.h2.special_options">Special option types</h2>
            <ul class="muted small">
                <li data-i18n="view.s1234.spec.lap">LEAP (Long-term Equity AnticiPation Securities) — 1+ year expiry, same § 1234 treatment</li>
                <li data-i18n="view.s1234.spec.warrant">Warrants: § 1234 applies; cost basis adjusts by premium</li>
                <li data-i18n="view.s1234.spec.bond_call">Bond call/put features: § 1271(a) interest treatment, not § 1234</li>
                <li data-i18n="view.s1234.spec.iso">ISO (incentive stock option): § 422 — NOT § 1234 (employee compensation)</li>
                <li data-i18n="view.s1234.spec.nso">NSO (non-qualified stock option): § 421 + § 83 — NOT § 1234 (compensation)</li>
                <li data-i18n="view.s1234.spec.s423_espp">§ 423 ESPP: NOT § 1234 (qualified plan)</li>
                <li data-i18n="view.s1234.spec.s1233_short">§ 1233 short sales: holding period rules differ for matched positions</li>
                <li data-i18n="view.s1234.spec.s1259">§ 1259 constructive sale — short-against-box + similar offsetting positions</li>
                <li data-i18n="view.s1234.spec.equity_swap">Equity swap (NPC): § 871(m) dividend equivalents</li>
            </ul>
        </div>
    `;
    document.getElementById('s1234-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.position = fd.get('position');
        state.underlying_type = fd.get('underlying_type');
        state.option_premium = Number(fd.get('option_premium')) || 0;
        state.option_strike = Number(fd.get('option_strike')) || 0;
        state.underlying_basis = Number(fd.get('underlying_basis')) || 0;
        state.underlying_fmv = Number(fd.get('underlying_fmv')) || 0;
        state.is_capital_asset_underlying = !!fd.get('is_capital_asset_underlying');
        state.days_held = Number(fd.get('days_held')) || 0;
        state.is_long_term = !!fd.get('is_long_term');
        state.transaction_type = fd.get('transaction_type');
        state.is_lapse = !!fd.get('is_lapse');
        state.is_exercise = !!fd.get('is_exercise');
        state.is_close_out = !!fd.get('is_close_out');
        state.is_writer = !!fd.get('is_writer');
        state.underlying_actually_received = !!fd.get('underlying_actually_received');
        state.is_dealer = !!fd.get('is_dealer');
        state.is_s1234a_4_section = !!fd.get('is_s1234a_4_section');
        state.is_section_1256_contract = !!fd.get('is_section_1256_contract');
        state.s1256_60_40_split = !!fd.get('s1256_60_40_split');
        state.is_qualified_covered_call = !!fd.get('is_qualified_covered_call');
        state.s1092_straddle_position = !!fd.get('s1092_straddle_position');
        state.s1233_short_against_box = !!fd.get('s1233_short_against_box');
        state.s1259_constructive_sale = !!fd.get('s1259_constructive_sale');
        state.gain_loss_amount = Number(fd.get('gain_loss_amount')) || 0;
        state.holding_period_tacks = !!fd.get('holding_period_tacks');
        state.is_LEAP_long_dated = !!fd.get('is_LEAP_long_dated');
        state.is_employee_option_nso = !!fd.get('is_employee_option_nso');
        state.s83_b_election = !!fd.get('s83_b_election');
        state.is_iso_option = !!fd.get('is_iso_option');
        state.s421_b_disqualifying = !!fd.get('s421_b_disqualifying');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1234-output');
    if (!el) return;
    let character = 'CAPITAL';
    if (state.is_writer && state.is_lapse) character = 'ORDINARY (writer lapse)';
    else if (!state.is_capital_asset_underlying) character = 'ORDINARY (non-capital underlying)';
    else if (state.is_section_1256_contract) character = '60% LTCG / 40% STCG';
    else if (state.is_writer) character = 'STCG (writer always short-term)';
    else if (state.is_long_term) character = 'LTCG';
    else character = 'STCG';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1234.h2.result">§ 1234 character + result</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s1234.card.position">Position</div><div class="value">${esc(state.position)}</div></div>
                <div class="card"><div class="label" data-i18n="view.s1234.card.character">Character</div><div class="value">${character}</div></div>
                <div class="card"><div class="label" data-i18n="view.s1234.card.gain">Gain / loss</div><div class="value">$${state.gain_loss_amount.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s1234.card.lapse">Lapse?</div><div class="value">${state.is_lapse ? 'YES' : 'NO'}</div></div>
                <div class="card ${state.is_section_1256_contract ? 'warn' : ''}"><div class="label" data-i18n="view.s1234.card.s1256">§ 1256?</div><div class="value">${state.is_section_1256_contract ? 'YES (60/40 + MTM)' : 'NO'}</div></div>
            </div>
        </div>
    `;
}
