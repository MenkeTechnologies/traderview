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

import { api } from '../api.js';
import { esc } from '../util.js';
import { applyUiI18n, t } from '../i18n.js';

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
                    <span class="muted small" data-i18n="view.reverse_mortgage.field.age">Youngest borrower age</span>
                    <input type="number" id="rm-age" step="1" min="62" max="100" value="70" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.reverse_mortgage.field.value">Home value $</span>
                    <input type="number" id="rm-value" step="10000" min="0" value="650000" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.reverse_mortgage.field.mortgage">Existing mortgage $</span>
                    <input type="number" id="rm-mortgage" step="1000" min="0" value="120000" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.reverse_mortgage.field.rate">Expected rate %</span>
                    <input type="number" id="rm-rate" step="0.1" min="3" max="12" value="7.5" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.reverse_mortgage.field.appr">Home appreciation %/yr</span>
                    <input type="number" id="rm-appr" step="0.1" min="-5" max="10" value="3.0" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.reverse_mortgage.field.payout">Payout type</span>
                    <select id="rm-payout" style="width:100%">
                        <option value="lump" selected data-i18n="view.reverse_mortgage.opt.lump">Lump sum</option>
                        <option value="tenure" data-i18n="view.reverse_mortgage.opt.tenure">Tenure (monthly for life)</option>
                        <option value="line" data-i18n="view.reverse_mortgage.opt.line">Line of credit</option>
                    </select>
                </label>
            </div>
            <button class="btn btn-sm primary" id="rm-run" data-i18n="view.reverse_mortgage.btn.run">⚡ Compute</button>
            <div id="rm-result" style="margin-top:12px"></div>
        </div>
    `;
    applyUiI18n(mount);
    mount.querySelectorAll('#rm-age, #rm-value, #rm-mortgage, #rm-rate, #rm-appr, #rm-payout').forEach(el => {
        el.addEventListener('input', () => compute(mount));
    });
    mount.querySelector('#rm-run').addEventListener('click', () => compute(mount));
    compute(mount);
}

async function compute(mount) {
    const result = mount.querySelector('#rm-result');
    const body = {
        age: Math.max(62, parseInt(mount.querySelector('#rm-age').value, 10) || 62),
        home_value_usd: parseFloat(mount.querySelector('#rm-value').value) || 0,
        existing_mortgage_usd: parseFloat(mount.querySelector('#rm-mortgage').value) || 0,
        expected_rate_pct: parseFloat(mount.querySelector('#rm-rate').value) || 0,
        appreciation_pct: parseFloat(mount.querySelector('#rm-appr').value) || 0,
        payout: mount.querySelector('#rm-payout').value,
    };
    try {
        const r = await api.calcReverseMortgage(body);
        renderResult(result, r, body);
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}

function renderResult(result, r, body) {
    const subMaxClaim = t('view.reverse_mortgage.sub.max_claim', { home: '$' + fmt(body.home_value_usd, 0) });
    const subPlf = t('view.reverse_mortgage.sub.plf', { age: String(body.age), rate: fmt(body.expected_rate_pct, 2) + '%' });
    let payoutDescription;
    if (r.payout === 'tenure') {
        payoutDescription = t('view.reverse_mortgage.payout.tenure', {
            amount: '$' + fmt(r.monthly_payment_usd, 0), years: String(r.years_expected),
        });
    } else if (r.payout === 'line') {
        payoutDescription = t('view.reverse_mortgage.payout.line', { amount: '$' + fmt(r.net_available_usd, 0) });
    } else {
        payoutDescription = t('view.reverse_mortgage.payout.lump', { amount: '$' + fmt(r.net_available_usd, 0) });
    }
    const note = t('view.reverse_mortgage.note', {
        bg: fmt(r.balance_growth_pct, 2) + '%', appr: fmt(body.appreciation_pct, 1) + '%',
    });
    const statusIntact = t('view.reverse_mortgage.status.intact');
    const statusUnderwater = t('view.reverse_mortgage.status.underwater');
    const rows = r.rows.map((row) => `<tr>
                <td><strong>${row.year}</strong></td>
                <td class="neg">$${fmt(row.loan_balance_usd, 0)}</td>
                <td>$${fmt(row.home_value_usd, 0)}</td>
                <td class="${row.equity_remaining_usd > 0 ? 'pos' : 'neg'}">$${fmt(row.equity_remaining_usd, 0)}</td>
                <td class="${row.underwater ? 'neg' : 'muted'}">${esc(row.underwater ? statusUnderwater : statusIntact)}</td>
            </tr>`).join('');
    result.innerHTML = `
        <div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
            <div class="card"><div class="label" data-i18n="view.reverse_mortgage.card.max_claim">Max claim amount</div><div class="value">$${fmt(r.max_claim_usd, 0)}</div><div class="muted small">${esc(subMaxClaim)}</div></div>
            <div class="card"><div class="label" data-i18n="view.reverse_mortgage.card.plf">Principal Limit Factor (PLF)</div><div class="value">${fmt(r.plf_pct, 1)}%</div><div class="muted small">${esc(subPlf)}</div></div>
            <div class="card"><div class="label" data-i18n="view.reverse_mortgage.card.principal">Principal limit</div><div class="value">$${fmt(r.principal_limit_usd, 0)}</div></div>
            <div class="card"><div class="label" data-i18n="view.reverse_mortgage.card.costs">Costs (MIP + origination + closing)</div><div class="value neg">-$${fmt(r.total_costs_usd, 0)}</div></div>
            <div class="card"><div class="label" data-i18n="view.reverse_mortgage.card.payoff">Existing mortgage payoff</div><div class="value neg">-$${fmt(body.existing_mortgage_usd, 0)}</div></div>
            <div class="card"><div class="label" data-i18n="view.reverse_mortgage.card.net">Net available</div><div class="value pos">$${fmt(r.net_available_usd, 0)}</div><div class="muted small">${esc(payoutDescription)}</div></div>
        </div>
        <h3 class="section-title" data-i18n="view.reverse_mortgage.h3.projection">Balance vs home value over time</h3>
        <p class="muted small">${esc(note)}</p>
        <table class="trades" data-table-key="rm-rows">
            <thead><tr>
                <th data-i18n="view.reverse_mortgage.th.year">Year</th>
                <th data-i18n="view.reverse_mortgage.th.balance">Loan balance</th>
                <th data-i18n="view.reverse_mortgage.th.home">Home value</th>
                <th data-i18n="view.reverse_mortgage.th.equity">Equity remaining</th>
                <th data-i18n="view.reverse_mortgage.th.status">Status</th>
            </tr></thead>
            <tbody>${rows}</tbody>
        </table>
    `;
    applyUiI18n(result);
}

function fmt(n, d) {
    if (n == null || !Number.isFinite(Number(n))) return '—';
    return Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}
