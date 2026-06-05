// IRC § 1059 — Corporate Shareholder's Basis in Stock Reduced by Nontaxed Portion of Extraordinary Dividend.
// Anti-abuse: shareholder corp's basis reduced by NON-TAXED portion (DRD) of extraordinary dividend.
// "Extraordinary dividend" = dividend ≥ 5% of basis (common) / 10% (preferred), 85-day window.
// If basis reduced below $0: excess = capital gain in year of subsequent disposition.

import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    dividend_amount: 0,
    shareholder_basis: 0,
    is_preferred_stock: false,
    dividend_pct_of_basis: 0,
    s1059_a_extraordinary_threshold: 5,
    days_held: 0,
    days_held_at_record: 0,
    s1059_d_2_year_holding_required: false,
    nontaxed_portion: 0,
    s243_drd_pct: 50,
    s246a_holding_period_satisfied: true,
    s1059_a_basis_reduction: 0,
    excess_capital_gain_on_disposition: 0,
    s1059_c_aggregation_rule: false,
    aggregation_window_days: 85,
    aggregated_dividend_amount: 0,
    s1059_e_extraordinary_redemption: false,
    redemption_partial_liquidation: false,
    s302_b_redemption_qualifying: false,
    is_recapitalization: false,
    s1059_f_qualified_preferred: false,
    s1059_g_anti_avoidance: false,
    is_e_p_basis_test_election: false,
    e_p_at_dividend: 0,
    s1059_d_2_a_basis_alternative: false,
    s248_organizational_expense: 0,
    multiple_extraordinary_dividends: false,
    cumulative_extraordinary: 0,
    fairmark_safe_harbor: false,
    s1059_b_after_basis_reduction: 0,
};

