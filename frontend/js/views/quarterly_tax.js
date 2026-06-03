// Quarterly Tax Estimator — projects estimated tax payments + safe-harbor.
//
// Pulls YTD realized P&L from /trades/stats, computes SE + federal income tax
// per the 1040-ES schedule, applies the safe-harbor rule (110% of prior-year
// liability for AGI > $150k; 100% otherwise), and shows the dollar amount
// to send in for the next quarterly deadline (April 15 / June 15 / Sept 15 /
// Jan 15 of following year).
//
// Pure client-side: no new backend routes needed.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const SS_BASE_2024 = 168_600;  // Social-security wage base
const SS_RATE = 0.124;
const MEDICARE_RATE = 0.029;
const MEDICARE_ADD = 0.009; // additional Medicare > $200k single / $250k MFJ
const SE_DEDUCTION = 0.9235;

const FED_BRACKETS_2024_SINGLE = [
    [11_600,  0.10],
    [47_150,  0.12],
    [100_525, 0.22],
    [191_950, 0.24],
    [243_725, 0.32],
    [609_350, 0.35],
    [Infinity, 0.37],
];

let state = {
    ytdTradingPnl: 0,
    otherIncome: 0,
    priorYearTax: 0,
    filingStatus: 'single',
    quarterlyPaid: [0, 0, 0, 0],  // Q1-Q4
};

