// Net Investment Income Tax (NIIT) — additional 3.8% surtax on
// investment income for high earners. Enacted under ACA (2013) as the
// "Medicare contribution tax." Applies to lesser of:
//   (a) net investment income, OR
//   (b) modified AGI in excess of threshold
// Thresholds (NOT inflation-adjusted — frozen since 2013):
//   single $200,000 · MFJ $250,000 · MFS $125,000 · HoH $200,000
// Net investment income = interest + dividends + cap gains + rents
// + royalties + non-qual annuities + passive biz income MINUS allocable
// deductions. Excludes wages, self-employment, retirement distributions,
// active business income.

import { esc } from '../util.js';

const THRESHOLD = { single: 200000, mfj: 250000, mfs: 125000, hoh: 200000 };
const NIIT_RATE = 0.038;

export async function renderNiitCalculator(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.niit_calculator.title">// NIIT · 3.8% INVESTMENT SURTAX</span></h1>
        <p class="muted small" data-i18n-html="view.niit_calculator.intro">
            Affordable Care Act's <strong>3.8% Net Investment Income Tax</strong>.
            Applies to lesser of (a) net investment income, or (b) MAGI over the
            threshold. Thresholds <strong>have NOT been indexed since 2013</strong> —
            $200k single / $250k MFJ / $125k MFS — increasingly affecting middle
            earners as nominal income drifts upward.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small">Filing status</span>
                    <select id="niit-status" style="width:100%">
                        <option value="single">Single</option>
                        <option value="mfj" selected>MFJ</option>
                        <option value="mfs">MFS</option>
                        <option value="hoh">HoH</option>
                    </select>
                </label>
                <label>
                    <span class="muted small">MAGI $</span>
                    <input type="number" id="niit-magi" step="1000" min="0" value="320000" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Interest income $</span>
                    <input type="number" id="niit-interest" step="100" min="0" value="5000" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Dividends (qual + ord) $</span>
                    <input type="number" id="niit-div" step="100" min="0" value="12000" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Net capital gains $</span>
                    <input type="number" id="niit-gains" step="100" value="40000" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Rental net income $</span>
                    <input type="number" id="niit-rent" step="100" value="8000" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Royalties + passive biz $</span>
                    <input type="number" id="niit-passive" step="100" min="0" value="0" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Allocable deductions $</span>
                    <input type="number" id="niit-ded" step="100" min="0" value="0" style="width:100%">
                </label>
            </div>
            <button class="btn btn-sm primary" id="niit-run">⚡ Compute</button>
            <div id="niit-result" style="margin-top:12px"></div>
        </div>
    `;
    mount.querySelectorAll('#niit-status, #niit-magi, #niit-interest, #niit-div, #niit-gains, #niit-rent, #niit-passive, #niit-ded').forEach(el => {
        el.addEventListener('input', () => compute(mount));
    });
    mount.querySelector('#niit-run').addEventListener('click', () => compute(mount));
    compute(mount);
}

function compute(mount) {
    const status = mount.querySelector('#niit-status').value;
    const magi = parseFloat(mount.querySelector('#niit-magi').value) || 0;
    const interest = parseFloat(mount.querySelector('#niit-interest').value) || 0;
    const div = parseFloat(mount.querySelector('#niit-div').value) || 0;
    const gains = parseFloat(mount.querySelector('#niit-gains').value) || 0;
    const rent = parseFloat(mount.querySelector('#niit-rent').value) || 0;
    const passive = parseFloat(mount.querySelector('#niit-passive').value) || 0;
    const ded = parseFloat(mount.querySelector('#niit-ded').value) || 0;
    const result = mount.querySelector('#niit-result');

    const grossInvestment = interest + div + gains + rent + passive;
    const netInvestment = Math.max(0, grossInvestment - ded);
    const thresh = THRESHOLD[status];
    const magiExcess = Math.max(0, magi - thresh);
    const subject = Math.min(netInvestment, magiExcess);
    const niitTax = subject * NIIT_RATE;
    const effOnInvestment = netInvestment > 0 ? niitTax / netInvestment : 0;

    result.innerHTML = `
        <div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
            <div class="card"><div class="label">MAGI threshold</div><div class="value">$${fmt(thresh, 0)}</div><div class="muted small">${esc(status.toUpperCase())} (frozen 2013)</div></div>
            <div class="card"><div class="label">MAGI excess</div><div class="value ${magiExcess > 0 ? 'neg' : 'pos'}">$${fmt(magiExcess, 0)}</div></div>
            <div class="card"><div class="label">Net investment income</div><div class="value">$${fmt(netInvestment, 0)}</div></div>
            <div class="card"><div class="label">Subject to NIIT</div><div class="value">$${fmt(subject, 0)}</div><div class="muted small">min(net inv, MAGI excess)</div></div>
            <div class="card"><div class="label">NIIT @ 3.8%</div><div class="value neg"><strong>$${fmt(niitTax, 0)}</strong></div></div>
            <div class="card"><div class="label">Effective on investment $</div><div class="value">${fmt(effOnInvestment * 100, 2)}%</div></div>
        </div>
        <h3 class="section-title">Investment income detail</h3>
        <table class="trades" data-table-key="niit-detail">
            <thead><tr><th>Component</th><th>Amount</th></tr></thead>
            <tbody>
                <tr><td>Interest</td><td>$${fmt(interest, 0)}</td></tr>
                <tr><td>Dividends</td><td>$${fmt(div, 0)}</td></tr>
                <tr><td>Net capital gains</td><td>$${fmt(gains, 0)}</td></tr>
                <tr><td>Rental net income</td><td>$${fmt(rent, 0)}</td></tr>
                <tr><td>Royalties / passive biz</td><td>$${fmt(passive, 0)}</td></tr>
                <tr><td class="muted">Less: allocable deductions</td><td class="muted">-$${fmt(ded, 0)}</td></tr>
                <tr><td><strong>Net investment income</strong></td><td><strong>$${fmt(netInvestment, 0)}</strong></td></tr>
            </tbody>
        </table>
        ${magiExcess <= 0 ? '<p class="pos small" style="margin-top:8px"><strong>Not subject to NIIT</strong> — MAGI below threshold.</p>' : netInvestment <= 0 ? '<p class="pos small" style="margin-top:8px"><strong>No net investment income</strong> — NIIT does not apply.</p>' : ''}
    `;
}

function fmt(n, d) {
    if (n == null || !Number.isFinite(Number(n))) return '—';
    return Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}
