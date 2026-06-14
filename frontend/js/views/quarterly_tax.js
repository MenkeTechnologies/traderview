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

async function renderOutput() {
    const el = document.getElementById('qtax-output');
    if (!el) return;
    const proj = projectAnnual();
    let r;
    try {
        r = await api.calcQuarterlyTax({
            ytd_trading_pnl_usd: state.ytdTradingPnl,
            total_income_usd: proj.totalIncome,
            prior_year_tax_usd: state.priorYearTax,
            filing_status: state.filingStatus,
            quarterly_paid_usd: state.quarterlyPaid,
        });
    } catch (e) {
        el.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
        return;
    }
    const seTax = r.se_tax_usd;
    const taxableIncome = r.taxable_income_usd;
    const fedIncomeTax = r.fed_income_tax_usd;
    const projectedTotal = r.projected_total_usd;
    const safeHarborFloor = r.safe_harbor_floor_usd;
    const perQuarter = r.per_quarter_usd;
    const effectiveRate = r.effective_rate_pct;
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
                    <div class="value">${effectiveRate.toFixed(1)}%</div>
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
                    const qs = r.quarters[q - 1];
                    const cls = qs.status_key === 'ok' ? 'pos' : qs.status_key === 'partial' ? '' : 'neg';
                    const status = qs.status_key === 'ok' ? t('view.qtax.status.ok')
                        : qs.status_key === 'partial' ? t('view.qtax.status.partial', { short: qs.short_usd.toFixed(0) })
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
