// IRC § 6011 — General Requirement of Return + Reportable Transaction Disclosure (Form 8886).
// § 6011(a) + Reg § 1.6011-4 require taxpayer to file Form 8886 if participated in reportable transaction.
// Pairs with § 6111 (material advisor 8918) + § 6707A taxpayer penalty + § 6707 advisor penalty.
// 5 categories: listed, confidential, contractual protection, loss, transactions of interest.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    is_taxpayer_participated: false,
    rt_category: 'listed',
    participated_year: 2024,
    is_form_8886_filed: false,
    days_late: 0,
    s6707a_penalty_assessed: 0,
    s6011_b_protected_disclosure: false,
    is_tax_shelter: false,
    is_listed_transaction: false,
    is_substantially_similar: false,
    loss_transaction_amount: 0,
    confidential_obligation: false,
    contractual_protection_amount: 0,
    s6011_e_no_filing_required: false,
    is_disclosure_advance: false,
    transaction_year_first_engaged: 2024,
    tax_benefit_amount: 0,
    s6011_reportable_amount: 0,
    s6501_c_10_sol_extension: false,
    s7203_failure_to_file: false,
    s7201_evasion: false,
    s6112_list_request_received: false,
    rev_rul_2002_57_tested: false,
    notice_2009_7_listed_transaction: false,
    reasonable_cause_defense: false,
    voluntary_disclosure: false,
};

