// Bank reconciliation generator — adjusted bank vs adjusted book balance and the
// reconciling difference, via /calc/bank-reconciliation.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderBankReconciliation(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.bankrec.h1.title">// BANK RECONCILIATION</span></h1>
        <p class="muted small" data-i18n="view.bankrec.hint.intro">
            Ties the book cash balance to the bank statement balance. The bank side adds deposits in transit
            and subtracts outstanding checks; the book side adds interest earned and subtracts service charges
            and returned (NSF) items. When the two adjusted balances agree, the account is reconciled. Drafting
            aid, not accounting advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.bankrec.h2.inputs">Reconciliation inputs</h2>
            <form id="bankrec-form" class="inline-form">
                <label><span data-i18n="view.bankrec.label.company">Company</span>
                    <input type="text" name="company_name" value="Acme Co" required></label>
                <label><span data-i18n="view.bankrec.label.account">Account</span>
                    <input type="text" name="account_label" value="Operating checking" required></label>
                <label><span data-i18n="view.bankrec.label.stmtdate">Statement date</span>
                    <input type="date" name="statement_date" value="2026-06-30" required></label>
                <label><span data-i18n="view.bankrec.label.bankbal">Bank statement balance ($)</span>
                    <input type="number" step="10" name="bank_statement_balance_usd" value="10000" required></label>
                <label><span data-i18n="view.bankrec.label.dit">Deposits in transit ($)</span>
                    <input type="number" step="10" name="deposits_in_transit_usd" value="2000"></label>
                <label><span data-i18n="view.bankrec.label.oc">Outstanding checks ($)</span>
                    <input type="number" step="10" name="outstanding_checks_usd" value="1500"></label>
                <label><span data-i18n="view.bankrec.label.bookbal">Book balance ($)</span>
                    <input type="number" step="10" name="book_balance_usd" value="10650" required></label>
                <label><span data-i18n="view.bankrec.label.interest">Interest earned ($)</span>
                    <input type="number" step="1" name="interest_earned_usd" value="50"></label>
                <label><span data-i18n="view.bankrec.label.fees">Service charges ($)</span>
                    <input type="number" step="1" name="service_charges_usd" value="100"></label>
                <label><span data-i18n="view.bankrec.label.nsf">NSF / returned items ($)</span>
                    <input type="number" step="1" name="nsf_returns_usd" value="100"></label>
                <label><span data-i18n="view.bankrec.label.date">Prepared date</span>
                    <input type="date" name="date" value="2026-07-01" required></label>
            </form>
        </div>
        <div id="bankrec-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#bankrec-form');
    const num = (k) => Number(form.querySelector(`[name="${k}"]`).value) || 0;
    const generate = async () => {
        const body = {
            company_name: (form.querySelector('[name="company_name"]').value || '').trim(),
            account_label: (form.querySelector('[name="account_label"]').value || '').trim(),
            statement_date: form.querySelector('[name="statement_date"]').value,
            bank_statement_balance_usd: num('bank_statement_balance_usd'),
            deposits_in_transit_usd: num('deposits_in_transit_usd'),
            outstanding_checks_usd: num('outstanding_checks_usd'),
            book_balance_usd: num('book_balance_usd'),
            interest_earned_usd: num('interest_earned_usd'),
            service_charges_usd: num('service_charges_usd'),
            nsf_returns_usd: num('nsf_returns_usd'),
            date: form.querySelector('[name="date"]').value,
        };
        try {
            const doc = await api.calcBankReconciliation(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.bankrec.toast.error'), { level: 'error' });
        }
    };
    const live = debounce(generate, 250);
    form.addEventListener('input', () => { live(); });
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function docToText(doc) {
    const lines = [doc.title.toUpperCase(), ''];
    for (const c of doc.clauses) lines.push(c.heading, c.body, '');
    return lines.join('\n');
}

function renderResult(mount, doc) {
    LAST_DOC = doc;
    const statusKey = doc.reconciled ? 'view.bankrec.reconciled' : 'view.bankrec.unbalanced';
    const statusCls = doc.reconciled ? 'pos' : 'neg';
    const el = mount.querySelector('#bankrec-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card ${statusCls}"><div class="label" data-i18n="view.bankrec.card.status">Status</div>
                    <div class="value" data-i18n="${statusKey}">${doc.reconciled ? 'Reconciled' : 'Out of balance'}</div></div>
                <div class="card"><div class="label" data-i18n="view.bankrec.card.bank">Adjusted bank</div>
                    <div class="value">${money(doc.adjusted_bank_balance_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.bankrec.card.book">Adjusted book</div>
                    <div class="value">${money(doc.adjusted_book_balance_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.bankrec.card.diff">Difference</div>
                    <div class="value">${money(doc.difference_usd)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="bankrec-copy" type="button" data-i18n="view.bankrec.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="bankrec-download" type="button" data-i18n="view.bankrec.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#bankrec-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.bankrec.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.bankrec.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#bankrec-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'bank-reconciliation.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
