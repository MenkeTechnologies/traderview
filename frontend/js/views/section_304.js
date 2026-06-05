// IRC § 304 — Redemption Through Related Corporations.
// Anti-abuse: sale of stock to RELATED corporation treated as REDEMPTION + DIVIDEND.
// Brother-sister (§ 304(a)(1)): controlling shareholder sells stock of one corp to another.
// Parent-sub (§ 304(a)(2)): shareholder sells parent stock to subsidiary.

import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    transaction_type: 'brother_sister',
    issuing_corp: '',
    acquiring_corp: '',
    seller_ownership_issuing_pct: 0,
    seller_ownership_acquiring_pct: 0,
    s304_control_threshold: 50,
    s304_b_brother_sister_test: false,
    s304_a_2_parent_sub_test: false,
    is_s304_a_1_brother_sister: false,
    is_s304_a_2_parent_sub: false,
    sale_proceeds: 0,
    seller_stock_basis: 0,
    issuing_corp_e_p: 0,
    acquiring_corp_e_p: 0,
    s301_distribution_treatment: false,
    s301_c_1_dividend: 0,
    s301_c_2_basis_recovery: 0,
    s301_c_3_capital_gain: 0,
    s318_attribution_applied: false,
    s304_b_1_constructive_dividend: 0,
    is_s302_qualifying_redemption: false,
    s302_b_test_satisfied: false,
    s304_b_3_full_termination: false,
    s356_a_2_e_p_attribution: false,
    s304_c_control_test_50pct: false,
    foreign_acquiring_corp: false,
    s304_b_5_foreign_blocker: false,
    s956_inclusion_avoided: false,
    s245a_drd_eligible: false,
    s959_pti_distribution: false,
    boot_received: 0,
    is_corporate_seller: false,
    s243_drd_pct: 50,
    multiple_corp_chain: false,
};