export async function renderSection6011(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6011.h1.title">// § 6011 REPORTABLE TRANSACTION DISCLOSURE (Form 8886)</span></h1>
        <p class="muted small" data-i18n="view.s6011.hint.intro">
            <strong>§ 6011 + Reg § 1.6011-4</strong> requires taxpayer to file <strong>Form 8886</strong>
            attached to return for each reportable transaction. <strong>5 categories:</strong>
            (1) LISTED transactions (Notice 2009-7 + others), (2) CONFIDENTIAL transactions (fee +
            confidentiality restriction), (3) CONTRACTUAL PROTECTION transactions (refund if no tax
            benefit), (4) LOSS transactions (&gt; $10M C-corp / $2M individual / $5M partnership),
            (5) TRANSACTIONS OF INTEREST (TOI — IRS designated). <strong>Pairs with:</strong>
            § 6111 material advisor 8918 + § 6707A taxpayer penalty (75% of decrease in tax / $200K
            cap / $100K min for listed) + § 6707 advisor penalty + § 6501(c)(10) SOL extension to
            1 year after filing. <strong>OPR + Circular 230</strong> disciplinary for practitioners.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6011.h2.inputs">Inputs</h2>
            <form id="s6011-form" class="inline-form">
                <label><span data-i18n="view.s6011.label.participated">Participated?</span>
                    <input type="checkbox" name="is_taxpayer_participated" ${state.is_taxpayer_participated ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6011.label.category">RT category</span>
                    <select name="rt_category">
                        <option value="listed" ${state.rt_category === 'listed' ? 'selected' : ''}>Listed (Notice 2009-7 et al)</option>
                        <option value="confidential" ${state.rt_category === 'confidential' ? 'selected' : ''}>Confidential</option>
                        <option value="contractual" ${state.rt_category === 'contractual' ? 'selected' : ''}>Contractual protection</option>
                        <option value="loss" ${state.rt_category === 'loss' ? 'selected' : ''}>Loss transactions</option>
                        <option value="toi" ${state.rt_category === 'toi' ? 'selected' : ''}>Transactions of interest (TOI)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6011.label.year">Participated year</span>
                    <input type="number" step="1" name="participated_year" value="${state.participated_year}"></label>
                <label><span data-i18n="view.s6011.label.filed">Form 8886 filed?</span>
                    <input type="checkbox" name="is_form_8886_filed" ${state.is_form_8886_filed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6011.label.late">Days late</span>
                    <input type="number" step="1" name="days_late" value="${state.days_late}"></label>
                <label><span data-i18n="view.s6011.label.s6707a">§ 6707A penalty ($)</span>
                    <input type="number" step="1000" name="s6707a_penalty_assessed" value="${state.s6707a_penalty_assessed}"></label>
                <label><span data-i18n="view.s6011.label.protected">§ 6011(b) protected?</span>
                    <input type="checkbox" name="s6011_b_protected_disclosure" ${state.s6011_b_protected_disclosure ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6011.label.shelter">Tax shelter?</span>
                    <input type="checkbox" name="is_tax_shelter" ${state.is_tax_shelter ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6011.label.listed">Listed transaction?</span>
                    <input type="checkbox" name="is_listed_transaction" ${state.is_listed_transaction ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6011.label.similar">Substantially similar?</span>
                    <input type="checkbox" name="is_substantially_similar" ${state.is_substantially_similar ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6011.label.loss">Loss transaction ($)</span>
                    <input type="number" step="100000" name="loss_transaction_amount" value="${state.loss_transaction_amount}"></label>
                <label><span data-i18n="view.s6011.label.confidential">Confidential obligation?</span>
                    <input type="checkbox" name="confidential_obligation" ${state.confidential_obligation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6011.label.contractual">Contractual protection ($)</span>
                    <input type="number" step="1000" name="contractual_protection_amount" value="${state.contractual_protection_amount}"></label>
                <label><span data-i18n="view.s6011.label.no_filing">§ 6011(e) no filing?</span>
                    <input type="checkbox" name="s6011_e_no_filing_required" ${state.s6011_e_no_filing_required ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6011.label.advance">Advance disclosure?</span>
                    <input type="checkbox" name="is_disclosure_advance" ${state.is_disclosure_advance ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6011.label.engaged_year">First engaged year</span>
                    <input type="number" step="1" name="transaction_year_first_engaged" value="${state.transaction_year_first_engaged}"></label>
                <label><span data-i18n="view.s6011.label.benefit">Tax benefit ($)</span>
                    <input type="number" step="10000" name="tax_benefit_amount" value="${state.tax_benefit_amount}"></label>
                <label><span data-i18n="view.s6011.label.reportable">Reportable amount ($)</span>
                    <input type="number" step="10000" name="s6011_reportable_amount" value="${state.s6011_reportable_amount}"></label>
                <label><span data-i18n="view.s6011.label.sol">§ 6501(c)(10) SOL ext?</span>
                    <input type="checkbox" name="s6501_c_10_sol_extension" ${state.s6501_c_10_sol_extension ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6011.label.s7203">§ 7203 failure to file?</span>
                    <input type="checkbox" name="s7203_failure_to_file" ${state.s7203_failure_to_file ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6011.label.s7201">§ 7201 evasion?</span>
                    <input type="checkbox" name="s7201_evasion" ${state.s7201_evasion ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6011.label.s6112">§ 6112 list request received?</span>
                    <input type="checkbox" name="s6112_list_request_received" ${state.s6112_list_request_received ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6011.label.rr_2002">Rev. Rul. 2002-57?</span>
                    <input type="checkbox" name="rev_rul_2002_57_tested" ${state.rev_rul_2002_57_tested ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6011.label.notice_2009">Notice 2009-7 listed?</span>
                    <input type="checkbox" name="notice_2009_7_listed_transaction" ${state.notice_2009_7_listed_transaction ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6011.label.cause">Reasonable cause?</span>
                    <input type="checkbox" name="reasonable_cause_defense" ${state.reasonable_cause_defense ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6011.label.voluntary">Voluntary disclosure?</span>
                    <input type="checkbox" name="voluntary_disclosure" ${state.voluntary_disclosure ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s6011.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6011-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6011.h2.categories">5 categories (Reg § 1.6011-4)</h2>
            <ol class="muted small">
                <li data-i18n="view.s6011.cat.listed">Listed transactions — IRS-designated abusive (Notice 2009-7 etc., basket option contracts, Roth 401(k) etc.)</li>
                <li data-i18n="view.s6011.cat.confidential">Confidential — fee paid + restriction on disclosure to others (Reg § 1.6011-4(b)(3))</li>
                <li data-i18n="view.s6011.cat.contractual">Contractual protection — refund/reduction if no tax benefit (Reg § 1.6011-4(b)(4))</li>
                <li data-i18n="view.s6011.cat.loss">Loss transactions — $10M C-corp / $2M individual / $5M partnership / $50M for 5+ year (Reg § 1.6011-4(b)(5))</li>
                <li data-i18n="view.s6011.cat.toi">Transactions of Interest — IRS designates as worthy of scrutiny (notice / rev. proc.)</li>
                <li data-i18n="view.s6011.cat.substantially_similar">"Substantially similar" expands net (Reg § 1.6011-4(c)(4))</li>
                <li data-i18n="view.s6011.cat.s6707a_listed">Listed: 75% of decrease in tax / $200K cap / $100K min individual</li>
                <li data-i18n="view.s6011.cat.s6707a_other">Other reportable: 75% / $50K cap / $10K min individual</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6011.h2.form_8886">Form 8886 mechanics</h2>
            <ul class="muted small">
                <li data-i18n="view.s6011.f8886.attached">Attached to return for year of participation</li>
                <li data-i18n="view.s6011.f8886.first_year">Plus separate copy sent to OTSA (Office of Tax Shelter Analysis) for FIRST year</li>
                <li data-i18n="view.s6011.f8886.each_year">Filed EACH YEAR taxpayer continues to participate</li>
                <li data-i18n="view.s6011.f8886.tax_benefit">Reports: category, transaction description, parties, projected tax benefit, fees paid</li>
                <li data-i18n="view.s6011.f8886.amended">Late filing: amended return + Form 8886</li>
                <li data-i18n="view.s6011.f8886.s6707a">§ 6707A penalty: separate from accuracy-related, separate from interest</li>
                <li data-i18n="view.s6011.f8886.s6011_g">§ 6011(g) failure to file = automatic § 6707A penalty (no reasonable cause for listed)</li>
                <li data-i18n="view.s6011.f8886.s6664_d">§ 6664(d) — reasonable cause defense LIMITED for listed transactions</li>
                <li data-i18n="view.s6011.f8886.advance_disclosure">Advance disclosure: file ahead of return — avoid some penalties</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6011.h2.penalties">§ 6707A taxpayer penalty</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s6011.tbl.transaction">Transaction</th><th data-i18n="view.s6011.tbl.percentage">% of tax decrease</th><th data-i18n="view.s6011.tbl.individual">Individual min/max</th><th data-i18n="view.s6011.tbl.entity">Entity min/max</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s6011.tbl.listed_p">Listed transaction</td><td>75%</td><td>$100K / $200K</td><td>$200K / $200K</td></tr>
                    <tr><td data-i18n="view.s6011.tbl.other_p">Other reportable</td><td>75%</td><td>$10K / $50K</td><td>$50K / $50K</td></tr>
                    <tr><td data-i18n="view.s6011.tbl.tax_benefit_basis">Floor protection</td><td>—</td><td>Listed = $100K</td><td>Listed = $200K</td></tr>
                    <tr><td data-i18n="view.s6011.tbl.s6707A_h">§ 6707A(h) repealed</td><td>—</td><td>Old rescission via Commissioner</td><td>2010 Small Business Jobs Act repealed</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6011.h2.sol">§ 6501(c)(10) SOL extension</h2>
            <ul class="muted small">
                <li data-i18n="view.s6011.sol.purpose">Extends 3-year ASED to ONE year after Form 8886 filed (if late)</li>
                <li data-i18n="view.s6011.sol.unlimited">If never filed: ASED essentially UNLIMITED for that item</li>
                <li data-i18n="view.s6011.sol.disclosure_protects">Timely Form 8886 disclosure: SOL runs normally from filing of underlying return</li>
                <li data-i18n="view.s6011.sol.s6501_a">§ 6501(a) general 3-yr SOL starts from later of due date or actual filing</li>
                <li data-i18n="view.s6011.sol.s6501_e">§ 6501(e) 6-yr SOL on 25%+ income omissions still applies</li>
                <li data-i18n="view.s6011.sol.criminal_6yr">§ 6531 criminal SOL 6 years still applies</li>
                <li data-i18n="view.s6011.sol.s6664_b">§ 6664(b) penalty SOL 3 years after disclosed transaction return filed</li>
                <li data-i18n="view.s6011.sol.s6404_f">§ 6404(f) interest abatement window 5 years</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6011.h2.related">Related provisions + interactions</h2>
            <ul class="muted small">
                <li data-i18n="view.s6011.rel.s6111">§ 6111 — material advisor disclosure (Form 8918) — separate filing</li>
                <li data-i18n="view.s6011.rel.s6112">§ 6112 — material advisor list maintenance ($10K/day after 20 day request)</li>
                <li data-i18n="view.s6011.rel.s6707">§ 6707 — material advisor penalty for failure to disclose</li>
                <li data-i18n="view.s6011.rel.s6708">§ 6708 — failure to maintain list ($10K/day)</li>
                <li data-i18n="view.s6011.rel.s6700">§ 6700 — promoter penalty (organizing abusive shelter)</li>
                <li data-i18n="view.s6011.rel.s6701">§ 6701 — aiding understatement of liability ($1K/$10K per occurrence)</li>
                <li data-i18n="view.s6011.rel.s6662">§ 6662 — accuracy-related penalty (20%/40%) — coordinates</li>
                <li data-i18n="view.s6011.rel.s7408">§ 7408 — injunction against promoter/advisor</li>
                <li data-i18n="view.s6011.rel.circular_230">Circular 230 § 10.51 — practitioner discipline for false/fraudulent advice</li>
                <li data-i18n="view.s6011.rel.s7525">§ 7525 — federally authorized tax practitioner privilege LIMITED for tax shelters</li>
            </ul>
        </div>
    `;
    document.getElementById('s6011-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.is_taxpayer_participated = !!fd.get('is_taxpayer_participated');
        state.rt_category = fd.get('rt_category');
        state.participated_year = Number(fd.get('participated_year')) || 0;
        state.is_form_8886_filed = !!fd.get('is_form_8886_filed');
        state.days_late = Number(fd.get('days_late')) || 0;
        state.s6707a_penalty_assessed = Number(fd.get('s6707a_penalty_assessed')) || 0;
        state.s6011_b_protected_disclosure = !!fd.get('s6011_b_protected_disclosure');
        state.is_tax_shelter = !!fd.get('is_tax_shelter');
        state.is_listed_transaction = !!fd.get('is_listed_transaction');
        state.is_substantially_similar = !!fd.get('is_substantially_similar');
        state.loss_transaction_amount = Number(fd.get('loss_transaction_amount')) || 0;
        state.confidential_obligation = !!fd.get('confidential_obligation');
        state.contractual_protection_amount = Number(fd.get('contractual_protection_amount')) || 0;
        state.s6011_e_no_filing_required = !!fd.get('s6011_e_no_filing_required');
        state.is_disclosure_advance = !!fd.get('is_disclosure_advance');
        state.transaction_year_first_engaged = Number(fd.get('transaction_year_first_engaged')) || 0;
        state.tax_benefit_amount = Number(fd.get('tax_benefit_amount')) || 0;
        state.s6011_reportable_amount = Number(fd.get('s6011_reportable_amount')) || 0;
        state.s6501_c_10_sol_extension = !!fd.get('s6501_c_10_sol_extension');
        state.s7203_failure_to_file = !!fd.get('s7203_failure_to_file');
        state.s7201_evasion = !!fd.get('s7201_evasion');
        state.s6112_list_request_received = !!fd.get('s6112_list_request_received');
        state.rev_rul_2002_57_tested = !!fd.get('rev_rul_2002_57_tested');
        state.notice_2009_7_listed_transaction = !!fd.get('notice_2009_7_listed_transaction');
        state.reasonable_cause_defense = !!fd.get('reasonable_cause_defense');
        state.voluntary_disclosure = !!fd.get('voluntary_disclosure');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6011-output');
    if (!el) return;
    let penalty = 0;
    if (state.is_taxpayer_participated && !state.is_form_8886_filed) {
        if (state.is_listed_transaction) {
            penalty = Math.max(100_000, Math.min(state.tax_benefit_amount * 0.75, 200_000));
        } else {
            penalty = Math.max(10_000, Math.min(state.tax_benefit_amount * 0.75, 50_000));
        }
    }
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6011.h2.result">§ 6011 + § 6707A assessment</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s6011.card.cat">Category</div><div class="value">${esc(state.rt_category)}</div></div>
                <div class="card ${state.is_form_8886_filed ? 'pos' : 'neg'}"><div class="label" data-i18n="view.s6011.card.filed">Form 8886 filed?</div><div class="value">${state.is_form_8886_filed ? 'YES' : 'NO'}</div></div>
                <div class="card ${penalty > 0 ? 'neg' : 'pos'}"><div class="label" data-i18n="view.s6011.card.penalty">§ 6707A penalty</div><div class="value">$${penalty.toLocaleString()}</div></div>
                <div class="card warn"><div class="label" data-i18n="view.s6011.card.sol">§ 6501(c)(10) SOL</div><div class="value">${state.is_form_8886_filed ? 'normal 3yr' : 'OPEN UNTIL FILED'}</div></div>
            </div>
        </div>
    `;
}
