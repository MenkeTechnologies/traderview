// Promissory note generator — amortizes a fixed loan and assembles the note's
// clauses, via /calc/promissory-note. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderPromissoryNote(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.pnote.h1.title">// PROMISSORY NOTE</span></h1>
        <p class="muted small" data-i18n="view.pnote.hint.intro">
            A borrower's written promise to repay a fixed loan — for owner financing, intra-family
            loans, business notes, or seller carry-back. It amortizes the loan (level monthly payment,
            total interest, maturity date) and assembles the operative clauses: promise to pay, interest,
            repayment, prepayment, default/acceleration, and governing law. Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.pnote.h2.inputs">Note terms</h2>
            <form id="pnote-form" class="inline-form">
                <label><span data-i18n="view.pnote.label.state">Governing state</span>
                    <input type="text" name="governing_state" value="Texas" required></label>
                <label><span data-i18n="view.pnote.label.lender">Lender name</span>
                    <input type="text" name="lender_name" value=""></label>
                <label><span data-i18n="view.pnote.label.borrower">Borrower name</span>
                    <input type="text" name="borrower_name" value=""></label>
                <label><span data-i18n="view.pnote.label.principal">Principal ($)</span>
                    <input type="number" step="0.01" min="0" name="principal_usd" value="10000" required></label>
                <label><span data-i18n="view.pnote.label.rate">Annual rate (%)</span>
                    <input type="number" step="0.001" min="0" name="annual_rate_pct" value="6" required></label>
                <label><span data-i18n="view.pnote.label.term">Term (months)</span>
                    <input type="number" step="1" min="1" name="term_months" value="12" required></label>
                <label><span data-i18n="view.pnote.label.start">Note date</span>
                    <input type="date" name="start_date" value="2026-01-01" required></label>
                <label><span data-i18n="view.pnote.label.late_fee">Late fee per installment ($)</span>
                    <input type="number" step="0.01" min="0" name="late_fee_usd" value="0"></label>
                <label><span data-i18n="view.pnote.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.pnote.ph.statute'))}"></label>
            </form>
        </div>
        <div id="pnote-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#pnote-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            lender_name: (fd.get('lender_name') || '').trim(),
            borrower_name: (fd.get('borrower_name') || '').trim(),
            principal_usd: Number(fd.get('principal_usd')) || 0,
            annual_rate_pct: Number(fd.get('annual_rate_pct')) || 0,
            term_months: Math.round(Number(fd.get('term_months')) || 0),
            start_date: fd.get('start_date'),
            late_fee_usd: Number(fd.get('late_fee_usd')) || 0,
            governing_state: (fd.get('governing_state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcPromissoryNote(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.pnote.toast.error'), { level: 'error' });
        }
    };
    const live = debounce(generate, 250);
    form.addEventListener('input', () => { live(); });
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function docToText(doc) {
    const lines = [doc.title.toUpperCase()];
    if (doc.statutory_citation) lines.push(doc.statutory_citation);
    lines.push('');
    for (const c of doc.clauses) lines.push(c.heading, c.body, '');
    return lines.join('\n');
}

function renderResult(mount, doc) {
    LAST_DOC = doc;
    const el = mount.querySelector('#pnote-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.pnote.card.payment">Monthly payment</div>
                    <div class="value">${money(doc.monthly_payment_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.pnote.card.interest">Total interest</div>
                    <div class="value">${money(doc.total_interest_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.pnote.card.total">Total of payments</div>
                    <div class="value">${money(doc.total_of_payments_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.pnote.card.maturity">Maturity</div>
                    <div class="value">${esc(doc.maturity_date || '—')}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="pnote-copy" type="button" data-i18n="view.pnote.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="pnote-download" type="button" data-i18n="view.pnote.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#pnote-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.pnote.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.pnote.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#pnote-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'promissory-note.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
