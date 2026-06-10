// Real Estate Cap Rate + Cash-on-Cash + 1% Rule + GRM.
// Standard rental-property underwriting suite:
//   - Cap Rate     = NOI / property value (UNLEVERED yield)
//   - Cash-on-Cash = annual cash flow / cash invested (LEVERED return)
//   - 1% Rule      = monthly rent ≥ 1% of price (heuristic for cash flow positive)
//   - GRM          = price / annual gross rent (lower = better deal)
// NOI = gross rents − vacancy − op-ex (taxes, insurance, maintenance,
// management, HOA) but NOT debt service or depreciation.

import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderRealEstateCapRate(mount, _state) {
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
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small">Purchase price $</span>
                    <input type="number" id="re-price" step="5000" min="0" value="350000" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Monthly rent $</span>
                    <input type="number" id="re-rent" step="50" min="0" value="2800" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Down payment %</span>
                    <input type="number" id="re-down" step="5" min="0" max="100" value="25" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Mortgage rate %</span>
                    <input type="number" id="re-rate" step="0.125" min="0" max="20" value="7.0" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Loan term (years)</span>
                    <input type="number" id="re-term" step="1" min="1" max="40" value="30" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Closing costs $</span>
                    <input type="number" id="re-closing" step="500" min="0" value="9000" style="width:100%">
                </label>
            </div>
            <h3 class="section-title">Operating expenses (annual)</h3>
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small">Vacancy %</span>
                    <input type="number" id="re-vac" step="0.5" min="0" max="50" value="5" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Property tax $/yr</span>
                    <input type="number" id="re-tax" step="100" min="0" value="3800" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Insurance $/yr</span>
                    <input type="number" id="re-ins" step="100" min="0" value="1400" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Maintenance / CapEx % of rent</span>
                    <input type="number" id="re-maint" step="0.5" min="0" max="50" value="10" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Property mgmt % of rent</span>
                    <input type="number" id="re-mgmt" step="0.5" min="0" max="20" value="8" style="width:100%">
                </label>
                <label>
                    <span class="muted small">HOA + utilities $/mo</span>
                    <input type="number" id="re-hoa" step="25" min="0" value="0" style="width:100%">
                </label>
            </div>
            <button class="btn btn-sm primary" id="re-run">⚡ Compute</button>
            <div id="re-result" style="margin-top:12px"></div>
        </div>
    `;
    mount.querySelectorAll('#re-price, #re-rent, #re-down, #re-rate, #re-term, #re-closing, #re-vac, #re-tax, #re-ins, #re-maint, #re-mgmt, #re-hoa').forEach(el => {
        el.addEventListener('input', () => compute(mount));
    });
    mount.querySelector('#re-run').addEventListener('click', () => compute(mount));
    compute(mount);
}

function compute(mount) {
    const price = parseFloat(mount.querySelector('#re-price').value) || 0;
    const rent = parseFloat(mount.querySelector('#re-rent').value) || 0;
    const downPct = parseFloat(mount.querySelector('#re-down').value) / 100;
    const rate = parseFloat(mount.querySelector('#re-rate').value) / 100;
    const term = parseInt(mount.querySelector('#re-term').value, 10) || 30;
    const closing = parseFloat(mount.querySelector('#re-closing').value) || 0;
    const vacPct = parseFloat(mount.querySelector('#re-vac').value) / 100;
    const propTax = parseFloat(mount.querySelector('#re-tax').value) || 0;
    const ins = parseFloat(mount.querySelector('#re-ins').value) || 0;
    const maintPct = parseFloat(mount.querySelector('#re-maint').value) / 100;
    const mgmtPct = parseFloat(mount.querySelector('#re-mgmt').value) / 100;
    const hoaMo = parseFloat(mount.querySelector('#re-hoa').value) || 0;
    const result = mount.querySelector('#re-result');

    if (price <= 0 || rent <= 0) {
        result.innerHTML = `<p class="muted">Price and rent > 0 required.</p>`;
        return;
    }

    const grossAnnualRent = rent * 12;
    const vacancy = grossAnnualRent * vacPct;
    const maint = grossAnnualRent * maintPct;
    const mgmt = grossAnnualRent * mgmtPct;
    const hoaAnnual = hoaMo * 12;
    const effectiveRent = grossAnnualRent - vacancy;
    const opex = propTax + ins + maint + mgmt + hoaAnnual;
    const noi = effectiveRent - opex;
    const capRate = noi / price;

    // Mortgage payment.
    const loan = price * (1 - downPct);
    const r_m = rate / 12;
    const n = term * 12;
    const pi = r_m === 0 ? loan / n : loan * r_m * Math.pow(1 + r_m, n) / (Math.pow(1 + r_m, n) - 1);
    const annualDebtService = pi * 12;
    const cashFlow = noi - annualDebtService;
    const cashInvested = price * downPct + closing;
    const cashOnCash = cashInvested > 0 ? cashFlow / cashInvested : 0;
    const dscr = annualDebtService > 0 ? noi / annualDebtService : Infinity;

    const onePctRule = (rent / price) * 100;
    const onePctOk = onePctRule >= 1.0;
    const grm = price / grossAnnualRent;

    const verdict = capRate >= 0.08 ? 'STRONG' : capRate >= 0.06 ? 'OK' : capRate >= 0.04 ? 'WEAK' : 'PASS';
    const verdictCls = capRate >= 0.08 ? 'pos' : capRate >= 0.04 ? '' : 'neg';

    result.innerHTML = `
        <div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
            <div class="card"><div class="label">Cap rate (unlevered)</div><div class="value ${verdictCls}"><strong>${fmt(capRate * 100, 2)}%</strong></div><div class="muted small">NOI / price</div></div>
            <div class="card"><div class="label">Cash-on-cash return</div><div class="value ${cashOnCash >= 0.08 ? 'pos' : cashOnCash >= 0 ? '' : 'neg'}"><strong>${fmt(cashOnCash * 100, 2)}%</strong></div><div class="muted small">CF / cash invested</div></div>
            <div class="card"><div class="label">Annual cash flow</div><div class="value ${cashFlow > 0 ? 'pos' : 'neg'}">$${fmt(cashFlow, 0)}</div><div class="muted small">$${fmt(cashFlow / 12, 0)}/mo</div></div>
            <div class="card"><div class="label">DSCR</div><div class="value ${dscr >= 1.25 ? 'pos' : dscr >= 1 ? '' : 'neg'}">${dscr === Infinity ? '∞' : fmt(dscr, 2)}</div><div class="muted small">≥1.25 for lender approval</div></div>
            <div class="card"><div class="label">1% rule (rent/price)</div><div class="value ${onePctOk ? 'pos' : 'neg'}">${fmt(onePctRule, 2)}%</div><div class="muted small">≥1% = ${onePctOk ? '✓' : '✗'} cash flow signal</div></div>
            <div class="card"><div class="label">GRM</div><div class="value">${fmt(grm, 1)}</div><div class="muted small">Price / annual rent — lower better</div></div>
            <div class="card"><div class="label">Verdict</div><div class="value ${verdictCls}"><strong>${esc(verdict)}</strong></div></div>
        </div>
        <h3 class="section-title">P&L breakdown</h3>
        <table class="trades" data-table-key="re-pl">
            <thead><tr><th>Line</th><th>Annual</th><th>Monthly</th></tr></thead>
            <tbody>
                <tr><td>Gross scheduled rent</td><td>$${fmt(grossAnnualRent, 0)}</td><td>$${fmt(grossAnnualRent / 12, 0)}</td></tr>
                <tr><td class="muted">Less vacancy (${fmt(vacPct * 100, 1)}%)</td><td class="muted">-$${fmt(vacancy, 0)}</td><td class="muted">-$${fmt(vacancy / 12, 0)}</td></tr>
                <tr><td><strong>Effective gross income</strong></td><td><strong>$${fmt(effectiveRent, 0)}</strong></td><td><strong>$${fmt(effectiveRent / 12, 0)}</strong></td></tr>
                <tr><td>Property tax</td><td>-$${fmt(propTax, 0)}</td><td>-$${fmt(propTax / 12, 0)}</td></tr>
                <tr><td>Insurance</td><td>-$${fmt(ins, 0)}</td><td>-$${fmt(ins / 12, 0)}</td></tr>
                <tr><td>Maintenance / CapEx</td><td>-$${fmt(maint, 0)}</td><td>-$${fmt(maint / 12, 0)}</td></tr>
                <tr><td>Property management</td><td>-$${fmt(mgmt, 0)}</td><td>-$${fmt(mgmt / 12, 0)}</td></tr>
                <tr><td>HOA + utilities</td><td>-$${fmt(hoaAnnual, 0)}</td><td>-$${fmt(hoaMo, 0)}</td></tr>
                <tr><td><strong>NOI</strong></td><td><strong>$${fmt(noi, 0)}</strong></td><td><strong>$${fmt(noi / 12, 0)}</strong></td></tr>
                <tr><td class="muted">Less debt service (P+I)</td><td class="muted">-$${fmt(annualDebtService, 0)}</td><td class="muted">-$${fmt(pi, 0)}</td></tr>
                <tr><td><strong>Cash flow</strong></td><td class="${cashFlow > 0 ? 'pos' : 'neg'}"><strong>$${fmt(cashFlow, 0)}</strong></td><td class="${cashFlow > 0 ? 'pos' : 'neg'}"><strong>$${fmt(cashFlow / 12, 0)}</strong></td></tr>
            </tbody>
        </table>
    `;
}

function fmt(n, d) {
    if (n == null || !Number.isFinite(Number(n))) return '—';
    return Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}
