// S-Corp Election Calculator — compares LLC sole-prop vs S-corp tax burden
// at a given net business income. S-corp wins by carving out "reasonable
// compensation" as W-2 wages (subject to SE tax) and treating the rest as
// distributions (NOT subject to SE tax). Catch: S-corp costs $1.5-3k/yr in
// payroll + filing fees and you can't underpay reasonable comp.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    net_income: 200_000,
    reasonable_comp_pct: 0.40,  // 40% of net as W-2 wages is a common heuristic
    payroll_cost: 600,           // Gusto / OnPay roughly $50/mo per employee
    extra_filing_cost: 1_200,    // S-corp tax prep premium vs sole-prop
    marginal_rate: 0.32,
};

export async function renderScorpCalc(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.scorp.h1.title">// S-CORP ELECTION CALCULATOR</span></h1>
        <p class="muted small" data-i18n="view.scorp.hint.intro">
            S-corp election saves SE tax on the portion NOT paid as W-2 wages.
            Reasonable-comp rule: IRS expects ~40-60% of profit as W-2 for service
            businesses. Below ~$80k net income, S-corp overhead usually exceeds savings.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.scorp.h2.inputs">Inputs</h2>
            <form id="sc-form" class="inline-form">
                <label><span data-i18n="view.scorp.label.net_income">Net business income ($)</span>
                    <input type="number" step="0.01" name="net_income" value="${state.net_income}"></label>
                <label><span data-i18n="view.scorp.label.reasonable_comp_pct">Reasonable comp as % of net</span>
                    <input type="number" step="0.05" name="reasonable_comp_pct" value="${state.reasonable_comp_pct}" min="0.10" max="1.0"></label>
                <label><span data-i18n="view.scorp.label.payroll_cost">Payroll service cost ($/yr)</span>
                    <input type="number" step="0.01" name="payroll_cost" value="${state.payroll_cost}"></label>
                <label><span data-i18n="view.scorp.label.extra_filing">S-corp tax prep premium ($/yr)</span>
                    <input type="number" step="0.01" name="extra_filing_cost" value="${state.extra_filing_cost}"></label>
                <label><span data-i18n="view.scorp.label.marginal_rate">Marginal federal rate %</span>
                    <input type="number" step="0.5" name="marginal_rate" value="${(state.marginal_rate * 100).toFixed(1)}"></label>
                <button class="primary" type="submit" data-i18n="view.scorp.btn.recompute">Recompute</button>
            </form>
        </div>
        <div id="sc-output"></div>
    `;
    document.getElementById('sc-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.net_income = Number(fd.get('net_income')) || 0;
        state.reasonable_comp_pct = Number(fd.get('reasonable_comp_pct')) || 0.40;
        state.payroll_cost = Number(fd.get('payroll_cost')) || 0;
        state.extra_filing_cost = Number(fd.get('extra_filing_cost')) || 0;
        state.marginal_rate = (Number(fd.get('marginal_rate')) || 32) / 100;
        renderOutput();
    });
    renderOutput();
}

async function renderOutput() {
    const el = document.getElementById('sc-output');
    if (!el) return;
    let r;
    try {
        r = await api.calcScorpElection({
            net_income_usd: state.net_income,
            reasonable_comp_fraction: state.reasonable_comp_pct,
            payroll_cost_usd: state.payroll_cost,
            extra_filing_cost_usd: state.extra_filing_cost,
            marginal_rate_pct: state.marginal_rate * 100,
        });
    } catch (e) {
        el.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
        return;
    }
    const sp = { se_base: r.se_base_usd, se_tax: r.se_tax_usd };
    const sc = {
        w2_wages: r.w2_wages_usd,
        distributions: r.distributions_usd,
        fica_employee: r.fica_employee_usd,
        fica_employer: r.fica_employer_usd,
    };
    const totalOverhead = r.total_overhead_usd;
    const grossSavings = r.gross_savings_usd;
    const netSavings = r.net_savings_usd;
    const cls = netSavings > 0 ? 'pos' : netSavings < 0 ? 'neg' : '';
    const recommendation = t('view.scorp.recommend.' + r.recommendation);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.scorp.h2.comparison">Side-by-side comparison</h2>
            <div class="cards">
                <div class="card ${cls}">
                    <div class="label" data-i18n="view.scorp.card.net_savings">Net annual savings (S-corp − sole-prop)</div>
                    <div class="value">$${netSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.scorp.card.gross_savings">Gross SE tax savings</div>
                    <div class="value">$${grossSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.scorp.card.overhead">Overhead (payroll + prep)</div>
                    <div class="value">$${totalOverhead.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.scorp.card.recommendation">Recommendation</div>
                    <div class="value">${esc(recommendation)}</div>
                </div>
            </div>
        </div>
        <div class="panel-grid">
            <div class="chart-panel">
                <h2 data-i18n="view.scorp.h2.sole_prop">LLC / Sole Prop</h2>
                <table class="trades"><tbody>
                    <tr><td data-i18n="view.scorp.row.net_income">Net business income</td>
                        <td>$${state.net_income.toLocaleString()}</td></tr>
                    <tr><td data-i18n="view.scorp.row.se_base">SE tax base (92.35%)</td>
                        <td>$${sp.se_base.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td></tr>
                    <tr><td><strong data-i18n="view.scorp.row.se_tax">Total SE tax (15.3%)</strong></td>
                        <td><strong class="neg">$${sp.se_tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</strong></td></tr>
                    <tr><td data-i18n="view.scorp.row.half_se">½ SE deduction</td>
                        <td>$${(sp.se_tax / 2).toLocaleString(undefined, { maximumFractionDigits: 0 })}</td></tr>
                </tbody></table>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.scorp.h2.scorp">S-Corp</h2>
                <table class="trades"><tbody>
                    <tr><td data-i18n="view.scorp.row.w2_wages">W-2 wages (reasonable comp)</td>
                        <td>$${sc.w2_wages.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td></tr>
                    <tr><td data-i18n="view.scorp.row.distributions">K-1 distributions (no SE tax)</td>
                        <td>$${sc.distributions.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td></tr>
                    <tr><td data-i18n="view.scorp.row.fica_employee">FICA — employee side</td>
                        <td>$${sc.fica_employee.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td></tr>
                    <tr><td data-i18n="view.scorp.row.fica_employer">FICA — employer side</td>
                        <td>$${sc.fica_employer.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td></tr>
                    <tr><td><strong data-i18n="view.scorp.row.total_fica">Total FICA</strong></td>
                        <td><strong class="neg">$${(sc.fica_employee + sc.fica_employer).toLocaleString(undefined, { maximumFractionDigits: 0 })}</strong></td></tr>
                </tbody></table>
            </div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.scorp.h2.caveats">Caveats</h2>
            <p class="muted small" data-i18n="view.scorp.caveats.body">
                Reasonable comp is enforced — IRS reclassifies low-balled wages.
                S-corp adds Form 1120-S, K-1 issuance, quarterly payroll filings, state
                franchise taxes (CA $800/yr, NY/IL fees). Below ~$80k net income, overhead
                usually exceeds savings. Above $400k+ income, FICA cap reduces marginal benefit.
                Many traders use a SINGLE-OWNER S-corp specifically for QBI + SE savings.
            </p>
        </div>
    `;
}