export async function renderSection304(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s304.h1.title">// § 304 RELATED CORPORATION REDEMPTION</span></h1>
        <p class="muted small" data-i18n="view.s304.hint.intro">
            <strong>§ 304</strong> ANTI-ABUSE — sale of stock to RELATED corp treated as REDEMPTION of
            acquiring corp's stock + § 301 DISTRIBUTION (potential dividend).
            <strong>§ 304(a)(1) Brother-Sister:</strong> shareholder owning ≥ 50% (by vote or value) of
            BOTH issuing corp + acquiring corp transfers issuing-corp stock to acquiring.
            <strong>§ 304(a)(2) Parent-Sub:</strong> shareholder transfers PARENT corp stock to its
            SUBSIDIARY. <strong>§ 318 attribution</strong> applied broadly for control test.
            <strong>§ 304(b)(1):</strong> distribution under § 301; first dividend (up to E&P of
            ACQUIRING corp, then ISSUING corp), then basis recovery, then capital gain.
            <strong>§ 302(b) escape:</strong> if redemption-like terms met after attribution, sale
            treatment + capital gain instead. <strong>§ 956 avoidance:</strong> historic use case via
            foreign acquiring blockers — extensively regulated post-TCJA.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s304.h2.inputs">Inputs</h2>
            <form id="s304-form" class="inline-form">
                <label><span data-i18n="view.s304.label.type">Transaction type</span>
                    <select name="transaction_type">
                        <option value="brother_sister" ${state.transaction_type === 'brother_sister' ? 'selected' : ''}>Brother-sister (§ 304(a)(1))</option>
                        <option value="parent_sub" ${state.transaction_type === 'parent_sub' ? 'selected' : ''}>Parent-subsidiary (§ 304(a)(2))</option>
                        <option value="chain" ${state.transaction_type === 'chain' ? 'selected' : ''}>Multi-tier chain</option>
                    </select>
                </label>
                <label><span data-i18n="view.s304.label.issuing">Issuing corp</span>
                    <input type="text" name="issuing_corp" value="${esc(state.issuing_corp)}"></label>
                <label><span data-i18n="view.s304.label.acquiring">Acquiring corp</span>
                    <input type="text" name="acquiring_corp" value="${esc(state.acquiring_corp)}"></label>
                <label><span data-i18n="view.s304.label.seller_iss">Seller ownership issuing %</span>
                    <input type="number" step="0.1" name="seller_ownership_issuing_pct" value="${state.seller_ownership_issuing_pct}"></label>
                <label><span data-i18n="view.s304.label.seller_acq">Seller ownership acquiring %</span>
                    <input type="number" step="0.1" name="seller_ownership_acquiring_pct" value="${state.seller_ownership_acquiring_pct}"></label>
                <label><span data-i18n="view.s304.label.threshold">Control threshold %</span>
                    <input type="number" step="0.1" name="s304_control_threshold" value="${state.s304_control_threshold}"></label>
                <label><span data-i18n="view.s304.label.b_test">§ 304(b) BS test?</span>
                    <input type="checkbox" name="s304_b_brother_sister_test" ${state.s304_b_brother_sister_test ? 'checked' : ''}></label>
                <label><span data-i18n="view.s304.label.a2_test">§ 304(a)(2) PS test?</span>
                    <input type="checkbox" name="s304_a_2_parent_sub_test" ${state.s304_a_2_parent_sub_test ? 'checked' : ''}></label>
                <label><span data-i18n="view.s304.label.is_a1">§ 304(a)(1) brother-sister?</span>
                    <input type="checkbox" name="is_s304_a_1_brother_sister" ${state.is_s304_a_1_brother_sister ? 'checked' : ''}></label>
                <label><span data-i18n="view.s304.label.is_a2">§ 304(a)(2) parent-sub?</span>
                    <input type="checkbox" name="is_s304_a_2_parent_sub" ${state.is_s304_a_2_parent_sub ? 'checked' : ''}></label>
                <label><span data-i18n="view.s304.label.proceeds">Sale proceeds ($)</span>
                    <input type="number" step="0.01" name="sale_proceeds" value="${state.sale_proceeds}"></label>
                <label><span data-i18n="view.s304.label.basis">Seller stock basis ($)</span>
                    <input type="number" step="0.01" name="seller_stock_basis" value="${state.seller_stock_basis}"></label>
                <label><span data-i18n="view.s304.label.issuing_ep">Issuing E&P ($)</span>
                    <input type="number" step="0.01" name="issuing_corp_e_p" value="${state.issuing_corp_e_p}"></label>
                <label><span data-i18n="view.s304.label.acquiring_ep">Acquiring E&P ($)</span>
                    <input type="number" step="0.01" name="acquiring_corp_e_p" value="${state.acquiring_corp_e_p}"></label>
                <label><span data-i18n="view.s304.label.s301">§ 301 distribution?</span>
                    <input type="checkbox" name="s301_distribution_treatment" ${state.s301_distribution_treatment ? 'checked' : ''}></label>
                <label><span data-i18n="view.s304.label.s301_c1">§ 301(c)(1) dividend ($)</span>
                    <input type="number" step="0.01" name="s301_c_1_dividend" value="${state.s301_c_1_dividend}"></label>
                <label><span data-i18n="view.s304.label.s301_c2">§ 301(c)(2) basis ($)</span>
                    <input type="number" step="0.01" name="s301_c_2_basis_recovery" value="${state.s301_c_2_basis_recovery}"></label>
                <label><span data-i18n="view.s304.label.s301_c3">§ 301(c)(3) cap gain ($)</span>
                    <input type="number" step="0.01" name="s301_c_3_capital_gain" value="${state.s301_c_3_capital_gain}"></label>
                <label><span data-i18n="view.s304.label.s318">§ 318 attribution?</span>
                    <input type="checkbox" name="s318_attribution_applied" ${state.s318_attribution_applied ? 'checked' : ''}></label>
                <label><span data-i18n="view.s304.label.constructive">§ 304(b)(1) constructive div ($)</span>
                    <input type="number" step="0.01" name="s304_b_1_constructive_dividend" value="${state.s304_b_1_constructive_dividend}"></label>
                <label><span data-i18n="view.s304.label.s302q">§ 302 qualifying?</span>
                    <input type="checkbox" name="is_s302_qualifying_redemption" ${state.is_s302_qualifying_redemption ? 'checked' : ''}></label>
                <label><span data-i18n="view.s304.label.s302b">§ 302(b) satisfied?</span>
                    <input type="checkbox" name="s302_b_test_satisfied" ${state.s302_b_test_satisfied ? 'checked' : ''}></label>
                <label><span data-i18n="view.s304.label.s304b3">§ 304(b)(3) full term?</span>
                    <input type="checkbox" name="s304_b_3_full_termination" ${state.s304_b_3_full_termination ? 'checked' : ''}></label>
                <label><span data-i18n="view.s304.label.s356_ep">§ 356(a)(2) E&P attribution?</span>
                    <input type="checkbox" name="s356_a_2_e_p_attribution" ${state.s356_a_2_e_p_attribution ? 'checked' : ''}></label>
                <label><span data-i18n="view.s304.label.s304c">§ 304(c) control 50%?</span>
                    <input type="checkbox" name="s304_c_control_test_50pct" ${state.s304_c_control_test_50pct ? 'checked' : ''}></label>
                <label><span data-i18n="view.s304.label.foreign">Foreign acquiring?</span>
                    <input type="checkbox" name="foreign_acquiring_corp" ${state.foreign_acquiring_corp ? 'checked' : ''}></label>
                <label><span data-i18n="view.s304.label.s304b5">§ 304(b)(5) foreign blocker?</span>
                    <input type="checkbox" name="s304_b_5_foreign_blocker" ${state.s304_b_5_foreign_blocker ? 'checked' : ''}></label>
                <label><span data-i18n="view.s304.label.s956">§ 956 avoided?</span>
                    <input type="checkbox" name="s956_inclusion_avoided" ${state.s956_inclusion_avoided ? 'checked' : ''}></label>
                <label><span data-i18n="view.s304.label.s245a">§ 245A DRD eligible?</span>
                    <input type="checkbox" name="s245a_drd_eligible" ${state.s245a_drd_eligible ? 'checked' : ''}></label>
                <label><span data-i18n="view.s304.label.s959">§ 959 PTI?</span>
                    <input type="checkbox" name="s959_pti_distribution" ${state.s959_pti_distribution ? 'checked' : ''}></label>
                <label><span data-i18n="view.s304.label.boot">Boot received ($)</span>
                    <input type="number" step="0.01" name="boot_received" value="${state.boot_received}"></label>
                <label><span data-i18n="view.s304.label.corp_seller">Corporate seller?</span>
                    <input type="checkbox" name="is_corporate_seller" ${state.is_corporate_seller ? 'checked' : ''}></label>
                <label><span data-i18n="view.s304.label.drd_pct">§ 243 DRD %</span>
                    <input type="number" step="0.1" name="s243_drd_pct" value="${state.s243_drd_pct}"></label>
                <label><span data-i18n="view.s304.label.multi">Multi-tier chain?</span>
                    <input type="checkbox" name="multiple_corp_chain" ${state.multiple_corp_chain ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s304.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s304-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s304.h2.brother_sister">§ 304(a)(1) Brother-Sister mechanics</h2>
            <ol class="muted small">
                <li data-i18n="view.s304.bs.transferor">Seller owns ≥ 50% of BOTH issuing + acquiring corps</li>
                <li data-i18n="view.s304.bs.transfer">Seller transfers issuing-corp stock to acquiring corp for cash + property</li>
                <li data-i18n="view.s304.bs.deemed">DEEMED REDEMPTION by issuing corp + § 351 contribution to acquiring</li>
                <li data-i18n="view.s304.bs.s301">§ 301 distribution treatment applied: 1st acquiring E&P, then issuing E&P</li>
                <li data-i18n="view.s304.bs.s318">§ 318 attribution for control test (BROAD)</li>
                <li data-i18n="view.s304.bs.basis_to_acquiring">Acquiring's basis in issuing stock = seller's basis in issuing stock</li>
                <li data-i18n="view.s304.bs.purpose">Purpose: prevent disguising dividend as sale (between commonly-controlled corps)</li>
                <li data-i18n="view.s304.bs.s304_b_2">§ 304(b)(2): combined E&P rule — pool both corps' E&P</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s304.h2.parent_sub">§ 304(a)(2) Parent-Subsidiary mechanics</h2>
            <ol class="muted small">
                <li data-i18n="view.s304.ps.parent_stock">Shareholder owns parent corp stock</li>
                <li data-i18n="view.s304.ps.transfer">Shareholder transfers PARENT stock to SUBSIDIARY for cash</li>
                <li data-i18n="view.s304.ps.deemed">DEEMED REDEMPTION by parent of its own stock + § 351 contribution to subsidiary</li>
                <li data-i18n="view.s304.ps.purpose">Eliminates "earnings stripping" via downstream stock sales</li>
                <li data-i18n="view.s304.ps.s301">§ 301 distribution from parent (after § 304 deemed redemption)</li>
                <li data-i18n="view.s304.ps.allocation">E&P: parent's E&P utilized first (only parent's)</li>
                <li data-i18n="view.s304.ps.s304_a_1_a">§ 304(a)(2)(A) — must transfer parent stock to subsidiary</li>
                <li data-i18n="view.s304.ps.parent_controls">Parent must control subsidiary (50%+ vote OR value)</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s304.h2.s301_ordering">§ 301 ordering after § 304</h2>
            <ol class="muted small">
                <li data-i18n="view.s304.ord.1">First: § 301(c)(1) DIVIDEND up to ACQUIRING corp E&P</li>
                <li data-i18n="view.s304.ord.2">Second: § 301(c)(1) DIVIDEND up to ISSUING corp E&P (Brother-sister only)</li>
                <li data-i18n="view.s304.ord.3">Third: § 301(c)(2) BASIS RECOVERY (reduces basis in acquiring corp stock)</li>
                <li data-i18n="view.s304.ord.4">Fourth: § 301(c)(3) CAPITAL GAIN (after basis exhausted)</li>
                <li data-i18n="view.s304.ord.ep_combined">Brother-sister: combined E&P pool (both corps)</li>
                <li data-i18n="view.s304.ord.ep_parent">Parent-sub: parent's E&P only</li>
                <li data-i18n="view.s304.ord.s956_loss">If parent/issuing has no E&P: basis recovery + capital gain (escape dividend)</li>
                <li data-i18n="view.s304.ord.s243_drd">If buyer is corp: § 243 DRD potentially available + § 246A debt-financed limitations</li>
                <li data-i18n="view.s304.ord.s1059">§ 1059 may apply to corp seller (extraordinary dividend)</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s304.h2.escape">Escape under § 302</h2>
            <ul class="muted small">
                <li data-i18n="view.s304.esc.s304_b_3">§ 304(b)(3): if § 302(b)(1)-(4) test met → SALE/EXCHANGE treatment (capital gain only)</li>
                <li data-i18n="view.s304.esc.substantial">Substantial disproportionate redemption: &lt; 50% AND &lt; 80% × prior ownership</li>
                <li data-i18n="view.s304.esc.complete">Complete termination of seller's interest</li>
                <li data-i18n="view.s304.esc.partial">Partial liquidation under § 302(e)</li>
                <li data-i18n="view.s304.esc.essentially">"Essentially equivalent to dividend" failure (facts &amp; circumstances)</li>
                <li data-i18n="view.s304.esc.s318_first">§ 318 attribution applied FIRST to determine "ownership reduction"</li>
                <li data-i18n="view.s304.esc.s302_c_waiver">§ 302(c)(2)(A) family attribution waiver (10-yr restriction)</li>
                <li data-i18n="view.s304.esc.attribution_blocks">Practical: § 318 attribution often blocks § 302 escape (related parties)</li>
                <li data-i18n="view.s304.esc.related_party_redemption">Related-party redemptions: typically caught by § 304 + § 318</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s304.h2.s304_b_5">§ 304(b)(5) foreign blocker rules</h2>
            <ul class="muted small">
                <li data-i18n="view.s304.fb.purpose">Anti-abuse: foreign acquiring corp used to repatriate CFC earnings without § 956 inclusion</li>
                <li data-i18n="view.s304.fb.s304_b_5">§ 304(b)(5) — special rules when acquiring corp is FOREIGN</li>
                <li data-i18n="view.s304.fb.E_P_ordering">E&P ordering: foreign issuing E&P first, then domestic-source attributable to acquiring</li>
                <li data-i18n="view.s304.fb.s956">§ 956: 50%+ owned CFC stock transferred → still potential § 956 inclusion</li>
                <li data-i18n="view.s304.fb.s245a">§ 245A 100% DRD potentially available if recharacterized dividend qualifies</li>
                <li data-i18n="view.s304.fb.s959">§ 959 PTI rules — distributions out of previously taxed income preserved</li>
                <li data-i18n="view.s304.fb.regs">Reg § 1.304-2 + § 1.367(b)-4 extensive coordination</li>
                <li data-i18n="view.s304.fb.s367_b">§ 367(b) outbound transfer recharacterization</li>
                <li data-i18n="view.s304.fb.notice_88_38">Notice 88-38 + subsequent guidance on foreign blockers</li>
                <li data-i18n="view.s304.fb.s7701_l">§ 7701(l) anti-conduit rules may apply if treaty country blocker used</li>
            </ul>
        </div>
    `;
    document.getElementById('s304-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.transaction_type = fd.get('transaction_type');
        state.issuing_corp = fd.get('issuing_corp') || '';
        state.acquiring_corp = fd.get('acquiring_corp') || '';
        state.seller_ownership_issuing_pct = Number(fd.get('seller_ownership_issuing_pct')) || 0;
        state.seller_ownership_acquiring_pct = Number(fd.get('seller_ownership_acquiring_pct')) || 0;
        state.s304_control_threshold = Number(fd.get('s304_control_threshold')) || 0;
        state.s304_b_brother_sister_test = !!fd.get('s304_b_brother_sister_test');
        state.s304_a_2_parent_sub_test = !!fd.get('s304_a_2_parent_sub_test');
        state.is_s304_a_1_brother_sister = !!fd.get('is_s304_a_1_brother_sister');
        state.is_s304_a_2_parent_sub = !!fd.get('is_s304_a_2_parent_sub');
        state.sale_proceeds = Number(fd.get('sale_proceeds')) || 0;
        state.seller_stock_basis = Number(fd.get('seller_stock_basis')) || 0;
        state.issuing_corp_e_p = Number(fd.get('issuing_corp_e_p')) || 0;
        state.acquiring_corp_e_p = Number(fd.get('acquiring_corp_e_p')) || 0;
        state.s301_distribution_treatment = !!fd.get('s301_distribution_treatment');
        state.s301_c_1_dividend = Number(fd.get('s301_c_1_dividend')) || 0;
        state.s301_c_2_basis_recovery = Number(fd.get('s301_c_2_basis_recovery')) || 0;
        state.s301_c_3_capital_gain = Number(fd.get('s301_c_3_capital_gain')) || 0;
        state.s318_attribution_applied = !!fd.get('s318_attribution_applied');
        state.s304_b_1_constructive_dividend = Number(fd.get('s304_b_1_constructive_dividend')) || 0;
        state.is_s302_qualifying_redemption = !!fd.get('is_s302_qualifying_redemption');
        state.s302_b_test_satisfied = !!fd.get('s302_b_test_satisfied');
        state.s304_b_3_full_termination = !!fd.get('s304_b_3_full_termination');
        state.s356_a_2_e_p_attribution = !!fd.get('s356_a_2_e_p_attribution');
        state.s304_c_control_test_50pct = !!fd.get('s304_c_control_test_50pct');
        state.foreign_acquiring_corp = !!fd.get('foreign_acquiring_corp');
        state.s304_b_5_foreign_blocker = !!fd.get('s304_b_5_foreign_blocker');
        state.s956_inclusion_avoided = !!fd.get('s956_inclusion_avoided');
        state.s245a_drd_eligible = !!fd.get('s245a_drd_eligible');
        state.s959_pti_distribution = !!fd.get('s959_pti_distribution');
        state.boot_received = Number(fd.get('boot_received')) || 0;
        state.is_corporate_seller = !!fd.get('is_corporate_seller');
        state.s243_drd_pct = Number(fd.get('s243_drd_pct')) || 0;
        state.multiple_corp_chain = !!fd.get('multiple_corp_chain');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s304-output');
    if (!el) return;
    const total_ep = state.issuing_corp_e_p + state.acquiring_corp_e_p;
    const dividend = Math.min(state.sale_proceeds, total_ep);
    const basis_recovery = Math.max(0, Math.min(state.sale_proceeds - dividend, state.seller_stock_basis));
    const cap_gain = state.sale_proceeds - dividend - basis_recovery;
    const s304_applies = state.seller_ownership_issuing_pct >= 50 && state.seller_ownership_acquiring_pct >= 50;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s304.h2.result">§ 304 result</h2>
            <div class="cards">
                <div class="card ${s304_applies ? 'warn' : 'pos'}"><div class="label" data-i18n="view.s304.card.applies">§ 304 applies?</div><div class="value">${s304_applies ? 'YES' : 'NO'}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.s304.card.dividend">§ 301(c)(1) dividend</div><div class="value">$${dividend.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s304.card.basis">§ 301(c)(2) basis recovery</div><div class="value">$${basis_recovery.toLocaleString()}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.s304.card.cap_gain">§ 301(c)(3) cap gain</div><div class="value">$${cap_gain.toLocaleString()}</div></div>
            </div>
        </div>
    `;
}
