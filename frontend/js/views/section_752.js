// IRC § 752 — Partnership Liabilities + Outside Basis.
// § 752(a) increase in partner's share of liabilities = deemed contribution.
// § 752(b) decrease in partner's share of liabilities = deemed distribution.
// Distinction between recourse (Reg § 1.752-2) + nonrecourse (Reg § 1.752-3) crucial.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    partner_outside_basis_start: 0,
    partner_capital_account: 0,
    profit_share_pct: 0,
    loss_share_pct: 0,
    total_recourse_liabilities: 0,
    partner_share_recourse_eoy: 0,
    total_nonrecourse_liabilities: 0,
    partner_share_nonrecourse_eoy: 0,
    qualified_nonrecourse_financing: 0,
    is_real_estate_qnr: false,
    starts_with_minimum_gain: 0,
    ends_with_minimum_gain: 0,
    s704_b_book_capital: 0,
    s752_a_increase: 0,
    s752_b_decrease: 0,
    is_disregarded_entity: false,
    partner_guarantee: 0,
    bottom_dollar_guarantee: false,
    is_atrisk_partner: true,
    s465_at_risk_basis: 0,
    s7041d_3_share: 0,
    is_recourse_default: true,
    nonrecourse_first_tier: 0,
    nonrecourse_second_tier: 0,
    nonrecourse_third_tier: 0,
};

