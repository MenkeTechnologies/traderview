// IRC § 6045A — Information Required in Connection with Transfers of Covered Securities.
// Broker must furnish basis statement to receiving broker upon transfer of covered securities.
// Coordinates with § 6045 broker reporting (Form 1099-B) + § 1012 average cost basis rules.

import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    transferring_broker: '',
    receiving_broker: '',
    security_type: 'equity',
    is_covered_security: true,
    acquisition_date: '',
    transfer_date: '',
    cusip: '',
    shares_transferred: 0,
    original_basis: 0,
    holding_period: 'long_term',
    s1012_average_cost: false,
    s1012_specific_id: false,
    s1012_fifo: true,
    is_dividend_reinvestment: false,
    drip_basis: 0,
    wash_sale_disallowed: 0,
    s1259_constructive_sale: false,
    s1233_short_sale: false,
    s1091_wash_sale_active: false,
    days_late_furnishing: 0,
    is_failure_to_furnish: false,
    s6722_penalty_per_form: 0,
    forms_failed_count: 0,
    intentional_disregard: false,
    s6045a_furnished: true,
    is_account_to_account_transfer: false,
    is_transfer_in_kind: false,
    rev_proc_2009_29_safe: false,
    short_term_pre_2014: false,
};

export async function renderSection6045A(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6045a.h1.title">// § 6045A COVERED SECURITY TRANSFER REPORTING</span></h1>
        <p class="muted small" data-i18n="view.s6045a.hint.intro">
            <strong>§ 6045A</strong> requires transferring broker to furnish basis statement to
            RECEIVING broker upon transfer of covered securities. <strong>"Covered security"</strong>
            = acquired after applicable phase-in date: equity 2011, mutual fund / DRIP 2012, debt /
            options 2014, simple debt 2015, complex debt 2016. <strong>15-day window</strong>: statement
            due within 15 days of transfer (Treas. Reg. § 1.6045A-1). <strong>Information required:</strong>
            CUSIP, # shares, acquisition date, adjusted basis, original basis, S/T vs L/T classification,
            wash sale disallowance, § 1259 constructive sale flag, average cost method election.
            <strong>§ 6722 penalty</strong> for failure to furnish: $60 / $130 / $310 / $630 (intentional).
            <strong>§ 6045B</strong> separately requires organizational actions reporting (mergers, splits, etc).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6045a.h2.inputs">Inputs</h2>
            <form id="s6045a-form" class="inline-form">
                <label><span data-i18n="view.s6045a.label.transferring">Transferring broker</span>
                    <input type="text" name="transferring_broker" value="${esc(state.transferring_broker)}"></label>
                <label><span data-i18n="view.s6045a.label.receiving">Receiving broker</span>
                    <input type="text" name="receiving_broker" value="${esc(state.receiving_broker)}"></label>
                <label><span data-i18n="view.s6045a.label.security_type">Security type</span>
                    <select name="security_type">
                        <option value="equity" ${state.security_type === 'equity' ? 'selected' : ''}>Equity (post-2011)</option>
                        <option value="mutual_fund" ${state.security_type === 'mutual_fund' ? 'selected' : ''}>Mutual fund / DRIP (post-2012)</option>
                        <option value="simple_debt" ${state.security_type === 'simple_debt' ? 'selected' : ''}>Simple debt (post-2014)</option>
                        <option value="complex_debt" ${state.security_type === 'complex_debt' ? 'selected' : ''}>Complex debt (post-2016)</option>
                        <option value="options" ${state.security_type === 'options' ? 'selected' : ''}>Options (post-2014)</option>
                        <option value="dvp" ${state.security_type === 'dvp' ? 'selected' : ''}>DvP (post-2014)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6045a.label.covered">Covered security?</span>
                    <input type="checkbox" name="is_covered_security" ${state.is_covered_security ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6045a.label.acq">Acquisition date</span>
                    <input type="date" name="acquisition_date" value="${state.acquisition_date}"></label>
                <label><span data-i18n="view.s6045a.label.transfer">Transfer date</span>
                    <input type="date" name="transfer_date" value="${state.transfer_date}"></label>
                <label><span data-i18n="view.s6045a.label.cusip">CUSIP</span>
                    <input type="text" name="cusip" value="${esc(state.cusip)}"></label>
                <label><span data-i18n="view.s6045a.label.shares">Shares</span>
                    <input type="number" step="1" name="shares_transferred" value="${state.shares_transferred}"></label>
                <label><span data-i18n="view.s6045a.label.basis">Original basis ($)</span>
                    <input type="number" step="0.01" name="original_basis" value="${state.original_basis}"></label>
                <label><span data-i18n="view.s6045a.label.holding">Holding period</span>
                    <select name="holding_period">
                        <option value="long_term" ${state.holding_period === 'long_term' ? 'selected' : ''}>Long-term (&gt; 1 yr)</option>
                        <option value="short_term" ${state.holding_period === 'short_term' ? 'selected' : ''}>Short-term (≤ 1 yr)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6045a.label.average">§ 1012 average cost?</span>
                    <input type="checkbox" name="s1012_average_cost" ${state.s1012_average_cost ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6045a.label.spec_id">§ 1012 specific ID?</span>
                    <input type="checkbox" name="s1012_specific_id" ${state.s1012_specific_id ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6045a.label.fifo">§ 1012 FIFO default?</span>
                    <input type="checkbox" name="s1012_fifo" ${state.s1012_fifo ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6045a.label.drip">DRIP?</span>
                    <input type="checkbox" name="is_dividend_reinvestment" ${state.is_dividend_reinvestment ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6045a.label.drip_amt">DRIP basis ($)</span>
                    <input type="number" step="0.01" name="drip_basis" value="${state.drip_basis}"></label>
                <label><span data-i18n="view.s6045a.label.wash">Wash sale disallowed ($)</span>
                    <input type="number" step="0.01" name="wash_sale_disallowed" value="${state.wash_sale_disallowed}"></label>
                <label><span data-i18n="view.s6045a.label.s1259">§ 1259 constructive sale?</span>
                    <input type="checkbox" name="s1259_constructive_sale" ${state.s1259_constructive_sale ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6045a.label.s1233">§ 1233 short sale?</span>
                    <input type="checkbox" name="s1233_short_sale" ${state.s1233_short_sale ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6045a.label.s1091">§ 1091 wash sale active?</span>
                    <input type="checkbox" name="s1091_wash_sale_active" ${state.s1091_wash_sale_active ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6045a.label.days">Days late furnishing</span>
                    <input type="number" step="1" name="days_late_furnishing" value="${state.days_late_furnishing}"></label>
                <label><span data-i18n="view.s6045a.label.failure">Failure to furnish?</span>
                    <input type="checkbox" name="is_failure_to_furnish" ${state.is_failure_to_furnish ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6045a.label.s6722">§ 6722 per-form ($)</span>
                    <input type="number" step="10" name="s6722_penalty_per_form" value="${state.s6722_penalty_per_form}"></label>
                <label><span data-i18n="view.s6045a.label.count">Forms failed</span>
                    <input type="number" step="1" name="forms_failed_count" value="${state.forms_failed_count}"></label>
                <label><span data-i18n="view.s6045a.label.intentional">Intentional disregard?</span>
                    <input type="checkbox" name="intentional_disregard" ${state.intentional_disregard ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6045a.label.furnished">§ 6045A furnished?</span>
                    <input type="checkbox" name="s6045a_furnished" ${state.s6045a_furnished ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6045a.label.acct2acct">Account-to-account?</span>
                    <input type="checkbox" name="is_account_to_account_transfer" ${state.is_account_to_account_transfer ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6045a.label.in_kind">In-kind transfer?</span>
                    <input type="checkbox" name="is_transfer_in_kind" ${state.is_transfer_in_kind ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6045a.label.safe">Rev Proc 2009-29 safe?</span>
                    <input type="checkbox" name="rev_proc_2009_29_safe" ${state.rev_proc_2009_29_safe ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6045a.label.st_pre">S/T pre-2014?</span>
                    <input type="checkbox" name="short_term_pre_2014" ${state.short_term_pre_2014 ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s6045a.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6045a-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6045a.h2.phase_in">Covered security phase-in dates</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s6045a.tbl.security">Security type</th><th data-i18n="view.s6045a.tbl.effective">Effective date</th><th data-i18n="view.s6045a.tbl.citation">Reg citation</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s6045a.tbl.equity">Equity (corporate stock)</td><td>Jan 1, 2011</td><td>§ 1.6045-1(d)(2)</td></tr>
                    <tr><td data-i18n="view.s6045a.tbl.mf_drip">Mutual fund + DRIP</td><td>Jan 1, 2012</td><td>§ 1.6045-1(c)(3)</td></tr>
                    <tr><td data-i18n="view.s6045a.tbl.simple_debt">Simple debt (fixed maturity, fixed rate)</td><td>Jan 1, 2014</td><td>§ 1.6045-1(n)(2)</td></tr>
                    <tr><td data-i18n="view.s6045a.tbl.complex_debt">Complex debt (variable rate, contingent, etc.)</td><td>Jan 1, 2016</td><td>§ 1.6045-1(n)(2)(iii)</td></tr>
                    <tr><td data-i18n="view.s6045a.tbl.options">Options</td><td>Jan 1, 2014</td><td>§ 1.6045-1(d)(7)</td></tr>
                    <tr><td data-i18n="view.s6045a.tbl.dvp">DvP accounts</td><td>Jan 1, 2014</td><td>§ 1.6045-1(d)(11)</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6045a.h2.required_info">Required transfer statement information</h2>
            <ol class="muted small">
                <li data-i18n="view.s6045a.req.cusip">CUSIP / security identifier</li>
                <li data-i18n="view.s6045a.req.shares">Number of shares / face amount</li>
                <li data-i18n="view.s6045a.req.acq_date">Acquisition date (or "various" if multiple)</li>
                <li data-i18n="view.s6045a.req.basis">Original basis + adjusted basis</li>
                <li data-i18n="view.s6045a.req.holding">Short-term vs long-term holding period</li>
                <li data-i18n="view.s6045a.req.wash_sale">Wash sale disallowance under § 1091</li>
                <li data-i18n="view.s6045a.req.s1259">§ 1259 constructive sale flag (if applicable)</li>
                <li data-i18n="view.s6045a.req.method">Cost basis method (FIFO / specific ID / average)</li>
                <li data-i18n="view.s6045a.req.drip">DRIP indicator + adjusted basis</li>
                <li data-i18n="view.s6045a.req.s305c">§ 305(c) stock dividends + § 351 / § 354 / § 355 reorganization adjustments</li>
                <li data-i18n="view.s6045a.req.transferor">Transferor TIN + name + address</li>
                <li data-i18n="view.s6045a.req.transferee">Transferee TIN + name + address</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6045a.h2.cost_basis_methods">§ 1012 cost basis methods</h2>
            <ul class="muted small">
                <li data-i18n="view.s6045a.method.fifo">FIFO (First In First Out) — default for equities</li>
                <li data-i18n="view.s6045a.method.specific">Specific identification — must elect by sale date</li>
                <li data-i18n="view.s6045a.method.average">Average cost — only mutual funds / DRIPs eligible</li>
                <li data-i18n="view.s6045a.method.average_election">Average cost election: binding for that asset class + can change account-by-account</li>
                <li data-i18n="view.s6045a.method.spec_id">Specific ID: must identify before settlement date (Rev. Rul. 73-330)</li>
                <li data-i18n="view.s6045a.method.lift">LIFO not permitted by IRS (Reg § 1.1012-1)</li>
                <li data-i18n="view.s6045a.method.high_low">"High cost" or "low cost" allowed within FIFO/specific ID framework</li>
                <li data-i18n="view.s6045a.method.tax_loss_harvest">Tax loss harvesting: specific ID strongly favored for control</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6045a.h2.penalties">§ 6722 furnishing penalties (per recipient)</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s6045a.tbl.violation">Violation</th><th data-i18n="view.s6045a.tbl.amt">Per form</th><th data-i18n="view.s6045a.tbl.cap">Annual cap</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s6045a.tbl.30days">Furnished within 30 days</td><td>$60</td><td>$232,500</td></tr>
                    <tr><td data-i18n="view.s6045a.tbl.aug1">Furnished after 30 days but by Aug 1</td><td>$130</td><td>$664,500</td></tr>
                    <tr><td data-i18n="view.s6045a.tbl.aftaug1">Furnished after Aug 1 or not at all</td><td>$310</td><td>$3,783,000</td></tr>
                    <tr><td data-i18n="view.s6045a.tbl.intentional">Intentional disregard</td><td>$630</td><td>NO CAP</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6045a.h2.related">Related reporting + interactions</h2>
            <ul class="muted small">
                <li data-i18n="view.s6045a.rel.s6045">§ 6045 broker reporting Form 1099-B (sale reporting)</li>
                <li data-i18n="view.s6045a.rel.s6045b">§ 6045B organizational actions (mergers, splits, spin-offs) — Form 8937</li>
                <li data-i18n="view.s6045a.rel.s6049">§ 6049 interest reporting (1099-INT)</li>
                <li data-i18n="view.s6045a.rel.s1091">§ 1091 wash sale — track 30-day window across accounts</li>
                <li data-i18n="view.s6045a.rel.s1259">§ 1259 constructive sale of appreciated financial position</li>
                <li data-i18n="view.s6045a.rel.s1233">§ 1233 short sale rules</li>
                <li data-i18n="view.s6045a.rel.s1234">§ 1234 options character</li>
                <li data-i18n="view.s6045a.rel.s305c">§ 305(c) basis adjustment on deemed distributions</li>
                <li data-i18n="view.s6045a.rel.s301">§ 301(c)(3) return of capital reduces basis</li>
                <li data-i18n="view.s6045a.rel.foreign">FATCA Chapter 4 withholding interacts with reporting</li>
            </ul>
        </div>
    `;
    document.getElementById('s6045a-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.transferring_broker = fd.get('transferring_broker') || '';
        state.receiving_broker = fd.get('receiving_broker') || '';
        state.security_type = fd.get('security_type');
        state.is_covered_security = !!fd.get('is_covered_security');
        state.acquisition_date = fd.get('acquisition_date') || '';
        state.transfer_date = fd.get('transfer_date') || '';
        state.cusip = fd.get('cusip') || '';
        state.shares_transferred = Number(fd.get('shares_transferred')) || 0;
        state.original_basis = Number(fd.get('original_basis')) || 0;
        state.holding_period = fd.get('holding_period');
        state.s1012_average_cost = !!fd.get('s1012_average_cost');
        state.s1012_specific_id = !!fd.get('s1012_specific_id');
        state.s1012_fifo = !!fd.get('s1012_fifo');
        state.is_dividend_reinvestment = !!fd.get('is_dividend_reinvestment');
        state.drip_basis = Number(fd.get('drip_basis')) || 0;
        state.wash_sale_disallowed = Number(fd.get('wash_sale_disallowed')) || 0;
        state.s1259_constructive_sale = !!fd.get('s1259_constructive_sale');
        state.s1233_short_sale = !!fd.get('s1233_short_sale');
        state.s1091_wash_sale_active = !!fd.get('s1091_wash_sale_active');
        state.days_late_furnishing = Number(fd.get('days_late_furnishing')) || 0;
        state.is_failure_to_furnish = !!fd.get('is_failure_to_furnish');
        state.s6722_penalty_per_form = Number(fd.get('s6722_penalty_per_form')) || 0;
        state.forms_failed_count = Number(fd.get('forms_failed_count')) || 0;
        state.intentional_disregard = !!fd.get('intentional_disregard');
        state.s6045a_furnished = !!fd.get('s6045a_furnished');
        state.is_account_to_account_transfer = !!fd.get('is_account_to_account_transfer');
        state.is_transfer_in_kind = !!fd.get('is_transfer_in_kind');
        state.rev_proc_2009_29_safe = !!fd.get('rev_proc_2009_29_safe');
        state.short_term_pre_2014 = !!fd.get('short_term_pre_2014');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6045a-output');
    if (!el) return;
    let per_form = 0;
    if (state.intentional_disregard) per_form = 630;
    else if (state.days_late_furnishing > 213) per_form = 310;
    else if (state.days_late_furnishing > 30) per_form = 130;
    else if (state.days_late_furnishing > 0) per_form = 60;
    const total = per_form * state.forms_failed_count;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6045a.h2.result">§ 6045A penalty assessment</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s6045a.card.security">Security type</div><div class="value">${esc(state.security_type)}</div></div>
                <div class="card ${state.is_covered_security ? 'pos' : ''}"><div class="label" data-i18n="view.s6045a.card.covered">Covered?</div><div class="value">${state.is_covered_security ? 'YES' : 'NO'}</div></div>
                <div class="card"><div class="label" data-i18n="view.s6045a.card.per">Per-form penalty</div><div class="value">$${per_form.toLocaleString()}</div></div>
                <div class="card ${total > 0 ? 'neg' : 'pos'}"><div class="label" data-i18n="view.s6045a.card.total">Total § 6722 penalty</div><div class="value">$${total.toLocaleString()}</div></div>
            </div>
        </div>
    `;
}
