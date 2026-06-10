// Reverse Mortgage (HECM) — FHA-insured Home Equity Conversion
// Mortgage. Borrower must be ≥62, principal residence, no income/credit
// test. Lender pays YOU (lump sum, line of credit, monthly tenure, term).
// Balance compounds; due when borrower moves out, sells, or dies.
// Non-recourse: heirs never owe more than home value.
//
// HUD Principal Limit Factor (PLF) tables key off age + expected rate.
// This impl uses a simple linear approximation that captures the right
// shape (older + lower rates = more cash). Use HUD's actual PLF table
// for a binding quote.

import { esc } from '../util.js';
import { t } from '../i18n.js';

const FHA_MAX_2025 = 1209750;        // FHA HECM lending limit (2025)
const ORIG_FEE_BASE = 2500;          // higher of $2,500 or 2% of first $200k + 1% over $200k, capped at $6k

export async function renderReverseMortgage(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.reverse_mortgage.title">// REVERSE MORTGAGE (HECM)</span></h1>
        <p class="muted small" data-i18n-html="view.reverse_mortgage.intro">
            FHA-insured Home Equity Conversion Mortgage. Borrower ≥62, home
            is primary residence. Lender pays YOU; balance compounds; due
            when you leave the home permanently. Non-recourse: heirs never
            owe more than the home is worth. 2025 FHA lending limit:
            <strong>$1,209,750</strong>. PLF below is a linear approximation —
            use HUD's official table for a real quote.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small">Youngest borrower age</span>
                    <input type="number" id="rm-age" step="1" min="62" max="100" value="70" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Home value $</span>
                    <input type="number" id="rm-value" step="10000" min="0" value="650000" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Existing mortgage $</span>
                    <input type="number" id="rm-mortgage" step="1000" min="0" value="120000" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Expected rate %</span>
                    <input type="number" id="rm-rate" step="0.1" min="3" max="12" value="7.5" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Home appreciation %/yr</span>
                    <input type="number" id="rm-appr" step="0.1" min="-5" max="10" value="3.0" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Payout type</span>
                    <select id="rm-payout" style="width:100%">
                        <option value="lump" selected>Lump sum</option>
                        <option value="tenure">Tenure (monthly for life)</option>
                        <option value="line">Line of credit</option>
                    </select>
                </label>
            </div>
            <button class="btn btn-sm primary" id="rm-run">⚡ Compute</button>
            <div id="rm-result" style="margin-top:12px"></div>
        </div>
    `;
    mount.querySelectorAll('#rm-age, #rm-value, #rm-mortgage, #rm-rate, #rm-appr, #rm-payout').forEach(el => {
        el.addEventListener('input', () => compute(mount));
    });
    mount.querySelector('#rm-run').addEventListener('click', () => compute(mount));
    compute(mount);
}

function compute(mount) {
    const age = Math.max(62, parseInt(mount.querySelector('#rm-age').value, 10) || 62);
    const value = parseFloat(mount.querySelector('#rm-value').value) || 0;
    const mortgage = parseFloat(mount.querySelector('#rm-mortgage').value) || 0;
    const rate = parseFloat(mount.querySelector('#rm-rate').value) / 100;
    const appr = parseFloat(mount.querySelector('#rm-appr').value) / 100;
    const payout = mount.querySelector('#rm-payout').value;
    const result = mount.querySelector('#rm-result');

    const maxClaim = Math.min(value, FHA_MAX_2025);
    // Linear PLF approximation: age 62 ~ 40%, age 80 ~ 60%, age 90+ ~ 75%
    // adjusted down by (expected_rate - 5%) × 4 percentage points.
    let plf = 0.40 + Math.max(0, age - 62) * 0.012;
    plf -= Math.max(0, rate - 0.05) * 4;
    plf = Math.max(0.20, Math.min(0.75, plf));
    const principalLimit = maxClaim * plf;
    // MIP (2.0% upfront), origination fee, closing costs (~$3-6k).
    const upfrontMip = maxClaim * 0.02;
    const origFee = Math.min(6000, Math.max(2500, Math.min(200000, maxClaim) * 0.02 + Math.max(0, maxClaim - 200000) * 0.01));
    const closing = 4000;
    const totalCosts = upfrontMip + origFee + closing;
    const netAvailableYear1 = principalLimit - mortgage - totalCosts;

    let payoutDescription;
    let monthlyPayment = 0;
    if (payout === 'lump') {
        payoutDescription = `<strong>$${fmt(netAvailableYear1, 0)}</strong> lump sum after closing`;
    } else if (payout === 'tenure') {
        // Tenure formula: PV of annuity at expected rate + 0.5% MIP, life expectancy via 100 - age.
        const yearsExpected = 100 - age;
        const r_m = (rate + 0.005) / 12;
        const n = yearsExpected * 12;
        monthlyPayment = r_m === 0
            ? netAvailableYear1 / n
            : netAvailableYear1 * r_m / (1 - Math.pow(1 + r_m, -n));
        payoutDescription = `<strong>$${fmt(monthlyPayment, 0)}/mo</strong> for life (assumes ${yearsExpected}-yr expectancy)`;
    } else {
        payoutDescription = `<strong>$${fmt(netAvailableYear1, 0)}</strong> credit line (grows at expected rate)`;
    }

    // Balance growth — assume max-cash-taken-out, balance grows at rate + 0.5% annual MIP.
    const balanceGrowth = rate + 0.005;
    let balance = mortgage + totalCosts + (payout === 'lump' ? netAvailableYear1 : 0);
    const rows = [];
    let homeValue = value;
    for (const yr of [1, 5, 10, 15, 20, 25]) {
        const futureBalance = balance * Math.pow(1 + balanceGrowth, yr);
        const futureHome = homeValue * Math.pow(1 + appr, yr);
        const equityRemaining = Math.max(0, futureHome - futureBalance);
        rows.push({
            year: yr,
            futureBalance,
            futureHome,
            equityRemaining,
            underwater: futureBalance > futureHome,
        });
    }

    result.innerHTML = `
        <div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
            <div class="card"><div class="label">Max claim amount</div><div class="value">$${fmt(maxClaim, 0)}</div><div class="muted small">min(home $${fmt(value, 0)}, FHA cap)</div></div>
            <div class="card"><div class="label">Principal Limit Factor (PLF)</div><div class="value">${fmt(plf * 100, 1)}%</div><div class="muted small">Approx — age ${age}, rate ${fmt(rate * 100, 2)}%</div></div>
            <div class="card"><div class="label">Principal limit</div><div class="value">$${fmt(principalLimit, 0)}</div></div>
            <div class="card"><div class="label">Costs (MIP + origination + closing)</div><div class="value neg">-$${fmt(totalCosts, 0)}</div></div>
            <div class="card"><div class="label">Existing mortgage payoff</div><div class="value neg">-$${fmt(mortgage, 0)}</div></div>
            <div class="card"><div class="label">Net available</div><div class="value pos">$${fmt(netAvailableYear1, 0)}</div><div class="muted small">${payoutDescription}</div></div>
        </div>
        <h3 class="section-title">Balance vs home value over time</h3>
        <p class="muted small">Assumes maximum cash drawn upfront, balance compounds at ${fmt(balanceGrowth * 100, 2)}%/yr (rate + 0.5% ongoing MIP), home appreciates ${fmt(appr * 100, 1)}%/yr.</p>
        <table class="trades" data-table-key="rm-rows">
            <thead><tr>
                <th>Year</th>
                <th>Loan balance</th>
                <th>Home value</th>
                <th>Equity remaining</th>
                <th>Status</th>
            </tr></thead>
            <tbody>${rows.map(r => `<tr>
                <td><strong>${r.year}</strong></td>
                <td class="neg">$${fmt(r.futureBalance, 0)}</td>
                <td>$${fmt(r.futureHome, 0)}</td>
                <td class="${r.equityRemaining > 0 ? 'pos' : 'neg'}">$${fmt(r.equityRemaining, 0)}</td>
                <td class="${r.underwater ? 'neg' : 'muted'}">${r.underwater ? 'Underwater (non-recourse → heirs walk away)' : 'Equity intact'}</td>
            </tr>`).join('')}</tbody>
        </table>
    `;
}

function fmt(n, d) {
    if (n == null || !Number.isFinite(Number(n))) return '—';
    return Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}
