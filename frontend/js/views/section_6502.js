// IRC § 6502 — Collection Statute Expiration Date (CSED).
// IRS must collect within 10 years of assessment. After CSED: debt legally uncollectible.
// SUSPENSIONS that extend CSED: OIC pending, CDP hearing, bankruptcy, IA pending, Innocent Spouse, military, etc.
// Plus 30 days after each suspension event. Form 4340 to verify dates.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-csed-v1';
const TEN_YEAR_DAYS = 10 * 365;

function load() { try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); } catch { return []; } }
function save(rows) { try { localStorage.setItem(LS_KEY, JSON.stringify(rows)); } catch { /* ignore */ } }

let state = {
    debts: load(),
};

export async function renderSection6502(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6502.h1.title">// § 6502 CSED 10-YR COLLECTION SOL</span></h1>
        <p class="muted small" data-i18n="view.s6502.hint.intro">
            IRS must collect within <strong>10 years of assessment</strong>. After CSED:
            debt legally UNCOLLECTIBLE (lien auto-releases). <strong>Suspensions extend CSED:</strong>
            OIC pending, CDP hearing, bankruptcy (+ 6 mo), military combat, IA pending, Innocent
            Spouse application (+ 60 days each). <strong>Verify dates via Form 4340</strong> (account
            transcript). Misjudgment by IRS happens — fight for the original CSED.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6502.h2.add">Log tax debt</h2>
            <form id="s6502-form" class="inline-form">
                <label><span data-i18n="view.s6502.label.tax_year">Tax year</span>
                    <input type="number" step="1" name="tax_year" value="2020" required></label>
                <label><span data-i18n="view.s6502.label.assessment_date">Assessment date</span>
                    <input type="date" name="assessment_date" required></label>
                <label><span data-i18n="view.s6502.label.original_amount">Original assessed amount ($)</span>
                    <input type="number" step="1000" name="original_amount" required></label>
                <label><span data-i18n="view.s6502.label.current_balance">Current balance ($)</span>
                    <input type="number" step="100" name="current_balance"></label>
                <label><span data-i18n="view.s6502.label.suspension_days">Suspension days (cumulative)</span>
                    <input type="number" step="1" name="suspension_days" value="0"></label>
                <button class="primary" type="submit" data-i18n="view.s6502.btn.add">Add</button>
            </form>
        </div>
        <div id="s6502-summary"></div>
        <div id="s6502-table" class="chart-panel"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6502.h2.tolling">CSED tolling events</h2>
            <ul class="muted small">
                <li data-i18n="view.s6502.toll.oic">§ 7122 OIC pending: pendency + 30 days after rejection/withdrawal</li>
                <li data-i18n="view.s6502.toll.cdp">§ 6320/§ 6330 CDP hearing: from CDP request through final determination + appeals</li>
                <li data-i18n="view.s6502.toll.ia">Installment agreement pending: from submission + 30 days after termination</li>
                <li data-i18n="view.s6502.toll.bankruptcy">Bankruptcy stay + 6 months after dismissal/discharge</li>
                <li data-i18n="view.s6502.toll.combat">§ 7508 combat zone deferral + 180 days</li>
                <li data-i18n="view.s6502.toll.innocent_spouse">§ 6015 Innocent Spouse election + 60 days after IRS final determination</li>
                <li data-i18n="view.s6502.toll.tao">Taxpayer Assistance Order (TAO) pending</li>
                <li data-i18n="view.s6502.toll.abroad">Continuous absence abroad (Form 9000 waiver)</li>
                <li data-i18n="view.s6502.toll.signed_extension">§ 6502(a)(2) waiver extension signed by taxpayer (RARE; avoid)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6502.h2.csed_strategies">CSED strategies as deadline nears</h2>
            <ul class="muted small">
                <li data-i18n="view.s6502.strat.run_out_clock">Run out the clock: hold tight + comply with current filing requirements</li>
                <li data-i18n="view.s6502.strat.cnc">Currently Not Collectible (CNC) status preserves CSED running</li>
                <li data-i18n="view.s6502.strat.partial_pay">Partial-pay IA: CSED still runs, debt forgiven at CSED</li>
                <li data-i18n="view.s6502.strat.no_oic">DON'T file OIC just before CSED — extends collection time</li>
                <li data-i18n="view.s6502.strat.no_ia">DON'T enter IA in final year — extends CSED</li>
                <li data-i18n="view.s6502.strat.review_transcript">Review Form 4340 annually — IRS sometimes miscalculates CSED</li>
                <li data-i18n="view.s6502.strat.no_borrow">Don't borrow money to pay near-CSED debt</li>
                <li data-i18n="view.s6502.strat.fpa">File-then-pay: Get-out-of-jail-free if pay before CSED but it just expired</li>
            </ul>
        </div>
    `;
    document.getElementById('s6502-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.debts.push({
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            tax_year: Number(fd.get('tax_year')),
            assessment_date: fd.get('assessment_date'),
            original_amount: Number(fd.get('original_amount')) || 0,
            current_balance: Number(fd.get('current_balance')) || Number(fd.get('original_amount')) || 0,
            suspension_days: Number(fd.get('suspension_days')) || 0,
        });
        save(state.debts);
        e.target.reset();
        showToast(t('view.s6502.toast.added'), { level: 'success' });
        render();
    });
    render();
}

function csedDate(debt) {
    const assessmentDate = new Date(debt.assessment_date);
    const csed = new Date(assessmentDate);
    csed.setDate(csed.getDate() + TEN_YEAR_DAYS + debt.suspension_days);
    return csed;
}

function daysUntilCsed(debt) {
    const csed = csedDate(debt);
    return Math.floor((csed - new Date()) / (1000 * 60 * 60 * 24));
}

function render() {
    renderSummary();
    renderTable();
}

function renderSummary() {
    const el = document.getElementById('s6502-summary');
    if (!el) return;
    const totalBalance = state.debts.reduce((s, d) => s + d.current_balance, 0);
    const nearestCsed = state.debts.length > 0
        ? Math.min(...state.debts.map(daysUntilCsed))
        : null;
    const expiredCount = state.debts.filter(d => daysUntilCsed(d) <= 0).length;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6502.h2.summary">CSED summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s6502.card.count">Debts logged</div>
                    <div class="value">${state.debts.length}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6502.card.balance">Total balance</div>
                    <div class="value">$${totalBalance.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${nearestCsed !== null && nearestCsed > 365 ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s6502.card.nearest">Nearest CSED (days)</div>
                    <div class="value">${nearestCsed === null ? '—' : nearestCsed}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s6502.card.expired">Expired CSEDs</div>
                    <div class="value">${expiredCount}</div>
                </div>
            </div>
        </div>
    `;
}