export async function renderQuarterlyTax(mount, _appState) {
    const tok = currentViewToken();
    // Best-effort: prefill YTD P&L from /reports/overview if available.
    try {
        const ov = await api.reportsOverview?.();
        if (ov?.realized_pnl_ytd != null) state.ytdTradingPnl = Number(ov.realized_pnl_ytd);
    } catch { /* not wired yet — manual entry only */ }
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.qtax.h1.title">// QUARTERLY TAX ESTIMATOR</span></h1>
        <p class="muted small" data-i18n="view.qtax.hint.intro">
            Projects estimated tax payments + safe-harbor floor for trader-classified income.
            Defaults to 2024 brackets; SE tax assumes Schedule C trader (Section 475(f) marks
            don't affect SE owing — flag traders pay full SE on income).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.qtax.h2.inputs">Inputs</h2>
            <form id="qtax-form" class="inline-form">
                <label><span data-i18n="view.qtax.label.ytd_pnl">YTD trading P&L ($)</span>
                    <input type="number" step="0.01" name="ytd_pnl" value="${state.ytdTradingPnl}"></label>
                <label><span data-i18n="view.qtax.label.other_income">Other income ($)</span>
                    <input type="number" step="0.01" name="other_income" value="${state.otherIncome}"></label>
                <label><span data-i18n="view.qtax.label.prior_tax">Prior year tax ($)</span>
                    <input type="number" step="0.01" name="prior_tax" value="${state.priorYearTax}"></label>
                <label><span data-i18n="view.qtax.label.filing">Filing status</span>
                    <select name="filing_status">
                        <option value="single" ${state.filingStatus === 'single' ? 'selected' : ''}>Single</option>
                        <option value="mfj"    ${state.filingStatus === 'mfj' ? 'selected' : ''}>Married filing jointly</option>
                        <option value="hoh"    ${state.filingStatus === 'hoh' ? 'selected' : ''}>Head of household</option>
                    </select>
                </label>
                <label><span data-i18n="view.qtax.label.q1_paid">Q1 paid ($)</span>
                    <input type="number" step="0.01" name="q1_paid" value="${state.quarterlyPaid[0]}"></label>
                <label><span data-i18n="view.qtax.label.q2_paid">Q2 paid ($)</span>
                    <input type="number" step="0.01" name="q2_paid" value="${state.quarterlyPaid[1]}"></label>
                <label><span data-i18n="view.qtax.label.q3_paid">Q3 paid ($)</span>
                    <input type="number" step="0.01" name="q3_paid" value="${state.quarterlyPaid[2]}"></label>
                <label><span data-i18n="view.qtax.label.q4_paid">Q4 paid ($)</span>
                    <input type="number" step="0.01" name="q4_paid" value="${state.quarterlyPaid[3]}"></label>
                <button class="primary" type="submit" data-i18n="view.qtax.btn.recompute">Recompute</button>
            </form>
        </div>
        <div id="qtax-output"></div>
    `;
    document.getElementById('qtax-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.ytdTradingPnl = Number(fd.get('ytd_pnl')) || 0;
        state.otherIncome = Number(fd.get('other_income')) || 0;
        state.priorYearTax = Number(fd.get('prior_tax')) || 0;
        state.filingStatus = fd.get('filing_status') || 'single';
        state.quarterlyPaid = [
            Number(fd.get('q1_paid')) || 0,
            Number(fd.get('q2_paid')) || 0,
            Number(fd.get('q3_paid')) || 0,
            Number(fd.get('q4_paid')) || 0,
        ];
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('qtax-output');
    if (!el) return;
    const proj = projectAnnual();
    const seTax = computeSelfEmploymentTax(state.ytdTradingPnl);
    const halfSe = seTax / 2;
    const taxableIncome = Math.max(0, proj.totalIncome - halfSe - standardDeduction(state.filingStatus));
    const fedIncomeTax = computeFedTax(taxableIncome);
    const projectedTotal = fedIncomeTax + seTax;
    const safeHarborBase = Math.max(
        proj.totalIncome > 150_000 ? state.priorYearTax * 1.10 : state.priorYearTax,
        0
    );
    const safeHarborFloor = Math.min(projectedTotal * 0.90, safeHarborBase);
    const perQuarter = projectedTotal / 4;
    const totalPaid = state.quarterlyPaid.reduce((a, b) => a + b, 0);
    const remaining = Math.max(0, projectedTotal - totalPaid);
    const nextQ = nextQuarterInfo();
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.qtax.h2.summary">Annual summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.qtax.card.projected_total">Projected total tax</div>
                    <div class="value">$${projectedTotal.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.qtax.card.fed_income">Federal income tax</div>
                    <div class="value">$${fedIncomeTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.qtax.card.se_tax">SE tax</div>
                    <div class="value">$${seTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.qtax.card.taxable_income">Taxable income</div>
                    <div class="value">$${taxableIncome.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.qtax.card.effective_rate">Effective rate</div>
                    <div class="value">${proj.totalIncome > 0 ? ((projectedTotal / proj.totalIncome) * 100).toFixed(1) : '0.0'}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.qtax.card.safe_harbor">Safe-harbor floor</div>
                    <div class="value">$${safeHarborFloor.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.qtax.h2.quarterly">Quarterly schedule</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.qtax.th.quarter">Quarter</th>
                    <th data-i18n="view.qtax.th.due_date">Due date</th>
                    <th data-i18n="view.qtax.th.target">Target payment</th>
                    <th data-i18n="view.qtax.th.paid">Paid</th>
                    <th data-i18n="view.qtax.th.status">Status</th>
                </tr></thead>
                <tbody>${[1, 2, 3, 4].map(q => {
                    const paid = state.quarterlyPaid[q - 1];
                    const cls = paid >= perQuarter ? 'pos' : paid > 0 ? '' : 'neg';
                    const status = paid >= perQuarter ? t('view.qtax.status.ok')
                        : paid > 0 ? t('view.qtax.status.partial', { short: (perQuarter - paid).toFixed(0) })
                        : t('view.qtax.status.unpaid');
                    return `<tr>
                        <td>Q${q}</td>
                        <td>${esc(quarterDueDate(q))}</td>
                        <td>$${perQuarter.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                        <td>$${paid.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                        <td class="${cls}">${esc(status)}</td>
                    </tr>`;
                }).join('')}</tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.qtax.h2.next_action">Next action</h2>
            <p>
                <strong>${esc(t('view.qtax.next.quarter_label', { q: nextQ.quarter, date: nextQ.dueDate }))}</strong>
            </p>
            <p>
                ${esc(t('view.qtax.next.send', { amount: Math.max(0, perQuarter - state.quarterlyPaid[nextQ.quarter - 1]).toLocaleString(undefined, { maximumFractionDigits: 0 }) }))}
                · ${esc(t('view.qtax.next.url'))}: <a href="https://www.irs.gov/payments" target="_blank">irs.gov/payments</a>
            </p>
            <p class="muted small" data-i18n="view.qtax.next.disclaimer">
                Estimates only — not tax advice. Bracket data is 2024; state tax not included.
                Mark-to-market traders should consult a CPA on §475(f) ordinary-income treatment.
            </p>
        </div>
    `;
}

function projectAnnual() {
    // Naive linear projection — annualize YTD as today/365.
    const now = new Date();
    const start = new Date(now.getFullYear(), 0, 1);
    const elapsedDays = Math.max(1, Math.floor((now - start) / 86_400_000));
    const annualizedTrading = state.ytdTradingPnl * (365 / elapsedDays);
    const totalIncome = annualizedTrading + state.otherIncome;
    return { annualizedTrading, totalIncome, elapsedDays };
}

function computeSelfEmploymentTax(net) {
    if (net <= 0) return 0;
    const seBase = net * SE_DEDUCTION;
    const ssTaxable = Math.min(seBase, SS_BASE_2024);
    return ssTaxable * SS_RATE + seBase * MEDICARE_RATE;
}

function computeFedTax(taxable) {
    let owe = 0;
    let lastCap = 0;
    for (const [cap, rate] of FED_BRACKETS_2024_SINGLE) {
        const slice = Math.max(0, Math.min(taxable, cap) - lastCap);
        owe += slice * rate;
        if (taxable <= cap) break;
        lastCap = cap;
    }
    return owe;
}

function standardDeduction(status) {
    return ({ single: 14_600, mfj: 29_200, hoh: 21_900 })[status] || 14_600;
}

function quarterDueDate(q) {
    const y = new Date().getFullYear();
    return ({
        1: `${y}-04-15`,
        2: `${y}-06-15`,
        3: `${y}-09-15`,
        4: `${y + 1}-01-15`,
    })[q];
}

function nextQuarterInfo() {
    const now = new Date();
    const y = now.getFullYear();
    const m = now.getMonth() + 1;
    const quarter = m <= 3 ? 1 : m <= 5 ? 2 : m <= 8 ? 3 : m <= 12 ? 4 : 4;
    return { quarter, dueDate: quarterDueDate(quarter) };
}