export async function renderSection1059(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1059.h1.title">// § 1059 EXTRAORDINARY DIVIDEND BASIS REDUCTION</span></h1>
        <p class="muted small" data-i18n="view.s1059.hint.intro">
            <strong>§ 1059</strong> — corporate shareholder receiving "extraordinary dividend" must
            REDUCE basis in stock by NON-TAXED portion (typically § 243/§ 245/§ 245A DRD).
            <strong>"Extraordinary dividend":</strong> dividend ≥ 5% of adjusted basis (common stock)
            / 10% (preferred stock). <strong>§ 1059(c) aggregation:</strong> dividends within 85-day
            window aggregated for threshold test. <strong>§ 1059(d) holding period:</strong> 2-YEAR
            holding period BEFORE declaration (otherwise § 1059 ALWAYS applies regardless of
            extraordinary status). <strong>If basis reduced below $0:</strong> excess = capital gain
            in CURRENT year (not deferred to disposition under post-1997 amendments).
            <strong>§ 1059(e) extraordinary redemptions:</strong> partial liquidation + § 302(b)
            qualifying redemptions also subject. <strong>Purpose:</strong> deters "dividend stripping"
            — corp shareholder buys high-basis stock, receives big DRD-eligible dividend, then sells
            at loss.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1059.h2.inputs">Inputs</h2>
            <form id="s1059-form" class="inline-form">
                <label><span data-i18n="view.s1059.label.dividend">Dividend ($)</span>
                    <input type="number" step="0.01" name="dividend_amount" value="${state.dividend_amount}"></label>
                <label><span data-i18n="view.s1059.label.basis">Shareholder basis ($)</span>
                    <input type="number" step="0.01" name="shareholder_basis" value="${state.shareholder_basis}"></label>
                <label><span data-i18n="view.s1059.label.preferred">Preferred stock?</span>
                    <input type="checkbox" name="is_preferred_stock" ${state.is_preferred_stock ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1059.label.pct">Dividend % of basis</span>
                    <input type="number" step="0.1" name="dividend_pct_of_basis" value="${state.dividend_pct_of_basis}"></label>
                <label><span data-i18n="view.s1059.label.threshold">Extraordinary threshold %</span>
                    <input type="number" step="0.1" name="s1059_a_extraordinary_threshold" value="${state.s1059_a_extraordinary_threshold}"></label>
                <label><span data-i18n="view.s1059.label.days">Days held</span>
                    <input type="number" step="1" name="days_held" value="${state.days_held}"></label>
                <label><span data-i18n="view.s1059.label.days_record">Days held at record</span>
                    <input type="number" step="1" name="days_held_at_record" value="${state.days_held_at_record}"></label>
                <label><span data-i18n="view.s1059.label.s1059d">§ 1059(d) 2-yr required?</span>
                    <input type="checkbox" name="s1059_d_2_year_holding_required" ${state.s1059_d_2_year_holding_required ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1059.label.nontaxed">Non-taxed portion ($)</span>
                    <input type="number" step="0.01" name="nontaxed_portion" value="${state.nontaxed_portion}"></label>
                <label><span data-i18n="view.s1059.label.s243">§ 243 DRD %</span>
                    <input type="number" step="0.1" name="s243_drd_pct" value="${state.s243_drd_pct}"></label>
                <label><span data-i18n="view.s1059.label.s246a">§ 246A holding sat?</span>
                    <input type="checkbox" name="s246a_holding_period_satisfied" ${state.s246a_holding_period_satisfied ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1059.label.basis_red">§ 1059(a) basis reduction ($)</span>
                    <input type="number" step="0.01" name="s1059_a_basis_reduction" value="${state.s1059_a_basis_reduction}"></label>
                <label><span data-i18n="view.s1059.label.excess">Excess capital gain ($)</span>
                    <input type="number" step="0.01" name="excess_capital_gain_on_disposition" value="${state.excess_capital_gain_on_disposition}"></label>
                <label><span data-i18n="view.s1059.label.aggregation">§ 1059(c) aggregation?</span>
                    <input type="checkbox" name="s1059_c_aggregation_rule" ${state.s1059_c_aggregation_rule ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1059.label.window">Aggregation window (days)</span>
                    <input type="number" step="1" name="aggregation_window_days" value="${state.aggregation_window_days}"></label>
                <label><span data-i18n="view.s1059.label.agg_amt">Aggregated div ($)</span>
                    <input type="number" step="0.01" name="aggregated_dividend_amount" value="${state.aggregated_dividend_amount}"></label>
                <label><span data-i18n="view.s1059.label.s1059e">§ 1059(e) extraordinary redemption?</span>
                    <input type="checkbox" name="s1059_e_extraordinary_redemption" ${state.s1059_e_extraordinary_redemption ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1059.label.partial">Partial liquidation?</span>
                    <input type="checkbox" name="redemption_partial_liquidation" ${state.redemption_partial_liquidation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1059.label.s302b">§ 302(b) qualifying?</span>
                    <input type="checkbox" name="s302_b_redemption_qualifying" ${state.s302_b_redemption_qualifying ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1059.label.recap">Recapitalization?</span>
                    <input type="checkbox" name="is_recapitalization" ${state.is_recapitalization ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1059.label.s1059f">§ 1059(f) qualified preferred?</span>
                    <input type="checkbox" name="s1059_f_qualified_preferred" ${state.s1059_f_qualified_preferred ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1059.label.s1059g">§ 1059(g) anti-avoidance?</span>
                    <input type="checkbox" name="s1059_g_anti_avoidance" ${state.s1059_g_anti_avoidance ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1059.label.ep_election">§ 1059(d)(2)(A) E&P election?</span>
                    <input type="checkbox" name="is_e_p_basis_test_election" ${state.is_e_p_basis_test_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1059.label.ep">E&P at dividend ($)</span>
                    <input type="number" step="0.01" name="e_p_at_dividend" value="${state.e_p_at_dividend}"></label>
                <label><span data-i18n="view.s1059.label.s1059_d2a">§ 1059(d)(2)(A) alt basis?</span>
                    <input type="checkbox" name="s1059_d_2_a_basis_alternative" ${state.s1059_d_2_a_basis_alternative ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1059.label.s248">§ 248 org exp ($)</span>
                    <input type="number" step="0.01" name="s248_organizational_expense" value="${state.s248_organizational_expense}"></label>
                <label><span data-i18n="view.s1059.label.multi">Multiple extraordinary?</span>
                    <input type="checkbox" name="multiple_extraordinary_dividends" ${state.multiple_extraordinary_dividends ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1059.label.cumul">Cumulative ($)</span>
                    <input type="number" step="0.01" name="cumulative_extraordinary" value="${state.cumulative_extraordinary}"></label>
                <label><span data-i18n="view.s1059.label.fairmark">Fairmark safe harbor?</span>
                    <input type="checkbox" name="fairmark_safe_harbor" ${state.fairmark_safe_harbor ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1059.label.after">§ 1059(b) after-basis ($)</span>
                    <input type="number" step="0.01" name="s1059_b_after_basis_reduction" value="${state.s1059_b_after_basis_reduction}"></label>
                <button class="primary" type="submit" data-i18n="view.s1059.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1059-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1059.h2.extraordinary">"Extraordinary dividend" test</h2>
            <ol class="muted small">
                <li data-i18n="view.s1059.ex.common">Common stock: ≥ 5% of adjusted basis</li>
                <li data-i18n="view.s1059.ex.preferred">Preferred stock: ≥ 10% of adjusted basis</li>
                <li data-i18n="view.s1059.ex.s1059_c_aggregation">§ 1059(c) aggregation: all dividends within 85-day window</li>
                <li data-i18n="view.s1059.ex.s1059_c_2">85-day aggregation also for 365-day window if cumulative ≥ 20% basis</li>
                <li data-i18n="view.s1059.ex.no_actual_distribution">Constructive dividends + § 305(c) deemed distributions also subject</li>
                <li data-i18n="view.s1059.ex.s1059_a_4_election">§ 1059(a)(4) FMV election: substitute FMV basis instead of adjusted basis</li>
                <li data-i18n="view.s1059.ex.fmv_election">Useful when stock acquired at significant premium</li>
                <li data-i18n="view.s1059.ex.regs">Reg § 1.1059-1 - extensive examples + tracking rules</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1059.h2.holding_period">§ 1059(d) 2-year holding period</h2>
            <ul class="muted small">
                <li data-i18n="view.s1059.hp.exemption">Holding period &gt; 2 YEARS before declaration date: § 1059 typically NOT apply</li>
                <li data-i18n="view.s1059.hp.unless_5pct">UNLESS dividend independently meets 5%/10% threshold</li>
                <li data-i18n="view.s1059.hp.failed_2yr">&lt; 2-year holding: § 1059 ALWAYS applies regardless of size</li>
                <li data-i18n="view.s1059.hp.measurement">Measured from acquisition to declaration date</li>
                <li data-i18n="view.s1059.hp.s246_a_2">§ 246(a)(2) — actual holding period requirements for DRD eligibility (45 days)</li>
                <li data-i18n="view.s1059.hp.parallel_test">§ 1059 + § 246(c) — coordinated holding period scrutiny</li>
                <li data-i18n="view.s1059.hp.qualified_purchase">Purchased post-Aug 31, 1995: post-TIPRA amendments accelerated gain rather than disposition deferral</li>
                <li data-i18n="view.s1059.hp.pre_TIPRA">Pre-TIPRA: basis reduction triggered gain only at disposition</li>
                <li data-i18n="view.s1059.hp.post_TIPRA">Post-TIPRA: basis reduction below $0 → IMMEDIATE capital gain</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1059.h2.basis_reduction">Basis reduction mechanics</h2>
            <ul class="muted small">
                <li data-i18n="view.s1059.br.nontaxed">Reduce by NON-TAXED portion = dividend × DRD %</li>
                <li data-i18n="view.s1059.br.s243_50">§ 243 50% DRD: less than 20% owned (typical)</li>
                <li data-i18n="view.s1059.br.s243_65">§ 243 65% DRD: 20-79% owned</li>
                <li data-i18n="view.s1059.br.s243_100">§ 243 100% DRD: 80%+ owned (affiliated group)</li>
                <li data-i18n="view.s1059.br.s245a_100">§ 245A 100% DRD: foreign source (post-TCJA)</li>
                <li data-i18n="view.s1059.br.s245">§ 245 100% DRD: dividends from less-than-20% owned domestic that is foreign-source</li>
                <li data-i18n="view.s1059.br.s1059_b_below_zero">Basis reduced below $0 → excess = capital gain CURRENT year</li>
                <li data-i18n="view.s1059.br.s1059_c_calculation">For multiple stocks held, allocate basis reduction pro-rata</li>
                <li data-i18n="view.s1059.br.holding_period">Holding period of remaining stock NOT affected</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1059.h2.examples">Examples + applications</h2>
            <ul class="muted small">
                <li data-i18n="view.s1059.ex.dividend_stripping">"Dividend stripping" historically: buy at $100, $50 dividend with DRD, sell at $50 = $50 loss</li>
                <li data-i18n="view.s1059.ex.now_blocked">Post-§ 1059: $50 dividend × 50% DRD = $25 basis reduction → no artificial loss</li>
                <li data-i18n="view.s1059.ex.s246a_no_drd">§ 246A — partially debt-financed stock: reduced DRD percentage</li>
                <li data-i18n="view.s1059.ex.s246c_45day">§ 246(c) 45-day holding (90 days for preferred) — to qualify for DRD</li>
                <li data-i18n="view.s1059.ex.s246d_short">§ 246(d) — short sale + similar offsetting position eliminates DRD</li>
                <li data-i18n="view.s1059.ex.qualified_preferred">§ 1059(f) qualified preferred dividend: lower threshold</li>
                <li data-i18n="view.s1059.ex.s1059_e_partial_liquidation">§ 1059(e) partial liquidation: treated as extraordinary regardless of size</li>
                <li data-i18n="view.s1059.ex.s302_redemptions">§ 302(b)(1)/(2)/(3) redemptions: § 1059(e)(1) — § 1059 applies</li>
                <li data-i18n="view.s1059.ex.s301_excess">§ 301(c)(2) basis recovery + § 301(c)(3) capital gain — separate timing</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1059.h2.related">Related provisions</h2>
            <ul class="muted small">
                <li data-i18n="view.s1059.rel.s243">§ 243 — Dividends-Received Deduction (DRD)</li>
                <li data-i18n="view.s1059.rel.s245">§ 245 — Foreign-source DRD (limited)</li>
                <li data-i18n="view.s1059.rel.s245a">§ 245A — Foreign 100% DRD (post-TCJA)</li>
                <li data-i18n="view.s1059.rel.s246">§ 246 — DRD limitations (holding period)</li>
                <li data-i18n="view.s1059.rel.s246a">§ 246A — Debt-financed stock</li>
                <li data-i18n="view.s1059.rel.s301">§ 301(c)(2) — Distribution treated as basis recovery</li>
                <li data-i18n="view.s1059.rel.s302">§ 302 — Distributions in redemption</li>
                <li data-i18n="view.s1059.rel.s305">§ 305 — Stock dividends + § 305(c) deemed</li>
                <li data-i18n="view.s1059.rel.s311">§ 311 — Distributions of property by corp</li>
                <li data-i18n="view.s1059.rel.s246_c">§ 246(c) — short-sale anti-abuse for DRD</li>
                <li data-i18n="view.s1059.rel.f8810">Form 8810 — corporate AMT (formerly)</li>
                <li data-i18n="view.s1059.rel.s162a">§ 162(k) — golden parachute → ordinary deduction denied (similar anti-abuse)</li>
            </ul>
        </div>
    `;
    document.getElementById('s1059-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.dividend_amount = Number(fd.get('dividend_amount')) || 0;
        state.shareholder_basis = Number(fd.get('shareholder_basis')) || 0;
        state.is_preferred_stock = !!fd.get('is_preferred_stock');
        state.dividend_pct_of_basis = Number(fd.get('dividend_pct_of_basis')) || 0;
        state.s1059_a_extraordinary_threshold = Number(fd.get('s1059_a_extraordinary_threshold')) || 0;
        state.days_held = Number(fd.get('days_held')) || 0;
        state.days_held_at_record = Number(fd.get('days_held_at_record')) || 0;
        state.s1059_d_2_year_holding_required = !!fd.get('s1059_d_2_year_holding_required');
        state.nontaxed_portion = Number(fd.get('nontaxed_portion')) || 0;
        state.s243_drd_pct = Number(fd.get('s243_drd_pct')) || 0;
        state.s246a_holding_period_satisfied = !!fd.get('s246a_holding_period_satisfied');
        state.s1059_a_basis_reduction = Number(fd.get('s1059_a_basis_reduction')) || 0;
        state.excess_capital_gain_on_disposition = Number(fd.get('excess_capital_gain_on_disposition')) || 0;
        state.s1059_c_aggregation_rule = !!fd.get('s1059_c_aggregation_rule');
        state.aggregation_window_days = Number(fd.get('aggregation_window_days')) || 0;
        state.aggregated_dividend_amount = Number(fd.get('aggregated_dividend_amount')) || 0;
        state.s1059_e_extraordinary_redemption = !!fd.get('s1059_e_extraordinary_redemption');
        state.redemption_partial_liquidation = !!fd.get('redemption_partial_liquidation');
        state.s302_b_redemption_qualifying = !!fd.get('s302_b_redemption_qualifying');
        state.is_recapitalization = !!fd.get('is_recapitalization');
        state.s1059_f_qualified_preferred = !!fd.get('s1059_f_qualified_preferred');
        state.s1059_g_anti_avoidance = !!fd.get('s1059_g_anti_avoidance');
        state.is_e_p_basis_test_election = !!fd.get('is_e_p_basis_test_election');
        state.e_p_at_dividend = Number(fd.get('e_p_at_dividend')) || 0;
        state.s1059_d_2_a_basis_alternative = !!fd.get('s1059_d_2_a_basis_alternative');
        state.s248_organizational_expense = Number(fd.get('s248_organizational_expense')) || 0;
        state.multiple_extraordinary_dividends = !!fd.get('multiple_extraordinary_dividends');
        state.cumulative_extraordinary = Number(fd.get('cumulative_extraordinary')) || 0;
        state.fairmark_safe_harbor = !!fd.get('fairmark_safe_harbor');
        state.s1059_b_after_basis_reduction = Number(fd.get('s1059_b_after_basis_reduction')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1059-output');
    if (!el) return;
    const threshold = state.is_preferred_stock ? 10 : 5;
    const pct = state.shareholder_basis > 0 ? (state.dividend_amount / state.shareholder_basis) * 100 : 0;
    const is_extraordinary = pct >= threshold;
    const nontaxed = state.dividend_amount * (state.s243_drd_pct / 100);
    const basis_after = Math.max(0, state.shareholder_basis - nontaxed);
    const excess_gain = state.shareholder_basis - nontaxed < 0 ? Math.abs(state.shareholder_basis - nontaxed) : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1059.h2.result">§ 1059 analysis</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s1059.card.pct">Div % of basis</div><div class="value">${pct.toFixed(1)}%</div></div>
                <div class="card ${is_extraordinary ? 'warn' : 'pos'}"><div class="label" data-i18n="view.s1059.card.extra">Extraordinary?</div><div class="value">${is_extraordinary ? 'YES' : 'NO'}</div></div>
                <div class="card"><div class="label" data-i18n="view.s1059.card.nontaxed">Non-taxed (DRD)</div><div class="value">$${nontaxed.toLocaleString()}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.s1059.card.basis_after">Basis after reduction</div><div class="value">$${basis_after.toLocaleString()}</div></div>
                <div class="card ${excess_gain > 0 ? 'neg' : 'pos'}"><div class="label" data-i18n="view.s1059.card.excess">Excess capital gain</div><div class="value">$${excess_gain.toLocaleString()}</div></div>
            </div>
        </div>
    `;
}
