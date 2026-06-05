// IRC § 302 — Stock Redemption (Sale vs Dividend).
// Sale treatment: § 302(b)(1) not essentially equivalent, (b)(2) substantially disproportionate (80% / <50%),
// (b)(3) complete termination, (b)(4) partial liquidation. Otherwise § 301 dividend.
// Constructive ownership rules of § 318 apply (family + entity attribution + options).
// Waiver of family attribution available with 10-year look-back agreement.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    redemption_amount: 0,
    shareholder_basis: 0,
    pre_redemption_pct: 0,
    post_redemption_pct: 0,
    is_complete_termination: false,
    family_attribution_waiver: false,
    not_equivalent_dividend: false,
    partial_liquidation: false,
    e_and_p_available: 0,
    is_long_term: false,
    constructive_ownership_pct: 0,
};

export async function renderSection302(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s302.h1.title">// § 302 STOCK REDEMPTION</span></h1>
        <p class="muted small" data-i18n="view.s302.hint.intro">
            Redemption = corporation buys back its own stock from shareholder. <strong>Sale treatment</strong>
            (gain / loss) OR <strong>§ 301 dividend</strong> (ordinary income to extent of E&P). <strong>Sale
            tests § 302(b):</strong> (1) not essentially equivalent, (2) substantially disproportionate
            (post &lt; 80% of pre + post &lt; 50% total), (3) complete termination, (4) partial liquidation.
            <strong>§ 318 attribution</strong> — family + entity + options. Waiver of family attribution
            with 10-year look-back agreement.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s302.h2.inputs">Inputs</h2>
            <form id="s302-form" class="inline-form">
                <label><span data-i18n="view.s302.label.amount">Redemption amount ($)</span>
                    <input type="number" step="0.01" name="redemption_amount" value="${state.redemption_amount}"></label>
                <label><span data-i18n="view.s302.label.basis">Shareholder basis in redeemed stock ($)</span>
                    <input type="number" step="0.01" name="shareholder_basis" value="${state.shareholder_basis}"></label>
                <label><span data-i18n="view.s302.label.pre">Pre-redemption ownership %</span>
                    <input type="number" step="0.01" name="pre_redemption_pct" value="${state.pre_redemption_pct}"></label>
                <label><span data-i18n="view.s302.label.post">Post-redemption ownership %</span>
                    <input type="number" step="0.01" name="post_redemption_pct" value="${state.post_redemption_pct}"></label>
                <label><span data-i18n="view.s302.label.complete">Complete termination?</span>
                    <input type="checkbox" name="is_complete_termination" ${state.is_complete_termination ? 'checked' : ''}></label>
                <label><span data-i18n="view.s302.label.waiver">Family attribution waiver (10-yr)?</span>
                    <input type="checkbox" name="family_attribution_waiver" ${state.family_attribution_waiver ? 'checked' : ''}></label>
                <label><span data-i18n="view.s302.label.not_equivalent">Not essentially equivalent (b)(1)?</span>
                    <input type="checkbox" name="not_equivalent_dividend" ${state.not_equivalent_dividend ? 'checked' : ''}></label>
                <label><span data-i18n="view.s302.label.partial">Partial liquidation (b)(4)?</span>
                    <input type="checkbox" name="partial_liquidation" ${state.partial_liquidation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s302.label.ep">E&P available ($)</span>
                    <input type="number" step="0.01" name="e_and_p_available" value="${state.e_and_p_available}"></label>
                <label><span data-i18n="view.s302.label.lt">Long-term holding?</span>
                    <input type="checkbox" name="is_long_term" ${state.is_long_term ? 'checked' : ''}></label>
                <label><span data-i18n="view.s302.label.constructive">Constructive ownership % (post-§ 318)</span>
                    <input type="number" step="0.01" name="constructive_ownership_pct" value="${state.constructive_ownership_pct}"></label>
                <button class="primary" type="submit" data-i18n="view.s302.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s302-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s302.h2.tests">§ 302(b) safe harbors</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s302.th.test">Test</th>
                    <th data-i18n="view.s302.th.standard">Standard</th>
                    <th data-i18n="view.s302.th.notes">Notes</th>
                </tr></thead>
                <tbody>
                    <tr><td>(b)(1) Not essentially equivalent</td><td>"Meaningful reduction"</td><td>Fact-specific; Davis case requires substantial reduction</td></tr>
                    <tr><td>(b)(2) Substantially disproportionate</td><td>Post &lt; 80% pre AND post &lt; 50% total voting</td><td>Mechanical safe harbor</td></tr>
                    <tr><td>(b)(3) Complete termination</td><td>Zero post-redemption ownership</td><td>Waiver of family attribution available</td></tr>
                    <tr><td>(b)(4) Partial liquidation</td><td>Corporate-level test — non-pro-rata + active business</td><td>Non-corp shareholders only post-1986</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s302.h2.attribution">§ 318 Constructive ownership</h2>
            <ul class="muted small">
                <li data-i18n="view.s302.attr.family">Family: spouse + children + grandchildren + parents (NOT siblings)</li>
                <li data-i18n="view.s302.attr.partnership">Partnership: 100% from / to partner pro-rata</li>
                <li data-i18n="view.s302.attr.corporation">Corporation: 50%+ shareholder treated as owning corp's stock pro-rata</li>
                <li data-i18n="view.s302.attr.trust">Trust + estate: full attribution to beneficiaries by interest</li>
                <li data-i18n="view.s302.attr.options">Options: option to acquire treated as owned</li>
                <li data-i18n="view.s302.attr.waiver">Family attribution may be waived with 10-yr look-back agreement (§ 302(c))</li>
                <li data-i18n="view.s302.attr.cant_waive_entity">Cannot waive entity attribution (corp, partnership, trust)</li>
                <li data-i18n="view.s302.attr.recurring">Attribution applied iteratively until stable</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s302.h2.dividend_warning">Dividend treatment consequences</h2>
            <ul class="muted small">
                <li data-i18n="view.s302.div.ordinary">Full amount taxed as DIVIDEND to extent of E&P (no basis recovery)</li>
                <li data-i18n="view.s302.div.qualified">May be qualified dividend if individual + 60-day holding + domestic</li>
                <li data-i18n="view.s302.div.basis_lost">Basis NOT recovered against dividend — added to remaining shares' basis</li>
                <li data-i18n="view.s302.div.corporation_drd">Corporate shareholder: § 243 DRD (50%/65%/100%) may apply</li>
                <li data-i18n="view.s302.div.foreign">Foreign shareholder: 30% FDAP withholding (or treaty rate)</li>
                <li data-i18n="view.s302.div.no_capital">No long-term capital gains rate, no loss recognition</li>
            </ul>
        </div>
    `;
    document.getElementById('s302-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.redemption_amount = Number(fd.get('redemption_amount')) || 0;
        state.shareholder_basis = Number(fd.get('shareholder_basis')) || 0;
        state.pre_redemption_pct = Number(fd.get('pre_redemption_pct')) || 0;
        state.post_redemption_pct = Number(fd.get('post_redemption_pct')) || 0;
        state.is_complete_termination = !!fd.get('is_complete_termination');
        state.family_attribution_waiver = !!fd.get('family_attribution_waiver');
        state.not_equivalent_dividend = !!fd.get('not_equivalent_dividend');
        state.partial_liquidation = !!fd.get('partial_liquidation');
        state.e_and_p_available = Number(fd.get('e_and_p_available')) || 0;
        state.is_long_term = !!fd.get('is_long_term');
        state.constructive_ownership_pct = Number(fd.get('constructive_ownership_pct')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s302-output');
    if (!el) return;
    const substantiallyDisproportionate = state.post_redemption_pct < 0.80 * state.pre_redemption_pct && state.post_redemption_pct < 50;
    const completeTerm = state.is_complete_termination && (state.constructive_ownership_pct === 0 || state.family_attribution_waiver);
    const saleTreatment = substantiallyDisproportionate || completeTerm || state.not_equivalent_dividend || state.partial_liquidation;
    const gain = Math.max(0, state.redemption_amount - state.shareholder_basis);
    const saleTax = saleTreatment ? gain * (state.is_long_term ? 0.20 : 0.37) : 0;
    const dividendAmt = !saleTreatment ? Math.min(state.redemption_amount, state.e_and_p_available) : 0;
    const dividendTax = dividendAmt * 0.20;
    const finalTax = saleTreatment ? saleTax : dividendTax;
    let activeTest = 'none';
    if (completeTerm) activeTest = 'complete_term';
    else if (substantiallyDisproportionate) activeTest = 'substantially_disproportionate';
    else if (state.not_equivalent_dividend) activeTest = 'not_equivalent';
    else if (state.partial_liquidation) activeTest = 'partial_liquidation';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s302.h2.result">§ 302 outcome</h2>
            <div class="cards">
                <div class="card ${saleTreatment ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s302.card.treatment">Treatment</div>
                    <div class="value">${saleTreatment ? esc(t('view.s302.treat.sale')) : esc(t('view.s302.treat.dividend'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s302.card.test">Qualifying test</div>
                    <div class="value">${esc(t('view.s302.test.' + activeTest))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s302.card.pre_post">Pre / Post %</div>
                    <div class="value">${state.pre_redemption_pct.toFixed(2)}% / ${state.post_redemption_pct.toFixed(2)}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s302.card.gain">Realized gain</div>
                    <div class="value">$${gain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s302.card.sale_tax">Sale tax (if sale)</div>
                    <div class="value">$${saleTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s302.card.dividend_tax">Dividend tax (if dividend)</div>
                    <div class="value">$${dividendTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s302.card.final">Final tax</div>
                    <div class="value">$${finalTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${!saleTreatment && state.shareholder_basis > 0 ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s302.dividend_note">
                    Dividend treatment: $${state.shareholder_basis.toLocaleString()} basis is NOT recovered
                    against dividend amount. Instead added to remaining shares' basis (or lost if complete
                    termination via constructive). This is the major punishment of dividend characterization.
                </p>
            ` : ''}
        </div>
    `;
}
