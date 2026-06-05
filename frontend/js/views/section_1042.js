// IRC § 1042 — ESOP Gain Deferral / Rollover.
// Sell qualified stock of closely-held domestic C-corp to ESOP that owns ≥ 30% post-sale.
// Reinvest proceeds in qualified replacement property (QRP) within 12 months.
// Capital gain DEFERRED until QRP sold. Basis from sold stock carries over to QRP.
// Massive estate planning + business succession tool.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const MIN_ESOP_OWNERSHIP_POST = 0.30;
const REINVEST_WINDOW_MONTHS = 12;
const REINVEST_WINDOW_PRIOR_MONTHS = 3;

let state = {
    sale_price: 0,
    cost_basis: 0,
    esop_post_ownership_pct: 0,
    qrp_purchased: 0,
    months_to_qrp_purchase: 0,
    expected_qrp_growth: 0.06,
    expected_qrp_holding_years: 20,
    marginal_rate: 0.37,
    ltcg_rate: 0.20,
};

export async function renderSection1042(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1042.h1.title">// § 1042 ESOP GAIN DEFERRAL</span></h1>
        <p class="muted small" data-i18n="view.s1042.hint.intro">
            Sell qualified stock of closely-held domestic C-corp to ESOP that owns
            <strong>≥ 30% post-sale</strong>. Reinvest proceeds in <strong>qualified replacement
            property (QRP)</strong> within 12 months (or 3 mo prior). Cap gain DEFERRED until QRP
            sold. Basis carries over. <strong>Estate planning hack:</strong> die holding QRP →
            step-up at death wipes out deferred gain (§ 1014). Massive succession + tax tool
            for retiring founders.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1042.h2.inputs">Inputs</h2>
            <form id="s1042-form" class="inline-form">
                <label><span data-i18n="view.s1042.label.sale">Sale price to ESOP ($)</span>
                    <input type="number" step="0.01" name="sale_price" value="${state.sale_price}"></label>
                <label><span data-i18n="view.s1042.label.basis">Your cost basis ($)</span>
                    <input type="number" step="0.01" name="cost_basis" value="${state.cost_basis}"></label>
                <label><span data-i18n="view.s1042.label.esop_pct">ESOP post-sale ownership %</span>
                    <input type="number" step="0.01" name="esop_post_ownership_pct" value="${state.esop_post_ownership_pct}"></label>
                <label><span data-i18n="view.s1042.label.qrp">QRP purchased ($)</span>
                    <input type="number" step="0.01" name="qrp_purchased" value="${state.qrp_purchased}"></label>
                <label><span data-i18n="view.s1042.label.months">Months to QRP purchase</span>
                    <input type="number" step="1" name="months_to_qrp_purchase" value="${state.months_to_qrp_purchase}"></label>
                <label><span data-i18n="view.s1042.label.growth">QRP expected growth %</span>
                    <input type="number" step="0.01" name="expected_qrp_growth" value="${state.expected_qrp_growth}"></label>
                <label><span data-i18n="view.s1042.label.years">Years held (if not stepped up)</span>
                    <input type="number" step="1" name="expected_qrp_holding_years" value="${state.expected_qrp_holding_years}"></label>
                <label><span data-i18n="view.s1042.label.marginal">Marginal %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <label><span data-i18n="view.s1042.label.ltcg">LTCG %</span>
                    <input type="number" step="0.01" name="ltcg_rate" value="${state.ltcg_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s1042.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1042-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1042.h2.requirements">Requirements</h2>
            <ul class="muted small">
                <li data-i18n="view.s1042.req.c_corp">Selling shareholder of domestic C-corp (not S-corp; S → § 1042 unavailable but S-ESOP has tax-free advantages)</li>
                <li data-i18n="view.s1042.req.holding_3yr">Stock held ≥ 3 years before sale</li>
                <li data-i18n="view.s1042.req.esop_30pct">ESOP owns ≥ 30% of total outstanding stock immediately AFTER sale</li>
                <li data-i18n="view.s1042.req.qrp_window">Acquire QRP within 12 months after sale (or 3 mo before, with Notice of Election)</li>
                <li data-i18n="view.s1042.req.notice">File Notice of Election (§ 1042(c)(3)(B)) with return</li>
                <li data-i18n="view.s1042.req.esop_statement">ESOP files statement consenting to § 4978 excise on early disposition</li>
                <li data-i18n="view.s1042.req.no_family">Seller / family + 25% shareholders can't receive ESOP allocations for 10 yrs</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1042.h2.qrp">Qualified Replacement Property (QRP)</h2>
            <ul class="muted small">
                <li data-i18n="view.s1042.qrp.qualified">Stock OR bonds of US OPERATING companies</li>
                <li data-i18n="view.s1042.qrp.50_pct_rule">Issuer must derive &lt; 25% passive income and have ≥ 50% assets active</li>
                <li data-i18n="view.s1042.qrp.not_qualified">NOT: govt bonds, mutual funds, REITs, MLPs, S-corps, foreign corps</li>
                <li data-i18n="view.s1042.qrp.floating_rate">Common QRP: "floating-rate notes" of operating cos (low risk + § 1042-qualified)</li>
                <li data-i18n="view.s1042.qrp.margin_loan">Can use margin loan against QRP to monetize without triggering gain</li>
                <li data-i18n="view.s1042.qrp.diversify">Limited diversification options — managed-account QRP common</li>
            </ul>
        </div>
    `;
    document.getElementById('s1042-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.sale_price = Number(fd.get('sale_price')) || 0;
        state.cost_basis = Number(fd.get('cost_basis')) || 0;
        state.esop_post_ownership_pct = Number(fd.get('esop_post_ownership_pct')) || 0;
        state.qrp_purchased = Number(fd.get('qrp_purchased')) || 0;
        state.months_to_qrp_purchase = Number(fd.get('months_to_qrp_purchase')) || 0;
        state.expected_qrp_growth = Number(fd.get('expected_qrp_growth')) || 0.06;
        state.expected_qrp_holding_years = Number(fd.get('expected_qrp_holding_years')) || 20;
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.37;
        state.ltcg_rate = Number(fd.get('ltcg_rate')) || 0.20;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1042-output');
    if (!el) return;
    const passes30Pct = state.esop_post_ownership_pct >= MIN_ESOP_OWNERSHIP_POST * 100;
    const passesWindow = state.months_to_qrp_purchase <= REINVEST_WINDOW_MONTHS;
    const reinvestmentRatio = state.sale_price > 0 ? Math.min(1, state.qrp_purchased / state.sale_price) : 0;
    const totalGain = Math.max(0, state.sale_price - state.cost_basis);
    const gainDeferred = passes30Pct && passesWindow ? totalGain * reinvestmentRatio : 0;
    const gainRecognized = totalGain - gainDeferred;
    const immediateTax = gainRecognized * state.ltcg_rate;
    const taxIfNoElection = totalGain * state.ltcg_rate;
    const immediateSavings = taxIfNoElection - immediateTax;
    // Eventual tax if QRP sold while alive
    const qrpFutureValue = state.qrp_purchased * Math.pow(1 + state.expected_qrp_growth, state.expected_qrp_holding_years);
    const qrpFutureGain = qrpFutureValue - state.cost_basis;
    const eventualTax = qrpFutureGain * state.ltcg_rate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1042.h2.result">§ 1042 outcome</h2>
            <div class="cards">
                <div class="card ${passes30Pct ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s1042.card.passes_30">Passes 30% ESOP test</div>
                    <div class="value">${passes30Pct ? esc(t('view.s1042.status.yes')) : esc(t('view.s1042.status.no'))}</div>
                </div>
                <div class="card ${passesWindow ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s1042.card.passes_window">Within 12-mo reinvestment</div>
                    <div class="value">${passesWindow ? esc(t('view.s1042.status.yes')) : esc(t('view.s1042.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1042.card.total_gain">Total gain</div>
                    <div class="value">$${totalGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1042.card.gain_deferred">Gain deferred</div>
                    <div class="value">$${gainDeferred.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1042.card.gain_recognized">Gain recognized now</div>
                    <div class="value">$${gainRecognized.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1042.card.immediate_savings">Immediate tax savings</div>
                    <div class="value">$${immediateSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1042.card.eventual_tax">Eventual tax if QRP sold</div>
                    <div class="value">$${eventualTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1042.card.step_up">If hold till death (step-up)</div>
                    <div class="value">$0</div>
                </div>
            </div>
        </div>
    `;
}