function renderTable() {
    const el = document.getElementById('s6502-table');
    if (!el) return;
    if (!state.debts.length) {
        el.innerHTML = `<h2 data-i18n="view.s6502.h2.debts">Debts</h2>
            <p class="muted" data-i18n="view.s6502.empty">No debts logged.</p>`;
        return;
    }
    const sorted = [...state.debts].sort((a, b) => daysUntilCsed(a) - daysUntilCsed(b));
    el.innerHTML = `
        <h2 data-i18n="view.s6502.h2.debts">Debts</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.s6502.th.year">Year</th>
                <th data-i18n="view.s6502.th.assessment">Assessment</th>
                <th data-i18n="view.s6502.th.balance">Balance</th>
                <th data-i18n="view.s6502.th.suspension">Suspension days</th>
                <th data-i18n="view.s6502.th.csed">CSED date</th>
                <th data-i18n="view.s6502.th.days_left">Days left</th>
                <th data-i18n="view.s6502.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(d => {
                const csed = csedDate(d);
                const daysLeft = daysUntilCsed(d);
                return `<tr>
                    <td>${d.tax_year}</td>
                    <td class="muted">${esc(d.assessment_date)}</td>
                    <td>$${d.current_balance.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="muted">${d.suspension_days}</td>
                    <td class="muted">${csed.toISOString().slice(0, 10)}</td>
                    <td class="${daysLeft <= 0 ? 'pos' : (daysLeft < 365 ? 'neg' : '')}">${daysLeft}</td>
                    <td><button class="link neg" data-del="${esc(d.id)}" data-i18n="view.s6502.btn.delete">delete</button></td>
                </tr>`;
            }).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.debts = state.debts.filter(d => d.id !== btn.dataset.del);
            save(state.debts);
            render();
        });
    });
}
