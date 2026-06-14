// Real Estate Cap Rate + Cash-on-Cash + 1% Rule + GRM + DSCR.
// Standard rental-property underwriting suite. Computation runs server-side via
// /calc/cap-rate (traderview-core::cap_rate) — a faithful port of the former
// client-side math, now Python-pinned and unit-tested. Styling is class-based
// (no inline styles) so it renders correctly in release WebKit.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import { esc } from '../util.js';
import * as enh from '../calc_enhance.js';

const fmt = (n, d) => (n == null || !Number.isFinite(Number(n)) ? '—' : Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d }));
const VIEW = 'real-estate-cap-rate';
let lastReport = null;
let lastBody = null;

export async function renderRealEstateCapRate(mount, _state) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.real_estate_cap_rate.title">// RENTAL PROPERTY UNDERWRITING</span></h1>
        <p class="muted small" data-i18n-html="view.real_estate_cap_rate.intro">
            Cap rate, cash-on-cash return, 1% rule, gross rent multiplier — the
            standard rental-property underwriting toolkit. <strong>Cap rate</strong>
            is the unlevered yield (NOI ÷ price), market-comparable.
            <strong>Cash-on-cash</strong> includes leverage and is what you actually
            see in your bank account / year. <strong>1% rule</strong> heuristic:
            monthly rent ≥ 1% of purchase price typically cash-flow positive in most
            markets (was easier pre-2020 — most markets need 0.8% now).
        </p>
        <div class="chart-panel">
            <h3 class="section-title">Property</h3>
            <div class="re-grid">
                <label><span class="muted small">Purchase price $</span>
                    <input type="number" id="re-price" step="5000" min="0" value="350000"></label>
                <label><span class="muted small">Monthly rent $</span>
                    <input type="number" id="re-rent" step="50" min="0" value="2800"></label>
                <label><span class="muted small">Down payment %</span>
                    <input type="number" id="re-down" step="5" min="0" max="100" value="25"></label>
                <label><span class="muted small">Mortgage rate %</span>
                    <input type="number" id="re-rate" step="0.125" min="0" max="20" value="7.0"></label>
                <label><span class="muted small">Loan term (years)</span>
                    <input type="number" id="re-term" step="1" min="1" max="40" value="30"></label>
                <label><span class="muted small">Closing costs $</span>
                    <input type="number" id="re-closing" step="500" min="0" value="9000"></label>
            </div>
            <h3 class="section-title">Operating expenses (annual)</h3>
            <div class="re-grid">
                <label><span class="muted small">Vacancy %</span>
                    <input type="number" id="re-vac" step="0.5" min="0" max="50" value="5"></label>
                <label><span class="muted small">Property tax $/yr</span>
                    <input type="number" id="re-tax" step="100" min="0" value="3800"></label>
                <label><span class="muted small">Insurance $/yr</span>
                    <input type="number" id="re-ins" step="100" min="0" value="1400"></label>
                <label><span class="muted small">Maintenance / CapEx % of rent</span>
                    <input type="number" id="re-maint" step="0.5" min="0" max="50" value="10"></label>
                <label><span class="muted small">Property mgmt % of rent</span>
                    <input type="number" id="re-mgmt" step="0.5" min="0" max="20" value="8"></label>
                <label><span class="muted small">HOA + utilities $/mo</span>
                    <input type="number" id="re-hoa" step="25" min="0" value="0"></label>
            </div>
            <button class="btn btn-sm primary" id="re-run">⚡ Compute</button>
            <div id="re-tools" class="ce-toolbar"></div>
            <div id="re-result" class="re-result"></div>
        </div>
    `;
    applyUiI18n(mount);

    const num = (id) => parseFloat(mount.querySelector(id).value) || 0;
    const readBody = () => ({
        price_usd: num('#re-price'),
        monthly_rent_usd: num('#re-rent'),
        down_payment_frac: num('#re-down') / 100,
        mortgage_rate_frac: num('#re-rate') / 100,
        loan_term_years: Math.round(num('#re-term')) || 30,
        closing_costs_usd: num('#re-closing'),
        vacancy_frac: num('#re-vac') / 100,
        property_tax_annual_usd: num('#re-tax'),
        insurance_annual_usd: num('#re-ins'),
        maintenance_frac: num('#re-maint') / 100,
        management_frac: num('#re-mgmt') / 100,
        hoa_monthly_usd: num('#re-hoa'),
    });
    const compute = async () => {
        const body = readBody();
        try {
            const r = await api.calcCapRate(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            await renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || 'Could not compute the underwriting.', { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#re-tools'), {
        viewId: VIEW, link: false, filename: 'cap-rate.csv',
        getRows: () => reportRows(lastReport, lastBody),
    });
    mount.querySelectorAll('#re-price, #re-rent, #re-down, #re-rate, #re-term, #re-closing, #re-vac, #re-tax, #re-ins, #re-maint, #re-mgmt, #re-hoa').forEach(el => {
        el.addEventListener('input', debounce(compute, 250));
    });
    mount.querySelector('#re-run').addEventListener('click', compute);
    compute();
}

function reportRows(r, body) {
    if (!r || !r.valid) return [];
    return [
        ['metric', 'value'],
        ['cap_rate_pct', r.cap_rate_pct],
        ['cash_on_cash_pct', r.cash_on_cash_pct],
        ['annual_cash_flow_usd', r.annual_cash_flow_usd],
        ['dscr', r.dscr == null ? '' : r.dscr],
        ['one_pct_rule_pct', r.one_pct_rule_pct],
        ['grm', r.grm],
        ['noi_usd', r.noi_usd],
        ['verdict', r.verdict],
    ];
}

async function renderResult(mount, r, body, tok) {
    const result = mount.querySelector('#re-result');
    if (!r.valid) {
        result.innerHTML = `<p class="muted">Price and rent &gt; 0 required.</p>`;
        return;
    }
    const verdictCls = r.cap_rate_pct >= 8 ? 'pos' : r.cap_rate_pct >= 4 ? '' : 'neg';
    const dscrTxt = r.dscr == null ? '∞' : fmt(r.dscr, 2);
    // Line chart: cap rate as purchase price sweeps 0.5× → 1.5× (yield falls as price rises).
    const base = body.price_usd || 350000;
    const xs = enh.linspace(base * 0.5, base * 1.5, 13);
    const pts = await Promise.all(xs.map(async (p) => {
        const rr = await api.calcCapRate({ ...body, price_usd: p });
        return { x: p / 1000, y: rr && rr.valid ? rr.cap_rate_pct : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'price $k', ylabel: 'cap rate %' });
    result.innerHTML = `
        <div class="cards">
            <div class="card"><div class="label">Cap rate (unlevered)</div><div class="value ${verdictCls}"><strong>${fmt(r.cap_rate_pct, 2)}%</strong></div><div class="muted small">NOI / price</div></div>
            <div class="card"><div class="label">Cash-on-cash return</div><div class="value ${r.cash_on_cash_pct >= 8 ? 'pos' : r.cash_on_cash_pct >= 0 ? '' : 'neg'}"><strong>${fmt(r.cash_on_cash_pct, 2)}%</strong></div><div class="muted small">CF / cash invested</div></div>
            <div class="card"><div class="label">Annual cash flow</div><div class="value ${r.annual_cash_flow_usd > 0 ? 'pos' : 'neg'}">$${fmt(r.annual_cash_flow_usd, 0)}</div><div class="muted small">$${fmt(r.monthly_cash_flow_usd, 0)}/mo</div></div>
            <div class="card"><div class="label">DSCR</div><div class="value ${r.dscr == null || r.dscr >= 1.25 ? 'pos' : r.dscr >= 1 ? '' : 'neg'}">${dscrTxt}</div><div class="muted small">≥1.25 for lender approval</div></div>
            <div class="card"><div class="label">1% rule (rent/price)</div><div class="value ${r.one_pct_ok ? 'pos' : 'neg'}">${fmt(r.one_pct_rule_pct, 2)}%</div><div class="muted small">≥1% = ${r.one_pct_ok ? '✓' : '✗'} cash flow signal</div></div>
            <div class="card"><div class="label">GRM</div><div class="value">${fmt(r.grm, 1)}</div><div class="muted small">Price / annual rent — lower better</div></div>
            <div class="card"><div class="label">Verdict</div><div class="value ${verdictCls}"><strong>${esc(r.verdict)}</strong></div></div>
        </div>
        ${chart}
        <h3 class="section-title">P&L breakdown</h3>
        <table class="trades" data-table-key="re-pl">
            <thead><tr><th>Line</th><th>Annual</th><th>Monthly</th></tr></thead>
            <tbody>
                <tr><td>Gross scheduled rent</td><td>$${fmt(r.gross_annual_rent_usd, 0)}</td><td>$${fmt(r.gross_annual_rent_usd / 12, 0)}</td></tr>
                <tr><td class="muted">Less vacancy (${fmt(body.vacancy_frac * 100, 1)}%)</td><td class="muted">-$${fmt(r.vacancy_loss_usd, 0)}</td><td class="muted">-$${fmt(r.vacancy_loss_usd / 12, 0)}</td></tr>
                <tr><td><strong>Effective gross income</strong></td><td><strong>$${fmt(r.effective_gross_income_usd, 0)}</strong></td><td><strong>$${fmt(r.effective_gross_income_usd / 12, 0)}</strong></td></tr>
                <tr><td>Property tax</td><td>-$${fmt(body.property_tax_annual_usd, 0)}</td><td>-$${fmt(body.property_tax_annual_usd / 12, 0)}</td></tr>
                <tr><td>Insurance</td><td>-$${fmt(body.insurance_annual_usd, 0)}</td><td>-$${fmt(body.insurance_annual_usd / 12, 0)}</td></tr>
                <tr><td>Maintenance / CapEx</td><td>-$${fmt(r.maintenance_usd, 0)}</td><td>-$${fmt(r.maintenance_usd / 12, 0)}</td></tr>
                <tr><td>Property management</td><td>-$${fmt(r.management_usd, 0)}</td><td>-$${fmt(r.management_usd / 12, 0)}</td></tr>
                <tr><td>HOA + utilities</td><td>-$${fmt(r.hoa_annual_usd, 0)}</td><td>-$${fmt(body.hoa_monthly_usd, 0)}</td></tr>
                <tr><td><strong>NOI</strong></td><td><strong>$${fmt(r.noi_usd, 0)}</strong></td><td><strong>$${fmt(r.noi_usd / 12, 0)}</strong></td></tr>
                <tr><td class="muted">Less debt service (P+I)</td><td class="muted">-$${fmt(r.annual_debt_service_usd, 0)}</td><td class="muted">-$${fmt(r.monthly_pi_usd, 0)}</td></tr>
                <tr><td><strong>Cash flow</strong></td><td class="${r.annual_cash_flow_usd > 0 ? 'pos' : 'neg'}"><strong>$${fmt(r.annual_cash_flow_usd, 0)}</strong></td><td class="${r.annual_cash_flow_usd > 0 ? 'pos' : 'neg'}"><strong>$${fmt(r.monthly_cash_flow_usd, 0)}</strong></td></tr>
            </tbody>
        </table>
    `;
}
