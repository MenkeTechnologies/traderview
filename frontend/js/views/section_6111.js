// IRC § 6111 — Material Advisor Disclosure (Form 8918) for Reportable Transactions.
// Material advisors of "reportable transactions" must file Form 8918 by last day of month
// following calendar quarter in which becomes material advisor.
// Pairs with § 6707 ($50K-$200K civil + greater of $200K or 75% gross income) + § 6707A taxpayer.

import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    is_material_advisor: false,
    threshold_fee_met: false,
    fee_amount: 0,
    is_reportable_transaction: false,
    rt_type: 'listed',
    quarter_became_advisor: 'Q1',
    year: 2024,
    form_8918_filed: false,
    days_late: 0,
    is_failure_to_file: false,
    list_maintained: false,
    gross_income_from_advice: 0,
    is_listed_transaction: false,
    intentional_disregard: false,
};

export async function renderSection6111(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6111.h1.title">// § 6111 MATERIAL ADVISOR DISCLOSURE</span></h1>
        <p class="muted small" data-i18n="view.s6111.hint.intro">
            <strong>Material advisor</strong> of "reportable transaction" must file <strong>Form 8918</strong>
            by last day of month following calendar quarter. <strong>"Material advisor"</strong> =
            provides material aid/assistance/advice AND directly/indirectly receives gross income
            <strong>$50K (natural person)</strong> or <strong>$250K (any other)</strong> for listed,
            <strong>$10K / $25K</strong> for other reportable. <strong>§ 6707 penalty:</strong> $50,000
            (other reportable) OR greater of <strong>$200,000</strong> OR <strong>75% gross income</strong>
            (listed/§ 6707A intentional disregard). <strong>§ 6112 list maintenance</strong> separately
            penalized $10,000/day after IRS request. <strong>5 categories:</strong> listed,
            confidential, contractual protection, loss, transactions of interest.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6111.h2.inputs">Inputs</h2>
            <form id="s6111-form" class="inline-form">
                <label><span data-i18n="view.s6111.label.material">Material advisor?</span>
                    <input type="checkbox" name="is_material_advisor" ${state.is_material_advisor ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6111.label.threshold">Fee threshold met?</span>
                    <input type="checkbox" name="threshold_fee_met" ${state.threshold_fee_met ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6111.label.fee">Fee amount ($)</span>
                    <input type="number" step="0.01" name="fee_amount" value="${state.fee_amount}"></label>
                <label><span data-i18n="view.s6111.label.reportable">Reportable transaction?</span>
                    <input type="checkbox" name="is_reportable_transaction" ${state.is_reportable_transaction ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6111.label.rt_type">RT category</span>
                    <select name="rt_type">
                        <option value="listed" ${state.rt_type === 'listed' ? 'selected' : ''}>Listed (Notice 2009-7 etc.)</option>
                        <option value="confidential" ${state.rt_type === 'confidential' ? 'selected' : ''}>Confidential (limit on disclosure)</option>
                        <option value="contractual_protection" ${state.rt_type === 'contractual_protection' ? 'selected' : ''}>Contractual protection (refund if not work)</option>
                        <option value="loss" ${state.rt_type === 'loss' ? 'selected' : ''}>Loss ($10M C-corp / $2M individual / $5M partner)</option>
                        <option value="transactions_of_interest" ${state.rt_type === 'transactions_of_interest' ? 'selected' : ''}>Transactions of interest (TOI)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6111.label.quarter">Quarter became advisor</span>
                    <select name="quarter_became_advisor">
                        <option value="Q1" ${state.quarter_became_advisor === 'Q1' ? 'selected' : ''}>Q1 (Mar 31) → due Apr 30</option>
                        <option value="Q2" ${state.quarter_became_advisor === 'Q2' ? 'selected' : ''}>Q2 (Jun 30) → due Jul 31</option>
                        <option value="Q3" ${state.quarter_became_advisor === 'Q3' ? 'selected' : ''}>Q3 (Sep 30) → due Oct 31</option>
                        <option value="Q4" ${state.quarter_became_advisor === 'Q4' ? 'selected' : ''}>Q4 (Dec 31) → due Jan 31</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6111.label.year">Year</span>
                    <input type="number" step="1" name="year" value="${state.year}"></label>
                <label><span data-i18n="view.s6111.label.filed">Form 8918 filed?</span>
                    <input type="checkbox" name="form_8918_filed" ${state.form_8918_filed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6111.label.days_late">Days late</span>
                    <input type="number" step="1" name="days_late" value="${state.days_late}"></label>
                <label><span data-i18n="view.s6111.label.failure">Failure to file?</span>
                    <input type="checkbox" name="is_failure_to_file" ${state.is_failure_to_file ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6111.label.list">§ 6112 list maintained?</span>
                    <input type="checkbox" name="list_maintained" ${state.list_maintained ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6111.label.gross">Gross income from advice ($)</span>
                    <input type="number" step="0.01" name="gross_income_from_advice" value="${state.gross_income_from_advice}"></label>
                <label><span data-i18n="view.s6111.label.listed">Listed transaction?</span>
                    <input type="checkbox" name="is_listed_transaction" ${state.is_listed_transaction ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6111.label.intentional">Intentional disregard?</span>
                    <input type="checkbox" name="intentional_disregard" ${state.intentional_disregard ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s6111.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6111-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6111.h2.categories">5 categories of reportable transactions (Reg § 1.6011-4)</h2>
            <ol class="muted small">
                <li data-i18n="view.s6111.cat.listed">Listed — Notice 2009-7 abusive transactions (variable annuities, killer Bs, etc.)</li>
                <li data-i18n="view.s6111.cat.confidential">Confidential — fee paid for tax advice + advisor places restriction on disclosure</li>
                <li data-i18n="view.s6111.cat.contractual">Contractual protection — refund/reduction if no tax benefit</li>
                <li data-i18n="view.s6111.cat.loss">Loss transactions — $10M (C-corp), $2M (individual), $5M (partnership)</li>
                <li data-i18n="view.s6111.cat.toi">Transactions of Interest — IRS designates as warranting scrutiny (notice/rev. proc.)</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6111.h2.thresholds">Material advisor fee thresholds</h2>
            <ul class="muted small">
                <li data-i18n="view.s6111.thresh.listed_natural">Listed transaction, natural person: $50,000 gross income</li>
                <li data-i18n="view.s6111.thresh.listed_other">Listed transaction, other (corp/trust/partnership): $250,000</li>
                <li data-i18n="view.s6111.thresh.other_natural">Other reportable, natural person: $10,000</li>
                <li data-i18n="view.s6111.thresh.other_other">Other reportable, other: $25,000</li>
                <li data-i18n="view.s6111.thresh.expectation">"Reasonably expects" to receive — not actually received</li>
                <li data-i18n="view.s6111.thresh.cumulative">Cumulative across multiple participants in same transaction</li>
                <li data-i18n="view.s6111.thresh.related">Includes related entities + family members</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6111.h2.penalties">§ 6707 + § 6707A penalty structure</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s6111.tbl.violation">Violation</th><th data-i18n="view.s6111.tbl.advisor">Advisor (§ 6707)</th><th data-i18n="view.s6111.tbl.taxpayer">Taxpayer (§ 6707A)</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s6111.tbl.listed">Listed transaction failure</td><td data-i18n="view.s6111.tbl.listed_advisor">Greater of $200K OR 75% gross income</td><td data-i18n="view.s6111.tbl.listed_tp">75% of decrease in tax / $200K cap / $100K min</td></tr>
                    <tr><td data-i18n="view.s6111.tbl.other">Other reportable failure</td><td data-i18n="view.s6111.tbl.other_advisor">$50,000</td><td data-i18n="view.s6111.tbl.other_tp">75% of decrease in tax / $50K cap / $10K min</td></tr>
                    <tr><td data-i18n="view.s6111.tbl.intentional">Intentional disregard</td><td data-i18n="view.s6111.tbl.intentional_advisor">Greater of $200K OR 75%</td><td data-i18n="view.s6111.tbl.intentional_tp">Reasonable cause defense narrow</td></tr>
                    <tr><td data-i18n="view.s6111.tbl.list">§ 6112 list maintenance</td><td data-i18n="view.s6111.tbl.list_amt">$10,000/day after 20 business days</td><td data-i18n="view.s6111.tbl.list_tp">N/A</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6111.h2.list_maintenance">§ 6112 list maintenance</h2>
            <ul class="muted small">
                <li data-i18n="view.s6111.list.required">Material advisor must maintain list of advisees + transactions</li>
                <li data-i18n="view.s6111.list.contents">List: name, address, TIN, amount invested, copy of advice, all fees</li>
                <li data-i18n="view.s6111.list.retention">7-year retention period from date last became material advisor</li>
                <li data-i18n="view.s6111.list.production">Must produce within 20 business days of IRS written request</li>
                <li data-i18n="view.s6111.list.penalty_clock">$10,000/day starts on day 21 — continues until full production</li>
                <li data-i18n="view.s6111.list.no_cap">NO cap on total — can exceed $200K easily</li>
                <li data-i18n="view.s6111.list.privilege">Attorney-client privilege NOT general defense — § 7525 narrow practitioner privilege</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6111.h2.coordination">Coordination + interaction</h2>
            <ul class="muted small">
                <li data-i18n="view.s6111.coord.taxpayer">Taxpayer separately files Form 8886 (§ 6011) attached to return</li>
                <li data-i18n="view.s6111.coord.eited_taxpayer">Even if advisor fails to file 8918, taxpayer still required 8886</li>
                <li data-i18n="view.s6111.coord.sol_exception">§ 6501(c)(10) extended SOL: 1 year after Form 8886 filed</li>
                <li data-i18n="view.s6111.coord.list_extension">List request triggers attorney-supervisor review at IRS national office</li>
                <li data-i18n="view.s6111.coord.criminal">Criminal prosecution possible: § 7203 (failure to file information) + § 7206 (false)</li>
                <li data-i18n="view.s6111.coord.injunction">§ 7408 injunction available against repeat material advisor</li>
                <li data-i18n="view.s6111.coord.disclosure_taxpayer">Strategy disclosure may trigger client wave of audit + § 6662 accuracy</li>
            </ul>
        </div>
    `;
    document.getElementById('s6111-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.is_material_advisor = !!fd.get('is_material_advisor');
        state.threshold_fee_met = !!fd.get('threshold_fee_met');
        state.fee_amount = Number(fd.get('fee_amount')) || 0;
        state.is_reportable_transaction = !!fd.get('is_reportable_transaction');
        state.rt_type = fd.get('rt_type');
        state.quarter_became_advisor = fd.get('quarter_became_advisor');
        state.year = Number(fd.get('year')) || 0;
        state.form_8918_filed = !!fd.get('form_8918_filed');
        state.days_late = Number(fd.get('days_late')) || 0;
        state.is_failure_to_file = !!fd.get('is_failure_to_file');
        state.list_maintained = !!fd.get('list_maintained');
        state.gross_income_from_advice = Number(fd.get('gross_income_from_advice')) || 0;
        state.is_listed_transaction = !!fd.get('is_listed_transaction');
        state.intentional_disregard = !!fd.get('intentional_disregard');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6111-output');
    if (!el) return;
    let advisor_penalty = 0;
    if (state.is_failure_to_file) {
        if (state.is_listed_transaction || state.intentional_disregard) {
            advisor_penalty = Math.max(200_000, state.gross_income_from_advice * 0.75);
        } else {
            advisor_penalty = 50_000;
        }
    }
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6111.h2.result">§ 6707 advisor penalty assessment</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s6111.card.material">Material advisor?</div>
                    <div class="value">${state.is_material_advisor ? 'YES' : 'NO'}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6111.card.category">Category</div>
                    <div class="value">${esc(state.rt_type)}</div>
                </div>
                <div class="card ${state.is_failure_to_file ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s6111.card.filed">Form 8918 filed?</div>
                    <div class="value">${state.form_8918_filed ? 'YES' : 'NO'}</div>
                </div>
                <div class="card ${advisor_penalty > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s6111.card.penalty">§ 6707 penalty</div>
                    <div class="value">$${advisor_penalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