export async function renderSection752(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s752.h1.title">// § 752 PARTNERSHIP LIABILITIES + BASIS</span></h1>
        <p class="muted small" data-i18n="view.s752.hint.intro">
            <strong>§ 752(a)</strong> — increase in partner's share of liabilities = DEEMED CASH
            CONTRIBUTION. <strong>§ 752(b)</strong> — decrease = DEEMED CASH DISTRIBUTION (potential
            gain if exceeds basis). <strong>RECOURSE liabilities (Reg § 1.752-2):</strong> share via
            CONSTRUCTIVE LIQUIDATION analysis — who bears economic risk of loss. <strong>NONRECOURSE
            (Reg § 1.752-3):</strong> 3-tier allocation: (1) partnership minimum gain, (2) § 704(c)
            minimum gain, (3) excess by profit-sharing % or other reasonable method. <strong>§ 465
            at-risk</strong> separate analysis — nonrecourse generally NOT at-risk except qualified
            nonrecourse real estate financing. <strong>§ 1.752-7 bottom-dollar guarantee anti-abuse</strong>
            since 2016 — disregarded for basis purposes.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s752.h2.inputs">Inputs</h2>
            <form id="s752-form" class="inline-form">
                <label><span data-i18n="view.s752.label.start_basis">Outside basis start ($)</span>
                    <input type="number" step="10000" name="partner_outside_basis_start" value="${state.partner_outside_basis_start}"></label>
                <label><span data-i18n="view.s752.label.cap_account">§ 704(b) capital account ($)</span>
                    <input type="number" step="10000" name="partner_capital_account" value="${state.partner_capital_account}"></label>
                <label><span data-i18n="view.s752.label.profit_pct">Profit share %</span>
                    <input type="number" step="0.1" name="profit_share_pct" value="${state.profit_share_pct}"></label>
                <label><span data-i18n="view.s752.label.loss_pct">Loss share %</span>
                    <input type="number" step="0.1" name="loss_share_pct" value="${state.loss_share_pct}"></label>
                <label><span data-i18n="view.s752.label.total_recourse">Total recourse liabilities ($)</span>
                    <input type="number" step="10000" name="total_recourse_liabilities" value="${state.total_recourse_liabilities}"></label>
                <label><span data-i18n="view.s752.label.share_recourse">Partner share recourse EOY ($)</span>
                    <input type="number" step="10000" name="partner_share_recourse_eoy" value="${state.partner_share_recourse_eoy}"></label>
                <label><span data-i18n="view.s752.label.total_nr">Total nonrecourse ($)</span>
                    <input type="number" step="10000" name="total_nonrecourse_liabilities" value="${state.total_nonrecourse_liabilities}"></label>
                <label><span data-i18n="view.s752.label.share_nr">Partner share NR EOY ($)</span>
                    <input type="number" step="10000" name="partner_share_nonrecourse_eoy" value="${state.partner_share_nonrecourse_eoy}"></label>
                <label><span data-i18n="view.s752.label.qnr">Qualified NR financing ($)</span>
                    <input type="number" step="10000" name="qualified_nonrecourse_financing" value="${state.qualified_nonrecourse_financing}"></label>
                <label><span data-i18n="view.s752.label.is_qnr">Real estate QNR?</span>
                    <input type="checkbox" name="is_real_estate_qnr" ${state.is_real_estate_qnr ? 'checked' : ''}></label>
                <label><span data-i18n="view.s752.label.mg_start">Min gain start ($)</span>
                    <input type="number" step="10000" name="starts_with_minimum_gain" value="${state.starts_with_minimum_gain}"></label>
                <label><span data-i18n="view.s752.label.mg_end">Min gain end ($)</span>
                    <input type="number" step="10000" name="ends_with_minimum_gain" value="${state.ends_with_minimum_gain}"></label>
                <label><span data-i18n="view.s752.label.s704b">§ 704(b) book capital ($)</span>
                    <input type="number" step="10000" name="s704_b_book_capital" value="${state.s704_b_book_capital}"></label>
                <label><span data-i18n="view.s752.label.s752a">§ 752(a) increase ($)</span>
                    <input type="number" step="10000" name="s752_a_increase" value="${state.s752_a_increase}"></label>
                <label><span data-i18n="view.s752.label.s752b">§ 752(b) decrease ($)</span>
                    <input type="number" step="10000" name="s752_b_decrease" value="${state.s752_b_decrease}"></label>
                <label><span data-i18n="view.s752.label.disregarded">Disregarded entity?</span>
                    <input type="checkbox" name="is_disregarded_entity" ${state.is_disregarded_entity ? 'checked' : ''}></label>
                <label><span data-i18n="view.s752.label.guarantee">Partner guarantee ($)</span>
                    <input type="number" step="10000" name="partner_guarantee" value="${state.partner_guarantee}"></label>
                <label><span data-i18n="view.s752.label.bottom_dollar">Bottom-dollar guarantee?</span>
                    <input type="checkbox" name="bottom_dollar_guarantee" ${state.bottom_dollar_guarantee ? 'checked' : ''}></label>
                <label><span data-i18n="view.s752.label.at_risk">At-risk partner?</span>
                    <input type="checkbox" name="is_atrisk_partner" ${state.is_atrisk_partner ? 'checked' : ''}></label>
                <label><span data-i18n="view.s752.label.s465">§ 465 at-risk basis ($)</span>
                    <input type="number" step="10000" name="s465_at_risk_basis" value="${state.s465_at_risk_basis}"></label>
                <label><span data-i18n="view.s752.label.s704d3">§ 704(d)(3) share ($)</span>
                    <input type="number" step="10000" name="s7041d_3_share" value="${state.s7041d_3_share}"></label>
                <label><span data-i18n="view.s752.label.recourse_default">Recourse default?</span>
                    <input type="checkbox" name="is_recourse_default" ${state.is_recourse_default ? 'checked' : ''}></label>
                <label><span data-i18n="view.s752.label.nr_t1">NR tier 1 (min gain) ($)</span>
                    <input type="number" step="10000" name="nonrecourse_first_tier" value="${state.nonrecourse_first_tier}"></label>
                <label><span data-i18n="view.s752.label.nr_t2">NR tier 2 (§ 704(c)) ($)</span>
                    <input type="number" step="10000" name="nonrecourse_second_tier" value="${state.nonrecourse_second_tier}"></label>
                <label><span data-i18n="view.s752.label.nr_t3">NR tier 3 (excess) ($)</span>
                    <input type="number" step="10000" name="nonrecourse_third_tier" value="${state.nonrecourse_third_tier}"></label>
                <button class="primary" type="submit" data-i18n="view.s752.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s752-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s752.h2.recourse">Recourse share — constructive liquidation (Reg § 1.752-2)</h2>
            <ol class="muted small">
                <li data-i18n="view.s752.rec.step1">Assume all partnership liabilities become due + payable at FMV</li>
                <li data-i18n="view.s752.rec.step2">Partnership sells all assets for $0 → losses charged to capital accounts</li>
                <li data-i18n="view.s752.rec.step3">Partner's deficit in capital account = obligation to restore (if any)</li>
                <li data-i18n="view.s752.rec.step4">Partner bears EROL (economic risk of loss) = recourse share</li>
                <li data-i18n="view.s752.rec.deficit_restoration">DRO (deficit restoration obligation) = mandatory if § 704(b) safe harbor used</li>
                <li data-i18n="view.s752.rec.guarantee">Partner guarantee = EROL only if NO reimbursement right</li>
                <li data-i18n="view.s752.rec.indemnity">Indemnity vs guarantee — indemnity transfers EROL between partners</li>
                <li data-i18n="view.s752.rec.bottom_dollar_2016">§ 1.752-2 (2016 anti-abuse): bottom-dollar guarantee disregarded</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s752.h2.nonrecourse">Nonrecourse share — 3-tier (Reg § 1.752-3)</h2>
            <ol class="muted small">
                <li data-i18n="view.s752.nr.t1">Tier 1: partnership minimum gain (PMG) — partner's share per § 704(b)</li>
                <li data-i18n="view.s752.nr.t2">Tier 2: § 704(c) minimum gain — partner's share of built-in gain on contributed property</li>
                <li data-i18n="view.s752.nr.t3">Tier 3: remainder allocated by profit-sharing % OR other reasonable method</li>
                <li data-i18n="view.s752.nr.t3_alt">Tier 3 alternative methods: SIBR (significant item method), liquidation method, profit method</li>
                <li data-i18n="view.s752.nr.pmg_def">PMG = excess of NR debt over book basis of secured property</li>
                <li data-i18n="view.s752.nr.s704_c_min_gain">§ 704(c) min gain = built-in gain reduced by depreciation</li>
                <li data-i18n="view.s752.nr.consistent">Method must be CONSISTENT year-to-year unless reasonable cause</li>
                <li data-i18n="view.s752.nr.exculpatory">Exculpatory liability: lender recourse to partnership but NOT individual partner</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s752.h2.basis">Outside basis mechanics</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s752.tbl.event">Event</th><th data-i18n="view.s752.tbl.impact">Basis impact</th><th data-i18n="view.s752.tbl.citation">Citation</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s752.tbl.contribution">Capital contribution</td><td>+ basis</td><td>§ 722</td></tr>
                    <tr><td data-i18n="view.s752.tbl.liab_increase">§ 752(a) liability share increase</td><td>+ deemed contribution</td><td>§ 752(a)</td></tr>
                    <tr><td data-i18n="view.s752.tbl.income_share">Allocable income</td><td>+ basis</td><td>§ 705(a)(1)</td></tr>
                    <tr><td data-i18n="view.s752.tbl.distribution">Cash distribution</td><td>- basis (up to basis, then gain)</td><td>§ 705(a)(2) + § 731</td></tr>
                    <tr><td data-i18n="view.s752.tbl.liab_decrease">§ 752(b) liability share decrease</td><td>- deemed distribution</td><td>§ 752(b)</td></tr>
                    <tr><td data-i18n="view.s752.tbl.loss_share">Allocable loss</td><td>- basis (limited to basis)</td><td>§ 704(d)</td></tr>
                    <tr><td data-i18n="view.s752.tbl.distribution_excess">Excess distribution</td><td>= gain recognized</td><td>§ 731(a)(1)</td></tr>
                    <tr><td data-i18n="view.s752.tbl.basis_floor">Basis floor</td><td>$0 (cannot go negative)</td><td>§ 705(a)</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s752.h2.at_risk">§ 465 at-risk interaction</h2>
            <ul class="muted small">
                <li data-i18n="view.s752.atrisk.separate">§ 465 at-risk separate from outside basis</li>
                <li data-i18n="view.s752.atrisk.nr_excluded">Nonrecourse generally NOT at-risk — § 465(b)(3)</li>
                <li data-i18n="view.s752.atrisk.qnr">EXCEPTION: § 465(b)(6) qualified nonrecourse financing in real estate</li>
                <li data-i18n="view.s752.atrisk.qnr_definition">QNR = secured by real property + held by activity of holding real property + lender unrelated</li>
                <li data-i18n="view.s752.atrisk.recourse_general">Recourse: generally at-risk to extent of personal liability</li>
                <li data-i18n="view.s752.atrisk.guarantee_no_reimb">Guarantee w/o reimbursement = at-risk</li>
                <li data-i18n="view.s752.atrisk.suspended">Disallowed losses suspended + carry forward indefinitely</li>
                <li data-i18n="view.s752.atrisk.recapture">§ 465(e) recapture if at-risk falls below $0 (e.g., refinance NR)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s752.h2.special">Special situations</h2>
            <ul class="muted small">
                <li data-i18n="view.s752.spec.contribution_basis">Contributed property with liab > basis: § 357(c) does NOT apply (partnership uses § 752)</li>
                <li data-i18n="view.s752.spec.disguised_sale">§ 707(a)(2)(B) disguised sale: contribution + distribution within 2 years</li>
                <li data-i18n="view.s752.spec.s704c">§ 704(c) built-in gain forced allocation on contribution</li>
                <li data-i18n="view.s752.spec.gracias_share">"Gracias" share method — pro-rata by profit share for tier 3</li>
                <li data-i18n="view.s752.spec.partnership_distribution">Partnership-level liability decrease may trigger § 752(b) to all partners</li>
                <li data-i18n="view.s752.spec.related_party">§ 752(b)(2) related-party transfer: 90% test for liability attribution</li>
                <li data-i18n="view.s752.spec.tiered_partnerships">Tiered partnerships: look through upper-tier to lower partner</li>
                <li data-i18n="view.s752.spec.dre">Disregarded entity (DRE): owner deemed to bear EROL</li>
            </ul>
        </div>
    `;
    document.getElementById('s752-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.partner_outside_basis_start = Number(fd.get('partner_outside_basis_start')) || 0;
        state.partner_capital_account = Number(fd.get('partner_capital_account')) || 0;
        state.profit_share_pct = Number(fd.get('profit_share_pct')) || 0;
        state.loss_share_pct = Number(fd.get('loss_share_pct')) || 0;
        state.total_recourse_liabilities = Number(fd.get('total_recourse_liabilities')) || 0;
        state.partner_share_recourse_eoy = Number(fd.get('partner_share_recourse_eoy')) || 0;
        state.total_nonrecourse_liabilities = Number(fd.get('total_nonrecourse_liabilities')) || 0;
        state.partner_share_nonrecourse_eoy = Number(fd.get('partner_share_nonrecourse_eoy')) || 0;
        state.qualified_nonrecourse_financing = Number(fd.get('qualified_nonrecourse_financing')) || 0;
        state.is_real_estate_qnr = !!fd.get('is_real_estate_qnr');
        state.starts_with_minimum_gain = Number(fd.get('starts_with_minimum_gain')) || 0;
        state.ends_with_minimum_gain = Number(fd.get('ends_with_minimum_gain')) || 0;
        state.s704_b_book_capital = Number(fd.get('s704_b_book_capital')) || 0;
        state.s752_a_increase = Number(fd.get('s752_a_increase')) || 0;
        state.s752_b_decrease = Number(fd.get('s752_b_decrease')) || 0;
        state.is_disregarded_entity = !!fd.get('is_disregarded_entity');
        state.partner_guarantee = Number(fd.get('partner_guarantee')) || 0;
        state.bottom_dollar_guarantee = !!fd.get('bottom_dollar_guarantee');
        state.is_atrisk_partner = !!fd.get('is_atrisk_partner');
        state.s465_at_risk_basis = Number(fd.get('s465_at_risk_basis')) || 0;
        state.s7041d_3_share = Number(fd.get('s7041d_3_share')) || 0;
        state.is_recourse_default = !!fd.get('is_recourse_default');
        state.nonrecourse_first_tier = Number(fd.get('nonrecourse_first_tier')) || 0;
        state.nonrecourse_second_tier = Number(fd.get('nonrecourse_second_tier')) || 0;
        state.nonrecourse_third_tier = Number(fd.get('nonrecourse_third_tier')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s752-output');
    if (!el) return;
    const net_change = state.s752_a_increase - state.s752_b_decrease;
    const ending_basis = state.partner_outside_basis_start + net_change;
    const gain_on_excess = ending_basis < 0 ? Math.abs(ending_basis) : 0;
    const nr_total = state.nonrecourse_first_tier + state.nonrecourse_second_tier + state.nonrecourse_third_tier;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s752.h2.result">§ 752 basis impact</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s752.card.start">Start basis</div><div class="value">$${state.partner_outside_basis_start.toLocaleString()}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.s752.card.s752a">§ 752(a) increase</div><div class="value">+$${state.s752_a_increase.toLocaleString()}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.s752.card.s752b">§ 752(b) decrease</div><div class="value">−$${state.s752_b_decrease.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s752.card.ending">Ending basis</div><div class="value">$${Math.max(0, ending_basis).toLocaleString()}</div></div>
                <div class="card ${gain_on_excess > 0 ? 'neg' : 'pos'}"><div class="label" data-i18n="view.s752.card.gain">§ 731(a)(1) excess gain</div><div class="value">$${gain_on_excess.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s752.card.nr_total">NR 3-tier total</div><div class="value">$${nr_total.toLocaleString()}</div></div>
            </div>
        </div>
    `;
}
